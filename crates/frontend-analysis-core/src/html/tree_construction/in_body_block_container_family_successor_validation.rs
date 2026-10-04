//! Candidate-independent validation of the selected in-body block-container
//! family expansion (Issue #894).
//!
//! The question validated here is deliberately narrow: can the accepted
//! selected-ordinary `div | section` lifecycle be widened to exactly
//! `article | aside | footer | header | main | nav` without changing the
//! underlying semantic architecture? This module is validation only. It does
//! not choose production placement or a production representation, and it
//! creates no public or cross-run compatibility promise.
//!
//! Lower layer: the accepted batch tokenizer is consumed only as source-backed
//! token evidence (token order and kind, interpreted and raw tag names, exact
//! complete tag ranges, attribute and self-closing evidence, Character and EOF
//! evidence, and completion state). No production tree-construction semantics
//! (the driver, session, or result modules) are imported or used as an oracle;
//! `validation_module_does_not_import_production_tree_semantics` enforces that
//! on this file's own text.
//!
//! Expected meaning comes from three independent sources:
//!
//! 1. the pinned WHATWG HTML rules (the in-body "address, article, aside, …"
//!    start-tag branch, the matching end-tag branch, "generate implied end
//!    tags", "close a `p` element", the scope definitions), executed by the
//!    private candidate [`Machine`] below, including genuine scope walks rather
//!    than a shortcut over production state;
//! 2. hand-authored GOLD (tree, ordered relation log, diagnostics, raw authored
//!    names, exact source ranges); and
//! 3. two independent oracles for generated cells and programs (a closed-form
//!    suffix oracle and a separately written list-based reference model).
//!
//! Normative authority pins (Issue #894, zero-base selection in #348 comment
//! `5983215720`): WHATWG HTML commit `4c5586afa4fc6f2feeebcd25be1dca017cb51298`,
//! source blob `1286cf2f87000e82ec573794caa8081de5c68917`. WPT and html5lib
//! are challenge/corroboration pins only and define no expected semantics.
//!
//! Bounded candidate configuration: ordinary document, scripting disabled,
//! Data tokenizer start, no context element, HTML namespace only, empty active
//! formatting list, no template insertion-mode stack, no foreign content, and
//! the existing document-shell path ending in `InBody`. Closed candidate stack:
//!
//! ```text
//! [html, body] ++ B* ++ P?
//! B in {Div, Section, Article, Aside, Footer, Header, Main, Nav}
//! count(P) <= 1, and P present => P is current
//! ```
//!
//! Falsification targets exercised (numbering follows Issue #894): (1) the
//! eight names are one rule shape (`all_ordered_name_pairs_share_one_relation_shape`,
//! the spec-list test); (2) an enum shape proves nothing, so semantics are
//! re-derived; (3) targets are found by open-state identity, not source search
//! (`nearest_target_is_open_state_identity_not_source_search`); (4)/(5)
//! closure and recovery pop stay distinct and recovered nodes never receive a
//! fabricated end (`recovery_pops_and_target_closure_are_distinct_relations`);
//! (6) raw casing is retained from tokenizer evidence; (7)/(8) no name needs a
//! different scope/implied-end rule and the bounded paragraph projection is
//! checked against the genuine scope walk on every generated token
//! (`BoundedStackProjectionExceeded` is never produced by supported input);
//! (9)/(10)/(11) attributes, self-closing, and nearby WHATWG names stay
//! refused; (12) ignored ends mutate nothing structural; (13) EOF fabricates
//! nothing; (14) lower-layer incompleteness stays monotonic; (15)-(17) no new
//! phase, tokenizer feedback, or resource dimension is needed (the candidate
//! has none); (18) production output defines no GOLD.

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlTagToken, HtmlToken};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::{
    HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause, HtmlTokenizerRunResult,
};

const PINNED_WHATWG_COMMIT: &str = "4c5586afa4fc6f2feeebcd25be1dca017cb51298";
const PINNED_WHATWG_SOURCE_BLOB: &str = "1286cf2f87000e82ec573794caa8081de5c68917";
const PINNED_WPT_COMMIT: &str = "8e9969fd5559dcff933a1cf4e62e9f5bc16b7231";
const PINNED_HTML5LIB_TESTS_COMMIT: &str = "c777c408b61078ea2eb4acefc2535f54dbc8b28a";

// Hand transcription of the pinned WHATWG lists (in-body start-tag branch,
// in-body end-tag branch, "generate implied end tags", and the HTML-namespace
// members of the "has an element in scope" list). MathML/SVG scope members are
// outside the bounded HTML-only configuration.
#[rustfmt::skip]
const SPEC_START_GROUP: [&str; 25] = [
    "address", "article", "aside", "blockquote", "center", "details", "dialog", "dir", "div",
    "dl", "fieldset", "figcaption", "figure", "footer", "header", "hgroup", "main", "menu",
    "nav", "ol", "p", "search", "section", "summary", "ul",
];
#[rustfmt::skip]
const SPEC_END_GROUP: [&str; 28] = [
    "address", "article", "aside", "blockquote", "button", "center", "details", "dialog", "dir",
    "div", "dl", "fieldset", "figcaption", "figure", "footer", "header", "hgroup", "listing",
    "main", "menu", "nav", "ol", "pre", "search", "section", "select", "summary", "ul",
];
const SPEC_IMPLIED_END: [&str; 10] = [
    "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
];
const SPEC_HTML_SCOPE_BOUNDARIES: [&str; 10] = [
    "applet", "caption", "html", "table", "td", "th", "marquee", "object", "select", "template",
];
const SPEC_BUTTON_SCOPE_EXTRA: [&str; 1] = ["button"];

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
    P,
    Div,
    Section,
    Article,
    Aside,
    Footer,
    Header,
    Main,
    Nav,
}

const SELECTED: [Name; 8] = [
    Name::Div,
    Name::Section,
    Name::Article,
    Name::Aside,
    Name::Footer,
    Name::Header,
    Name::Main,
    Name::Nav,
];

const NON_MAIN_ADDITIONS: [Name; 5] = [
    Name::Article,
    Name::Aside,
    Name::Footer,
    Name::Header,
    Name::Nav,
];

const CANDIDATE_ADDITIONS: [Name; 6] = [
    Name::Article,
    Name::Aside,
    Name::Footer,
    Name::Header,
    Name::Main,
    Name::Nav,
];

