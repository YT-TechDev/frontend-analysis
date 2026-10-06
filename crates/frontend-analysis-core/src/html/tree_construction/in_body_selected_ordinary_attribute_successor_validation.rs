//! Candidate-independent validation of selected ordinary authored attribute
//! association (Issue #900).
//!
//! The question validated here is deliberately narrow: for a constructed
//! SelectedOrdinary element in the existing ordinary-document `InBody`
//! profile, can one tokenizer-complete, character-reference-free authored
//! attribute from the exact consumed start tag be associated with that
//! constructed node identity, while keeping exact authored source/syntax
//! evidence apart from the tokenizer-interpreted name and value? This module
//! is validation only. It chooses no production placement and no production
//! representation, adds no public or cross-run compatibility promise, and
//! changes no production semantics. Production does not associate attributes
//! with nodes today and this module does not claim it does.
//!
//! Selected domain (Issue #348 comment `5992832273`, Issue #900):
//!
//! ```text
//! element names: div | section | article | aside | footer | header | main | nav
//! attributes:    zero, or exactly one tokenizer-complete authored attribute
//! ```
//!
//! Lower layer: the accepted batch tokenizer is consumed only as source-backed
//! token evidence (token order and kind, interpreted and authored tag names,
//! complete tag ranges, attribute evidence with its value syntax and
//! disposition, self-closing evidence, tokenizer diagnostics, and completion
//! state). Tokens carry no node identity, so tokenizer evidence is not the
//! expected association: the private candidate [`Machine`] below decides which
//! model node receives which token-owned attribute evidence. No production
//! tree-construction semantics (driver, session, result, the public
//! `html::tree` report) are imported or used as an oracle;
//! `validation_module_does_not_import_production_tree_semantics` enforces that
//! on this file's own text. The machine never receives source text, so it
//! cannot rescan, search, or retokenize: association can only come from the
//! consumed token.
//!
//! Expected meaning comes from two independent sources:
//!
//! 1. the private [`Machine`], a tiny model of exactly the accepted shape
//!    `[html, body] ++ S*` with `S` in the eight selected names, using
//!    model-local creation-order identity (never production numeric ids); and
//! 2. hand-authored GOLD: authored fixtures state the node label, parent,
//!    authored tag evidence, authored attribute name/value anchors, value
//!    syntax, interpreted name/value, and tokenizer diagnostics, as explicit
//!    offsets. The bounded generators compute offsets only from the authored
//!    pieces they concatenate while building each source, never from
//!    tokenizer or production output.
//!
//! Normative authority pins (zero-base selection in #348 comment
//! `5992832273`): WHATWG HTML commit `4c5586afa4fc6f2feeebcd25be1dca017cb51298`,
//! source blob `1286cf2f87000e82ec573794caa8081de5c68917`. WPT and html5lib
//! are challenge/corroboration pins only and define no expected semantics.
//!
//! Two facts are kept apart on purpose:
//!
//! ```text
//! selected semantic theorem:   at most one associated authored attribute
//! != Product resource policy:  a second attribute reaches ResourceLimit
//! ```
//!
//! The one-attribute bound is a selection and a resource fact, not a claim
//! about HTML. `second_attribute_is_a_resource_fact_not_a_semantic_refusal`
//! shows the lower layer completes two attributes when its limit allows, and
//! that the model's own boundary is a different category from the lower-layer
//! `ResourceLimit`.
//!
//! Falsification targets exercised (numbering follows Issue #900 section 45):
//! (1) name-based association is falsified by same-name nesting
//! (`name_keyed_association_is_falsified_by_same_name_nesting`); (2) source
//! offsets alone do not identify ownership
//! (`offsets_alone_do_not_identify_ownership`); (3) the interpreted pair is not
//! sufficient evidence
//! (`interpreted_pair_alone_is_not_sufficient_evidence`); (4) supported value
//! syntaxes are not equivalent (same test and the syntax tests); (5) a
//! ResourceLimit is not a semantic refusal; (6) an `&` value is not partially
//! admitted (historical negative control, retired by Issue #906 when production
//! superseded that boundary);
//! (7) same-name nesting cannot confuse association (identity GOLD); (8) a later
//! incomplete input does not invalidate committed evidence
//! (`committed_attribute_survives_a_later_lower_layer_stop`); (9) mixed-case
//! authored spelling is retained; (10) authored U+0000 and interpreted U+FFFD
//! are not collapsed (`authored_nul_stays_exact_while_the_interpreted_value_is_replacement`).
//! Final-tree placement is falsified by
//! `final_tree_depth_pairing_is_falsified`.
//!
//! Bounded candidate configuration: ordinary document, scripting disabled,
//! Data tokenizer start, no context element, HTML namespace only, and the
//! existing document-shell path ending in `InBody`. The model adds no phase,
//! no tokenizer feedback, no resource dimension, no counter, no queue, and no
//! recursion: it makes one linear pass over a bounded fixture token vector and
//! every pop strictly shortens the open stack.

use std::collections::HashMap;

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::{
    HtmlAttributeDisposition, HtmlAttributeEvidence, HtmlAttributeValueSyntax, HtmlTagKind,
    HtmlToken,
};
use super::super::tokenizer::diagnostic::{
    HtmlTokenizerDiagnosticCode, HtmlTokenizerDiagnosticContext, HtmlTokenizerDiagnosticHandling,
    HtmlTokenizerRecoveryKind,
};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerResource};
use super::super::tokenizer::result::{
    HtmlTokenizerCapability, HtmlTokenizerCapabilityAvailability, HtmlTokenizerCompletion,
    HtmlTokenizerIncompleteCause, HtmlTokenizerMode, HtmlTokenizerRunResult,
    HtmlTokenizerUnsupportedTrigger,
};

const PINNED_WHATWG_COMMIT: &str = "4c5586afa4fc6f2feeebcd25be1dca017cb51298";
const PINNED_WHATWG_SOURCE_BLOB: &str = "1286cf2f87000e82ec573794caa8081de5c68917";
const SELECTION_AUTHORITY_COMMENT: &str = "5992832273";

type Range = (usize, usize);

// ---------------------------------------------------------------------------
// Evidence. Every anchor is compared as (SourceId, range) so provenance never
// collapses to bare offsets, and every authored anchor also keeps the exact
// authored spelling the anchor itself retains.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct Evidence {
    source_id: SourceId,
    range: Range,
}

fn evidence(anchor: &SourceAnchor) -> Evidence {
    Evidence {
        source_id: anchor.source_id(),
        range: (anchor.range().start(), anchor.range().end()),
    }
}

fn at(source_id: SourceId, range: Range) -> Evidence {
    Evidence { source_id, range }
}

/// Authored evidence: the anchor plus the exact authored spelling retained by
/// that anchor. The spelling is read from the retained anchor, never recovered
/// from the interpreted value or by rescanning source.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Authored {
    evidence: Evidence,
    spelling: String,
}

