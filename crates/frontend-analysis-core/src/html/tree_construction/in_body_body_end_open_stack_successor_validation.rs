//! Candidate-independent TC-S7 successor validation.
//!
//! TC-S7 is the bounded successor theorem "Selected In-Body `</body>`
//! Transition over the Open Bounded Stack with Stack Preservation and
//! After-Body Successor Composition" from Issue #374.
//!
//! This module consumes the accepted tokenizer only as lower-layer token/source
//! evidence. It imports no production tree-construction driver, session, result,
//! or production analysis output as a semantic oracle.
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
//! TC-S7 proves only the selected authored `</body>` transition from InBody,
//! plus its accepted TC-S2 AfterBody successors. The transition never pops the
//! bounded open stack. It is not arbitrary shell handling, generic scope, or a
//! generic HTML parser.

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlTagToken, HtmlToken};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;

const PINNED_WHATWG_COMMIT: &str = "508a037333d8a1806504303aeb489d931fabbef6";
const PINNED_WHATWG_BLOB: &str = "68dbcb98bbe1001c6ae2531be2368c608fbafddd";
const FRESH_WHATWG_HEAD: &str = "ae6c5d8ddfe6c819730f8f766d550dd1417e66c9";
const FRESH_WPT_HEAD: &str = "719d5e38fdd0903a18ed9007aba816c98cc491e0";

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
    Synthesized,
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
    AfterAfterBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticKind {
    MissingDoctype,
    BodyEndWithDisallowedOpenElements,
    AfterBodyCharacterData,
    UnmatchedParagraphEnd,
    UnmatchedBlockEnd,
    MisnestedBlockEnd,
    OpenBlockAtEof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    kind: DiagnosticKind,
    trigger: Option<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    AuthoredInsert {
        node: NodeId,
        name: Name,
        trigger: Evidence,
    },
    BodyEndDiagnostic {
        token_index: usize,
        trigger: Evidence,
    },
    BodyEndTransition {
        token_index: usize,
        trigger: Evidence,
        stack_before: Vec<NodeId>,
        stack_after: Vec<NodeId>,
    },
    AfterBodyCharacterDiagnostic {
        trigger: Evidence,
    },
    ReprocessSameTokenInBody {
        token_index: usize,
        trigger: Evidence,
    },
    HtmlEndTransition {
        trigger: Evidence,
    },
    ParagraphClose {
        target: NodeId,
        trigger: Evidence,
    },
    BlockDiagnostic {
        kind: DiagnosticKind,
        trigger: Evidence,
    },
    BlockRecoveryPop {
        popped: NodeId,
        target: NodeId,
        trigger: Evidence,
    },
    BlockClose {
        target: NodeId,
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
    MixedAfterBodyCharacterRun,
    BodyEndAttribute,
    BodyEndSelfClosing,
    HtmlEndWithOpenBoundedStack,
    BodyStartWithOpenBoundedStack,
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
    nodes: Vec<Node>,
    open: Vec<NodeId>,
    next_id: usize,
    diagnostics: Vec<Diagnostic>,
    actions: Vec<Action>,
    committed_end: usize,
    processed_tokens: usize,
    reprocess_count: usize,
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
    actions: Vec<Action>,
    open: Vec<NodeId>,
    phase: Phase,
    next_id: usize,
    committed_end: usize,
    processed_tokens: usize,
    reprocess_count: usize,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunClass {
    AllWhitespace,
    AllNonWhitespace,
    Mixed,
}

fn classify_run(interpreted: &str) -> RunClass {
    let mut whitespace = false;
    let mut other = false;
    for character in interpreted.chars() {
        if matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' ') {
            whitespace = true;
        } else {
            other = true;
        }
    }
    match (whitespace, other) {
        (true, true) => RunClass::Mixed,
        (true, false) => RunClass::AllWhitespace,
        (false, true) => RunClass::AllNonWhitespace,
        (false, false) => unreachable!("tokenizer emits no empty character token"),
    }
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
    actions: Vec<Action>,
    committed_end: usize,
    processed_tokens: usize,
    reprocess_count: usize,
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
            actions: Vec::new(),
            committed_end: 0,
            processed_tokens: 0,
            reprocess_count: 0,
        };
        machine.document = machine.allocate(None, NodeKind::Document);
        machine.html = machine.allocate(
            Some(machine.document),
            NodeKind::Element {
                name: Name::Html,
                origin: Origin::Synthesized,
            },
        );
        machine.allocate(
            Some(machine.html),
            NodeKind::Element {
                name: Name::Head,
                origin: Origin::Synthesized,
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

    fn nodes(&self) -> Vec<Node> {
        self.slots.iter().flatten().cloned().collect()
    }

    fn node(&self, id: NodeId) -> &Node {
        self.slots
            .iter()
            .flatten()
            .find(|node| node.id == id)
            .expect("semantic node identity")
    }

    fn node_mut(&mut self, id: NodeId) -> &mut Node {
        self.slots
            .iter_mut()
            .flatten()
            .find(|node| node.id == id)
            .expect("semantic node identity")
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

    fn has_open_block(&self) -> bool {
        self.open.iter().any(|id| self.name(*id).is_block())
    }

    fn has_open_bounded_content(&self) -> bool {
        self.open
            .iter()
            .any(|id| matches!(self.name(*id), Name::Div | Name::Section | Name::P))
    }

    fn current_is_p(&self) -> bool {
        self.name(self.current()) == Name::P
    }

    fn assert_bounded_stack(&self) -> bool {
        let names: Vec<Name> = self.open.iter().map(|id| self.name(*id)).collect();
        if names.len() < 2 || names[0] != Name::Html || names[1] != Name::Body {
            return false;
        }
        let mut saw_p = false;
        names[2..].iter().all(|name| match name {
            Name::Div | Name::Section if !saw_p => true,
            Name::P if !saw_p => {
                saw_p = true;
                true
            }
            _ => false,
        }) && (!saw_p || names.last() == Some(&Name::P))
    }

    fn assert_invariant(&self) {
        let valid = match self.phase {
            Phase::BeforeBody => {
                self.open.len() == 1
                    && self.open.first().map(|id| self.name(*id)) == Some(Name::Html)
            }
            Phase::InBody | Phase::AfterBody | Phase::AfterAfterBody => self.assert_bounded_stack(),
        };
        assert!(valid, "TC-S7 candidate stack invariant");
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            phase: self.phase,
            nodes: self.nodes(),
            open: self.open.clone(),
            next_id: self.next_id,
            diagnostics: self.diagnostics.clone(),
            actions: self.actions.clone(),
            committed_end: self.committed_end,
            processed_tokens: self.processed_tokens,
            reprocess_count: self.reprocess_count,
        }
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

    fn close_p(&mut self, trigger: Evidence) {
        assert!(self.current_is_p());
        let target = self.current();
        self.open.pop();
        self.actions
            .push(Action::ParagraphClose { target, trigger });
    }

    fn close_block(&mut self, name: Name, trigger: Evidence) {
        let Some(position) = self.open.iter().rposition(|id| self.name(*id) == name) else {
            self.push_diagnostic(DiagnosticKind::UnmatchedBlockEnd, Some(trigger.clone()));
            self.actions.push(Action::BlockDiagnostic {
                kind: DiagnosticKind::UnmatchedBlockEnd,
                trigger,
            });
            return;
        };
        let target = self.open[position];
        let intervening: Vec<NodeId> = self.open[position + 1..].iter().rev().copied().collect();
        if !intervening.is_empty() {
            self.push_diagnostic(DiagnosticKind::MisnestedBlockEnd, Some(trigger.clone()));
            self.actions.push(Action::BlockDiagnostic {
                kind: DiagnosticKind::MisnestedBlockEnd,
                trigger: trigger.clone(),
            });
        }
        for popped in intervening {
            assert_eq!(self.current(), popped);
            self.open.pop();
            self.actions.push(Action::BlockRecoveryPop {
                popped,
                target,
                trigger: trigger.clone(),
            });
        }
        assert_eq!(self.current(), target);
        self.open.pop();
        self.actions.push(Action::BlockClose { target, trigger });
    }

    fn process(&mut self, token_index: usize, token: &HtmlToken) -> Result<bool, Unsupported> {
        self.assert_invariant();
        let result = match self.phase {
            Phase::BeforeBody => self.process_before_body(token),
            Phase::InBody => self.process_in_body(token_index, token),
            Phase::AfterBody => self.process_after_body(token_index, token),
            Phase::AfterAfterBody => self.process_after_after_body(token),
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
        reject_non_candidate_shape(tag)?;
        self.insert_authored(
            Name::Body,
            evidence(tag.complete()),
            evidence(tag.name().source()),
        );
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
                let interpreted = tag.name().interpreted();
                if tag.kind() == HtmlTagKind::End && interpreted == "body" {
                    if !tag.attributes().is_empty() {
                        return Err(Unsupported::BodyEndAttribute);
                    }
                    if tag.self_closing_solidus().is_some() {
                        return Err(Unsupported::BodyEndSelfClosing);
                    }
                    let trigger = evidence(tag.complete());
                    let stack_before = self.open.clone();
                    if self.has_open_block() {
                        self.push_diagnostic(
                            DiagnosticKind::BodyEndWithDisallowedOpenElements,
                            Some(trigger.clone()),
                        );
                        self.actions.push(Action::BodyEndDiagnostic {
                            token_index,
                            trigger: trigger.clone(),
                        });
                    }
                    self.phase = Phase::AfterBody;
                    let stack_after = self.open.clone();
                    self.actions.push(Action::BodyEndTransition {
                        token_index,
                        trigger,
                        stack_before,
                        stack_after,
                    });
                    return Ok(false);
                }
                if tag.kind() == HtmlTagKind::End
                    && interpreted == "html"
                    && self.has_open_bounded_content()
                {
                    return Err(Unsupported::HtmlEndWithOpenBoundedStack);
                }
                if tag.kind() == HtmlTagKind::Start
                    && interpreted == "body"
                    && self.has_open_bounded_content()
                {
                    return Err(Unsupported::BodyStartWithOpenBoundedStack);
                }
                let name = interpreted_name(interpreted)?;
                reject_non_candidate_shape(tag)?;
                match (tag.kind(), name) {
                    (HtmlTagKind::Start, Name::P) => {
                        if self.current_is_p() {
                            self.close_p(evidence(tag.complete()));
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
                            self.close_p(evidence(tag.complete()));
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
                            self.close_p(evidence(tag.complete()));
                        } else {
                            self.push_diagnostic(
                                DiagnosticKind::UnmatchedParagraphEnd,
                                Some(evidence(tag.complete())),
                            );
                        }
                        Ok(false)
                    }
                    (HtmlTagKind::End, name) if name.is_block() => {
                        self.close_block(name, evidence(tag.complete()));
                        Ok(false)
                    }
                    _ => Err(Unsupported::OutsideCandidate),
                }
            }
        }
    }

    fn process_after_body(
        &mut self,
        token_index: usize,
        token: &HtmlToken,
    ) -> Result<bool, Unsupported> {
        match token {
            HtmlToken::EndOfFile(_) => Ok(true),
            HtmlToken::Character(character) => match classify_run(character.interpreted()) {
                RunClass::AllWhitespace => {
                    self.insert_text(character.interpreted(), evidence(character.source()));
                    Ok(false)
                }
                RunClass::AllNonWhitespace => {
                    let trigger = evidence(character.source());
                    self.push_diagnostic(
                        DiagnosticKind::AfterBodyCharacterData,
                        Some(trigger.clone()),
                    );
                    self.actions.push(Action::AfterBodyCharacterDiagnostic {
                        trigger: trigger.clone(),
                    });
                    self.phase = Phase::InBody;
                    self.reprocess_count += 1;
                    self.actions.push(Action::ReprocessSameTokenInBody {
                        token_index,
                        trigger: trigger.clone(),
                    });
                    self.insert_text(character.interpreted(), trigger);
                    Ok(false)
                }
                RunClass::Mixed => Err(Unsupported::MixedAfterBodyCharacterRun),
            },
            HtmlToken::Tag(tag)
                if tag.kind() == HtmlTagKind::End && tag.name().interpreted() == "html" =>
            {
                reject_non_candidate_shape(tag)?;
                let trigger = evidence(tag.complete());
                self.phase = Phase::AfterAfterBody;
                self.actions.push(Action::HtmlEndTransition { trigger });
                Ok(false)
            }
            _ => Err(Unsupported::OutsideCandidate),
        }
    }

    fn process_after_after_body(&mut self, token: &HtmlToken) -> Result<bool, Unsupported> {
        match token {
            HtmlToken::EndOfFile(_) => Ok(true),
            _ => Err(Unsupported::OutsideCandidate),
        }
    }
}

fn interpreted_name(name: &str) -> Result<Name, Unsupported> {
    match name {
        "html" => Ok(Name::Html),
        "body" => Ok(Name::Body),
        "div" => Ok(Name::Div),
        "section" => Ok(Name::Section),
        "p" => Ok(Name::P),
        _ => Err(Unsupported::GenericTag),
    }
}

fn reject_non_candidate_shape(tag: &HtmlTagToken) -> Result<(), Unsupported> {
    if !tag.attributes().is_empty() || tag.self_closing_solidus().is_some() {
        return Err(Unsupported::OutsideCandidate);
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
        actions: machine.actions,
        open: machine.open,
        phase: machine.phase,
        next_id: machine.next_id,
        committed_end: machine.committed_end,
        processed_tokens: machine.processed_tokens,
        reprocess_count: machine.reprocess_count,
        completion,
        refusal,
    }
}

fn observe(source: &str, source_id: u64) -> Observation {
    observe_with_layout(&tokenize_source(source, source_id), StorageLayout::COMPACT)
}

fn diagnostic_kinds(observation: &Observation) -> Vec<DiagnosticKind> {
    observation
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.kind)
        .collect()
}

fn open_names(observation: &Observation) -> Vec<Name> {
    observation
        .open
        .iter()
        .map(|id| {
            observation
                .nodes
                .iter()
                .find(|node| node.id == *id)
                .and_then(|node| match node.kind {
                    NodeKind::Element { name, .. } => Some(name),
                    _ => None,
                })
                .expect("open element")
        })
        .collect()
}

fn text_nodes(observation: &Observation) -> Vec<&Node> {
    observation
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, NodeKind::Text { .. }))
        .collect()
}

