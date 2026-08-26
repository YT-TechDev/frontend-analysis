//! Immutable, validated result meaning for TC-S1 and its accepted TC-S2,
//! TC-S3, TC-S4, TC-S5, TC-S6, and TC-S7 successors.
//!
//! This module owns the durable half of the accepted Candidate C model: the
//! frozen tree, constructed identity, authored/synthesized provenance,
//! selective action and diagnostic evidence, committed tree coverage, and
//! effective completion. It owns no mutable construction state and never
//! observes the tokenizer or the private
//! [`session`](super::session).
//!
//! # Distinct relations, never one generic fact
//!
//! A selected ordinary element leaves the open-element state in exactly one of
//! two ways, and the difference is durable meaning rather than presentation.
//! [`HtmlTreeActionKind::ClosedSelectedOrdinaryElement`] means the element's
//! *own* exact authored end tag closed it.
//! [`HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag`] means
//! *no matching end tag caused this pop*: the element was removed because an
//! enclosing element's end tag needed it out of the way. A later authored end
//! tag of the popped element's own name may still appear in the source — it is
//! then an unmatched end tag, and it still closes nothing. Collapsing the two
//! relations into one "closed by this token" fact would fabricate authored
//! evidence, so [`freeze`] proves them separately and proves that no element
//! receives both.
//!
//! TC-S5 adds a third, separate Paragraph domain rather than widening the
//! authored-only selected ordinary domain. A Paragraph may originate from its
//! own authored `<p>` start tag or be synthesized by the unmatched authored
//! `</p>` rule. A synthesized Paragraph has no authored start-tag evidence;
//! the `</p>` remains trigger / diagnostic / closure evidence only. Matching
//! P closure, start-triggered P closure, and unmatched-end synthesized P
//! closure are also separate durable relations and are never TC-S4 ancestor
//! recovery.
//!
//! Everything here is crate-private. No item is `pub`, serialized, or
//! promised across results, runs, source edits, or implementation revisions.
//!
//! # Freeze
//!
//! [`freeze`] is the single finalization boundary. It consumes the private
//! session's [`HtmlDocumentShellParts`] plus the validated
//! [`HtmlTokenizerRunResult`] and either returns an immutable
//! [`HtmlDocumentShellAnalysis`] whose invariants have all been checked, or a
//! typed [`HtmlTreeFreezeError`]. A freeze failure is an operation/boundary
//! error: it is neither an HTML parse diagnostic nor unsupported input.
//!
//! # What identity means here
//!
//! [`HtmlConstructedNodeId`] means *committed semantic creation-event order*
//! and nothing else. It is not a storage index, arena slot, pointer,
//! allocation order, [`SourceId`], source range, tokenizer token index, final
//! tree position, or runtime/browser identity. Stored relationships are
//! explicit identities; [`HtmlDocumentShellAnalysis::node`] resolves them by
//! searching for a matching identity, never by indexing storage, so private
//! storage order may be replaced without changing any durable meaning.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceRangeError, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::result::{HtmlTokenizerCompletion, HtmlTokenizerRunResult};

/// A result-scoped constructed-node identity.
///
/// The semantic meaning is the order in which the complete semantic
/// node-creation action committed during one tree-construction run. The counter that
/// admits these identities advances only after a creation action has fully
/// committed, so a refused or unsupported action consumes no identity.
///
/// No raw-value accessor exists, and no encoding, cross-result, cross-run,
/// cross-edit, cross-revision, public, serialized, or runtime-correlation
/// stability is promised. [`Ord`] compares committed creation order, which is
/// semantic; the `Debug` projection is a debugging aid only and is not a
/// contract.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct HtmlConstructedNodeId(u32);

impl fmt::Debug for HtmlConstructedNodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "HtmlConstructedNodeId(creation#{})", self.0)
    }
}

/// Admits committed semantic creation-event identities for exactly one run.
///
/// Held privately by the construction session. [`Self::reserve`] resolves the
/// only fallible part up front so the whole creation action can be prepared
/// before any mutation; [`Self::commit`] then advances, so the ordinal counts
/// committed creation events and nothing else.
pub(super) struct HtmlConstructedIdentityCounter {
    next: u32,
}

impl HtmlConstructedIdentityCounter {
    pub(super) const fn new() -> Self {
        Self { next: 0 }
    }

    /// Reserves the identity the next committed creation event will receive,
    /// without advancing.
    ///
    /// Returns `None` on ordinal exhaustion, which is an internal invariant
    /// condition rather than ordinary input behavior; callers turn it into a
    /// typed session error rather than a panic. Reserving proves the headroom
    /// [`Self::commit`] then consumes, so every fallible part of a creation
    /// action can be resolved before any mutation happens.
    pub(super) const fn reserve(&self) -> Option<HtmlConstructedNodeId> {
        match self.next.checked_add(1) {
            Some(_) => Some(HtmlConstructedNodeId(self.next)),
            None => None,
        }
    }

    /// Advances past a previously reserved identity.
    ///
    /// Called only after the complete semantic creation action has committed,
    /// so the ordinal means committed creation-event order and never counts a
    /// refused or unsupported action. [`Self::reserve`] already proved the
    /// headroom this consumes.
    pub(super) fn commit(&mut self, reserved: HtmlConstructedNodeId) {
        debug_assert_eq!(reserved.0, self.next);
        self.next = self.next.saturating_add(1);
    }

    /// How many committed semantic creation events this run admitted.
    pub(super) const fn admitted(&self) -> u32 {
        self.next
    }
}

/// The three element names the TC-S1 document shell proves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlShellElementName {
    Html,
    Head,
    Body,
}

/// Why a shell element with no authored source exists.
///
/// A synthesis cause is not an authored origin and carries no source
/// evidence. The token that made the structure necessary is recorded
/// separately as action trigger evidence.
///
/// TC-S1 proves exactly one cause. *Which* element was implied is already the
/// element's own [`HtmlShellElement::name`], so it is deliberately not
/// restated here where the two could drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlSynthesisCause {
    /// The document's implied-structure rules required this shell element to
    /// exist before the trigger token could be processed.
    ImpliedByDocumentStructure,
}

/// Where a shell element node's existence comes from.
#[derive(Clone)]
pub(crate) enum HtmlShellElementOrigin {
    /// Exact retained authored evidence, propagated unchanged from the
    /// validated start-tag token: the complete authored tag and the raw
    /// tag-name spelling.
    Authored {
        complete: SourceAnchor,
        raw_name: SourceAnchor,
    },
    /// No authored source. This is explicit semantic absence, never an empty
    /// range, dummy anchor, nearest-token anchor, or parent anchor.
    Synthesized(HtmlSynthesisCause),
}

impl fmt::Debug for HtmlShellElementOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authored { complete, raw_name } => formatter
                .debug_struct("Authored")
                .field("source_id", &complete.source_id())
                .field("complete_range", &complete.range())
                .field("raw_name_range", &raw_name.range())
                .finish(),
            Self::Synthesized(cause) => formatter.debug_tuple("Synthesized").field(cause).finish(),
        }
    }
}

/// A shell element observation.
#[derive(Debug, Clone)]
pub(crate) struct HtmlShellElement {
    name: HtmlShellElementName,
    origin: HtmlShellElementOrigin,
}

impl HtmlShellElement {
    pub(super) fn new(name: HtmlShellElementName, origin: HtmlShellElementOrigin) -> Self {
        Self { name, origin }
    }

    pub(crate) fn name(&self) -> HtmlShellElementName {
        self.name
    }

    pub(crate) fn origin(&self) -> &HtmlShellElementOrigin {
        &self.origin
    }
}

/// The closed selected ordinary HTML-element name domain the accepted TC-S4
/// theorem proves.
///
/// Deliberately separate from [`HtmlShellElementName`], which stays
/// `html`/`head`/`body` only: neither domain may be stretched to carry the
/// other's meaning. TC-S3 closed this domain at `div`; TC-S4 extends it to
/// exactly `div` and `section` and no further. It is still not an
/// arbitrary-name, generic ordinary-element, generic block-element, or
/// namespace-switching representation, and membership is decided by type
/// rather than by a stored string.
///
/// Equality on this type is what makes an end tag *matching*: two selected
/// ordinary elements correspond exactly when this closed name is equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlSelectedOrdinaryElementName {
    Div,
    Section,
}

impl HtmlSelectedOrdinaryElementName {
    /// The interpreted tag name this closed domain member is spelled with.
    ///
    /// Used only to correlate a recorded closure or recovery against the
    /// retained emitted end-tag token. It is not a parser table and performs
    /// no source lookup.
    const fn interpreted(self) -> &'static str {
        match self {
            Self::Div => "div",
            Self::Section => "section",
        }
    }
}

/// A selected ordinary HTML element observation.
///
/// The HTML namespace is a type invariant here rather than a stored field:
/// TC-S3 proves no namespace switching and no foreign content, so no generic
/// namespace enum is introduced to record a value that cannot vary.
///
/// A selected ordinary element is authored-only. The accepted theorem creates
/// one exactly from its own authored start tag, so there is no synthesized
/// variant and no synthesis cause: its matching end tag is closure evidence
/// recorded separately as [`HtmlTreeActionKind::ClosedSelectedOrdinaryElement`]
/// and is never this node's origin. TC-S5 intentionally keeps this theorem
/// unchanged and uses [`HtmlParagraphElement`] for P instead.
#[derive(Clone)]
pub(crate) struct HtmlSelectedOrdinaryElement {
    name: HtmlSelectedOrdinaryElementName,
    complete: SourceAnchor,
    raw_name: SourceAnchor,
}

impl HtmlSelectedOrdinaryElement {
    pub(super) fn new(
        name: HtmlSelectedOrdinaryElementName,
        complete: SourceAnchor,
        raw_name: SourceAnchor,
    ) -> Self {
        Self {
            name,
            complete,
            raw_name,
        }
    }

    pub(crate) fn name(&self) -> HtmlSelectedOrdinaryElementName {
        self.name
    }

    /// The exact retained complete authored start tag, propagated unchanged
    /// from the validated start-tag token.
    pub(crate) fn complete(&self) -> &SourceAnchor {
        &self.complete
    }

    /// The exact retained raw tag-name spelling. A mixed-case `<DiV>` or
    /// `<SeCtIoN>` keeps its exact authored spelling here while the
    /// interpreted name stays [`HtmlSelectedOrdinaryElementName::Div`] or
    /// [`HtmlSelectedOrdinaryElementName::Section`].
    pub(crate) fn raw_name(&self) -> &SourceAnchor {
        &self.raw_name
    }
}

impl fmt::Debug for HtmlSelectedOrdinaryElement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlSelectedOrdinaryElement")
            .field("name", &self.name)
            .field("source_id", &self.complete.source_id())
            .field("complete_range", &self.complete.range())
            .field("raw_name_range", &self.raw_name.range())
            .finish()
    }
}

/// Why a Paragraph with no authored start-tag source exists.
///
/// This is a distinct synthesis domain from [`HtmlSynthesisCause`]: shell
/// structure is implied by document construction, whereas this cause belongs
/// only to the bounded unmatched-`</p>` InBody rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlParagraphSynthesisCause {
    UnmatchedParagraphEndTag,
}

/// Where a constructed Paragraph's existence comes from.
#[derive(Clone)]
pub(crate) enum HtmlParagraphElementOrigin {
    /// Exact retained authored `<p>` start-tag evidence.
    Authored {
        complete: SourceAnchor,
        raw_name: SourceAnchor,
    },
    /// Explicit absence of authored start-tag source. The unmatched `</p>`
    /// trigger is retained separately and is never copied here.
    Synthesized(HtmlParagraphSynthesisCause),
}

impl fmt::Debug for HtmlParagraphElementOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authored { complete, raw_name } => formatter
                .debug_struct("Authored")
                .field("source_id", &complete.source_id())
                .field("complete_range", &complete.range())
                .field("raw_name_range", &raw_name.range())
                .finish(),
            Self::Synthesized(cause) => formatter.debug_tuple("Synthesized").field(cause).finish(),
        }
    }
}

/// A bounded HTML Paragraph observation.
///
/// The element name is a type invariant: this type means exactly HTML `p` and
/// nothing else. That avoids widening the authored-only `Div | Section`
/// domain or introducing a generic arbitrary-name element representation.
#[derive(Debug, Clone)]
pub(crate) struct HtmlParagraphElement {
    origin: HtmlParagraphElementOrigin,
}

impl HtmlParagraphElement {
    pub(super) fn new(origin: HtmlParagraphElementOrigin) -> Self {
        Self { origin }
    }

    pub(crate) fn origin(&self) -> &HtmlParagraphElementOrigin {
        &self.origin
    }
}

/// The interpreted name of a constructed element, in whichever closed domain
/// owns it.
///
/// This is a projection for reading an element's name without first knowing
/// which domain it belongs to. It never merges the domains: each arm stays
/// exactly its own closed meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlElementName {
    Shell(HtmlShellElementName),
    SelectedOrdinary(HtmlSelectedOrdinaryElementName),
    Paragraph,
}

/// What kind of element a constructed element node is.
///
/// Shell, selected ordinary, and Paragraph meaning are separate closed
/// domains. A shell element may be authored or synthesized; a selected
/// ordinary element is always authored; a Paragraph has its own authored or
/// unmatched-end-synthesized origin model.
#[derive(Debug, Clone)]
pub(crate) enum HtmlElement {
    Shell(HtmlShellElement),
    SelectedOrdinary(HtmlSelectedOrdinaryElement),
    Paragraph(HtmlParagraphElement),
}

impl HtmlElement {
    pub(crate) fn name(&self) -> HtmlElementName {
        match self {
            Self::Shell(element) => HtmlElementName::Shell(element.name()),
            Self::SelectedOrdinary(element) => HtmlElementName::SelectedOrdinary(element.name()),
            Self::Paragraph(_) => HtmlElementName::Paragraph,
        }
    }

    /// The shell element this is, when it is one.
    pub(crate) fn shell(&self) -> Option<&HtmlShellElement> {
        match self {
            Self::Shell(element) => Some(element),
            Self::SelectedOrdinary(_) | Self::Paragraph(_) => None,
        }
    }

    /// The selected ordinary element this is, when it is one.
    pub(crate) fn selected_ordinary(&self) -> Option<&HtmlSelectedOrdinaryElement> {
        match self {
            Self::SelectedOrdinary(element) => Some(element),
            Self::Shell(_) | Self::Paragraph(_) => None,
        }
    }

    /// The Paragraph this is, when it is one.
    pub(crate) fn paragraph(&self) -> Option<&HtmlParagraphElement> {
        match self {
            Self::Paragraph(element) => Some(element),
            Self::Shell(_) | Self::SelectedOrdinary(_) => None,
        }
    }
}

/// One exact ordered authored character contribution to a text node.
///
/// The retained anchor is the originating validated character token's own
/// source evidence, cloned unchanged. No source rescanning, source searching,
/// endpoint reconstruction, or second tokenizer produces it.
#[derive(Clone)]
pub(crate) struct HtmlTextContribution {
    source: SourceAnchor,
    interpreted: String,
}

impl HtmlTextContribution {
    pub(super) fn new(source: SourceAnchor, interpreted: String) -> Self {
        Self {
            source,
            interpreted,
        }
    }

    pub(crate) fn source(&self) -> &SourceAnchor {
        &self.source
    }

    pub(crate) fn interpreted(&self) -> &str {
        &self.interpreted
    }
}

