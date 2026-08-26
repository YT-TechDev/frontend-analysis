//! Candidate-independent TC-S5 successor validation.
//!
//! TC-S5 is the bounded successor theorem "Selected In-Body `p` Lifecycle
//! with Bounded Implicit Closure and Unmatched-End Synthesis" (Issue #365).
//! This module is validation-only. It consumes the accepted batch tokenizer as
//! lower-layer token/source evidence and deliberately imports no production
//! tree-construction semantics from `driver`, `session`, or `result`.
//!
//! The load-bearing normative authority is the #348 pinned WHATWG HTML source:
//! commit `508a037333d8a1806504303aeb489d931fabbef6`, blob
//! `68dbcb98bbe1001c6ae2531be2368c608fbafddd`.
//!
//! The candidate state theorem is intentionally closed:
//!
//! ```text
//! S = [html, body] ++ B* ++ P?
//! B in {Div, Section}
//! count(P) <= 1
//! P present => P is current
//! ```
//!
//! Under that theorem the close-p implied-end step is proven only as a bounded
//! no-op. This file does not introduce a generic scope engine or generalized
//! implied-end machinery.

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
    CandidateFixture { id: "P1", bytes: b"<body><p>x</p>", length: 14, sha256: "048f590bc619f617486995f66886344f8cdb98aff38878cff4e5a7767268fcc4", required_ranges: &[((0,6), b"<body>"), ((6,9), b"<p>"), ((7,8), b"p"), ((9,10), b"x"), ((10,14), b"</p>"), ((12,13), b"p")] },
    CandidateFixture { id: "P2", bytes: b"<body><P>x</p>", length: 14, sha256: "eb308af0a2406054311fb2b1830d7ad5cc44e9ab5ddba9b3343c1344ee1df718", required_ranges: &[((6,9), b"<P>"), ((7,8), b"P"), ((10,14), b"</p>"), ((12,13), b"p")] },
    CandidateFixture { id: "P3", bytes: b"<body><p>a<p>b</p>", length: 18, sha256: "e8f1e7b038acc12016c661ca43dcf82e01da8f057c8f23d082eeee8f3bcce2aa", required_ranges: &[((6,9), b"<p>"), ((10,13), b"<p>"), ((14,18), b"</p>")] },
    CandidateFixture { id: "P4", bytes: b"<body><p>a<div>b</div>", length: 22, sha256: "36c14fe35133b89cfb8fe568d342538c03c4a51e64525867c16e57e193c7cb7e", required_ranges: &[((6,9), b"<p>"), ((10,15), b"<div>"), ((16,22), b"</div>")] },
    CandidateFixture { id: "P5", bytes: b"<body><p>a<section>b</section>", length: 30, sha256: "6fe08ea502e0cf06438fdf43289251cb064c1e498831cfdc05e30b3d11bacc62", required_ranges: &[((6,9), b"<p>"), ((10,19), b"<section>"), ((20,30), b"</section>")] },
    CandidateFixture { id: "P6", bytes: b"<body><div><p>x</p></div>", length: 25, sha256: "ca658e443401aa80f2c4dd40f3ebcc999867d42746601451163f6395c025eb3c", required_ranges: &[((6,11), b"<div>"), ((11,14), b"<p>"), ((15,19), b"</p>"), ((19,25), b"</div>")] },
    CandidateFixture { id: "P7", bytes: b"<body></p>", length: 10, sha256: "580c495a71f5ede4dbd5f7b1f984b0ace409e5e23de52b3127889f4d0de28dbc", required_ranges: &[((6,10), b"</p>"), ((8,9), b"p")] },
    CandidateFixture { id: "P8", bytes: b"<body><div></p>x</div>", length: 22, sha256: "0f9b4566303bd54768dbde080baedb79f9f999fc2a9a093f43791c1992b0708a", required_ranges: &[((6,11), b"<div>"), ((11,15), b"</p>"), ((15,16), b"x"), ((16,22), b"</div>")] },
    CandidateFixture { id: "P9", bytes: b"<body></p></p>", length: 14, sha256: "3176d4fcc56896d4549ce7280cb06d19b07b1ede07eff658b1bb07d7f12b29fe", required_ranges: &[((6,10), b"</p>"), ((10,14), b"</p>")] },
    CandidateFixture { id: "P10", bytes: b"<body><p>x", length: 10, sha256: "2df8fbfb898e5f5a6076f6e8f8896c254a81f47eec50911ac4d79ebb5f9101bf", required_ranges: &[((6,9), b"<p>"), ((9,10), b"x")] },
    CandidateFixture { id: "P11", bytes: b"<body><div><p>x", length: 15, sha256: "2b5a7cc613c3dc6a8b2809b70decf44e09991a3940c64be5d2d7be9926cc9e54", required_ranges: &[((6,11), b"<div>"), ((11,14), b"<p>"), ((14,15), b"x")] },
    CandidateFixture { id: "P12", bytes: b"<body><div><p></div>", length: 20, sha256: "2124bda9e02459b9a39cf29778617da7f1e4cf6a269342db68339fa9ad816ee8", required_ranges: &[((11,14), b"<p>"), ((14,20), b"</div>")] },
    CandidateFixture { id: "P13", bytes: b"<body><section><p></section>", length: 28, sha256: "4f2cbeba418b58e07b7c0b85e56dedc5ae9d8710716a6feefbc71610f7003c28", required_ranges: &[((15,18), b"<p>"), ((18,28), b"</section>")] },
    CandidateFixture { id: "P14", bytes: b"<body><p id=x>", length: 14, sha256: "39b5c2f3c03d45d89feb783806b61adcc16f5b7db3a5a6afc829504dd68393d9", required_ranges: &[((6,14), b"<p id=x>"), ((7,8), b"p"), ((9,13), b"id=x")] },
    CandidateFixture { id: "P15", bytes: b"<body><p/>", length: 10, sha256: "c333a1b69f4440a1b85756593f68ac802cadb101f2205153b05cb0fda614af45", required_ranges: &[((6,10), b"<p/>"), ((7,8), b"p"), ((8,9), b"/")] },
    CandidateFixture { id: "P16", bytes: b"<body></body><p>", length: 16, sha256: "242bfc6d4578a20bcac9952be92f5b9bbd91491da24b4e67935217a88758781d", required_ranges: &[((6,13), b"</body>"), ((13,16), b"<p>")] },
    CandidateFixture { id: "P17", bytes: b"<body><p></body>", length: 16, sha256: "e7134dedb9e68017c356b35c6b289dd657353da0bc3b6075c5ca00ec3b337f4d", required_ranges: &[((6,9), b"<p>"), ((9,16), b"</body>")] },
    CandidateFixture { id: "P18", bytes: b"<body><div>x</div><section>y</section>", length: 38, sha256: "eacc86ff52247da192bcbe2cb598e1d9585939168cf025aa5149a51ee8b54740", required_ranges: &[((6,11), b"<div>"), ((12,18), b"</div>"), ((18,27), b"<section>"), ((28,38), b"</section>")] },
    CandidateFixture { id: "P19", bytes: b"<body><div><section></div>", length: 26, sha256: "e64ebe85f8adcefd7b35780e32238bb43957d9e81641bb333b9cd64319dc1171", required_ranges: &[((6,11), b"<div>"), ((11,20), b"<section>"), ((20,26), b"</div>")] },
    CandidateFixture { id: "P20", bytes: b"<body><p>Z</p>", length: 14, sha256: "888661223c2799313f9853e65f5b135cc3af31c22274217b88a26103400bc30b", required_ranges: &[((0,6), b"<body>"), ((6,9), b"<p>"), ((7,8), b"p"), ((9,10), b"Z"), ((10,14), b"</p>"), ((12,13), b"p")] },
    CandidateFixture { id: "P21", bytes: b"<body><p></p id=x>", length: 18, sha256: "e2ffd03cdcb289cb0d571c89676fbb21ad672cedd38d94dda3744f4ff9dcaa9a", required_ranges: &[((6,9), b"<p>"), ((9,18), b"</p id=x>"), ((11,12), b"p"), ((13,17), b"id=x")] },
];