fn parent_name(observation: &Observation, node: &Node) -> Name {
    let parent = node.parent.expect("text parent");
    observation
        .nodes
        .iter()
        .find(|candidate| candidate.id == parent)
        .and_then(|candidate| match candidate.kind {
            NodeKind::Element { name, .. } => Some(name),
            _ => None,
        })
        .expect("element parent")
}

fn body_end_actions(observation: &Observation) -> Vec<&Action> {
    observation
        .actions
        .iter()
        .filter(|action| {
            matches!(
                action,
                Action::BodyEndDiagnostic { .. } | Action::BodyEndTransition { .. }
            )
        })
        .collect()
}

fn assert_complete(observation: &Observation) {
    assert_eq!(observation.completion, Completion::Complete);
    assert!(observation.refusal.is_none());
}

fn assert_refusal(source: &str, capability: Unsupported) {
    let observation = observe(source, 1);
    let refusal = observation.refusal.expect("refusal record");
    assert_eq!(refusal.capability, capability);
    assert_eq!(refusal.before, refusal.after);
    assert_eq!(
        observation.completion,
        Completion::Unsupported {
            capability,
            token_index: refusal.token_index,
        }
    );
}

#[test]
fn authority_and_independence_are_frozen() {
    assert_eq!(
        PINNED_WHATWG_COMMIT,
        "508a037333d8a1806504303aeb489d931fabbef6"
    );
    assert_eq!(
        PINNED_WHATWG_BLOB,
        "68dbcb98bbe1001c6ae2531be2368c608fbafddd"
    );
    assert_eq!(
        FRESH_WHATWG_HEAD,
        "ae6c5d8ddfe6c819730f8f766d550dd1417e66c9"
    );
    assert_eq!(FRESH_WPT_HEAD, "719d5e38fdd0903a18ed9007aba816c98cc491e0");

    let source = include_str!("in_body_body_end_open_stack_successor_validation.rs");
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
fn s7_01_to_s7_06_body_end_preserves_exact_stack_and_diagnostic_cardinality() {
    let cases = [
        (
            "<body><p></body>",
            vec![Name::Html, Name::Body, Name::P],
            0usize,
            2usize,
        ),
        (
            "<body><div></body>",
            vec![Name::Html, Name::Body, Name::Div],
            1,
            2,
        ),
        (
            "<body><section></body>",
            vec![Name::Html, Name::Body, Name::Section],
            1,
            2,
        ),
        (
            "<body><div><section></body>",
            vec![Name::Html, Name::Body, Name::Div, Name::Section],
            1,
            3,
        ),
        (
            "<body><div><p></body>",
            vec![Name::Html, Name::Body, Name::Div, Name::P],
            1,
            3,
        ),
        (
            "<body><div><section><p></body>",
            vec![Name::Html, Name::Body, Name::Div, Name::Section, Name::P],
            1,
            4,
        ),
    ];
    for (source, expected_open, body_diagnostics, expected_token_index) in cases {
        let observation = observe(source, 1);
        assert_complete(&observation);
        assert_eq!(observation.phase, Phase::AfterBody, "{source}");
        assert_eq!(open_names(&observation), expected_open, "{source}");
        assert_eq!(
            observation
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.kind
                    == DiagnosticKind::BodyEndWithDisallowedOpenElements)
                .count(),
            body_diagnostics,
            "{source}"
        );
        let transition = body_end_actions(&observation)
            .into_iter()
            .find_map(|action| match action {
                Action::BodyEndTransition {
                    token_index,
                    stack_before,
                    stack_after,
                    ..
                } => Some((*token_index, stack_before, stack_after)),
                _ => None,
            })
            .expect("body-end transition");
        assert_eq!(transition.0, expected_token_index, "{source}");
        assert_eq!(transition.1, transition.2, "{source}");
        assert!(
            observation.actions.iter().all(|action| !matches!(
                action,
                Action::ParagraphClose { .. }
                    | Action::BlockRecoveryPop { .. }
                    | Action::BlockClose { .. }
            )),
            "body-end transition must not close or recover bounded content: {source}"
        );
        for action in body_end_actions(&observation) {
            if let Action::BodyEndDiagnostic { token_index, .. } = action {
                assert_eq!(*token_index, expected_token_index, "{source}");
            }
        }
    }
}