impl fmt::Debug for HtmlTextContribution {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlTextContribution")
            .field("source_id", &self.source.source_id())
            .field("range", &self.source.range())
            .field("interpreted_byte_len", &self.interpreted.len())
            .finish()
    }
}

/// A text observation: the interpreted characters plus the exact ordered
/// authored contributions that produced them.
#[derive(Clone)]
pub(crate) struct HtmlTextNode {
    interpreted: String,
    contributions: Vec<HtmlTextContribution>,
}

impl HtmlTextNode {
    pub(super) fn new(interpreted: String, contributions: Vec<HtmlTextContribution>) -> Self {
        Self {
            interpreted,
            contributions,
        }
    }

    pub(crate) fn interpreted(&self) -> &str {
        &self.interpreted
    }

    /// The exact ordered non-empty authored character contributions, in
    /// committed contribution order, which is also strictly increasing source
    /// order for TC-S1.
    pub(crate) fn contributions(&self) -> &[HtmlTextContribution] {
        &self.contributions
    }

    pub(super) fn append(&mut self, contribution: HtmlTextContribution) {
        self.interpreted.push_str(contribution.interpreted());
        self.contributions.push(contribution);
    }
}

impl fmt::Debug for HtmlTextNode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlTextNode")
            .field("interpreted_byte_len", &self.interpreted.len())
            .field("contribution_count", &self.contributions.len())
            .finish()
    }
}

/// What kind of observation a constructed node is.
#[derive(Debug, Clone)]
pub(crate) enum HtmlTreeNodeKind {
    /// The document root. It has no authored source and no synthesis cause:
    /// the root is the parse result's container, not an implied element.
    Document,
    Element(HtmlElement),
    Text(HtmlTextNode),
}

/// Exact authored source evidence for a node that has any.
///
/// `None` from [`HtmlTreeNode::authored_source`] is explicit semantic
/// absence, never an empty or sentinel range.
#[derive(Debug, Clone, Copy)]
pub(crate) enum HtmlAuthoredSource<'node> {
    StartTag {
        complete: &'node SourceAnchor,
        raw_name: &'node SourceAnchor,
    },
    Characters(&'node [HtmlTextContribution]),
}

/// One immutable constructed node.
///
/// Relationships are explicit [`HtmlConstructedNodeId`] values. They are never
/// storage positions, and nothing here may be resolved by indexing storage.
#[derive(Debug, Clone)]
pub(crate) struct HtmlTreeNode {
    id: HtmlConstructedNodeId,
    parent: Option<HtmlConstructedNodeId>,
    children: Vec<HtmlConstructedNodeId>,
    kind: HtmlTreeNodeKind,
}

impl HtmlTreeNode {
    pub(super) fn new(
        id: HtmlConstructedNodeId,
        parent: Option<HtmlConstructedNodeId>,
        children: Vec<HtmlConstructedNodeId>,
        kind: HtmlTreeNodeKind,
    ) -> Self {
        Self {
            id,
            parent,
            children,
            kind,
        }
    }

    pub(crate) fn id(&self) -> HtmlConstructedNodeId {
        self.id
    }

    pub(crate) fn parent(&self) -> Option<HtmlConstructedNodeId> {
        self.parent
    }

    /// Final constructed child order. This is result meaning, independent of
    /// source order and of private storage order.
    pub(crate) fn children(&self) -> &[HtmlConstructedNodeId] {
        &self.children
    }

    pub(crate) fn kind(&self) -> &HtmlTreeNodeKind {
        &self.kind
    }

    /// The node's exact authored source evidence, or `None` when the node has
    /// none. The document root, every synthesized shell element, and every
    /// unmatched-end-synthesized Paragraph return `None`.
    pub(crate) fn authored_source(&self) -> Option<HtmlAuthoredSource<'_>> {
        match &self.kind {
            HtmlTreeNodeKind::Document => None,
            HtmlTreeNodeKind::Element(HtmlElement::Shell(shell)) => match shell.origin() {
                HtmlShellElementOrigin::Authored { complete, raw_name } => {
                    Some(HtmlAuthoredSource::StartTag { complete, raw_name })
                }
                HtmlShellElementOrigin::Synthesized(_) => None,
            },
            // A selected ordinary element is authored-only: its origin is its
            // own exact start tag, never the matching end tag that closed it.
            HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) => {
                Some(HtmlAuthoredSource::StartTag {
                    complete: selected.complete(),
                    raw_name: selected.raw_name(),
                })
            }
            HtmlTreeNodeKind::Element(HtmlElement::Paragraph(paragraph)) => {
                match paragraph.origin() {
                    HtmlParagraphElementOrigin::Authored { complete, raw_name } => {
                        Some(HtmlAuthoredSource::StartTag { complete, raw_name })
                    }
                    HtmlParagraphElementOrigin::Synthesized(_) => None,
                }
            }
            HtmlTreeNodeKind::Text(text) => {
                Some(HtmlAuthoredSource::Characters(text.contributions()))
            }
        }
    }

    pub(super) fn push_child(&mut self, child: HtmlConstructedNodeId) {
        self.children.push(child);
    }

    pub(super) fn text_mut(&mut self) -> Option<&mut HtmlTextNode> {
        match &mut self.kind {
            HtmlTreeNodeKind::Text(text) => Some(text),
            HtmlTreeNodeKind::Document | HtmlTreeNodeKind::Element(_) => None,
        }
    }
}

/// Which emitted token caused a recorded action, diagnostic, or unsupported
/// stop.
///
/// Trigger evidence is deliberately distinct from authored origin. A trigger
/// token is never presented as the authored source of structure it merely
/// made necessary.
#[derive(Clone)]
pub(crate) struct HtmlTreeTokenTrigger {
    token_index: usize,
    kind: HtmlTreeTriggerKind,
}

impl HtmlTreeTokenTrigger {
    pub(super) fn authored(token_index: usize, boundary: SourceAnchor) -> Self {
        Self {
            token_index,
            kind: HtmlTreeTriggerKind::Authored(boundary),
        }
    }

    pub(super) fn end_of_file(token_index: usize) -> Self {
        Self {
            token_index,
            kind: HtmlTreeTriggerKind::EndOfFile,
        }
    }

    /// The trigger token's index in the retained tokenizer run. This is
    /// lower-layer traceability, never constructed-node identity.
    pub(crate) fn token_index(&self) -> usize {
        self.token_index
    }

    pub(crate) fn kind(&self) -> &HtmlTreeTriggerKind {
        &self.kind
    }

    /// The trigger token's complete authored boundary when it has one.
    pub(crate) fn authored_boundary(&self) -> Option<&SourceAnchor> {
        match &self.kind {
            HtmlTreeTriggerKind::Authored(anchor) => Some(anchor),
            HtmlTreeTriggerKind::EndOfFile => None,
        }
    }
}

impl fmt::Debug for HtmlTreeTokenTrigger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlTreeTokenTrigger")
            .field("token_index", &self.token_index)
            .field("kind", &self.kind)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum HtmlTreeTriggerKind {
    /// The trigger token's own complete authored boundary.
    Authored(SourceAnchor),
    /// The retained run's end-of-file token, which has no authored extent.
    /// No empty or dummy anchor stands in for it.
    EndOfFile,
}

/// How a supported shell element left the open-element state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlShellClosure {
    /// The trigger token is the element's own authored end tag.
    AuthoredEndTag,
    /// The trigger token required the element to be closed first.
    ImpliedByToken,
}

/// How a Paragraph left the open-element state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlParagraphClosure {
    /// Its own authored `</p>` closed the current P.
    MatchingEndTag,
    /// A following `<p>`, `<div>`, or `<section>` start closed the current P
    /// before the new element was inserted.
    StartTriggered,
    /// An unmatched authored `</p>` synthesized a source-less P and that same
    /// end tag immediately closed it.
    UnmatchedEndTagSynthesized,
}

/// One committed action, with the token that triggered it.
#[derive(Debug, Clone)]
pub(crate) struct HtmlTreeAction {
    kind: HtmlTreeActionKind,
    trigger: HtmlTreeTokenTrigger,
}

impl HtmlTreeAction {
    pub(super) fn new(kind: HtmlTreeActionKind, trigger: HtmlTreeTokenTrigger) -> Self {
        Self { kind, trigger }
    }

    pub(crate) fn kind(&self) -> &HtmlTreeActionKind {
        &self.kind
    }

    pub(crate) fn trigger(&self) -> &HtmlTreeTokenTrigger {
        &self.trigger
    }
}

/// The selective committed-action vocabulary this subsystem proves.
///
/// This is deliberately not a complete construction event log: it records only
/// what a supported query needs in order to explain a durable observation.
/// TC-S3 extended it with selected ordinary insertion/closure, TC-S4 with one
/// narrow recovery relation, and TC-S5 with three Paragraph lifecycle
/// relations. TC-S7 deliberately reuses the existing Body acknowledgement
/// rather than adding another action, so this remains a selective vocabulary
/// rather than a generic DOM mutation log.
#[derive(Debug, Clone)]
pub(crate) enum HtmlTreeActionKind {
    /// A shell element node was created from the trigger token's own authored
    /// start tag. Here, and only here, the trigger is also the node's
    /// authored origin.
    InsertedAuthoredShellElement {
        node: HtmlConstructedNodeId,
        name: HtmlShellElementName,
    },
    /// A shell element node with no authored source was created because the
    /// trigger token required implied structure. The trigger is *not* the
    /// node's authored origin.
    InsertedSynthesizedShellElement {
        node: HtmlConstructedNodeId,
        name: HtmlShellElementName,
        cause: HtmlSynthesisCause,
    },
    /// A text node was created for the trigger token's characters.
    InsertedTextNode { node: HtmlConstructedNodeId },
    /// The trigger token's characters were appended to the adjacent text node
    /// that already exists at the insertion position.
    AppendedToTextNode { node: HtmlConstructedNodeId },
    /// A selected ordinary element node was created from the trigger token's
    /// own authored start tag. Here, as for an authored shell element, the
    /// trigger is also the node's authored origin.
    InsertedAuthoredSelectedOrdinaryElement {
        node: HtmlConstructedNodeId,
        name: HtmlSelectedOrdinaryElementName,
    },
    /// An open selected ordinary element was closed by its own exact authored
    /// end tag, which this action's trigger retains. No node was created, and
    /// the end tag is closure evidence only: it is never the closed node's
    /// authored origin.
    ClosedSelectedOrdinaryElement {
        node: HtmlConstructedNodeId,
        name: HtmlSelectedOrdinaryElementName,
    },
    /// An open selected ordinary element was popped, without being closed by
    /// an end tag of its own name, because the trigger token is the exact
    /// authored end tag of a *different* selected ordinary element further
    /// out on the open-element stack.
    ///
    /// This is deliberately **not** [`Self::ClosedSelectedOrdinaryElement`]:
    /// no matching end tag caused this pop, so recording a closure for it
    /// would fabricate authored evidence. A later authored end tag of the
    /// popped element's own name is therefore unmatched if it appears after
    /// this recovery. `node` is the intervening element actually popped and
    /// `target` is the nearest same-name selected ordinary element whose end
    /// tag caused the pop. Both are semantic creation-event identities, never
    /// storage positions. One authored target end tag may legitimately carry
    /// several ordered recovery relations plus exactly one target closure,
    /// while remaining the authored origin of none of those nodes. No new
    /// constructed identity is admitted by this relation.
    PoppedSelectedOrdinaryElementByAncestorEndTag {
        node: HtmlConstructedNodeId,
        target: HtmlConstructedNodeId,
    },
    /// A selected ordinary end tag with no matching open element was ignored.
    /// No node was created, closed, or otherwise mutated, and no constructed
    /// identity was admitted. The accompanying parse diagnostic is separate
    /// evidence with its own meaning.
    IgnoredUnmatchedSelectedOrdinaryEndTag {
        name: HtmlSelectedOrdinaryElementName,
    },
    /// An authored `<p>` created a Paragraph. The trigger is the node's own
    /// authored start origin.
    InsertedAuthoredParagraphElement { node: HtmlConstructedNodeId },
    /// An unmatched authored `</p>` created a source-less Paragraph. The
    /// trigger is causal evidence only and is never the node's authored origin.
    InsertedSynthesizedParagraphElement {
        node: HtmlConstructedNodeId,
        cause: HtmlParagraphSynthesisCause,
    },
    /// An open Paragraph was removed for exactly one validated TC-S5 reason.
    ClosedParagraphElement {
        node: HtmlConstructedNodeId,
        closure: HtmlParagraphClosure,
    },
    /// The current Paragraph was implied-popped because the trigger is the
    /// exact authored end tag of the nearest same-name selected ordinary
    /// target. This is TC-S6 implied-end evidence, not a Paragraph closure and
    /// not TC-S4 selected-ordinary recovery. The target end tag is causal
    /// evidence only and is never the Paragraph's authored origin. No
    /// constructed identity is admitted.
    PoppedParagraphElementBySelectedOrdinaryEndTag {
        node: HtmlConstructedNodeId,
        target: HtmlConstructedNodeId,
    },
    /// An open shell element was closed. No node was created.
    ClosedShellElement {
        node: HtmlConstructedNodeId,
        name: HtmlShellElementName,
        closure: HtmlShellClosure,
    },
    /// A supported shell end tag moved the document position without
    /// creating, closing, or mutating any node.
    AcknowledgedShellEndTag { name: HtmlShellElementName },
    /// A duplicate shell start tag created no node and admitted no identity.
    DuplicateShellStartTagCreatedNoNode { name: HtmlShellElementName },
    /// The trigger token was handed to a different actual insertion mode
    /// without being consumed. TC-S2's accepted `AfterBody -> InBody`
    /// recovery makes this a same-token move to a mode that is not
    /// necessarily later; reprocessing still keeps one emitted token as one
    /// semantic observation.
    ReprocessedToken,
    /// Document parsing stopped at the trigger token.
    StoppedParsing,
}

impl HtmlTreeActionKind {
    /// The constructed node this action concerns, when it concerns one.
    pub(crate) fn subject(&self) -> Option<HtmlConstructedNodeId> {
        match self {
            Self::InsertedAuthoredShellElement { node, .. }
            | Self::InsertedSynthesizedShellElement { node, .. }
            | Self::InsertedTextNode { node }
            | Self::AppendedToTextNode { node }
            | Self::InsertedAuthoredSelectedOrdinaryElement { node, .. }
            | Self::ClosedSelectedOrdinaryElement { node, .. }
            | Self::PoppedSelectedOrdinaryElementByAncestorEndTag { node, .. }
            | Self::InsertedAuthoredParagraphElement { node }
            | Self::InsertedSynthesizedParagraphElement { node, .. }
            | Self::ClosedParagraphElement { node, .. }
            | Self::PoppedParagraphElementBySelectedOrdinaryEndTag { node, .. }
            | Self::ClosedShellElement { node, .. } => Some(*node),
            Self::IgnoredUnmatchedSelectedOrdinaryEndTag { .. }
            | Self::AcknowledgedShellEndTag { .. }
            | Self::DuplicateShellStartTagCreatedNoNode { .. }
            | Self::ReprocessedToken
            | Self::StoppedParsing => None,
        }
    }
}

