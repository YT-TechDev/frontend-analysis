//! Immutable, validated result meaning for TC-S1 and its accepted TC-S2,
//! TC-S3, and TC-S4 successors.
//!
//! This module owns the durable half of the accepted Candidate C model: the
//! frozen tree, constructed identity, authored/synthesized provenance,
//! selective action and diagnostic evidence, committed tree coverage, and
//! effective completion. It owns no mutable construction state and never
//! observes the tokenizer or the private
//! [`session`](super::session).
//!
//! # Two distinct relations, never one generic fact
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
/// node-creation action committed during one TC-S1 run. The counter that
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
/// and is never this node's origin.
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

/// The interpreted name of a constructed element, in whichever closed domain
/// owns it.
///
/// This is a projection for reading an element's name without first knowing
/// which domain it belongs to. It never merges the two domains: each arm
/// stays exactly its own closed enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlElementName {
    Shell(HtmlShellElementName),
    SelectedOrdinary(HtmlSelectedOrdinaryElementName),
}

/// What kind of element a constructed element node is.
///
/// Shell meaning and selected ordinary meaning are kept in separate closed
/// domains so neither can be made semantically false by the other. A shell
/// element may be authored or synthesized; a selected ordinary element is
/// always authored.
#[derive(Debug, Clone)]
pub(crate) enum HtmlElement {
    Shell(HtmlShellElement),
    SelectedOrdinary(HtmlSelectedOrdinaryElement),
}

impl HtmlElement {
    pub(crate) fn name(&self) -> HtmlElementName {
        match self {
            Self::Shell(element) => HtmlElementName::Shell(element.name()),
            Self::SelectedOrdinary(element) => HtmlElementName::SelectedOrdinary(element.name()),
        }
    }

    /// The shell element this is, when it is one.
    pub(crate) fn shell(&self) -> Option<&HtmlShellElement> {
        match self {
            Self::Shell(element) => Some(element),
            Self::SelectedOrdinary(_) => None,
        }
    }