#[test]
fn s7_07_s7_08_after_body_eof_keeps_open_nodes_without_in_body_eof_diagnostic() {
    for source in ["<body><p></body>", "<body><div></body>"] {
        let observation = observe(source, 1);
        assert_complete(&observation);
        assert_eq!(observation.phase, Phase::AfterBody);
        assert!(!diagnostic_kinds(&observation).contains(&DiagnosticKind::OpenBlockAtEof));
    }
}

#[test]
fn s7_09_to_s7_12_whitespace_delegation_keeps_after_body_and_retained_parent() {
    let cases = [
        ("<body><p></body> ", Name::P, " "),
        ("<body><div></body> ", Name::Div, " "),
        ("<body><div><p></body> ", Name::P, " "),
        ("<body><p></body> \t", Name::P, " \t"),
    ];
    for (source, expected_parent, expected_text) in cases {
        let observation = observe(source, 1);
        assert_complete(&observation);
        assert_eq!(observation.phase, Phase::AfterBody, "{source:?}");
        assert_eq!(observation.reprocess_count, 0, "{source:?}");
        assert!(!diagnostic_kinds(&observation).contains(&DiagnosticKind::AfterBodyCharacterData));
        let texts = text_nodes(&observation);
        let text = texts.last().expect("delegated text");
        assert_eq!(
            parent_name(&observation, text),
            expected_parent,
            "{source:?}"
        );
        let NodeKind::Text {
            interpreted,
            contributions,
        } = &text.kind
        else {
            panic!("text node")
        };
        assert_eq!(interpreted, expected_text, "{source:?}");
        assert_eq!(contributions.len(), 1, "aggregate contribution");
    }
}