/// A supported parse diagnostic.
///
/// Tree diagnostics are authored-input evidence and are orthogonal to
/// effective completion: a supported run may be `Complete` while carrying
/// parse diagnostics and recovery evidence. They are also independent of the
/// retained tokenizer run's diagnostics, which remain authoritative in the
/// tokenizer result and are never copied into this vocabulary.
#[derive(Debug, Clone)]
pub(crate) struct HtmlTreeDiagnostic {
    code: HtmlTreeDiagnosticCode,
    trigger: HtmlTreeTokenTrigger,
    recovery: HtmlTreeRecovery,
}

impl HtmlTreeDiagnostic {
    pub(super) fn new(
        code: HtmlTreeDiagnosticCode,
        trigger: HtmlTreeTokenTrigger,
        recovery: HtmlTreeRecovery,
    ) -> Self {
        Self {
            code,
            trigger,
            recovery,
        }
    }

    pub(crate) fn code(&self) -> HtmlTreeDiagnosticCode {
        self.code
    }

    pub(crate) fn trigger(&self) -> &HtmlTreeTokenTrigger {
        &self.trigger
    }

    pub(crate) fn recovery(&self) -> HtmlTreeRecovery {
        self.recovery
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeDiagnosticCode {
    /// Ordinary document parsing reached content with no DOCTYPE.
    MissingDoctype,
    /// A `head` start tag appeared while the head insertion mode was current.
    DuplicateHeadStartTag,
    /// A `body` start tag appeared while a body element was already open.
    DuplicateBodyStartTag,
    /// A non-whitespace character run appeared while `AfterBody` was the
    /// actual insertion mode.
    AfterBodyCharacterData,
    /// An authored plain `</body>` switched the actual insertion mode from
    /// `InBody` to `AfterBody` while one or more selected ordinary `Div |
    /// Section` elements remained open. Exactly one is recorded per such
    /// body-end token, independent of selected depth; P alone does not cause
    /// it.
    BodyEndTagWithOpenSelectedOrdinaryElements,
    /// A selected ordinary end tag appeared with no matching selected ordinary
    /// element in the bounded selected-element scope.
    UnmatchedSelectedOrdinaryEndTag,
    /// A selected ordinary end tag appeared while its nearest same-name
    /// selected ordinary element was open but was not the current node, so
    /// one or more differently-nested selected ordinary elements had to be
    /// popped before it could be closed.
    ///
    /// Exactly one of these is recorded per misnested end tag, however many
    /// intervening elements the recovery popped. Which elements were popped,
    /// and which target they were popped for, is recorded once each as
    /// [`HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag`]
    /// rather than duplicated here.
    MisnestedSelectedOrdinaryEndTag,
    /// Document parsing reached end of file while a selected ordinary element
    /// was still open.
    OpenSelectedOrdinaryElementAtEndOfFile,
    /// An authored `</p>` appeared while no P was present in the bounded
    /// TC-S5 button-scope reduction.
    UnmatchedParagraphEndTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeRecovery {
    /// Construction continued; the private document mode became quirks.
    ContinuedInQuirksDocumentMode,
    /// The duplicate shell start tag produced no node and no identity.
    DuplicateShellStartTagProducedNoNode,
    /// The actual insertion mode changed from `AfterBody` to `InBody` and the
    /// same admitted token was reprocessed there.
    SwitchedToInBodyAndReprocessedSameToken,
    /// The authored plain `</body>` moved the actual insertion mode to
    /// `AfterBody` while preserving the complete bounded open stack and every
    /// constructed identity.
    SwitchedToAfterBodyPreservingOpenElements,
    /// The token was ignored. The constructed tree, the open elements, the
    /// actual insertion mode, constructed identity, and closure evidence were
    /// all left exactly as they were.
    IgnoredToken,
    /// The intervening open selected ordinary elements were popped in
    /// current-first order and the nearest same-name target was then closed
    /// by its own exact authored end tag. No end tag was synthesized for a
    /// popped element and no node was created.
    PoppedInterveningSelectedOrdinaryElementsAndClosedTarget,
    /// Document parsing stopped normally with the selected ordinary element
    /// still open. Nothing was popped, synthesized, or closed, and no closure
    /// evidence was fabricated for the end-of-file token.
    StoppedParsingWithOpenSelectedOrdinaryElements,
    /// The unmatched-P recovery inserted one source-less Paragraph and closed
    /// that exact node under the same authored `</p>` trigger.
    SynthesizedParagraphElementAndClosedIt,
}

/// A capability boundary reached by admitted input.
///
/// Unsupported coverage is *not* evidence that the authored source is invalid
/// HTML, and it is not a tokenizer condition. It says only that this
/// tree-construction subsystem has reached a rule outside its proved action
/// set. Variants are durable semantic evidence: a successor adds a new variant
/// for a new boundary rather than silently widening or renaming predecessor
/// meanings. TC-S5 therefore adds Paragraph-specific variants while keeping
/// shell and selected-ordinary variants exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeCapability {
    /// A tag whose interpreted name belongs to none of the three closed
    /// admitted domains: shell, selected ordinary, or Paragraph.
    ///
    /// This is frozen predecessor meaning and keeps it exactly: selected
    /// ordinary and Paragraph tags never reach it, because those domains own
    /// their own shape and placement capabilities rather than widening this
    /// generic unproved-name boundary.
    NonShellElementTag,
    /// Attribute evidence on a shell tag. TC-S1 proves no attribute
    /// semantics, including attribute merging on duplicate shell tags.
    ShellTagAttribute,
    /// A self-closing solidus on a shell tag.
    SelfClosingShellTag,
    /// Character data whose supported handling would depend on the
    /// whitespace/non-whitespace distinction the current document position
    /// makes. TC-S1 proves no whitespace-sensitive character handling.
    WhitespaceSensitiveCharacterData,
    /// Character data reached in a document position TC-S1 does not prove.
    UnprovedCharacterDataPosition,
    /// A shell start tag reached in a document position TC-S1 does not prove.
    UnprovedShellStartTagPosition,
    /// A shell end tag reached in a document position TC-S1 does not prove.
    UnprovedShellEndTagPosition,
    /// A selected ordinary tag reached an actual insertion mode other than
    /// `in body`. The accepted TC-S3/TC-S4 selected `Div | Section` rules are
    /// proved only there, so the tag is refused before any shell walk, recovery,
    /// missing-DOCTYPE, mode, action, coverage, or identity effect.
    SelectedOrdinaryTagOutsideInBody,
    /// A shell tag reached `in body` while a selected ordinary element was
    /// still open. TC-S3 proves no shell interaction over an open selected
    /// ordinary element, so the tag is refused before any partial mutation.
    ShellTagWithOpenSelectedOrdinaryElement,
    /// Attribute evidence on a selected ordinary tag. TC-S3 proves no
    /// attribute semantics for the selected ordinary domain, and deliberately
    /// does not report the shell-specific
    /// [`Self::ShellTagAttribute`], which would be false about a `div`.
    SelectedOrdinaryTagAttribute,
    /// A self-closing solidus on a selected ordinary tag. Kept distinct from
    /// [`Self::SelfClosingShellTag`] for the same reason.
    SelfClosingSelectedOrdinaryTag,
    /// A Paragraph tag reached an actual insertion mode other than `in body`.
    ParagraphTagOutsideInBody,
    /// Attribute evidence on a Paragraph tag is outside TC-S5's plain-P shape.
    ParagraphTagAttribute,
    /// A self-closing solidus on a Paragraph tag is outside TC-S5.
    SelfClosingParagraphTag,
    /// A shell tag was reached while P is current; shell/P crossing is outside
    /// the bounded TC-S5 theorem and is refused before any partial shell effect.
    ShellTagWithOpenParagraphElement,
}

/// The exact typed evidence for an unsupported stop.
#[derive(Debug, Clone)]
pub(crate) struct HtmlTreeUnsupportedCapability {
    capability: HtmlTreeCapability,
    trigger: HtmlTreeTokenTrigger,
}

impl HtmlTreeUnsupportedCapability {
    pub(super) fn new(capability: HtmlTreeCapability, trigger: HtmlTreeTokenTrigger) -> Self {
        Self {
            capability,
            trigger,
        }
    }

    pub(crate) fn capability(&self) -> HtmlTreeCapability {
        self.capability
    }

    /// The exact token that stopped construction. This is trigger evidence
    /// only: the refused action committed no mutation, so nothing in the
    /// frozen tree originates from it.
    pub(crate) fn trigger(&self) -> &HtmlTreeTokenTrigger {
        &self.trigger
    }
}

/// Why an effective tree-construction result is not `Complete`.
#[derive(Debug, Clone)]
pub(crate) enum HtmlTreeIncompleteCause {
    /// Tree construction processed every emitted token it was given, but the
    /// retained tokenizer run is itself incomplete.
    ///
    /// The exact lower-layer meaning — `UnsupportedCapability`,
    /// `ResourceLimit`, `InvalidConfiguration`, or
    /// `InternalInvariantFailure` — remains authoritative on
    /// [`HtmlDocumentShellAnalysis::tokenizer_run`] and is deliberately not
    /// duplicated, re-encoded, or lossily summarized here.
    LowerLayerIncomplete,
    /// Tree construction stopped before mutation at input outside its proved envelope.
    ///
    /// The retained tokenizer run's own completion remains separately
    /// authoritative and may additionally be incomplete.
    UnsupportedCapability(HtmlTreeUnsupportedCapability),
}

/// Effective tree-construction completion.
///
/// `Complete` requires all three of: tokenizer completion `Complete`, every
/// emitted token processed through end of file by supported actions, and a
/// successful freeze. Lower-layer incompleteness is never upgraded.
#[derive(Debug, Clone)]
pub(crate) enum HtmlTreeCompletion {
    Complete,
    Incomplete(HtmlTreeIncompleteCause),
}

impl HtmlTreeCompletion {
    pub(crate) const fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }
}

/// How far committed tree construction actually got.
///
/// Byte coverage alone is not treated as sufficient progress evidence: the
/// processed-token count is recorded explicitly beside it. Committed tree
/// coverage is a different measurement from the retained tokenizer run's own
/// coverage and the two must not be conflated. Tree construction may commit
/// strictly less than the tokenizer processed.
#[derive(Clone)]
pub(crate) struct HtmlTreeCommittedCoverage {
    committed_prefix: SourceAnchor,
    processed_tokens: usize,
}

impl HtmlTreeCommittedCoverage {
    /// The retained-source prefix whose emitted tokens were completely
    /// processed by committed tree-construction actions.
    pub(crate) fn committed_prefix(&self) -> &SourceAnchor {
        &self.committed_prefix
    }

    pub(crate) fn committed_end(&self) -> usize {
        self.committed_prefix.range().end()
    }

    /// How many emitted tokens of the retained run were completely processed.
    /// This is parser progress, not byte progress.
    pub(crate) fn processed_tokens(&self) -> usize {
        self.processed_tokens
    }
}

impl fmt::Debug for HtmlTreeCommittedCoverage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlTreeCommittedCoverage")
            .field("source_id", &self.committed_prefix.source_id())
            .field("committed_prefix_range", &self.committed_prefix.range())
            .field("processed_tokens", &self.processed_tokens)
            .finish()
    }
}

/// The immutable, validated HTML tree-construction analysis for the currently
/// accepted bounded production frontier.
///
/// Retains the validated [`HtmlTokenizerRunResult`] by value so tokenizer
/// tokens, diagnostics, coverage, completion, limits, and usage remain
/// authoritative in one place rather than being duplicated.
#[derive(Clone)]
pub(crate) struct HtmlDocumentShellAnalysis {
    tokenizer_run: HtmlTokenizerRunResult,
    nodes: Vec<HtmlTreeNode>,
    root: HtmlConstructedNodeId,
    diagnostics: Vec<HtmlTreeDiagnostic>,
    actions: Vec<HtmlTreeAction>,
    coverage: HtmlTreeCommittedCoverage,
    completion: HtmlTreeCompletion,
}

impl HtmlDocumentShellAnalysis {
    /// The retained validated tokenizer run. Tokenizer completion,
    /// diagnostics, coverage, and resource evidence remain authoritative
    /// here.
    pub(crate) fn tokenizer_run(&self) -> &HtmlTokenizerRunResult {
        &self.tokenizer_run
    }

    pub(crate) fn root(&self) -> HtmlConstructedNodeId {
        self.root
    }

    /// Resolves a constructed identity.
    ///
    /// Deliberately a search over stored nodes for a matching identity, never
    /// an index into storage: identity is committed creation-event order, not
    /// a storage slot, so replacing or permuting private storage must not
    /// change any answer.
    pub(crate) fn node(&self, id: HtmlConstructedNodeId) -> Option<&HtmlTreeNode> {
        self.nodes.iter().find(|node| node.id() == id)
    }

    pub(crate) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// All constructed nodes in committed semantic creation order.
    pub(crate) fn nodes_in_creation_order(&self) -> Vec<&HtmlTreeNode> {
        let mut ordered: Vec<&HtmlTreeNode> = self.nodes.iter().collect();
        ordered.sort_by_key(|node| node.id());
        ordered
    }

    /// Supported tree-construction parse diagnostics, in committed order.
    pub(crate) fn diagnostics(&self) -> &[HtmlTreeDiagnostic] {
        &self.diagnostics
    }

    /// Committed action and disposition evidence, in committed order.
    pub(crate) fn actions(&self) -> &[HtmlTreeAction] {
        &self.actions
    }

    pub(crate) fn coverage(&self) -> &HtmlTreeCommittedCoverage {
        &self.coverage
    }

    pub(crate) fn completion(&self) -> &HtmlTreeCompletion {
        &self.completion
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.completion.is_complete()
    }

    /// Test-only storage perturbation.
    ///
    /// Reverses private node storage without touching any identity or
    /// relationship, so tests can prove that durable meaning survives storage
    /// replacement and that nothing resolves relationships by storage
    /// position.
    #[cfg(test)]
    pub(super) fn with_reversed_storage(mut self) -> Self {
        self.nodes.reverse();
        self
    }
}

impl fmt::Debug for HtmlDocumentShellAnalysis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlDocumentShellAnalysis")
            .field("tokenizer_run", &self.tokenizer_run)
            .field("node_count", &self.nodes.len())
            .field("root", &self.root)
            .field("diagnostic_count", &self.diagnostics.len())
            .field("action_count", &self.actions.len())
            .field("coverage", &self.coverage)
            .field("completion", &self.completion)
            .finish()
    }
}

/// The private session's finalization hand-off.
///
/// This is construction output on its way through the freeze boundary, not an
/// Analysis Result: it never reaches a consumer, and no mutable session state
/// travels inside it.
pub(super) struct HtmlDocumentShellParts {
    pub(super) nodes: Vec<HtmlTreeNode>,
    pub(super) root: HtmlConstructedNodeId,
    pub(super) admitted_creation_events: u32,
    pub(super) diagnostics: Vec<HtmlTreeDiagnostic>,
    pub(super) actions: Vec<HtmlTreeAction>,
    pub(super) processed_tokens: usize,
    pub(super) committed_prefix_end: usize,
    pub(super) completion: HtmlTreeCompletion,
    /// The semantic identities of the selected ordinary elements that were
    /// still open on the private session's own open-element stack when the run
    /// finished, innermost last.
    ///
    /// This is an immutable snapshot taken at hand-off, not the mutable stack:
    /// it exists so [`freeze`] can check the committed action stream against
    /// the state it actually describes, and it is consumed and discarded here.
    /// It never reaches [`HtmlDocumentShellAnalysis`] and no consumer can
    /// observe it or the parser stack behind it.
    pub(super) final_open_selected_ordinary: Vec<HtmlConstructedNodeId>,
    /// The P still open on the private stack at hand-off, if any. Under the
    /// accepted TC-S5 theorem there can be at most one and it is current.
    /// This immutable checkpoint is consumed by freeze and never escapes.
    pub(super) final_open_paragraph: Option<HtmlConstructedNodeId>,
}