    /// The selected ordinary element this is, when it is one.
    pub(crate) fn selected_ordinary(&self) -> Option<&HtmlSelectedOrdinaryElement> {
        match self {
            Self::SelectedOrdinary(element) => Some(element),
            Self::Shell(_) => None,
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
    /// none. The document root and every synthesized shell element return
    /// `None`.
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
/// TC-S3 extended it with the selected ordinary insertion, closure, and
/// ignored-end variants and nothing else.
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
    /// This is deliberately **not**
    /// [`Self::ClosedSelectedOrdinaryElement`]: no matching end tag caused
    /// this pop, so recording a closure for it would fabricate authored
    /// evidence. A later authored end tag of `node`'s own name may still
    /// appear in the source; because `node` already left the open state here,
    /// that end tag is unmatched and closes nothing. `node` is the intervening
    /// element that
    /// was actually popped and `target` is the nearest same-name selected
    /// ordinary element whose end tag caused the pop; both are semantic
    /// creation-event identities, never storage positions, and they are
    /// always distinct. The trigger this action carries is that same exact
    /// authored end tag, which is also the trigger of the target's own
    /// matching closure — one authored end tag legitimately participates in
    /// several ordered recovery relations plus exactly one closure, and is
    /// the authored origin of none of them.
    ///
    /// No node was created and no constructed identity was admitted.
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
    /// An open shell element was closed. No node was created.
    ClosedShellElement {
        node: HtmlConstructedNodeId,
        name: HtmlShellElementName,
        closure: HtmlShellClosure,
    },
    /// A supported shell end tag moved the document position without
    /// creating, closing, or mutating any node.
    AcknowledgedShellEndTag { name: HtmlShellElementName },
    /// A duplicate shell start tag created no node and admitted no
    /// constructed identity.
    DuplicateShellStartTagCreatedNoNode { name: HtmlShellElementName },
    /// The trigger token was handed to a different actual insertion mode
    /// without being consumed. TC-S2's accepted `AfterBody -> InBody`
    /// recovery makes this a same-token move to a mode that is not
    /// necessarily later; reprocessing still keeps one token as one
    /// observation.
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
/// Tree diagnostics are authored-input evidence. They are independent of
/// effective completion: a `Complete` result normally carries at least the
/// missing-DOCTYPE diagnostic, and neither TC-S3 diagnostic forces
/// incompleteness either. They are also independent of the retained
/// tokenizer run's own diagnostics, which are never copied here.
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
}

/// A capability boundary reached by admitted input.
///
/// Unsupported coverage is *not* evidence that the source is invalid HTML,
/// and it is not a tokenizer condition. It records that this subsystem's
/// proved action set does not contain the reached rule.
///
/// Variants are frozen evidence: a successor adds its own rather than
/// widening or renaming an existing one, so predecessor results keep saying
/// exactly what they always said.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeCapability {
    /// A tag naming an element outside the proved `html`/`head`/`body` shell.
    ///
    /// This is frozen predecessor meaning and keeps it exactly: it is reported
    /// for a name in neither closed admitted domain. A selected ordinary tag
    /// never reaches it, and TC-S3 added its own variants below rather than
    /// widening this one.
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
    /// `in body`. TC-S3 proves the selected `div` rules only there, so the
    /// tag is refused before any shell walk, recovery, missing-DOCTYPE, mode,
    /// action, coverage, or identity effect.
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

/// Why an effective TC-S1 result is not `Complete`.
#[derive(Debug, Clone)]
pub(crate) enum HtmlTreeIncompleteCause {
    /// TC-S1 processed every emitted token it was given, but the retained
    /// tokenizer run is itself incomplete.
    ///
    /// The exact lower-layer meaning — `UnsupportedCapability`,
    /// `ResourceLimit`, `InvalidConfiguration`, or
    /// `InternalInvariantFailure` — remains authoritative on
    /// [`HtmlDocumentShellAnalysis::tokenizer_run`] and is deliberately not
    /// duplicated, re-encoded, or lossily summarized here.
    LowerLayerIncomplete,
    /// TC-S1 stopped before mutation at input outside its proved envelope.
    ///
    /// The retained tokenizer run's own completion remains separately
    /// authoritative and may additionally be incomplete.
    UnsupportedCapability(HtmlTreeUnsupportedCapability),
}

/// Effective TC-S1 completion.
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
/// coverage and the two must not be conflated. TC-S1 may commit strictly less
/// than the tokenizer processed.
#[derive(Clone)]
pub(crate) struct HtmlTreeCommittedCoverage {
    committed_prefix: SourceAnchor,
    processed_tokens: usize,
}

impl HtmlTreeCommittedCoverage {
    /// The retained-source prefix whose emitted tokens were completely
    /// processed by committed TC-S1 actions.
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

/// The immutable, validated TC-S1 analysis.
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

    /// Supported TC-S1 parse diagnostics, in committed order.
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
/// boundary produced something it must never produce, not that the authored
/// source was bad or that a capability is missing.
///
/// Every variant carries only structural evidence — constructed identities,
/// roles, counts, [`SourceId`], and [`SourceRangeError`]. `Debug` and
/// `Display` never expose arbitrary authored source content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlTreeFreezeError {
    /// Two stored nodes carry the same constructed identity.
    DuplicateConstructedIdentity(HtmlConstructedNodeId),
    /// The stored node inventory does not match the identities the session's
    /// committed creation counter admitted.
    CreationEventInventoryMismatch { admitted: usize, stored: usize },
    /// A stored identity was never admitted by the creation counter.
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
    /// The stored nodes are not exactly the nodes reachable from the root.
    UnreachableOrCyclicStructure { reachable: usize, stored: usize },
    /// A text node has no contributions, an empty contribution, contributions
    /// that are not in strictly increasing source order, or interpreted text
    /// that is not the exact ordered concatenation of its contributions.
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
    /// Retained evidence revalidated, but the exact supplied [`SourceText`]
    /// carries different content at that range.
    MismatchedSourceEvidence { role: HtmlTreeEvidenceRole },
    /// A node's raw tag-name evidence is not contained in its complete
    /// authored start-tag evidence.
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
    /// One retained selected ordinary end token supplied more than one
    /// terminal selected-end decision.
    ///
    /// One dispatch of one authored selected end tag reaches exactly one of
    /// the three terminal cells: a current-target matching closure, a
    /// recovery group's target closure, or an ignored unmatched disposition.
    /// A group may carry many ordered recovery pops before its one closure,
    /// but the token is spent once that closure or disposition commits. A
    /// replay that spends the same retained token twice — for example to
    /// close a second, further-out same-name ancestor after the first group
    /// already succeeded — describes semantics no single dispatch can
    /// produce.
    DuplicateSelectedOrdinaryEndTokenDecision { token_index: usize },
    /// A recorded ignored unmatched selected ordinary end tag does not
    /// resolve, in the retained tokenizer run, to the exact emitted end tag
    /// of its own recorded selected name.
    UnmatchedSelectedOrdinaryEndTriggerIsNotTheMatchingEndTag { token_index: usize },
    /// A selected ordinary end tag was recorded as unmatched while a
    /// same-name selected ordinary element was in fact still open at that
    /// point in the replayed lifecycle. That is the closing or recovering
    /// cell, not the ignored one.
    UnmatchedSelectedOrdinaryEndTagWithOpenTarget(HtmlConstructedNodeId),
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

/// Validates the private session's construction output and freezes it into an
/// immutable [`HtmlDocumentShellAnalysis`].
///
/// This is the only way an [`HtmlDocumentShellAnalysis`] is created. Every
/// durable invariant this subsystem promises is checked here rather than
/// assumed from how the session happens to be written.
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

/// Validates the complete selected ordinary lifecycle-evidence theorem.
///
/// Replays the committed selected ordinary insertion, heterogeneous
/// recovery-pop, and matching-closure actions in committed order over a
/// private stack of semantic constructed identities, correlating each
/// recovery and closure against the retained tokenizer run, then against the
/// committed diagnostics, and finally against the session's own actual final
/// open selected state. Together that proves every part of the theorem:
///
/// - every insertion, recovery, and closure endpoint resolves to a stored
///   selected ordinary element, and every closure subject carries the
///   recorded name;
/// - a selected ordinary identity is inserted at most once, so the replay
///   stack cannot be padded with a repeated identity;
/// - a closure trigger resolves, in the retained run, to the exact emitted
///   matching end tag for that element — not merely to some valid authored
///   anchor — so a start tag, an unrelated token, or end of file can never
///   stand in as closure evidence;
/// - a recovery trigger resolves the same way to the exact emitted matching
///   end tag of the recovery's own *target*, which is what keeps one authored
///   end tag able to cause several ordered pops plus one closure without
///   becoming any of those nodes' authored origin;
/// - a recorded recovery target really is the nearest currently-open selected
///   ordinary element of its own name, recomputed here from the replayed
///   stack rather than trusted from the recorded field;
/// - recovery and closure order is stack-consistent (LIFO) for the selected
///   slice, so a skipped intervening element, a reversed suffix, a duplicate
///   or extra pop, a pop after the element already left the open state, and a
///   non-current target closed without its required recovery are all
///   rejected;
/// - a recovery group is contiguous and is terminated by exactly its own
///   target's closure under exactly its own trigger token, so an intervening
///   element can never receive a fabricated matching closure and the target
///   can never be recovery-popped instead of closed;
/// - the misnested diagnostics are exactly one per committed recovery group,
///   with that group's own exact authored end tag — recorrelated against the
///   retained run, not merely against the recorded token index — and the
///   accepted recovery summary,
///   and the ignored unmatched end-tag actions and diagnostics name exactly
///   the same trigger tokens; and
/// - the identities still open after the replay are exactly, and in the same
///   order as, the session's actual final open selected ordinary elements.
///
/// That last comparison is what makes this validation of construction output
/// rather than trust in how the session happens to be written: a session that
/// popped a selected element without recording its recovery or closure would
/// otherwise be indistinguishable from the valid end-of-file-open case.
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
    // One per group, in the same order, carrying the accepted recovery
    // summary and — checked against the retained run rather than against the
    // recorded token index alone — that group's own exact authored end tag.
    let paired =
        group_tokens == misnested_tokens
            && recovery_groups
                .iter()
                .zip(&misnested)
                .all(|(group, found)| {
                    found.recovery()
                == HtmlTreeRecovery::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget
                && is_matching_end_tag_trigger(group.target_name, found.trigger(), tokenizer_run)
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
    let Some(boundary) = trigger.authored_boundary() else {
        return false;
    };
    let Some(HtmlToken::Tag(tag)) = tokenizer_run.tokens().get(trigger.token_index()) else {
        return false;
    };
    if !matches!(tag.kind(), HtmlTagKind::End) {
        return false;
    }
    if tag.name().interpreted() != name.interpreted() {
        return false;
    }
    // The recorded anchor must be that token's own complete-tag evidence, not
    // merely an anchor that happens to revalidate.
    let complete = tag.complete();
    boundary.source_id() == complete.source_id()
        && boundary.range() == complete.range()
        && boundary.fragment() == complete.fragment()
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
/// shell every effective `Complete` TC-S1 result must contain.
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
