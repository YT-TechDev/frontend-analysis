//! Candidate-independent TC-S5 successor validation.
//!
//! TC-S5 is the bounded successor theorem "Selected In-Body `p` Lifecycle
//! with Bounded Implicit Closure and Unmatched-End Synthesis" from Issue #365.
//! This module consumes the accepted tokenizer only as lower-layer token/source
//! evidence. It intentionally imports no production tree-construction driver,
//! session, or result semantics.
//!
//! Normative authority: WHATWG HTML commit
//! `508a037333d8a1806504303aeb489d931fabbef6`, source blob
//! `68dbcb98bbe1001c6ae2531be2368c608fbafddd`.
//!
//! Closed candidate stack theorem:
//!
//! ```text
//! [html, body] ++ B* ++ P?
//! B in {Div, Section}
//! count(P) <= 1
//! P present => P is current
//! ```
//!
//! Therefore the close-P implied-end step is validated only as a bounded no-op.
//! No generic button-scope or implied-end engine is introduced here.

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlTagToken, HtmlToken};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;

const PINNED_WHATWG_COMMIT: &str = "508a037333d8a1806504303aeb489d931fabbef6";
const PINNED_WHATWG_BLOB: &str = "68dbcb98bbe1001c6ae2531be2368c608fbafddd";

#[derive(Debug, Clone, Copy)]
struct Fixture {
    id: &'static str,
    source: &'static str,
}

#[rustfmt::skip]
const FIXTURES: &[Fixture] = &[
    Fixture { id: "P1", source: "<body><p>x</p>" },
    Fixture { id: "P2", source: "<body><P>x</p>" },
    Fixture { id: "P3", source: "<body><p>a<p>b</p>" },
    Fixture { id: "P4", source: "<body><p>a<div>b</div>" },
    Fixture { id: "P5", source: "<body><p>a<section>b</section>" },
    Fixture { id: "P6", source: "<body><div><p>x</p></div>" },
    Fixture { id: "P7", source: "<body></p>" },
    Fixture { id: "P8", source: "<body><div></p>x</div>" },
    Fixture { id: "P9", source: "<body></p></p>" },
    Fixture { id: "P10", source: "<body><p>x" },
    Fixture { id: "P11", source: "<body><div><p>x" },
    Fixture { id: "P12", source: "<body><div><p></div>" },
    Fixture { id: "P13", source: "<body><section><p></section>" },
    Fixture { id: "P14", source: "<body><p id=x>" },
    Fixture { id: "P15", source: "<body><p/>" },
    Fixture { id: "P16", source: "<body></body><p>" },
    Fixture { id: "P17", source: "<body><p></body>" },
    Fixture { id: "P18", source: "<body><div>x</div><section>y</section>" },
    Fixture { id: "P19", source: "<body><div><section></div>" },
    Fixture { id: "P20", source: "<body><p>Z</p>" },
    Fixture { id: "P21", source: "<body><p></p id=x>" },
];

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .expect("canonical TC-S5 fixture")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Evidence {
    source_id: SourceId,
    range: (usize, usize),
}

fn evidence(anchor: &SourceAnchor) -> Evidence {
    Evidence {
        source_id: anchor.source_id(),
        range: (anchor.range().start(), anchor.range().end()),
    }
}