/// Which retained evidence a source-validation freeze error concerns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeEvidenceRole {
    AuthoredCompleteTag,
    AuthoredRawName,
    TextContribution,
    ActionTrigger,
    DiagnosticTrigger,
    UnsupportedTrigger,
    CommittedCoverage,
}

/// Why an effective `Complete` claim was rejected at freeze.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeCompletionUpgrade {
    /// The retained tokenizer run is not `Complete`.
    RetainedTokenizerRunIsIncomplete,
    /// Emitted tokens remain unprocessed.
    EmittedTokensRemainUnprocessed,
    /// The committed tree is not a complete document shell.
    DocumentShellIsIncomplete,
}

/// A freeze/boundary invariant failure.
///
/// This vocabulary is deliberately separate from HTML parse diagnostics and
/// from [`HtmlTreeCapability`]: a freeze failure means the construction
/// boundary produced something it must never publish, not that authored HTML
/// is invalid or that capability coverage is missing. Existing TC-S1–TC-S4
/// variants retain their exact meaning; TC-S5 adds Paragraph-specific failures
/// and TC-S7 adds only bounded body-end replay failures. Every variant carries
/// structural/provenance evidence only;
/// `Debug` and `Display` never expose arbitrary authored source content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlTreeFreezeError {
    /// Two stored nodes carry the same constructed identity.
    DuplicateConstructedIdentity(HtmlConstructedNodeId),
    CreationEventInventoryMismatch {
        admitted: usize,
        stored: usize,
    },
    UnadmittedConstructedIdentity(HtmlConstructedNodeId),
    /// The declared root is not stored.
    MissingRootNode(HtmlConstructedNodeId),
    /// The declared root is not a Document node, or a Document node is stored
    /// somewhere other than the root.
    InvalidDocumentRoot(HtmlConstructedNodeId),
    /// The root records a parent relationship.
    RootMustNotHaveParent(HtmlConstructedNodeId),
    /// A non-root node records no parent relationship.
    MissingParentRelationship(HtmlConstructedNodeId),
    /// A relationship names an identity that is not stored.
    UnresolvedRelationship {
        from: HtmlConstructedNodeId,
        to: HtmlConstructedNodeId,
    },
    /// A parent/child relationship is not recorded mutually and exactly once
    /// on both sides.
    AsymmetricRelationship {
        parent: HtmlConstructedNodeId,
        child: HtmlConstructedNodeId,
    },
    /// A child's creation event precedes its parent's.
    ChildPrecedesParentCreation {
        parent: HtmlConstructedNodeId,
        child: HtmlConstructedNodeId,
    },
    UnreachableOrCyclicStructure {
        reachable: usize,
        stored: usize,
    },
    InvalidTextContributions(HtmlConstructedNodeId),
    /// Retained evidence is bound to a source identity other than the exact
    /// supplied [`SourceText`].
    ForeignSourceEvidence {
        role: HtmlTreeEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    /// Retained evidence did not revalidate through the exact supplied
    /// [`SourceText`].
    InvalidSourceEvidence {
        role: HtmlTreeEvidenceRole,
        error: SourceRangeError,
    },
    MismatchedSourceEvidence {
        role: HtmlTreeEvidenceRole,
    },
    AuthoredNameOutsideCompleteTag(HtmlConstructedNodeId),
    /// An action names a constructed identity that is not stored.
    UnresolvedActionSubject(HtmlConstructedNodeId),
    /// A recorded trigger names a token outside the retained run, or recorded
    /// evidence is not in non-decreasing token order.
    InvalidTokenProgression {
        role: HtmlTreeEvidenceRole,
        token_index: usize,
    },
    /// Committed tree coverage is not a valid prefix of the retained source.
    InvalidCommittedCoverage {
        committed_prefix_end: usize,
        source_len: usize,
    },
    /// Committed tree coverage claims more processed tokens than the retained
    /// run emitted.
    CommittedTokensExceedRetainedRun {
        processed_tokens: usize,
        emitted_tokens: usize,
    },
    /// Effective `Complete` was claimed without the conditions that permit
    /// it.
    CompletionUpgrade(HtmlTreeCompletionUpgrade),
    /// A node's authored origin is the same authored range as the unsupported
    /// trigger, which would leak identity for input that committed nothing.
    UnsupportedTriggerLeakedAsAuthoredOrigin(HtmlConstructedNodeId),
    /// A recorded selected ordinary closure names a node that is not a
    /// selected ordinary element of the recorded name.
    ClosureSubjectIsNotTheSelectedOrdinaryElement {
        node: HtmlConstructedNodeId,
        name: HtmlSelectedOrdinaryElementName,
    },
    /// A selected ordinary closure was recorded without the exact authored
    /// end-tag trigger that is its only permitted evidence. End of file
    /// fabricates no closure.
    FabricatedSelectedOrdinaryClosure(HtmlConstructedNodeId),
    /// A selected ordinary closure was recorded for an element that was not
    /// the innermost open selected ordinary element at that point, or that
    /// was never inserted or was already closed. Closure is unique and
    /// stack-consistent for the selected slice.
    NonLifoSelectedOrdinaryClosure(HtmlConstructedNodeId),
    /// A selected ordinary closure trigger does not resolve, in the retained
    /// tokenizer run, to the exact emitted matching end tag for that element.
    ///
    /// This is what makes closure evidence *matching* end-tag evidence rather
    /// than merely some valid authored anchor: a start tag, an unrelated
    /// authored token, a differently-named end tag, or an anchor that is not
    /// the retained token's own complete-tag evidence all land here.
    ClosureTriggerIsNotTheMatchingEndTag {
        node: HtmlConstructedNodeId,
        token_index: usize,
    },
    /// The same selected ordinary semantic identity was inserted more than
    /// once by the committed action stream.
    DuplicateSelectedOrdinaryInsertion(HtmlConstructedNodeId),
    /// A recorded final open selected ordinary identity does not resolve to a
    /// stored selected ordinary element.
    FinalOpenSelectedOrdinaryIsNotASelectedElement(HtmlConstructedNodeId),
    /// The selected ordinary elements left open by the committed action stream
    /// are not exactly, and in the same order as, the session's actual final
    /// open selected ordinary elements.
    FinalOpenSelectedOrdinaryStateMismatch {
        replayed: Vec<HtmlConstructedNodeId>,
        actual: Vec<HtmlConstructedNodeId>,
    },
    /// A recorded heterogeneous recovery pop names a subject that is not a
    /// stored selected ordinary element.
    RecoverySubjectIsNotSelectedOrdinaryElement(HtmlConstructedNodeId),
    /// A recorded heterogeneous recovery pop names a target that is not a
    /// stored selected ordinary element.
    RecoveryTargetIsNotSelectedOrdinaryElement(HtmlConstructedNodeId),
    /// A recorded heterogeneous recovery pop names the same identity as both
    /// the popped element and the target it was popped for. The target is
    /// closed by its own matching end tag and is never recovery-popped.
    SelfTargetingSelectedOrdinaryRecovery(HtmlConstructedNodeId),
    /// A recorded heterogeneous recovery pop does not resolve, in the
    /// retained tokenizer run, to the exact emitted matching end tag of its
    /// target. End of file, a start tag, a character run, an unrelated end
    /// tag, a differently-named end tag, and an anchor that is not the
    /// retained token's own complete-tag evidence all land here.
    RecoveryTriggerIsNotMatchingTargetEndTag {
        node: HtmlConstructedNodeId,
        token_index: usize,
    },
    /// A recorded heterogeneous recovery names a target that was not open, or
    /// was not the nearest currently-open selected ordinary element of its own
    /// name, at the point the recovery was committed.
    RecoveryTargetIsNotNearestMatchingSelectedOrdinary(HtmlConstructedNodeId),
    /// A recorded heterogeneous recovery pop names an element that was not the
    /// current selected ordinary element at that point: the suffix was
    /// popped out of order, an intervening element was skipped, the same
    /// element was popped twice, or the element had already left the open
    /// state.
    NonLifoSelectedOrdinaryRecovery(HtmlConstructedNodeId),
    /// A committed recovery group was never terminated by its target's own
    /// matching closure: the action stream ended, or an unrelated action was
    /// committed, while the group was still open.
    UnterminatedSelectedOrdinaryRecovery(HtmlConstructedNodeId),
    /// A committed recovery group is terminated by a closure that is not its
    /// target's, or by a closure whose trigger token is not the recovery
    /// group's own trigger token.
    SelectedOrdinaryRecoveryClosureMismatch {
        target: HtmlConstructedNodeId,
        closed: HtmlConstructedNodeId,
    },
    /// The misnested selected ordinary end-tag diagnostics are not exactly
    /// one per committed recovery group, with the group's own trigger token
    /// and the accepted recovery summary.
    SelectedOrdinaryRecoveryDiagnosticMismatch {
        recovery_groups: Vec<usize>,
        misnested_diagnostics: Vec<usize>,
    },
    /// The ignored unmatched selected ordinary end-tag actions and the
    /// unmatched selected ordinary end-tag diagnostics do not name exactly
    /// the same trigger tokens, or an unmatched diagnostic does not carry
    /// that action's own exact authored end tag and the ignored-token
    /// recovery.
    UnmatchedSelectedOrdinaryEndTagDiagnosticMismatch {
        actions: Vec<usize>,
        diagnostics: Vec<usize>,
    },
    DuplicateSelectedOrdinaryEndTokenDecision {
        token_index: usize,
    },
    UnmatchedSelectedOrdinaryEndTriggerIsNotTheMatchingEndTag {
        token_index: usize,
    },
    UnmatchedSelectedOrdinaryEndTagWithOpenTarget(HtmlConstructedNodeId),
    ParagraphActionSubjectIsNotParagraph(HtmlConstructedNodeId),
    DuplicateParagraphInsertion(HtmlConstructedNodeId),
    ParagraphInsertionInventoryMismatch(HtmlConstructedNodeId),
    ParagraphAuthoredInsertionTriggerMismatch {
        node: HtmlConstructedNodeId,
        token_index: usize,
    },
    ParagraphSynthesizedInsertionTriggerMismatch {
        node: HtmlConstructedNodeId,
        token_index: usize,
    },
    ParagraphClosureTriggerMismatch {
        node: HtmlConstructedNodeId,
        token_index: usize,
    },
    ParagraphImpliedPopTargetIsNotSelectedOrdinaryElement(HtmlConstructedNodeId),
    ParagraphImpliedPopTargetIsNotNearestMatchingSelectedOrdinary(HtmlConstructedNodeId),
    ParagraphImpliedPopTriggerMismatch {
        node: HtmlConstructedNodeId,
        target: HtmlConstructedNodeId,
        token_index: usize,
    },
    ParagraphImpliedPopContinuationMismatch {
        token_index: usize,
    },
    NonLifoParagraphInteraction(HtmlConstructedNodeId),
    ParagraphStartTriggeredInsertionMismatch {
        token_index: usize,
    },
    ParagraphSynthesisClosureMismatch {
        token_index: usize,
    },
    UnmatchedParagraphDiagnosticMismatch {
        syntheses: Vec<usize>,
        diagnostics: Vec<usize>,
    },
    FinalOpenParagraphIsNotParagraph(HtmlConstructedNodeId),
    FinalOpenParagraphStateMismatch {
        replayed: Option<HtmlConstructedNodeId>,
        actual: Option<HtmlConstructedNodeId>,
    },
    /// A body acknowledgement was not triggered by the retained authored
    /// plain `</body>` token it names.
    BodyEndAcknowledgementTriggerMismatch {
        token_index: usize,
    },
    /// One retained `</body>` token was claimed to perform the body-end
    /// transition more than once.
    DuplicateBodyEndAcknowledgement {
        token_index: usize,
    },
    /// A body acknowledgement was recorded while the replayed body position
    /// was not `InBody`.
    BodyEndAcknowledgementOutsideInBody {
        token_index: usize,
    },
    /// The dedicated TC-S7 diagnostic count did not equal the replayed
    /// selected-open condition for this body-end token.
    BodyEndDiagnosticCardinalityMismatch {
        token_index: usize,
        selected_open: usize,
        diagnostics: usize,
    },
    /// The one required TC-S7 diagnostic did not carry the body action's exact
    /// retained trigger and dedicated recovery meaning.
    BodyEndDiagnosticTriggerOrRecoveryMismatch {
        token_index: usize,
    },
    /// A TC-S7 diagnostic exists without a corresponding body acknowledgement.
    OrphanBodyEndDiagnostic {
        token_index: usize,
    },
    /// A close, implied pop, selected recovery, synthesis, or node-creation
    /// action was attributed to the body-end trigger after the bounded
    /// transition was reached.
    BodyEndSameTriggerMutation {
        token_index: usize,
    },
    /// The independent TC-S7 mixed selected/P replay did not preserve the
    /// bounded open-content lifecycle.
    BodyEndOpenContentReplayMismatch {
        token_index: usize,
    },
    /// An AfterBody successor character did not preserve the retained current
    /// insertion parent or the accepted whitespace/non-whitespace transition.
    BodyEndAfterBodySuccessorMismatch {
        token_index: usize,
    },
    /// EOF was reached while the replayed body position remained outside
    /// `InBody`, but the InBody selected-open EOF diagnostic was fabricated.
    BodyEndAfterBodyEofDiagnosticMismatch {
        token_index: usize,
    },
    /// The TC-S7 mixed replay's final selected/P state does not equal the
    /// immutable session hand-off checkpoints.
    BodyEndFinalOpenStateMismatch,
}

impl fmt::Display for HtmlTreeFreezeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HTML document shell freeze invariant violation: {self:?}"
        )
    }
}

impl Error for HtmlTreeFreezeError {}