#[test]
fn s7_13_to_s7_15_non_whitespace_reprocesses_once_and_retains_parent() {
    let cases = [
        ("<body><p></body>x", Name::P, false),
        ("<body><div></body>x", Name::Div, true),
        ("<body><div><p></body>x", Name::P, true),
    ];
    for (source, expected_parent, has_body_diag) in cases {
        let observation = observe(source, 1);
        assert_complete(&observation);
        assert_eq!(observation.phase, Phase::InBody, "{source}");
        assert_eq!(observation.reprocess_count, 1, "{source}");
        assert_eq!(
            diagnostic_kinds(&observation)
                .iter()
                .filter(|kind| **kind == DiagnosticKind::AfterBodyCharacterData)
                .count(),
            1,
            "{source}"
        );
        assert_eq!(
            diagnostic_kinds(&observation)
                .iter()
                .filter(|kind| **kind == DiagnosticKind::BodyEndWithDisallowedOpenElements)
                .count(),
            usize::from(has_body_diag),
            "{source}"
        );
        let texts = text_nodes(&observation);
        let text = texts.last().expect("reprocessed text");
        assert_eq!(parent_name(&observation, text), expected_parent, "{source}");
    }
}

#[test]
fn s7_16_mixed_aggregate_refusal_is_exactly_transactional() {
    assert_refusal(
        "<body><p></body> x",
        Unsupported::MixedAfterBodyCharacterRun,
    );
}

