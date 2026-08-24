//! Immutable, validated TC-S1 result meaning.
//!
//! This module owns the durable half of the accepted Candidate C model: the
//! frozen tree, constructed identity, authored/synthesized provenance,
//! selective action and diagnostic evidence, committed tree coverage, and
//! effective completion. It owns no mutable construction state and never
//! observes the tokenizer or the private
//! [`session`](super::session).
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
    Element(HtmlShellElement),
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
            HtmlTreeNodeKind::Element(element) => match element.origin() {
                HtmlShellElementOrigin::Authored { complete, raw_name } => {
                    Some(HtmlAuthoredSource::StartTag { complete, raw_name })
                }
                HtmlShellElementOrigin::Synthesized(_) => None,
            },
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

/// One committed TC-S1 action, with the token that triggered it.
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

/// The selective committed-action vocabulary TC-S1 proves.
///
/// This is deliberately not a complete construction event log: it records only
/// what a supported TC-S1 query needs in order to explain a durable
/// observation.
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
    /// The trigger token was handed to a later insertion mode without being
    /// consumed. Reprocessing keeps one token as one observation.
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
            | Self::ClosedShellElement { node, .. } => Some(*node),
            Self::AcknowledgedShellEndTag { .. }
            | Self::DuplicateShellStartTagCreatedNoNode { .. }
            | Self::ReprocessedToken
            | Self::StoppedParsing => None,
        }
    }
}

/// A supported TC-S1 parse diagnostic.
///
/// Tree diagnostics are authored-input evidence. They are independent of
/// effective completion: a `Complete` TC-S1 result normally carries at least
/// the missing-DOCTYPE diagnostic. They are also independent of the retained
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeRecovery {
    /// Construction continued; the private document mode became quirks.
    ContinuedInQuirksDocumentMode,
    /// The duplicate shell start tag produced no node and no identity.
    DuplicateShellStartTagProducedNoNode,
}

/// A TC-S1 capability boundary reached by admitted input.
///
/// Unsupported coverage is *not* evidence that the source is invalid HTML,
/// and it is not a tokenizer condition. It records that TC-S1's proved action
/// set does not contain the reached rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeCapability {
    /// A tag naming an element outside the proved `html`/`head`/`body` shell.
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
}

/// The exact typed evidence for a TC-S1 unsupported stop.
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

/// A TC-S1 freeze/boundary invariant failure.
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
/// durable invariant TC-S1 promises is checked here rather than assumed from
/// how the session happens to be written.
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
    } = parts;

    validate_identity_inventory(&nodes, admitted_creation_events)?;
    validate_structure(&nodes, root)?;
    validate_node_evidence(source, &nodes)?;
    validate_action_evidence(source, &nodes, &actions, tokenizer_run.tokens().len())?;
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
            HtmlTreeNodeKind::Element(element) => match element.origin() {
                HtmlShellElementOrigin::Authored { complete, raw_name } => {
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
                HtmlShellElementOrigin::Synthesized(_) => {}
            },
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
    matches!(node.kind(), HtmlTreeNodeKind::Element(element) if element.name() == name)
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