/// Validates private construction output and freezes it into immutable result
/// meaning. This is the only constructor of [`HtmlDocumentShellAnalysis`]:
/// durable invariants are checked at this ownership boundary rather than
/// trusted because the private session happens to produce them today. Existing
/// TC-S1–TC-S4 checks remain intact; TC-S5 adds an independent Paragraph
/// lifecycle replay beside the selected-ordinary replay, and TC-S7 adds one
/// mixed open-content/body-position replay over both retained lifecycles.
pub(super) fn freeze(
    source: &SourceText,
    tokenizer_run: HtmlTokenizerRunResult,
    parts: HtmlDocumentShellParts,
) -> Result<HtmlDocumentShellAnalysis, HtmlTreeFreezeError> {
    let HtmlDocumentShellParts {
        nodes,
        root,
        admitted_creation_events,
        diagnostics,
        actions,
        processed_tokens,
        committed_prefix_end,
        completion,
        final_open_selected_ordinary,
        final_open_paragraph,
    } = parts;

    validate_identity_inventory(&nodes, admitted_creation_events)?;
    validate_structure(&nodes, root)?;
    validate_node_evidence(source, &nodes)?;
    validate_action_evidence(source, &nodes, &actions, tokenizer_run.tokens().len())?;
    validate_selected_ordinary_lifecycle(
        &nodes,
        &actions,
        &diagnostics,
        &tokenizer_run,
        &final_open_selected_ordinary,
    )?;
    validate_paragraph_lifecycle(
        &nodes,
        &actions,
        &diagnostics,
        &tokenizer_run,
        final_open_paragraph,
    )?;
    validate_body_end_open_stack_transitions(
        &nodes,
        &actions,
        &diagnostics,
        &tokenizer_run,
        &final_open_selected_ordinary,
        final_open_paragraph,
    )?;
    validate_diagnostic_evidence(source, &diagnostics, tokenizer_run.tokens().len())?;
    validate_completion(
        source,
        &nodes,
        root,
        &tokenizer_run,
        processed_tokens,
        &completion,
    )?;

    let source_len = source.as_str().len();
    if committed_prefix_end > source_len {
        return Err(HtmlTreeFreezeError::InvalidCommittedCoverage {
            committed_prefix_end,
            source_len,
        });
    }
    if processed_tokens > tokenizer_run.tokens().len() {
        return Err(HtmlTreeFreezeError::CommittedTokensExceedRetainedRun {
            processed_tokens,
            emitted_tokens: tokenizer_run.tokens().len(),
        });
    }
    let committed_prefix = source.anchor(0, committed_prefix_end).map_err(|error| {
        HtmlTreeFreezeError::InvalidSourceEvidence {
            role: HtmlTreeEvidenceRole::CommittedCoverage,
            error,
        }
    })?;

    Ok(HtmlDocumentShellAnalysis {
        tokenizer_run,
        nodes,
        root,
        diagnostics,
        actions,
        coverage: HtmlTreeCommittedCoverage {
            committed_prefix,
            processed_tokens,
        },
        completion,
    })
}

fn validate_identity_inventory(
    nodes: &[HtmlTreeNode],
    admitted_creation_events: u32,
) -> Result<(), HtmlTreeFreezeError> {
    let admitted = admitted_creation_events as usize;
    if nodes.len() != admitted {
        return Err(HtmlTreeFreezeError::CreationEventInventoryMismatch {
            admitted,
            stored: nodes.len(),
        });
    }
    for (position, node) in nodes.iter().enumerate() {
        if node.id().0 >= admitted_creation_events {
            return Err(HtmlTreeFreezeError::UnadmittedConstructedIdentity(
                node.id(),
            ));
        }
        if nodes[..position]
            .iter()
            .any(|earlier| earlier.id() == node.id())
        {
            return Err(HtmlTreeFreezeError::DuplicateConstructedIdentity(node.id()));
        }
    }
    Ok(())
}

/// Resolves an identity by searching stored nodes. Never indexes storage.
fn find(nodes: &[HtmlTreeNode], id: HtmlConstructedNodeId) -> Option<&HtmlTreeNode> {
    nodes.iter().find(|node| node.id() == id)
}

fn validate_structure(
    nodes: &[HtmlTreeNode],
    root: HtmlConstructedNodeId,
) -> Result<(), HtmlTreeFreezeError> {
    let root_node = find(nodes, root).ok_or(HtmlTreeFreezeError::MissingRootNode(root))?;
    if !matches!(root_node.kind(), HtmlTreeNodeKind::Document) {
        return Err(HtmlTreeFreezeError::InvalidDocumentRoot(root));
    }
    if root_node.parent().is_some() {
        return Err(HtmlTreeFreezeError::RootMustNotHaveParent(root));
    }

    for node in nodes {
        if node.id() != root {
            if matches!(node.kind(), HtmlTreeNodeKind::Document) {
                return Err(HtmlTreeFreezeError::InvalidDocumentRoot(node.id()));
            }
            let parent_id = node
                .parent()
                .ok_or(HtmlTreeFreezeError::MissingParentRelationship(node.id()))?;
            let parent =
                find(nodes, parent_id).ok_or(HtmlTreeFreezeError::UnresolvedRelationship {
                    from: node.id(),
                    to: parent_id,
                })?;
            if parent
                .children()
                .iter()
                .filter(|id| **id == node.id())
                .count()
                != 1
            {
                return Err(HtmlTreeFreezeError::AsymmetricRelationship {
                    parent: parent_id,
                    child: node.id(),
                });
            }
            if parent_id >= node.id() {
                return Err(HtmlTreeFreezeError::ChildPrecedesParentCreation {
                    parent: parent_id,
                    child: node.id(),
                });
            }
        }

        for child_id in node.children() {
            let child =
                find(nodes, *child_id).ok_or(HtmlTreeFreezeError::UnresolvedRelationship {
                    from: node.id(),
                    to: *child_id,
                })?;
            if child.parent() != Some(node.id()) {
                return Err(HtmlTreeFreezeError::AsymmetricRelationship {
                    parent: node.id(),
                    child: *child_id,
                });
            }
        }
    }

    // Reachability doubles as the acyclicity proof: every non-root node has
    // exactly one recorded parent, so a cycle would detach its members from
    // the root and the reachable count would fall short of the stored count.
    let mut reachable = 0usize;
    let mut frontier = vec![root];
    let mut visited: Vec<HtmlConstructedNodeId> = Vec::new();
    while let Some(id) = frontier.pop() {
        if visited.contains(&id) {
            continue;
        }
        visited.push(id);
        reachable += 1;
        let node = find(nodes, id)
            .ok_or(HtmlTreeFreezeError::UnresolvedRelationship { from: root, to: id })?;
        frontier.extend(node.children().iter().copied());
    }
    if reachable != nodes.len() {
        return Err(HtmlTreeFreezeError::UnreachableOrCyclicStructure {
            reachable,
            stored: nodes.len(),
        });
    }
    Ok(())
}

fn validate_node_evidence(
    source: &SourceText,
    nodes: &[HtmlTreeNode],
) -> Result<(), HtmlTreeFreezeError> {
    for node in nodes {
        match node.kind() {
            HtmlTreeNodeKind::Document => {}
            HtmlTreeNodeKind::Element(element) => {
                let authored = match element {
                    HtmlElement::Shell(shell) => match shell.origin() {
                        HtmlShellElementOrigin::Authored { complete, raw_name } => {
                            Some((complete, raw_name))
                        }
                        HtmlShellElementOrigin::Synthesized(_) => None,
                    },
                    HtmlElement::SelectedOrdinary(selected) => {
                        Some((selected.complete(), selected.raw_name()))
                    }
                    HtmlElement::Paragraph(paragraph) => match paragraph.origin() {
                        HtmlParagraphElementOrigin::Authored { complete, raw_name } => {
                            Some((complete, raw_name))
                        }
                        HtmlParagraphElementOrigin::Synthesized(_) => None,
                    },
                };
                if let Some((complete, raw_name)) = authored {
                    validate_evidence(source, HtmlTreeEvidenceRole::AuthoredCompleteTag, complete)?;
                    validate_evidence(source, HtmlTreeEvidenceRole::AuthoredRawName, raw_name)?;
                    if complete.range().start() > raw_name.range().start()
                        || raw_name.range().end() > complete.range().end()
                    {
                        return Err(HtmlTreeFreezeError::AuthoredNameOutsideCompleteTag(
                            node.id(),
                        ));
                    }
                }
            }
            HtmlTreeNodeKind::Text(text) => {
                if text.contributions().is_empty() {
                    return Err(HtmlTreeFreezeError::InvalidTextContributions(node.id()));
                }
                let mut rebuilt = String::new();
                let mut previous_end: Option<usize> = None;
                for contribution in text.contributions() {
                    validate_evidence(
                        source,
                        HtmlTreeEvidenceRole::TextContribution,
                        contribution.source(),
                    )?;
                    if contribution.source().range().is_empty()
                        || contribution.interpreted().is_empty()
                    {
                        return Err(HtmlTreeFreezeError::InvalidTextContributions(node.id()));
                    }
                    if let Some(previous_end) = previous_end
                        && contribution.source().range().start() < previous_end
                    {
                        return Err(HtmlTreeFreezeError::InvalidTextContributions(node.id()));
                    }
                    previous_end = Some(contribution.source().range().end());
                    rebuilt.push_str(contribution.interpreted());
                }
                if rebuilt != text.interpreted() {
                    return Err(HtmlTreeFreezeError::InvalidTextContributions(node.id()));
                }
            }
        }
    }
    Ok(())
}

fn validate_action_evidence(
    source: &SourceText,
    nodes: &[HtmlTreeNode],
    actions: &[HtmlTreeAction],
    emitted_tokens: usize,
) -> Result<(), HtmlTreeFreezeError> {
    let mut previous_index: Option<usize> = None;
    for action in actions {
        validate_trigger(
            source,
            HtmlTreeEvidenceRole::ActionTrigger,
            action.trigger(),
            emitted_tokens,
            &mut previous_index,
        )?;
        if let Some(subject) = action.kind().subject()
            && find(nodes, subject).is_none()
        {
            return Err(HtmlTreeFreezeError::UnresolvedActionSubject(subject));
        }
    }
    Ok(())
}

/// Validates the complete selected-ordinary lifecycle-evidence theorem.
///
/// TC-S5 leaves this relation intact; a separate Paragraph replay below checks
/// the mixed-stack interactions this selected-only projection deliberately does
/// not own. This replay independently proves that:
///
/// - every selected insertion, recovery, and closure endpoint resolves by
///   semantic constructed identity to the expected stored selected element;
/// - each selected identity is inserted at most once, so action evidence cannot
///   pad the replay stack with a repeated node;
/// - matching closures and recovery groups correlate to the retained emitted
///   end tag itself, not merely to any revalidating source anchor;
/// - a heterogeneous recovery target is recomputed as the nearest currently
///   open selected element of its name, and the intervening suffix is popped in
///   strict current-first order before exactly that target is closed;
/// - an intervening recovery-popped element never receives a fabricated matching
///   closure, while the target's authored end tag may legitimately explain
///   several ordered pops plus its one closure;
/// - one retained selected end token spends exactly one terminal selected-end
///   decision: current closure, recovery-target closure, or ignored-unmatched
///   disposition;
/// - misnested and unmatched diagnostics pair one-for-one with the replayed
///   recovery/disposition groups and retain the group's exact trigger evidence;
///   and
/// - the identities still open after replay are exactly, and in the same order
///   as, the session's immutable final-open selected snapshot.
///
/// The final comparison is what makes this validation of construction output
/// rather than trust in session implementation: an unrecorded pop or closure
/// cannot masquerade as a legitimate end-of-file-open state. No source search,
/// rescan, or retokenization participates in any of these checks.
fn validate_selected_ordinary_lifecycle(
    nodes: &[HtmlTreeNode],
    actions: &[HtmlTreeAction],
    diagnostics: &[HtmlTreeDiagnostic],
    tokenizer_run: &HtmlTokenizerRunResult,
    final_open_selected_ordinary: &[HtmlConstructedNodeId],
) -> Result<(), HtmlTreeFreezeError> {
    let mut open: Vec<HtmlConstructedNodeId> = Vec::new();
    let mut inserted: Vec<HtmlConstructedNodeId> = Vec::new();
    // The recovery group currently awaiting its target's own matching
    // closure, as `(target, trigger token index)`. A group is opened by its
    // first recovery pop and must be closed before any other action commits.
    let mut pending: Option<(HtmlConstructedNodeId, usize)> = None;
    let mut recovery_groups: Vec<ReplayedRecoveryGroup> = Vec::new();
    let mut ignored_unmatched: Vec<ReplayedUnmatchedEnd> = Vec::new();
    // Which retained tokens have already spent their one terminal
    // selected-end decision. Recovery pops are not terminal; the closure or
    // disposition that ends the dispatch is.
    let mut spent_end_tokens: Vec<usize> = Vec::new();

    for action in actions {
        match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { node, name } => {
                reject_interleaved_recovery(pending)?;
                expect_selected_ordinary(nodes, *node, *name)?;
                if inserted.contains(node) {
                    return Err(HtmlTreeFreezeError::DuplicateSelectedOrdinaryInsertion(
                        *node,
                    ));
                }
                inserted.push(*node);
                open.push(*node);
            }
            HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { node, target } => {
                if selected_ordinary_name(nodes, *node).is_none() {
                    return Err(
                        HtmlTreeFreezeError::RecoverySubjectIsNotSelectedOrdinaryElement(*node),
                    );
                }
                let Some(target_name) = selected_ordinary_name(nodes, *target) else {
                    return Err(
                        HtmlTreeFreezeError::RecoveryTargetIsNotSelectedOrdinaryElement(*target),
                    );
                };
                if node == target {
                    return Err(HtmlTreeFreezeError::SelfTargetingSelectedOrdinaryRecovery(
                        *node,
                    ));
                }
                if !is_matching_end_tag_trigger(target_name, action.trigger(), tokenizer_run) {
                    return Err(
                        HtmlTreeFreezeError::RecoveryTriggerIsNotMatchingTargetEndTag {
                            node: *node,
                            token_index: action.trigger().token_index(),
                        },
                    );
                }
                match pending {
                    Some((pending_target, pending_token)) => {
                        // A recovery group belongs to exactly one target under
                        // exactly one authored end tag.
                        if pending_target != *target
                            || pending_token != action.trigger().token_index()
                        {
                            return Err(HtmlTreeFreezeError::UnterminatedSelectedOrdinaryRecovery(
                                pending_target,
                            ));
                        }
                    }
                    None => {
                        // Recomputed from the replayed stack, so a recorded
                        // target that merely looks plausible is still rejected.
                        if nearest_open_selected_ordinary(nodes, &open, target_name)
                            != Some(*target)
                        {
                            return Err(
                                HtmlTreeFreezeError::RecoveryTargetIsNotNearestMatchingSelectedOrdinary(
                                    *target,
                                ),
                            );
                        }
                        pending = Some((*target, action.trigger().token_index()));
                    }
                }
                if open.last() != Some(node) {
                    return Err(HtmlTreeFreezeError::NonLifoSelectedOrdinaryRecovery(*node));
                }
                open.pop();
            }
            HtmlTreeActionKind::ClosedSelectedOrdinaryElement { node, name } => {
                expect_selected_ordinary(nodes, *node, *name)?;
                validate_closure_trigger(*node, *name, action.trigger(), tokenizer_run)?;
                if let Some((target, token)) = pending {
                    if target != *node || token != action.trigger().token_index() {
                        return Err(
                            HtmlTreeFreezeError::SelectedOrdinaryRecoveryClosureMismatch {
                                target,
                                closed: *node,
                            },
                        );
                    }
                    recovery_groups.push(ReplayedRecoveryGroup {
                        trigger_token: token,
                        target_name: *name,
                    });
                    pending = None;
                }
                if open.last() != Some(node) {
                    return Err(HtmlTreeFreezeError::NonLifoSelectedOrdinaryClosure(*node));
                }
                open.pop();
                // Checked after the stack-consistency rules above, so a
                // duplicated closure of one element keeps reporting the
                // predecessor non-LIFO meaning it always did.
                spend_end_token(&mut spent_end_tokens, action.trigger().token_index())?;
            }
            HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag { name } => {
                reject_interleaved_recovery(pending)?;
                // Independently proved, not inferred from the recorded token
                // index: the trigger really is the retained emitted end tag of
                // this recorded selected name, and no same-name target was
                // open, so the ignored cell really is the cell that applied.
                if !is_matching_end_tag_trigger(*name, action.trigger(), tokenizer_run) {
                    return Err(
                        HtmlTreeFreezeError::UnmatchedSelectedOrdinaryEndTriggerIsNotTheMatchingEndTag {
                            token_index: action.trigger().token_index(),
                        },
                    );
                }
                if let Some(target) = nearest_open_selected_ordinary(nodes, &open, *name) {
                    return Err(
                        HtmlTreeFreezeError::UnmatchedSelectedOrdinaryEndTagWithOpenTarget(target),
                    );
                }
                spend_end_token(&mut spent_end_tokens, action.trigger().token_index())?;
                ignored_unmatched.push(ReplayedUnmatchedEnd {
                    trigger_token: action.trigger().token_index(),
                    name: *name,
                });
            }
            _ => reject_interleaved_recovery(pending)?,
        }
    }
    reject_interleaved_recovery(pending)?;

    validate_selected_ordinary_diagnostics(
        diagnostics,
        &recovery_groups,
        &ignored_unmatched,
        tokenizer_run,
    )?;

    for id in final_open_selected_ordinary {
        if selected_ordinary_name(nodes, *id).is_none() {
            return Err(HtmlTreeFreezeError::FinalOpenSelectedOrdinaryIsNotASelectedElement(*id));
        }
    }
    if open != final_open_selected_ordinary {
        return Err(
            HtmlTreeFreezeError::FinalOpenSelectedOrdinaryStateMismatch {
                replayed: open,
                actual: final_open_selected_ordinary.to_vec(),
            },
        );
    }
    Ok(())
}