impl Name {
    fn as_str(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Head => "head",
            Self::Body => "body",
            Self::P => "p",
            Self::Div => "div",
            Self::Section => "section",
            Self::Article => "article",
            Self::Aside => "aside",
            Self::Footer => "footer",
            Self::Header => "header",
            Self::Main => "main",
            Self::Nav => "nav",
        }
    }

    fn from_interpreted(name: &str) -> Option<Self> {
        [Self::Body, Self::P]
            .into_iter()
            .chain(SELECTED)
            .find(|candidate| candidate.as_str() == name)
    }

    fn is_selected(self) -> bool {
        SELECTED.contains(&self)
    }

    /// The representable subset of the pinned scope-boundary list. Only `html`
    /// can appear in the closed candidate stack.
    fn is_scope_boundary(self) -> bool {
        matches!(self, Self::Html)
    }

    /// The representable subset of the pinned implied-end list. Only `p` can
    /// appear in the closed candidate stack.
    fn is_implied_end(self) -> bool {
        matches!(self, Self::P)
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
    UnmatchedSelectedEnd,
    MisnestedSelectedEnd,
    OpenSelectedAtEof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    kind: DiagnosticKind,
    trigger: Option<Evidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PClosureKind {
    StartTriggered,
    MatchingEnd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PClosure {
    kind: PClosureKind,
    target: NodeId,
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
struct RecoveryPop {
    popped: NodeId,
    target: NodeId,
    token_index: usize,
    trigger: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Closure {
    target: NodeId,
    name: Name,
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
    ParagraphClose {
        kind: PClosureKind,
        target: NodeId,
        trigger: Evidence,
    },
    ImpliedParagraphPop {
        paragraph: NodeId,
        selected_target: NodeId,
        trigger: Evidence,
    },
    SelectedDiagnostic {
        kind: DiagnosticKind,
        name: Name,
        trigger: Evidence,
    },
    SelectedIgnored {
        name: Name,
        trigger: Evidence,
    },
    RecoveryPop {
        popped: NodeId,
        target: NodeId,
        trigger: Evidence,
    },
    SelectedClose {
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
    SelectedAttribute,
    SelectedSelfClosing,
    ParagraphTagShape,
    UnmatchedParagraphEnd,
    SelectedOutsideInBody,
    BodyEndWithOpenParagraph,
    BodyEndWithOpenSelected,
    BoundedStackProjectionExceeded,
    GenericTag,
    OutsideCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LowerLayerCategory {
    UnsupportedCapability,
    ResourceLimit,
    InvalidConfiguration,
    InternalInvariantFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Completion {
    Complete,
    Unsupported {
        capability: Unsupported,
        token_index: usize,
    },
    LowerLayerIncomplete(LowerLayerCategory),
    MissingEndOfFile,
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
    implied_p_pops: Vec<ImpliedPPop>,
    recoveries: Vec<RecoveryPop>,
    closures: Vec<Closure>,
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
    implied_p_pops: Vec<ImpliedPPop>,
    recoveries: Vec<RecoveryPop>,
    closures: Vec<Closure>,
    actions: Vec<Action>,
    open: Vec<NodeId>,
    phase: Phase,
    next_id: usize,
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum EndDisposition {
    Unmatched,
    Matched {
        target: NodeId,
        implied_pops: Vec<NodeId>,
        recovery: Vec<NodeId>,
    },
}

/// A complete deterministic effect, prepared without mutating the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Plan {
    InsertBody {
        complete: Evidence,
        raw_name: Evidence,
    },
    Characters {
        interpreted: String,
        contribution: Evidence,
    },
    Eof {
        trigger: Evidence,
    },
    Start {
        name: Name,
        close_paragraph: Option<NodeId>,
        complete: Evidence,
        raw_name: Evidence,
    },
    EndParagraph {
        target: NodeId,
        trigger: Evidence,
    },
    EndSelected {
        name: Name,
        trigger: Evidence,
        disposition: EndDisposition,
    },
    EnterAfterBody,
    AfterBodyEof,
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
    implied_p_pops: Vec<ImpliedPPop>,
    recoveries: Vec<RecoveryPop>,
    closures: Vec<Closure>,
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
            implied_p_pops: Vec::new(),
            recoveries: Vec::new(),
            closures: Vec::new(),
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

    fn open_names(&self) -> Vec<Name> {
        self.open.iter().map(|id| self.name(*id)).collect()
    }

    fn has_open_selected(&self) -> bool {
        self.open.iter().any(|id| self.name(*id).is_selected())
    }

    fn has_open_paragraph(&self) -> bool {
        self.open.iter().any(|id| self.name(*id) == Name::P)
    }

    /// The Standard's "has an element in a specific scope" walk: the position
    /// of the first matching node from the current node downward, or `None`
    /// when a scope boundary is reached first.
    fn position_in_scope(&self, matches: impl Fn(Name) -> bool) -> Option<usize> {
        for position in (0..self.open.len()).rev() {
            let name = self.name(self.open[position]);
            if matches(name) {
                return Some(position);
            }
            if name.is_scope_boundary() {
                return None;
            }
        }
        None
    }

    fn position_of_same_name_in_scope(&self, wanted: Name) -> Option<usize> {
        self.position_in_scope(|name| name == wanted)
    }

    /// `button` is not representable in the closed candidate stack, so button
    /// scope coincides with ordinary scope here.
    fn position_of_p_in_button_scope(&self) -> Option<usize> {
        self.position_in_scope(|name| name == Name::P)
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
            implied_p_pops: self.implied_p_pops.clone(),
            recoveries: self.recoveries.clone(),
            closures: self.closures.clone(),
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
                        name if name.is_selected() && !saw_p => true,
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
            "bounded stack invariant violated: phase={:?} names={names:?}",
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

    fn pop_expected(&mut self, expected: NodeId) {
        assert_eq!(self.current(), expected, "pop must remove the current node");
        let before = self.open.len();
        self.open.pop();
        assert!(self.open.len() < before, "every pop strictly shortens");
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

    fn process(&mut self, token_index: usize, token: &HtmlToken) -> Result<bool, Unsupported> {
        self.assert_invariant();
        let plan = self.plan(token)?;
        let stop = self.apply(token_index, plan);
        self.assert_invariant();
        Ok(stop)
    }

    /// Classify, resolve the target and full suffix, resolve the retained
    /// trigger, and prepare the complete effect. Takes `&self`: no mutation can
    /// precede a refusal.
    fn plan(&self, token: &HtmlToken) -> Result<Plan, Unsupported> {
        match self.phase {
            Phase::BeforeBody => self.plan_before_body(token),
            Phase::InBody => self.plan_in_body(token),
            Phase::AfterBody => Self::plan_after_body(token),
        }
    }

    fn plan_before_body(&self, token: &HtmlToken) -> Result<Plan, Unsupported> {
        let HtmlToken::Tag(tag) = token else {
            return Err(Unsupported::OutsideCandidate);
        };
        if tag.kind() != HtmlTagKind::Start || tag.name().interpreted() != "body" {
            return Err(Unsupported::OutsideCandidate);
        }
        if !tag.attributes().is_empty() || tag.self_closing_solidus().is_some() {
            return Err(Unsupported::OutsideCandidate);
        }
        Ok(Plan::InsertBody {
            complete: evidence(tag.complete()),
            raw_name: evidence(tag.name().source()),
        })
    }

    fn plan_in_body(&self, token: &HtmlToken) -> Result<Plan, Unsupported> {
        match token {
            HtmlToken::Character(character) => Ok(Plan::Characters {
                interpreted: character.interpreted().to_owned(),
                contribution: evidence(character.source()),
            }),
            HtmlToken::Doctype(_) => Err(Unsupported::OutsideCandidate),
            HtmlToken::EndOfFile(eof) => Ok(Plan::Eof {
                trigger: evidence(eof.source()),
            }),
            HtmlToken::Tag(tag) => {
                let name = Name::from_interpreted(tag.name().interpreted())
                    .ok_or(Unsupported::GenericTag)?;
                reject_tag_shape(name, tag)?;
                let trigger = evidence(tag.complete());
                match (tag.kind(), name) {
                    (HtmlTagKind::Start, Name::Body) => Err(Unsupported::OutsideCandidate),
                    (HtmlTagKind::Start, name) => {
                        // "If the stack has a p element in button scope, close a
                        // p element": under the closed stack theorem the only
                        // admissible case is p current, where "generate implied
                        // end tags except p" is a no-op and the pop-until-p loop
                        // pops exactly p. Anything else would need broader
                        // implied-end semantics and is reported, not extended.
                        let close_paragraph = match self.position_of_p_in_button_scope() {
                            None => None,
                            Some(position) if position + 1 == self.open.len() => {
                                Some(self.open[position])
                            }
                            Some(_) => return Err(Unsupported::BoundedStackProjectionExceeded),
                        };
                        Ok(Plan::Start {
                            name,
                            close_paragraph,
                            complete: trigger,
                            raw_name: evidence(tag.name().source()),
                        })
                    }
                    (HtmlTagKind::End, Name::P) => match self.position_of_p_in_button_scope() {
                        None => Err(Unsupported::UnmatchedParagraphEnd),
                        Some(position) if position + 1 == self.open.len() => {
                            Ok(Plan::EndParagraph {
                                target: self.open[position],
                                trigger,
                            })
                        }
                        Some(_) => Err(Unsupported::BoundedStackProjectionExceeded),
                    },
                    (HtmlTagKind::End, Name::Body) => {
                        if self.has_open_paragraph() {
                            Err(Unsupported::BodyEndWithOpenParagraph)
                        } else if self.has_open_selected() {
                            Err(Unsupported::BodyEndWithOpenSelected)
                        } else {
                            Ok(Plan::EnterAfterBody)
                        }
                    }
                    (HtmlTagKind::End, name) => {
                        let disposition = self.plan_selected_end(name)?;
                        Ok(Plan::EndSelected {
                            name,
                            trigger,
                            disposition,
                        })
                    }
                }
            }
        }
    }

    fn plan_selected_end(&self, name: Name) -> Result<EndDisposition, Unsupported> {
        let Some(target_position) = self.position_of_same_name_in_scope(name) else {
            return Ok(EndDisposition::Unmatched);
        };
        let target = self.open[target_position];

        // "Generate implied end tags": while the current node is in the implied
        // list, pop it. The target is a selected name and is never implied-end,
        // so the walk cannot cross it.
        let mut top = self.open.len();
        let mut implied_pops = Vec::new();
        while top > target_position + 1 && self.name(self.open[top - 1]).is_implied_end() {
            implied_pops.push(self.open[top - 1]);
            top -= 1;
        }

        // "Pop elements until the same-name element has been popped": every node
        // above the target is popped. The bounded theorem models only selected
        // nodes there.
        let recovery: Vec<NodeId> = self.open[target_position + 1..top]
            .iter()
            .rev()
            .copied()
            .collect();
        if recovery.iter().any(|id| !self.name(*id).is_selected()) {
            return Err(Unsupported::BoundedStackProjectionExceeded);
        }

        Ok(EndDisposition::Matched {
            target,
            implied_pops,
            recovery,
        })
    }

    fn plan_after_body(token: &HtmlToken) -> Result<Plan, Unsupported> {
        match token {
            HtmlToken::EndOfFile(_) => Ok(Plan::AfterBodyEof),
            HtmlToken::Tag(tag)
                if Name::from_interpreted(tag.name().interpreted())
                    .is_some_and(Name::is_selected) =>
            {
                Err(Unsupported::SelectedOutsideInBody)
            }
            _ => Err(Unsupported::OutsideCandidate),
        }
    }

    /// Commit one prepared effect. Returns whether parsing stops.
    fn apply(&mut self, token_index: usize, plan: Plan) -> bool {
        match plan {
            Plan::InsertBody { complete, raw_name } => {
                let body = self.insert_authored(Name::Body, complete, raw_name);
                assert_eq!(self.current(), body);
                self.phase = Phase::InBody;
                false
            }
            Plan::Characters {
                interpreted,
                contribution,
            } => {
                self.insert_text(&interpreted, contribution);
                false
            }
            Plan::Eof { trigger } => {
                if self.has_open_selected() {
                    self.push_diagnostic(DiagnosticKind::OpenSelectedAtEof, Some(trigger));
                }
                true
            }
            Plan::Start {
                name,
                close_paragraph,
                complete,
                raw_name,
            } => {
                if let Some(paragraph) = close_paragraph {
                    self.pop_expected(paragraph);
                    self.p_closures.push(PClosure {
                        kind: PClosureKind::StartTriggered,
                        target: paragraph,
                        token_index,
                        trigger: complete.clone(),
                    });
                    self.actions.push(Action::ParagraphClose {
                        kind: PClosureKind::StartTriggered,
                        target: paragraph,
                        trigger: complete.clone(),
                    });
                }
                self.insert_authored(name, complete, raw_name);
                false
            }
            Plan::EndParagraph { target, trigger } => {
                self.pop_expected(target);
                self.p_closures.push(PClosure {
                    kind: PClosureKind::MatchingEnd,
                    target,
                    token_index,
                    trigger: trigger.clone(),
                });
                self.actions.push(Action::ParagraphClose {
                    kind: PClosureKind::MatchingEnd,
                    target,
                    trigger,
                });
                false
            }
            Plan::EndSelected {
                name,
                trigger,
                disposition,
            } => {
                self.apply_selected_end(token_index, name, trigger, disposition);
                false
            }
            Plan::EnterAfterBody => {
                self.phase = Phase::AfterBody;
                false
            }
            Plan::AfterBodyEof => true,
        }
    }

    fn apply_selected_end(
        &mut self,
        token_index: usize,
        name: Name,
        trigger: Evidence,
        disposition: EndDisposition,
    ) {
        let EndDisposition::Matched {
            target,
            implied_pops,
            recovery,
        } = disposition
        else {
            self.push_diagnostic(DiagnosticKind::UnmatchedSelectedEnd, Some(trigger.clone()));
            self.actions.push(Action::SelectedDiagnostic {
                kind: DiagnosticKind::UnmatchedSelectedEnd,
                name,
                trigger: trigger.clone(),
            });
            self.actions.push(Action::SelectedIgnored { name, trigger });
            return;
        };

        for paragraph in implied_pops {
            self.pop_expected(paragraph);
            self.implied_p_pops.push(ImpliedPPop {
                paragraph,
                selected_target: target,
                token_index,
                trigger: trigger.clone(),
            });
            self.actions.push(Action::ImpliedParagraphPop {
                paragraph,
                selected_target: target,
                trigger: trigger.clone(),
            });
        }

        if !recovery.is_empty() {
            self.push_diagnostic(DiagnosticKind::MisnestedSelectedEnd, Some(trigger.clone()));
            self.actions.push(Action::SelectedDiagnostic {
                kind: DiagnosticKind::MisnestedSelectedEnd,
                name,
                trigger: trigger.clone(),
            });
        }

        for popped in recovery {
            self.pop_expected(popped);
            self.recoveries.push(RecoveryPop {
                popped,
                target,
                token_index,
                trigger: trigger.clone(),
            });
            self.actions.push(Action::RecoveryPop {
                popped,
                target,
                trigger: trigger.clone(),
            });
        }

        self.pop_expected(target);
        self.closures.push(Closure {
            target,
            name,
            token_index,
            trigger: trigger.clone(),
        });
        self.actions.push(Action::SelectedClose {
            target,
            name,
            trigger,
        });
    }
}

fn reject_tag_shape(name: Name, tag: &HtmlTagToken) -> Result<(), Unsupported> {
    if !tag.attributes().is_empty() {
        return Err(match name {
            Name::P => Unsupported::ParagraphTagShape,
            name if name.is_selected() => Unsupported::SelectedAttribute,
            _ => Unsupported::OutsideCandidate,
        });
    }
    if tag.self_closing_solidus().is_some() {
        return Err(match name {
            Name::P => Unsupported::ParagraphTagShape,
            name if name.is_selected() => Unsupported::SelectedSelfClosing,
            _ => Unsupported::OutsideCandidate,
        });
    }
    Ok(())
}

fn token_end(token: &HtmlToken) -> usize {
    match token {
        HtmlToken::Character(character) => character.source().range().end(),
        HtmlToken::Tag(tag) => tag.complete().range().end(),
        HtmlToken::Doctype(doctype) => doctype.complete().range().end(),
        HtmlToken::EndOfFile(eof) => eof.source().range().end(),
    }
}

fn generous_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn tokenize_with(
    source: &str,
    source_id: u64,
    limits: HtmlTokenizerLimits,
) -> HtmlTokenizerRunResult {
    tokenize(
        &SourceText::new(SourceId::new(source_id), source.to_owned()),
        limits,
    )
}

fn tokenize_source(source: &str, source_id: u64) -> HtmlTokenizerRunResult {
    tokenize_with(source, source_id, generous_limits())
}

fn lower_layer_category(run: &HtmlTokenizerRunResult) -> Option<LowerLayerCategory> {
    match run.completion() {
        HtmlTokenizerCompletion::Complete => None,
        HtmlTokenizerCompletion::Incomplete(cause) => Some(match cause {
            HtmlTokenizerIncompleteCause::UnsupportedCapability(_) => {
                LowerLayerCategory::UnsupportedCapability
            }
            HtmlTokenizerIncompleteCause::ResourceLimit(_) => LowerLayerCategory::ResourceLimit,
            HtmlTokenizerIncompleteCause::InvalidConfiguration(_) => {
                LowerLayerCategory::InvalidConfiguration
            }
            HtmlTokenizerIncompleteCause::InternalInvariantFailure(_) => {
                LowerLayerCategory::InternalInvariantFailure
            }
        }),
    }
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
                assert!(
                    machine.committed_end <= run.coverage().processed_end(),
                    "candidate must not claim evidence beyond lower-layer coverage"
                );
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
    } else if let Some(category) = lower_layer_category(run) {
        Completion::LowerLayerIncomplete(category)
    } else if stopped && machine.processed_tokens == run.tokens().len() {
        Completion::Complete
    } else {
        Completion::MissingEndOfFile
    };

    Observation {
        nodes: machine.nodes(),
        diagnostics: machine.diagnostics,
        p_closures: machine.p_closures,
        implied_p_pops: machine.implied_p_pops,
        recoveries: machine.recoveries,
        closures: machine.closures,
        actions: machine.actions,
        open: machine.open,
        phase: machine.phase,
        next_id: machine.next_id,
        completion,
        refusal,
    }
}

fn observe_source(source: &str, source_id: u64) -> Observation {
    observe_with_layout(&tokenize_source(source, source_id), StorageLayout::COMPACT)
}

fn assert_complete(observation: &Observation) {
    assert_eq!(observation.completion, Completion::Complete);
    assert!(observation.refusal.is_none());
}

fn assert_refusal(source: &str, capability: Unsupported) -> Observation {
    let observation = observe_source(source, 1);
    let refusal = observation
        .refusal
        .clone()
        .unwrap_or_else(|| panic!("expected refusal for {source:?}: {observation:?}"));
    assert_eq!(refusal.capability, capability, "{source:?}");
    assert_eq!(refusal.before, refusal.after, "{source:?}");
    assert_eq!(
        observation.completion,
        Completion::Unsupported {
            capability,
            token_index: refusal.token_index,
        },
        "{source:?}"
    );
    observation
}

// ---------------------------------------------------------------------------
// Test-side projections. These render an observation to compact strings so the
// hand-authored GOLD below stays reviewable. They define no semantics.
// ---------------------------------------------------------------------------

fn range_text(evidence: &Evidence) -> String {
    format!("{}-{}", evidence.range.0, evidence.range.1)
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

fn open_names(observation: &Observation) -> Vec<&'static str> {
    observation
        .open
        .iter()
        .map(|id| node_name(observation, *id).as_str())
        .collect()
}

fn render_node(observation: &Observation, id: NodeId) -> String {
    let node = observation
        .nodes
        .iter()
        .find(|node| node.id == id)
        .expect("render node");
    match &node.kind {
        NodeKind::Text { interpreted, .. } => format!("{interpreted:?}"),
        NodeKind::Element { name, .. } => {
            let children: Vec<String> = observation
                .nodes
                .iter()
                .filter(|child| child.parent == Some(id))
                .map(|child| render_node(observation, child.id))
                .collect();
            if children.is_empty() {
                name.as_str().to_owned()
            } else {
                format!("{}({})", name.as_str(), children.join(","))
            }
        }
        NodeKind::Document => panic!("document is not rendered"),
    }
}

fn render_tree(observation: &Observation) -> String {
    render_node(observation, NodeId(1))
}

fn render_actions(observation: &Observation) -> Vec<String> {
    let name = |id: &NodeId| node_name(observation, *id).as_str();
    observation
        .actions
        .iter()
        // Body creation is predecessor shell semantics; GOLD logs start after it.
        .filter(|action| {
            !matches!(
                action,
                Action::AuthoredInsert {
                    name: Name::Body,
                    ..
                }
            )
        })
        .map(|action| match action {
            Action::AuthoredInsert {
                node,
                name: actual,
                trigger,
            } => format!(
                "insert {}#{}@{}",
                actual.as_str(),
                node.0,
                range_text(trigger)
            ),
            Action::ParagraphClose {
                kind,
                target,
                trigger,
            } => format!(
                "p-close[{}] #{}@{}",
                match kind {
                    PClosureKind::StartTriggered => "start",
                    PClosureKind::MatchingEnd => "end",
                },
                target.0,
                range_text(trigger)
            ),
            Action::ImpliedParagraphPop {
                paragraph,
                selected_target,
                trigger,
            } => format!(
                "implied-p-pop #{} for {}#{}@{}",
                paragraph.0,
                name(selected_target),
                selected_target.0,
                range_text(trigger)
            ),
            Action::SelectedDiagnostic {
                kind,
                name: actual,
                trigger,
            } => format!(
                "diag {} {}@{}",
                match kind {
                    DiagnosticKind::UnmatchedSelectedEnd => "unmatched",
                    DiagnosticKind::MisnestedSelectedEnd => "misnested",
                    other => panic!("unexpected selected diagnostic {other:?}"),
                },
                actual.as_str(),
                range_text(trigger)
            ),
            Action::SelectedIgnored {
                name: actual,
                trigger,
            } => format!("ignore {}@{}", actual.as_str(), range_text(trigger)),
            Action::RecoveryPop {
                popped,
                target,
                trigger,
            } => format!(
                "recover-pop {}#{} -> {}#{}@{}",
                name(popped),
                popped.0,
                name(target),
                target.0,
                range_text(trigger)
            ),
            Action::SelectedClose {
                target,
                name: actual,
                trigger,
            } => format!(
                "close {}#{}@{}",
                actual.as_str(),
                target.0,
                range_text(trigger)
            ),
            Action::TextInsert {
                node,
                parent,
                contribution,
            } => format!(
                "text-new #{} in #{}@{}",
                node.0,
                parent.0,
                range_text(contribution)
            ),
            Action::TextAppend {
                node,
                parent,
                contribution,
            } => format!(
                "text-append #{} in #{}@{}",
                node.0,
                parent.0,
                range_text(contribution)
            ),
        })
        .collect()
}

fn render_diagnostics(observation: &Observation) -> Vec<String> {
    observation
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let kind = match diagnostic.kind {
                DiagnosticKind::MissingDoctype => "MissingDoctype",
                DiagnosticKind::UnmatchedSelectedEnd => "UnmatchedSelectedEnd",
                DiagnosticKind::MisnestedSelectedEnd => "MisnestedSelectedEnd",
                DiagnosticKind::OpenSelectedAtEof => "OpenSelectedAtEof",
            };
            match &diagnostic.trigger {
                Some(trigger) => format!("{kind}@{}", range_text(trigger)),
                None => kind.to_owned(),
            }
        })
        .collect()
}

fn authored_raw_names(source: &str, observation: &Observation) -> Vec<String> {
    observation
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Element {
                origin: Origin::Authored { complete, raw_name },
                ..
            } => {
                assert!(complete.range.0 <= raw_name.range.0);
                assert!(raw_name.range.1 <= complete.range.1);
                Some(source[raw_name.range.0..raw_name.range.1].to_owned())
            }
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Hand-authored GOLD. Node identities: document #0, html #1, head #2, body #3,
// then one identity per accepted selected start or new text node in creation
// order. Every expectation below was derived from the pinned rules and the
// source text, never from running production tree construction.
// ---------------------------------------------------------------------------

struct Gold {
    id: &'static str,
    source: &'static str,
    tree: &'static str,
    log: &'static [&'static str],
    diagnostics: &'static [&'static str],
    open: &'static [&'static str],
    raw_names: &'static [&'static str],
}

#[rustfmt::skip]
const GOLD: &[Gold] = &[
    // Basic creation and closure: every candidate addition individually.
    Gold { id: "A1", source: "<body><article></article>", tree: "html(head,body(article))",
        log: &["insert article#4@6-15", "close article#4@15-25"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "article"] },
    Gold { id: "A2", source: "<body><aside></aside>", tree: "html(head,body(aside))",
        log: &["insert aside#4@6-13", "close aside#4@13-21"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "aside"] },
    Gold { id: "A3", source: "<body><footer></footer>", tree: "html(head,body(footer))",
        log: &["insert footer#4@6-14", "close footer#4@14-23"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "footer"] },
    Gold { id: "A4", source: "<body><header></header>", tree: "html(head,body(header))",
        log: &["insert header#4@6-14", "close header#4@14-23"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "header"] },
    Gold { id: "A5", source: "<body><main></main>", tree: "html(head,body(main))",
        log: &["insert main#4@6-12", "close main#4@12-19"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "main"] },
    Gold { id: "A6", source: "<body><nav></nav>", tree: "html(head,body(nav))",
        log: &["insert nav#4@6-11", "close nav#4@11-17"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "nav"] },
    // Mixed-case authored provenance.
    Gold { id: "M1", source: "<body><ArTiClE>x</aRtIcLe>", tree: "html(head,body(article(\"x\")))",
        log: &["insert article#4@6-15", "text-new #5 in #4@15-16", "close article#4@16-26"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "ArTiClE"] },
    Gold { id: "M2", source: "<body><NaV>x</nAv>", tree: "html(head,body(nav(\"x\")))",
        log: &["insert nav#4@6-11", "text-new #5 in #4@11-12", "close nav#4@12-18"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "NaV"] },
    // Same-name nesting.
    Gold { id: "N1", source: "<body><article><article>x</article></article>",
        tree: "html(head,body(article(article(\"x\"))))",
        log: &["insert article#4@6-15", "insert article#5@15-24", "text-new #6 in #5@24-25",
               "close article#5@25-35", "close article#4@35-45"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"],
        raw_names: &["body", "article", "article"] },
    Gold { id: "N2", source: "<body><nav><nav>x</nav></nav>", tree: "html(head,body(nav(nav(\"x\"))))",
        log: &["insert nav#4@6-11", "insert nav#5@11-16", "text-new #6 in #5@16-17",
               "close nav#5@17-23", "close nav#4@23-29"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "nav", "nav"] },
    // Existing -> new and new -> existing composition (well nested).
    Gold { id: "H1", source: "<body><div><article></article></div>", tree: "html(head,body(div(article)))",
        log: &["insert div#4@6-11", "insert article#5@11-20", "close article#5@20-30", "close div#4@30-36"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "div", "article"] },
    Gold { id: "H2", source: "<body><section><main></main></section>", tree: "html(head,body(section(main)))",
        log: &["insert section#4@6-15", "insert main#5@15-21", "close main#5@21-28", "close section#4@28-38"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "section", "main"] },
    Gold { id: "H3", source: "<body><article><section></section></article>", tree: "html(head,body(article(section)))",
        log: &["insert article#4@6-15", "insert section#5@15-24", "close section#5@24-34", "close article#4@34-44"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "article", "section"] },
    Gold { id: "H4", source: "<body><nav><div></div></nav>", tree: "html(head,body(nav(div)))",
        log: &["insert nav#4@6-11", "insert div#5@11-16", "close div#5@16-22", "close nav#4@22-28"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "nav", "div"] },
    // Existing/new heterogeneous recovery.
    Gold { id: "R1", source: "<body><section><main></section>", tree: "html(head,body(section(main)))",
        log: &["insert section#4@6-15", "insert main#5@15-21", "diag misnested section@21-31",
               "recover-pop main#5 -> section#4@21-31", "close section#4@21-31"],
        diagnostics: &["MissingDoctype", "MisnestedSelectedEnd@21-31"], open: &["html", "body"],
        raw_names: &["body", "section", "main"] },
    Gold { id: "R2", source: "<body><nav><div></nav>", tree: "html(head,body(nav(div)))",
        log: &["insert nav#4@6-11", "insert div#5@11-16", "diag misnested nav@16-22",
               "recover-pop div#5 -> nav#4@16-22", "close nav#4@16-22"],
        diagnostics: &["MissingDoctype", "MisnestedSelectedEnd@16-22"], open: &["html", "body"],
        raw_names: &["body", "nav", "div"] },
    // New/new heterogeneous recovery with nearest-target and ordered pops.
    Gold { id: "R3", source: "<body><article><nav></article>", tree: "html(head,body(article(nav)))",
        log: &["insert article#4@6-15", "insert nav#5@15-20", "diag misnested article@20-30",
               "recover-pop nav#5 -> article#4@20-30", "close article#4@20-30"],
        diagnostics: &["MissingDoctype", "MisnestedSelectedEnd@20-30"], open: &["html", "body"],
        raw_names: &["body", "article", "nav"] },
    Gold { id: "R4", source: "<body><header><main><aside></header>",
        tree: "html(head,body(header(main(aside))))",
        log: &["insert header#4@6-14", "insert main#5@14-20", "insert aside#6@20-27",
               "diag misnested header@27-36", "recover-pop aside#6 -> header#4@27-36",
               "recover-pop main#5 -> header#4@27-36", "close header#4@27-36"],
        diagnostics: &["MissingDoctype", "MisnestedSelectedEnd@27-36"], open: &["html", "body"],
        raw_names: &["body", "header", "main", "aside"] },
    Gold { id: "R5", source: "<body><main><nav><main></nav></main>",
        tree: "html(head,body(main(nav(main))))",
        log: &["insert main#4@6-12", "insert nav#5@12-17", "insert main#6@17-23",
               "diag misnested nav@23-29", "recover-pop main#6 -> nav#5@23-29", "close nav#5@23-29",
               "close main#4@29-36"],
        diagnostics: &["MissingDoctype", "MisnestedSelectedEnd@23-29"], open: &["html", "body"],
        raw_names: &["body", "main", "nav", "main"] },
    // Paragraph composition.
    Gold { id: "P1", source: "<body><p>x<article>y</article>", tree: "html(head,body(p(\"x\"),article(\"y\")))",
        log: &["insert p#4@6-9", "text-new #5 in #4@9-10", "p-close[start] #4@10-19",
               "insert article#6@10-19", "text-new #7 in #6@19-20", "close article#6@20-30"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "p", "article"] },
    Gold { id: "P2", source: "<body><p>x<nav>y</nav>", tree: "html(head,body(p(\"x\"),nav(\"y\")))",
        log: &["insert p#4@6-9", "text-new #5 in #4@9-10", "p-close[start] #4@10-15",
               "insert nav#6@10-15", "text-new #7 in #6@15-16", "close nav#6@16-22"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "p", "nav"] },
    Gold { id: "P3", source: "<body><article><p>x</article>", tree: "html(head,body(article(p(\"x\"))))",
        log: &["insert article#4@6-15", "insert p#5@15-18", "text-new #6 in #5@18-19",
               "implied-p-pop #5 for article#4@19-29", "close article#4@19-29"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "article", "p"] },
    Gold { id: "P4", source: "<body><header><main><p>x</header>",
        tree: "html(head,body(header(main(p(\"x\")))))",
        log: &["insert header#4@6-14", "insert main#5@14-20", "insert p#6@20-23",
               "text-new #7 in #6@23-24", "implied-p-pop #6 for header#4@24-33",
               "diag misnested header@24-33", "recover-pop main#5 -> header#4@24-33",
               "close header#4@24-33"],
        diagnostics: &["MissingDoctype", "MisnestedSelectedEnd@24-33"], open: &["html", "body"],
        raw_names: &["body", "header", "main", "p"] },
    Gold { id: "P5", source: "<body><aside><p>x</p></aside>", tree: "html(head,body(aside(p(\"x\"))))",
        log: &["insert aside#4@6-13", "insert p#5@13-16", "text-new #6 in #5@16-17",
               "p-close[end] #5@17-21", "close aside#4@21-29"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "aside", "p"] },
    // Unmatched ends.
    Gold { id: "U1", source: "<body></article>", tree: "html(head,body)",
        log: &["diag unmatched article@6-16", "ignore article@6-16"],
        diagnostics: &["MissingDoctype", "UnmatchedSelectedEnd@6-16"], open: &["html", "body"],
        raw_names: &["body"] },
    Gold { id: "U2", source: "<body></nav>", tree: "html(head,body)",
        log: &["diag unmatched nav@6-12", "ignore nav@6-12"],
        diagnostics: &["MissingDoctype", "UnmatchedSelectedEnd@6-12"], open: &["html", "body"],
        raw_names: &["body"] },
    Gold { id: "U3", source: "<body><div></aside></div>", tree: "html(head,body(div))",
        log: &["insert div#4@6-11", "diag unmatched aside@11-19", "ignore aside@11-19",
               "close div#4@19-25"],
        diagnostics: &["MissingDoctype", "UnmatchedSelectedEnd@11-19"], open: &["html", "body"],
        raw_names: &["body", "div"] },
    Gold { id: "U4", source: "<body><p>x</footer>", tree: "html(head,body(p(\"x\")))",
        log: &["insert p#4@6-9", "text-new #5 in #4@9-10", "diag unmatched footer@10-19",
               "ignore footer@10-19"],
        diagnostics: &["MissingDoctype", "UnmatchedSelectedEnd@10-19"], open: &["html", "body", "p"],
        raw_names: &["body", "p"] },
    // EOF-open selected suffixes: no fabricated end, closure, or recovery pop.
    Gold { id: "E1", source: "<body><article>x", tree: "html(head,body(article(\"x\")))",
        log: &["insert article#4@6-15", "text-new #5 in #4@15-16"],
        diagnostics: &["MissingDoctype", "OpenSelectedAtEof@16-16"], open: &["html", "body", "article"],
        raw_names: &["body", "article"] },
    Gold { id: "E2", source: "<body><div><nav>x", tree: "html(head,body(div(nav(\"x\"))))",
        log: &["insert div#4@6-11", "insert nav#5@11-16", "text-new #6 in #5@16-17"],
        diagnostics: &["MissingDoctype", "OpenSelectedAtEof@17-17"],
        open: &["html", "body", "div", "nav"], raw_names: &["body", "div", "nav"] },
    // Siblings and text before/inside/after selected nodes.
    Gold { id: "S1", source: "<body><article>x</article>y<nav>z</nav>",
        tree: "html(head,body(article(\"x\"),\"y\",nav(\"z\")))",
        log: &["insert article#4@6-15", "text-new #5 in #4@15-16", "close article#4@16-26",
               "text-new #6 in #3@26-27", "insert nav#7@27-32", "text-new #8 in #7@32-33",
               "close nav#7@33-39"],
        diagnostics: &["MissingDoctype"], open: &["html", "body"], raw_names: &["body", "article", "nav"] },
    // Sibling construction after recovery: the recovered main gets no closure
    // and a later sibling starts under the shared body.
    Gold { id: "S2", source: "<body><nav><main></nav><aside></aside>",
        tree: "html(head,body(nav(main),aside))",
        log: &["insert nav#4@6-11", "insert main#5@11-17", "diag misnested nav@17-23",
               "recover-pop main#5 -> nav#4@17-23", "close nav#4@17-23", "insert aside#6@23-30",
               "close aside#6@30-38"],
        diagnostics: &["MissingDoctype", "MisnestedSelectedEnd@17-23"], open: &["html", "body"],
        raw_names: &["body", "nav", "main", "aside"] },
];

fn gold(id: &str) -> &'static Gold {
    GOLD.iter()
        .find(|gold| gold.id == id)
        .expect("gold fixture")
}

// ---------------------------------------------------------------------------
// Independent generated-cell oracle: a closed-form suffix computation over
// name lists. It shares no code with the machine.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct CellOutcome {
    unmatched: bool,
    implied_p_pop: bool,
    misnested: bool,
    recovery_names_current_first: Vec<Name>,
    closes_target: bool,
    final_blocks: Vec<Name>,
    final_p: bool,
}

fn closed_form_oracle(blocks: &[Name], p_current: bool, end: Name) -> CellOutcome {
    let Some(position) = blocks.iter().rposition(|name| *name == end) else {
        return CellOutcome {
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
    CellOutcome {
        unmatched: false,
        implied_p_pop: p_current,
        misnested: !recovery_names_current_first.is_empty(),
        recovery_names_current_first,
        closes_target: true,
        final_blocks: blocks[..position].to_vec(),
        final_p: false,
    }
}

fn machine_cell_outcome(blocks: &[Name], p_current: bool, end: Name) -> CellOutcome {
    let mut source = String::from("<body>");
    for name in blocks {
        source.push_str(&format!("<{}>", name.as_str()));
    }
    if p_current {
        source.push_str("<p>");
    }
    let trigger_start = source.len();
    source.push_str(&format!("</{}>", end.as_str()));
    let trigger_end = source.len();

    let run = tokenize_source(&source, 1);
    let mut machine = Machine::new(StorageLayout::COMPACT);
    let mut trigger = None;
    for (token_index, token) in run.tokens().iter().enumerate() {
        let stop = machine
            .process(token_index, token)
            .unwrap_or_else(|capability| {
                panic!("generated cell refused: {source:?} {capability:?}")
            });
        assert!(!stop, "{source:?}");
        machine.commit(token);
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
    let on_trigger = |action: &&Action| match action {
        Action::ImpliedParagraphPop {
            trigger: actual, ..
        }
        | Action::SelectedDiagnostic {
            trigger: actual, ..
        }
        | Action::SelectedIgnored {
            trigger: actual, ..
        }
        | Action::RecoveryPop {
            trigger: actual, ..
        }
        | Action::SelectedClose {
            trigger: actual, ..
        } => actual == &trigger,
        _ => false,
    };
    let actions: Vec<&Action> = machine.actions.iter().filter(on_trigger).collect();

    let open_names = machine.open_names();
    CellOutcome {
        unmatched: actions
            .iter()
            .any(|action| matches!(action, Action::SelectedIgnored { .. })),
        implied_p_pop: actions
            .iter()
            .any(|action| matches!(action, Action::ImpliedParagraphPop { .. })),
        misnested: actions.iter().any(|action| {
            matches!(
                action,
                Action::SelectedDiagnostic {
                    kind: DiagnosticKind::MisnestedSelectedEnd,
                    ..
                }
            )
        }),
        recovery_names_current_first: actions
            .iter()
            .filter_map(|action| match action {
                Action::RecoveryPop { popped, .. } => Some(machine.name(*popped)),
                _ => None,
            })
            .collect(),
        closes_target: actions
            .iter()
            .any(|action| matches!(action, Action::SelectedClose { .. })),
        final_p: open_names.last() == Some(&Name::P),
        final_blocks: open_names
            .into_iter()
            .filter(|name| name.is_selected())
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Independent generated-program reference model: a list-based interpreter that
// shares no code with the machine (no scope walk, no plans, no node arena).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Start(Name),
    End(Name),
    StartP,
    EndP,
    Text(char),
}

struct Lcg(u64);

impl Lcg {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as usize) % bound
    }
}

#[derive(Debug, Default)]
struct RefNode {
    label: String,
    is_text: bool,
    children: Vec<usize>,
}

#[derive(Debug)]
struct RefModel {
    nodes: Vec<RefNode>,
    stack: Vec<usize>,
    unmatched: usize,
    misnested: usize,
    recoveries: usize,
    implied: usize,
}

impl RefModel {
    fn new() -> Self {
        Self {
            nodes: vec![RefNode {
                label: "body".to_owned(),
                ..RefNode::default()
            }],
            stack: vec![0],
            unmatched: 0,
            misnested: 0,
            recoveries: 0,
            implied: 0,
        }
    }

    fn label(&self, position: usize) -> &str {
        &self.nodes[self.stack[position]].label
    }

    fn top_is_p(&self) -> bool {
        self.label(self.stack.len() - 1) == "p"
    }

    fn push(&mut self, label: &str) {
        let id = self.nodes.len();
        self.nodes.push(RefNode {
            label: label.to_owned(),
            ..RefNode::default()
        });
        let parent = *self.stack.last().expect("body stays open");
        self.nodes[parent].children.push(id);
        self.stack.push(id);
    }

    fn apply(&mut self, op: Op) {
        match op {
            Op::Start(name) => {
                if self.top_is_p() {
                    self.stack.pop();
                }
                self.push(name.as_str());
            }
            Op::StartP => {
                if self.top_is_p() {
                    self.stack.pop();
                }
                self.push("p");
            }
            Op::EndP => {
                assert!(self.top_is_p(), "generator only emits matching </p>");
                self.stack.pop();
            }
            Op::End(name) => {
                let wanted = name.as_str();
                let Some(position) = (1..self.stack.len())
                    .rev()
                    .find(|position| self.label(*position) == wanted)
                else {
                    self.unmatched += 1;
                    return;
                };
                if self.top_is_p() {
                    self.implied += 1;
                    self.stack.pop();
                }
                let intervening = self.stack.len() - 1 - position;
                if intervening > 0 {
                    self.misnested += 1;
                    self.recoveries += intervening;
                }
                self.stack.truncate(position);
            }
            Op::Text(character) => {
                let parent = *self.stack.last().expect("body stays open");
                if let Some(last) = self.nodes[parent].children.last().copied()
                    && self.nodes[last].is_text
                {
                    self.nodes[last].label.push(character);
                    return;
                }
                let id = self.nodes.len();
                self.nodes.push(RefNode {
                    label: character.to_string(),
                    is_text: true,
                    children: Vec::new(),
                });
                self.nodes[parent].children.push(id);
            }
        }
    }

    fn render(&self, id: usize) -> String {
        let node = &self.nodes[id];
        if node.is_text {
            return format!("{:?}", node.label);
        }
        if node.children.is_empty() {
            node.label.clone()
        } else {
            let children: Vec<String> = node
                .children
                .iter()
                .map(|child| self.render(*child))
                .collect();
            format!("{}({})", node.label, children.join(","))
        }
    }

    fn open_labels(&self) -> Vec<String> {
        (0..self.stack.len())
            .map(|position| self.label(position).to_owned())
            .collect()
    }

    fn has_open_selected(&self) -> bool {
        self.open_labels()
            .iter()
            .any(|label| label != "body" && label != "p")
    }
}

fn generate_program(seed: u64, length: usize) -> Vec<Op> {
    let mut rng = Lcg(seed ^ 0x9E37_79B9_7F4A_7C15);
    let mut model = RefModel::new();
    let mut ops = Vec::new();
    for _ in 0..length {
        let roll = rng.next(100);
        let op = if roll < 35 {
            Op::Start(SELECTED[rng.next(SELECTED.len())])
        } else if roll < 65 {
            // Bias ends toward currently open names so recovery is exercised.
            let open: Vec<Name> = (1..model.stack.len())
                .filter_map(|position| Name::from_interpreted(model.label(position)))
                .filter(|name| name.is_selected())
                .collect();
            if !open.is_empty() && rng.next(10) < 7 {
                Op::End(open[rng.next(open.len())])
            } else {
                Op::End(SELECTED[rng.next(SELECTED.len())])
            }
        } else if roll < 75 {
            Op::StartP
        } else if roll < 80 && model.top_is_p() {
            Op::EndP
        } else {
            Op::Text(['x', 'y', 'z'][rng.next(3)])
        };
        model.apply(op);
        ops.push(op);
    }
    ops
}

fn vary_case(name: &str, selector: usize) -> String {
    match selector % 3 {
        0 => name.to_owned(),
        1 => name.to_ascii_uppercase(),
        _ => name
            .chars()
            .enumerate()
            .map(|(index, character)| {
                if index % 2 == 0 {
                    character.to_ascii_uppercase()
                } else {
                    character
                }
            })
            .collect(),
    }
}

fn program_source(ops: &[Op], salt: usize) -> String {
    let mut source = String::from("<body>");
    for (index, op) in ops.iter().enumerate() {
        match op {
            Op::Start(name) => {
                source.push_str(&format!("<{}>", vary_case(name.as_str(), index + salt)))
            }
            Op::End(name) => source.push_str(&format!(
                "</{}>",
                vary_case(name.as_str(), index + salt + 1)
            )),
            Op::StartP => source.push_str("<p>"),
            Op::EndP => source.push_str("</p>"),
            Op::Text(character) => source.push(*character),
        }
    }
    source
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn authority_pins_and_closed_domain_are_exact() {
    assert_eq!(
        PINNED_WHATWG_COMMIT,
        "4c5586afa4fc6f2feeebcd25be1dca017cb51298"
    );
    assert_eq!(
        PINNED_WHATWG_SOURCE_BLOB,
        "1286cf2f87000e82ec573794caa8081de5c68917"
    );
    assert_eq!(
        PINNED_WPT_COMMIT,
        "8e9969fd5559dcff933a1cf4e62e9f5bc16b7231"
    );
    assert_eq!(
        PINNED_HTML5LIB_TESTS_COMMIT,
        "c777c408b61078ea2eb4acefc2535f54dbc8b28a"
    );

    let names: Vec<&str> = SELECTED.iter().map(|name| name.as_str()).collect();
    assert_eq!(
        names,
        [
            "div", "section", "article", "aside", "footer", "header", "main", "nav"
        ]
    );
    let additions: Vec<&str> = CANDIDATE_ADDITIONS
        .iter()
        .map(|name| name.as_str())
        .collect();
    assert_eq!(
        additions,
        ["article", "aside", "footer", "header", "main", "nav"]
    );
    for addition in CANDIDATE_ADDITIONS {
        assert!(addition.is_selected());
        assert!(![Name::Div, Name::Section].contains(&addition));
    }
    assert_eq!(
        SELECTED.len(),
        CANDIDATE_ADDITIONS.len() + 2,
        "the selected domain is exactly div | section plus the six additions"
    );
    assert!(!Name::P.is_selected());
    assert!(!Name::Body.is_selected());
    assert!(!Name::Html.is_selected());
}

#[test]
fn validation_module_does_not_import_production_tree_semantics() {
    let source = include_str!("in_body_block_container_family_successor_validation.rs");
    let forbidden = [
        ["use super::", "driver"].concat(),
        ["use super::", "session"].concat(),
        ["use super::", "result"].concat(),
        ["super::", "driver"].concat(),
        ["super::", "session"].concat(),
        ["super::", "result"].concat(),
        ["tree_construction", "::driver"].concat(),
        ["tree_construction", "::session"].concat(),
        ["tree_construction", "::result"].concat(),
        ["tree_construction", "::validation"].concat(),
    ];
    for forbidden in forbidden {
        assert!(
            !source.contains(&forbidden),
            "forbidden production oracle: {forbidden}"
        );
    }
}

#[test]
fn all_eight_names_sit_in_one_rule_family_and_in_no_scope_or_implied_end_list() {
    assert_eq!(SPEC_START_GROUP.len(), 25);
    assert_eq!(SPEC_END_GROUP.len(), 28);
    assert_eq!(SPEC_IMPLIED_END.len(), 10);
    for name in SELECTED {
        let text = name.as_str();
        assert!(SPEC_START_GROUP.contains(&text), "{text} start rule family");
        assert!(SPEC_END_GROUP.contains(&text), "{text} end rule family");
        assert!(
            !SPEC_IMPLIED_END.contains(&text),
            "{text} must not be implied-end"
        );
        assert!(
            !SPEC_HTML_SCOPE_BOUNDARIES.contains(&text),
            "{text} must not be a scope boundary"
        );
        assert!(
            !SPEC_BUTTON_SCOPE_EXTRA.contains(&text),
            "{text} must not be a button-scope boundary"
        );
        assert!(!name.is_scope_boundary(), "{text}");
        assert!(!name.is_implied_end(), "{text}");
    }
    // The representable boundary/implied-end subsets mirror the pinned lists
    // restricted to the closed stack alphabet.
    for name in [Name::Html, Name::Head, Name::Body, Name::P]
        .into_iter()
        .chain(SELECTED)
    {
        assert_eq!(
            name.is_scope_boundary(),
            SPEC_HTML_SCOPE_BOUNDARIES.contains(&name.as_str())
        );
        assert_eq!(
            name.is_implied_end(),
            SPEC_IMPLIED_END.contains(&name.as_str())
        );
    }
}

/// (name, start complete, start raw, end complete, end raw)
type EvidenceRow = (
    Name,
    (usize, usize),
    (usize, usize),
    (usize, usize),
    (usize, usize),
);

#[test]
fn tokenizer_evidence_for_each_name_matches_hand_ranges() {
    #[rustfmt::skip]
    let table: [EvidenceRow; 6] = [
        (Name::Article, (6, 15), (7, 14), (15, 25), (17, 24)),
        (Name::Aside, (6, 13), (7, 12), (13, 21), (15, 20)),
        (Name::Footer, (6, 14), (7, 13), (14, 23), (16, 22)),
        (Name::Header, (6, 14), (7, 13), (14, 23), (16, 22)),
        (Name::Main, (6, 12), (7, 11), (12, 19), (14, 18)),
        (Name::Nav, (6, 11), (7, 10), (11, 17), (13, 16)),
    ];
    for (name, start_complete, start_raw, end_complete, end_raw) in table {
        let source = format!("<body><{0}></{0}>", name.as_str());
        let run = tokenize_source(&source, 9);
        assert!(!run.is_incomplete());
        let tokens = run.tokens();
        assert_eq!(tokens.len(), 4, "{source}");
        let HtmlToken::Tag(start) = &tokens[1] else {
            panic!("start tag")
        };
        let HtmlToken::Tag(end) = &tokens[2] else {
            panic!("end tag")
        };
        assert_eq!(start.kind(), HtmlTagKind::Start);
        assert_eq!(end.kind(), HtmlTagKind::End);
        assert_eq!(start.name().interpreted(), name.as_str());
        assert_eq!(end.name().interpreted(), name.as_str());
        assert_eq!(
            evidence(start.complete()),
            expected_evidence(9, start_complete)
        );
        assert_eq!(
            evidence(start.name().source()),
            expected_evidence(9, start_raw)
        );
        assert_eq!(evidence(end.complete()), expected_evidence(9, end_complete));
        assert_eq!(evidence(end.name().source()), expected_evidence(9, end_raw));
        assert!(start.attributes().is_empty() && start.self_closing_solidus().is_none());
        assert!(matches!(tokens[3], HtmlToken::EndOfFile(_)));
    }
}

#[test]
fn hand_authored_gold_matches_the_candidate_machine_for_every_canonical_case() {
    for gold in GOLD {
        let observation = observe_source(gold.source, 5);
        assert_complete(&observation);
        assert_eq!(render_tree(&observation), gold.tree, "{} tree", gold.id);
        assert_eq!(
            render_actions(&observation),
            gold.log
                .iter()
                .map(|line| (*line).to_owned())
                .collect::<Vec<_>>(),
            "{} relation log",
            gold.id
        );
        assert_eq!(
            render_diagnostics(&observation),
            gold.diagnostics
                .iter()
                .map(|line| (*line).to_owned())
                .collect::<Vec<_>>(),
            "{} diagnostics",
            gold.id
        );
        assert_eq!(open_names(&observation), gold.open, "{} open", gold.id);
        assert_eq!(
            authored_raw_names(gold.source, &observation),
            gold.raw_names
                .iter()
                .map(|line| (*line).to_owned())
                .collect::<Vec<_>>(),
            "{} raw names",
            gold.id
        );
        assert_eq!(observation.phase, Phase::InBody, "{} stays InBody", gold.id);
    }
    assert_eq!(GOLD.len(), 32);
}

#[test]
fn every_candidate_addition_is_exercised_individually_with_exact_provenance() {
    for name in CANDIDATE_ADDITIONS {
        let source = format!("<body><{0}></{0}>", name.as_str());
        let observation = observe_source(&source, 21);
        assert_complete(&observation);
        let elements = nodes_named(&observation, name);
        assert_eq!(elements.len(), 1, "{source}");
        let NodeKind::Element {
            origin: Origin::Authored { complete, raw_name },
            ..
        } = &elements[0].kind
        else {
            panic!("authored origin")
        };
        let width = name.as_str().len();
        assert_eq!(*complete, expected_evidence(21, (6, 8 + width)));
        assert_eq!(*raw_name, expected_evidence(21, (7, 7 + width)));
        assert_eq!(observation.closures.len(), 1);
        assert_eq!(observation.closures[0].target, elements[0].id);
        assert_eq!(
            observation.closures[0].trigger,
            expected_evidence(21, (8 + width, 11 + 2 * width))
        );
        assert!(observation.recoveries.is_empty());
        assert!(observation.implied_p_pops.is_empty());
        assert!(observation.p_closures.is_empty());
    }
}

#[test]
fn authored_casing_is_retained_as_evidence_and_never_reconstructed() {
    for name in SELECTED {
        for selector in 0..3 {
            let start = vary_case(name.as_str(), selector);
            let end = vary_case(name.as_str(), selector + 1);
            let source = format!("<body><{start}>x</{end}>");
            let run = tokenize_source(&source, 31);
            let observation = observe_with_layout(&run, StorageLayout::COMPACT);
            assert_complete(&observation);
            assert_eq!(observation.closures.len(), 1, "{source}");

            let element = nodes_named(&observation, name)[0];
            let NodeKind::Element {
                origin: Origin::Authored { complete, raw_name },
                ..
            } = &element.kind
            else {
                panic!("authored origin")
            };
            assert_eq!(&source[raw_name.range.0..raw_name.range.1], start);
            assert_eq!(raw_name.range.1 - raw_name.range.0, name.as_str().len());
            assert!(complete.range.0 < raw_name.range.0);
            assert!(start.eq_ignore_ascii_case(name.as_str()));

            // The matching end retains its own authored casing as the trigger
            // evidence: the complete tag text, with its raw name inside it.
            let trigger = &observation.closures[0].trigger;
            assert_eq!(
                &source[trigger.range.0..trigger.range.1],
                format!("</{end}>")
            );
        }
    }
}

#[test]
fn recovery_pops_and_target_closure_are_distinct_relations() {
    for id in ["R1", "R2", "R3", "R4", "R5", "P4", "S2"] {
        let observation = observe_source(gold(id).source, 41);
        assert_complete(&observation);
        let recovered: Vec<NodeId> = observation
            .recoveries
            .iter()
            .map(|recovery| recovery.popped)
            .collect();
        let closed: Vec<NodeId> = observation
            .closures
            .iter()
            .map(|closure| closure.target)
            .collect();
        for node in &recovered {
            assert!(!closed.contains(node), "{id}: recovered node got a closure");
        }
        // One end trigger carries N recovery pops and exactly one target closure.
        for closure in &observation.closures {
            let pops_for_trigger = observation
                .recoveries
                .iter()
                .filter(|recovery| recovery.trigger == closure.trigger)
                .count();
            let closures_for_trigger = observation
                .closures
                .iter()
                .filter(|other| other.trigger == closure.trigger)
                .count();
            assert_eq!(closures_for_trigger, 1, "{id}");
            for recovery in observation
                .recoveries
                .iter()
                .filter(|recovery| recovery.trigger == closure.trigger)
            {
                assert_eq!(recovery.target, closure.target, "{id}");
            }
            if pops_for_trigger > 0 {
                let misnested = observation
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.kind == DiagnosticKind::MisnestedSelectedEnd
                            && diagnostic.trigger.as_ref() == Some(&closure.trigger)
                    })
                    .count();
                assert_eq!(misnested, 1, "{id}: one misnested diagnostic per trigger");
            }
        }
        // A recovered node is never origin-confused with the trigger: its
        // authored origin evidence is its own start tag, not the end trigger.
        for recovery in &observation.recoveries {
            let node = observation
                .nodes
                .iter()
                .find(|node| node.id == recovery.popped)
                .expect("recovered node");
            let NodeKind::Element {
                origin: Origin::Authored { complete, .. },
                ..
            } = &node.kind
            else {
                panic!("authored")
            };
            assert_ne!(*complete, recovery.trigger, "{id}");
        }
    }
}

#[test]
fn nearest_target_is_open_state_identity_not_source_search() {
    // A closed earlier <nav> precedes the open one: a source search for the
    // first <nav> would pick #4, but only #6 is an open same-name target.
    let source = "<body><nav></nav><article><nav><main></nav>";
    let observation = observe_source(source, 51);
    assert_complete(&observation);
    let navs = nodes_named(&observation, Name::Nav);
    assert_eq!(navs.len(), 2);
    let open_nav = navs[1].id;
    assert_eq!(observation.closures.len(), 2);
    assert_eq!(observation.closures[0].target, navs[0].id);
    assert_eq!(observation.closures[1].target, open_nav);
    assert_eq!(observation.recoveries.len(), 1);
    assert_eq!(observation.recoveries[0].target, open_nav);
    assert_eq!(
        node_name(&observation, observation.recoveries[0].popped),
        Name::Main
    );
    assert_eq!(open_names(&observation), ["html", "body", "article"]);

    // Same-name nesting: the nearest (innermost) target is selected.
    let nested = observe_source("<body><aside><aside><footer></aside>", 52);
    assert_complete(&nested);
    let asides = nodes_named(&nested, Name::Aside);
    assert_eq!(nested.closures[0].target, asides[1].id);
    assert_eq!(open_names(&nested), ["html", "body", "aside"]);
}

#[test]
fn unmatched_ends_mutate_nothing_structural_and_allocate_no_identity() {
    let with_ignored = observe_source("<body><div></aside></nav><section>y</section>", 61);
    let control = observe_source("<body><div><section>y</section>", 61);
    assert_complete(&with_ignored);
    assert_complete(&control);
    assert_eq!(with_ignored.next_id, control.next_id);
    assert_eq!(with_ignored.nodes.len(), control.nodes.len());
    assert_eq!(
        nodes_named(&with_ignored, Name::Section)[0].id,
        nodes_named(&control, Name::Section)[0].id,
        "ignored selected ends must not consume semantic identity"
    );
    assert_eq!(with_ignored.closures.len(), control.closures.len());
    assert!(with_ignored.recoveries.is_empty());
    assert!(with_ignored.implied_p_pops.is_empty());
    assert_eq!(
        with_ignored
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == DiagnosticKind::UnmatchedSelectedEnd)
            .count(),
        2
    );

    // Token-level: the fingerprint changes only by the one diagnostic and the
    // two ignore actions; tree, stack, identity counter, and relations stay.
    let run = tokenize_source("<body><article>x</nav>", 62);
    let mut machine = Machine::new(StorageLayout::COMPACT);
    for (index, token) in run.tokens().iter().enumerate().take(3) {
        machine.process(index, token).expect("selected prefix");
        machine.commit(token);
    }
    let before = machine.fingerprint();
    machine
        .process(3, &run.tokens()[3])
        .expect("ignored end is supported");
    machine.commit(&run.tokens()[3]);
    let after = machine.fingerprint();
    assert_eq!(before.nodes, after.nodes);
    assert_eq!(before.open, after.open);
    assert_eq!(before.next_id, after.next_id);
    assert_eq!(before.phase, after.phase);
    assert_eq!(before.closures, after.closures);
    assert_eq!(before.recoveries, after.recoveries);
    assert_eq!(before.implied_p_pops, after.implied_p_pops);
    assert_eq!(before.p_closures, after.p_closures);
    assert_eq!(after.diagnostics.len(), before.diagnostics.len() + 1);
    assert_eq!(after.actions.len(), before.actions.len() + 2);
}

#[test]
fn eof_open_selected_suffixes_fabricate_no_end_closure_or_pop() {
    for name in CANDIDATE_ADDITIONS {
        let source = format!("<body><{0}><{0}>x", name.as_str());
        let observation = observe_source(&source, 71);
        assert_complete(&observation);
        assert!(observation.closures.is_empty(), "{source}");
        assert!(observation.recoveries.is_empty(), "{source}");
        assert!(observation.implied_p_pops.is_empty(), "{source}");
        assert_eq!(
            open_names(&observation),
            ["html", "body", name.as_str(), name.as_str()]
        );
        assert_eq!(
            observation
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.kind == DiagnosticKind::OpenSelectedAtEof)
                .count(),
            1
        );
        assert!(
            !render_actions(&observation)
                .iter()
                .any(|line| line.starts_with("close") || line.starts_with("recover"))
        );
    }
}

#[test]
fn text_parentage_and_coalescing_follow_the_selected_insertion_point() {
    for name in SELECTED {
        let source = format!("<body>a<{0}>bc</{0}>de", name.as_str());
        let observation = observe_source(&source, 81);
        assert_complete(&observation);
        assert_eq!(
            render_tree(&observation),
            format!("html(head,body(\"a\",{}(\"bc\"),\"de\"))", name.as_str()),
            "{source}"
        );
        let body = nodes_named(&observation, Name::Body)[0].id;
        let element = nodes_named(&observation, name)[0].id;
        let texts: Vec<&Node> = observation
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, NodeKind::Text { .. }))
            .collect();
        assert_eq!(texts.len(), 3);
        assert_eq!(texts[0].parent, Some(body));
        assert_eq!(texts[1].parent, Some(element));
        assert_eq!(texts[2].parent, Some(body));

        // Contributions are exact, contiguous, and union to the text run.
        let NodeKind::Text { contributions, .. } = &texts[1].kind else {
            panic!("text")
        };
        let start = 6 + 1 + 2 + name.as_str().len();
        assert_eq!(contributions.first().unwrap().range.0, start);
        assert_eq!(contributions.last().unwrap().range.1, start + 2);
        for pair in contributions.windows(2) {
            assert_eq!(pair[0].range.1, pair[1].range.0);
        }
        // Appends allocate no identity: ids are body#3, text#4, element#5,
        // text#6, text#7.
        assert_eq!(observation.next_id, 8);
    }
}