#[test]
fn s7_17_s7_18_html_end_moves_after_after_body_without_popping_retained_stack() {
    let cases = [
        (
            "<body><p></body></html>",
            vec![Name::Html, Name::Body, Name::P],
        ),
        (
            "<body><div></body></html>",
            vec![Name::Html, Name::Body, Name::Div],
        ),
    ];
    for (source, expected_open) in cases {
        let observation = observe(source, 1);
        assert_complete(&observation);
        assert_eq!(observation.phase, Phase::AfterAfterBody);
        assert_eq!(open_names(&observation), expected_open);
        assert!(
            observation
                .actions
                .iter()
                .any(|action| matches!(action, Action::HtmlEndTransition { .. }))
        );
    }
}

#[test]
fn s7_19_to_s7_22_later_in_body_actions_prove_original_nodes_remained_open() {
    let p = observe("<body><p></body>x</p>", 1);
    assert_complete(&p);
    assert_eq!(open_names(&p), vec![Name::Html, Name::Body]);
    assert!(
        p.actions
            .iter()
            .any(|action| matches!(action, Action::ParagraphClose { .. }))
    );

    let div = observe("<body><div></body>x</div>", 1);
    assert_complete(&div);
    assert_eq!(open_names(&div), vec![Name::Html, Name::Body]);
    assert!(
        div.actions
            .iter()
            .any(|action| matches!(action, Action::BlockClose { .. }))
    );

    let recovered = observe("<body><div><section></body>x</div>", 1);
    assert_complete(&recovered);
    assert_eq!(open_names(&recovered), vec![Name::Html, Name::Body]);
    let recovery: Vec<NodeId> = recovered
        .actions
        .iter()
        .filter_map(|action| match action {
            Action::BlockRecoveryPop { popped, .. } => Some(*popped),
            _ => None,
        })
        .collect();
    assert_eq!(recovery.len(), 1);

    let repeated = observe("<body><p></body>x</body>", 1);
    assert_complete(&repeated);
    assert_eq!(repeated.phase, Phase::AfterBody);
    assert_eq!(repeated.reprocess_count, 1);
    assert_eq!(
        repeated
            .actions
            .iter()
            .filter(|action| matches!(action, Action::BodyEndTransition { .. }))
            .count(),
        2
    );
}