/// One committed heterogeneous recovery group, as the freeze replay
/// reconstructed it rather than as the session described it.
struct ReplayedRecoveryGroup {
    /// The retained-run index of the one authored end tag the whole group
    /// shares.
    trigger_token: usize,
    /// The closed selected name of the group's target, which is also the name
    /// its authored end tag must be spelled with.
    target_name: HtmlSelectedOrdinaryElementName,
}

/// One committed ignored unmatched selected ordinary end tag, as the freeze
/// replay reconstructed it.
struct ReplayedUnmatchedEnd {
    trigger_token: usize,
    name: HtmlSelectedOrdinaryElementName,
}

/// Records that a retained token has spent its one terminal selected-end
/// decision, rejecting a second one.
fn spend_end_token(spent: &mut Vec<usize>, token_index: usize) -> Result<(), HtmlTreeFreezeError> {
    if spent.contains(&token_index) {
        return Err(HtmlTreeFreezeError::DuplicateSelectedOrdinaryEndTokenDecision { token_index });
    }
    spent.push(token_index);
    Ok(())
}

/// Rejects an open recovery group that something other than its own target's
/// matching closure reached.
const fn reject_interleaved_recovery(
    pending: Option<(HtmlConstructedNodeId, usize)>,
) -> Result<(), HtmlTreeFreezeError> {
    match pending {
        Some((target, _)) => Err(HtmlTreeFreezeError::UnterminatedSelectedOrdinaryRecovery(
            target,
        )),
        None => Ok(()),
    }
}

/// Correlates the selected ordinary diagnostics against the replayed action
/// stream.
///
/// One misnested diagnostic per committed recovery group, carrying that
/// group's own exact authored end tag and the accepted recovery summary; and
/// exactly the ignored unmatched end-tag actions' trigger tokens as unmatched
/// diagnostics. A missing, duplicated, wrongly-triggered, wrongly-summarized,
/// or recovery-free misnested diagnostic all fail here — including one whose
/// recorded token index is right but whose recorded boundary is not that
/// token's own complete authored evidence.
fn validate_selected_ordinary_diagnostics(
    diagnostics: &[HtmlTreeDiagnostic],
    recovery_groups: &[ReplayedRecoveryGroup],
    ignored_unmatched: &[ReplayedUnmatchedEnd],
    tokenizer_run: &HtmlTokenizerRunResult,
) -> Result<(), HtmlTreeFreezeError> {
    let misnested: Vec<&HtmlTreeDiagnostic> = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code() == HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag
        })
        .collect();
    let group_tokens: Vec<usize> = recovery_groups
        .iter()
        .map(|group| group.trigger_token)
        .collect();
    let misnested_tokens: Vec<usize> = misnested
        .iter()
        .map(|diagnostic| diagnostic.trigger().token_index())
        .collect();
    let paired = group_tokens == misnested_tokens
        && recovery_groups
            .iter()
            .zip(&misnested)
            .all(|(group, found)| {
                found.recovery()
                    == HtmlTreeRecovery::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget
                    && is_matching_end_tag_trigger(
                        group.target_name,
                        found.trigger(),
                        tokenizer_run,
                    )
            });
    if !paired {
        return Err(
            HtmlTreeFreezeError::SelectedOrdinaryRecoveryDiagnosticMismatch {
                recovery_groups: group_tokens,
                misnested_diagnostics: misnested_tokens,
            },
        );
    }

    let unmatched: Vec<&HtmlTreeDiagnostic> = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code() == HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag
        })
        .collect();
    let unmatched_action_tokens: Vec<usize> = ignored_unmatched
        .iter()
        .map(|end| end.trigger_token)
        .collect();
    let unmatched_diagnostic_tokens: Vec<usize> = unmatched
        .iter()
        .map(|diagnostic| diagnostic.trigger().token_index())
        .collect();
    // One per ignored disposition, in the same order, carrying that
    // disposition's own exact authored end tag — recorrelated against the
    // retained run — and the accepted ignored-token recovery.
    let paired = unmatched_action_tokens == unmatched_diagnostic_tokens
        && ignored_unmatched
            .iter()
            .zip(&unmatched)
            .all(|(end, found)| {
                found.recovery() == HtmlTreeRecovery::IgnoredToken
                    && is_matching_end_tag_trigger(end.name, found.trigger(), tokenizer_run)
            });
    if !paired {
        return Err(
            HtmlTreeFreezeError::UnmatchedSelectedOrdinaryEndTagDiagnosticMismatch {
                actions: unmatched_action_tokens,
                diagnostics: unmatched_diagnostic_tokens,
            },
        );
    }
    Ok(())
}

/// The closed selected ordinary name of a stored node, when it is one.
fn selected_ordinary_name(
    nodes: &[HtmlTreeNode],
    id: HtmlConstructedNodeId,
) -> Option<HtmlSelectedOrdinaryElementName> {
    match find(nodes, id)?.kind() {
        HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) => Some(selected.name()),
        HtmlTreeNodeKind::Document
        | HtmlTreeNodeKind::Element(HtmlElement::Shell(_))
        | HtmlTreeNodeKind::Element(HtmlElement::Paragraph(_))
        | HtmlTreeNodeKind::Text(_) => None,
    }
}

/// The nearest currently-open selected ordinary element of `name`, innermost
/// first, over the replayed selected stack.
fn nearest_open_selected_ordinary(
    nodes: &[HtmlTreeNode],
    open: &[HtmlConstructedNodeId],
    name: HtmlSelectedOrdinaryElementName,
) -> Option<HtmlConstructedNodeId> {
    open.iter()
        .rev()
        .copied()
        .find(|id| selected_ordinary_name(nodes, *id) == Some(name))
}

/// Proves a recorded closure trigger is the exact emitted matching end tag.
///
/// Reads the retained emitted token at the trigger's own index and compares it
/// against the recorded evidence. This is correlation of retained evidence, not
/// source discovery: no source search, rescan, or retokenization occurs, and
/// the tokenizer is neither consulted nor re-run.
fn validate_closure_trigger(
    node: HtmlConstructedNodeId,
    name: HtmlSelectedOrdinaryElementName,
    trigger: &HtmlTreeTokenTrigger,
    tokenizer_run: &HtmlTokenizerRunResult,
) -> Result<(), HtmlTreeFreezeError> {
    // End of file has no authored extent and may never close anything.
    if trigger.authored_boundary().is_none() {
        return Err(HtmlTreeFreezeError::FabricatedSelectedOrdinaryClosure(node));
    }
    if is_matching_end_tag_trigger(name, trigger, tokenizer_run) {
        Ok(())
    } else {
        Err(HtmlTreeFreezeError::ClosureTriggerIsNotTheMatchingEndTag {
            node,
            token_index: trigger.token_index(),
        })
    }
}

/// Whether a recorded trigger is the retained run's own emitted end tag for
/// `name`.
///
/// Shared by matching-closure and heterogeneous recovery validation so the two
/// relations are proved against exactly the same authored evidence, which is
/// what lets one authored end tag legitimately trigger several ordered
/// recovery pops plus exactly one closure. Reads the retained emitted token at
/// the trigger's own index: this is correlation of retained evidence, not
/// source discovery, and no source search, rescan, or retokenization occurs.
fn is_matching_end_tag_trigger(
    name: HtmlSelectedOrdinaryElementName,
    trigger: &HtmlTreeTokenTrigger,
    tokenizer_run: &HtmlTokenizerRunResult,
) -> bool {
    is_exact_tag_trigger(
        trigger,
        tokenizer_run,
        HtmlTagKind::End,
        &[name.interpreted()],
    )
}

/// Resolves a recorded selected ordinary subject by semantic constructed
/// identity and checks that it really is that element.
fn expect_selected_ordinary(
    nodes: &[HtmlTreeNode],
    id: HtmlConstructedNodeId,
    name: HtmlSelectedOrdinaryElementName,
) -> Result<(), HtmlTreeFreezeError> {
    if find(nodes, id).is_none() {
        return Err(HtmlTreeFreezeError::UnresolvedActionSubject(id));
    }
    if selected_ordinary_name(nodes, id) == Some(name) {
        Ok(())
    } else {
        Err(HtmlTreeFreezeError::ClosureSubjectIsNotTheSelectedOrdinaryElement { node: id, name })
    }
}