fn authored(anchor: &SourceAnchor) -> Authored {
    Authored {
        evidence: evidence(anchor),
        spelling: anchor.fragment().to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Model vocabulary.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SelectedName {
    Div,
    Section,
    Article,
    Aside,
    Footer,
    Header,
    Main,
    Nav,
}

const SELECTED: [SelectedName; 8] = [
    SelectedName::Div,
    SelectedName::Section,
    SelectedName::Article,
    SelectedName::Aside,
    SelectedName::Footer,
    SelectedName::Header,
    SelectedName::Main,
    SelectedName::Nav,
];

impl SelectedName {
    fn as_str(self) -> &'static str {
        match self {
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
        SELECTED
            .into_iter()
            .find(|candidate| candidate.as_str() == name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelName {
    Html,
    Head,
    Body,
    Selected(SelectedName),
}

/// Model-local creation-order identity. It is assigned by the model at the
/// construction event and is never a production node id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModelNodeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SynthesisCause {
    ImpliedHtml,
    ImpliedHead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Origin {
    Synthesized(SynthesisCause),
    Authored {
        token_index: usize,
        trigger: Evidence,
        raw_name: Authored,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModelSyntax {
    Missing,
    MissingAfterEquals {
        equals: Evidence,
        boundary: Evidence,
    },
    Unquoted {
        equals: Evidence,
        value: Authored,
    },
    DoubleQuoted {
        equals: Evidence,
        open_quote: Evidence,
        value: Authored,
        close_quote: Evidence,
    },
    SingleQuoted {
        equals: Evidence,
        open_quote: Evidence,
        value: Authored,
        close_quote: Evidence,
    },
}

/// The attribute evidence a node owns: authored attribute/name/value evidence
/// and value syntax kept distinct from the interpreted name and value.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelAttribute {
    complete: Evidence,
    authored_name: Authored,
    interpreted_name: String,
    syntax: ModelSyntax,
    interpreted_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelNode {
    id: ModelNodeId,
    parent: Option<ModelNodeId>,
    name: ModelName,
    origin: Origin,
    attribute: Option<ModelAttribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    BeforeBody,
    InBody,
}

/// The model's own bounded-theorem boundary. These are not lower-layer
/// categories and never claim HTML forbids the shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Refusal {
    /// The token is outside the accepted `[html, body] ++ S*` profile.
    OutsideProfile,
    /// A `<body>` start tag carrying attributes (shell attributes).
    ShellStartAttributes,
    /// A start or end tag whose name is not one of the eight selected names.
    OutsideSelectedDomain,
    /// A selected start tag with more than one authored attribute. The
    /// selection covers zero or one; this is not the lower-layer limit.
    MoreThanOneAttribute,
    /// A tag with a self-closing solidus.
    SelfClosingTag,
    /// A selected end tag carrying attribute evidence.
    EndTagAttributes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LowerLayerCategory {
    UnsupportedCapability,
    ResourceLimit,
    InvalidConfiguration,
    InternalInvariantFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LowerTrigger {
    Input(Evidence),
    EmittedToken { token_index: usize },
}

/// The exact lower-layer stop, retained with its payload so the validator can
/// distinguish categories without collapsing them into one "incomplete".
#[derive(Debug, Clone, PartialEq, Eq)]
enum LowerStop {
    Unsupported {
        capability: HtmlTokenizerCapability,
        availability: HtmlTokenizerCapabilityAvailability,
        trigger: LowerTrigger,
    },
    ResourceLimit {
        resource: HtmlTokenizerResource,
        limit: usize,
        attempted: usize,
        at: Evidence,
    },
    InvalidConfiguration,
    InternalInvariantFailure,
}

impl LowerStop {
    fn category(&self) -> LowerLayerCategory {
        match self {
            Self::Unsupported { .. } => LowerLayerCategory::UnsupportedCapability,
            Self::ResourceLimit { .. } => LowerLayerCategory::ResourceLimit,
            Self::InvalidConfiguration => LowerLayerCategory::InvalidConfiguration,
            Self::InternalInvariantFailure => LowerLayerCategory::InternalInvariantFailure,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LowerDiagnostic {
    code: HtmlTokenizerDiagnosticCode,
    location: Range,
    context: HtmlTokenizerDiagnosticContext,
    handling: HtmlTokenizerDiagnosticHandling,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Completion {
    Complete,
    ModelRefused {
        refusal: Refusal,
        token_index: usize,
    },
    LowerLayerIncomplete(LowerLayerCategory),
    MissingEndOfFile,
}

// ---------------------------------------------------------------------------
// The candidate-independent model.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct Machine {
    nodes: Vec<ModelNode>,
    open: Vec<ModelNodeId>,
    phase: Phase,
    closures: Vec<ModelNodeId>,
    recovery_pops: Vec<ModelNodeId>,
    ignored_ends: Vec<SelectedName>,
    committed_end: usize,
    processed_tokens: usize,
}

impl Machine {
    fn new() -> Self {
        let mut machine = Self {
            nodes: Vec::new(),
            open: Vec::new(),
            phase: Phase::BeforeBody,
            closures: Vec::new(),
            recovery_pops: Vec::new(),
            ignored_ends: Vec::new(),
            committed_end: 0,
            processed_tokens: 0,
        };
        let html = machine.allocate(
            None,
            ModelName::Html,
            Origin::Synthesized(SynthesisCause::ImpliedHtml),
            None,
        );
        machine.allocate(
            Some(html),
            ModelName::Head,
            Origin::Synthesized(SynthesisCause::ImpliedHead),
            None,
        );
        machine.open.push(html);
        machine.assert_invariant();
        machine
    }

    fn allocate(
        &mut self,
        parent: Option<ModelNodeId>,
        name: ModelName,
        origin: Origin,
        attribute: Option<ModelAttribute>,
    ) -> ModelNodeId {
        let id = ModelNodeId(self.nodes.len());
        self.nodes.push(ModelNode {
            id,
            parent,
            name,
            origin,
            attribute,
        });
        id
    }

    fn node(&self, id: ModelNodeId) -> &ModelNode {
        &self.nodes[id.0]
    }

    fn current(&self) -> ModelNodeId {
        *self.open.last().expect("open element")
    }

    fn assert_invariant(&self) {
        let names: Vec<ModelName> = self.open.iter().map(|id| self.node(*id).name).collect();
        match self.phase {
            Phase::BeforeBody => assert_eq!(names, [ModelName::Html]),
            Phase::InBody => {
                assert!(
                    names.len() >= 2 && names[0] == ModelName::Html && names[1] == ModelName::Body,
                    "bounded stack invariant violated: {names:?}"
                );
                assert!(
                    names[2..]
                        .iter()
                        .all(|name| matches!(name, ModelName::Selected(_))),
                    "bounded stack invariant violated: {names:?}"
                );
            }
        }

        // Association invariants: only an authored selected node can own
        // attribute evidence, and that evidence is nested in the exact start
        // tag that constructed the node.
        for node in &self.nodes {
            let Some(attribute) = &node.attribute else {
                continue;
            };
            let Origin::Authored { trigger, .. } = &node.origin else {
                panic!("a synthesized node must never own authored attribute evidence");
            };
            assert!(matches!(node.name, ModelName::Selected(_)));
            assert_eq!(attribute.complete.source_id, trigger.source_id);
            assert!(
                trigger.range.0 <= attribute.complete.range.0
                    && attribute.complete.range.1 <= trigger.range.1,
                "attribute evidence must be nested in its constructing start tag"
            );
        }
        let anchors: Vec<&Evidence> = self
            .nodes
            .iter()
            .filter_map(|node| node.attribute.as_ref().map(|attribute| &attribute.complete))
            .collect();
        for (index, anchor) in anchors.iter().enumerate() {
            assert!(
                !anchors[index + 1..].contains(anchor),
                "one attribute evidence may be owned by only one node"
            );
        }
    }

    fn commit(&mut self, token: &HtmlToken) {
        self.committed_end = token_end(token);
        self.processed_tokens += 1;
    }

    /// Consume one token. Every refusal is decided before any mutation.
    /// Returns whether parsing stops.
    fn process(&mut self, token_index: usize, token: &HtmlToken) -> Result<bool, Refusal> {
        self.assert_invariant();
        let stop = match self.phase {
            Phase::BeforeBody => self.process_before_body(token_index, token)?,
            Phase::InBody => self.process_in_body(token_index, token)?,
        };
        self.assert_invariant();
        Ok(stop)
    }

    fn process_before_body(
        &mut self,
        token_index: usize,
        token: &HtmlToken,
    ) -> Result<bool, Refusal> {
        let HtmlToken::Tag(tag) = token else {
            return Err(Refusal::OutsideProfile);
        };
        if tag.kind() != HtmlTagKind::Start || tag.name().interpreted() != "body" {
            return Err(Refusal::OutsideProfile);
        }
        if !tag.attributes().is_empty() {
            return Err(Refusal::ShellStartAttributes);
        }
        if tag.self_closing_solidus().is_some() {
            return Err(Refusal::SelfClosingTag);
        }
        let html = self.current();
        let body = self.allocate(
            Some(html),
            ModelName::Body,
            Origin::Authored {
                token_index,
                trigger: evidence(tag.complete()),
                raw_name: authored(tag.name().source()),
            },
            None,
        );
        self.open.push(body);
        self.phase = Phase::InBody;
        Ok(false)
    }

    fn process_in_body(&mut self, token_index: usize, token: &HtmlToken) -> Result<bool, Refusal> {
        match token {
            HtmlToken::Character(_) | HtmlToken::Doctype(_) => Err(Refusal::OutsideProfile),
            HtmlToken::EndOfFile(_) => Ok(true),
            HtmlToken::Tag(tag) => {
                let name = SelectedName::from_interpreted(tag.name().interpreted())
                    .ok_or(Refusal::OutsideSelectedDomain)?;
                if tag.self_closing_solidus().is_some() {
                    return Err(Refusal::SelfClosingTag);
                }
                match tag.kind() {
                    HtmlTagKind::Start => {
                        let attribute = match tag.attributes() {
                            [] => None,
                            [only] => Some(model_attribute(only)),
                            _ => return Err(Refusal::MoreThanOneAttribute),
                        };
                        // The construction event: the node and the attribute
                        // evidence carried by this exact consumed start tag are
                        // bound together here, once, and never again.
                        let parent = self.current();
                        let id = self.allocate(
                            Some(parent),
                            ModelName::Selected(name),
                            Origin::Authored {
                                token_index,
                                trigger: evidence(tag.complete()),
                                raw_name: authored(tag.name().source()),
                            },
                            attribute,
                        );
                        self.open.push(id);
                    }
                    HtmlTagKind::End => {
                        if !tag.attributes().is_empty() {
                            return Err(Refusal::EndTagAttributes);
                        }
                        self.apply_selected_end(name);
                    }
                }
                Ok(false)
            }
        }
    }

    /// Nearest same-name open node by open-state identity; every open node
    /// above it is popped as recovery. An end tag with no open same-name node
    /// is ignored. It never creates or alters attribute evidence.
    fn apply_selected_end(&mut self, name: SelectedName) {
        let Some(target_position) = self
            .open
            .iter()
            .rposition(|id| self.node(*id).name == ModelName::Selected(name))
        else {
            self.ignored_ends.push(name);
            return;
        };
        while self.open.len() > target_position + 1 {
            let before = self.open.len();
            let popped = self.open.pop().expect("open element above target");
            assert!(self.open.len() < before, "every pop strictly shortens");
            self.recovery_pops.push(popped);
        }
        let target = self.open.pop().expect("target");
        self.closures.push(target);
    }
}

fn model_attribute(attribute: &HtmlAttributeEvidence) -> ModelAttribute {
    assert_eq!(
        attribute.disposition(),
        HtmlAttributeDisposition::Effective,
        "a sole attribute is always effective"
    );
    ModelAttribute {
        complete: evidence(attribute.complete()),
        authored_name: authored(attribute.name().source()),
        interpreted_name: attribute.name().interpreted().to_owned(),
        syntax: model_syntax(attribute.value_syntax()),
        interpreted_value: attribute.interpreted_value().to_owned(),
    }
}

fn model_syntax(syntax: &HtmlAttributeValueSyntax) -> ModelSyntax {
    match syntax {
        HtmlAttributeValueSyntax::Missing => ModelSyntax::Missing,
        HtmlAttributeValueSyntax::MissingAfterEquals {
            equals,
            value_boundary,
        } => ModelSyntax::MissingAfterEquals {
            equals: evidence(equals),
            boundary: evidence(value_boundary),
        },
        HtmlAttributeValueSyntax::Unquoted { equals, value } => ModelSyntax::Unquoted {
            equals: evidence(equals),
            value: authored(value),
        },
        HtmlAttributeValueSyntax::DoubleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => ModelSyntax::DoubleQuoted {
            equals: evidence(equals),
            open_quote: evidence(open_quote),
            value: authored(value),
            close_quote: evidence(close_quote),
        },
        HtmlAttributeValueSyntax::SingleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => ModelSyntax::SingleQuoted {
            equals: evidence(equals),
            open_quote: evidence(open_quote),
            value: authored(value),
            close_quote: evidence(close_quote),
        },
    }
}

fn token_end(token: &HtmlToken) -> usize {
    match token {
        HtmlToken::Character(character) => character.source().range().end(),
        HtmlToken::Tag(tag) => tag.complete().range().end(),
        HtmlToken::Doctype(doctype) => doctype.complete().range().end(),
        HtmlToken::EndOfFile(eof) => eof.source().range().end(),
    }
}

// ---------------------------------------------------------------------------
// Lower-layer bridge and observation.
// ---------------------------------------------------------------------------

/// The frozen Product tokenizer vector, hand-transcribed (source 36_864,
/// steps 79_872, tokens 6_144, diagnostics 256, attributes per tag 1,
/// retained bytes 49_152, temporary buffer 0). It is a lower-layer resource
/// fact, restated here so the validator imports no production tree module.
fn product_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(36_864, 79_872, 6_144, 256, 1, 49_152, 0)
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

fn lower_stop(run: &HtmlTokenizerRunResult) -> Option<LowerStop> {
    match run.completion() {
        HtmlTokenizerCompletion::Complete => None,
        HtmlTokenizerCompletion::Incomplete(cause) => Some(match cause {
            HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported) => {
                LowerStop::Unsupported {
                    capability: unsupported.capability(),
                    availability: unsupported.availability(),
                    trigger: match unsupported.trigger() {
                        HtmlTokenizerUnsupportedTrigger::Input(anchor) => {
                            LowerTrigger::Input(evidence(anchor))
                        }
                        HtmlTokenizerUnsupportedTrigger::EmittedToken { token_index, .. } => {
                            LowerTrigger::EmittedToken {
                                token_index: *token_index,
                            }
                        }
                    },
                }
            }
            HtmlTokenizerIncompleteCause::ResourceLimit(limit) => LowerStop::ResourceLimit {
                resource: limit.resource(),
                limit: limit.limit(),
                attempted: limit.attempted(),
                at: evidence(limit.at()),
            },
            HtmlTokenizerIncompleteCause::InvalidConfiguration(_) => {
                LowerStop::InvalidConfiguration
            }
            HtmlTokenizerIncompleteCause::InternalInvariantFailure(_) => {
                LowerStop::InternalInvariantFailure
            }
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    source_id: SourceId,
    nodes: Vec<ModelNode>,
    open: Vec<ModelNodeId>,
    closures: Vec<ModelNodeId>,
    recovery_pops: Vec<ModelNodeId>,
    ignored_ends: Vec<SelectedName>,
    completion: Completion,
    lower_stop: Option<LowerStop>,
    lower_diagnostics: Vec<LowerDiagnostic>,
    lower_processed_end: usize,
    committed_end: usize,
    token_count: usize,
}

impl Observation {
    fn selected_nodes(&self) -> Vec<&ModelNode> {
        self.nodes
            .iter()
            .filter(|node| matches!(node.name, ModelName::Selected(_)))
            .collect()
    }

    fn node_named(&self, name: ModelName) -> &ModelNode {
        self.nodes
            .iter()
            .find(|node| node.name == name)
            .expect("model node")
    }

    fn depth(&self, node: &ModelNode) -> usize {
        let mut depth = 0;
        let mut parent = node.parent;
        while let Some(id) = parent {
            let ancestor = &self.nodes[id.0];
            if matches!(ancestor.name, ModelName::Selected(_)) {
                depth += 1;
            }
            parent = ancestor.parent;
        }
        depth
    }

    fn interpreted_pairs(&self) -> Vec<Option<(String, String)>> {
        self.selected_nodes()
            .iter()
            .map(|node| {
                node.attribute.as_ref().map(|attribute| {
                    (
                        attribute.interpreted_name.clone(),
                        attribute.interpreted_value.clone(),
                    )
                })
            })
            .collect()
    }
}

fn observe_run(source_id: SourceId, run: &HtmlTokenizerRunResult) -> Observation {
    let mut machine = Machine::new();
    let mut refusal = None;
    let mut stopped = false;

    for (token_index, token) in run.tokens().iter().enumerate() {
        let before = machine.clone();
        match machine.process(token_index, token) {
            Ok(stop) => {
                machine.commit(token);
                assert!(
                    machine.committed_end <= run.coverage().processed_end(),
                    "the model must not claim evidence beyond lower-layer coverage"
                );
                if stop {
                    stopped = true;
                    break;
                }
            }
            Err(reason) => {
                assert_eq!(before, machine, "a refusal must be transactional");
                refusal = Some((reason, token_index));
                break;
            }
        }
    }

    let stop = lower_stop(run);
    let completion = if let Some((refusal, token_index)) = refusal {
        Completion::ModelRefused {
            refusal,
            token_index,
        }
    } else if let Some(stop) = &stop {
        Completion::LowerLayerIncomplete(stop.category())
    } else if stopped && machine.processed_tokens == run.tokens().len() {
        Completion::Complete
    } else {
        Completion::MissingEndOfFile
    };

    Observation {
        source_id,
        nodes: machine.nodes,
        open: machine.open,
        closures: machine.closures,
        recovery_pops: machine.recovery_pops,
        ignored_ends: machine.ignored_ends,
        completion,
        lower_stop: stop,
        lower_diagnostics: run
            .diagnostics()
            .iter()
            .map(|diagnostic| LowerDiagnostic {
                code: diagnostic.code(),
                location: (
                    diagnostic.location().range().start(),
                    diagnostic.location().range().end(),
                ),
                context: diagnostic.context(),
                handling: diagnostic.handling(),
            })
            .collect(),
        lower_processed_end: run.coverage().processed_end(),
        committed_end: machine.committed_end,
        token_count: run.tokens().len(),
    }
}

fn observe(source: &str, source_id: u64, limits: HtmlTokenizerLimits) -> Observation {
    observe_run(
        SourceId::new(source_id),
        &tokenize_with(source, source_id, limits),
    )
}

// ---------------------------------------------------------------------------
// Hand-authored GOLD vocabulary. GOLD states labels, parents, and explicit
// offsets; it defines no semantics and is never derived from candidate output.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct GoldName {
    range: Range,
    authored: String,
    interpreted: String,
}

fn nm(range: Range, authored: &str, interpreted: &str) -> GoldName {
    GoldName {
        range,
        authored: authored.to_owned(),
        interpreted: interpreted.to_owned(),
    }
}

#[derive(Debug, Clone)]
struct GoldText {
    range: Range,
    authored: String,
}

fn tx(range: Range, authored: &str) -> GoldText {
    GoldText {
        range,
        authored: authored.to_owned(),
    }
}

#[derive(Debug, Clone)]
enum GoldSyntax {
    Missing,
    MissingAfterEquals {
        equals: Range,
        boundary: Range,
    },
    Unquoted {
        equals: Range,
        value: GoldText,
    },
    DoubleQuoted {
        equals: Range,
        open: Range,
        value: GoldText,
        close: Range,
    },
    SingleQuoted {
        equals: Range,
        open: Range,
        value: GoldText,
        close: Range,
    },
}

#[derive(Debug, Clone)]
struct GoldAttribute {
    complete: Range,
    name: GoldName,
    syntax: GoldSyntax,
    interpreted_value: String,
}

fn attr_missing(complete: Range, name: GoldName) -> GoldAttribute {
    GoldAttribute {
        complete,
        name,
        syntax: GoldSyntax::Missing,
        interpreted_value: String::new(),
    }
}

fn attr_missing_after_equals(
    complete: Range,
    name: GoldName,
    equals: Range,
    boundary: Range,
) -> GoldAttribute {
    GoldAttribute {
        complete,
        name,
        syntax: GoldSyntax::MissingAfterEquals { equals, boundary },
        interpreted_value: String::new(),
    }
}

fn attr_unquoted(
    complete: Range,
    name: GoldName,
    equals: Range,
    value: GoldText,
    interpreted_value: &str,
) -> GoldAttribute {
    GoldAttribute {
        complete,
        name,
        syntax: GoldSyntax::Unquoted { equals, value },
        interpreted_value: interpreted_value.to_owned(),
    }
}

fn attr_double(
    complete: Range,
    name: GoldName,
    equals: Range,
    (open, close): (Range, Range),
    value: GoldText,
    interpreted_value: &str,
) -> GoldAttribute {
    GoldAttribute {
        complete,
        name,
        syntax: GoldSyntax::DoubleQuoted {
            equals,
            open,
            value,
            close,
        },
        interpreted_value: interpreted_value.to_owned(),
    }
}

fn attr_single(
    complete: Range,
    name: GoldName,
    equals: Range,
    (open, close): (Range, Range),
    value: GoldText,
    interpreted_value: &str,
) -> GoldAttribute {
    GoldAttribute {
        complete,
        name,
        syntax: GoldSyntax::SingleQuoted {
            equals,
            open,
            value,
            close,
        },
        interpreted_value: interpreted_value.to_owned(),
    }
}

#[derive(Debug, Clone)]
struct GoldNode {
    label: &'static str,
    name: SelectedName,
    /// Label of an earlier node, or `"body"`.
    parent: &'static str,
    trigger: Range,
    raw_name: (Range, String),
    attribute: Option<GoldAttribute>,
}

fn node(
    label: &'static str,
    name: SelectedName,
    parent: &'static str,
    trigger: Range,
    raw_name: (Range, &str),
    attribute: Option<GoldAttribute>,
) -> GoldNode {
    GoldNode {
        label,
        name,
        parent,
        trigger,
        raw_name: (raw_name.0, raw_name.1.to_owned()),
        attribute,
    }
}

#[derive(Debug, Clone, Default)]
struct Gold {
    nodes: Vec<GoldNode>,
    open: Vec<&'static str>,
    closed: Vec<&'static str>,
    recovered: Vec<&'static str>,
    ignored: Vec<SelectedName>,
    diagnostics: Vec<LowerDiagnostic>,
}

fn gold(nodes: Vec<GoldNode>) -> Gold {
    Gold {
        nodes,
        ..Gold::default()
    }
}

fn expected_syntax(source_id: SourceId, syntax: &GoldSyntax) -> ModelSyntax {
    let text = |value: &GoldText| Authored {
        evidence: at(source_id, value.range),
        spelling: value.authored.clone(),
    };
    match syntax {
        GoldSyntax::Missing => ModelSyntax::Missing,
        GoldSyntax::MissingAfterEquals { equals, boundary } => ModelSyntax::MissingAfterEquals {
            equals: at(source_id, *equals),
            boundary: at(source_id, *boundary),
        },
        GoldSyntax::Unquoted { equals, value } => ModelSyntax::Unquoted {
            equals: at(source_id, *equals),
            value: text(value),
        },
        GoldSyntax::DoubleQuoted {
            equals,
            open,
            value,
            close,
        } => ModelSyntax::DoubleQuoted {
            equals: at(source_id, *equals),
            open_quote: at(source_id, *open),
            value: text(value),
            close_quote: at(source_id, *close),
        },
        GoldSyntax::SingleQuoted {
            equals,
            open,
            value,
            close,
        } => ModelSyntax::SingleQuoted {
            equals: at(source_id, *equals),
            open_quote: at(source_id, *open),
            value: text(value),
            close_quote: at(source_id, *close),
        },
    }
}

fn expected_attribute(source_id: SourceId, gold: &GoldAttribute) -> ModelAttribute {
    ModelAttribute {
        complete: at(source_id, gold.complete),
        authored_name: Authored {
            evidence: at(source_id, gold.name.range),
            spelling: gold.name.authored.clone(),
        },
        interpreted_name: gold.name.interpreted.clone(),
        syntax: expected_syntax(source_id, &gold.syntax),
        interpreted_value: gold.interpreted_value.clone(),
    }
}

/// The shell predecessor every fixture shares: `<body>` is authored at 0..6
/// with the authored name at 1..5, and html/head are synthesized with no
/// authored evidence at all.
fn assert_shell_predecessor(context: &str, observation: &Observation) {
    let source_id = observation.source_id;
    let html = observation.node_named(ModelName::Html);
    let head = observation.node_named(ModelName::Head);
    assert_eq!(
        html.origin,
        Origin::Synthesized(SynthesisCause::ImpliedHtml),
        "{context}"
    );
    assert_eq!(
        head.origin,
        Origin::Synthesized(SynthesisCause::ImpliedHead),
        "{context}"
    );
    assert!(
        html.attribute.is_none() && head.attribute.is_none(),
        "{context}"
    );
    let body = observation.node_named(ModelName::Body);
    assert_eq!(
        body.origin,
        Origin::Authored {
            token_index: 0,
            trigger: at(source_id, (0, 6)),
            raw_name: Authored {
                evidence: at(source_id, (1, 5)),
                spelling: "body".to_owned(),
            },
        },
        "{context}"
    );
    assert!(body.attribute.is_none(), "{context}");
}

fn assert_matches_gold(context: &str, observation: &Observation, gold: &Gold) {
    assert_eq!(observation.completion, Completion::Complete, "{context}");
    assert!(observation.lower_stop.is_none(), "{context}");
    assert_shell_predecessor(context, observation);

    let source_id = observation.source_id;
    let selected = observation.selected_nodes();
    assert_eq!(selected.len(), gold.nodes.len(), "{context}");

    // Labels are bound to model identities by creation order only; every
    // other assertion is on explicit GOLD data, so a swapped association still
    // fails against that node's own GOLD attribute.
    let body = observation.node_named(ModelName::Body).id;
    let mut labelled: Vec<(&'static str, ModelNodeId)> = Vec::new();
    let resolve = |labelled: &[(&'static str, ModelNodeId)], label: &str| -> ModelNodeId {
        if label == "body" {
            return body;
        }
        labelled
            .iter()
            .find(|(candidate, _)| *candidate == label)
            .map(|(_, id)| *id)
            .unwrap_or_else(|| panic!("{context}: unknown GOLD label {label}"))
    };

    for (expected, actual) in gold.nodes.iter().zip(&selected) {
        assert_eq!(
            actual.name,
            ModelName::Selected(expected.name),
            "{context}: {}",
            expected.label
        );
        assert_eq!(
            actual.parent,
            Some(resolve(&labelled, expected.parent)),
            "{context}: parent of {}",
            expected.label
        );
        assert_eq!(
            actual.origin,
            Origin::Authored {
                token_index: actual_token_index(actual),
                trigger: at(source_id, expected.trigger),
                raw_name: Authored {
                    evidence: at(source_id, expected.raw_name.0),
                    spelling: expected.raw_name.1.clone(),
                },
            },
            "{context}: origin of {}",
            expected.label
        );
        assert_eq!(
            actual.attribute,
            expected
                .attribute
                .as_ref()
                .map(|attribute| expected_attribute(source_id, attribute)),
            "{context}: attribute of {}",
            expected.label
        );
        labelled.push((expected.label, actual.id));
    }

    let ids = |labels: &[&'static str]| -> Vec<ModelNodeId> {
        labels
            .iter()
            .map(|label| resolve(&labelled, label))
            .collect()
    };
    let mut expected_open = vec![
        observation.node_named(ModelName::Html).id,
        observation.node_named(ModelName::Body).id,
    ];
    expected_open.extend(ids(&gold.open));
    assert_eq!(observation.open, expected_open, "{context}: open stack");
    assert_eq!(observation.closures, ids(&gold.closed), "{context}: closed");
    assert_eq!(
        observation.recovery_pops,
        ids(&gold.recovered),
        "{context}: recovery pops"
    );
    assert_eq!(observation.ignored_ends, gold.ignored, "{context}: ignored");
    assert_eq!(
        observation.lower_diagnostics, gold.diagnostics,
        "{context}: tokenizer diagnostics"
    );
}

/// The construction token index is model bookkeeping that GOLD does not
/// restate; the trigger range is what GOLD pins. Reading it back keeps the
/// origin comparison exact without importing production indices.
fn actual_token_index(node: &ModelNode) -> usize {
    match &node.origin {
        Origin::Authored { token_index, .. } => *token_index,
        Origin::Synthesized(_) => panic!("selected nodes are authored"),
    }
}

/// Positive-case harness: the association must hold under the frozen Product
/// vector, must be identical under a different (generous) resource
/// configuration when the lower layer completes in both, and must repeat
/// deterministically.
fn check_positive(source: &str, gold: &Gold) -> Observation {
    let product = observe(source, 1, product_limits());
    assert_matches_gold(source, &product, gold);
    let generous = observe(source, 1, generous_limits());
    assert_eq!(
        product, generous,
        "{source:?}: association must not depend on resource configuration"
    );
    for _ in 0..3 {
        assert_eq!(
            observe(source, 1, product_limits()),
            product,
            "{source:?}: deterministic"
        );
    }
    product
}

// ---------------------------------------------------------------------------
// Hand-authored fixtures. Offsets are counted by hand from the literal text.
// ---------------------------------------------------------------------------

/// `<body><article id="a"></article>`
fn fixture_double_quoted() -> (&'static str, Gold) {
    (
        "<body><article id=\"a\"></article>",
        Gold {
            closed: vec!["article"],
            ..gold(vec![node(
                "article",
                SelectedName::Article,
                "body",
                (6, 22),
                ((7, 14), "article"),
                Some(attr_double(
                    (15, 21),
                    nm((15, 17), "id", "id"),
                    (17, 18),
                    ((18, 19), (20, 21)),
                    tx((19, 20), "a"),
                    "a",
                )),
            )])
        },
    )
}

/// `<body><nav class='b'></nav>`
fn fixture_single_quoted() -> (&'static str, Gold) {
    (
        "<body><nav class='b'></nav>",
        Gold {
            closed: vec!["nav"],
            ..gold(vec![node(
                "nav",
                SelectedName::Nav,
                "body",
                (6, 21),
                ((7, 10), "nav"),
                Some(attr_single(
                    (11, 20),
                    nm((11, 16), "class", "class"),
                    (16, 17),
                    ((17, 18), (19, 20)),
                    tx((18, 19), "b"),
                    "b",
                )),
            )])
        },
    )
}

/// `<body><div id=a></div>`
fn fixture_unquoted() -> (&'static str, Gold) {
    (
        "<body><div id=a></div>",
        Gold {
            closed: vec!["div"],
            ..gold(vec![node(
                "div",
                SelectedName::Div,
                "body",
                (6, 16),
                ((7, 10), "div"),
                Some(attr_unquoted(
                    (11, 15),
                    nm((11, 13), "id", "id"),
                    (13, 14),
                    tx((14, 15), "a"),
                    "a",
                )),
            )])
        },
    )
}

/// `<body><div id=a/></div>`: the trailing `/` is part of an unquoted value,
/// not a self-closing solidus. Falsifies "a slash before `>` always
/// self-closes".
fn fixture_unquoted_with_slash() -> (&'static str, Gold) {
    (
        "<body><div id=a/></div>",
        Gold {
            closed: vec!["div"],
            ..gold(vec![node(
                "div",
                SelectedName::Div,
                "body",
                (6, 17),
                ((7, 10), "div"),
                Some(attr_unquoted(
                    (11, 16),
                    nm((11, 13), "id", "id"),
                    (13, 14),
                    tx((14, 16), "a/"),
                    "a/",
                )),
            )])
        },
    )
}

/// `<body><div id=""></div>` (empty double-quoted value).
fn fixture_empty_double() -> (&'static str, Gold) {
    (
        "<body><div id=\"\"></div>",
        Gold {
            closed: vec!["div"],
            ..gold(vec![node(
                "div",
                SelectedName::Div,
                "body",
                (6, 17),
                ((7, 10), "div"),
                Some(attr_double(
                    (11, 16),
                    nm((11, 13), "id", "id"),
                    (13, 14),
                    ((14, 15), (15, 16)),
                    tx((15, 15), ""),
                    "",
                )),
            )])
        },
    )
}

/// `<body><div id=''></div>` (empty single-quoted value).
fn fixture_empty_single() -> (&'static str, Gold) {
    (
        "<body><div id=''></div>",
        Gold {
            closed: vec!["div"],
            ..gold(vec![node(
                "div",
                SelectedName::Div,
                "body",
                (6, 17),
                ((7, 10), "div"),
                Some(attr_single(
                    (11, 16),
                    nm((11, 13), "id", "id"),
                    (13, 14),
                    ((14, 15), (15, 16)),
                    tx((15, 15), ""),
                    "",
                )),
            )])
        },
    )
}

/// `<body><div hidden></div>` (no `=`: Missing).
fn fixture_missing_value() -> (&'static str, Gold) {
    (
        "<body><div hidden></div>",
        Gold {
            closed: vec!["div"],
            ..gold(vec![node(
                "div",
                SelectedName::Div,
                "body",
                (6, 18),
                ((7, 10), "div"),
                Some(attr_missing((11, 17), nm((11, 17), "hidden", "hidden"))),
            )])
        },
    )
}

/// `<body><div id=></div>` (`=` then `>`: MissingAfterEquals), with the
/// tokenizer's own MissingAttributeValue parse diagnostic retained.
fn fixture_missing_after_equals() -> (&'static str, Gold) {
    (
        "<body><div id=></div>",
        Gold {
            closed: vec!["div"],
            diagnostics: vec![LowerDiagnostic {
                code: HtmlTokenizerDiagnosticCode::MissingAttributeValue,
                location: (14, 15),
                context: HtmlTokenizerDiagnosticContext::BeforeAttributeValue,
                handling: HtmlTokenizerDiagnosticHandling::Recovered(
                    HtmlTokenizerRecoveryKind::CompletedTagWithMissingAttributeValue,
                ),
            }],
            ..gold(vec![node(
                "div",
                SelectedName::Div,
                "body",
                (6, 15),
                ((7, 10), "div"),
                Some(attr_missing_after_equals(
                    (11, 14),
                    nm((11, 13), "id", "id"),
                    (13, 14),
                    (14, 14),
                )),
            )])
        },
    )
}

/// `<body><ArTiClE ClAsS='x'></aRtIcLe>`: authored spelling is retained
/// against the normalized interpreted names.
fn fixture_mixed_case() -> (&'static str, Gold) {
    (
        "<body><ArTiClE ClAsS='x'></aRtIcLe>",
        Gold {
            closed: vec!["article"],
            ..gold(vec![node(
                "article",
                SelectedName::Article,
                "body",
                (6, 25),
                ((7, 14), "ArTiClE"),
                Some(attr_single(
                    (15, 24),
                    nm((15, 20), "ClAsS", "class"),
                    (20, 21),
                    ((21, 22), (23, 24)),
                    tx((22, 23), "x"),
                    "x",
                )),
            )])
        },
    )
}

/// `<body><article id="outer"><article id="inner"></article></article>`
fn fixture_same_name_nesting() -> (&'static str, Gold) {
    (
        "<body><article id=\"outer\"><article id=\"inner\"></article></article>",
        Gold {
            closed: vec!["inner", "outer"],
            ..gold(vec![
                node(
                    "outer",
                    SelectedName::Article,
                    "body",
                    (6, 26),
                    ((7, 14), "article"),
                    Some(attr_double(
                        (15, 25),
                        nm((15, 17), "id", "id"),
                        (17, 18),
                        ((18, 19), (24, 25)),
                        tx((19, 24), "outer"),
                        "outer",
                    )),
                ),
                node(
                    "inner",
                    SelectedName::Article,
                    "outer",
                    (26, 46),
                    ((27, 34), "article"),
                    Some(attr_double(
                        (35, 45),
                        nm((35, 37), "id", "id"),
                        (37, 38),
                        ((38, 39), (44, 45)),
                        tx((39, 44), "inner"),
                        "inner",
                    )),
                ),
            ])
        },
    )
}

/// `<body><article id="a"><nav class="b"></nav></article>`
fn fixture_heterogeneous_nesting() -> (&'static str, Gold) {
    (
        "<body><article id=\"a\"><nav class=\"b\"></nav></article>",
        Gold {
            closed: vec!["nav", "article"],
            ..gold(vec![
                node(
                    "article",
                    SelectedName::Article,
                    "body",
                    (6, 22),
                    ((7, 14), "article"),
                    Some(attr_double(
                        (15, 21),
                        nm((15, 17), "id", "id"),
                        (17, 18),
                        ((18, 19), (20, 21)),
                        tx((19, 20), "a"),
                        "a",
                    )),
                ),
                node(
                    "nav",
                    SelectedName::Nav,
                    "article",
                    (22, 37),
                    ((23, 26), "nav"),
                    Some(attr_double(
                        (27, 36),
                        nm((27, 32), "class", "class"),
                        (32, 33),
                        ((33, 34), (35, 36)),
                        tx((34, 35), "b"),
                        "b",
                    )),
                ),
            ])
        },
    )
}

/// `<body><div id="x"></div><div id="x"></div>`: identical interpreted
/// evidence in two construction events yields two identities, distinguished
/// by their authored evidence.
fn fixture_identical_evidence_twice() -> (&'static str, Gold) {
    (
        "<body><div id=\"x\"></div><div id=\"x\"></div>",
        Gold {
            closed: vec!["first", "second"],
            ..gold(vec![
                node(
                    "first",
                    SelectedName::Div,
                    "body",
                    (6, 18),
                    ((7, 10), "div"),
                    Some(attr_double(
                        (11, 17),
                        nm((11, 13), "id", "id"),
                        (13, 14),
                        ((14, 15), (16, 17)),
                        tx((15, 16), "x"),
                        "x",
                    )),
                ),
                node(
                    "second",
                    SelectedName::Div,
                    "body",
                    (24, 36),
                    ((25, 28), "div"),
                    Some(attr_double(
                        (29, 35),
                        nm((29, 31), "id", "id"),
                        (31, 32),
                        ((32, 33), (34, 35)),
                        tx((33, 34), "x"),
                        "x",
                    )),
                ),
            ])
        },
    )
}

/// `<body><div id=o><nav class=i></nav></div><section id=s></section>`: a
/// nested node plus a later sibling, so creation order, depth order, and
/// final-tree placement can be told apart.
fn fixture_nested_then_sibling() -> (&'static str, Gold) {
    (
        "<body><div id=o><nav class=i></nav></div><section id=s></section>",
        Gold {
            closed: vec!["nav", "div", "section"],
            ..gold(vec![
                node(
                    "div",
                    SelectedName::Div,
                    "body",
                    (6, 16),
                    ((7, 10), "div"),
                    Some(attr_unquoted(
                        (11, 15),
                        nm((11, 13), "id", "id"),
                        (13, 14),
                        tx((14, 15), "o"),
                        "o",
                    )),
                ),
                node(
                    "nav",
                    SelectedName::Nav,
                    "div",
                    (16, 29),
                    ((17, 20), "nav"),
                    Some(attr_unquoted(
                        (21, 28),
                        nm((21, 26), "class", "class"),
                        (26, 27),
                        tx((27, 28), "i"),
                        "i",
                    )),
                ),
                node(
                    "section",
                    SelectedName::Section,
                    "body",
                    (41, 55),
                    ((42, 49), "section"),
                    Some(attr_unquoted(
                        (50, 54),
                        nm((50, 52), "id", "id"),
                        (52, 53),
                        tx((53, 54), "s"),
                        "s",
                    )),
                ),
            ])
        },
    )
}

/// `<body><div id=a><section class=b></div>`: `</div>` pops `section` by
/// recovery and closes `div`. Both nodes keep their own attribute evidence.
fn fixture_recovery() -> (&'static str, Gold) {
    (
        "<body><div id=a><section class=b></div>",
        Gold {
            recovered: vec!["section"],
            closed: vec!["div"],
            ..gold(vec![
                node(
                    "div",
                    SelectedName::Div,
                    "body",
                    (6, 16),
                    ((7, 10), "div"),
                    Some(attr_unquoted(
                        (11, 15),
                        nm((11, 13), "id", "id"),
                        (13, 14),
                        tx((14, 15), "a"),
                        "a",
                    )),
                ),
                node(
                    "section",
                    SelectedName::Section,
                    "div",
                    (16, 33),
                    ((17, 24), "section"),
                    Some(attr_unquoted(
                        (25, 32),
                        nm((25, 30), "class", "class"),
                        (30, 31),
                        tx((31, 32), "b"),
                        "b",
                    )),
                ),
            ])
        },
    )
}

/// `<body><nav id=a></nav></nav>`: the second `</nav>` matches nothing, is
/// ignored, and neither creates a node nor touches evidence.
fn fixture_unmatched_end() -> (&'static str, Gold) {
    (
        "<body><nav id=a></nav></nav>",
        Gold {
            closed: vec!["nav"],
            ignored: vec![SelectedName::Nav],
            ..gold(vec![node(
                "nav",
                SelectedName::Nav,
                "body",
                (6, 16),
                ((7, 10), "nav"),
                Some(attr_unquoted(
                    (11, 15),
                    nm((11, 13), "id", "id"),
                    (13, 14),
                    tx((14, 15), "a"),
                    "a",
                )),
            )])
        },
    )
}

/// `<body><div></div>`: zero attributes.
fn fixture_zero_attributes() -> (&'static str, Gold) {
    (
        "<body><div></div>",
        Gold {
            closed: vec!["div"],
            ..gold(vec![node(
                "div",
                SelectedName::Div,
                "body",
                (6, 11),
                ((7, 10), "div"),
                None,
            )])
        },
    )
}

fn nul_gold(
    attribute: GoldAttribute,
    trigger: Range,
    diagnostic: (Range, HtmlTokenizerDiagnosticContext),
) -> Gold {
    Gold {
        closed: vec!["div"],
        diagnostics: vec![LowerDiagnostic {
            code: HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter,
            location: diagnostic.0,
            context: diagnostic.1,
            handling: HtmlTokenizerDiagnosticHandling::Recovered(
                HtmlTokenizerRecoveryKind::ReplacedNullWithReplacementCharacter,
            ),
        }],
        ..gold(vec![node(
            "div",
            SelectedName::Div,
            "body",
            trigger,
            ((7, 10), "div"),
            Some(attribute),
        )])
    }
}

/// `<body><div id="a\0b"></div>`: authored NUL in a double-quoted value.
fn fixture_nul_double() -> (String, Gold) {
    (
        "<body><div id=\"a\u{0}b\"></div>".to_owned(),
        nul_gold(
            attr_double(
                (11, 19),
                nm((11, 13), "id", "id"),
                (13, 14),
                ((14, 15), (18, 19)),
                tx((15, 18), "a\u{0}b"),
                "a\u{fffd}b",
            ),
            (6, 20),
            (
                (16, 17),
                HtmlTokenizerDiagnosticContext::AttributeValueDoubleQuoted,
            ),
        ),
    )
}

/// `<body><div id='a\0b'></div>`: authored NUL in a single-quoted value.
fn fixture_nul_single() -> (String, Gold) {
    (
        "<body><div id='a\u{0}b'></div>".to_owned(),
        nul_gold(
            attr_single(
                (11, 19),
                nm((11, 13), "id", "id"),
                (13, 14),
                ((14, 15), (18, 19)),
                tx((15, 18), "a\u{0}b"),
                "a\u{fffd}b",
            ),
            (6, 20),
            (
                (16, 17),
                HtmlTokenizerDiagnosticContext::AttributeValueSingleQuoted,
            ),
        ),
    )
}

/// `<body><div id=a\0b></div>`: authored NUL in an unquoted value.
fn fixture_nul_unquoted() -> (String, Gold) {
    (
        "<body><div id=a\u{0}b></div>".to_owned(),
        nul_gold(
            attr_unquoted(
                (11, 17),
                nm((11, 13), "id", "id"),
                (13, 14),
                tx((14, 17), "a\u{0}b"),
                "a\u{fffd}b",
            ),
            (6, 18),
            (
                (15, 16),
                HtmlTokenizerDiagnosticContext::AttributeValueUnquoted,
            ),
        ),
    )
}

/// Control: an authored U+FFFD (3 UTF-8 bytes) interprets to the same value as
/// an authored NUL but carries different authored evidence and no diagnostic.
fn fixture_authored_replacement_character() -> (String, Gold) {
    (
        "<body><div id=\"a\u{fffd}b\"></div>".to_owned(),
        Gold {
            closed: vec!["div"],
            ..gold(vec![node(
                "div",
                SelectedName::Div,
                "body",
                (6, 22),
                ((7, 10), "div"),
                Some(attr_double(
                    (11, 21),
                    nm((11, 13), "id", "id"),
                    (13, 14),
                    ((14, 15), (20, 21)),
                    tx((15, 20), "a\u{fffd}b"),
                    "a\u{fffd}b",
                )),
            )])
        },
    )
}

fn hand_authored_positive_fixtures() -> Vec<(String, Gold)> {
    let borrowed = |(source, gold): (&'static str, Gold)| (source.to_owned(), gold);
    vec![
        borrowed(fixture_double_quoted()),
        borrowed(fixture_single_quoted()),
        borrowed(fixture_unquoted()),
        borrowed(fixture_unquoted_with_slash()),
        borrowed(fixture_empty_double()),
        borrowed(fixture_empty_single()),
        borrowed(fixture_missing_value()),
        borrowed(fixture_missing_after_equals()),
        borrowed(fixture_mixed_case()),
        borrowed(fixture_same_name_nesting()),
        borrowed(fixture_heterogeneous_nesting()),
        borrowed(fixture_identical_evidence_twice()),
        borrowed(fixture_nested_then_sibling()),
        borrowed(fixture_recovery()),
        borrowed(fixture_unmatched_end()),
        borrowed(fixture_zero_attributes()),
        fixture_nul_double(),
        fixture_nul_single(),
        fixture_nul_unquoted(),
        fixture_authored_replacement_character(),
    ]
}

// ---------------------------------------------------------------------------
// Bounded deterministic generator. Expected evidence is computed only from the
// authored pieces concatenated while the source is built.
// ---------------------------------------------------------------------------

struct SourceBuilder {
    text: String,
}

impl SourceBuilder {
    fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    fn push(&mut self, piece: &str) -> Range {
        let start = self.text.len();
        self.text.push_str(piece);
        (start, self.text.len())
    }
}

#[derive(Debug, Clone, Copy)]
enum Form {
    Missing,
    MissingAfterEquals,
    Unquoted,
    DoubleQuoted,
    SingleQuoted,
    EmptyDoubleQuoted,
    EmptySingleQuoted,
}

const FORMS: [Form; 7] = [
    Form::Missing,
    Form::MissingAfterEquals,
    Form::Unquoted,
    Form::DoubleQuoted,
    Form::SingleQuoted,
    Form::EmptyDoubleQuoted,
    Form::EmptySingleQuoted,
];

#[derive(Debug, Clone, Copy)]
enum CaseStyle {
    Lower,
    Upper,
    Mixed,
}

impl CaseStyle {
    fn apply(self, text: &str) -> String {
        match self {
            Self::Lower => text.to_owned(),
            Self::Upper => text.to_ascii_uppercase(),
            Self::Mixed => text
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
}

fn push_attribute(
    builder: &mut SourceBuilder,
    spelling: &str,
    form: Form,
    value: &str,
) -> GoldAttribute {
    let name_range = builder.push(spelling);
    let name = nm(name_range, spelling, &spelling.to_ascii_lowercase());
    match form {
        Form::Missing => attr_missing(name_range, name),
        Form::MissingAfterEquals => {
            let equals = builder.push("=");
            attr_missing_after_equals((name_range.0, equals.1), name, equals, (equals.1, equals.1))
        }
        Form::Unquoted => {
            let equals = builder.push("=");
            let value_range = builder.push(value);
            attr_unquoted(
                (name_range.0, value_range.1),
                name,
                equals,
                tx(value_range, value),
                value,
            )
        }
        Form::DoubleQuoted | Form::EmptyDoubleQuoted => {
            let value = if matches!(form, Form::EmptyDoubleQuoted) {
                ""
            } else {
                value
            };
            let equals = builder.push("=");
            let open = builder.push("\"");
            let value_range = builder.push(value);
            let close = builder.push("\"");
            attr_double(
                (name_range.0, close.1),
                name,
                equals,
                (open, close),
                tx(value_range, value),
                value,
            )
        }
        Form::SingleQuoted | Form::EmptySingleQuoted => {
            let value = if matches!(form, Form::EmptySingleQuoted) {
                ""
            } else {
                value
            };
            let equals = builder.push("=");
            let open = builder.push("'");
            let value_range = builder.push(value);
            let close = builder.push("'");
            attr_single(
                (name_range.0, close.1),
                name,
                equals,
                (open, close),
                tx(value_range, value),
                value,
            )
        }
    }
}

fn push_start_tag(
    builder: &mut SourceBuilder,
    spelling: &str,
    attribute: Option<(&str, Form, &str)>,
) -> (Range, Range, Option<GoldAttribute>) {
    let open = builder.push("<");
    let raw_name = builder.push(spelling);
    let attribute = attribute.map(|(name, form, value)| {
        builder.push(" ");
        push_attribute(builder, name, form, value)
    });
    let close = builder.push(">");
    ((open.0, close.1), raw_name, attribute)
}

// ---------------------------------------------------------------------------
// Tests: authority, domain, firewall.
// ---------------------------------------------------------------------------

#[test]
fn authority_pins_and_closed_domain_are_exact() {
    let source = include_str!("in_body_selected_ordinary_attribute_successor_validation.rs");
    for pin in [
        PINNED_WHATWG_COMMIT,
        PINNED_WHATWG_SOURCE_BLOB,
        SELECTION_AUTHORITY_COMMENT,
    ] {
        assert!(source.contains(pin), "module documentation records {pin}");
    }
    assert_eq!(PINNED_WHATWG_COMMIT.len(), 40);
    assert_eq!(PINNED_WHATWG_SOURCE_BLOB.len(), 40);

    // The exact eight selected names of Issue #900, in Issue order.
    let names: Vec<&str> = SELECTED.iter().map(|name| name.as_str()).collect();
    assert_eq!(
        names,
        [
            "div", "section", "article", "aside", "footer", "header", "main", "nav"
        ]
    );
    for (index, name) in SELECTED.iter().enumerate() {
        assert_eq!(SelectedName::from_interpreted(name.as_str()), Some(*name));
        assert!(!SELECTED[index + 1..].contains(name), "no duplicate names");
    }
}

#[test]
fn product_vector_is_the_frozen_one_attribute_vector() {
    let limits = product_limits();
    assert_eq!(limits.max_source_bytes(), 36_864);
    assert_eq!(limits.max_transition_steps(), 79_872);
    assert_eq!(limits.max_emitted_tokens(), 6_144);
    assert_eq!(limits.max_diagnostics(), 256);
    assert_eq!(limits.max_attributes_per_tag(), 1);
    assert_eq!(limits.max_retained_interpreted_bytes(), 49_152);
    assert_eq!(limits.max_temporary_buffer_bytes(), 0);
}

#[test]
fn validation_module_does_not_import_production_tree_semantics() {
    let source = include_str!("in_body_selected_ordinary_attribute_successor_validation.rs");
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
        ["super::super::", "tree"].concat(),
        ["crate::html::", "tree"].concat(),
        ["super::super::", "analysis"].concat(),
        ["super::super::", "parser"].concat(),
        ["re", "gex"].concat(),
    ];
    for forbidden in forbidden {
        assert!(
            !source.contains(&forbidden),
            "forbidden production oracle: {forbidden}"
        );
    }
}

// ---------------------------------------------------------------------------
// Tests: hand-authored positive cases.
// ---------------------------------------------------------------------------

#[test]
fn double_quoted_attribute_associates_with_the_constructed_article() {
    let (source, gold) = fixture_double_quoted();
    let observation = check_positive(source, &gold);
    let selected = observation.selected_nodes();
    assert_eq!(selected.len(), 1);
    let attribute = selected[0].attribute.as_ref().expect("one attribute");
    assert_eq!(attribute.interpreted_name, "id");
    assert_eq!(attribute.interpreted_value, "a");
    assert!(matches!(attribute.syntax, ModelSyntax::DoubleQuoted { .. }));
}

#[test]
fn single_quoted_attribute_keeps_exact_syntax_evidence() {
    let (source, gold) = fixture_single_quoted();
    let observation = check_positive(source, &gold);
    let attribute = observation.selected_nodes()[0]
        .attribute
        .clone()
        .expect("one attribute");
    assert!(matches!(attribute.syntax, ModelSyntax::SingleQuoted { .. }));
}

#[test]
fn unquoted_attribute_keeps_exact_syntax_evidence() {
    let (source, gold) = fixture_unquoted();
    let observation = check_positive(source, &gold);
    let attribute = observation.selected_nodes()[0]
        .attribute
        .clone()
        .expect("one attribute");
    assert!(matches!(attribute.syntax, ModelSyntax::Unquoted { .. }));

    // A trailing `/` inside an unquoted value is value text, not a solidus.
    let (source, gold) = fixture_unquoted_with_slash();
    check_positive(source, &gold);
}

#[test]
fn empty_value_missing_value_and_missing_after_equals_stay_distinct() {
    let cases = [
        fixture_empty_double(),
        fixture_empty_single(),
        fixture_missing_value(),
        fixture_missing_after_equals(),
    ];
    let mut syntaxes = Vec::new();
    for (source, gold) in cases {
        let observation = check_positive(source, &gold);
        let attribute = observation.selected_nodes()[0]
            .attribute
            .clone()
            .expect("one attribute");
        // All four interpret to the empty value; only authored evidence
        // distinguishes them.
        assert_eq!(attribute.interpreted_value, "", "{source:?}");
        syntaxes.push(attribute.syntax);
    }
    for (index, syntax) in syntaxes.iter().enumerate() {
        for other in &syntaxes[index + 1..] {
            assert_ne!(syntax, other, "forms with equal interpreted value differ");
        }
    }
    assert!(matches!(syntaxes[0], ModelSyntax::DoubleQuoted { .. }));
    assert!(matches!(syntaxes[1], ModelSyntax::SingleQuoted { .. }));
    assert!(matches!(syntaxes[2], ModelSyntax::Missing));
    assert!(matches!(
        syntaxes[3],
        ModelSyntax::MissingAfterEquals { .. }
    ));
}

#[test]
fn mixed_case_authored_spelling_is_retained_against_normalized_names() {
    let (source, gold) = fixture_mixed_case();
    let observation = check_positive(source, &gold);
    let node = observation.selected_nodes()[0].clone();
    let Origin::Authored { raw_name, .. } = &node.origin else {
        panic!("authored");
    };
    assert_eq!(raw_name.spelling, "ArTiClE");
    assert_eq!(node.name, ModelName::Selected(SelectedName::Article));
    let attribute = node.attribute.expect("one attribute");
    assert_eq!(attribute.authored_name.spelling, "ClAsS");
    assert_eq!(attribute.interpreted_name, "class");
    assert_ne!(attribute.authored_name.spelling, attribute.interpreted_name);
}

#[test]
fn same_name_nesting_binds_each_attribute_to_its_own_construction_event() {
    let (source, gold) = fixture_same_name_nesting();
    let observation = check_positive(source, &gold);
    let selected = observation.selected_nodes();
    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].name, selected[1].name, "same element name");
    assert_ne!(selected[0].id, selected[1].id);
    assert_eq!(selected[1].parent, Some(selected[0].id));
    assert_eq!(
        selected[0].attribute.as_ref().unwrap().interpreted_value,
        "outer"
    );
    assert_eq!(
        selected[1].attribute.as_ref().unwrap().interpreted_value,
        "inner"
    );
}

#[test]
fn heterogeneous_nesting_follows_the_consumed_start_tags() {
    let (source, gold) = fixture_heterogeneous_nesting();
    let observation = check_positive(source, &gold);
    let selected = observation.selected_nodes();
    assert_eq!(selected[0].name, ModelName::Selected(SelectedName::Article));
    assert_eq!(selected[1].name, ModelName::Selected(SelectedName::Nav));
    assert_eq!(
        selected[0].attribute.as_ref().unwrap().interpreted_name,
        "id"
    );
    assert_eq!(
        selected[1].attribute.as_ref().unwrap().interpreted_name,
        "class"
    );
}

#[test]
fn identical_evidence_in_two_construction_events_yields_two_identities() {
    let (source, gold) = fixture_identical_evidence_twice();
    let observation = check_positive(source, &gold);
    let selected = observation.selected_nodes();
    let first = selected[0].attribute.as_ref().unwrap();
    let second = selected[1].attribute.as_ref().unwrap();
    // Same interpreted name and value, same syntax shape, different identity
    // and different authored anchors.
    assert_eq!(first.interpreted_name, second.interpreted_name);
    assert_eq!(first.interpreted_value, second.interpreted_value);
    assert_ne!(selected[0].id, selected[1].id);
    assert_ne!(first.complete, second.complete);
    assert_ne!(first, second);
}

#[test]
fn association_survives_recovery_and_unmatched_ends_without_mutation() {
    let (source, gold) = fixture_recovery();
    let observation = check_positive(source, &gold);
    let selected = observation.selected_nodes();
    // Both nodes are closed (one by recovery, one by closure) and keep exactly
    // the evidence they were constructed with.
    assert_eq!(observation.open.len(), 2);
    assert_eq!(
        selected[0].attribute.as_ref().unwrap().interpreted_value,
        "a"
    );
    assert_eq!(
        selected[1].attribute.as_ref().unwrap().interpreted_value,
        "b"
    );

    let (source, gold) = fixture_unmatched_end();
    let observation = check_positive(source, &gold);
    assert_eq!(observation.selected_nodes().len(), 1);
}

#[test]
fn zero_attribute_start_fabricates_no_evidence_and_synthesized_nodes_carry_none() {
    let (source, gold) = fixture_zero_attributes();
    let observation = check_positive(source, &gold);
    assert!(observation.selected_nodes()[0].attribute.is_none());
    // html and head are model-synthesized: no token named them, and none of the
    // nodes without an authored start-tag origin owns attribute evidence.
    let run = tokenize_with(source, 1, product_limits());
    for token in run.tokens() {
        if let HtmlToken::Tag(tag) = token {
            assert!(!matches!(tag.name().interpreted(), "html" | "head"));
        }
    }
    for node in &observation.nodes {
        if matches!(node.origin, Origin::Synthesized(_)) {
            assert!(node.attribute.is_none());
        }
    }
}

#[test]
fn authored_nul_stays_exact_while_the_interpreted_value_is_replacement() {
    for (source, gold) in [
        fixture_nul_double(),
        fixture_nul_single(),
        fixture_nul_unquoted(),
    ] {
        // Byte-aware construction: the authored source contains a raw 0x00.
        assert!(source.as_bytes().contains(&0), "{source:?}");
        let observation = check_positive(&source, &gold);
        let attribute = observation.selected_nodes()[0]
            .attribute
            .clone()
            .expect("one attribute");
        let authored_value = match &attribute.syntax {
            ModelSyntax::DoubleQuoted { value, .. }
            | ModelSyntax::SingleQuoted { value, .. }
            | ModelSyntax::Unquoted { value, .. } => value.spelling.clone(),
            other => panic!("unexpected syntax {other:?}"),
        };
        assert_eq!(authored_value, "a\u{0}b", "authored evidence keeps U+0000");
        assert_eq!(
            attribute.interpreted_value, "a\u{fffd}b",
            "interpreted value is U+FFFD"
        );
        assert_ne!(authored_value, attribute.interpreted_value);
        assert_eq!(observation.lower_diagnostics.len(), 1);
    }

    // Control: an authored U+FFFD interprets identically but is different
    // authored evidence with no diagnostic. Authored != interpreted, and the
    // interpreted value alone cannot recover which was authored.
    let (nul_source, nul_gold) = fixture_nul_double();
    let (fffd_source, fffd_gold) = fixture_authored_replacement_character();
    let nul = check_positive(&nul_source, &nul_gold);
    let fffd = check_positive(&fffd_source, &fffd_gold);
    let nul_attribute = nul.selected_nodes()[0].attribute.clone().unwrap();
    let fffd_attribute = fffd.selected_nodes()[0].attribute.clone().unwrap();
    assert_eq!(
        nul_attribute.interpreted_value,
        fffd_attribute.interpreted_value
    );
    assert_ne!(nul_attribute.syntax, fffd_attribute.syntax);
    assert!(fffd.lower_diagnostics.is_empty());
}

#[test]
fn distinct_source_ids_do_not_collapse_provenance() {
    let (source, gold) = fixture_double_quoted();
    let mut observations = Vec::new();
    for source_id in [1_u64, 2, 41, u64::MAX] {
        let observation = observe(source, source_id, product_limits());
        assert_eq!(observation.source_id, SourceId::new(source_id));
        assert_matches_gold(source, &observation, &gold);
        let node = observation.selected_nodes()[0].clone();
        let attribute = node.attribute.expect("one attribute");
        // Every anchor in the evidence carries this run's exact SourceId.
        assert_eq!(attribute.complete.source_id, SourceId::new(source_id));
        assert_eq!(
            attribute.authored_name.evidence.source_id,
            SourceId::new(source_id)
        );
        let Origin::Authored {
            trigger, raw_name, ..
        } = &node.origin
        else {
            panic!("authored");
        };
        assert_eq!(trigger.source_id, SourceId::new(source_id));
        assert_eq!(raw_name.evidence.source_id, SourceId::new(source_id));
        match attribute.syntax {
            ModelSyntax::DoubleQuoted {
                equals,
                open_quote,
                value,
                close_quote,
            } => {
                for anchor in [equals, open_quote, value.evidence, close_quote] {
                    assert_eq!(anchor.source_id, SourceId::new(source_id));
                }
            }
            other => panic!("unexpected syntax {other:?}"),
        }
        observations.push(observation);
    }
    for (index, observation) in observations.iter().enumerate() {
        for other in &observations[index + 1..] {
            assert_ne!(observation.nodes, other.nodes, "equal bytes, distinct ids");
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: bounded generated challenge cells.
// ---------------------------------------------------------------------------

#[test]
fn every_selected_name_syntax_form_and_case_perturbation_associates_exactly() {
    let mut cells = 0;
    for name in SELECTED {
        for element_case in [CaseStyle::Lower, CaseStyle::Upper, CaseStyle::Mixed] {
            for attribute_case in [CaseStyle::Lower, CaseStyle::Mixed] {
                for form in FORMS {
                    let mut builder = SourceBuilder::new();
                    builder.push("<body>");
                    let element = element_case.apply(name.as_str());
                    let attribute_name = attribute_case.apply("class");
                    let (trigger, raw_name, attribute) =
                        push_start_tag(&mut builder, &element, Some((&attribute_name, form, "v1")));
                    builder.push(&format!("</{}>", name.as_str()));
                    let expected = Gold {
                        closed: vec!["n"],
                        diagnostics: if matches!(form, Form::MissingAfterEquals) {
                            let close_gt = trigger.1 - 1;
                            vec![LowerDiagnostic {
                                code: HtmlTokenizerDiagnosticCode::MissingAttributeValue,
                                location: (close_gt, close_gt + 1),
                                context: HtmlTokenizerDiagnosticContext::BeforeAttributeValue,
                                handling: HtmlTokenizerDiagnosticHandling::Recovered(
                                    HtmlTokenizerRecoveryKind::CompletedTagWithMissingAttributeValue,
                                ),
                            }]
                        } else {
                            Vec::new()
                        },
                        ..gold(vec![node(
                            "n",
                            name,
                            "body",
                            trigger,
                            (raw_name, &element),
                            attribute,
                        )])
                    };
                    check_positive(&builder.text, &expected);
                    cells += 1;
                }
            }
        }
    }
    assert_eq!(cells, 8 * 3 * 2 * 7);
}

#[test]
fn every_ordered_name_pair_nests_with_identity_association() {
    let mut pairs = 0;
    for (outer_index, outer) in SELECTED.into_iter().enumerate() {
        for (inner_index, inner) in SELECTED.into_iter().enumerate() {
            // Rotate syntax forms so pairs exercise different form pairings.
            let outer_form = FORMS[(outer_index + inner_index) % FORMS.len()];
            let inner_form = FORMS[(outer_index * 3 + inner_index + 1) % FORMS.len()];
            let mut builder = SourceBuilder::new();
            builder.push("<body>");
            let (outer_trigger, outer_raw, outer_attribute) =
                push_start_tag(&mut builder, outer.as_str(), Some(("id", outer_form, "o")));
            let (inner_trigger, inner_raw, inner_attribute) = push_start_tag(
                &mut builder,
                inner.as_str(),
                Some(("class", inner_form, "i")),
            );
            builder.push(&format!("</{}></{}>", inner.as_str(), outer.as_str()));

            let mut diagnostics = Vec::new();
            for (form, trigger) in [(outer_form, outer_trigger), (inner_form, inner_trigger)] {
                if matches!(form, Form::MissingAfterEquals) {
                    diagnostics.push(LowerDiagnostic {
                        code: HtmlTokenizerDiagnosticCode::MissingAttributeValue,
                        location: (trigger.1 - 1, trigger.1),
                        context: HtmlTokenizerDiagnosticContext::BeforeAttributeValue,
                        handling: HtmlTokenizerDiagnosticHandling::Recovered(
                            HtmlTokenizerRecoveryKind::CompletedTagWithMissingAttributeValue,
                        ),
                    });
                }
            }
            let expected = Gold {
                closed: vec!["inner", "outer"],
                diagnostics,
                ..gold(vec![
                    node(
                        "outer",
                        outer,
                        "body",
                        outer_trigger,
                        (outer_raw, outer.as_str()),
                        outer_attribute,
                    ),
                    node(
                        "inner",
                        inner,
                        "outer",
                        inner_trigger,
                        (inner_raw, inner.as_str()),
                        inner_attribute,
                    ),
                ])
            };
            check_positive(&builder.text, &expected);
            pairs += 1;
        }
    }
    assert_eq!(pairs, 64);
}

// ---------------------------------------------------------------------------
// Tests: falsified assumptions. Each wrong strategy is implemented over
// lower-layer token evidence and model identity only, then shown to disagree
// with the model on a fixture built to discriminate it.
// ---------------------------------------------------------------------------

fn selected_start_tags(
    run: &HtmlTokenizerRunResult,
) -> Vec<(SelectedName, Option<(String, String)>)> {
    run.tokens()
        .iter()
        .filter_map(|token| {
            let HtmlToken::Tag(tag) = token else {
                return None;
            };
            if tag.kind() != HtmlTagKind::Start {
                return None;
            }
            let name = SelectedName::from_interpreted(tag.name().interpreted())?;
            Some((
                name,
                tag.attributes().first().map(|attribute| {
                    (
                        attribute.name().interpreted().to_owned(),
                        attribute.interpreted_value().to_owned(),
                    )
                }),
            ))
        })
        .collect()
}

fn name_keyed_assignment(
    run: &HtmlTokenizerRunResult,
    last_wins: bool,
) -> Vec<Option<(String, String)>> {
    let tags = selected_start_tags(run);
    let mut table: HashMap<SelectedName, Option<(String, String)>> = HashMap::new();
    for (name, attribute) in &tags {
        if last_wins || !table.contains_key(name) {
            table.insert(*name, attribute.clone());
        }
    }
    tags.iter().map(|(name, _)| table[name].clone()).collect()
}

#[test]
fn name_keyed_association_is_falsified_by_same_name_nesting() {
    // Same-name fixture: both wrong name-keyed strategies fail.
    let (source, gold) = fixture_same_name_nesting();
    let observation = check_positive(source, &gold);
    let run = tokenize_with(source, 1, product_limits());
    let truth = observation.interpreted_pairs();
    assert_ne!(name_keyed_assignment(&run, true), truth, "last-wins");
    assert_ne!(name_keyed_assignment(&run, false), truth, "first-wins");

    // Every selected name falsifies name-keying on its own diagonal.
    for name in SELECTED {
        let source = format!(
            "<body><{n} id=\"outer\"><{n} class=\"inner\"></{n}></{n}>",
            n = name.as_str()
        );
        let observation = observe(&source, 1, product_limits());
        assert_eq!(observation.completion, Completion::Complete, "{source}");
        let run = tokenize_with(&source, 1, product_limits());
        assert_ne!(
            name_keyed_assignment(&run, true),
            observation.interpreted_pairs(),
            "{source}"
        );
        assert_ne!(
            name_keyed_assignment(&run, false),
            observation.interpreted_pairs(),
            "{source}"
        );
    }

    // Heterogeneous names cannot falsify name-keying: distinct keys happen to
    // agree. This is why repeated-name fixtures are mandatory.
    let (source, gold) = fixture_heterogeneous_nesting();
    let observation = check_positive(source, &gold);
    let run = tokenize_with(source, 1, product_limits());
    assert_eq!(
        name_keyed_assignment(&run, true),
        observation.interpreted_pairs()
    );
}

#[test]
fn final_tree_depth_pairing_is_falsified() {
    // Wrong strategy: order nodes by final-tree depth, then hand out the
    // attributes in token order. Valid only if depth order equaled creation
    // order, which the nested-then-sibling fixture breaks.
    let (source, gold) = fixture_nested_then_sibling();
    let observation = check_positive(source, &gold);
    let truth = observation.interpreted_pairs();

    let mut by_depth: Vec<&ModelNode> = observation.selected_nodes();
    by_depth.sort_by_key(|node| (observation.depth(node), node.id.0));
    let in_token_order = truth.clone();
    let mut paired = vec![None; truth.len()];
    for (slot, node) in by_depth.iter().enumerate() {
        paired[node.id.0 - 3] = in_token_order[slot].clone();
    }
    assert_ne!(paired, truth, "depth pairing assigns the wrong attributes");
}

#[test]
fn offsets_alone_do_not_identify_ownership() {
    let (source, _) = fixture_double_quoted();
    let first = observe(source, 1, product_limits());
    let second = observe(source, 2, product_limits());
    let offsets = |observation: &Observation| -> Vec<Range> {
        observation
            .selected_nodes()
            .iter()
            .filter_map(|node| {
                node.attribute
                    .as_ref()
                    .map(|attribute| attribute.complete.range)
            })
            .collect()
    };
    assert_eq!(offsets(&first), offsets(&second), "bare offsets collide");
    assert_ne!(first.nodes, second.nodes, "full evidence does not");
    assert_ne!(
        first.selected_nodes()[0].attribute,
        second.selected_nodes()[0].attribute
    );
}

#[test]
fn interpreted_pair_alone_is_not_sufficient_evidence() {
    // Three authored syntaxes, one interpreted (name, value) pair.
    let mut associations = Vec::new();
    for source in [
        "<body><div id=a></div>",
        "<body><div id=\"a\"></div>",
        "<body><div id='a'></div>",
    ] {
        let observation = observe(source, 1, product_limits());
        assert_eq!(observation.completion, Completion::Complete);
        associations.push(observation.selected_nodes()[0].attribute.clone().unwrap());
    }
    for attribute in &associations {
        assert_eq!(attribute.interpreted_name, "id");
        assert_eq!(attribute.interpreted_value, "a");
    }
    for (index, attribute) in associations.iter().enumerate() {
        for other in &associations[index + 1..] {
            assert_ne!(attribute, other);
            assert_ne!(attribute.syntax, other.syntax);
        }
    }

    // Authored name case: same interpreted pair, distinct authored name.
    let lower = observe("<body><div id=a></div>", 1, product_limits());
    let upper = observe("<body><div ID=a></div>", 1, product_limits());
    let lower = lower.selected_nodes()[0].attribute.clone().unwrap();
    let upper = upper.selected_nodes()[0].attribute.clone().unwrap();
    assert_eq!(lower.interpreted_name, upper.interpreted_name);
    assert_ne!(lower.authored_name.spelling, upper.authored_name.spelling);
    assert_ne!(lower, upper);
}

// ---------------------------------------------------------------------------
// Tests: negative and boundary controls.
// ---------------------------------------------------------------------------

fn assert_no_selected_node(context: &str, observation: &Observation) {
    assert!(
        observation.selected_nodes().is_empty(),
        "{context}: no selected node may exist: {observation:?}"
    );
    assert_shell_predecessor(context, observation);
}

#[test]
fn second_attribute_is_a_resource_fact_not_a_semantic_refusal() {
    // Frozen Product vector: the second attribute reaches ResourceLimit
    // (limit 1, attempted 2) at the offset where it begins. The start tag is
    // never emitted, so nothing is associated.
    for (source, second_attribute_offset) in [
        ("<body><div a=1 b=2></div>", 15),
        ("<body><div a a></div>", 13),
    ] {
        let observation = observe(source, 1, product_limits());
        assert_eq!(
            observation.lower_stop,
            Some(LowerStop::ResourceLimit {
                resource: HtmlTokenizerResource::AttributesPerTag,
                limit: 1,
                attempted: 2,
                at: at(
                    SourceId::new(1),
                    (second_attribute_offset, second_attribute_offset)
                ),
            }),
            "{source:?}"
        );
        assert_eq!(
            observation.completion,
            Completion::LowerLayerIncomplete(LowerLayerCategory::ResourceLimit),
            "{source:?}"
        );
        assert_no_selected_node(source, &observation);
    }

    // The same text under a limit that allows it: the lower layer completes
    // both attributes. The resource boundary was a policy fact, not HTML.
    let run = tokenize_with("<body><div a=1 b=2></div>", 1, generous_limits());
    assert!(matches!(
        run.completion(),
        HtmlTokenizerCompletion::Complete
    ));
    let HtmlToken::Tag(tag) = &run.tokens()[1] else {
        panic!("start tag");
    };
    assert_eq!(tag.attributes().len(), 2);

    // The model's own bounded-theorem boundary is a different category from
    // the lower-layer ResourceLimit, and never reports one as the other.
    let observation = observe("<body><div a=1 b=2></div>", 1, generous_limits());
    assert_eq!(
        observation.completion,
        Completion::ModelRefused {
            refusal: Refusal::MoreThanOneAttribute,
            token_index: 1,
        }
    );
    assert!(observation.lower_stop.is_none());
    assert_no_selected_node("two attributes", &observation);

    // The duplicate disposition is retained lower-layer evidence only.
    let run = tokenize_with("<body><div a a></div>", 1, generous_limits());
    let HtmlToken::Tag(tag) = &run.tokens()[1] else {
        panic!("start tag");
    };
    assert_eq!(
        tag.attributes()[0].disposition(),
        HtmlAttributeDisposition::Effective
    );
    assert_eq!(
        tag.attributes()[1].disposition(),
        HtmlAttributeDisposition::DuplicateOf { first_index: 0 }
    );

    // The limit is configuration, not a semantic maximum: limit 0 refuses even
    // the first attribute; limit 2 admits two.
    let zero = HtmlTokenizerLimits::new(36_864, 79_872, 6_144, 256, 0, 49_152, 0);
    let observation = observe("<body><div id=a></div>", 1, zero);
    assert_eq!(
        observation.lower_stop,
        Some(LowerStop::ResourceLimit {
            resource: HtmlTokenizerResource::AttributesPerTag,
            limit: 0,
            attempted: 1,
            at: at(SourceId::new(1), (11, 11)),
        })
    );
    assert_no_selected_node("limit zero", &observation);
}

#[test]
fn outside_domain_starts_keep_attributes_outside_the_theorem() {
    // Shell: `<body>` with an attribute is refused before any node exists.
    let observation = observe("<body id=a>", 1, product_limits());
    assert_eq!(
        observation.completion,
        Completion::ModelRefused {
            refusal: Refusal::ShellStartAttributes,
            token_index: 0,
        }
    );
    assert!(observation.lower_stop.is_none());
    assert!(
        observation
            .nodes
            .iter()
            .all(|node| node.attribute.is_none())
    );
    assert!(
        observation
            .nodes
            .iter()
            .all(|node| !matches!(node.name, ModelName::Body)),
        "body is not constructed from a refused shell start"
    );

    // Paragraph and nearby non-selected names carrying one attribute.
    for name in [
        "p",
        "span",
        "search",
        "address",
        "blockquote",
        "center",
        "details",
        "dialog",
        "dir",
        "dl",
        "fieldset",
        "figcaption",
        "figure",
        "hgroup",
        "menu",
        "ol",
        "summary",
        "ul",
        "divx",
        "mainx",
        "sectio",
        "x-div",
        "html",
        "head",
        "body",
    ] {
        let source = format!("<body><{name} id=a>");
        let observation = observe(&source, 1, product_limits());
        assert!(observation.lower_stop.is_none(), "{source}");
        assert_eq!(
            observation.completion,
            Completion::ModelRefused {
                refusal: Refusal::OutsideSelectedDomain,
                token_index: 1,
            },
            "{source}"
        );
        assert_no_selected_node(&source, &observation);
    }

    // Names whose start tag the tokenizer itself defers (tree-controlled
    // tokenizer modes). The start tag is emitted at token index 1 and the lower
    // layer then stops with UnsupportedCapability; the model refuses the same
    // token as outside the selected domain. Both facts are retained and
    // neither is upgraded into the other or into attribute meaning.
    for (name, mode) in [
        ("style", HtmlTokenizerMode::RawText),
        ("title", HtmlTokenizerMode::Rcdata),
        ("textarea", HtmlTokenizerMode::Rcdata),
    ] {
        let source = format!("<body><{name} id=a>");
        let observation = observe(&source, 1, product_limits());
        assert_eq!(
            observation.lower_stop,
            Some(LowerStop::Unsupported {
                capability: HtmlTokenizerCapability::ContextDependentTokenizerMode { mode },
                availability: HtmlTokenizerCapabilityAvailability::Deferred,
                trigger: LowerTrigger::EmittedToken { token_index: 1 },
            }),
            "{source}"
        );
        assert_eq!(
            observation.completion,
            Completion::ModelRefused {
                refusal: Refusal::OutsideSelectedDomain,
                token_index: 1,
            },
            "{source}"
        );
        assert_no_selected_node(&source, &observation);
    }
}

#[test]
fn end_tag_attributes_are_never_associated_with_a_start_node() {
    // A start tag without an attribute, closed by an end tag that carries one.
    let observation = observe("<body><div></div id=x>", 1, product_limits());
    assert_eq!(
        observation.completion,
        Completion::ModelRefused {
            refusal: Refusal::EndTagAttributes,
            token_index: 2,
        }
    );
    // The lower layer completed and kept its own parse diagnostic.
    assert!(observation.lower_stop.is_none());
    assert_eq!(
        observation.lower_diagnostics,
        vec![LowerDiagnostic {
            code: HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
            location: (17, 19),
            context: HtmlTokenizerDiagnosticContext::AttributeName,
            handling: HtmlTokenizerDiagnosticHandling::Recovered(
                HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
            ),
        }]
    );
    let selected = observation.selected_nodes();
    assert_eq!(selected.len(), 1);
    assert!(selected[0].attribute.is_none(), "no attribute fabricated");

    // A start tag with its own attribute keeps it; the end tag's attribute
    // neither replaces nor joins it.
    let observation = observe("<body><div id=\"a\"></div id=x>", 1, product_limits());
    assert_eq!(
        observation.completion,
        Completion::ModelRefused {
            refusal: Refusal::EndTagAttributes,
            token_index: 2,
        }
    );
    assert_eq!(
        observation.lower_diagnostics,
        vec![LowerDiagnostic {
            code: HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
            location: (24, 26),
            context: HtmlTokenizerDiagnosticContext::AttributeName,
            handling: HtmlTokenizerDiagnosticHandling::Recovered(
                HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
            ),
        }]
    );
    let selected = observation.selected_nodes();
    assert_eq!(selected.len(), 1);
    let attribute = selected[0].attribute.as_ref().expect("start attribute");
    assert_eq!(attribute.interpreted_value, "a");
    assert_eq!(attribute.complete.range, (11, 17));
    assert_eq!(observation.open.len(), 3, "refused end tag closed nothing");
}

#[test]
fn self_closing_selected_starts_stay_outside_the_theorem() {
    for source in [
        "<body><div/></div>",
        "<body><div id=\"a\"/></div>",
        "<body><nav class='b' /></nav>",
    ] {
        let observation = observe(source, 1, product_limits());
        assert!(observation.lower_stop.is_none(), "{source:?}");
        assert_eq!(
            observation.completion,
            Completion::ModelRefused {
                refusal: Refusal::SelfClosingTag,
                token_index: 1,
            },
            "{source:?}"
        );
        assert_no_selected_node(source, &observation);
    }
}

#[test]
fn eof_and_abandoned_tags_fabricate_no_attribute() {
    // EOF inside the start tag abandons it: the lower layer completes with a
    // parse diagnostic, emits no start token, and no node or attribute exists.
    for (source, location, context) in [
        (
            "<body><div id=\"a\"",
            17,
            HtmlTokenizerDiagnosticContext::AfterAttributeValueQuoted,
        ),
        (
            "<body><div id=\"a",
            16,
            HtmlTokenizerDiagnosticContext::AttributeValueDoubleQuoted,
        ),
    ] {
        let observation = observe(source, 1, product_limits());
        assert_eq!(observation.completion, Completion::Complete, "{source:?}");
        assert_eq!(
            observation.lower_diagnostics,
            vec![LowerDiagnostic {
                code: HtmlTokenizerDiagnosticCode::EofInTag,
                location: (location, location),
                context,
                handling: HtmlTokenizerDiagnosticHandling::Recovered(
                    HtmlTokenizerRecoveryKind::AbandonedIncompleteTagAtEof,
                ),
            }],
            "{source:?}"
        );
        assert_no_selected_node(source, &observation);
    }

    // A committed start tag keeps its attribute when only the *end* tag is
    // abandoned at EOF; the node stays open and the diagnostic is lower-layer.
    let observation = observe("<body><div id=\"a\"></div", 1, product_limits());
    assert_eq!(observation.completion, Completion::Complete);
    let selected = observation.selected_nodes();
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].attribute.as_ref().unwrap().complete.range,
        (11, 17)
    );
    assert_eq!(observation.open.len(), 3, "div stays open");
    assert!(observation.closures.is_empty());
    assert_eq!(
        observation.lower_diagnostics,
        vec![LowerDiagnostic {
            code: HtmlTokenizerDiagnosticCode::EofInTag,
            location: (23, 23),
            context: HtmlTokenizerDiagnosticContext::TagName,
            handling: HtmlTokenizerDiagnosticHandling::Recovered(
                HtmlTokenizerRecoveryKind::AbandonedIncompleteTagAtEof,
            ),
        }]
    );
}

#[test]
fn committed_attribute_survives_a_later_lower_layer_stop() {
    // `<!x>` is a MarkupDeclaration the tokenizer defers. The valid selected
    // start before it is already committed evidence; the stop adds nothing.
    let observation = observe("<body><div id=\"a\"><!x>", 1, product_limits());
    assert_eq!(
        observation.lower_stop,
        Some(LowerStop::Unsupported {
            capability: HtmlTokenizerCapability::MarkupDeclaration,
            availability: HtmlTokenizerCapabilityAvailability::Deferred,
            trigger: LowerTrigger::Input(at(SourceId::new(1), (18, 20))),
        })
    );
    assert_eq!(
        observation.completion,
        Completion::LowerLayerIncomplete(LowerLayerCategory::UnsupportedCapability)
    );
    let selected = observation.selected_nodes();
    assert_eq!(selected.len(), 1);
    let attribute = selected[0].attribute.clone().expect("committed attribute");
    assert_eq!(attribute.complete.range, (11, 17));
    assert_eq!(attribute.interpreted_name, "id");
    assert_eq!(attribute.interpreted_value, "a");
    // Committed evidence ends exactly at the start tag; the model claims
    // nothing past the lower-layer coverage and has no EOF-derived meaning.
    assert_eq!(observation.committed_end, 18);
    assert_eq!(observation.lower_processed_end, 18);
    assert_eq!(observation.open.len(), 3, "div is still open");
    assert!(observation.closures.is_empty());

    // Nested: both committed attributes survive, and no future attribute
    // evidence is invented for anything after the stop.
    let observation = observe(
        "<body><article id=\"outer\"><nav class=\"b\"><!x>",
        1,
        product_limits(),
    );
    assert_eq!(
        observation.completion,
        Completion::LowerLayerIncomplete(LowerLayerCategory::UnsupportedCapability)
    );
    assert_eq!(
        observation.lower_stop,
        Some(LowerStop::Unsupported {
            capability: HtmlTokenizerCapability::MarkupDeclaration,
            availability: HtmlTokenizerCapabilityAvailability::Deferred,
            trigger: LowerTrigger::Input(at(SourceId::new(1), (41, 43))),
        })
    );
    assert_eq!(
        observation.interpreted_pairs(),
        vec![
            Some(("id".to_owned(), "outer".to_owned())),
            Some(("class".to_owned(), "b".to_owned())),
        ]
    );
    assert_eq!(observation.committed_end, 41);
}

#[test]
fn lower_layer_categories_stay_distinct() {
    let unsupported = observe("<body><div id=a><!x>", 1, product_limits());
    let resource = observe("<body><div a=1 b=2>", 1, product_limits());
    let invalid = observe(
        "<body><div id=a></div>",
        1,
        HtmlTokenizerLimits::new(36_864, 0, 6_144, 256, 1, 49_152, 0),
    );
    let refused = observe("<body><p id=a>", 1, product_limits());
    let complete = observe("<body><div id=a></div>", 1, product_limits());

    assert_eq!(
        unsupported.completion,
        Completion::LowerLayerIncomplete(LowerLayerCategory::UnsupportedCapability)
    );
    assert_eq!(
        resource.completion,
        Completion::LowerLayerIncomplete(LowerLayerCategory::ResourceLimit)
    );
    assert_eq!(
        invalid.completion,
        Completion::LowerLayerIncomplete(LowerLayerCategory::InvalidConfiguration)
    );
    assert_eq!(invalid.lower_stop, Some(LowerStop::InvalidConfiguration));
    assert!(matches!(
        refused.completion,
        Completion::ModelRefused { .. }
    ));
    assert_eq!(complete.completion, Completion::Complete);

    // A tokenizer parse diagnostic is not an incompleteness.
    let diagnosed = observe("<body><div id=></div>", 1, product_limits());
    assert_eq!(diagnosed.completion, Completion::Complete);
    assert!(!diagnosed.lower_diagnostics.is_empty());

    let all = [&unsupported, &resource, &invalid, &refused, &complete];
    for (index, observation) in all.iter().enumerate() {
        for other in &all[index + 1..] {
            assert_ne!(observation.completion, other.completion);
        }
    }
    // InternalInvariantFailure cannot be induced from valid fixtures; it keeps
    // its own category and is never folded into another.
    assert_ne!(
        LowerStop::InternalInvariantFailure.category(),
        LowerLayerCategory::UnsupportedCapability
    );
    assert_ne!(
        LowerStop::InternalInvariantFailure.category(),
        LowerLayerCategory::ResourceLimit
    );
}

#[test]
fn selected_domain_admits_exactly_the_eight_names() {
    for name in SELECTED {
        let source = format!("<body><{n} id=a></{n}>", n = name.as_str());
        let observation = observe(&source, 1, product_limits());
        assert_eq!(observation.completion, Completion::Complete, "{source}");
        assert_eq!(observation.selected_nodes().len(), 1, "{source}");
    }
    for name in [
        "p", "span", "search", "address", "divs", "xdiv", "mai", "navv",
    ] {
        let source = format!("<body><{name} id=a></{name}>");
        let observation = observe(&source, 1, product_limits());
        assert_eq!(
            observation.completion,
            Completion::ModelRefused {
                refusal: Refusal::OutsideSelectedDomain,
                token_index: 1,
            },
            "{source}"
        );
        assert!(SelectedName::from_interpreted(name).is_none(), "{name}");
    }
}

#[test]
fn repeated_runs_agree_on_identity_association_ordering_and_completion() {
    for (source, gold) in hand_authored_positive_fixtures() {
        let first = check_positive(&source, &gold);
        for _ in 0..5 {
            let again = observe(&source, 1, product_limits());
            assert_eq!(again.nodes, first.nodes, "{source:?}");
            assert_eq!(again.interpreted_pairs(), first.interpreted_pairs());
            assert_eq!(again.completion, first.completion);
            assert_eq!(again.closures, first.closures);
        }
    }
}