#[test]
fn s7_23_to_s7_26_shape_and_non_candidate_crossings_refuse_before_mutation() {
    assert_refusal("<body><p></body id=x>", Unsupported::BodyEndAttribute);
    assert_refusal("<body><p></body/>", Unsupported::BodyEndSelfClosing);
    assert_refusal("<body><p></html>", Unsupported::HtmlEndWithOpenBoundedStack);
    assert_refusal(
        "<body><p><body>",
        Unsupported::BodyStartWithOpenBoundedStack,
    );
}

#[test]
fn s7_27_trigger_provenance_keeps_source_identity_complete_range_and_raw_spelling_distinct() {
    let source = "<body><div></BoDy>";
    let run = tokenize_source(source, 77);
    let body_end = run
        .tokens()
        .iter()
        .find_map(|token| match token {
            HtmlToken::Tag(tag)
                if tag.kind() == HtmlTagKind::End && tag.name().interpreted() == "body" =>
            {
                Some(tag)
            }
            _ => None,
        })
        .expect("body end token");
    assert_eq!(evidence(body_end.complete()).source_id, SourceId::new(77));
    assert_eq!(evidence(body_end.complete()).range, (11, 18));
    assert_eq!(evidence(body_end.name().source()).range, (13, 17));
    assert_eq!(&source[13..17], "BoDy");

    let observation = observe_with_layout(&run, StorageLayout::COMPACT);
    assert_complete(&observation);
    let trigger = observation
        .actions
        .iter()
        .find_map(|action| match action {
            Action::BodyEndTransition { trigger, .. } => Some(trigger),
            _ => None,
        })
        .expect("transition trigger");
    assert_eq!(trigger.source_id, SourceId::new(77));
    assert_eq!(trigger.range, (11, 18));
}