/// Validates TC-S5 Paragraph lifecycle meaning independently of session code.
///
/// It replays the mixed selected-ordinary / Paragraph open-content actions,
/// correlates each P action to retained tokenizer evidence, proves the bounded
/// P-current invariant, checks start-triggered and synthesized action order,
/// pairs unmatched-P diagnostics to the synthesized insertion, and finally
/// compares replay state with the session's immutable final-open checkpoint.
fn validate_paragraph_lifecycle(
    nodes: &[HtmlTreeNode],
    actions: &[HtmlTreeAction],
    diagnostics: &[HtmlTreeDiagnostic],
    tokenizer_run: &HtmlTokenizerRunResult,
    final_open_paragraph: Option<HtmlConstructedNodeId>,
) -> Result<(), HtmlTreeFreezeError> {
    let paragraph_nodes: Vec<HtmlConstructedNodeId> = nodes
        .iter()
        .filter_map(|node| paragraph(nodes, node.id()).map(|_| node.id()))
        .collect();
    let mut inserted: Vec<HtmlConstructedNodeId> = Vec::new();
    let mut open_content: Vec<HtmlConstructedNodeId> = Vec::new();
    let mut synthesis_tokens = Vec::new();

    for (index, action) in actions.iter().enumerate() {
        match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { node, .. } => {
                if open_content
                    .iter()
                    .any(|open| paragraph(nodes, *open).is_some())
                {
                    return Err(HtmlTreeFreezeError::NonLifoParagraphInteraction(*node));
                }
                open_content.push(*node);
            }
            HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { node, .. }
            | HtmlTreeActionKind::ClosedSelectedOrdinaryElement { node, .. } => {
                if open_content.last() != Some(node) {
                    return Err(HtmlTreeFreezeError::NonLifoParagraphInteraction(*node));
                }
                open_content.pop();
            }
            HtmlTreeActionKind::InsertedAuthoredParagraphElement { node } => {
                let Some(element) = paragraph(nodes, *node) else {
                    return Err(HtmlTreeFreezeError::ParagraphActionSubjectIsNotParagraph(
                        *node,
                    ));
                };
                if inserted.contains(node) {
                    return Err(HtmlTreeFreezeError::DuplicateParagraphInsertion(*node));
                }
                if open_content
                    .iter()
                    .any(|open| paragraph(nodes, *open).is_some())
                {
                    return Err(HtmlTreeFreezeError::NonLifoParagraphInteraction(*node));
                }
                if !paragraph_authored_insertion_matches(element, action.trigger(), tokenizer_run) {
                    return Err(
                        HtmlTreeFreezeError::ParagraphAuthoredInsertionTriggerMismatch {
                            node: *node,
                            token_index: action.trigger().token_index(),
                        },
                    );
                }
                inserted.push(*node);
                open_content.push(*node);
            }
            HtmlTreeActionKind::InsertedSynthesizedParagraphElement { node, cause } => {
                let Some(element) = paragraph(nodes, *node) else {
                    return Err(HtmlTreeFreezeError::ParagraphActionSubjectIsNotParagraph(
                        *node,
                    ));
                };
                if inserted.contains(node) {
                    return Err(HtmlTreeFreezeError::DuplicateParagraphInsertion(*node));
                }
                if open_content
                    .iter()
                    .any(|open| paragraph(nodes, *open).is_some())
                {
                    return Err(HtmlTreeFreezeError::NonLifoParagraphInteraction(*node));
                }
                if *cause != HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag
                    || !matches!(
                        element.origin(),
                        HtmlParagraphElementOrigin::Synthesized(
                            HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag
                        )
                    )
                    || !is_exact_tag_trigger(
                        action.trigger(),
                        tokenizer_run,
                        HtmlTagKind::End,
                        &["p"],
                    )
                {
                    return Err(
                        HtmlTreeFreezeError::ParagraphSynthesizedInsertionTriggerMismatch {
                            node: *node,
                            token_index: action.trigger().token_index(),
                        },
                    );
                }
                inserted.push(*node);
                open_content.push(*node);
                synthesis_tokens.push(action.trigger().token_index());

                let Some(next) = actions.get(index + 1) else {
                    return Err(HtmlTreeFreezeError::ParagraphSynthesisClosureMismatch {
                        token_index: action.trigger().token_index(),
                    });
                };
                if !matches!(
                    next.kind(),
                    HtmlTreeActionKind::ClosedParagraphElement {
                        node: closed,
                        closure: HtmlParagraphClosure::UnmatchedEndTagSynthesized,
                    } if *closed == *node
                        && next.trigger().token_index() == action.trigger().token_index()
                        && same_trigger(next.trigger(), action.trigger())
                ) {
                    return Err(HtmlTreeFreezeError::ParagraphSynthesisClosureMismatch {
                        token_index: action.trigger().token_index(),
                    });
                }
            }
            HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { node, target } => {
                if paragraph(nodes, *node).is_none() {
                    return Err(HtmlTreeFreezeError::ParagraphActionSubjectIsNotParagraph(
                        *node,
                    ));
                }
                let Some(target_name) = selected_ordinary_name(nodes, *target) else {
                    return Err(
                        HtmlTreeFreezeError::ParagraphImpliedPopTargetIsNotSelectedOrdinaryElement(
                            *target,
                        ),
                    );
                };
                if open_content.last() != Some(node) {
                    return Err(HtmlTreeFreezeError::NonLifoParagraphInteraction(*node));
                }
                if nearest_open_selected_ordinary(nodes, &open_content, target_name)
                    != Some(*target)
                {
                    return Err(
                        HtmlTreeFreezeError::ParagraphImpliedPopTargetIsNotNearestMatchingSelectedOrdinary(
                            *target,
                        ),
                    );
                }
                if !is_matching_end_tag_trigger(target_name, action.trigger(), tokenizer_run) {
                    return Err(HtmlTreeFreezeError::ParagraphImpliedPopTriggerMismatch {
                        node: *node,
                        target: *target,
                        token_index: action.trigger().token_index(),
                    });
                }
                open_content.pop();

                let Some(next) = actions.get(index + 1) else {
                    return Err(
                        HtmlTreeFreezeError::ParagraphImpliedPopContinuationMismatch {
                            token_index: action.trigger().token_index(),
                        },
                    );
                };
                let same = same_trigger(next.trigger(), action.trigger());
                let continuation = match next.kind() {
                    HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag {
                        target: recovery_target,
                        ..
                    } => *recovery_target == *target,
                    HtmlTreeActionKind::ClosedSelectedOrdinaryElement { node: closed, name } => {
                        *closed == *target && *name == target_name
                    }
                    _ => false,
                };
                if !same || !continuation {
                    return Err(
                        HtmlTreeFreezeError::ParagraphImpliedPopContinuationMismatch {
                            token_index: action.trigger().token_index(),
                        },
                    );
                }
            }
            HtmlTreeActionKind::ClosedParagraphElement { node, closure } => {
                if paragraph(nodes, *node).is_none() {
                    return Err(HtmlTreeFreezeError::ParagraphActionSubjectIsNotParagraph(
                        *node,
                    ));
                }
                if open_content.last() != Some(node) {
                    return Err(HtmlTreeFreezeError::NonLifoParagraphInteraction(*node));
                }
                let valid_trigger = match closure {
                    HtmlParagraphClosure::MatchingEndTag
                    | HtmlParagraphClosure::UnmatchedEndTagSynthesized => is_exact_tag_trigger(
                        action.trigger(),
                        tokenizer_run,
                        HtmlTagKind::End,
                        &["p"],
                    ),
                    HtmlParagraphClosure::StartTriggered => is_exact_tag_trigger(
                        action.trigger(),
                        tokenizer_run,
                        HtmlTagKind::Start,
                        &["p", "div", "section"],
                    ),
                };
                if !valid_trigger {
                    return Err(HtmlTreeFreezeError::ParagraphClosureTriggerMismatch {
                        node: *node,
                        token_index: action.trigger().token_index(),
                    });
                }
                if *closure == HtmlParagraphClosure::UnmatchedEndTagSynthesized
                    && !matches!(
                        paragraph(nodes, *node).map(HtmlParagraphElement::origin),
                        Some(HtmlParagraphElementOrigin::Synthesized(
                            HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag
                        ))
                    )
                {
                    return Err(HtmlTreeFreezeError::ParagraphClosureTriggerMismatch {
                        node: *node,
                        token_index: action.trigger().token_index(),
                    });
                }
                open_content.pop();

                if *closure == HtmlParagraphClosure::StartTriggered {
                    let Some(next) = actions.get(index + 1) else {
                        return Err(
                            HtmlTreeFreezeError::ParagraphStartTriggeredInsertionMismatch {
                                token_index: action.trigger().token_index(),
                            },
                        );
                    };
                    let same = next.trigger().token_index() == action.trigger().token_index()
                        && same_trigger(next.trigger(), action.trigger());
                    let expected = retained_start_tag_name(action.trigger(), tokenizer_run);
                    let next_matches = matches!(
                        (expected, next.kind()),
                        (
                            Some("p"),
                            HtmlTreeActionKind::InsertedAuthoredParagraphElement { .. },
                        ) | (
                            Some("div"),
                            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                                name: HtmlSelectedOrdinaryElementName::Div,
                                ..
                            },
                        ) | (
                            Some("section"),
                            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                                name: HtmlSelectedOrdinaryElementName::Section,
                                ..
                            },
                        )
                    );
                    if !same || !next_matches {
                        return Err(
                            HtmlTreeFreezeError::ParagraphStartTriggeredInsertionMismatch {
                                token_index: action.trigger().token_index(),
                            },
                        );
                    }
                }
            }
            _ => {}
        }
    }

    for node in paragraph_nodes {
        if inserted
            .iter()
            .filter(|inserted| **inserted == node)
            .count()
            != 1
        {
            return Err(HtmlTreeFreezeError::ParagraphInsertionInventoryMismatch(
                node,
            ));
        }
    }

    let unmatched: Vec<&HtmlTreeDiagnostic> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code() == HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag)
        .collect();
    let diagnostic_tokens: Vec<usize> = unmatched
        .iter()
        .map(|diagnostic| diagnostic.trigger().token_index())
        .collect();
    let paired = synthesis_tokens == diagnostic_tokens
        && unmatched.iter().all(|diagnostic| {
            diagnostic.recovery() == HtmlTreeRecovery::SynthesizedParagraphElementAndClosedIt
                && is_exact_tag_trigger(
                    diagnostic.trigger(),
                    tokenizer_run,
                    HtmlTagKind::End,
                    &["p"],
                )
        });
    if !paired {
        return Err(HtmlTreeFreezeError::UnmatchedParagraphDiagnosticMismatch {
            syntheses: synthesis_tokens,
            diagnostics: diagnostic_tokens,
        });
    }

    let replayed_paragraphs: Vec<HtmlConstructedNodeId> = open_content
        .iter()
        .copied()
        .filter(|id| paragraph(nodes, *id).is_some())
        .collect();
    if replayed_paragraphs.len() > 1
        || replayed_paragraphs
            .first()
            .is_some_and(|paragraph| open_content.last() != Some(paragraph))
    {
        return Err(HtmlTreeFreezeError::NonLifoParagraphInteraction(
            *replayed_paragraphs
                .last()
                .expect("invalid Paragraph state is non-empty"),
        ));
    }
    let replayed = replayed_paragraphs.first().copied();
    if let Some(actual) = final_open_paragraph
        && paragraph(nodes, actual).is_none()
    {
        return Err(HtmlTreeFreezeError::FinalOpenParagraphIsNotParagraph(
            actual,
        ));
    }
    if replayed != final_open_paragraph {
        return Err(HtmlTreeFreezeError::FinalOpenParagraphStateMismatch {
            replayed,
            actual: final_open_paragraph,
        });
    }
    Ok(())
}

/// The body position reconstructed from durable action chronology.
///
/// This is freeze-owned replay state, not a snapshot copied from the mutable
/// session. `InBody` begins only after the retained Body insertion action;
/// every later transition is reconstructed from the existing shell
/// acknowledgement and same-token reprocess actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplayedBodyPosition {
    InBody,
    AfterBody,
    AfterAfterBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplayedBodyCharacterClass {
    AllHtmlWhitespace,
    AllNonHtmlWhitespace,
    Mixed,
}

/// Independently validates TC-S7 body-end and successor composition.
///
/// The mutable session contributes no conclusion to this replay. It consumes
/// only retained tokenizer tokens, committed actions and diagnostics, stored
/// semantic identities, and the two immutable final-open checkpoints. In one
/// ordered pass it reconstructs the mixed `Div | Section | P` lifecycle and
/// body position, then proves that a plain authored `</body>`:
///
/// - is acknowledged once from `InBody`;
/// - records exactly one dedicated diagnostic iff a selected ordinary element
///   is open (P alone is allowed);
/// - preserves the complete open-content state and every identity;
/// - performs no same-trigger close, implied pop, recovery, synthesis, or
///   creation after the transition;
/// - leaves AfterBody whitespace under the retained current node without
///   reprocess;
/// - pairs non-whitespace with the accepted diagnostic and exactly-one
///   reprocess before insertion under that same retained parent; and
/// - never fabricates the InBody selected-open EOF diagnostic while the
///   replayed position remains AfterBody or AfterAfterBody.
fn validate_body_end_open_stack_transitions(
    nodes: &[HtmlTreeNode],
    actions: &[HtmlTreeAction],
    diagnostics: &[HtmlTreeDiagnostic],
    tokenizer_run: &HtmlTokenizerRunResult,
    final_open_selected_ordinary: &[HtmlConstructedNodeId],
    final_open_paragraph: Option<HtmlConstructedNodeId>,
) -> Result<(), HtmlTreeFreezeError> {
    let body = nodes
        .iter()
        .find(|node| is_shell_element(node, HtmlShellElementName::Body))
        .map(HtmlTreeNode::id);
    let mut position = None;
    let mut open_content: Vec<HtmlConstructedNodeId> = Vec::new();
    let mut body_end_tokens = Vec::new();
    let mut matched_body_diagnostics = Vec::new();
    let mut pending_reprocessed_text: Option<(usize, HtmlConstructedNodeId)> = None;
    let mut consumed_successor_text_tokens = Vec::new();

    for (action_index, action) in actions.iter().enumerate() {
        let token_index = action.trigger().token_index();
        if let Some((pending_token, _)) = pending_reprocessed_text
            && pending_token != token_index
        {
            return Err(HtmlTreeFreezeError::BodyEndAfterBodySuccessorMismatch {
                token_index: pending_token,
            });
        }

        match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredShellElement {
                name: HtmlShellElementName::Body,
                ..
            }
            | HtmlTreeActionKind::InsertedSynthesizedShellElement {
                name: HtmlShellElementName::Body,
                ..
            } => {
                position = Some(ReplayedBodyPosition::InBody);
            }
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { node, .. } => {
                open_content.push(*node);
            }
            HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { node, .. }
            | HtmlTreeActionKind::ClosedSelectedOrdinaryElement { node, .. } => {
                if open_content.last() != Some(node) {
                    return Err(HtmlTreeFreezeError::BodyEndOpenContentReplayMismatch {
                        token_index,
                    });
                }
                open_content.pop();
            }
            HtmlTreeActionKind::InsertedAuthoredParagraphElement { node }
            | HtmlTreeActionKind::InsertedSynthesizedParagraphElement { node, .. } => {
                open_content.push(*node);
            }
            HtmlTreeActionKind::ClosedParagraphElement { node, .. }
            | HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { node, .. } => {
                if open_content.last() != Some(node) {
                    return Err(HtmlTreeFreezeError::BodyEndOpenContentReplayMismatch {
                        token_index,
                    });
                }
                open_content.pop();
            }
            HtmlTreeActionKind::AcknowledgedShellEndTag {
                name: HtmlShellElementName::Body,
            } => {
                if !is_plain_body_end_trigger(action.trigger(), tokenizer_run) {
                    return Err(HtmlTreeFreezeError::BodyEndAcknowledgementTriggerMismatch {
                        token_index,
                    });
                }
                if body_end_tokens.contains(&token_index) {
                    return Err(HtmlTreeFreezeError::DuplicateBodyEndAcknowledgement {
                        token_index,
                    });
                }
                if position != Some(ReplayedBodyPosition::InBody) {
                    return Err(HtmlTreeFreezeError::BodyEndAcknowledgementOutsideInBody {
                        token_index,
                    });
                }

                let selected_open = open_content
                    .iter()
                    .filter(|id| selected_ordinary_name(nodes, **id).is_some())
                    .count();
                let body_diagnostics: Vec<(usize, &HtmlTreeDiagnostic)> = diagnostics
                    .iter()
                    .enumerate()
                    .filter(|(_, diagnostic)| {
                        diagnostic.code()
                            == HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements
                            && diagnostic.trigger().token_index() == token_index
                    })
                    .collect();
                let expected = usize::from(selected_open != 0);
                if body_diagnostics.len() != expected {
                    return Err(HtmlTreeFreezeError::BodyEndDiagnosticCardinalityMismatch {
                        token_index,
                        selected_open,
                        diagnostics: body_diagnostics.len(),
                    });
                }
                if let [(diagnostic_index, diagnostic)] = body_diagnostics.as_slice() {
                    if diagnostic.recovery()
                        != HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements
                        || !same_trigger(diagnostic.trigger(), action.trigger())
                        || !is_plain_body_end_trigger(diagnostic.trigger(), tokenizer_run)
                        || diagnostics.iter().enumerate().any(|(other_index, other)| {
                            other_index > *diagnostic_index
                                && same_trigger(other.trigger(), diagnostic.trigger())
                        })
                    {
                        return Err(
                            HtmlTreeFreezeError::BodyEndDiagnosticTriggerOrRecoveryMismatch {
                                token_index,
                            },
                        );
                    }
                    matched_body_diagnostics.push(*diagnostic_index);
                }

                // The action order is load-bearing. Shell creation/closure may
                // legitimately precede the acknowledgement when one early
                // `</body>` token walks through implied shell construction.
                // No selected/P/text creation may share a body-end trigger,
                // and nothing at all may mutate after the acknowledgement
                // under that consumed token.
                for (other_index, other) in actions.iter().enumerate() {
                    if other_index == action_index
                        || !same_trigger(other.trigger(), action.trigger())
                    {
                        continue;
                    }
                    // A duplicate body acknowledgement is not a mutation;
                    // let the ordered replay reach it and report the more
                    // exact duplicate-decision failure below.
                    if matches!(
                        other.kind(),
                        HtmlTreeActionKind::AcknowledgedShellEndTag {
                            name: HtmlShellElementName::Body
                        }
                    ) {
                        continue;
                    }
                    let forbidden_anywhere = matches!(
                        other.kind(),
                        HtmlTreeActionKind::InsertedTextNode { .. }
                            | HtmlTreeActionKind::AppendedToTextNode { .. }
                            | HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { .. }
                            | HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
                            | HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
                            | HtmlTreeActionKind::InsertedAuthoredParagraphElement { .. }
                            | HtmlTreeActionKind::InsertedSynthesizedParagraphElement { .. }
                            | HtmlTreeActionKind::ClosedParagraphElement { .. }
                            | HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
                    );
                    if forbidden_anywhere || other_index > action_index {
                        return Err(HtmlTreeFreezeError::BodyEndSameTriggerMutation {
                            token_index,
                        });
                    }
                }

                body_end_tokens.push(token_index);
                position = Some(ReplayedBodyPosition::AfterBody);
            }
            HtmlTreeActionKind::AcknowledgedShellEndTag {
                name: HtmlShellElementName::Html,
            } if position == Some(ReplayedBodyPosition::AfterBody) => {
                position = Some(ReplayedBodyPosition::AfterAfterBody);
            }
            HtmlTreeActionKind::ReprocessedToken
                if position == Some(ReplayedBodyPosition::AfterBody) =>
            {
                if replayed_body_character_class(action.trigger(), tokenizer_run)
                    != Some(ReplayedBodyCharacterClass::AllNonHtmlWhitespace)
                    || actions
                        .iter()
                        .filter(|candidate| {
                            matches!(candidate.kind(), HtmlTreeActionKind::ReprocessedToken)
                                && same_trigger(candidate.trigger(), action.trigger())
                        })
                        .count()
                        != 1
                {
                    return Err(HtmlTreeFreezeError::BodyEndAfterBodySuccessorMismatch {
                        token_index,
                    });
                }
                let after_body_diagnostics: Vec<&HtmlTreeDiagnostic> = diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.code() == HtmlTreeDiagnosticCode::AfterBodyCharacterData
                            && diagnostic.trigger().token_index() == token_index
                    })
                    .collect();
                if !matches!(after_body_diagnostics.as_slice(), [diagnostic]
                    if diagnostic.recovery()
                        == HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken
                        && same_trigger(diagnostic.trigger(), action.trigger()))
                {
                    return Err(HtmlTreeFreezeError::BodyEndAfterBodySuccessorMismatch {
                        token_index,
                    });
                }
                let Some(expected_parent) = open_content.last().copied().or(body) else {
                    return Err(HtmlTreeFreezeError::BodyEndAfterBodySuccessorMismatch {
                        token_index,
                    });
                };
                pending_reprocessed_text = Some((token_index, expected_parent));
                position = Some(ReplayedBodyPosition::InBody);
            }
            HtmlTreeActionKind::InsertedTextNode { node }
            | HtmlTreeActionKind::AppendedToTextNode { node } => {
                if let Some((pending_token, expected_parent)) = pending_reprocessed_text {
                    if pending_token != token_index
                        || find(nodes, *node).and_then(HtmlTreeNode::parent)
                            != Some(expected_parent)
                        || actions
                            .iter()
                            .filter(|candidate| {
                                matches!(
                                    candidate.kind(),
                                    HtmlTreeActionKind::InsertedTextNode { .. }
                                        | HtmlTreeActionKind::AppendedToTextNode { .. }
                                ) && same_trigger(candidate.trigger(), action.trigger())
                            })
                            .count()
                            != 1
                    {
                        return Err(HtmlTreeFreezeError::BodyEndAfterBodySuccessorMismatch {
                            token_index,
                        });
                    }
                    pending_reprocessed_text = None;
                    consumed_successor_text_tokens.push(token_index);
                } else if position == Some(ReplayedBodyPosition::AfterBody)
                    && !body_end_tokens.is_empty()
                {
                    let Some(expected_parent) = open_content.last().copied().or(body) else {
                        return Err(HtmlTreeFreezeError::BodyEndAfterBodySuccessorMismatch {
                            token_index,
                        });
                    };
                    let text_actions = actions
                        .iter()
                        .filter(|candidate| {
                            matches!(
                                candidate.kind(),
                                HtmlTreeActionKind::InsertedTextNode { .. }
                                    | HtmlTreeActionKind::AppendedToTextNode { .. }
                            ) && same_trigger(candidate.trigger(), action.trigger())
                        })
                        .count();
                    let reprocesses = actions
                        .iter()
                        .filter(|candidate| {
                            matches!(candidate.kind(), HtmlTreeActionKind::ReprocessedToken)
                                && same_trigger(candidate.trigger(), action.trigger())
                        })
                        .count();
                    let after_body_diagnostics = diagnostics
                        .iter()
                        .filter(|diagnostic| {
                            diagnostic.code() == HtmlTreeDiagnosticCode::AfterBodyCharacterData
                                && diagnostic.trigger().token_index() == token_index
                        })
                        .count();
                    if replayed_body_character_class(action.trigger(), tokenizer_run)
                        != Some(ReplayedBodyCharacterClass::AllHtmlWhitespace)
                        || find(nodes, *node).and_then(HtmlTreeNode::parent)
                            != Some(expected_parent)
                        || text_actions != 1
                        || reprocesses != 0
                        || after_body_diagnostics != 0
                        || consumed_successor_text_tokens.contains(&token_index)
                    {
                        return Err(HtmlTreeFreezeError::BodyEndAfterBodySuccessorMismatch {
                            token_index,
                        });
                    }
                    consumed_successor_text_tokens.push(token_index);
                }
            }
            HtmlTreeActionKind::StoppedParsing
                if matches!(
                    position,
                    Some(ReplayedBodyPosition::AfterBody | ReplayedBodyPosition::AfterAfterBody)
                ) =>
            {
                if !is_end_of_file_trigger(action.trigger(), tokenizer_run) {
                    return Err(HtmlTreeFreezeError::BodyEndAfterBodySuccessorMismatch {
                        token_index,
                    });
                }
                let selected_is_open = open_content
                    .iter()
                    .any(|id| selected_ordinary_name(nodes, *id).is_some());
                let fabricated = diagnostics.iter().any(|diagnostic| {
                    diagnostic.code()
                        == HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile
                        && diagnostic.trigger().token_index() == token_index
                });
                if selected_is_open && fabricated {
                    return Err(HtmlTreeFreezeError::BodyEndAfterBodyEofDiagnosticMismatch {
                        token_index,
                    });
                }
            }
            _ => {}
        }
    }

    if let Some((token_index, _)) = pending_reprocessed_text {
        return Err(HtmlTreeFreezeError::BodyEndAfterBodySuccessorMismatch { token_index });
    }
    for (diagnostic_index, diagnostic) in diagnostics.iter().enumerate() {
        if diagnostic.code() == HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements
            && !matched_body_diagnostics.contains(&diagnostic_index)
        {
            return Err(HtmlTreeFreezeError::OrphanBodyEndDiagnostic {
                token_index: diagnostic.trigger().token_index(),
            });
        }
    }

    let replayed_selected: Vec<HtmlConstructedNodeId> = open_content
        .iter()
        .copied()
        .filter(|id| selected_ordinary_name(nodes, *id).is_some())
        .collect();
    let replayed_paragraphs: Vec<HtmlConstructedNodeId> = open_content
        .iter()
        .copied()
        .filter(|id| paragraph(nodes, *id).is_some())
        .collect();
    let replayed_paragraph = replayed_paragraphs.first().copied();
    if replayed_selected != final_open_selected_ordinary
        || replayed_paragraphs.len() > 1
        || replayed_paragraph != final_open_paragraph
    {
        return Err(HtmlTreeFreezeError::BodyEndFinalOpenStateMismatch);
    }

    Ok(())
}

