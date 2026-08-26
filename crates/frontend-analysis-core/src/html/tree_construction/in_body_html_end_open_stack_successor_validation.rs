//! Candidate-independent TC-S8 successor validation.
//!
//! TC-S8 is the bounded theorem "Selected In-Body `</html>` over the Open
//! Bounded Stack with Body-End Audit, Stack Preservation, and Same-Token
//! AfterBody Reprocessing" from Issue #380.
//!
//! This module consumes the accepted tokenizer only as lower-layer token/source
//! evidence. It imports no production tree-construction driver, session, result,
//! or production analysis output as a semantic oracle.
//!
//! Normative authority: WHATWG HTML commit
//! `508a037333d8a1806504303aeb489d931fabbef6`, source blob
//! `68dbcb98bbe1001c6ae2531be2368c608fbafddd`.
//!
//! Freshness markers at frontier selection:
//! WHATWG `64b40967d74792ffbfa18ec431074e060412f557`,
//! WPT `3ba06522cce6462b7042de96a4dd0bcb67d02616`.

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;

const PINNED_WHATWG_COMMIT: &str = "508a037333d8a1806504303aeb489d931fabbef6";
const PINNED_WHATWG_BLOB: &str = "68dbcb98bbe1001c6ae2531be2368c608fbafddd";
const FRESH_WHATWG_HEAD: &str = "64b40967d74792ffbfa18ec431074e060412f557";
const FRESH_WPT_HEAD: &str = "3ba06522cce6462b7042de96a4dd0bcb67d02616";

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
    HtmlEndAudit,
    BodyEndAudit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    kind: DiagnosticKind,
    token_index: usize,
    trigger: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    Dispatch {
        phase: Phase,
        token_index: usize,
        trigger: Evidence,
    },
    Insert {
        node: NodeId,
        name: Name,
        token_index: usize,
        trigger: Evidence,
    },
    TextInsert {
        node: NodeId,
        parent: NodeId,
        token_index: usize,
        contribution: Evidence,
    },
    AuditDiagnostic {
        kind: DiagnosticKind,
        token_index: usize,
        trigger: Evidence,
    },
    Transition {
        from: Phase,
        to: Phase,
        token_index: usize,
        trigger: Evidence,
    },
    ReprocessSameToken {
        token_index: usize,
        trigger: Evidence,
    },
    Consume {
        phase: Phase,
        token_index: usize,
        trigger: Evidence,
    },
    StopAtEof {
        phase: Phase,
        token_index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unsupported {
    HtmlEndAttribute,
    HtmlEndSelfClosing,
    BodyStartWithOpenBoundedStack,
    OtherShellEnd,
    AfterAfterBodyCharacterData,
    AfterAfterBodyTag,
    OutsideClosedCandidate,
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

struct Machine {
    slots: Vec<Option<Node>>,
    layout: StorageLayout,
    next_id: usize,
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
            open: Vec::new(),
            phase: Phase::BeforeBody,
            diagnostics: Vec::new(),
            actions: Vec::new(),
            committed_end: 0,
            processed_tokens: 0,
            reprocess_count: 0,
        };
        let html = machine.allocate(
            None,
            NodeKind::Element {
                name: Name::Html,
                origin: Origin::Synthesized,
            },
        );
        machine.open.push(html);
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

    fn allocate_probe(&mut self) -> NodeId {
        let parent = self.current();
        self.allocate(
            Some(parent),
            NodeKind::Element {
                name: Name::P,
                origin: Origin::Synthesized,
            },
        )
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

    fn name(&self, id: NodeId) -> Name {
        match self.node(id).kind {
            NodeKind::Element { name, .. } => name,
            NodeKind::Text { .. } => panic!("text is never open"),
        }
    }

    fn current(&self) -> NodeId {
        *self.open.last().expect("open element")
    }

    fn open_names(&self) -> Vec<Name> {
        self.open.iter().map(|id| self.name(*id)).collect()
    }

    fn has_block(&self) -> bool {
        self.open.iter().any(|id| self.name(*id).is_block())
    }

    fn has_bounded_content(&self) -> bool {
        self.open
            .iter()
            .any(|id| matches!(self.name(*id), Name::Div | Name::Section | Name::P))
    }

    fn body_in_scope_bounded(&self) -> bool {
        self.open_names().get(1) == Some(&Name::Body)
    }

    fn closed_stack_invariant(&self) -> bool {
        let names = self.open_names();
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
        match self.phase {
            Phase::BeforeBody => assert_eq!(self.open_names(), vec![Name::Html]),
            Phase::InBody | Phase::AfterBody | Phase::AfterAfterBody => {
                assert!(self.closed_stack_invariant(), "closed TC-S8 stack")
            }
        }
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

    fn insert_authored(
        &mut self,
        name: Name,
        token_index: usize,
        complete: Evidence,
        raw_name: Evidence,
    ) {
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
        self.actions.push(Action::Insert {
            node: id,
            name,
            token_index,
            trigger: complete,
        });
    }

    fn insert_text(&mut self, token_index: usize, interpreted: &str, contribution: Evidence) {
        let parent = self.current();
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
            token_index,
            contribution,
        });
    }

    fn dispatch(&mut self, phase: Phase, token_index: usize, trigger: Evidence) {
        self.actions.push(Action::Dispatch {
            phase,
            token_index,
            trigger,
        });
    }

    fn transition(&mut self, to: Phase, token_index: usize, trigger: Evidence) {
        let from = self.phase;
        self.actions.push(Action::Transition {
            from,
            to,
            token_index,
            trigger,
        });
        self.phase = to;
    }

    fn diagnostic(&mut self, kind: DiagnosticKind, token_index: usize, trigger: Evidence) {
        self.diagnostics.push(Diagnostic {
            kind,
            token_index,
            trigger: trigger.clone(),
        });
        self.actions.push(Action::AuditDiagnostic {
            kind,
            token_index,
            trigger,
        });
    }

    fn consume(&mut self, token_index: usize, trigger: Evidence) {
        self.actions.push(Action::Consume {
            phase: self.phase,
            token_index,
            trigger,
        });
    }

    fn commit(&mut self, token: &HtmlToken) {
        self.committed_end = token_end(token);
        self.processed_tokens += 1;
    }

    fn process_html_end(
        &mut self,
        token_index: usize,
        token: &HtmlToken,
        trigger: Evidence,
    ) -> Result<(), Unsupported> {
        let HtmlToken::Tag(tag) = token else {
            return Err(Unsupported::OutsideClosedCandidate);
        };
        if !tag.attributes().is_empty() {
            return Err(Unsupported::HtmlEndAttribute);
        }
        if tag.self_closing_solidus().is_some() {
            return Err(Unsupported::HtmlEndSelfClosing);
        }
        if !self.body_in_scope_bounded() || !self.closed_stack_invariant() {
            return Err(Unsupported::OutsideClosedCandidate);
        }

        let stack_before = self.open.clone();
        let next_id_before = self.next_id;
        self.dispatch(Phase::InBody, token_index, trigger.clone());
        if self.has_block() {
            self.diagnostic(DiagnosticKind::HtmlEndAudit, token_index, trigger.clone());
        }
        assert_eq!(self.open, stack_before);
        assert_eq!(self.next_id, next_id_before);
        self.transition(Phase::AfterBody, token_index, trigger.clone());
        self.actions.push(Action::ReprocessSameToken {
            token_index,
            trigger: trigger.clone(),
        });
        self.reprocess_count += 1;
        self.dispatch(Phase::AfterBody, token_index, trigger.clone());
        self.transition(Phase::AfterAfterBody, token_index, trigger.clone());
        self.consume(token_index, trigger);
        assert_eq!(self.open, stack_before);
        assert_eq!(self.next_id, next_id_before);
        Ok(())
    }

    fn process_body_end(
        &mut self,
        token_index: usize,
        trigger: Evidence,
    ) -> Result<(), Unsupported> {
        if !self.body_in_scope_bounded() || !self.closed_stack_invariant() {
            return Err(Unsupported::OutsideClosedCandidate);
        }
        let stack_before = self.open.clone();
        let next_id_before = self.next_id;
        self.dispatch(Phase::InBody, token_index, trigger.clone());
        if self.has_block() {
            self.diagnostic(DiagnosticKind::BodyEndAudit, token_index, trigger.clone());
        }
        self.transition(Phase::AfterBody, token_index, trigger.clone());
        self.consume(token_index, trigger);
        assert_eq!(self.open, stack_before);
        assert_eq!(self.next_id, next_id_before);
        Ok(())
    }

    fn step(&mut self, token_index: usize, token: &HtmlToken) -> Result<Step, Unsupported> {
        let trigger = token_evidence(token);
        match self.phase {
            Phase::BeforeBody => match token {
                HtmlToken::Tag(tag)
                    if tag.kind() == HtmlTagKind::Start
                        && tag.name().interpreted().eq_ignore_ascii_case("body") =>
                {
                    if !tag.attributes().is_empty() || tag.self_closing_solidus().is_some() {
                        return Err(Unsupported::OutsideClosedCandidate);
                    }
                    self.dispatch(Phase::BeforeBody, token_index, trigger.clone());
                    self.insert_authored(
                        Name::Body,
                        token_index,
                        trigger.clone(),
                        evidence(tag.name().source()),
                    );
                    self.transition(Phase::InBody, token_index, trigger.clone());
                    self.consume(token_index, trigger);
                    Ok(Step::Consumed)
                }
                HtmlToken::EndOfFile(_) => Err(Unsupported::OutsideClosedCandidate),
                _ => Err(Unsupported::OutsideClosedCandidate),
            },
            Phase::InBody => match token {
                HtmlToken::Character(character) => {
                    self.dispatch(Phase::InBody, token_index, trigger.clone());
                    self.insert_text(token_index, character.interpreted(), trigger.clone());
                    self.consume(token_index, trigger);
                    Ok(Step::Consumed)
                }
                HtmlToken::Tag(tag)
                    if tag.kind() == HtmlTagKind::Start
                        && tag.name().interpreted().eq_ignore_ascii_case("div") =>
                {
                    self.dispatch(Phase::InBody, token_index, trigger.clone());
                    self.insert_authored(
                        Name::Div,
                        token_index,
                        trigger.clone(),
                        evidence(tag.name().source()),
                    );
                    self.consume(token_index, trigger);
                    Ok(Step::Consumed)
                }
                HtmlToken::Tag(tag)
                    if tag.kind() == HtmlTagKind::Start
                        && tag.name().interpreted().eq_ignore_ascii_case("section") =>
                {
                    self.dispatch(Phase::InBody, token_index, trigger.clone());
                    self.insert_authored(
                        Name::Section,
                        token_index,
                        trigger.clone(),
                        evidence(tag.name().source()),
                    );
                    self.consume(token_index, trigger);
                    Ok(Step::Consumed)
                }
                HtmlToken::Tag(tag)
                    if tag.kind() == HtmlTagKind::Start
                        && tag.name().interpreted().eq_ignore_ascii_case("p") =>
                {
                    if self.open_names().contains(&Name::P) {
                        return Err(Unsupported::OutsideClosedCandidate);
                    }
                    self.dispatch(Phase::InBody, token_index, trigger.clone());
                    self.insert_authored(
                        Name::P,
                        token_index,
                        trigger.clone(),
                        evidence(tag.name().source()),
                    );
                    self.consume(token_index, trigger);
                    Ok(Step::Consumed)
                }
                HtmlToken::Tag(tag)
                    if tag.kind() == HtmlTagKind::Start
                        && tag.name().interpreted().eq_ignore_ascii_case("body") =>
                {
                    if self.has_bounded_content() {
                        Err(Unsupported::BodyStartWithOpenBoundedStack)
                    } else {
                        Err(Unsupported::OutsideClosedCandidate)
                    }
                }
                HtmlToken::Tag(tag)
                    if tag.kind() == HtmlTagKind::End
                        && tag.name().interpreted().eq_ignore_ascii_case("html") =>
                {
                    self.process_html_end(token_index, token, trigger)?;
                    Ok(Step::Consumed)
                }
                HtmlToken::Tag(tag)
                    if tag.kind() == HtmlTagKind::End
                        && tag.name().interpreted().eq_ignore_ascii_case("body") =>
                {
                    if !tag.attributes().is_empty() || tag.self_closing_solidus().is_some() {
                        return Err(Unsupported::OutsideClosedCandidate);
                    }
                    self.process_body_end(token_index, trigger)?;
                    Ok(Step::Consumed)
                }
                HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::End => {
                    Err(Unsupported::OtherShellEnd)
                }
                HtmlToken::Tag(_) => Err(Unsupported::OutsideClosedCandidate),
                HtmlToken::EndOfFile(_) => Err(Unsupported::OutsideClosedCandidate),
            },
            Phase::AfterBody => match token {
                HtmlToken::Tag(tag)
                    if tag.kind() == HtmlTagKind::End
                        && tag.name().interpreted().eq_ignore_ascii_case("html") =>
                {
                    if !tag.attributes().is_empty() {
                        return Err(Unsupported::HtmlEndAttribute);
                    }
                    if tag.self_closing_solidus().is_some() {
                        return Err(Unsupported::HtmlEndSelfClosing);
                    }
                    self.dispatch(Phase::AfterBody, token_index, trigger.clone());
                    self.transition(Phase::AfterAfterBody, token_index, trigger.clone());
                    self.consume(token_index, trigger);
                    Ok(Step::Consumed)
                }
                HtmlToken::EndOfFile(_) => {
                    self.actions.push(Action::StopAtEof {
                        phase: Phase::AfterBody,
                        token_index,
                    });
                    Ok(Step::Stopped)
                }
                _ => Err(Unsupported::OutsideClosedCandidate),
            },
            Phase::AfterAfterBody => match token {
                HtmlToken::EndOfFile(_) => {
                    self.actions.push(Action::StopAtEof {
                        phase: Phase::AfterAfterBody,
                        token_index,
                    });
                    Ok(Step::Stopped)
                }
                HtmlToken::Character(_) => Err(Unsupported::AfterAfterBodyCharacterData),
                HtmlToken::Tag(_) => Err(Unsupported::AfterAfterBodyTag),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Consumed,
    Stopped,
}

fn token_end(token: &HtmlToken) -> usize {
    match token {
        HtmlToken::Character(character) => character.source().range().end(),
        HtmlToken::Tag(tag) => tag.complete().range().end(),
        HtmlToken::EndOfFile(eof) => eof.source().range().start(),
    }
}

fn token_evidence(token: &HtmlToken) -> Evidence {
    match token {
        HtmlToken::Character(character) => evidence(character.source()),
        HtmlToken::Tag(tag) => evidence(tag.complete()),
        HtmlToken::EndOfFile(eof) => evidence(eof.source()),
    }
}

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn observe(source: &str) -> Observation {
    observe_with(source, 1, StorageLayout::COMPACT, limits())
}

fn observe_with(
    source_text: &str,
    source_id: u64,
    layout: StorageLayout,
    limits: HtmlTokenizerLimits,
) -> Observation {
    let source = SourceText::new(SourceId::new(source_id), source_text.to_owned());
    let run = tokenize(&source, limits);
    observe_run(&run, layout)
}

fn observe_run(run: &HtmlTokenizerRunResult, layout: StorageLayout) -> Observation {
    let mut machine = Machine::new(layout);
    let mut stopped = false;
    let mut refusal = None;

    for (token_index, token) in run.tokens().iter().enumerate() {
        let before = machine.snapshot();
        match machine.step(token_index, token) {
            Ok(step) => {
                machine.commit(token);
                machine.assert_invariant();
                if step == Step::Stopped {
                    stopped = true;
                    break;
                }
            }
            Err(capability) => {
                let after = machine.snapshot();
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

    let completion = if let Some(refusal) = &refusal {
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

fn open_names(observation: &Observation) -> Vec<Name> {
    observation
        .open
        .iter()
        .map(|id| {
            let node = observation
                .nodes
                .iter()
                .find(|node| node.id == *id)
                .expect("open identity resolves");
            match node.kind {
                NodeKind::Element { name, .. } => name,
                NodeKind::Text { .. } => panic!("text is never open"),
            }
        })
        .collect()
}

fn diagnostic_count(observation: &Observation, kind: DiagnosticKind) -> usize {
    observation
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.kind == kind)
        .count()
}

fn candidate_token_index(observation: &Observation) -> usize {
    observation
        .actions
        .iter()
        .find_map(|action| match action {
            Action::ReprocessSameToken { token_index, .. } => Some(*token_index),
            _ => None,
        })
        .expect("candidate reprocess")
}

fn candidate_actions(observation: &Observation) -> Vec<&Action> {
    let token_index = candidate_token_index(observation);
    observation
        .actions
        .iter()
        .filter(|action| action_token_index(action) == Some(token_index))
        .collect()
}

fn action_token_index(action: &Action) -> Option<usize> {
    match action {
        Action::Dispatch { token_index, .. }
        | Action::Insert { token_index, .. }
        | Action::TextInsert { token_index, .. }
        | Action::AuditDiagnostic { token_index, .. }
        | Action::Transition { token_index, .. }
        | Action::ReprocessSameToken { token_index, .. }
        | Action::Consume { token_index, .. }
        | Action::StopAtEof { token_index, .. } => Some(*token_index),
    }
}

fn candidate_trigger_evidence(observation: &Observation) -> Vec<Evidence> {
    candidate_actions(observation)
        .into_iter()
        .filter_map(|action| match action {
            Action::Dispatch { trigger, .. }
            | Action::AuditDiagnostic { trigger, .. }
            | Action::Transition { trigger, .. }
            | Action::ReprocessSameToken { trigger, .. }
            | Action::Consume { trigger, .. } => Some(trigger.clone()),
            Action::Insert { .. } | Action::TextInsert { .. } | Action::StopAtEof { .. } => None,
        })
        .collect()
}

fn semantic_signature(observation: &Observation) -> (Vec<Name>, Phase, usize, usize, usize) {
    (
        open_names(observation),
        observation.phase,
        diagnostic_count(observation, DiagnosticKind::HtmlEndAudit),
        observation.reprocess_count,
        observation.next_id,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Oracle {
    audit_diagnostics: usize,
    final_stack: Vec<Name>,
    mode_path: Vec<Phase>,
    reprocess_count: usize,
    identity_allocation: usize,
    lifecycle_mutations: usize,
}

fn closed_form_oracle(blocks: &[Name], p: bool) -> Oracle {
    let mut final_stack = vec![Name::Html, Name::Body];
    final_stack.extend_from_slice(blocks);
    if p {
        final_stack.push(Name::P);
    }
    Oracle {
        audit_diagnostics: usize::from(!blocks.is_empty()),
        final_stack,
        mode_path: vec![Phase::InBody, Phase::AfterBody, Phase::AfterAfterBody],
        reprocess_count: 1,
        identity_allocation: 0,
        lifecycle_mutations: 0,
    }
}

fn generated_source(blocks: &[Name], p: bool, suffix: &str) -> String {
    let mut source = String::from("<body>");
    for block in blocks {
        source.push_str(match block {
            Name::Div => "<div>",
            Name::Section => "<section>",
            _ => panic!("generated block domain"),
        });
    }
    if p {
        source.push_str("<p>x");
    }
    source.push_str("</html>");
    source.push_str(suffix);
    source
}

fn assert_transactional_refusal(observation: &Observation, capability: Unsupported) {
    let refusal = observation.refusal.as_ref().expect("expected refusal");
    assert_eq!(refusal.capability, capability);
    assert_eq!(refusal.before, refusal.after, "refusal is transactional");
}

#[test]
fn authority_markers_and_candidate_independence_are_pinned() {
    assert_eq!(PINNED_WHATWG_COMMIT.len(), 40);
    assert_eq!(PINNED_WHATWG_BLOB.len(), 40);
    assert_eq!(FRESH_WHATWG_HEAD.len(), 40);
    assert_eq!(FRESH_WPT_HEAD.len(), 40);

    let source = include_str!("in_body_html_end_open_stack_successor_validation.rs");
    for forbidden in [
        ["super::", "driver"].concat(),
        ["super::", "session"].concat(),
        ["super::", "result"].concat(),
        ["in_body_html_end_open_stack_successor_", "production"].concat(),
    ] {
        assert!(
            !source.contains(&forbidden),
            "forbidden semantic oracle: {forbidden}"
        );
    }
}

#[test]
fn h1_h8_closed_stack_audit_cardinality_stack_identity_and_p_distinction() {
    let cases = [
        ("<body></html>", 0usize, vec![Name::Html, Name::Body]),
        (
            "<body><p>x</html>",
            0,
            vec![Name::Html, Name::Body, Name::P],
        ),
        (
            "<body><div>x</html>",
            1,
            vec![Name::Html, Name::Body, Name::Div],
        ),
        (
            "<body><section>x</html>",
            1,
            vec![Name::Html, Name::Body, Name::Section],
        ),
        (
            "<body><div><section>x</html>",
            1,
            vec![Name::Html, Name::Body, Name::Div, Name::Section],
        ),
        (
            "<body><div><p>x</html>",
            1,
            vec![Name::Html, Name::Body, Name::Div, Name::P],
        ),
        (
            "<body><section><div><section><p>x</html>",
            1,
            vec![
                Name::Html,
                Name::Body,
                Name::Section,
                Name::Div,
                Name::Section,
                Name::P,
            ],
        ),
    ];

    for (source, expected_diagnostics, expected_stack) in cases {
        let observation = observe(source);
        assert_eq!(observation.completion, Completion::Complete, "{source}");
        assert_eq!(observation.phase, Phase::AfterAfterBody, "{source}");
        assert_eq!(open_names(&observation), expected_stack, "{source}");
        assert_eq!(
            diagnostic_count(&observation, DiagnosticKind::HtmlEndAudit),
            expected_diagnostics,
            "{source}"
        );
        assert_eq!(observation.reprocess_count, 1, "{source}");
        assert_eq!(observation.committed_end, source.len(), "{source}");
    }
}

#[test]
fn h3_h9_h25_exact_mixed_case_evidence_one_token_two_modes_one_reprocess() {
    let source = "<body><div>x</HtMl>";
    let source_text = SourceText::new(SourceId::new(77), source.to_owned());
    let run = tokenize(&source_text, limits());
    let observation = observe_run(&run, StorageLayout::COMPACT);
    assert_eq!(observation.completion, Completion::Complete);
    let token_index = candidate_token_index(&observation);
    let actions = candidate_actions(&observation);

    let expected_kinds: Vec<&str> = actions
        .iter()
        .map(|action| match action {
            Action::Dispatch {
                phase: Phase::InBody,
                ..
            } => "dispatch-in-body",
            Action::AuditDiagnostic {
                kind: DiagnosticKind::HtmlEndAudit,
                ..
            } => "audit",
            Action::Transition {
                from: Phase::InBody,
                to: Phase::AfterBody,
                ..
            } => "to-after-body",
            Action::ReprocessSameToken { .. } => "reprocess",
            Action::Dispatch {
                phase: Phase::AfterBody,
                ..
            } => "dispatch-after-body",
            Action::Transition {
                from: Phase::AfterBody,
                to: Phase::AfterAfterBody,
                ..
            } => "to-after-after-body",
            Action::Consume {
                phase: Phase::AfterAfterBody,
                ..
            } => "consume",
            other => panic!("unexpected candidate action: {other:?}"),
        })
        .collect();
    assert_eq!(
        expected_kinds,
        vec![
            "dispatch-in-body",
            "audit",
            "to-after-body",
            "reprocess",
            "dispatch-after-body",
            "to-after-after-body",
            "consume",
        ]
    );
    assert_eq!(
        actions
            .iter()
            .filter(|action| matches!(
                action,
                Action::Dispatch {
                    phase: Phase::InBody,
                    ..
                }
            ))
            .count(),
        1
    );
    assert_eq!(
        actions
            .iter()
            .filter(|action| matches!(
                action,
                Action::Dispatch {
                    phase: Phase::AfterBody,
                    ..
                }
            ))
            .count(),
        1
    );
    assert_eq!(observation.reprocess_count, 1);

    let triggers = candidate_trigger_evidence(&observation);
    assert!(!triggers.is_empty());
    assert!(triggers.iter().all(|trigger| trigger == &triggers[0]));
    assert_eq!(triggers[0].source_id, SourceId::new(77));
    assert_eq!(triggers[0].range, (12, 19));

    let candidate_token = &run.tokens()[token_index];
    let HtmlToken::Tag(candidate_tag) = candidate_token else {
        panic!("candidate token must be the retained authored html end tag")
    };
    assert_eq!(candidate_tag.kind(), HtmlTagKind::End);
    assert!(
        candidate_tag
            .name()
            .interpreted()
            .eq_ignore_ascii_case("html")
    );
    let raw_name = evidence(candidate_tag.name().source());
    assert_eq!(raw_name.source_id, SourceId::new(77));
    assert_eq!(raw_name.range, (14, 18));
    assert_eq!(&source[raw_name.range.0..raw_name.range.1], "HtMl");
    assert_eq!(token_evidence(candidate_token), triggers[0]);
    assert!(
        actions
            .iter()
            .all(|action| action_token_index(action) == Some(token_index))
    );
}

#[test]
fn h10_h11_candidate_has_zero_lifecycle_text_or_identity_mutation() {
    let source = SourceText::new(SourceId::new(1), "<body><div><p>x</html>".to_owned());
    let run = tokenize(&source, limits());
    let mut machine = Machine::new(StorageLayout::COMPACT);
    let candidate_index = run
        .tokens()
        .iter()
        .position(|token| matches!(token, HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::End && tag.name().interpreted().eq_ignore_ascii_case("html")))
        .expect("html end token");

    for (token_index, token) in run.tokens()[..candidate_index].iter().enumerate() {
        assert_eq!(machine.step(token_index, token), Ok(Step::Consumed));
        machine.commit(token);
    }
    let before = machine.snapshot();
    assert_eq!(
        machine.step(candidate_index, &run.tokens()[candidate_index]),
        Ok(Step::Consumed)
    );
    machine.commit(&run.tokens()[candidate_index]);
    let after = machine.snapshot();

    assert_eq!(after.open, before.open);
    assert_eq!(after.next_id, before.next_id);
    assert_eq!(after.nodes, before.nodes);
    let new_actions = &after.actions[before.actions.len()..];
    assert!(
        new_actions
            .iter()
            .all(|action| !matches!(action, Action::Insert { .. } | Action::TextInsert { .. }))
    );

    let mut baseline_probe = Machine {
        slots: machine.slots.clone(),
        layout: machine.layout,
        next_id: before.next_id,
        open: before.open.clone(),
        phase: Phase::InBody,
        diagnostics: before.diagnostics.clone(),
        actions: before.actions.clone(),
        committed_end: before.committed_end,
        processed_tokens: before.processed_tokens,
        reprocess_count: before.reprocess_count,
    };
    let mut post_probe = machine;
    assert_eq!(baseline_probe.allocate_probe(), post_probe.allocate_probe());
}

#[test]
fn h12_h15_after_after_body_eof_is_phase_sensitive_and_preserves_open_stack() {
    for source in [
        "<body></html>",
        "<body><p></html>",
        "<body><div></html>",
        "<body><div><p></html>",
    ] {
        let observation = observe(source);
        assert_eq!(observation.completion, Completion::Complete, "{source}");
        assert_eq!(observation.phase, Phase::AfterAfterBody, "{source}");
        assert!(observation.actions.iter().any(|action| matches!(
            action,
            Action::StopAtEof {
                phase: Phase::AfterAfterBody,
                ..
            }
        )));
        assert_eq!(
            observation
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.kind == DiagnosticKind::HtmlEndAudit)
                .count(),
            usize::from(open_names(&observation).iter().any(|name| name.is_block()))
        );
    }
}

#[test]
fn h16_tc_s7_body_end_predecessor_control_stays_one_phase() {
    let observation = observe("<body><div><p>x</body>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.phase, Phase::AfterBody);
    assert_eq!(
        diagnostic_count(&observation, DiagnosticKind::BodyEndAudit),
        1
    );
    assert_eq!(
        diagnostic_count(&observation, DiagnosticKind::HtmlEndAudit),
        0
    );
    assert_eq!(observation.reprocess_count, 0);
    assert_eq!(
        open_names(&observation),
        vec![Name::Html, Name::Body, Name::Div, Name::P]
    );
}

#[test]
fn h17_direct_after_body_html_end_has_no_in_body_audit_phase() {
    let observation = observe("<body><div></body></html>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.phase, Phase::AfterAfterBody);
    assert_eq!(
        diagnostic_count(&observation, DiagnosticKind::BodyEndAudit),
        1
    );
    assert_eq!(
        diagnostic_count(&observation, DiagnosticKind::HtmlEndAudit),
        0
    );
    assert_eq!(observation.reprocess_count, 0);
    let html_index = observation
        .actions
        .iter()
        .filter_map(|action| match action {
            Action::Dispatch {
                phase: Phase::AfterBody,
                token_index,
                ..
            } => Some(*token_index),
            _ => None,
        })
        .next_back()
        .expect("direct after body html dispatch");
    assert_eq!(
        observation
            .actions
            .iter()
            .filter(|action| matches!(action, Action::Dispatch { phase: Phase::InBody, token_index, .. } if *token_index == html_index))
            .count(),
        0
    );
}

#[test]
fn h18_h24_excluded_shapes_and_after_after_body_successors_refuse_transactionally() {
    let cases = [
        (
            "<body><div><body>",
            Unsupported::BodyStartWithOpenBoundedStack,
        ),
        ("<body><div></html x>", Unsupported::HtmlEndAttribute),
        ("<body><div></html/>", Unsupported::HtmlEndSelfClosing),
        ("<body><div></head>", Unsupported::OtherShellEnd),
        (
            "<body><div></html> ",
            Unsupported::AfterAfterBodyCharacterData,
        ),
        (
            "<body><div></html>x",
            Unsupported::AfterAfterBodyCharacterData,
        ),
        ("<body><div></html></div>", Unsupported::AfterAfterBodyTag),
    ];

    for (source, capability) in cases {
        let observation = observe(source);
        assert_transactional_refusal(&observation, capability);
        assert!(
            matches!(observation.completion, Completion::Unsupported { capability: actual, .. } if actual == capability)
        );
    }
}

#[test]
fn h26_audit_diagnostic_is_exact_html_end_trigger_not_inner_origin() {
    let source = "<body><div><p>x</html>";
    let observation = observe_with(source, 91, StorageLayout::COMPACT, limits());
    let token_index = candidate_token_index(&observation);
    let diagnostic = observation
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.kind == DiagnosticKind::HtmlEndAudit)
        .expect("html audit diagnostic");
    assert_eq!(diagnostic.token_index, token_index);
    assert_eq!(diagnostic.trigger.source_id, SourceId::new(91));
    assert_eq!(diagnostic.trigger.range, (15, 22));

    for node in &observation.nodes {
        if let NodeKind::Element {
            origin: Origin::Authored { complete, .. },
            ..
        } = &node.kind
        {
            assert_ne!(complete.range, diagnostic.trigger.range);
        }
    }
}

#[test]
fn h27_source_id_changes_only_authored_evidence_not_normalized_semantics() {
    let source = "<body><section><div><p>x</html>";
    let one = observe_with(source, 1, StorageLayout::COMPACT, limits());
    let two = observe_with(source, 2, StorageLayout::COMPACT, limits());
    assert_eq!(semantic_signature(&one), semantic_signature(&two));
    assert!(
        candidate_trigger_evidence(&one)
            .iter()
            .all(|trigger| trigger.source_id == SourceId::new(1))
    );
    assert!(
        candidate_trigger_evidence(&two)
            .iter()
            .all(|trigger| trigger.source_id == SourceId::new(2))
    );
}

#[test]
fn h28_private_storage_layout_does_not_change_semantic_results() {
    let source = "<body><section><div><p>x</html>";
    let compact = observe_with(source, 5, StorageLayout::COMPACT, limits());
    let padded = observe_with(source, 5, StorageLayout::PADDED, limits());
    assert_eq!(semantic_signature(&compact), semantic_signature(&padded));
    assert_eq!(compact.diagnostics, padded.diagnostics);
    assert_eq!(compact.actions, padded.actions);
}

#[test]
fn h29_lower_layer_incompleteness_is_never_upgraded() {
    let source = SourceText::new(SourceId::new(1), "<body><div></html>xxxxxxxx".to_owned());
    let run = tokenize(&source, HtmlTokenizerLimits::new(1, 1, 1, 1, 1, 1, 1));
    assert!(run.is_incomplete());
    let observation = observe_run(&run, StorageLayout::COMPACT);
    assert_ne!(observation.completion, Completion::Complete);
}

#[test]
fn h30_generated_bounded_stacks_match_independent_closed_form_oracle() {
    let names = [Name::Div, Name::Section];
    for depth in 0..=4 {
        let combinations = 1usize << depth;
        for mask in 0..combinations {
            let blocks: Vec<Name> = (0..depth).map(|index| names[(mask >> index) & 1]).collect();
            for p in [false, true] {
                let source = generated_source(&blocks, p, "");
                let observation = observe(&source);
                let oracle = closed_form_oracle(&blocks, p);
                assert_eq!(observation.completion, Completion::Complete, "{source}");
                assert_eq!(
                    diagnostic_count(&observation, DiagnosticKind::HtmlEndAudit),
                    oracle.audit_diagnostics,
                    "{source}"
                );
                assert_eq!(open_names(&observation), oracle.final_stack, "{source}");
                assert_eq!(
                    observation.reprocess_count, oracle.reprocess_count,
                    "{source}"
                );

                let token_index = candidate_token_index(&observation);
                let candidate_identity_baseline = 2 + blocks.len() + if p { 2 } else { 0 };
                let identity_allocation = observation
                    .next_id
                    .checked_sub(candidate_identity_baseline)
                    .expect("candidate cannot consume pre-existing identity");
                assert_eq!(identity_allocation, oracle.identity_allocation, "{source}");
                assert_eq!(
                    observation.nodes.len(),
                    candidate_identity_baseline,
                    "{source}"
                );
                let lifecycle_mutations = candidate_actions(&observation)
                    .into_iter()
                    .filter(|action| {
                        matches!(action, Action::Insert { .. } | Action::TextInsert { .. })
                    })
                    .count();
                assert_eq!(lifecycle_mutations, oracle.lifecycle_mutations, "{source}");

                let mode_path: Vec<Phase> = observation
                    .actions
                    .iter()
                    .filter_map(|action| match action {
                        Action::Dispatch {
                            phase,
                            token_index: action_token,
                            ..
                        } if *action_token == token_index => Some(*phase),
                        Action::Transition {
                            to,
                            token_index: action_token,
                            ..
                        } if *action_token == token_index => Some(*to),
                        _ => None,
                    })
                    .fold(Vec::new(), |mut phases, phase| {
                        if phases.last() != Some(&phase) {
                            phases.push(phase);
                        }
                        phases
                    });
                assert_eq!(mode_path, oracle.mode_path, "{source}");
            }
        }
    }
}

#[test]
fn exact_committed_coverage_and_processed_token_count_are_pinned() {
    for source in [
        "<body></html>",
        "<body><p>x</html>",
        "<body><div><section><p>x</html>",
    ] {
        let source_text = SourceText::new(SourceId::new(1), source.to_owned());
        let run = tokenize(&source_text, limits());
        let observation = observe_run(&run, StorageLayout::COMPACT);
        assert_eq!(observation.completion, Completion::Complete, "{source}");
        assert_eq!(observation.committed_end, source.len(), "{source}");
        assert_eq!(observation.processed_tokens, run.tokens().len(), "{source}");
    }
}