#[test]
fn identity_is_creation_event_based_and_survives_storage_perturbation() {
    for gold in GOLD {
        let run = tokenize_source(gold.source, 91);
        let compact = observe_with_layout(&run, StorageLayout::COMPACT);
        let padded = observe_with_layout(&run, StorageLayout::PADDED);
        assert_eq!(compact, padded, "{}", gold.id);
    }

    // Identity admission counts: +1 per accepted start and per new text node;
    // +0 for appends, ends, recovery pops, ignored ends, diagnostics, and EOF.
    let observation = observe_source("<body><nav><main>ab</nav></aside>y", 92);
    assert_complete(&observation);
    let allocated = observation.next_id;
    let creations = observation
        .actions
        .iter()
        .filter(|action| {
            matches!(
                action,
                Action::AuthoredInsert { .. } | Action::TextInsert { .. }
            )
        })
        .count();
    // document, html, head are allocated by construction rather than by an action.
    assert_eq!(allocated, 3 + creations);
    assert_eq!(allocated, 3 + 1 + 2 + 1 + 1);
}

#[test]
fn distinct_source_ids_for_equal_bytes_stay_distinct_in_all_evidence() {
    let source = "<body><article><nav>x</article>";
    let first = observe_source(source, 101);
    let second = observe_source(source, 202);
    assert_complete(&first);
    assert_complete(&second);
    assert_ne!(first, second);

    let mut seen = 0;
    for (observation, id) in [(&first, 101), (&second, 202)] {
        for node in &observation.nodes {
            match &node.kind {
                NodeKind::Element {
                    origin: Origin::Authored { complete, raw_name },
                    ..
                } => {
                    assert_eq!(complete.source_id, SourceId::new(id));
                    assert_eq!(raw_name.source_id, SourceId::new(id));
                    seen += 1;
                }
                NodeKind::Text { contributions, .. } => {
                    for contribution in contributions {
                        assert_eq!(contribution.source_id, SourceId::new(id));
                        seen += 1;
                    }
                }
                _ => {}
            }
        }
        for closure in &observation.closures {
            assert_eq!(closure.trigger.source_id, SourceId::new(id));
            seen += 1;
        }
        for recovery in &observation.recoveries {
            assert_eq!(recovery.trigger.source_id, SourceId::new(id));
            seen += 1;
        }
    }
    assert!(seen > 10);

    // Modulo SourceId the semantics are identical and ranges are unchanged.
    assert_eq!(render_actions(&first), render_actions(&second));
    assert_eq!(render_tree(&first), render_tree(&second));
    assert_eq!(first.next_id, second.next_id);
}

