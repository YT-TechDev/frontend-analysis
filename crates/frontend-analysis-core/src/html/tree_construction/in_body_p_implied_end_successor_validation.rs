//! Candidate-independent TC-S6 successor validation.
//!
//! TC-S6 is the bounded successor theorem "Selected In-Body `div` / `section`
//! End Tags over Current P with Bounded Non-Noop Implied-End Handling" from
//! Issue #369. This module consumes the accepted tokenizer only as lower-layer
//! token/source evidence. It does not consume production tree-construction
//! semantics as an oracle.
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
//! TC-S6 proves only the selected-end cell where current P makes the Standard's
//! implied-end step materially mutate the stack. It is not a generic scope
//! engine and not a generic implied-end implementation.

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
    Fixture { id: "I1", source: "<body><div><p>x</div>" },
    Fixture { id: "I2", source: "<body><section><p>x</section>" },
    Fixture { id: "I3", source: "<body><div><p>x</DiV>" },
    Fixture { id: "I4", source: "<body><div><section><p>x</div>" },
    Fixture { id: "I5", source: "<body><section><div><p>x</section>" },
    Fixture { id: "I6", source: "<body><div><section><div><p>x</section>" },
    Fixture { id: "I7", source: "<body><div><p>x</section>" },
    Fixture { id: "I8", source: "<body><p>x</div>" },
    Fixture { id: "I9", source: "<body><div><p>x</section></section>" },
    Fixture { id: "I10", source: "<body><div><p>x</div></p>" },
    Fixture { id: "I11", source: "<body><div><p>x</div>y" },
    Fixture { id: "I12", source: "<body><div><p>x</section>" },
    Fixture { id: "I13", source: "<body><div><p>x</div id=x>" },
    Fixture { id: "I14", source: "<body><div><p>x</div/>" },
    Fixture { id: "I15", source: "<body></body></div>" },
    Fixture { id: "I16", source: "<body><p></body>" },
    Fixture { id: "I17", source: "<body><div><section></div>" },
    Fixture { id: "I18", source: "<body><p>a<p>b</p>" },
    Fixture { id: "I19", source: "<body><DiV><P>x</dIv>" },
    Fixture { id: "I20", source: "<body><div><p>x</div><section>y</section>" },
    Fixture { id: "I21", source: "<body><div><section><p>x</div>" },
    Fixture { id: "I23", source: "<body><div><p>x</div id=x>" },
];

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .expect("canonical TC-S6 fixture")
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
enum DiagnosticKind {
    MissingDoctype,
    UnmatchedPEnd,
    UnmatchedBlockEnd,
    MisnestedBlockEnd,
    OpenBlockAtEof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    kind: DiagnosticKind,
    trigger: Option<Evidence>,
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
struct ImpliedPPop {
    paragraph: NodeId,
    selected_target: NodeId,
    token_index: usize,
    trigger: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockRecovery {
    popped: NodeId,
    target: NodeId,
    token_index: usize,
    trigger: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    AuthoredInsert {
        node: NodeId,
        name: Name,
        trigger: Evidence,
    },
    PDiagnostic {
        trigger: Evidence,
    },
    SynthesizedP {
        node: NodeId,
        trigger: Evidence,
    },
    PClose {
        kind: PClosureKind,
        target: NodeId,
        trigger: Evidence,
    },
    ImpliedPPop {
        paragraph: NodeId,
        selected_target: NodeId,
        trigger: Evidence,
    },
    BlockDiagnostic {
        kind: DiagnosticKind,
        name: Name,
        trigger: Evidence,
    },
    BlockIgnored {
        name: Name,
        trigger: Evidence,
    },
    BlockRecoveryPop {
        popped: NodeId,
        target: NodeId,
        trigger: Evidence,
    },
    BlockClose {
        target: NodeId,
        name: Name,
        trigger: Evidence,
    },
    TextInsert {
        node: NodeId,
        parent: NodeId,
        contribution: Evidence,
    },
    TextAppend {
        node: NodeId,
        parent: NodeId,
        contribution: Evidence,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unsupported {
    PStartAttribute,
    PEndAttribute,
    PSelfClosing,
    POutsideInBody,
    SelectedEndOutsideInBody,
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
struct Fingerprint {
    nodes: Vec<Node>,
    open: Vec<NodeId>,
    phase: Phase,
    next_id: usize,
    committed_end: usize,
    processed_tokens: usize,
    diagnostics: Vec<Diagnostic>,
    p_closures: Vec<PClosure>,
    p_syntheses: Vec<PSynthesis>,
    implied_p_pops: Vec<ImpliedPPop>,
    block_recovery: Vec<BlockRecovery>,
    actions: Vec<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RefusalRecord {
    capability: Unsupported,
    token_index: usize,
    before: Fingerprint,
    after: Fingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    nodes: Vec<Node>,
    diagnostics: Vec<Diagnostic>,
    p_closures: Vec<PClosure>,
    p_syntheses: Vec<PSynthesis>,
    implied_p_pops: Vec<ImpliedPPop>,
    block_recovery: Vec<BlockRecovery>,
    actions: Vec<Action>,
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
        leading_padding: 5,
        inter_node_padding: 3,
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
    implied_p_pops: Vec<ImpliedPPop>,
    block_recovery: Vec<BlockRecovery>,
    actions: Vec<Action>,
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
            diagnostics: vec![Diagnostic {
                kind: DiagnosticKind::MissingDoctype,
                trigger: None,
            }],
            p_closures: Vec::new(),
            p_syntheses: Vec::new(),
            implied_p_pops: Vec::new(),
            block_recovery: Vec::new(),
            actions: Vec::new(),
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

    fn nodes(&self) -> Vec<Node> {
        self.slots.iter().flatten().cloned().collect()
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

    fn open_names(&self) -> Vec<Name> {
        self.open.iter().map(|id| self.name(*id)).collect()
    }

    fn fingerprint(&self) -> Fingerprint {
        Fingerprint {
            nodes: self.nodes(),
            open: self.open.clone(),
            phase: self.phase,
            next_id: self.next_id,
            committed_end: self.committed_end,
            processed_tokens: self.processed_tokens,
            diagnostics: self.diagnostics.clone(),
            p_closures: self.p_closures.clone(),
            p_syntheses: self.p_syntheses.clone(),
            implied_p_pops: self.implied_p_pops.clone(),
            block_recovery: self.block_recovery.clone(),
            actions: self.actions.clone(),
        }
    }

    fn assert_invariant(&self) {
        let names = self.open_names();
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
            "TC-S6 stack invariant violated: phase={:?} names={names:?}",
            self.phase
        );
        let p_present = names.contains(&Name::P);
        assert_eq!(p_present, names.last() == Some(&Name::P));
    }

    fn commit(&mut self, token: &HtmlToken) {
        self.committed_end = token_end(token);
        self.processed_tokens += 1;
    }

    fn push_diagnostic(&mut self, kind: DiagnosticKind, trigger: Option<Evidence>) {
        self.diagnostics.push(Diagnostic { kind, trigger });
    }

    fn insert_authored(&mut self, name: Name, complete: Evidence, raw_name: Evidence) -> NodeId {
        let parent = self.current();
        let id = self.allocate(
            Some(parent),
            NodeKind::Element {
                name,
                origin: Origin::Authored {
                    complete: complete.clone(),
                    raw_name,
                },
            },
        );
        self.open.push(id);
        self.actions.push(Action::AuthoredInsert {
            node: id,
            name,
            trigger: complete,
        });
        id
    }

    fn close_p(&mut self, kind: PClosureKind, token_index: usize, trigger: Evidence) {
        assert!(self.current_is_p());
        let target = self.current();
        self.open.pop();
        self.p_closures.push(PClosure {
            kind,
            target,
            token_index,
            trigger: trigger.clone(),
        });
        self.actions.push(Action::PClose {
            kind,
            target,
            trigger,
        });
    }

    fn synthesize_p_for_unmatched_end(&mut self, token_index: usize, trigger: Evidence) {
        assert!(!self.current_is_p());
        self.push_diagnostic(DiagnosticKind::UnmatchedPEnd, Some(trigger.clone()));
        self.actions.push(Action::PDiagnostic {
            trigger: trigger.clone(),
        });

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
        self.actions.push(Action::SynthesizedP {
            node: id,
            trigger: trigger.clone(),
        });
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
            contributions.push(contribution.clone());
            self.actions.push(Action::TextAppend {
                node: id,
                parent,
                contribution,
            });
            return;
        }

        let id = self.allocate(
            Some(parent),
            NodeKind::Text {
                interpreted: interpreted.to_owned(),
                contributions: vec![contribution.clone()],
            },
        );
        self.actions.push(Action::TextInsert {
            node: id,
            parent,
            contribution,
        });
    }

    fn close_block(&mut self, name: Name, token_index: usize, trigger: Evidence) {
        let target_position = self.open.iter().rposition(|id| self.name(*id) == name);

        let Some(position) = target_position else {
            self.push_diagnostic(DiagnosticKind::UnmatchedBlockEnd, Some(trigger.clone()));
            self.actions.push(Action::BlockDiagnostic {
                kind: DiagnosticKind::UnmatchedBlockEnd,
                name,
                trigger: trigger.clone(),
            });
            self.actions.push(Action::BlockIgnored { name, trigger });
            return;
        };

        let target = self.open[position];

        if self.current_is_p() {
            let paragraph = self.current();
            self.open.pop();
            self.implied_p_pops.push(ImpliedPPop {
                paragraph,
                selected_target: target,
                token_index,
                trigger: trigger.clone(),
            });
            self.actions.push(Action::ImpliedPPop {
                paragraph,
                selected_target: target,
                trigger: trigger.clone(),
            });
        }

        let intervening: Vec<NodeId> = self.open[position + 1..].iter().rev().copied().collect();
        if !intervening.is_empty() {
            self.push_diagnostic(DiagnosticKind::MisnestedBlockEnd, Some(trigger.clone()));
            self.actions.push(Action::BlockDiagnostic {
                kind: DiagnosticKind::MisnestedBlockEnd,
                name,
                trigger: trigger.clone(),
            });
        }

        for popped in intervening {
            assert_eq!(self.current(), popped);
            self.open.pop();
            self.block_recovery.push(BlockRecovery {
                popped,
                target,
                token_index,
                trigger: trigger.clone(),
            });
            self.actions.push(Action::BlockRecoveryPop {
                popped,
                target,
                trigger: trigger.clone(),
            });
        }

        assert_eq!(self.current(), target);
        self.open.pop();
        self.actions.push(Action::BlockClose {
            target,
            name,
            trigger,
        });
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
            HtmlToken::EndOfFile(eof) => {
                if self.has_open_block() {
                    self.push_diagnostic(
                        DiagnosticKind::OpenBlockAtEof,
                        Some(evidence(eof.source())),
                    );
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
                        self.close_block(name, token_index, evidence(tag.complete()));
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
            HtmlToken::Tag(tag)
                if matches!(tag.name().interpreted(), "div" | "section")
                    && tag.kind() == HtmlTagKind::End =>
            {
                Err(Unsupported::SelectedEndOutsideInBody)
            }
            _ => Err(Unsupported::OutsideCandidate),
        }
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
        let before = machine.fingerprint();
        match machine.process(token_index, token) {
            Ok(stop) => {
                machine.commit(token);
                if stop {
                    stopped = true;
                    break;
                }
            }
            Err(capability) => {
                let after = machine.fingerprint();
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
        implied_p_pops: machine.implied_p_pops,
        block_recovery: machine.block_recovery,
        actions: machine.actions,
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

fn node_name(observation: &Observation, id: NodeId) -> Name {
    let node = observation
        .nodes
        .iter()
        .find(|node| node.id == id)
        .expect("observation node");
    match node.kind {
        NodeKind::Element { name, .. } => name,
        _ => panic!("element name"),
    }
}

fn nodes_named(observation: &Observation, name: Name) -> Vec<&Node> {
    observation
        .nodes
        .iter()
        .filter(
            |node| matches!(node.kind, NodeKind::Element { name: actual, .. } if actual == name),
        )
        .collect()
}

fn text_nodes(observation: &Observation) -> Vec<&Node> {
    observation
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, NodeKind::Text { .. }))
        .collect()
}

fn diagnostic_kinds(observation: &Observation) -> Vec<DiagnosticKind> {
    observation
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.kind)
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

fn trigger_actions<'a>(observation: &'a Observation, trigger: &Evidence) -> Vec<&'a Action> {
    observation
        .actions
        .iter()
        .filter(|action| match action {
            Action::AuthoredInsert {
                trigger: actual, ..
            }
            | Action::PDiagnostic { trigger: actual }
            | Action::SynthesizedP {
                trigger: actual, ..
            }
            | Action::PClose {
                trigger: actual, ..
            }
            | Action::ImpliedPPop {
                trigger: actual, ..
            }
            | Action::BlockDiagnostic {
                trigger: actual, ..
            }
            | Action::BlockIgnored {
                trigger: actual, ..
            }
            | Action::BlockRecoveryPop {
                trigger: actual, ..
            }
            | Action::BlockClose {
                trigger: actual, ..
            } => actual == trigger,
            Action::TextInsert { contribution, .. } | Action::TextAppend { contribution, .. } => {
                contribution == trigger
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClosedFormOutcome {
    unmatched: bool,
    implied_p_pop: bool,
    misnested: bool,
    recovery_names_current_first: Vec<Name>,
    closes_target: bool,
    final_blocks: Vec<Name>,
    final_p: bool,
}

fn closed_form_oracle(blocks: &[Name], p_current: bool, end: Name) -> ClosedFormOutcome {
    assert!(end.is_block());
    assert!(blocks.iter().all(|name| name.is_block()));

    let Some(position) = blocks.iter().rposition(|name| *name == end) else {
        return ClosedFormOutcome {
            unmatched: true,
            implied_p_pop: false,
            misnested: false,
            recovery_names_current_first: Vec::new(),
            closes_target: false,
            final_blocks: blocks.to_vec(),
            final_p: p_current,
        };
    };

    let recovery_names_current_first: Vec<Name> =
        blocks[position + 1..].iter().rev().copied().collect();
    ClosedFormOutcome {
        unmatched: false,
        implied_p_pop: p_current,
        misnested: !recovery_names_current_first.is_empty(),
        recovery_names_current_first,
        closes_target: true,
        final_blocks: blocks[..position].to_vec(),
        final_p: false,
    }
}

fn machine_cell_outcome(blocks: &[Name], p_current: bool, end: Name) -> ClosedFormOutcome {
    let mut source = String::from("<body>");
    for name in blocks {
        source.push_str(match name {
            Name::Div => "<div>",
            Name::Section => "<section>",
            _ => panic!("generated block"),
        });
    }
    if p_current {
        source.push_str("<p>");
    }
    let trigger_start = source.len();
    source.push_str(match end {
        Name::Div => "</div>",
        Name::Section => "</section>",
        _ => panic!("generated end"),
    });
    let trigger_end = source.len();

    let run = tokenize_source(&source, 1);
    let mut machine = Machine::new(StorageLayout::COMPACT);
    let mut trigger = None;
    for (token_index, token) in run.tokens().iter().enumerate() {
        if matches!(token, HtmlToken::EndOfFile(_)) {
            break;
        }
        let before = machine.fingerprint();
        let stop = machine
            .process(token_index, token)
            .unwrap_or_else(|capability| {
                panic!("generated cell refused: {source:?} {capability:?}")
            });
        assert!(!stop);
        machine.commit(token);
        assert_ne!(before.processed_tokens, machine.processed_tokens);
        if let HtmlToken::Tag(tag) = token
            && tag.kind() == HtmlTagKind::End
            && tag.complete().range().start() == trigger_start
        {
            trigger = Some(evidence(tag.complete()));
            break;
        }
    }

    let trigger = trigger.expect("generated selected end trigger");
    assert_eq!(trigger.range, (trigger_start, trigger_end));
    let actions: Vec<&Action> = machine
        .actions
        .iter()
        .filter(|action| match action {
            Action::ImpliedPPop {
                trigger: actual, ..
            }
            | Action::BlockDiagnostic {
                trigger: actual, ..
            }
            | Action::BlockIgnored {
                trigger: actual, ..
            }
            | Action::BlockRecoveryPop {
                trigger: actual, ..
            }
            | Action::BlockClose {
                trigger: actual, ..
            } => actual == &trigger,
            _ => false,
        })
        .collect();

    let unmatched = actions
        .iter()
        .any(|action| matches!(action, Action::BlockIgnored { .. }));
    let implied_p_pop = actions
        .iter()
        .any(|action| matches!(action, Action::ImpliedPPop { .. }));
    let misnested = actions.iter().any(|action| {
        matches!(
            action,
            Action::BlockDiagnostic {
                kind: DiagnosticKind::MisnestedBlockEnd,
                ..
            }
        )
    });
    let recovery_names_current_first = actions
        .iter()
        .filter_map(|action| match action {
            Action::BlockRecoveryPop { popped, .. } => Some(machine.name(*popped)),
            _ => None,
        })
        .collect();
    let closes_target = actions
        .iter()
        .any(|action| matches!(action, Action::BlockClose { .. }));
    let open_names = machine.open_names();
    let final_p = open_names.last() == Some(&Name::P);
    let final_blocks = open_names
        .into_iter()
        .filter(|name| name.is_block())
        .collect();

    ClosedFormOutcome {
        unmatched,
        implied_p_pop,
        misnested,
        recovery_names_current_first,
        closes_target,
        final_blocks,
        final_p,
    }
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
    assert_eq!(FIXTURES.len(), 22);
    for fixture in FIXTURES {
        assert!(fixture.id.starts_with('I'));
        assert!(fixture.source.starts_with("<body>"));
    }
}

#[test]
fn validation_module_does_not_import_production_tree_semantics() {
    let source = include_str!("in_body_p_implied_end_successor_validation.rs");
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
fn i1_i2_target_current_after_implied_pop_has_exact_order_and_no_misnested_diagnostic() {
    for (id, name, trigger_range) in [("I1", Name::Div, (15, 21)), ("I2", Name::Section, (19, 29))]
    {
        let observation = observe_fixture(id, 11);
        assert_complete(&observation);
        assert_eq!(
            diagnostic_kinds(&observation),
            vec![DiagnosticKind::MissingDoctype],
            "{id}"
        );
        assert_eq!(observation.implied_p_pops.len(), 1, "{id}");
        assert!(observation.block_recovery.is_empty(), "{id}");
        assert!(observation.p_closures.is_empty(), "{id}");

        let implied = &observation.implied_p_pops[0];
        assert_eq!(
            implied.trigger,
            expected_evidence(11, trigger_range),
            "{id}"
        );
        assert_eq!(node_name(&observation, implied.paragraph), Name::P, "{id}");
        assert_eq!(
            node_name(&observation, implied.selected_target),
            name,
            "{id}"
        );

        let actions = trigger_actions(&observation, &implied.trigger);
        assert_eq!(actions.len(), 2, "{id}");
        assert!(
            matches!(actions[0], Action::ImpliedPPop { paragraph, selected_target, .. }
            if *paragraph == implied.paragraph && *selected_target == implied.selected_target),
            "{id}"
        );
        assert!(
            matches!(actions[1], Action::BlockClose { target, name: actual, .. }
            if *target == implied.selected_target && *actual == name),
            "{id}"
        );
    }
}

#[test]
fn i3_case_insensitive_semantics_retain_exact_raw_end_tag_evidence() {
    let source = fixture("I3").source;
    let run = tokenize_source(source, 23);
    let end = run
        .tokens()
        .iter()
        .find_map(|token| match token {
            HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::End => Some(tag),
            _ => None,
        })
        .expect("I3 end tag");
    assert_eq!(end.name().interpreted(), "div");
    assert_eq!(evidence(end.complete()), expected_evidence(23, (15, 21)));
    let raw = evidence(end.name().source());
    assert_eq!(raw, expected_evidence(23, (17, 20)));
    assert_eq!(&source[raw.range.0..raw.range.1], "DiV");

    let observation = observe_with_layout(&run, StorageLayout::COMPACT);
    assert_complete(&observation);
    assert_eq!(observation.implied_p_pops.len(), 1);
    assert_eq!(
        observation.implied_p_pops[0].trigger,
        evidence(end.complete())
    );
}

#[test]
fn i4_i5_noncurrent_target_orders_implied_pop_diagnostic_recovery_then_close() {
    for (id, target_name, recovered_name, trigger_range) in [
        ("I4", Name::Div, Name::Section, (24, 30)),
        ("I5", Name::Section, Name::Div, (24, 34)),
    ] {
        let observation = observe_fixture(id, 31);
        assert_complete(&observation);
        assert_eq!(
            diagnostic_kinds(&observation),
            vec![
                DiagnosticKind::MissingDoctype,
                DiagnosticKind::MisnestedBlockEnd
            ],
            "{id}"
        );
        assert_eq!(observation.implied_p_pops.len(), 1, "{id}");
        assert_eq!(observation.block_recovery.len(), 1, "{id}");
        let implied = &observation.implied_p_pops[0];
        assert_eq!(
            implied.trigger,
            expected_evidence(31, trigger_range),
            "{id}"
        );
        assert_eq!(
            node_name(&observation, implied.selected_target),
            target_name,
            "{id}"
        );
        assert_eq!(
            node_name(&observation, observation.block_recovery[0].popped),
            recovered_name,
            "{id}"
        );
        assert_eq!(
            observation.block_recovery[0].target, implied.selected_target,
            "{id}"
        );

        let actions = trigger_actions(&observation, &implied.trigger);
        assert_eq!(actions.len(), 4, "{id}");
        assert!(matches!(actions[0], Action::ImpliedPPop { .. }), "{id}");
        assert!(
            matches!(
                actions[1],
                Action::BlockDiagnostic {
                    kind: DiagnosticKind::MisnestedBlockEnd,
                    ..
                }
            ),
            "{id}"
        );
        assert!(
            matches!(actions[2], Action::BlockRecoveryPop { .. }),
            "{id}"
        );
        assert!(matches!(actions[3], Action::BlockClose { .. }), "{id}");
    }
}

#[test]
fn i6_nearest_same_name_target_controls_exact_recovery_suffix() {
    let observation = observe_fixture("I6", 41);
    assert_complete(&observation);
    assert_eq!(observation.implied_p_pops.len(), 1);
    assert_eq!(observation.block_recovery.len(), 1);
    let implied = &observation.implied_p_pops[0];
    assert_eq!(
        node_name(&observation, implied.selected_target),
        Name::Section
    );
    assert_eq!(
        node_name(&observation, observation.block_recovery[0].popped),
        Name::Div
    );
    let remaining: Vec<Name> = observation
        .open
        .iter()
        .map(|id| node_name(&observation, *id))
        .collect();
    assert_eq!(remaining, vec![Name::Html, Name::Body, Name::Div]);
}

#[test]
fn i7_i8_target_absence_is_resolved_before_any_implied_p_mutation() {
    for (id, name) in [("I7", Name::Section), ("I8", Name::Div)] {
        let observation = observe_fixture(id, 51);
        assert_complete(&observation);
        assert!(observation.implied_p_pops.is_empty(), "{id}");
        assert!(observation.block_recovery.is_empty(), "{id}");
        assert!(observation.p_closures.is_empty(), "{id}");
        assert_eq!(
            diagnostic_kinds(&observation),
            if id == "I7" {
                vec![
                    DiagnosticKind::MissingDoctype,
                    DiagnosticKind::UnmatchedBlockEnd,
                    DiagnosticKind::OpenBlockAtEof,
                ]
            } else {
                vec![
                    DiagnosticKind::MissingDoctype,
                    DiagnosticKind::UnmatchedBlockEnd,
                ]
            },
            "{id}"
        );
        let unmatched = observation
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.kind == DiagnosticKind::UnmatchedBlockEnd)
            .and_then(|diagnostic| diagnostic.trigger.clone())
            .expect("unmatched trigger");
        let actions = trigger_actions(&observation, &unmatched);
        assert_eq!(actions.len(), 2, "{id}");
        assert!(matches!(
            actions[0],
            Action::BlockDiagnostic {
                kind: DiagnosticKind::UnmatchedBlockEnd,
                name: actual,
                ..
            } if *actual == name
        ));
        assert!(matches!(actions[1], Action::BlockIgnored { name: actual, .. } if *actual == name));
        assert_eq!(
            node_name(&observation, *observation.open.last().unwrap()),
            Name::P
        );
    }
}

#[test]
fn i9_repeated_unmatched_selected_ends_each_ignore_without_popping_p() {
    let observation = observe_fixture("I9", 61);
    assert_complete(&observation);
    assert!(observation.implied_p_pops.is_empty());
    assert_eq!(
        observation
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == DiagnosticKind::UnmatchedBlockEnd)
            .count(),
        2
    );
    assert_eq!(
        observation
            .actions
            .iter()
            .filter(|action| matches!(
                action,
                Action::BlockIgnored {
                    name: Name::Section,
                    ..
                }
            ))
            .count(),
        2
    );
    assert_eq!(
        node_name(&observation, *observation.open.last().unwrap()),
        Name::P
    );
}

#[test]
fn i10_selected_end_implied_pop_makes_later_p_end_unmatched_and_synthesized() {
    let observation = observe_fixture("I10", 71);
    assert_complete(&observation);
    assert_eq!(observation.implied_p_pops.len(), 1);
    assert_eq!(observation.p_syntheses.len(), 1);
    assert_eq!(observation.p_closures.len(), 1);
    assert_eq!(
        observation.p_closures[0].kind,
        PClosureKind::UnmatchedEndSynthesized
    );
    assert_ne!(
        observation.implied_p_pops[0].paragraph,
        observation.p_syntheses[0].node
    );
    assert!(matches!(
        observation
            .nodes
            .iter()
            .find(|node| node.id == observation.p_syntheses[0].node)
            .unwrap()
            .kind,
        NodeKind::Element {
            origin: Origin::Synthesized(SynthesisCause::UnmatchedPEnd),
            ..
        }
    ));
}

#[test]
fn i11_text_before_and_after_selected_end_has_exact_parentage() {
    let observation = observe_fixture("I11", 81);
    assert_complete(&observation);
    let p = nodes_named(&observation, Name::P)[0];
    let body = nodes_named(&observation, Name::Body)[0];
    let texts = text_nodes(&observation);
    assert_eq!(texts.len(), 2);
    assert_eq!(texts[0].parent, Some(p.id));
    assert_eq!(texts[1].parent, Some(body.id));
    let NodeKind::Text {
        interpreted: first, ..
    } = &texts[0].kind
    else {
        panic!("first text")
    };
    let NodeKind::Text {
        interpreted: second,
        ..
    } = &texts[1].kind
    else {
        panic!("second text")
    };
    assert_eq!(first, "x");
    assert_eq!(second, "y");
}

#[test]
fn i12_unmatched_selected_end_preserves_p_and_predecessor_open_block_eof_diagnostic() {
    let source = fixture("I12").source;
    let observation = observe_fixture("I12", 91);
    assert_complete(&observation);
    assert!(observation.implied_p_pops.is_empty());
    assert_eq!(
        diagnostic_kinds(&observation),
        vec![
            DiagnosticKind::MissingDoctype,
            DiagnosticKind::UnmatchedBlockEnd,
            DiagnosticKind::OpenBlockAtEof,
        ]
    );
    assert_eq!(
        observation.diagnostics[2].trigger,
        Some(expected_evidence(91, (source.len(), source.len())))
    );
    assert_eq!(
        node_name(&observation, *observation.open.last().unwrap()),
        Name::P
    );
}

#[test]
fn i13_i14_i23_excluded_selected_end_shapes_refuse_with_full_fingerprint_unchanged() {
    assert_refusal("I13", Unsupported::BlockAttribute);
    assert_refusal("I14", Unsupported::BlockSelfClosing);
    assert_refusal("I23", Unsupported::BlockAttribute);
}

#[test]
fn i15_selected_end_outside_in_body_is_refused_before_candidate_mutation() {
    assert_refusal("I15", Unsupported::SelectedEndOutsideInBody);
}

#[test]
fn i16_shell_p_crossing_remains_transactionally_outside_the_theorem() {
    assert_refusal("I16", Unsupported::BodyEndWithOpenP);
}

#[test]
fn i17_predecessor_tc_s4_recovery_without_p_is_unchanged() {
    let observation = observe_fixture("I17", 101);
    assert_complete(&observation);
    assert!(observation.implied_p_pops.is_empty());
    assert!(observation.p_closures.is_empty());
    assert!(observation.p_syntheses.is_empty());
    assert_eq!(observation.block_recovery.len(), 1);
    assert_eq!(
        diagnostic_kinds(&observation),
        vec![
            DiagnosticKind::MissingDoctype,
            DiagnosticKind::MisnestedBlockEnd
        ]
    );
    assert_eq!(
        node_name(&observation, observation.block_recovery[0].popped),
        Name::Section
    );
    assert_eq!(
        node_name(&observation, observation.block_recovery[0].target),
        Name::Div
    );
}

#[test]
fn i18_predecessor_tc_s5_start_triggered_and_matching_p_rules_are_unchanged() {
    let observation = observe_fixture("I18", 111);
    assert_complete(&observation);
    assert!(observation.implied_p_pops.is_empty());
    assert_eq!(observation.p_closures.len(), 2);
    assert_eq!(observation.p_closures[0].kind, PClosureKind::StartTriggered);
    assert_eq!(observation.p_closures[1].kind, PClosureKind::MatchingEnd);
    assert!(observation.p_syntheses.is_empty());
}

#[test]
fn i19_exact_authored_origins_and_selected_end_trigger_remain_separate() {
    let source = fixture("I19").source;
    let run = tokenize_source(source, 121);
    let observation = observe_with_layout(&run, StorageLayout::COMPACT);
    assert_complete(&observation);

    let p = nodes_named(&observation, Name::P)[0];
    let NodeKind::Element {
        origin: Origin::Authored { complete, raw_name },
        ..
    } = &p.kind
    else {
        panic!("authored P")
    };
    assert_eq!(*complete, expected_evidence(121, (11, 14)));
    assert_eq!(*raw_name, expected_evidence(121, (12, 13)));
    assert_eq!(&source[raw_name.range.0..raw_name.range.1], "P");

    let implied = &observation.implied_p_pops[0];
    assert_eq!(implied.paragraph, p.id);
    assert_eq!(implied.trigger, expected_evidence(121, (15, 21)));
    assert_ne!(implied.trigger.range, complete.range);
    let end = run
        .tokens()
        .iter()
        .find_map(|token| match token {
            HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::End => Some(tag),
            _ => None,
        })
        .expect("I19 end");
    let raw_end = evidence(end.name().source());
    assert_eq!(&source[raw_end.range.0..raw_end.range.1], "dIv");
}

#[test]
fn i20_implied_pop_allocates_no_identity_and_later_node_ids_match_explicit_p_close_control() {
    let implied = observe_fixture("I20", 131);
    let control = observe_source("<body><div><p>x</p></div><section>y</section>", 131);
    assert_complete(&implied);
    assert_complete(&control);
    assert_eq!(implied.nodes.len(), control.nodes.len());
    let implied_section = nodes_named(&implied, Name::Section)[0].id;
    let control_section = nodes_named(&control, Name::Section)[0].id;
    assert_eq!(implied_section, control_section);
    assert_eq!(implied.implied_p_pops.len(), 1);
    assert!(control.implied_p_pops.is_empty());
}

#[test]
fn i21_semantic_identity_and_relations_ignore_private_storage_padding() {
    let run = tokenize_source(fixture("I21").source, 141);
    let compact = observe_with_layout(&run, StorageLayout::COMPACT);
    let padded = observe_with_layout(&run, StorageLayout::PADDED);
    assert_eq!(compact.nodes, padded.nodes);
    assert_eq!(compact.diagnostics, padded.diagnostics);
    assert_eq!(compact.p_closures, padded.p_closures);
    assert_eq!(compact.p_syntheses, padded.p_syntheses);
    assert_eq!(compact.implied_p_pops, padded.implied_p_pops);
    assert_eq!(compact.block_recovery, padded.block_recovery);
    assert_eq!(compact.actions, padded.actions);
}

#[test]
fn i22_lower_layer_incompleteness_is_never_upgraded_to_complete() {
    let source = SourceText::new(SourceId::new(1), "<body><div><p>xxxxxxxx".to_owned());
    let run = tokenize(&source, HtmlTokenizerLimits::new(1, 1, 1, 1, 1, 1, 1));
    assert!(run.is_incomplete());
    let observation = observe_with_layout(&run, StorageLayout::COMPACT);
    assert_ne!(observation.completion, Completion::Complete);
}

#[test]
fn i24_generated_cells_agree_with_independent_closed_form_oracle() {
    for depth in 0_u32..=5 {
        for code in 0..2_usize.pow(depth) {
            let mut blocks = Vec::new();
            for shift in (0..depth).rev() {
                blocks.push(if (code >> shift) & 1 == 0 {
                    Name::Div
                } else {
                    Name::Section
                });
            }
            for p_current in [false, true] {
                for end in [Name::Div, Name::Section] {
                    let oracle = closed_form_oracle(&blocks, p_current, end);
                    let candidate = machine_cell_outcome(&blocks, p_current, end);
                    assert_eq!(
                        candidate, oracle,
                        "blocks={blocks:?} p={p_current} end={end:?}"
                    );
                }
            }
        }
    }
}