#[test]
fn s7_28_body_end_transition_consumes_no_constructed_identity() {
    for (without_end, with_end) in [
        ("<body><p>", "<body><p></body>"),
        ("<body><div>", "<body><div></body>"),
        ("<body><div><section><p>", "<body><div><section><p></body>"),
    ] {
        let control = observe(without_end, 1);
        let candidate = observe(with_end, 1);
        assert_eq!(control.next_id, candidate.next_id, "{with_end}");
    }
}

#[test]
fn committed_coverage_and_processed_token_count_are_exact() {
    let supported_source = "<body><div></body>x";
    let supported = observe(supported_source, 1);
    assert_complete(&supported);
    assert_eq!(supported.committed_end, 19);
    assert_eq!(supported.processed_tokens, 5);

    let refused_source = "<body><p></body> x";
    let refused = observe(refused_source, 1);
    assert_eq!(refused.committed_end, 16);
    assert_eq!(refused.processed_tokens, 3);
    let refusal = refused.refusal.as_ref().expect("mixed refusal");
    assert_eq!(refusal.token_index, 3);
}

#[test]
fn s7_29_lower_layer_incompleteness_is_never_upgraded() {
    let source = SourceText::new(SourceId::new(1), "<body><p></body>xxxxxxxx".to_owned());
    let run = tokenize(&source, HtmlTokenizerLimits::new(1, 1, 1, 1, 1, 1, 1));
    assert!(run.is_incomplete());
    let observation = observe_with_layout(&run, StorageLayout::COMPACT);
    assert_ne!(observation.completion, Completion::Complete);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SuccessorClass {
    Eof,
    Whitespace,
    NonWhitespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoldSummary {
    phase: Phase,
    open: Vec<Name>,
    diagnostics: Vec<DiagnosticKind>,
    text_parent: Option<Name>,
    reprocess_count: usize,
}

fn closed_form_gold(blocks: &[Name], p: bool, successor: SuccessorClass) -> GoldSummary {
    let mut open = vec![Name::Html, Name::Body];
    open.extend_from_slice(blocks);
    if p {
        open.push(Name::P);
    }
    let current = *open.last().expect("bounded current");
    let mut diagnostics = vec![DiagnosticKind::MissingDoctype];
    if !blocks.is_empty() {
        diagnostics.push(DiagnosticKind::BodyEndWithDisallowedOpenElements);
    }
    match successor {
        SuccessorClass::Eof => GoldSummary {
            phase: Phase::AfterBody,
            open,
            diagnostics,
            text_parent: None,
            reprocess_count: 0,
        },
        SuccessorClass::Whitespace => GoldSummary {
            phase: Phase::AfterBody,
            open,
            diagnostics,
            text_parent: Some(current),
            reprocess_count: 0,
        },
        SuccessorClass::NonWhitespace => {
            diagnostics.push(DiagnosticKind::AfterBodyCharacterData);
            if !blocks.is_empty() {
                diagnostics.push(DiagnosticKind::OpenBlockAtEof);
            }
            GoldSummary {
                phase: Phase::InBody,
                open,
                diagnostics,
                text_parent: Some(current),
                reprocess_count: 1,
            }
        }
    }
}

fn source_for(blocks: &[Name], p: bool, successor: SuccessorClass) -> String {
    let mut source = String::from("<body>");
    for block in blocks {
        source.push_str(match block {
            Name::Div => "<div>",
            Name::Section => "<section>",
            _ => unreachable!("generated block domain"),
        });
    }
    if p {
        source.push_str("<p>");
    }
    source.push_str("</body>");
    match successor {
        SuccessorClass::Eof => {}
        SuccessorClass::Whitespace => source.push(' '),
        SuccessorClass::NonWhitespace => source.push('x'),
    }
    source
}

fn generated_block_sequences(max_depth: usize) -> Vec<Vec<Name>> {
    let mut sequences = vec![Vec::new()];
    for depth in 1..=max_depth {
        for mask in 0..(1usize << depth) {
            let mut sequence = Vec::with_capacity(depth);
            for bit in 0..depth {
                sequence.push(if mask & (1usize << bit) == 0 {
                    Name::Div
                } else {
                    Name::Section
                });
            }
            sequences.push(sequence);
        }
    }
    sequences
}

#[test]
fn s7_30_generated_candidate_matches_independent_closed_form_gold() {
    for blocks in generated_block_sequences(4) {
        for p in [false, true] {
            for successor in [
                SuccessorClass::Eof,
                SuccessorClass::Whitespace,
                SuccessorClass::NonWhitespace,
            ] {
                let source = source_for(&blocks, p, successor);
                let observation = observe(&source, 9);
                assert_complete(&observation);
                let gold = closed_form_gold(&blocks, p, successor);
                assert_eq!(observation.phase, gold.phase, "{source}");
                assert_eq!(open_names(&observation), gold.open, "{source}");
                assert_eq!(diagnostic_kinds(&observation), gold.diagnostics, "{source}");
                assert_eq!(
                    observation.reprocess_count, gold.reprocess_count,
                    "{source}"
                );
                let actual_parent = text_nodes(&observation)
                    .last()
                    .map(|node| parent_name(&observation, node));
                assert_eq!(actual_parent, gold.text_parent, "{source}");
            }
        }
    }
}

#[test]
fn storage_layout_does_not_change_semantic_identity_or_evidence() {
    for source in [
        "<body><p></body> ",
        "<body><div><section><p></body>x</div>",
        "<body><div></body></html>",
    ] {
        let run = tokenize_source(source, 42);
        let compact = observe_with_layout(&run, StorageLayout::COMPACT);
        let padded = observe_with_layout(&run, StorageLayout::PADDED);
        assert_eq!(compact.nodes, padded.nodes, "{source}");
        assert_eq!(compact.diagnostics, padded.diagnostics, "{source}");
        assert_eq!(compact.actions, padded.actions, "{source}");
        assert_eq!(compact.open, padded.open, "{source}");
        assert_eq!(compact.phase, padded.phase, "{source}");
        assert_eq!(compact.completion, padded.completion, "{source}");
    }
}