fn expected_evidence(source_id: u64, range: (usize, usize)) -> Evidence {
    Evidence {
        source_id: SourceId::new(source_id),
        range,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NodeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Name {
    Html,
    Head,
    Body,
    Div,
    Section,
    P,
}

impl Name {
    fn is_block(self) -> bool {
        matches!(self, Self::Div | Self::Section)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Origin {
    Authored {
        complete: Evidence,
        raw_name: Evidence,
    },
    Synthesized(SynthesisCause),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SynthesisCause {
    ImpliedHtml,
    ImpliedHead,
    UnmatchedPEnd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeKind {
    Document,
    Element {
        name: Name,
        origin: Origin,
    },
    Text {
        interpreted: String,
        contributions: Vec<Evidence>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    id: NodeId,
    parent: Option<NodeId>,
    kind: NodeKind,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    BeforeBody,
    InBody,
    AfterBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Diagnostic {
    MissingDoctype,
    UnmatchedPEnd,
    UnmatchedBlockEnd,
    MisnestedBlockEnd,
    OpenBlockAtEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PClosureKind {
    MatchingEnd,
    StartTriggered,
    UnmatchedEndSynthesized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PClosure {
    kind: PClosureKind,
    target: NodeId,
    token_index: usize,
    trigger: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PSynthesis {
    node: NodeId,
    token_index: usize,
    trigger: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockRecovery {
    popped: NodeId,
    target: NodeId,
    token_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unsupported {
    PStartAttribute,
    PEndAttribute,
    PSelfClosing,
    POutsideInBody,
    BlockEndWithOpenP,
    BodyEndWithOpenP,
    BodyEndWithOpenBlock,
    BlockAttribute,
    BlockSelfClosing,
    GenericTag,
    OutsideCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Completion {
    Complete,
    Unsupported {
        capability: Unsupported,
        token_index: usize,
    },
    LowerLayerIncomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    phase: Phase,
    open: Vec<NodeId>,
    next_id: usize,
    committed_end: usize,
    processed_tokens: usize,
    diagnostic_count: usize,
    p_closure_count: usize,
    p_synthesis_count: usize,
    block_recovery_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RefusalRecord {
    capability: Unsupported,
    token_index: usize,
    before: Snapshot,
    after: Snapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    nodes: Vec<Node>,
    diagnostics: Vec<Diagnostic>,
    p_closures: Vec<PClosure>,
    p_syntheses: Vec<PSynthesis>,
    block_recovery: Vec<BlockRecovery>,
    open: Vec<NodeId>,
    phase: Phase,
    completion: Completion,
    refusal: Option<RefusalRecord>,
}

#[derive(Debug, Clone, Copy)]
struct StorageLayout {
    leading_padding: usize,
    inter_node_padding: usize,
}

impl StorageLayout {
    const COMPACT: Self = Self {
        leading_padding: 0,
        inter_node_padding: 0,
    };

    const PADDED: Self = Self {
        leading_padding: 3,
        inter_node_padding: 2,
    };
}

struct Machine {
    slots: Vec<Option<Node>>,
    layout: StorageLayout,
    next_id: usize,
    document: NodeId,
    html: NodeId,
    open: Vec<NodeId>,
    phase: Phase,
    diagnostics: Vec<Diagnostic>,
    p_closures: Vec<PClosure>,
    p_syntheses: Vec<PSynthesis>,
    block_recovery: Vec<BlockRecovery>,
    committed_end: usize,
    processed_tokens: usize,
}

impl Machine {
    fn new(layout: StorageLayout) -> Self {
        let mut machine = Self {
            slots: vec![None; layout.leading_padding],
            layout,
            next_id: 0,
            document: NodeId(0),
            html: NodeId(0),
            open: Vec::new(),
            phase: Phase::BeforeBody,
            diagnostics: vec![Diagnostic::MissingDoctype],
            p_closures: Vec::new(),
            p_syntheses: Vec::new(),
            block_recovery: Vec::new(),
            committed_end: 0,
            processed_tokens: 0,
        };

        machine.document = machine.allocate(None, NodeKind::Document);
        machine.html = machine.allocate(
            Some(machine.document),
            NodeKind::Element {
                name: Name::Html,
                origin: Origin::Synthesized(SynthesisCause::ImpliedHtml),
            },
        );
        machine.allocate(
            Some(machine.html),
            NodeKind::Element {
                name: Name::Head,
                origin: Origin::Synthesized(SynthesisCause::ImpliedHead),
            },
        );
        machine.open.push(machine.html);
        machine.assert_invariant();
        machine
    }

    fn allocate(&mut self, parent: Option<NodeId>, kind: NodeKind) -> NodeId {
        if self.next_id != 0 {
            self.slots
                .extend((0..self.layout.inter_node_padding).map(|_| None));
        }
        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.slots.push(Some(Node { id, parent, kind }));
        id
    }

    fn node(&self, id: NodeId) -> &Node {
        self.slots
            .iter()
            .flatten()
            .find(|node| node.id == id)
            .expect("semantic node id")
    }

    fn node_mut(&mut self, id: NodeId) -> &mut Node {
        self.slots
            .iter_mut()
            .flatten()
            .find(|node| node.id == id)
            .expect("semantic node id")
    }

    fn name(&self, id: NodeId) -> Name {
        match self.node(id).kind {
            NodeKind::Element { name, .. } => name,
            _ => panic!("open node must be an element"),
        }
    }

    fn current(&self) -> NodeId {
        *self.open.last().expect("open element")
    }

    fn current_is_p(&self) -> bool {
        self.name(self.current()) == Name::P
    }

    fn has_open_block(&self) -> bool {
        self.open.iter().any(|id| self.name(*id).is_block())
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            phase: self.phase,
            open: self.open.clone(),
            next_id: self.next_id,
            committed_end: self.committed_end,
            processed_tokens: self.processed_tokens,
            diagnostic_count: self.diagnostics.len(),
            p_closure_count: self.p_closures.len(),
            p_synthesis_count: self.p_syntheses.len(),
            block_recovery_count: self.block_recovery.len(),
        }
    }

    fn assert_invariant(&self) {
        let names: Vec<Name> = self.open.iter().map(|id| self.name(*id)).collect();
        let valid = match self.phase {
            Phase::BeforeBody => names == [Name::Html],
            Phase::AfterBody => names == [Name::Html, Name::Body],
            Phase::InBody => {
                if names.len() < 2 || names[0] != Name::Html || names[1] != Name::Body {
                    false
                } else {
                    let mut saw_p = false;
                    names[2..].iter().all(|name| match name {
                        Name::Div | Name::Section if !saw_p => true,
                        Name::P if !saw_p => {
                            saw_p = true;
                            true
                        }
                        _ => false,
                    })
                }
            }
        };
        assert!(
            valid,
            "TC-S5 stack invariant violated: phase={:?} names={names:?}",
            self.phase
        );
        let p_present = names.contains(&Name::P);
        assert_eq!(p_present, names.last() == Some(&Name::P));
    }

    fn commit(&mut self, token: &HtmlToken) {
        self.committed_end = token_end(token);
        self.processed_tokens += 1;
    }

    fn insert_authored(&mut self, name: Name, complete: Evidence, raw_name: Evidence) -> NodeId {
        let parent = self.current();
        let id = self.allocate(
            Some(parent),
            NodeKind::Element {
                name,
                origin: Origin::Authored { complete, raw_name },
            },
        );
        self.open.push(id);
        id
    }

    fn close_p(&mut self, kind: PClosureKind, token_index: usize, trigger: Evidence) {
        assert!(self.current_is_p());
        let implied_end_pops = 0usize;
        assert_eq!(implied_end_pops, 0, "bounded close-P implied-end step");
        let target = self.current();
        self.open.pop();
        self.p_closures.push(PClosure {
            kind,
            target,
            token_index,
            trigger,
        });
    }

    fn synthesize_p_for_unmatched_end(&mut self, token_index: usize, trigger: Evidence) {
        assert!(!self.current_is_p());
        let parent = self.current();
        let id = self.allocate(
            Some(parent),
            NodeKind::Element {
                name: Name::P,
                origin: Origin::Synthesized(SynthesisCause::UnmatchedPEnd),
            },
        );
        self.open.push(id);
        self.p_syntheses.push(PSynthesis {
            node: id,
            token_index,
            trigger: trigger.clone(),
        });
        self.diagnostics.push(Diagnostic::UnmatchedPEnd);
        self.close_p(PClosureKind::UnmatchedEndSynthesized, token_index, trigger);
    }

    fn insert_text(&mut self, interpreted: &str, contribution: Evidence) {
        let parent = self.current();
        let last_direct_child = self
            .slots
            .iter()
            .flatten()
            .filter(|node| node.parent == Some(parent))
            .max_by_key(|node| node.id)
            .map(|node| node.id);
        let adjacent =
            last_direct_child.filter(|id| matches!(self.node(*id).kind, NodeKind::Text { .. }));

        if let Some(id) = adjacent
            && let NodeKind::Text {
                interpreted: existing,
                contributions,
            } = &mut self.node_mut(id).kind
        {
            existing.push_str(interpreted);
            contributions.push(contribution);
            return;
        }

        self.allocate(
            Some(parent),
            NodeKind::Text {
                interpreted: interpreted.to_owned(),
                contributions: vec![contribution],
            },
        );
    }

    fn close_block(&mut self, name: Name, token_index: usize) {
        let Some(position) = self.open.iter().rposition(|id| self.name(*id) == name) else {
            self.diagnostics.push(Diagnostic::UnmatchedBlockEnd);
            return;
        };
        let target = self.open[position];
        let intervening: Vec<NodeId> = self.open[position + 1..].iter().rev().copied().collect();
        if !intervening.is_empty() {
            self.diagnostics.push(Diagnostic::MisnestedBlockEnd);
        }
        for popped in intervening {
            assert_eq!(self.current(), popped);
            self.open.pop();
            self.block_recovery.push(BlockRecovery {
                popped,
                target,
                token_index,
            });
        }
        assert_eq!(self.current(), target);
        self.open.pop();
    }

    fn process(&mut self, token_index: usize, token: &HtmlToken) -> Result<bool, Unsupported> {
        self.assert_invariant();
        let result = match self.phase {
            Phase::BeforeBody => self.process_before_body(token),
            Phase::InBody => self.process_in_body(token_index, token),
            Phase::AfterBody => self.process_after_body(token),
        };
        if result.is_ok() {
            self.assert_invariant();
        }
        result
    }

    fn process_before_body(&mut self, token: &HtmlToken) -> Result<bool, Unsupported> {
        let HtmlToken::Tag(tag) = token else {
            return Err(Unsupported::OutsideCandidate);
        };
        if tag.kind() != HtmlTagKind::Start || tag.name().interpreted() != "body" {
            return Err(Unsupported::OutsideCandidate);
        }
        if !tag.attributes().is_empty() || tag.self_closing_solidus().is_some() {
            return Err(Unsupported::OutsideCandidate);
        }
        let body = self.insert_authored(
            Name::Body,
            evidence(tag.complete()),
            evidence(tag.name().source()),
        );
        assert_eq!(self.current(), body);
        self.phase = Phase::InBody;
        Ok(false)
    }

    fn process_in_body(
        &mut self,
        token_index: usize,
        token: &HtmlToken,
    ) -> Result<bool, Unsupported> {
        match token {
            HtmlToken::Character(character) => {
                self.insert_text(character.interpreted(), evidence(character.source()));
                Ok(false)
            }
            HtmlToken::EndOfFile(_) => {
                if self.has_open_block() {
                    self.diagnostics.push(Diagnostic::OpenBlockAtEof);
                }
                Ok(true)
            }
            HtmlToken::Tag(tag) => {
                let name = interpreted_name(tag.name().interpreted())?;
                reject_tag_shape(name, tag)?;
                match (tag.kind(), name) {
                    (HtmlTagKind::Start, Name::P) => {
                        if self.current_is_p() {
                            self.close_p(
                                PClosureKind::StartTriggered,
                                token_index,
                                evidence(tag.complete()),
                            );
                        }
                        self.insert_authored(
                            Name::P,
                            evidence(tag.complete()),
                            evidence(tag.name().source()),
                        );
                        Ok(false)
                    }
                    (HtmlTagKind::Start, name) if name.is_block() => {
                        if self.current_is_p() {
                            self.close_p(
                                PClosureKind::StartTriggered,
                                token_index,
                                evidence(tag.complete()),
                            );
                        }
                        self.insert_authored(
                            name,
                            evidence(tag.complete()),
                            evidence(tag.name().source()),
                        );
                        Ok(false)
                    }
                    (HtmlTagKind::End, Name::P) => {
                        if self.current_is_p() {
                            self.close_p(
                                PClosureKind::MatchingEnd,
                                token_index,
                                evidence(tag.complete()),
                            );
                        } else {
                            self.synthesize_p_for_unmatched_end(
                                token_index,
                                evidence(tag.complete()),
                            );
                        }
                        Ok(false)
                    }
                    (HtmlTagKind::End, name) if name.is_block() => {
                        if self.current_is_p() {
                            return Err(Unsupported::BlockEndWithOpenP);
                        }
                        self.close_block(name, token_index);
                        Ok(false)
                    }
                    (HtmlTagKind::End, Name::Body) => {
                        if self.current_is_p() {
                            return Err(Unsupported::BodyEndWithOpenP);
                        }
                        if self.has_open_block() {
                            return Err(Unsupported::BodyEndWithOpenBlock);
                        }
                        self.phase = Phase::AfterBody;
                        Ok(false)
                    }
                    _ => Err(Unsupported::OutsideCandidate),
                }
            }
        }
    }

    fn process_after_body(&mut self, token: &HtmlToken) -> Result<bool, Unsupported> {
        match token {
            HtmlToken::EndOfFile(_) => Ok(true),
            HtmlToken::Tag(tag) if tag.name().interpreted() == "p" => {
                Err(Unsupported::POutsideInBody)
            }
            _ => Err(Unsupported::OutsideCandidate),
        }
    }

    fn nodes(&self) -> Vec<Node> {
        self.slots.iter().flatten().cloned().collect()
    }
}

fn interpreted_name(name: &str) -> Result<Name, Unsupported> {
    match name {
        "body" => Ok(Name::Body),
        "div" => Ok(Name::Div),
        "section" => Ok(Name::Section),
        "p" => Ok(Name::P),
        _ => Err(Unsupported::GenericTag),
    }
}

fn reject_tag_shape(name: Name, tag: &HtmlTagToken) -> Result<(), Unsupported> {
    if !tag.attributes().is_empty() {
        return Err(match (tag.kind(), name) {
            (HtmlTagKind::Start, Name::P) => Unsupported::PStartAttribute,
            (HtmlTagKind::End, Name::P) => Unsupported::PEndAttribute,
            (_, name) if name.is_block() => Unsupported::BlockAttribute,
            _ => Unsupported::OutsideCandidate,
        });
    }
    if tag.self_closing_solidus().is_some() {
        return Err(match name {
            Name::P => Unsupported::PSelfClosing,
            name if name.is_block() => Unsupported::BlockSelfClosing,
            _ => Unsupported::OutsideCandidate,
        });
    }
    Ok(())
}

fn token_end(token: &HtmlToken) -> usize {
    match token {
        HtmlToken::Character(character) => character.source().range().end(),
        HtmlToken::Tag(tag) => tag.complete().range().end(),
        HtmlToken::EndOfFile(eof) => eof.source().range().end(),
    }
}

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn tokenize_source(source: &str, source_id: u64) -> HtmlTokenizerRunResult {
    tokenize(
        &SourceText::new(SourceId::new(source_id), source.to_owned()),
        limits(),
    )
}

fn observe_with_layout(run: &HtmlTokenizerRunResult, layout: StorageLayout) -> Observation {
    let mut machine = Machine::new(layout);
    let mut refusal = None;
    let mut stopped = false;

    for (token_index, token) in run.tokens().iter().enumerate() {
        let before = machine.snapshot();
        match machine.process(token_index, token) {
            Ok(stop) => {
                machine.commit(token);
                if stop {
                    stopped = true;
                    break;
                }
            }
            Err(capability) => {
                let after = machine.snapshot();
                assert_eq!(before, after, "candidate refusal must be transactional");
                refusal = Some(RefusalRecord {
                    capability,
                    token_index,
                    before,
                    after,
                });
                break;
            }
        }
    }

    let completion = if let Some(ref refusal) = refusal {
        Completion::Unsupported {
            capability: refusal.capability,
            token_index: refusal.token_index,
        }
    } else if run.is_incomplete() {
        Completion::LowerLayerIncomplete
    } else if stopped && machine.processed_tokens == run.tokens().len() {
        Completion::Complete
    } else {
        Completion::LowerLayerIncomplete
    };

    Observation {
        nodes: machine.nodes(),
        diagnostics: machine.diagnostics,
        p_closures: machine.p_closures,
        p_syntheses: machine.p_syntheses,
        block_recovery: machine.block_recovery,
        open: machine.open,
        phase: machine.phase,
        completion,
        refusal,
    }
}

fn observe_source(source: &str, source_id: u64) -> Observation {
    observe_with_layout(&tokenize_source(source, source_id), StorageLayout::COMPACT)
}

fn observe_fixture(id: &str, source_id: u64) -> Observation {
    observe_source(fixture(id).source, source_id)
}

fn p_nodes(observation: &Observation) -> Vec<&Node> {
    observation
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, NodeKind::Element { name: Name::P, .. }))
        .collect()
}

fn text_nodes(observation: &Observation) -> Vec<&Node> {
    observation
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, NodeKind::Text { .. }))
        .collect()
}

fn assert_complete(observation: &Observation) {
    assert_eq!(observation.completion, Completion::Complete);
    assert!(observation.refusal.is_none());
}

fn assert_refusal(id: &str, capability: Unsupported) {
    let observation = observe_fixture(id, 1);
    assert_eq!(
        observation.completion,
        Completion::Unsupported {
            capability,
            token_index: observation
                .refusal
                .as_ref()
                .expect("refusal record")
                .token_index,
        }
    );
    let refusal = observation.refusal.expect("refusal record");
    assert_eq!(refusal.capability, capability);
    assert_eq!(refusal.before, refusal.after);
}

#[test]
fn authority_and_hand_authored_gold_matrix_are_frozen() {
    assert_eq!(
        PINNED_WHATWG_COMMIT,
        "508a037333d8a1806504303aeb489d931fabbef6"
    );
    assert_eq!(
        PINNED_WHATWG_BLOB,
        "68dbcb98bbe1001c6ae2531be2368c608fbafddd"
    );
    assert_eq!(FIXTURES.len(), 21);
    for (index, fixture) in FIXTURES.iter().enumerate() {
        assert_eq!(fixture.id, format!("P{}", index + 1));
        assert!(fixture.source.starts_with("<body>"));
    }
}

#[test]
fn validation_module_does_not_import_production_tree_semantics() {
    let source = include_str!("in_body_p_successor_validation.rs");
    let forbidden = [
        ["use super::", "driver"].concat(),
        ["use super::", "session"].concat(),
        ["use super::", "result"].concat(),
        ["tree_construction", "::driver"].concat(),
        ["tree_construction", "::session"].concat(),
        ["tree_construction", "::result"].concat(),
    ];
    for forbidden in forbidden {
        assert!(
            !source.contains(&forbidden),
            "forbidden oracle: {forbidden}"
        );
    }
}

#[test]
fn p1_authored_p_lifecycle_and_text_provenance_are_exact() {
    let observation = observe_fixture("P1", 11);
    assert_complete(&observation);
    let p = p_nodes(&observation)[0];
    let NodeKind::Element {
        origin: Origin::Authored { complete, raw_name },
        ..
    } = &p.kind
    else {
        panic!("authored P")
    };
    assert_eq!(*complete, expected_evidence(11, (6, 9)));
    assert_eq!(*raw_name, expected_evidence(11, (7, 8)));
    assert_eq!(observation.p_closures.len(), 1);
    assert_eq!(observation.p_closures[0].kind, PClosureKind::MatchingEnd);
    assert_eq!(observation.p_closures[0].token_index, 3);
    assert_eq!(
        observation.p_closures[0].trigger,
        expected_evidence(11, (10, 14))
    );
    let text = text_nodes(&observation)[0];
    let NodeKind::Text {
        interpreted,
        contributions,
    } = &text.kind
    else {
        panic!("text node")
    };
    assert_eq!(interpreted, "x");
    assert_eq!(contributions, &vec![expected_evidence(11, (9, 10))]);
}

#[test]
fn p2_interpreted_name_is_case_insensitive_but_raw_name_range_is_preserved() {
    let run = tokenize_source(fixture("P2").source, 12);
    let HtmlToken::Tag(start) = &run.tokens()[1] else {
        panic!("P2 start")
    };
    assert_eq!(start.name().interpreted(), "p");
    assert_eq!(evidence(start.complete()), expected_evidence(12, (6, 9)));
    assert_eq!(
        evidence(start.name().source()),
        expected_evidence(12, (7, 8))
    );
    assert_complete(&observe_with_layout(&run, StorageLayout::COMPACT));
}

#[test]
fn p3_second_p_start_closes_first_before_allocating_second() {
    let observation = observe_fixture("P3", 1);
    assert_complete(&observation);
    let ps = p_nodes(&observation);
    assert_eq!(ps.len(), 2);
    assert_ne!(ps[0].id, ps[1].id);
    assert_eq!(observation.p_closures.len(), 2);
    assert_eq!(observation.p_closures[0].kind, PClosureKind::StartTriggered);
    assert_eq!(observation.p_closures[0].target, ps[0].id);
    assert_eq!(observation.p_closures[0].trigger.range, (10, 13));
    assert_eq!(observation.p_closures[1].kind, PClosureKind::MatchingEnd);
    assert_eq!(observation.p_closures[1].target, ps[1].id);
}

#[test]
fn p4_p5_block_start_closes_p_without_tc_s4_ancestor_end_recovery() {
    for id in ["P4", "P5"] {
        let observation = observe_fixture(id, 1);
        assert_complete(&observation);
        assert_eq!(observation.p_closures.len(), 1, "{id}");
        assert_eq!(
            observation.p_closures[0].kind,
            PClosureKind::StartTriggered,
            "{id}"
        );
        assert!(observation.block_recovery.is_empty(), "{id}");
        assert_eq!(
            observation
                .diagnostics
                .iter()
                .filter(|diagnostic| **diagnostic == Diagnostic::UnmatchedPEnd)
                .count(),
            0,
            "{id}"
        );
    }
}

#[test]
fn p6_matching_end_closes_nested_p_without_closing_parent_block() {
    let observation = observe_fixture("P6", 1);
    assert_complete(&observation);
    assert_eq!(observation.p_closures.len(), 1);
    assert_eq!(observation.p_closures[0].kind, PClosureKind::MatchingEnd);
    assert!(observation.block_recovery.is_empty());
}

#[test]
fn p7_unmatched_end_synthesizes_source_less_p_then_closes_it() {
    let observation = observe_fixture("P7", 21);
    assert_complete(&observation);
    assert_eq!(observation.p_syntheses.len(), 1);
    assert_eq!(observation.p_syntheses[0].token_index, 1);
    assert_eq!(observation.p_closures.len(), 1);
    assert_eq!(
        observation.p_closures[0].kind,
        PClosureKind::UnmatchedEndSynthesized
    );
    assert_eq!(
        observation
            .diagnostics
            .iter()
            .filter(|diagnostic| **diagnostic == Diagnostic::UnmatchedPEnd)
            .count(),
        1
    );
    let p = p_nodes(&observation)[0];
    assert_eq!(p.id, observation.p_syntheses[0].node);
    assert!(matches!(
        p.kind,
        NodeKind::Element {
            origin: Origin::Synthesized(SynthesisCause::UnmatchedPEnd),
            ..
        }
    ));
    assert_eq!(observation.p_syntheses[0].trigger.range, (6, 10));
}

#[test]
fn p8_synthesized_p_is_placed_under_actual_current_block() {
    let observation = observe_fixture("P8", 1);
    assert_complete(&observation);
    let synthesized = observation.p_syntheses[0].node;
    let p = observation
        .nodes
        .iter()
        .find(|node| node.id == synthesized)
        .expect("synthesized P");
    let parent = p.parent.expect("P parent");
    let parent_node = observation
        .nodes
        .iter()
        .find(|node| node.id == parent)
        .expect("P parent node");
    assert!(matches!(
        parent_node.kind,
        NodeKind::Element {
            name: Name::Div,
            ..
        }
    ));
}

#[test]
fn p9_repeated_stray_end_tags_create_distinct_synthesized_identities() {
    let observation = observe_fixture("P9", 1);
    assert_complete(&observation);
    assert_eq!(observation.p_syntheses.len(), 2);
    assert_ne!(
        observation.p_syntheses[0].node,
        observation.p_syntheses[1].node
    );
    assert_eq!(
        observation
            .diagnostics
            .iter()
            .filter(|diagnostic| **diagnostic == Diagnostic::UnmatchedPEnd)
            .count(),
        2
    );
    assert_eq!(
        observation
            .p_closures
            .iter()
            .filter(|closure| closure.kind == PClosureKind::UnmatchedEndSynthesized)
            .count(),
        2
    );
}

#[test]
fn p10_p_only_eof_leaves_p_open_without_p_diagnostic_or_closure() {
    let observation = observe_fixture("P10", 1);
    assert_complete(&observation);
    assert_eq!(
        observation.open.last().map(|id| {
            observation
                .nodes
                .iter()
                .find(|node| node.id == *id)
                .and_then(|node| match node.kind {
                    NodeKind::Element { name, .. } => Some(name),
                    _ => None,
                })
        }),
        Some(Some(Name::P))
    );
    assert!(observation.p_closures.is_empty());
    assert!(observation.p_syntheses.is_empty());
    assert!(!observation.diagnostics.contains(&Diagnostic::UnmatchedPEnd));
    assert!(
        !observation
            .diagnostics
            .contains(&Diagnostic::OpenBlockAtEof)
    );
}

#[test]
fn p11_open_block_eof_diagnostic_remains_distinct_from_p() {
    let observation = observe_fixture("P11", 1);
    assert_complete(&observation);
    assert!(
        observation
            .diagnostics
            .contains(&Diagnostic::OpenBlockAtEof)
    );
    assert!(!observation.diagnostics.contains(&Diagnostic::UnmatchedPEnd));
    assert!(observation.p_closures.is_empty());
}

#[test]
fn p12_p13_non_noop_implied_end_cells_refuse_before_mutation() {
    assert_refusal("P12", Unsupported::BlockEndWithOpenP);
    assert_refusal("P13", Unsupported::BlockEndWithOpenP);
}

#[test]
fn p14_p15_p16_p17_and_p21_shape_and_crossing_exclusions_are_transactional() {
    assert_refusal("P14", Unsupported::PStartAttribute);
    assert_refusal("P15", Unsupported::PSelfClosing);
    assert_refusal("P16", Unsupported::POutsideInBody);
    assert_refusal("P17", Unsupported::BodyEndWithOpenP);
    assert_refusal("P21", Unsupported::PEndAttribute);
}

#[test]
fn p18_predecessor_normal_div_section_lifecycle_has_no_p_delta() {
    let observation = observe_fixture("P18", 1);
    assert_complete(&observation);
    assert!(observation.p_closures.is_empty());
    assert!(observation.p_syntheses.is_empty());
    assert!(observation.block_recovery.is_empty());
    assert_eq!(observation.phase, Phase::InBody);
}

#[test]
fn p19_predecessor_heterogeneous_recovery_remains_separate() {
    let observation = observe_fixture("P19", 1);
    assert_complete(&observation);
    assert!(observation.p_closures.is_empty());
    assert!(observation.p_syntheses.is_empty());
    assert_eq!(observation.block_recovery.len(), 1);
    let recovery = &observation.block_recovery[0];
    assert_ne!(recovery.popped, recovery.target);
    assert_eq!(recovery.token_index, 3);
    assert!(
        observation
            .diagnostics
            .contains(&Diagnostic::MisnestedBlockEnd)
    );
}

#[test]
fn p20_source_id_changes_provenance_not_constructed_identity() {
    let first = observe_fixture("P20", 7);
    let second = observe_fixture("P20", 9_999);
    assert_complete(&first);
    assert_complete(&second);
    assert_eq!(p_nodes(&first)[0].id, p_nodes(&second)[0].id);

    let NodeKind::Element {
        origin: Origin::Authored { complete: a, .. },
        ..
    } = &p_nodes(&first)[0].kind
    else {
        panic!("authored P")
    };
    let NodeKind::Element {
        origin: Origin::Authored { complete: b, .. },
        ..
    } = &p_nodes(&second)[0].kind
    else {
        panic!("authored P")
    };
    assert_eq!(a.range, b.range);
    assert_ne!(a.source_id, b.source_id);
}

#[test]
fn semantic_identity_is_independent_from_private_storage_padding() {
    for id in ["P3", "P7", "P8", "P9", "P19"] {
        let run = tokenize_source(fixture(id).source, 1);
        let compact = observe_with_layout(&run, StorageLayout::COMPACT);
        let padded = observe_with_layout(&run, StorageLayout::PADDED);
        assert_eq!(compact.nodes, padded.nodes, "{id}");
        assert_eq!(compact.p_closures, padded.p_closures, "{id}");
        assert_eq!(compact.p_syntheses, padded.p_syntheses, "{id}");
        assert_eq!(compact.block_recovery, padded.block_recovery, "{id}");
    }
}

#[test]
fn lower_layer_incompleteness_is_never_upgraded_to_complete() {
    let source = SourceText::new(SourceId::new(1), "<body><p>xxxxxxxx".to_owned());
    let run = tokenize(&source, HtmlTokenizerLimits::new(1, 1, 1, 1, 1, 1, 1));
    assert!(run.is_incomplete());
    let observation = observe_with_layout(&run, StorageLayout::COMPACT);
    assert_ne!(observation.completion, Completion::Complete);
}

#[test]
fn bounded_generated_sequences_preserve_invariants_or_refuse_cleanly() {
    const PIECES: [&str; 7] = [
        "<p>",
        "</p>",
        "<div>",
        "</div>",
        "<section>",
        "</section>",
        "x",
    ];

    for length in 0_u32..=4 {
        for code in 0..PIECES.len().pow(length) {
            let mut digits = Vec::new();
            let mut remaining = code;
            for _ in 0..length {
                digits.push(remaining % PIECES.len());
                remaining /= PIECES.len();
            }
            digits.reverse();

            let mut source = String::from("<body>");
            for digit in digits {
                source.push_str(PIECES[digit]);
            }

            let observation = observe_source(&source, 1);
            if let Some(ref refusal) = observation.refusal {
                assert_eq!(refusal.before, refusal.after, "{source}");
            }

            let unmatched_diagnostics = observation
                .diagnostics
                .iter()
                .filter(|diagnostic| **diagnostic == Diagnostic::UnmatchedPEnd)
                .count();
            let unmatched_closures = observation
                .p_closures
                .iter()
                .filter(|closure| closure.kind == PClosureKind::UnmatchedEndSynthesized)
                .count();
            assert_eq!(
                unmatched_diagnostics,
                observation.p_syntheses.len(),
                "{source}"
            );
            assert_eq!(
                unmatched_closures,
                observation.p_syntheses.len(),
                "{source}"
            );
        }
    }
}