#[test]
fn all_ordered_name_pairs_share_one_relation_shape() {
    // Falsification target 1: if any name needed a different rule, the shape of
    // the relation log (names erased) would depend on the name pair.
    fn shape(observation: &Observation) -> Vec<String> {
        render_actions(observation)
            .into_iter()
            .map(|line| {
                let mut line = line;
                for name in SELECTED {
                    line = line.replace(name.as_str(), "N");
                }
                line
            })
            .collect()
    }

    let mut baseline: Option<(Vec<String>, Vec<String>)> = None;
    for outer in SELECTED {
        for inner in SELECTED {
            if outer == inner {
                continue;
            }
            // Name widths differ, so ranges differ; erase them too.
            let source = format!(
                "<body><{outer}><{inner}></{outer}>",
                outer = outer.as_str(),
                inner = inner.as_str()
            );
            let observation = observe_source(&source, 111);
            assert_complete(&observation);
            let erased: Vec<String> = shape(&observation)
                .into_iter()
                .map(|line| line.split('@').next().unwrap().to_owned())
                .collect();
            let diagnostics: Vec<String> = observation
                .diagnostics
                .iter()
                .map(|diagnostic| format!("{:?}", diagnostic.kind))
                .collect();
            match &baseline {
                None => baseline = Some((erased, diagnostics)),
                Some((expected_log, expected_diagnostics)) => {
                    assert_eq!(&erased, expected_log, "{source}");
                    assert_eq!(&diagnostics, expected_diagnostics, "{source}");
                }
            }
        }
    }
    let (log, _) = baseline.expect("pairs");
    assert_eq!(
        log,
        [
            "insert N#4",
            "insert N#5",
            "diag misnested N",
            "recover-pop N#5 -> N#4",
            "close N#4",
        ]
    );
}