fn is_plain_body_end_trigger(
    trigger: &HtmlTreeTokenTrigger,
    tokenizer_run: &HtmlTokenizerRunResult,
) -> bool {
    let Some(HtmlToken::Tag(tag)) = tokenizer_run.tokens().get(trigger.token_index()) else {
        return false;
    };
    tag.kind() == HtmlTagKind::End
        && tag.name().interpreted() == "body"
        && tag.attributes().is_empty()
        && tag.self_closing_solidus().is_none()
        && exact_anchor(trigger.authored_boundary(), Some(tag.complete()))
}

fn is_end_of_file_trigger(
    trigger: &HtmlTreeTokenTrigger,
    tokenizer_run: &HtmlTokenizerRunResult,
) -> bool {
    matches!(
        tokenizer_run.tokens().get(trigger.token_index()),
        Some(HtmlToken::EndOfFile(_))
    ) && trigger.authored_boundary().is_none()
}

fn replayed_body_character_class(
    trigger: &HtmlTreeTokenTrigger,
    tokenizer_run: &HtmlTokenizerRunResult,
) -> Option<ReplayedBodyCharacterClass> {
    let Some(HtmlToken::Character(character)) = tokenizer_run.tokens().get(trigger.token_index())
    else {
        return None;
    };
    if !exact_anchor(trigger.authored_boundary(), Some(character.source())) {
        return None;
    }
    let mut whitespace = false;
    let mut non_whitespace = false;
    for value in character.interpreted().chars() {
        if matches!(value, '\t' | '\n' | '\u{000c}' | '\r' | ' ') {
            whitespace = true;
        } else {
            non_whitespace = true;
        }
    }
    Some(match (whitespace, non_whitespace) {
        (true, true) => ReplayedBodyCharacterClass::Mixed,
        (false, true) => ReplayedBodyCharacterClass::AllNonHtmlWhitespace,
        (true, false) | (false, false) => ReplayedBodyCharacterClass::AllHtmlWhitespace,
    })
}

fn paragraph(nodes: &[HtmlTreeNode], id: HtmlConstructedNodeId) -> Option<&HtmlParagraphElement> {
    match find(nodes, id)?.kind() {
        HtmlTreeNodeKind::Element(HtmlElement::Paragraph(paragraph)) => Some(paragraph),
        _ => None,
    }
}

fn paragraph_authored_insertion_matches(
    paragraph: &HtmlParagraphElement,
    trigger: &HtmlTreeTokenTrigger,
    tokenizer_run: &HtmlTokenizerRunResult,
) -> bool {
    let HtmlParagraphElementOrigin::Authored { complete, raw_name } = paragraph.origin() else {
        return false;
    };
    let Some(HtmlToken::Tag(tag)) = tokenizer_run.tokens().get(trigger.token_index()) else {
        return false;
    };
    tag.kind() == HtmlTagKind::Start
        && tag.name().interpreted() == "p"
        && exact_anchor(trigger.authored_boundary(), Some(tag.complete()))
        && exact_anchor(Some(complete), Some(tag.complete()))
        && exact_anchor(Some(raw_name), Some(tag.name().source()))
}

fn retained_start_tag_name<'a>(
    trigger: &HtmlTreeTokenTrigger,
    tokenizer_run: &'a HtmlTokenizerRunResult,
) -> Option<&'a str> {
    let Some(HtmlToken::Tag(tag)) = tokenizer_run.tokens().get(trigger.token_index()) else {
        return None;
    };
    if tag.kind() != HtmlTagKind::Start
        || !exact_anchor(trigger.authored_boundary(), Some(tag.complete()))
    {
        return None;
    }
    Some(tag.name().interpreted())
}

fn is_exact_tag_trigger(
    trigger: &HtmlTreeTokenTrigger,
    tokenizer_run: &HtmlTokenizerRunResult,
    kind: HtmlTagKind,
    names: &[&str],
) -> bool {
    let Some(HtmlToken::Tag(tag)) = tokenizer_run.tokens().get(trigger.token_index()) else {
        return false;
    };
    tag.kind() == kind
        && names.contains(&tag.name().interpreted())
        && exact_anchor(trigger.authored_boundary(), Some(tag.complete()))
}

fn same_trigger(first: &HtmlTreeTokenTrigger, second: &HtmlTreeTokenTrigger) -> bool {
    first.token_index() == second.token_index()
        && exact_anchor(first.authored_boundary(), second.authored_boundary())
}

fn exact_anchor(first: Option<&SourceAnchor>, second: Option<&SourceAnchor>) -> bool {
    match (first, second) {
        (None, None) => true,
        (Some(first), Some(second)) => {
            first.source_id() == second.source_id()
                && first.range() == second.range()
                && first.fragment() == second.fragment()
        }
        _ => false,
    }
}

fn validate_diagnostic_evidence(
    source: &SourceText,
    diagnostics: &[HtmlTreeDiagnostic],
    emitted_tokens: usize,
) -> Result<(), HtmlTreeFreezeError> {
    let mut previous_index: Option<usize> = None;
    for diagnostic in diagnostics {
        validate_trigger(
            source,
            HtmlTreeEvidenceRole::DiagnosticTrigger,
            diagnostic.trigger(),
            emitted_tokens,
            &mut previous_index,
        )?;
    }
    Ok(())
}

fn validate_trigger(
    source: &SourceText,
    role: HtmlTreeEvidenceRole,
    trigger: &HtmlTreeTokenTrigger,
    emitted_tokens: usize,
    previous_index: &mut Option<usize>,
) -> Result<(), HtmlTreeFreezeError> {
    if trigger.token_index() >= emitted_tokens {
        return Err(HtmlTreeFreezeError::InvalidTokenProgression {
            role,
            token_index: trigger.token_index(),
        });
    }
    if let Some(previous) = *previous_index
        && trigger.token_index() < previous
    {
        return Err(HtmlTreeFreezeError::InvalidTokenProgression {
            role,
            token_index: trigger.token_index(),
        });
    }
    *previous_index = Some(trigger.token_index());
    if let Some(boundary) = trigger.authored_boundary() {
        validate_evidence(source, role, boundary)?;
    }
    Ok(())
}

fn validate_completion(
    source: &SourceText,
    nodes: &[HtmlTreeNode],
    root: HtmlConstructedNodeId,
    tokenizer_run: &HtmlTokenizerRunResult,
    processed_tokens: usize,
    completion: &HtmlTreeCompletion,
) -> Result<(), HtmlTreeFreezeError> {
    match completion {
        HtmlTreeCompletion::Complete => {
            if !matches!(
                tokenizer_run.completion(),
                HtmlTokenizerCompletion::Complete
            ) {
                return Err(HtmlTreeFreezeError::CompletionUpgrade(
                    HtmlTreeCompletionUpgrade::RetainedTokenizerRunIsIncomplete,
                ));
            }
            if processed_tokens != tokenizer_run.tokens().len() {
                return Err(HtmlTreeFreezeError::CompletionUpgrade(
                    HtmlTreeCompletionUpgrade::EmittedTokensRemainUnprocessed,
                ));
            }
            if !is_complete_document_shell(nodes, root) {
                return Err(HtmlTreeFreezeError::CompletionUpgrade(
                    HtmlTreeCompletionUpgrade::DocumentShellIsIncomplete,
                ));
            }
            Ok(())
        }
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete) => Ok(()),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => {
            let mut previous_index = None;
            validate_trigger(
                source,
                HtmlTreeEvidenceRole::UnsupportedTrigger,
                unsupported.trigger(),
                tokenizer_run.tokens().len(),
                &mut previous_index,
            )?;
            let Some(boundary) = unsupported.trigger().authored_boundary() else {
                return Ok(());
            };
            for node in nodes {
                if let Some(HtmlAuthoredSource::StartTag { complete, .. }) = node.authored_source()
                    && complete.range() == boundary.range()
                {
                    return Err(
                        HtmlTreeFreezeError::UnsupportedTriggerLeakedAsAuthoredOrigin(node.id()),
                    );
                }
            }
            Ok(())
        }
    }
}

/// Whether the committed tree is the complete `Document -> html(head, body)`
/// shell every effective `Complete` result must contain. Content below body is
/// intentionally unrestricted by this shell-completeness check.
fn is_complete_document_shell(nodes: &[HtmlTreeNode], root: HtmlConstructedNodeId) -> bool {
    let Some(root_node) = find(nodes, root) else {
        return false;
    };
    let [html_id] = root_node.children() else {
        return false;
    };
    let Some(html) = find(nodes, *html_id) else {
        return false;
    };
    if !is_shell_element(html, HtmlShellElementName::Html) {
        return false;
    }
    let [head_id, body_id] = html.children() else {
        return false;
    };
    let (Some(head), Some(body)) = (find(nodes, *head_id), find(nodes, *body_id)) else {
        return false;
    };
    is_shell_element(head, HtmlShellElementName::Head)
        && is_shell_element(body, HtmlShellElementName::Body)
}

fn is_shell_element(node: &HtmlTreeNode, name: HtmlShellElementName) -> bool {
    matches!(
        node.kind(),
        HtmlTreeNodeKind::Element(HtmlElement::Shell(shell)) if shell.name() == name
    )
}

/// Validates already-retained evidence against the exact supplied
/// [`SourceText`].
///
/// This is revalidation of retained evidence, not source discovery: no source
/// search, delimiter scan, endpoint reconstruction, or retokenization occurs.
fn validate_evidence(
    source: &SourceText,
    role: HtmlTreeEvidenceRole,
    anchor: &SourceAnchor,
) -> Result<(), HtmlTreeFreezeError> {
    if anchor.source_id() != source.id() {
        return Err(HtmlTreeFreezeError::ForeignSourceEvidence {
            role,
            expected: source.id(),
            actual: anchor.source_id(),
        });
    }
    if !source.retains_exact_anchor_source(anchor) {
        return Err(HtmlTreeFreezeError::MismatchedSourceEvidence { role });
    }
    let range = anchor.range();
    let revalidated = source
        .anchor(range.start(), range.end())
        .map_err(|error| HtmlTreeFreezeError::InvalidSourceEvidence { role, error })?;
    if revalidated.fragment() != anchor.fragment() {
        return Err(HtmlTreeFreezeError::MismatchedSourceEvidence { role });
    }
    Ok(())
}