const CANDIDATE_IDS: [&str; 21] = [
    "P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8", "P9", "P10", "P11", "P12", "P13", "P14", "P15", "P16", "P17", "P18", "P19", "P20", "P21",
];

fn fixture(id: &str) -> &'static CandidateFixture {
    CANDIDATE_FIXTURES.iter().find(|fixture| fixture.id == id).expect("canonical P fixture")
}

impl CandidateFixture {
    fn source_text(&self) -> &'static str {
        std::str::from_utf8(self.bytes).expect("canonical fixtures are UTF-8")
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
    CandidateEvidence { source_id: SourceId::new(source_id), range }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CandidateNodeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateNamespace { Html }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateElementName { Html, Head, Body, Div, Section, P }

impl CandidateElementName {
    fn is_selected_block(self) -> bool { matches!(self, Self::Div | Self::Section) }
    fn is_p(self) -> bool { self == Self::P }
    fn permitted_at_in_body_eof(self) -> bool { matches!(self, Self::Html | Self::Body | Self::P) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateMode { Initial, BeforeHtml, BeforeHead, InHead, AfterHead, InBody, AfterBody }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateSynthesisCause { ImpliedHtml, ImpliedHead, ImpliedBody, UnmatchedPEnd }

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateOrigin {
    Authored { complete: CandidateEvidence, raw_name: CandidateEvidence },
    Synthesized { cause: CandidateSynthesisCause },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTrigger {
    Authored { index: usize, complete: CandidateEvidence, raw_name: Option<CandidateEvidence> },
    EndOfFile { index: usize },
}

impl CandidateTrigger {
    fn index(&self) -> usize {
        match self { Self::Authored { index, .. } | Self::EndOfFile { index } => *index }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDiagnosticCode {
    MissingDoctype,
    UnmatchedPEndTag,
    UnmatchedSelectedEndTag,
    MisnestedSelectedEndTag,
    OpenSelectedBlockAtEndOfFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateDiagnostic { code: CandidateDiagnosticCode, trigger: CandidateTrigger }

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateRecovery {
    ContinuedInQuirksDocumentMode { trigger: CandidateTrigger },
    IgnoredSelectedEndTag { trigger: CandidateTrigger },
    PoppedBySelectedAncestorEndTag { popped_identity: CandidateNodeId, target_identity: CandidateNodeId, exact_end_trigger: CandidateTrigger },
    StoppedParsingWithOpenSelectedBlock { trigger: CandidateTrigger },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidatePClosureKind { MatchingEnd, StartTriggered, UnmatchedEndSynthesized }

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidatePClosure {
    kind: CandidatePClosureKind,
    target_identity: CandidateNodeId,
    exact_trigger: CandidateTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidatePSynthesis {
    identity: CandidateNodeId,
    exact_unmatched_end_trigger: CandidateTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateSelectedClosure {
    target_identity: CandidateNodeId,
    exact_same_name_end_trigger: CandidateTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateUnsupported {
    PStartTagAttribute,
    PEndTagAttribute,
    PSelfClosingStartTag,
    PTagOutsideInBody,
    SelectedEndWithOpenP,
    BodyCloseWithOpenP,
    BodyCloseWithOpenSelectedBlock,
    SelectedTagOutsideInBody,
    SelectedStartTagAttribute,
    SelectedEndTagAttribute,
    SelectedSelfClosingStartTag,
    ShellTagAttribute,
    SelfClosingShellTag,
    GenericOrdinaryElement,
    WhitespaceSensitiveCharacterData,
    OutsideModelledCandidateCells,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDisposition { Consumed, Ignored, Reprocessed, Stopped, Refused(CandidateUnsupported) }

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateStackEntry { identity: CandidateNodeId, name: CandidateElementName }

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTree {
    Document { id: CandidateNodeId, children: Vec<CandidateTree> },
    Element { id: CandidateNodeId, name: CandidateElementName, namespace: CandidateNamespace, origin: CandidateOrigin, children: Vec<CandidateTree> },
    Text { id: CandidateNodeId, interpreted: String, contributions: Vec<CandidateEvidence> },
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
    open_content: Vec<CandidateStackEntry>,
    committed_prefix_end: usize,
    processed_tokens: usize,
    completion: CandidateCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateTokenRecord {
    index: usize,
    trigger: CandidateTrigger,
    mode_before: CandidateMode,
    mode_after: CandidateMode,
    disposition: CandidateDisposition,
    open_before: Vec<CandidateStackEntry>,
    open_after: Vec<CandidateStackEntry>,
    identity_count_before: usize,
    identity_count_after: usize,
    committed_before: usize,
    committed_after: usize,
    processed_before: usize,
    processed_after: usize,
}

impl CandidateTokenRecord {
    fn refusal(&self) -> Option<CandidateUnsupported> {
        match self.disposition { CandidateDisposition::Refused(capability) => Some(capability), _ => None }
    }
    fn stopped(&self) -> bool { self.disposition == CandidateDisposition::Stopped }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateObservation {
    tree: CandidateTree,
    diagnostics: Vec<CandidateDiagnostic>,
    recovery: Vec<CandidateRecovery>,
    p_closures: Vec<CandidatePClosure>,
    p_syntheses: Vec<CandidatePSynthesis>,
    selected_closures: Vec<CandidateSelectedClosure>,
    identity_count: usize,
    checkpoint: CandidateCheckpoint,
    tokens: Vec<CandidateTokenRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTokenShape<'a> {
    Characters { source: CandidateEvidence, interpreted: &'a str },
    StartTag { name: CandidateElementName, complete: CandidateEvidence, raw_name: CandidateEvidence },
    EndTag { name: CandidateElementName, complete: CandidateEvidence, raw_name: CandidateEvidence },
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
}

fn candidate_element_name(interpreted: &str) -> Option<CandidateElementName> {
    match interpreted {
        "html" => Some(CandidateElementName::Html),
        "head" => Some(CandidateElementName::Head),
        "body" => Some(CandidateElementName::Body),
        "div" => Some(CandidateElementName::Div),
        "section" => Some(CandidateElementName::Section),
        "p" => Some(CandidateElementName::P),
        _ => None,
    }
}

fn candidate_trigger(token: &HtmlToken, index: usize) -> CandidateTrigger {
    match token {
        HtmlToken::Character(character) => CandidateTrigger::Authored { index, complete: evidence(character.source()), raw_name: None },
        HtmlToken::Tag(tag) => CandidateTrigger::Authored { index, complete: evidence(tag.complete()), raw_name: Some(evidence(tag.name().source())) },
        HtmlToken::EndOfFile(_) => CandidateTrigger::EndOfFile { index },
    }
}

fn candidate_shape(token: &HtmlToken) -> Result<CandidateTokenShape<'_>, CandidateUnsupported> {
    match token {
        HtmlToken::Character(character) => Ok(CandidateTokenShape::Characters { source: evidence(character.source()), interpreted: character.interpreted() }),
        HtmlToken::Tag(tag) => {
            let Some(name) = candidate_element_name(tag.name().interpreted()) else { return Err(CandidateUnsupported::GenericOrdinaryElement); };
            if !tag.attributes().is_empty() {
                return Err(match (name, tag.kind()) {
                    (CandidateElementName::P, HtmlTagKind::Start) => CandidateUnsupported::PStartTagAttribute,
                    (CandidateElementName::P, HtmlTagKind::End) => CandidateUnsupported::PEndTagAttribute,
                    (name, HtmlTagKind::Start) if name.is_selected_block() => CandidateUnsupported::SelectedStartTagAttribute,
                    (name, HtmlTagKind::End) if name.is_selected_block() => CandidateUnsupported::SelectedEndTagAttribute,
                    _ => CandidateUnsupported::ShellTagAttribute,
                });
            }
            if tag.self_closing_solidus().is_some() {
                return Err(match (name, tag.kind()) {
                    (CandidateElementName::P, HtmlTagKind::Start) => CandidateUnsupported::PSelfClosingStartTag,
                    (name, HtmlTagKind::Start) if name.is_selected_block() => CandidateUnsupported::SelectedSelfClosingStartTag,
                    _ => CandidateUnsupported::SelfClosingShellTag,
                });
            }
            let complete = evidence(tag.complete());
            let raw_name = evidence(tag.name().source());
            Ok(match tag.kind() {
                HtmlTagKind::Start => CandidateTokenShape::StartTag { name, complete, raw_name },
                HtmlTagKind::End => CandidateTokenShape::EndTag { name, complete, raw_name },
            })
        }
        HtmlToken::EndOfFile(end_of_file) => Ok(CandidateTokenShape::EndOfFile { at: end_of_file.source().range().start() }),
    }
}

fn is_html_whitespace(character: char) -> bool { matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' ') }

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateSelectedEndPlan {
    target_identity: CandidateNodeId,
    target_name: CandidateElementName,
    intervening: Vec<CandidateNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateEffect {
    MissingDoctype,
    InsertShell { name: CandidateElementName, authored: bool },
    CloseHead,
    InsertCharacters,
    StartP { close_current_p: bool },
    StartSelected { name: CandidateElementName, close_current_p: bool },
    ClosePMatching,
    SynthesizePForUnmatchedEnd,
    CloseSelected { plan: CandidateSelectedEndPlan },
    UnmatchedSelected { name: CandidateElementName },
    OpenSelectedAtEof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateStep {
    Consume { effect: Option<CandidateEffect>, next: Option<CandidateMode> },
    Ignore { effect: CandidateEffect },
    Reprocess { effect: Option<CandidateEffect>, next: CandidateMode },
    Stop { effect: Option<CandidateEffect> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateArenaKind {
    Document,
    Element { name: CandidateElementName, origin: CandidateOrigin },
    Text { interpreted: String, contributions: Vec<CandidateEvidence> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateArenaNode { id: CandidateNodeId, children: Vec<CandidateNodeId>, kind: CandidateArenaKind }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateStorageLayout { leading_padding: usize, padding_before_each_node: usize, padding_after_even_identity: usize }

impl CandidateStorageLayout {
    const COMPACT: Self = Self { leading_padding: 0, padding_before_each_node: 0, padding_after_even_identity: 0 };
    const PADDED: Self = Self { leading_padding: 3, padding_before_each_node: 2, padding_after_even_identity: 1 };
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateFingerprint {
    slots: Vec<Option<CandidateArenaNode>>,
    document: CandidateNodeId,
    open_elements: Vec<CandidateNodeId>,
    head: Option<CandidateNodeId>,
    mode: CandidateMode,
    diagnostics: Vec<CandidateDiagnostic>,
    recovery: Vec<CandidateRecovery>,
    p_closures: Vec<CandidatePClosure>,
    p_syntheses: Vec<CandidatePSynthesis>,
    selected_closures: Vec<CandidateSelectedClosure>,
    identity_count: usize,
    committed_prefix_end: usize,
    processed_tokens: usize,
}

struct CandidateSession {
    layout: CandidateStorageLayout,
    slots: Vec<Option<CandidateArenaNode>>,
    document: CandidateNodeId,
    open_elements: Vec<CandidateNodeId>,
    head: Option<CandidateNodeId>,
    mode: CandidateMode,
    diagnostics: Vec<CandidateDiagnostic>,
    recovery: Vec<CandidateRecovery>,
    p_closures: Vec<CandidatePClosure>,
    p_syntheses: Vec<CandidatePSynthesis>,
    selected_closures: Vec<CandidateSelectedClosure>,
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
            recovery: Vec::new(),
            p_closures: Vec::new(),
            p_syntheses: Vec::new(),
            selected_closures: Vec::new(),
            identity_count: 0,
            committed_prefix_end: 0,
            processed_tokens: 0,
        };
        session.document = session.allocate(CandidateArenaKind::Document);
        session
    }

    fn allocate(&mut self, kind: CandidateArenaKind) -> CandidateNodeId {
        if self.slots.is_empty() { for _ in 0..self.layout.leading_padding { self.slots.push(None); } }
        for _ in 0..self.layout.padding_before_each_node { self.slots.push(None); }
        let id = CandidateNodeId(self.identity_count);
        self.identity_count += 1;
        self.slots.push(Some(CandidateArenaNode { id, children: Vec::new(), kind }));
        if id.0.is_multiple_of(2) { for _ in 0..self.layout.padding_after_even_identity { self.slots.push(None); } }
        id
    }

    fn node(&self, id: CandidateNodeId) -> &CandidateArenaNode {
        self.slots.iter().flatten().find(|node| node.id == id).expect("semantic node identity")
    }

    fn node_mut(&mut self, id: CandidateNodeId) -> &mut CandidateArenaNode {
        self.slots.iter_mut().flatten().find(|node| node.id == id).expect("semantic node identity")
    }

    fn element_name(&self, id: CandidateNodeId) -> CandidateElementName {
        match self.node(id).kind { CandidateArenaKind::Element { name, .. } => name, _ => panic!("open element must be element") }
    }

    fn current_node(&self) -> Option<CandidateNodeId> { self.open_elements.last().copied() }

    fn current_is_p(&self) -> bool { self.current_node().is_some_and(|id| self.element_name(id).is_p()) }

    fn has_p(&self) -> bool { self.open_elements.iter().any(|id| self.element_name(*id).is_p()) }

    fn has_selected_block(&self) -> bool { self.open_elements.iter().any(|id| self.element_name(*id).is_selected_block()) }

    fn open_content(&self) -> Vec<CandidateStackEntry> {
        self.open_elements.iter().filter_map(|id| {
            let name = self.element_name(*id);
            matches!(name, CandidateElementName::Html | CandidateElementName::Body | CandidateElementName::Div | CandidateElementName::Section | CandidateElementName::P)
                .then_some(CandidateStackEntry { identity: *id, name })
        }).collect()
    }

    fn selected_stack(&self) -> Vec<CandidateStackEntry> {
        self.open_elements.iter().filter_map(|id| {
            let name = self.element_name(*id);
            name.is_selected_block().then_some(CandidateStackEntry { identity: *id, name })
        }).collect()
    }

    fn bounded_p_in_button_scope(&self) -> bool {
        let present = self.open_elements.iter().any(|id| self.element_name(*id).is_p());
        assert_eq!(present, self.current_is_p(), "candidate invariant reduces P button-scope to current P");
        present
    }

    fn bounded_close_p_implied_end_step(&self) -> usize {
        assert!(self.current_is_p());
        0
    }

    fn prepare_selected_end(&self, target_name: CandidateElementName) -> Option<CandidateSelectedEndPlan> {
        assert!(target_name.is_selected_block());
        if self.current_is_p() { return None; }
        let selected = self.selected_stack();
        let target_position = selected.iter().rposition(|entry| entry.name == target_name)?;
        let target = selected[target_position].clone();
        Some(CandidateSelectedEndPlan {
            target_identity: target.identity,
            target_name,
            intervening: selected[target_position + 1..].iter().rev().map(|entry| entry.identity).collect(),
        })
    }

    fn assert_invariant(&self) {
        let names: Vec<CandidateElementName> = self.open_elements.iter().map(|id| self.element_name(*id)).collect();
        let valid = match names.as_slice() {
            [] | [CandidateElementName::Html] => true,
            [CandidateElementName::Html, CandidateElementName::Head] => true,
            [CandidateElementName::Html, CandidateElementName::Body, rest @ ..] => {
                let p_count = rest.iter().filter(|name| **name == CandidateElementName::P).count();
                p_count <= 1
                    && rest.iter().take(rest.len().saturating_sub(p_count)).all(|name| name.is_selected_block())
                    && (p_count == 0 || rest.last() == Some(&CandidateElementName::P))
            }
            _ => false,
        };
        assert!(valid, "TC-S5 candidate stack invariant: {names:?}");
        assert_eq!(self.has_p(), self.current_is_p());
    }

    fn fingerprint(&self) -> CandidateFingerprint {
        CandidateFingerprint {
            slots: self.slots.clone(),
            document: self.document,
            open_elements: self.open_elements.clone(),
            head: self.head,
            mode: self.mode,
            diagnostics: self.diagnostics.clone(),
            recovery: self.recovery.clone(),
            p_closures: self.p_closures.clone(),
            p_syntheses: self.p_syntheses.clone(),
            selected_closures: self.selected_closures.clone(),
            identity_count: self.identity_count,
            committed_prefix_end: self.committed_prefix_end,
            processed_tokens: self.processed_tokens,
        }
    }

    fn process(&mut self, index: usize, shape: CandidateTokenShape<'_>, trigger: CandidateTrigger) -> CandidateTokenRecord {
        let mode_before = self.mode;
        let open_before = self.open_content();
        let identity_count_before = self.identity_count;
        let committed_before = self.committed_prefix_end;
        let processed_before = self.processed_tokens;
        let mut visited = Vec::new();
        let disposition;

        loop {
            self.assert_invariant();
            assert!(!visited.contains(&self.mode), "same token revisited mode {:?}", self.mode);
            visited.push(self.mode);
            let before = self.fingerprint();
            let plan = match &shape {
                CandidateTokenShape::EndTag { name, .. } if self.mode == CandidateMode::InBody && name.is_selected_block() && !self.current_is_p() => self.prepare_selected_end(*name),
                _ => None,
            };
            match select(self.mode, self.current_is_p(), self.has_selected_block(), plan, &shape) {
                Err(capability) => {
                    assert_eq!(self.fingerprint(), before, "refusal mutates nothing");
                    disposition = CandidateDisposition::Refused(capability);
                    break;
                }
                Ok(CandidateStep::Reprocess { effect, next }) => {
                    if let Some(effect) = effect { self.apply(effect, &trigger, &shape); }
                    self.mode = next;
                }
                Ok(CandidateStep::Consume { effect, next }) => {
                    if let Some(effect) = effect { self.apply(effect, &trigger, &shape); }
                    if let Some(next) = next { self.mode = next; }
                    self.commit(&shape);
                    disposition = CandidateDisposition::Consumed;
                    break;
                }
                Ok(CandidateStep::Ignore { effect }) => {
                    self.apply(effect, &trigger, &shape);
                    self.commit(&shape);
                    disposition = CandidateDisposition::Ignored;
                    break;
                }
                Ok(CandidateStep::Stop { effect }) => {
                    if let Some(effect) = effect { self.apply(effect, &trigger, &shape); }
                    self.commit(&shape);
                    disposition = CandidateDisposition::Stopped;
                    break;
                }
            }
        }

        self.assert_invariant();
        CandidateTokenRecord {
            index,
            trigger,
            mode_before,
            mode_after: self.mode,
            disposition,
            open_before,
            open_after: self.open_content(),
            identity_count_before,
            identity_count_after: self.identity_count,
            committed_before,
            committed_after: self.committed_prefix_end,
            processed_before,
            processed_after: self.processed_tokens,
        }
    }

    fn apply(&mut self, effect: CandidateEffect, trigger: &CandidateTrigger, shape: &CandidateTokenShape<'_>) {
        match effect {
            CandidateEffect::MissingDoctype => {
                self.diagnostics.push(CandidateDiagnostic { code: CandidateDiagnosticCode::MissingDoctype, trigger: trigger.clone() });
                self.recovery.push(CandidateRecovery::ContinuedInQuirksDocumentMode { trigger: trigger.clone() });
            }
            CandidateEffect::InsertShell { name, authored } => self.insert_shell(name, authored, shape),
            CandidateEffect::CloseHead => { let head = self.head.expect("head"); assert_eq!(self.current_node(), Some(head)); self.open_elements.pop(); }
            CandidateEffect::InsertCharacters => self.insert_characters(shape),
            CandidateEffect::StartP { close_current_p } => {
                if close_current_p { self.close_p(CandidatePClosureKind::StartTriggered, trigger); }
                self.insert_authored_element(CandidateElementName::P, shape);
            }
            CandidateEffect::StartSelected { name, close_current_p } => {
                if close_current_p { self.close_p(CandidatePClosureKind::StartTriggered, trigger); }
                self.insert_authored_element(name, shape);
            }
            CandidateEffect::ClosePMatching => self.close_p(CandidatePClosureKind::MatchingEnd, trigger),
            CandidateEffect::SynthesizePForUnmatchedEnd => self.synthesize_p_for_unmatched_end(trigger),
            CandidateEffect::CloseSelected { plan } => self.close_selected(plan, trigger, shape),
            CandidateEffect::UnmatchedSelected { name } => {
                assert!(name.is_selected_block());
                self.diagnostics.push(CandidateDiagnostic { code: CandidateDiagnosticCode::UnmatchedSelectedEndTag, trigger: trigger.clone() });
                self.recovery.push(CandidateRecovery::IgnoredSelectedEndTag { trigger: trigger.clone() });
            }
            CandidateEffect::OpenSelectedAtEof => {
                assert!(self.open_elements.iter().map(|id| self.element_name(*id)).any(|name| !name.permitted_at_in_body_eof()));
                self.diagnostics.push(CandidateDiagnostic { code: CandidateDiagnosticCode::OpenSelectedBlockAtEndOfFile, trigger: trigger.clone() });
                self.recovery.push(CandidateRecovery::StoppedParsingWithOpenSelectedBlock { trigger: trigger.clone() });
            }
        }
    }

    fn insert_shell(&mut self, name: CandidateElementName, authored: bool, shape: &CandidateTokenShape<'_>) {
        let parent = if name == CandidateElementName::Html { self.document } else { self.current_node().expect("shell parent") };
        let origin = if authored {
            let CandidateTokenShape::StartTag { complete, raw_name, .. } = shape else { panic!("authored shell requires start tag") };
            CandidateOrigin::Authored { complete: complete.clone(), raw_name: raw_name.clone() }
        } else {
            CandidateOrigin::Synthesized { cause: match name {
                CandidateElementName::Html => CandidateSynthesisCause::ImpliedHtml,
                CandidateElementName::Head => CandidateSynthesisCause::ImpliedHead,
                CandidateElementName::Body => CandidateSynthesisCause::ImpliedBody,
                _ => panic!("only shell can be implied here"),
            }}
        };
        let id = self.allocate(CandidateArenaKind::Element { name, origin });
        self.node_mut(parent).children.push(id);
        self.open_elements.push(id);
        if name == CandidateElementName::Head { self.head = Some(id); }
    }

    fn insert_authored_element(&mut self, name: CandidateElementName, shape: &CandidateTokenShape<'_>) {
        let CandidateTokenShape::StartTag { name: shape_name, complete, raw_name } = shape else { panic!("authored insertion requires start tag") };
        assert_eq!(*shape_name, name);
        let parent = self.current_node().expect("in-body parent");
        let id = self.allocate(CandidateArenaKind::Element {
            name,
            origin: CandidateOrigin::Authored { complete: complete.clone(), raw_name: raw_name.clone() },
        });
        self.node_mut(parent).children.push(id);
        self.open_elements.push(id);
    }

    fn close_p(&mut self, kind: CandidatePClosureKind, trigger: &CandidateTrigger) {
        assert!(self.bounded_p_in_button_scope());
        assert_eq!(self.bounded_close_p_implied_end_step(), 0);
        let target = self.current_node().expect("current P");
        assert_eq!(self.element_name(target), CandidateElementName::P);
        self.open_elements.pop();
        self.p_closures.push(CandidatePClosure { kind, target_identity: target, exact_trigger: trigger.clone() });
    }

    fn synthesize_p_for_unmatched_end(&mut self, trigger: &CandidateTrigger) {
        assert!(!self.bounded_p_in_button_scope());
        let parent = self.current_node().expect("synthesized P parent");
        let id = self.allocate(CandidateArenaKind::Element {
            name: CandidateElementName::P,
            origin: CandidateOrigin::Synthesized { cause: CandidateSynthesisCause::UnmatchedPEnd },
        });
        self.node_mut(parent).children.push(id);
        self.open_elements.push(id);
        self.p_syntheses.push(CandidatePSynthesis { identity: id, exact_unmatched_end_trigger: trigger.clone() });
        self.diagnostics.push(CandidateDiagnostic { code: CandidateDiagnosticCode::UnmatchedPEndTag, trigger: trigger.clone() });
        self.close_p(CandidatePClosureKind::UnmatchedEndSynthesized, trigger);
    }

    fn close_selected(&mut self, plan: CandidateSelectedEndPlan, trigger: &CandidateTrigger, shape: &CandidateTokenShape<'_>) {
        assert!(!self.current_is_p());
        let CandidateTokenShape::EndTag { name, .. } = shape else { panic!("selected close requires end tag") };
        assert_eq!(*name, plan.target_name);
        if !plan.intervening.is_empty() {
            self.diagnostics.push(CandidateDiagnostic { code: CandidateDiagnosticCode::MisnestedSelectedEndTag, trigger: trigger.clone() });
        }
        for popped_identity in &plan.intervening {
            assert_eq!(self.current_node(), Some(*popped_identity));
            self.open_elements.pop();
            self.recovery.push(CandidateRecovery::PoppedBySelectedAncestorEndTag {
                popped_identity: *popped_identity,
                target_identity: plan.target_identity,
                exact_end_trigger: trigger.clone(),
            });
        }
        assert_eq!(self.current_node(), Some(plan.target_identity));
        self.open_elements.pop();
        self.selected_closures.push(CandidateSelectedClosure { target_identity: plan.target_identity, exact_same_name_end_trigger: trigger.clone() });
    }

    fn insert_characters(&mut self, shape: &CandidateTokenShape<'_>) {
        let CandidateTokenShape::Characters { source, interpreted } = shape else { panic!("character insertion requires characters") };
        let parent = self.current_node().expect("character parent");
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
            CandidateArenaKind::Element { name, origin } => CandidateTree::Element { id, name: *name, namespace: CandidateNamespace::Html, origin: origin.clone(), children },
            CandidateArenaKind::Text { interpreted, contributions } => CandidateTree::Text { id, interpreted: interpreted.clone(), contributions: contributions.clone() },
        }
    }
}

fn reject_whitespace_sensitive(shape: &CandidateTokenShape<'_>) -> Result<(), CandidateUnsupported> {
    if let CandidateTokenShape::Characters { interpreted, .. } = shape
        && interpreted.chars().any(is_html_whitespace)
    { return Err(CandidateUnsupported::WhitespaceSensitiveCharacterData); }
    Ok(())
}

fn shell_walk_allowed(shape: &CandidateTokenShape<'_>) -> Result<(), CandidateUnsupported> {
    match shape {
        CandidateTokenShape::StartTag { name: CandidateElementName::Body, .. } | CandidateTokenShape::Characters { .. } => Ok(()),
        CandidateTokenShape::StartTag { name: CandidateElementName::P, .. } | CandidateTokenShape::EndTag { name: CandidateElementName::P, .. } => Err(CandidateUnsupported::PTagOutsideInBody),
        CandidateTokenShape::StartTag { name, .. } | CandidateTokenShape::EndTag { name, .. } if name.is_selected_block() => Err(CandidateUnsupported::SelectedTagOutsideInBody),
        _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
    }
}

fn select(
    mode: CandidateMode,
    current_is_p: bool,
    has_selected_block: bool,
    selected_plan: Option<CandidateSelectedEndPlan>,
    shape: &CandidateTokenShape<'_>,
) -> Result<CandidateStep, CandidateUnsupported> {
    match mode {
        CandidateMode::Initial => { reject_whitespace_sensitive(shape)?; shell_walk_allowed(shape)?; Ok(CandidateStep::Reprocess { effect: Some(CandidateEffect::MissingDoctype), next: CandidateMode::BeforeHtml }) }
        CandidateMode::BeforeHtml => { reject_whitespace_sensitive(shape)?; shell_walk_allowed(shape)?; Ok(CandidateStep::Reprocess { effect: Some(CandidateEffect::InsertShell { name: CandidateElementName::Html, authored: false }), next: CandidateMode::BeforeHead }) }
        CandidateMode::BeforeHead => { reject_whitespace_sensitive(shape)?; shell_walk_allowed(shape)?; Ok(CandidateStep::Reprocess { effect: Some(CandidateEffect::InsertShell { name: CandidateElementName::Head, authored: false }), next: CandidateMode::InHead }) }
        CandidateMode::InHead => { reject_whitespace_sensitive(shape)?; shell_walk_allowed(shape)?; Ok(CandidateStep::Reprocess { effect: Some(CandidateEffect::CloseHead), next: CandidateMode::AfterHead }) }
        CandidateMode::AfterHead => {
            reject_whitespace_sensitive(shape)?; shell_walk_allowed(shape)?;
            match shape {
                CandidateTokenShape::StartTag { name: CandidateElementName::Body, .. } => Ok(CandidateStep::Consume { effect: Some(CandidateEffect::InsertShell { name: CandidateElementName::Body, authored: true }), next: Some(CandidateMode::InBody) }),
                _ => Ok(CandidateStep::Reprocess { effect: Some(CandidateEffect::InsertShell { name: CandidateElementName::Body, authored: false }), next: CandidateMode::InBody }),
            }
        }
        CandidateMode::InBody => match shape {
            CandidateTokenShape::Characters { .. } => Ok(CandidateStep::Consume { effect: Some(CandidateEffect::InsertCharacters), next: None }),
            CandidateTokenShape::StartTag { name: CandidateElementName::P, .. } => Ok(CandidateStep::Consume { effect: Some(CandidateEffect::StartP { close_current_p: current_is_p }), next: None }),
            CandidateTokenShape::StartTag { name, .. } if name.is_selected_block() => Ok(CandidateStep::Consume { effect: Some(CandidateEffect::StartSelected { name: *name, close_current_p: current_is_p }), next: None }),
            CandidateTokenShape::EndTag { name: CandidateElementName::P, .. } => Ok(CandidateStep::Consume { effect: Some(if current_is_p { CandidateEffect::ClosePMatching } else { CandidateEffect::SynthesizePForUnmatchedEnd }), next: None }),
            CandidateTokenShape::EndTag { name, .. } if name.is_selected_block() && current_is_p => Err(CandidateUnsupported::SelectedEndWithOpenP),
            CandidateTokenShape::EndTag { name, .. } if name.is_selected_block() => match selected_plan {
                Some(plan) => Ok(CandidateStep::Consume { effect: Some(CandidateEffect::CloseSelected { plan }), next: None }),
                None => Ok(CandidateStep::Ignore { effect: CandidateEffect::UnmatchedSelected { name: *name } }),
            },
            CandidateTokenShape::EndTag { name: CandidateElementName::Body, .. } if current_is_p => Err(CandidateUnsupported::BodyCloseWithOpenP),
            CandidateTokenShape::EndTag { name: CandidateElementName::Body, .. } if has_selected_block => Err(CandidateUnsupported::BodyCloseWithOpenSelectedBlock),
            CandidateTokenShape::EndTag { name: CandidateElementName::Body, .. } => Ok(CandidateStep::Consume { effect: None, next: Some(CandidateMode::AfterBody) }),
            CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop { effect: has_selected_block.then_some(CandidateEffect::OpenSelectedAtEof) }),
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
        CandidateMode::AfterBody => match shape {
            CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop { effect: None }),
            CandidateTokenShape::StartTag { name: CandidateElementName::P, .. } | CandidateTokenShape::EndTag { name: CandidateElementName::P, .. } => Err(CandidateUnsupported::PTagOutsideInBody),
            CandidateTokenShape::StartTag { name, .. } | CandidateTokenShape::EndTag { name, .. } if name.is_selected_block() => Err(CandidateUnsupported::SelectedTagOutsideInBody),
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
    }
}

fn generous_limits() -> HtmlTokenizerLimits { HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024) }

fn tokenize_text(text: &str, source_id: u64, limits: HtmlTokenizerLimits) -> HtmlTokenizerRunResult {
    tokenize(&SourceText::new(SourceId::new(source_id), text.to_owned()), limits)
}

fn run_for(id: &str, source_id: u64) -> HtmlTokenizerRunResult { tokenize_text(fixture(id).source_text(), source_id, generous_limits()) }

fn observe_with_layout(run: &HtmlTokenizerRunResult, layout: CandidateStorageLayout) -> CandidateObservation {
    let mut session = CandidateSession::new(layout);
    let mut records = Vec::new();
    let mut refusal = None;
    let mut stopped = false;

    for (index, token) in run.tokens().iter().enumerate() {
        let trigger = candidate_trigger(token, index);
        let before = session.fingerprint();
        let open_before = session.open_content();
        let shape = match candidate_shape(token) {
            Ok(shape) => shape,
            Err(capability) => {
                assert_eq!(session.fingerprint(), before, "shape refusal mutates nothing");
                records.push(CandidateTokenRecord {
                    index,
                    trigger: trigger.clone(),
                    mode_before: session.mode,
                    mode_after: session.mode,
                    disposition: CandidateDisposition::Refused(capability),
                    open_before: open_before.clone(),
                    open_after: open_before,
                    identity_count_before: session.identity_count,
                    identity_count_after: session.identity_count,
                    committed_before: session.committed_prefix_end,
                    committed_after: session.committed_prefix_end,
                    processed_before: session.processed_tokens,
                    processed_after: session.processed_tokens,
                });
                refusal = Some((capability, trigger));
                break;
            }
        };
        let record = session.process(index, shape, trigger.clone());
        if let Some(capability) = record.refusal() { refusal = Some((capability, trigger)); }
        let is_refused = record.refusal().is_some();
        let is_stopped = record.stopped();
        records.push(record);
        if is_refused { break; }
        if is_stopped { stopped = true; break; }
    }

    let completion = match refusal {
        Some((capability, trigger)) => CandidateCompletion::IncompleteUnsupported { capability, trigger },
        None if stopped && session.processed_tokens == run.tokens().len() && !run.is_incomplete() => CandidateCompletion::Complete,
        None => CandidateCompletion::IncompleteLowerLayer,
    };

    CandidateObservation {
        tree: session.tree(),
        diagnostics: session.diagnostics,
        recovery: session.recovery,
        p_closures: session.p_closures,
        p_syntheses: session.p_syntheses,
        selected_closures: session.selected_closures,
        identity_count: session.identity_count,
        checkpoint: CandidateCheckpoint {
            mode: session.mode,
            open_content: session.open_content(),
            committed_prefix_end: session.committed_prefix_end,
            processed_tokens: session.processed_tokens,
            completion,
        },
        tokens: records,
    }
}

fn observe(run: &HtmlTokenizerRunResult) -> CandidateObservation { observe_with_layout(run, CandidateStorageLayout::COMPACT) }
fn observe_fixture(id: &str, source_id: u64) -> CandidateObservation { observe(&run_for(id, source_id)) }
fn node_id(value: usize) -> CandidateNodeId { CandidateNodeId(value) }

fn authored_trigger(source_id: u64, index: usize, complete: (usize, usize), raw_name: Option<(usize, usize)>) -> CandidateTrigger {
    CandidateTrigger::Authored {
        index,
        complete: expected_evidence(source_id, complete),
        raw_name: raw_name.map(|range| expected_evidence(source_id, range)),
    }
}

fn p_closure(kind: CandidatePClosureKind, target: usize, trigger: CandidateTrigger) -> CandidatePClosure {
    CandidatePClosure { kind, target_identity: node_id(target), exact_trigger: trigger }
}

fn synthesized_p_nodes(tree: &CandidateTree, output: &mut Vec<(CandidateNodeId, CandidateOrigin)>) {
    match tree {
        CandidateTree::Document { children, .. } | CandidateTree::Element { children, .. } => {
            if let CandidateTree::Element { id, name: CandidateElementName::P, origin, .. } = tree {
                if matches!(origin, CandidateOrigin::Synthesized { cause: CandidateSynthesisCause::UnmatchedPEnd }) { output.push((*id, origin.clone())); }
            }
            for child in children { synthesized_p_nodes(child, output); }
        }
        CandidateTree::Text { .. } => {}
    }
}

fn collect_elements(tree: &CandidateTree, output: &mut Vec<(CandidateNodeId, CandidateElementName, CandidateOrigin)>) {
    match tree {
        CandidateTree::Document { children, .. } => for child in children { collect_elements(child, output); },
        CandidateTree::Element { id, name, origin, children, .. } => {
            output.push((*id, *name, origin.clone()));
            for child in children { collect_elements(child, output); }
        }
        CandidateTree::Text { .. } => {}
    }
}

fn collect_text(tree: &CandidateTree, output: &mut Vec<(String, Vec<(usize, usize)>)>) {
    match tree {
        CandidateTree::Document { children, .. } | CandidateTree::Element { children, .. } => for child in children { collect_text(child, output); },
        CandidateTree::Text { interpreted, contributions, .. } => output.push((interpreted.clone(), contributions.iter().map(|evidence| evidence.range).collect())),
    }
}

fn assert_complete(observation: &CandidateObservation) { assert_eq!(observation.checkpoint.completion, CandidateCompletion::Complete); }

fn assert_refusal(id: &str, capability: CandidateUnsupported) {
    let run = run_for(id, 1);
    let observation = observe(&run);
    let record = observation.tokens.last().expect("refusal record");
    assert_eq!(record.refusal(), Some(capability), "{id}");
    assert_eq!(record.open_before, record.open_after, "{id}");
    assert_eq!(record.identity_count_before, record.identity_count_after, "{id}");
    assert_eq!(record.committed_before, record.committed_after, "{id}");
    assert_eq!(record.processed_before, record.processed_after, "{id}");
    assert_eq!(
        observation.checkpoint.completion,
        CandidateCompletion::IncompleteUnsupported { capability, trigger: record.trigger.clone() },
        "{id}"
    );
}

fn normalize_source_ids(tree: &mut CandidateTree) {
    match tree {
        CandidateTree::Document { children, .. } => for child in children { normalize_source_ids(child); },
        CandidateTree::Element { origin, children, .. } => {
            if let CandidateOrigin::Authored { complete, raw_name } = origin {
                complete.source_id = SourceId::new(1);
                raw_name.source_id = SourceId::new(1);
            }
            for child in children { normalize_source_ids(child); }
        }
        CandidateTree::Text { contributions, .. } => for contribution in contributions { contribution.source_id = SourceId::new(1); },
    }
}

fn generated_candidate_sources() -> Vec<String> {
    const PIECES: [&str; 7] = ["<p>", "</p>", "<div>", "</div>", "<section>", "</section>", "x"];
    const MAX_SEQUENCE_LENGTH: u32 = 4;
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
    sources
}

#[test]
fn canonical_fixture_bytes_ranges_and_authority_are_frozen() {
    assert_eq!(CANDIDATE_FIXTURES.len(), CANDIDATE_IDS.len());
    assert_eq!(PINNED_WHATWG_COMMIT, "508a037333d8a1806504303aeb489d931fabbef6");
    assert_eq!(PINNED_WHATWG_SOURCE_BLOB, "68dbcb98bbe1001c6ae2531be2368c608fbafddd");
    for (fixture, id) in CANDIDATE_FIXTURES.iter().zip(CANDIDATE_IDS) {
        assert_eq!(fixture.id, id);
        assert_eq!(fixture.bytes.len(), fixture.length, "{id}");
        assert_eq!(fixture.source_text().as_bytes(), fixture.bytes, "{id}");
        assert_eq!(fixture.sha256.len(), 64, "{id}");
        assert!(fixture.sha256.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()), "{id}");
        for ((start, end), expected) in fixture.required_ranges {
            assert!(*start <= *end && *end <= fixture.length, "{id}");
            assert_eq!(&fixture.bytes[*start..*end], *expected, "{id} [{start},{end})");
        }
    }
}

#[test]
fn candidate_independence_and_closed_theorem_are_explicit() {
    let source = include_str!("in_body_p_successor_validation.rs");
    for pattern in [
        ["use super::", "driver"].concat(),
        ["use super::", "session"].concat(),
        ["use super::", "result"].concat(),
        ["construct_html_document_", "shell("].concat(),
    ] { assert!(!source.contains(pattern.as_str()), "forbidden semantic oracle import: {pattern}"); }
    assert!(CandidateElementName::Div.is_selected_block());
    assert!(CandidateElementName::Section.is_selected_block());
    assert!(!CandidateElementName::P.is_selected_block());
    assert!(CandidateElementName::P.permitted_at_in_body_eof());
}

#[test]
fn tokenizer_evidence_preserves_source_id_complete_and_raw_name_ranges() {
    for id in CANDIDATE_IDS {
        let run = run_for(id, 41);
        for token in run.tokens() {
            match token {
                HtmlToken::Character(character) => assert_eq!(character.source().source_id(), SourceId::new(41), "{id}"),
                HtmlToken::Tag(tag) => {
                    assert_eq!(tag.complete().source_id(), SourceId::new(41), "{id}");
                    assert_eq!(tag.name().source().source_id(), SourceId::new(41), "{id}");
                }
                HtmlToken::EndOfFile(_) => {}
            }
        }
    }
    let p2 = run_for("P2", 41);
    let HtmlToken::Tag(start) = &p2.tokens()[1] else { panic!("P2 start tag") };
    assert_eq!(start.name().interpreted(), "p");
    assert_eq!(evidence(start.complete()).range, (6, 9));
    assert_eq!(evidence(start.name().source()).range, (7, 8));
    let HtmlToken::Tag(end) = &p2.tokens()[3] else { panic!("P2 end tag") };
    assert_eq!(end.name().interpreted(), "p");
    assert_eq!(evidence(end.complete()).range, (10, 14));
    assert_eq!(evidence(end.name().source()).range, (12, 13));
}

#[test]
fn p1_and_p2_pin_authored_lifecycle_and_raw_spelling() {
    for id in ["P1", "P2"] {
        let observation = observe_fixture(id, 1);
        assert_complete(&observation);
        let mut elements = Vec::new();
        collect_elements(&observation.tree, &mut elements);
        let p = elements.iter().find(|(_, name, _)| *name == CandidateElementName::P).expect("authored P");
        assert_eq!(p.0, node_id(4));
        assert!(matches!(p.2, CandidateOrigin::Authored { .. }));
        assert_eq!(observation.p_syntheses, vec![]);
        assert_eq!(observation.p_closures, vec![p_closure(CandidatePClosureKind::MatchingEnd, 4, authored_trigger(1, 3, (10,14), Some((12,13))))]);
        assert_eq!(observation.diagnostics.len(), 1, "only MissingDoctype is predecessor diagnostic");
    }
    let p2 = observe_fixture("P2", 1);
    let mut elements = Vec::new();
    collect_elements(&p2.tree, &mut elements);
    let (_, _, CandidateOrigin::Authored { complete, raw_name }) = elements.iter().find(|(_, name, _)| *name == CandidateElementName::P).expect("P2 P") else { panic!("authored P") };
    assert_eq!(complete.range, (6,9));
    assert_eq!(raw_name.range, (7,8));
}

#[test]
fn p3_start_p_closes_current_before_allocating_new_p() {
    let observation = observe_fixture("P3", 1);
    assert_complete(&observation);
    assert_eq!(observation.p_closures, vec![
        p_closure(CandidatePClosureKind::StartTriggered, 4, authored_trigger(1, 3, (10,13), Some((11,12)))),
        p_closure(CandidatePClosureKind::MatchingEnd, 6, authored_trigger(1, 5, (14,18), Some((16,17)))),
    ]);
    assert_eq!(observation.p_syntheses, vec![]);
    let record = &observation.tokens[3];
    assert_eq!(record.identity_count_after, record.identity_count_before + 1);
    assert_eq!(record.disposition, CandidateDisposition::Consumed);
    let mut elements = Vec::new();
    collect_elements(&observation.tree, &mut elements);
    let p_ids: Vec<_> = elements.into_iter().filter(|(_, name, _)| *name == CandidateElementName::P).map(|(id, _, _)| id).collect();
    assert_eq!(p_ids, vec![node_id(4), node_id(6)]);
}

#[test]
fn p4_and_p5_block_starts_close_p_without_tc_s4_recovery() {
    for (id, trigger_range, raw_range) in [("P4", (10,15), (11,14)), ("P5", (10,19), (11,18))] {
        let observation = observe_fixture(id, 1);
        assert_complete(&observation);
        assert_eq!(observation.p_closures, vec![p_closure(CandidatePClosureKind::StartTriggered, 4, authored_trigger(1, 3, trigger_range, Some(raw_range)))]);
        assert!(observation.recovery.iter().all(|recovery| !matches!(recovery, CandidateRecovery::PoppedBySelectedAncestorEndTag { .. })));
        assert_eq!(observation.diagnostics.len(), 1, "start-triggered P closure has no P diagnostic");
    }
}

#[test]
fn p6_nested_p_matching_end_preserves_selected_parent() {
    let observation = observe_fixture("P6", 1);
    assert_complete(&observation);
    assert_eq!(observation.p_closures, vec![p_closure(CandidatePClosureKind::MatchingEnd, 5, authored_trigger(1, 4, (15,19), Some((17,18))))]);
    assert_eq!(observation.selected_closures.len(), 1);
    assert_eq!(observation.selected_closures[0].target_identity, node_id(4));
}

#[test]
fn p7_p8_p9_pin_source_less_unmatched_end_synthesis_and_distinct_identities() {
    let p7 = observe_fixture("P7", 1);
    assert_complete(&p7);
    assert_eq!(p7.diagnostics.iter().filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::UnmatchedPEndTag).count(), 1);
    assert_eq!(p7.p_syntheses, vec![CandidatePSynthesis { identity: node_id(4), exact_unmatched_end_trigger: authored_trigger(1, 1, (6,10), Some((8,9))) }]);
    assert_eq!(p7.p_closures, vec![p_closure(CandidatePClosureKind::UnmatchedEndSynthesized, 4, authored_trigger(1, 1, (6,10), Some((8,9))))]);
    let mut synthesized = Vec::new();
    synthesized_p_nodes(&p7.tree, &mut synthesized);
    assert_eq!(synthesized.len(), 1);
    assert!(matches!(synthesized[0].1, CandidateOrigin::Synthesized { cause: CandidateSynthesisCause::UnmatchedPEnd }));

    let p8 = observe_fixture("P8", 1);
    assert_complete(&p8);
    assert_eq!(p8.p_syntheses[0].identity, node_id(5), "synthesized P is placed under current div and gets its own identity");
    let mut texts = Vec::new();
    collect_text(&p8.tree, &mut texts);
    assert_eq!(texts, vec![("x".to_owned(), vec![(15,16)])]);

    let p9 = observe_fixture("P9", 1);
    assert_complete(&p9);
    assert_eq!(p9.p_syntheses.iter().map(|synthesis| synthesis.identity).collect::<Vec<_>>(), vec![node_id(4), node_id(5)]);
    assert_eq!(p9.p_closures.iter().map(|closure| closure.target_identity).collect::<Vec<_>>(), vec![node_id(4), node_id(5)]);
    assert_eq!(p9.diagnostics.iter().filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::UnmatchedPEndTag).count(), 2);
}

#[test]
fn p10_and_p11_pin_p_specific_eof_non_action_and_predecessor_block_eof_diagnostic() {
    let p10 = observe_fixture("P10", 1);
    assert_complete(&p10);
    assert!(p10.p_closures.is_empty());
    assert!(p10.p_syntheses.is_empty());
    assert_eq!(p10.diagnostics.len(), 1, "P-only EOF adds no diagnostic");
    assert_eq!(p10.checkpoint.open_content.last().map(|entry| entry.name), Some(CandidateElementName::P));

    let p11 = observe_fixture("P11", 1);
    assert_complete(&p11);
    assert!(p11.p_closures.is_empty());
    assert_eq!(p11.diagnostics.iter().filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::OpenSelectedBlockAtEndOfFile).count(), 1);
    assert_eq!(p11.diagnostics.iter().filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::UnmatchedPEndTag).count(), 0);
    assert!(matches!(p11.recovery.last(), Some(CandidateRecovery::StoppedParsingWithOpenSelectedBlock { .. })));
}

#[test]
fn p12_through_p17_and_p21_refuse_transactionally_before_candidate_mutation() {
    for (id, capability) in [
        ("P12", CandidateUnsupported::SelectedEndWithOpenP),
        ("P13", CandidateUnsupported::SelectedEndWithOpenP),
        ("P14", CandidateUnsupported::PStartTagAttribute),
        ("P15", CandidateUnsupported::PSelfClosingStartTag),
        ("P16", CandidateUnsupported::PTagOutsideInBody),
        ("P17", CandidateUnsupported::BodyCloseWithOpenP),
        ("P21", CandidateUnsupported::PEndTagAttribute),
    ] { assert_refusal(id, capability); }
}

#[test]
fn p18_and_p19_pin_predecessor_div_section_semantics_without_p_regression() {
    let p18 = observe_fixture("P18", 1);
    assert_complete(&p18);
    assert!(p18.p_closures.is_empty());
    assert!(p18.p_syntheses.is_empty());
    assert_eq!(p18.selected_closures.len(), 2);
    assert!(p18.recovery.iter().all(|recovery| !matches!(recovery, CandidateRecovery::PoppedBySelectedAncestorEndTag { .. })));

    let p19 = observe_fixture("P19", 1);
    assert_complete(&p19);
    assert!(p19.p_closures.is_empty());
    assert!(p19.p_syntheses.is_empty());
    assert_eq!(p19.selected_closures, vec![CandidateSelectedClosure { target_identity: node_id(4), exact_same_name_end_trigger: authored_trigger(1, 3, (20,26), Some((22,25))) }]);
    assert!(matches!(p19.recovery.get(1), Some(CandidateRecovery::PoppedBySelectedAncestorEndTag { popped_identity, target_identity, .. }) if *popped_identity == node_id(5) && *target_identity == node_id(4)));
    assert_eq!(p19.diagnostics.iter().filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::MisnestedSelectedEndTag).count(), 1);
}

#[test]
fn p20_proves_exact_source_id_ranges_and_trigger_origin_identity_separation() {
    let observation = observe_fixture("P20", 4_242);
    assert_complete(&observation);
    let mut elements = Vec::new();
    collect_elements(&observation.tree, &mut elements);
    let (identity, _, origin) = elements.into_iter().find(|(_, name, _)| *name == CandidateElementName::P).expect("P20 P");
    assert_eq!(identity, node_id(4));
    let CandidateOrigin::Authored { complete, raw_name } = origin else { panic!("authored P") };
    assert_eq!(complete, expected_evidence(4_242, (6,9)));
    assert_eq!(raw_name, expected_evidence(4_242, (7,8)));
    assert_eq!(observation.p_closures[0].exact_trigger, authored_trigger(4_242, 3, (10,14), Some((12,13))));
    assert_ne!(complete.range, match &observation.p_closures[0].exact_trigger { CandidateTrigger::Authored { complete, .. } => complete.range, _ => unreachable!() });
}

#[test]
fn bounded_close_p_scope_and_implied_end_step_are_exact_not_generic() {
    for id in ["P1", "P3", "P4", "P5", "P6", "P7", "P8", "P9"] {
        let observation = observe_fixture(id, 1);
        for closure in &observation.p_closures {
            assert!(matches!(closure.kind, CandidatePClosureKind::MatchingEnd | CandidatePClosureKind::StartTriggered | CandidatePClosureKind::UnmatchedEndSynthesized));
        }
    }
    let mut session = CandidateSession::new(CandidateStorageLayout::COMPACT);
    assert!(!session.has_p());
    assert!(!session.current_is_p());
}

#[test]
fn constructed_identity_is_semantic_and_storage_layout_independent() {
    for id in ["P3", "P8", "P9", "P19"] {
        let run = run_for(id, 77);
        let compact = observe_with_layout(&run, CandidateStorageLayout::COMPACT);
        let padded = observe_with_layout(&run, CandidateStorageLayout::PADDED);
        assert_eq!(compact.tree, padded.tree, "{id}");
        assert_eq!(compact.p_closures, padded.p_closures, "{id}");
        assert_eq!(compact.p_syntheses, padded.p_syntheses, "{id}");
        assert_eq!(compact.selected_closures, padded.selected_closures, "{id}");
        assert_eq!(compact.identity_count, padded.identity_count, "{id}");
    }
}

#[test]
fn source_id_changes_provenance_not_constructed_identity_or_tree_shape() {
    let run_a = run_for("P9", 7);
    let run_b = run_for("P9", 9_999);
    let a = observe(&run_a);
    let b = observe(&run_b);
    assert_eq!(a.p_syntheses.iter().map(|item| item.identity).collect::<Vec<_>>(), b.p_syntheses.iter().map(|item| item.identity).collect::<Vec<_>>());
    let mut tree_a = a.tree.clone();
    let mut tree_b = b.tree.clone();
    normalize_source_ids(&mut tree_a);
    normalize_source_ids(&mut tree_b);
    assert_eq!(tree_a, tree_b);
}

#[test]
fn every_supported_p_effect_has_exactly_one_emitted_token_trigger_and_no_redispatch_loop() {
    for id in CANDIDATE_IDS {
        let observation = observe_fixture(id, 1);
        for closure in &observation.p_closures { assert!(closure.exact_trigger.index() < observation.tokens.len(), "{id}"); }
        for synthesis in &observation.p_syntheses { assert!(synthesis.exact_unmatched_end_trigger.index() < observation.tokens.len(), "{id}"); }
        for record in &observation.tokens {
            if record.mode_before == CandidateMode::InBody {
                assert_eq!(record.mode_after, CandidateMode::InBody, "{id} token {}", record.index);
            }
        }
    }
}

#[test]
fn lower_layer_incompleteness_is_monotonic_and_never_upgraded() {
    let run = tokenize_text("<body><p>xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", 1, HtmlTokenizerLimits::new(1, 1, 1, 1, 1, 1, 1));
    assert!(run.is_incomplete());
    let observation = observe(&run);
    assert_ne!(observation.checkpoint.completion, CandidateCompletion::Complete);
}

#[test]
fn generated_bounded_sequences_preserve_invariants_or_refuse_at_a_clean_checkpoint() {
    for source in generated_candidate_sources() {
        let run = tokenize_text(&source, 1, generous_limits());
        let observation = observe(&run);
        for record in &observation.tokens {
            if record.refusal().is_some() {
                assert_eq!(record.open_before, record.open_after, "{source}");
                assert_eq!(record.identity_count_before, record.identity_count_after, "{source}");
                assert_eq!(record.committed_before, record.committed_after, "{source}");
                assert_eq!(record.processed_before, record.processed_after, "{source}");
            }
        }
        let mut synthesized = Vec::new();
        synthesized_p_nodes(&observation.tree, &mut synthesized);
        assert_eq!(synthesized.len(), observation.p_syntheses.len(), "{source}");
        for (id, origin) in synthesized {
            assert!(observation.p_syntheses.iter().any(|synthesis| synthesis.identity == id), "{source}");
            assert!(matches!(origin, CandidateOrigin::Synthesized { cause: CandidateSynthesisCause::UnmatchedPEnd }), "{source}");
        }
        let p_diagnostics = observation.diagnostics.iter().filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::UnmatchedPEndTag).count();
        assert_eq!(p_diagnostics, observation.p_syntheses.len(), "{source}");
        assert_eq!(observation.p_syntheses.len(), observation.p_closures.iter().filter(|closure| closure.kind == CandidatePClosureKind::UnmatchedEndSynthesized).count(), "{source}");
    }
}