#[test]
fn paragraph_projection_agrees_with_the_genuine_scope_walk_for_every_name() {
    for name in SELECTED {
        // p current at a selected start: the genuine "p in button scope" walk
        // finds exactly the current node, so the bounded close is sufficient.
        let source = format!("<body><p>x<{}>y", name.as_str());
        let run = tokenize_source(&source, 121);
        let mut machine = Machine::new(StorageLayout::COMPACT);
        for (index, token) in run.tokens().iter().enumerate() {
            if matches!(token, HtmlToken::EndOfFile(_)) {
                break;
            }
            if let HtmlToken::Tag(tag) = token
                && tag.name().interpreted() == name.as_str()
            {
                let position = machine
                    .position_of_p_in_button_scope()
                    .expect("p is in button scope");
                assert_eq!(position + 1, machine.open.len(), "p must be current");
            }
            machine.process(index, token).expect("bounded paragraph");
            machine.commit(token);
        }
        assert_eq!(machine.open_names(), [Name::Html, Name::Body, name]);

        // p open under selected ends: implied-end generation pops exactly the
        // one paragraph, for every name.
        let source = format!("<body><{0}><p>x</{0}>", name.as_str());
        let observation = observe_source(&source, 122);
        assert_complete(&observation);
        assert_eq!(observation.implied_p_pops.len(), 1, "{source}");
        assert!(observation.recoveries.is_empty(), "{source}");
        assert!(
            observation
                .diagnostics
                .iter()
                .all(|diagnostic| { diagnostic.kind != DiagnosticKind::MisnestedSelectedEnd })
        );
    }

    // Counterexample probe: a stack outside the closed theorem (a paragraph
    // that is not current) cannot be produced by supported tokens. If it ever
    // were, the planner reports it instead of silently extending implied-end
    // handling.
    let mut machine = Machine::new(StorageLayout::COMPACT);
    let run = tokenize_source("<body><article><p>x", 123);
    for (index, token) in run.tokens().iter().enumerate().take(4) {
        machine.process(index, token).expect("prefix");
        machine.commit(token);
    }
    let paragraph = machine.open[3];
    let nav = machine.allocate(
        Some(paragraph),
        NodeKind::Element {
            name: Name::Nav,
            origin: Origin::Synthesized(SynthesisCause::ImpliedHead),
        },
    );
    machine.open.push(nav);
    assert_eq!(machine.name(machine.open[3]), Name::P);
    assert_eq!(
        machine.plan_selected_end(Name::Article),
        Err(Unsupported::BoundedStackProjectionExceeded)
    );
}

#[test]
fn generated_cells_agree_with_the_closed_form_oracle_for_every_name() {
    let mut cells = 0;
    let mut check = |blocks: &[Name]| {
        for p_current in [false, true] {
            for end in SELECTED {
                let oracle = closed_form_oracle(blocks, p_current, end);
                let candidate = machine_cell_outcome(blocks, p_current, end);
                assert_eq!(
                    candidate, oracle,
                    "blocks={blocks:?} p={p_current} end={end:?}"
                );
                cells += 1;
            }
        }
    };

    // Exhaustive up to depth three over all eight names.
    check(&[]);
    for a in SELECTED {
        check(&[a]);
        for b in SELECTED {
            check(&[a, b]);
            for c in SELECTED {
                check(&[a, b, c]);
            }
        }
    }
    // Deterministic samples at depths four to six.
    let mut rng = Lcg(0x05EE_D894);
    for depth in 4..=6 {
        for _ in 0..120 {
            let blocks: Vec<Name> = (0..depth).map(|_| SELECTED[rng.next(8)]).collect();
            check(&blocks);
        }
    }
    assert!(cells > 10_000, "cells={cells}");
}

#[test]
fn generated_programs_agree_with_the_independent_reference_model() {
    let mut programs = 0;
    for seed in 0..900_u64 {
        let length = 1 + (seed as usize % 14);
        let ops = generate_program(seed, length);
        let mut model = RefModel::new();
        for op in &ops {
            model.apply(*op);
        }

        for salt in 0..2 {
            let source = program_source(&ops, salt);
            let run = tokenize_source(&source, 300 + seed);
            let compact = observe_with_layout(&run, StorageLayout::COMPACT);
            let padded = observe_with_layout(&run, StorageLayout::PADDED);
            assert_complete(&compact);
            assert_eq!(compact, padded, "{source}");
            // Repeat determinism.
            assert_eq!(compact, observe_with_layout(&run, StorageLayout::COMPACT));

            assert_eq!(
                render_node(&compact, nodes_named(&compact, Name::Body)[0].id),
                model.render(0),
                "{source}"
            );
            assert_eq!(
                open_names(&compact)[1..],
                model
                    .open_labels()
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                "{source}"
            );
            assert_eq!(compact.nodes.len(), 3 + model.nodes.len(), "{source}");
            assert_eq!(compact.next_id, 3 + model.nodes.len(), "{source}");
            let count = |kind: DiagnosticKind| {
                compact
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.kind == kind)
                    .count()
            };
            assert_eq!(
                count(DiagnosticKind::UnmatchedSelectedEnd),
                model.unmatched,
                "{source}"
            );
            assert_eq!(
                count(DiagnosticKind::MisnestedSelectedEnd),
                model.misnested,
                "{source}"
            );
            assert_eq!(
                count(DiagnosticKind::OpenSelectedAtEof),
                usize::from(model.has_open_selected()),
                "{source}"
            );
            assert_eq!(compact.recoveries.len(), model.recoveries, "{source}");
            assert_eq!(compact.implied_p_pops.len(), model.implied, "{source}");
            // Every end closes at most one node; recovered nodes never close.
            for recovery in &compact.recoveries {
                assert!(
                    compact
                        .closures
                        .iter()
                        .all(|closure| closure.target != recovery.popped),
                    "{source}"
                );
            }
        }
        programs += 1;
    }
    assert_eq!(programs, 900);
}

#[test]
fn refusals_are_transactional_with_the_full_state_fingerprint_unchanged() {
    // Negative controls (Issue #894) refuse at the exact boundary.
    let attribute = assert_refusal("<body><article id=x>", Unsupported::SelectedAttribute);
    assert_eq!(attribute.refusal.unwrap().token_index, 1);
    assert_refusal("<body><nav/>", Unsupported::SelectedSelfClosing);
    assert_refusal("<body></body><article>", Unsupported::SelectedOutsideInBody);
    assert_refusal(
        "<body></body></article>",
        Unsupported::SelectedOutsideInBody,
    );
    assert_refusal("<body><span>", Unsupported::GenericTag);
    assert_refusal("<body><search>", Unsupported::GenericTag);
    assert_refusal("<body><ul>", Unsupported::GenericTag);

    // End tags with attributes / self-closing stay outside the theorem.
    assert_refusal("<body><main></main id=x>", Unsupported::SelectedAttribute);
    assert_refusal("<body><header></header/>", Unsupported::SelectedSelfClosing);
    assert_refusal("<body><p id=x>", Unsupported::ParagraphTagShape);

    // Refusal over a deep, non-trivial state leaves that state untouched.
    let deep = assert_refusal(
        "<body><article><nav><p>x<footer id=x>",
        Unsupported::SelectedAttribute,
    );
    assert_eq!(open_names(&deep), ["html", "body", "article", "nav", "p"]);
    assert_eq!(deep.closures.len(), 0);
    assert_eq!(deep.p_closures.len(), 0, "no partial paragraph close");

    // Body end over open selected or paragraph state stays refused.
    assert_refusal(
        "<body><article></body>",
        Unsupported::BodyEndWithOpenSelected,
    );
    assert_refusal("<body><p></body>", Unsupported::BodyEndWithOpenParagraph);
    // Unmatched paragraph end synthesis is a predecessor rule outside this leaf.
    assert_refusal("<body><p>x</p></p>", Unsupported::UnmatchedParagraphEnd);
}

#[test]
fn nearby_whatwg_names_and_arbitrary_names_are_not_admitted() {
    let mut excluded: Vec<&str> = SPEC_START_GROUP
        .iter()
        .chain(SPEC_END_GROUP.iter())
        .copied()
        .filter(|name| *name != "p")
        .filter(|name| Name::from_interpreted(name).is_none())
        .collect();
    excluded.extend([
        "span",
        "h1",
        "h6",
        "script",
        "style",
        "title",
        "textarea",
        "custom-element",
        "a",
    ]);
    excluded.sort_unstable();
    excluded.dedup();
    assert!(excluded.len() >= 25);
    for name in &excluded {
        assert_refusal(&format!("<body><{name}>"), Unsupported::GenericTag);
        assert_refusal(&format!("<body></{name}>"), Unsupported::GenericTag);
        assert_refusal(&format!("<body><article><{name}>"), Unsupported::GenericTag);
    }
    for name in SELECTED {
        assert!(!excluded.contains(&name.as_str()));
    }
}

#[test]
fn candidate_adds_no_phase_no_feedback_and_no_resource_dimension() {
    // One batch tokenization feeds the whole candidate; no re-tokenization or
    // feedback is needed for any selected name, and the phase never leaves
    // InBody for selected tokens.
    for name in SELECTED {
        let source = format!("<body><{0}><{0}>x</{0}></{0}>", name.as_str());
        let run = tokenize_source(&source, 131);
        let mut machine = Machine::new(StorageLayout::COMPACT);
        for (index, token) in run.tokens().iter().enumerate() {
            let stop = machine.process(index, token).expect("selected input");
            machine.commit(token);
            if index >= 1 {
                assert_eq!(machine.phase, Phase::InBody, "{source}");
            }
            if stop {
                break;
            }
        }
        assert_eq!(machine.processed_tokens, run.tokens().len());
    }

    // Structural termination without a runtime budget: deep nesting completes
    // with exactly one pop per element and no depth or pop cap.
    let depth = 300;
    let mut source = String::from("<body>");
    for index in 0..depth {
        source.push_str(&format!("<{}>", SELECTED[index % 8].as_str()));
    }
    for index in (0..depth).rev() {
        source.push_str(&format!("</{}>", SELECTED[index % 8].as_str()));
    }
    let run = tokenize_with(
        &source,
        132,
        HtmlTokenizerLimits::new(65_536, 1_000_000, 4_096, 1_024, 256, 65_536, 1_024),
    );
    let observation = observe_with_layout(&run, StorageLayout::COMPACT);
    assert_complete(&observation);
    assert_eq!(observation.closures.len(), depth);
    assert!(observation.recoveries.is_empty());
    assert_eq!(open_names(&observation), ["html", "body"]);

    // Worst case for recovery: one end pops the whole suffix, each pop strictly
    // shortening the stack.
    let mut source = String::from("<body><main>");
    for index in 0..depth {
        source.push_str(&format!("<{}>", NON_MAIN_ADDITIONS[index % 5].as_str()));
    }
    source.push_str("</main>");
    let run = tokenize_with(
        &source,
        133,
        HtmlTokenizerLimits::new(65_536, 1_000_000, 4_096, 1_024, 256, 65_536, 1_024),
    );
    let observation = observe_with_layout(&run, StorageLayout::COMPACT);
    assert_complete(&observation);
    assert_eq!(observation.recoveries.len(), depth);
    assert_eq!(observation.closures.len(), 1);
    assert_eq!(open_names(&observation), ["html", "body"]);
    let recovery_order: Vec<NodeId> = observation
        .recoveries
        .iter()
        .map(|recovery| recovery.popped)
        .collect();
    let mut sorted = recovery_order.clone();
    sorted.sort_by(|left, right| right.cmp(left));
    assert_eq!(recovery_order, sorted, "recovery pops run current-first");
}

#[test]
fn lower_layer_incompleteness_is_monotonic_and_categories_stay_distinct() {
    // Resource limits: never upgraded to Complete.
    let source = "<body><article>xxxxxxxx</article>";
    let tiny = tokenize_with(source, 141, HtmlTokenizerLimits::new(1, 1, 1, 1, 1, 1, 1));
    assert!(tiny.is_incomplete());
    let observation = observe_with_layout(&tiny, StorageLayout::COMPACT);
    assert_eq!(
        observation.completion,
        Completion::LowerLayerIncomplete(LowerLayerCategory::ResourceLimit)
    );

    // A partial prefix: the candidate may reflect only committed coverage and
    // must stay incomplete even though a selected element was already built.
    let partial = tokenize_with(
        source,
        142,
        HtmlTokenizerLimits::new(1_024, 8_192, 3, 1_024, 256, 4_096, 1_024),
    );
    assert!(partial.is_incomplete());
    let observation = observe_with_layout(&partial, StorageLayout::COMPACT);
    assert_ne!(observation.completion, Completion::Complete);
    assert_eq!(
        observation.completion,
        Completion::LowerLayerIncomplete(LowerLayerCategory::ResourceLimit)
    );
    assert!(observation.closures.is_empty());
    assert!(
        observation
            .diagnostics
            .iter()
            .all(|diagnostic| { diagnostic.kind != DiagnosticKind::OpenSelectedAtEof }),
        "no EOF fact may be fabricated for an incomplete lower layer"
    );

    // Unsupported lower-layer capabilities (Comment / MarkupDeclaration /
    // AttributeValue Character References) stay non-Complete and keep their
    // category distinct from resource exhaustion.
    for source in [
        "<body><article><!-- c --></article>",
        "<body><nav><!x></nav>",
        "<body><article id=\"&amp;\">x</article>",
    ] {
        let run = tokenize_source(source, 143);
        let observation = observe_with_layout(&run, StorageLayout::COMPACT);
        assert_ne!(observation.completion, Completion::Complete, "{source}");
        if run.is_incomplete() {
            assert_ne!(
                lower_layer_category(&run),
                Some(LowerLayerCategory::ResourceLimit),
                "{source}"
            );
        }
    }

    // Parse diagnostics and selected recovery do not imply incompleteness.
    let recovered = observe_source("<body><article><nav></article></nav>", 144);
    assert_complete(&recovered);
    assert!(recovered.diagnostics.len() > 1);
}

#[test]
fn compliant_complete_runs_never_end_without_end_of_file() {
    for gold in GOLD {
        let observation = observe_source(gold.source, 151);
        assert_ne!(observation.completion, Completion::MissingEndOfFile);
    }
}
