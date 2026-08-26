//! Immutable, validated result meaning for TC-S1 and its accepted TC-S2,
//! TC-S3, TC-S4, and TC-S5 successors.
//!
//! This module owns the durable half of the accepted Candidate C model: the
//! frozen tree, constructed identity, authored/synthesized provenance,
//! selective action and diagnostic evidence, committed tree coverage, and
//! effective completion. It owns no mutable construction state and never
//! observes the tokenizer or the private [`session`](super::session).
//!
//! TC-S5 keeps the accepted authored-only `Div | Section` selected ordinary
//! domain closed and adds a separate Paragraph domain. That distinction is
//! load-bearing: a Paragraph may either originate in its own authored `<p>`
//! start tag or be synthesized by an unmatched authored `</p>` rule. A
//! synthesized Paragraph therefore has no authored start-tag source at all;
//! the end tag remains trigger / diagnostic / closure evidence only.
//!
//! Everything here is crate-private. No item is `pub`, serialized, or
//! promised across results, runs, source edits, or implementation revisions.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceRangeError, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::result::{HtmlTokenizerCompletion, HtmlTokenizerRunResult};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct HtmlConstructedNodeId(u32);

impl fmt::Debug for HtmlConstructedNodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "HtmlConstructedNodeId(creation#{})", self.0)
    }
}

pub(super) struct HtmlConstructedIdentityCounter {
    next: u32,
}

impl HtmlConstructedIdentityCounter {
    pub(super) const fn new() -> Self {
        Self { next: 0 }
    }

    pub(super) const fn reserve(&self) -> Option<HtmlConstructedNodeId> {
        match self.next.checked_add(1) {
            Some(_) => Some(HtmlConstructedNodeId(self.next)),
            None => None,
        }
    }

    pub(super) fn commit(&mut self, reserved: HtmlConstructedNodeId) {
        debug_assert_eq!(reserved.0, self.next);
        self.next = self.next.saturating_add(1);
    }

    pub(super) const fn admitted(&self) -> u32 {
        self.next
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlShellElementName {
    Html,
    Head,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlSynthesisCause {
    ImpliedByDocumentStructure,
}

#[derive(Clone)]
pub(crate) enum HtmlShellElementOrigin {
    Authored {
        complete: SourceAnchor,
        raw_name: SourceAnchor,
    },
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

/// The accepted authored-only ordinary domain remains exactly `Div | Section`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlSelectedOrdinaryElementName {
    Div,
    Section,
}

impl HtmlSelectedOrdinaryElementName {
    const fn interpreted(self) -> &'static str {
        match self {
            Self::Div => "div",
            Self::Section => "section",
        }
    }
}

/// A selected ordinary element is authored-only. TC-S5 deliberately does not
/// weaken this contract in order to represent Paragraph synthesis.
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

    pub(crate) fn complete(&self) -> &SourceAnchor {
        &self.complete
    }

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

/// Why a Paragraph with no authored start tag exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlParagraphSynthesisCause {
    /// The Standard's unmatched `</p>` rule inserted a source-less P before
    /// immediately closing it.
    UnmatchedParagraphEndTag,
}

/// Where a Paragraph node's existence comes from.
#[derive(Clone)]
pub(crate) enum HtmlParagraphElementOrigin {
    Authored {
        complete: SourceAnchor,
        raw_name: SourceAnchor,
    },
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

/// A constructed HTML `p` element in the bounded TC-S5 domain.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlElementName {
    Shell(HtmlShellElementName),
    SelectedOrdinary(HtmlSelectedOrdinaryElementName),
    Paragraph,
}

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

    pub(crate) fn shell(&self) -> Option<&HtmlShellElement> {
        match self {
            Self::Shell(element) => Some(element),
            Self::SelectedOrdinary(_) | Self::Paragraph(_) => None,
        }
    }

    pub(crate) fn selected_ordinary(&self) -> Option<&HtmlSelectedOrdinaryElement> {
        match self {
            Self::SelectedOrdinary(element) => Some(element),
            Self::Shell(_) | Self::Paragraph(_) => None,
        }
    }

    pub(crate) fn paragraph(&self) -> Option<&HtmlParagraphElement> {
        match self {
            Self::Paragraph(element) => Some(element),
            Self::Shell(_) | Self::SelectedOrdinary(_) => None,
        }
    }
}

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

#[derive(Debug, Clone)]
pub(crate) enum HtmlTreeNodeKind {
    Document,
    Element(HtmlElement),
    Text(HtmlTextNode),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum HtmlAuthoredSource<'node> {
    StartTag {
        complete: &'node SourceAnchor,
        raw_name: &'node SourceAnchor,
    },
    Characters(&'node [HtmlTextContribution]),
}

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

    pub(crate) fn children(&self) -> &[HtmlConstructedNodeId] {
        &self.children
    }

    pub(crate) fn kind(&self) -> &HtmlTreeNodeKind {
        &self.kind
    }

    pub(crate) fn authored_source(&self) -> Option<HtmlAuthoredSource<'_>> {
        match &self.kind {
            HtmlTreeNodeKind::Document => None,
            HtmlTreeNodeKind::Element(HtmlElement::Shell(shell)) => match shell.origin() {
                HtmlShellElementOrigin::Authored { complete, raw_name } => {
                    Some(HtmlAuthoredSource::StartTag { complete, raw_name })
                }
                HtmlShellElementOrigin::Synthesized(_) => None,
            },
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

    pub(crate) fn token_index(&self) -> usize {
        self.token_index
    }

    pub(crate) fn kind(&self) -> &HtmlTreeTriggerKind {
        &self.kind
    }

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
    Authored(SourceAnchor),
    EndOfFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlShellClosure {
    AuthoredEndTag,
    ImpliedByToken,
}

/// The distinct bounded TC-S5 reasons a Paragraph leaves the open stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlParagraphClosure {
    MatchingEndTag,
    StartTriggered,
    UnmatchedEndTagSynthesized,
}

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

#[derive(Debug, Clone)]
pub(crate) enum HtmlTreeActionKind {
    InsertedAuthoredShellElement {
        node: HtmlConstructedNodeId,
        name: HtmlShellElementName,
    },
    InsertedSynthesizedShellElement {
        node: HtmlConstructedNodeId,
        name: HtmlShellElementName,
        cause: HtmlSynthesisCause,
    },
    InsertedTextNode { node: HtmlConstructedNodeId },
    AppendedToTextNode { node: HtmlConstructedNodeId },
    InsertedAuthoredSelectedOrdinaryElement {
        node: HtmlConstructedNodeId,
        name: HtmlSelectedOrdinaryElementName,
    },
    ClosedSelectedOrdinaryElement {
        node: HtmlConstructedNodeId,
        name: HtmlSelectedOrdinaryElementName,
    },
    PoppedSelectedOrdinaryElementByAncestorEndTag {
        node: HtmlConstructedNodeId,
        target: HtmlConstructedNodeId,
    },
    IgnoredUnmatchedSelectedOrdinaryEndTag {
        name: HtmlSelectedOrdinaryElementName,
    },
    /// Authored Paragraph insertion. The trigger is also its authored origin.
    InsertedAuthoredParagraphElement { node: HtmlConstructedNodeId },
    /// Source-less Paragraph insertion caused by an unmatched authored `</p>`.
    InsertedSynthesizedParagraphElement {
        node: HtmlConstructedNodeId,
        cause: HtmlParagraphSynthesisCause,
    },
    /// A Paragraph left the open stack for one of the three validated causes.
    ClosedParagraphElement {
        node: HtmlConstructedNodeId,
        closure: HtmlParagraphClosure,
    },
    ClosedShellElement {
        node: HtmlConstructedNodeId,
        name: HtmlShellElementName,
        closure: HtmlShellClosure,
    },
    AcknowledgedShellEndTag { name: HtmlShellElementName },
    DuplicateShellStartTagCreatedNoNode { name: HtmlShellElementName },
    ReprocessedToken,
    StoppedParsing,
}

impl HtmlTreeActionKind {
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
            | Self::ClosedShellElement { node, .. } => Some(*node),
            Self::IgnoredUnmatchedSelectedOrdinaryEndTag { .. }
            | Self::AcknowledgedShellEndTag { .. }
            | Self::DuplicateShellStartTagCreatedNoNode { .. }
            | Self::ReprocessedToken
            | Self::StoppedParsing => None,
        }
    }
}

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
    MissingDoctype,
    DuplicateHeadStartTag,
    DuplicateBodyStartTag,
    AfterBodyCharacterData,
    UnmatchedSelectedOrdinaryEndTag,
    MisnestedSelectedOrdinaryEndTag,
    OpenSelectedOrdinaryElementAtEndOfFile,
    /// An authored `</p>` appeared while no P was in the bounded button scope.
    UnmatchedParagraphEndTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeRecovery {
    ContinuedInQuirksDocumentMode,
    DuplicateShellStartTagProducedNoNode,
    SwitchedToInBodyAndReprocessedSameToken,
    IgnoredToken,
    PoppedInterveningSelectedOrdinaryElementsAndClosedTarget,
    StoppedParsingWithOpenSelectedOrdinaryElements,
    /// The unmatched `</p>` rule diagnosed the token, inserted one source-less
    /// Paragraph under the current insertion parent, and closed that same node.
    SynthesizedParagraphElementAndClosedIt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeCapability {
    NonShellElementTag,
    ShellTagAttribute,
    SelfClosingShellTag,
    WhitespaceSensitiveCharacterData,
    UnprovedCharacterDataPosition,
    UnprovedShellStartTagPosition,
    UnprovedShellEndTagPosition,
    SelectedOrdinaryTagOutsideInBody,
    ShellTagWithOpenSelectedOrdinaryElement,
    SelectedOrdinaryTagAttribute,
    SelfClosingSelectedOrdinaryTag,
    /// A Paragraph tag reached an actual insertion mode other than `InBody`.
    ParagraphTagOutsideInBody,
    /// Attribute evidence on a Paragraph start or end tag.
    ParagraphTagAttribute,
    /// A self-closing solidus on a Paragraph tag.
    SelfClosingParagraphTag,
    /// A selected `</div>` / `</section>` end tag reached `InBody` with P
    /// current. The validated non-no-op implied-end cell remains excluded.
    SelectedOrdinaryEndTagWithOpenParagraphElement,
    /// A shell tag reached `InBody` with P current. Shell/P crossings remain
    /// outside TC-S5.
    ShellTagWithOpenParagraphElement,
}

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

    pub(crate) fn trigger(&self) -> &HtmlTreeTokenTrigger {
        &self.trigger
    }
}

#[derive(Debug, Clone)]
pub(crate) enum HtmlTreeIncompleteCause {
    LowerLayerIncomplete,
    UnsupportedCapability(HtmlTreeUnsupportedCapability),
}

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

#[derive(Clone)]
pub(crate) struct HtmlTreeCommittedCoverage {
    committed_prefix: SourceAnchor,
    processed_tokens: usize,
}

impl HtmlTreeCommittedCoverage {
    pub(crate) fn committed_prefix(&self) -> &SourceAnchor {
        &self.committed_prefix
    }

    pub(crate) fn committed_end(&self) -> usize {
        self.committed_prefix.range().end()
    }

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
    pub(crate) fn tokenizer_run(&self) -> &HtmlTokenizerRunResult {
        &self.tokenizer_run
    }

    pub(crate) fn root(&self) -> HtmlConstructedNodeId {
        self.root
    }

    pub(crate) fn node(&self, id: HtmlConstructedNodeId) -> Option<&HtmlTreeNode> {
        self.nodes.iter().find(|node| node.id() == id)
    }

    pub(crate) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn nodes_in_creation_order(&self) -> Vec<&HtmlTreeNode> {
        let mut ordered: Vec<&HtmlTreeNode> = self.nodes.iter().collect();
        ordered.sort_by_key(|node| node.id());
        ordered
    }

    pub(crate) fn diagnostics(&self) -> &[HtmlTreeDiagnostic] {
        &self.diagnostics
    }

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

pub(super) struct HtmlDocumentShellParts {
    pub(super) nodes: Vec<HtmlTreeNode>,
    pub(super) root: HtmlConstructedNodeId,
    pub(super) admitted_creation_events: u32,
    pub(super) diagnostics: Vec<HtmlTreeDiagnostic>,
    pub(super) actions: Vec<HtmlTreeAction>,
    pub(super) processed_tokens: usize,
    pub(super) committed_prefix_end: usize,
    pub(super) completion: HtmlTreeCompletion,
    pub(super) final_open_selected_ordinary: Vec<HtmlConstructedNodeId>,
    /// The Paragraph still open on the private session stack at hand-off, if
    /// any. Under TC-S5 there can be at most one and, when present, it is the
    /// current node. This immutable checkpoint is consumed by freeze and never
    /// reaches the analysis consumer.
    pub(super) final_open_paragraph: Option<HtmlConstructedNodeId>,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeCompletionUpgrade {
    RetainedTokenizerRunIsIncomplete,
    EmittedTokensRemainUnprocessed,
    DocumentShellIsIncomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlTreeFreezeError {
    DuplicateConstructedIdentity(HtmlConstructedNodeId),
    CreationEventInventoryMismatch { admitted: usize, stored: usize },
    UnadmittedConstructedIdentity(HtmlConstructedNodeId),
    MissingRootNode(HtmlConstructedNodeId),
    InvalidDocumentRoot(HtmlConstructedNodeId),
    RootMustNotHaveParent(HtmlConstructedNodeId),
    MissingParentRelationship(HtmlConstructedNodeId),
    UnresolvedRelationship {
        from: HtmlConstructedNodeId,
        to: HtmlConstructedNodeId,
    },
    AsymmetricRelationship {
        parent: HtmlConstructedNodeId,
        child: HtmlConstructedNodeId,
    },
    ChildPrecedesParentCreation {
        parent: HtmlConstructedNodeId,
        child: HtmlConstructedNodeId,
    },
    UnreachableOrCyclicStructure { reachable: usize, stored: usize },
    InvalidTextContributions(HtmlConstructedNodeId),
    ForeignSourceEvidence {
        role: HtmlTreeEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    InvalidSourceEvidence {
        role: HtmlTreeEvidenceRole,
        error: SourceRangeError,
    },
    MismatchedSourceEvidence { role: HtmlTreeEvidenceRole },
    AuthoredNameOutsideCompleteTag(HtmlConstructedNodeId),
    UnresolvedActionSubject(HtmlConstructedNodeId),
    InvalidTokenProgression {
        role: HtmlTreeEvidenceRole,
        token_index: usize,
    },
    InvalidCommittedCoverage {
        committed_prefix_end: usize,
        source_len: usize,
    },
    CommittedTokensExceedRetainedRun {
        processed_tokens: usize,
        emitted_tokens: usize,
    },
    CompletionUpgrade(HtmlTreeCompletionUpgrade),
    UnsupportedTriggerLeakedAsAuthoredOrigin(HtmlConstructedNodeId),
    ClosureSubjectIsNotTheSelectedOrdinaryElement {
        node: HtmlConstructedNodeId,
        name: HtmlSelectedOrdinaryElementName,
    },
    FabricatedSelectedOrdinaryClosure(HtmlConstructedNodeId),
    NonLifoSelectedOrdinaryClosure(HtmlConstructedNodeId),
    ClosureTriggerIsNotTheMatchingEndTag {
        node: HtmlConstructedNodeId,
        token_index: usize,
    },
    DuplicateSelectedOrdinaryInsertion(HtmlConstructedNodeId),
    FinalOpenSelectedOrdinaryIsNotASelectedElement(HtmlConstructedNodeId),
    FinalOpenSelectedOrdinaryStateMismatch {
        replayed: Vec<HtmlConstructedNodeId>,
        actual: Vec<HtmlConstructedNodeId>,
    },
    RecoverySubjectIsNotSelectedOrdinaryElement(HtmlConstructedNodeId),
    RecoveryTargetIsNotSelectedOrdinaryElement(HtmlConstructedNodeId),
    SelfTargetingSelectedOrdinaryRecovery(HtmlConstructedNodeId),
    RecoveryTriggerIsNotMatchingTargetEndTag {
        node: HtmlConstructedNodeId,
        token_index: usize,
    },
    RecoveryTargetIsNotNearestMatchingSelectedOrdinary(HtmlConstructedNodeId),
    NonLifoSelectedOrdinaryRecovery(HtmlConstructedNodeId),
    UnterminatedSelectedOrdinaryRecovery(HtmlConstructedNodeId),
    SelectedOrdinaryRecoveryClosureMismatch {
        target: HtmlConstructedNodeId,
        closed: HtmlConstructedNodeId,
    },
    SelectedOrdinaryRecoveryDiagnosticMismatch {
        recovery_groups: Vec<usize>,
        misnested_diagnostics: Vec<usize>,
    },
    UnmatchedSelectedOrdinaryEndTagDiagnosticMismatch {
        actions: Vec<usize>,
        diagnostics: Vec<usize>,
    },
    DuplicateSelectedOrdinaryEndTokenDecision { token_index: usize },
    UnmatchedSelectedOrdinaryEndTriggerIsNotTheMatchingEndTag { token_index: usize },
    UnmatchedSelectedOrdinaryEndTagWithOpenTarget(HtmlConstructedNodeId),
    /// A Paragraph action names a stored node that is not a Paragraph.
    ParagraphActionSubjectIsNotParagraph(HtmlConstructedNodeId),
    /// The same Paragraph identity is inserted twice in the durable action stream.
    DuplicateParagraphInsertion(HtmlConstructedNodeId),
    /// A stored Paragraph has no exactly-one insertion action.
    ParagraphInsertionInventoryMismatch(HtmlConstructedNodeId),
    /// An authored Paragraph insertion is not correlated to its exact retained
    /// authored `<p>` start token and source origin.
    ParagraphAuthoredInsertionTriggerMismatch {
        node: HtmlConstructedNodeId,
        token_index: usize,
    },
    /// A synthesized Paragraph insertion is not the exact source-less node
    /// caused by its retained unmatched authored `</p>` trigger.
    ParagraphSynthesizedInsertionTriggerMismatch {
        node: HtmlConstructedNodeId,
        token_index: usize,
    },
    /// A Paragraph closure has an invalid trigger, cause, or source relation.
    ParagraphClosureTriggerMismatch {
        node: HtmlConstructedNodeId,
        token_index: usize,
    },
    /// A Paragraph closure or selected-ordinary removal was not current in the
    /// replayed mixed selected/Paragraph open stack.
    NonLifoParagraphInteraction(HtmlConstructedNodeId),
    /// A start-triggered Paragraph close is not immediately followed by the
    /// start-token insertion that validated TC-S5 requires.
    ParagraphStartTriggeredInsertionMismatch { token_index: usize },
    /// An unmatched-P synthesized insertion is not immediately closed by the
    /// same end tag under the unmatched-synthesized closure cause.
    ParagraphSynthesisClosureMismatch { token_index: usize },
    /// Unmatched P diagnostics are not exactly one per synthesized Paragraph
    /// action with the same retained end-tag trigger and recovery summary.
    UnmatchedParagraphDiagnosticMismatch {
        syntheses: Vec<usize>,
        diagnostics: Vec<usize>,
    },
    /// The session's final-open Paragraph snapshot does not resolve to P.
    FinalOpenParagraphIsNotParagraph(HtmlConstructedNodeId),
    /// Freeze replay and the actual session checkpoint disagree about whether
    /// and which Paragraph remains open.
    FinalOpenParagraphStateMismatch {
        replayed: Option<HtmlConstructedNodeId>,
        actual: Option<HtmlConstructedNodeId>,
    },
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

fn validate_selected_ordinary_lifecycle(
    nodes: &[HtmlTreeNode],
    actions: &[HtmlTreeAction],
    diagnostics: &[HtmlTreeDiagnostic],
    tokenizer_run: &HtmlTokenizerRunResult,
    final_open_selected_ordinary: &[HtmlConstructedNodeId],
) -> Result<(), HtmlTreeFreezeError> {
    let mut open: Vec<HtmlConstructedNodeId> = Vec::new();
    let mut inserted: Vec<HtmlConstructedNodeId> = Vec::new();
    let mut pending: Option<(HtmlConstructedNodeId, usize)> = None;
    let mut recovery_groups: Vec<ReplayedRecoveryGroup> = Vec::new();
    let mut ignored_unmatched: Vec<ReplayedUnmatchedEnd> = Vec::new();
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
                        if pending_target != *target
                            || pending_token != action.trigger().token_index()
                        {
                            return Err(HtmlTreeFreezeError::UnterminatedSelectedOrdinaryRecovery(
                                pending_target,
                            ));
                        }
                    }
                    None => {
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
                spend_end_token(&mut spent_end_tokens, action.trigger().token_index())?;
            }
            HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag { name } => {
                reject_interleaved_recovery(pending)?;
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

struct ReplayedRecoveryGroup {
    trigger_token: usize,
    target_name: HtmlSelectedOrdinaryElementName,
}

struct ReplayedUnmatchedEnd {
    trigger_token: usize,
    name: HtmlSelectedOrdinaryElementName,
}

fn spend_end_token(spent: &mut Vec<usize>, token_index: usize) -> Result<(), HtmlTreeFreezeError> {
    if spent.contains(&token_index) {
        return Err(HtmlTreeFreezeError::DuplicateSelectedOrdinaryEndTokenDecision { token_index });
    }
    spent.push(token_index);
    Ok(())
}

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

fn validate_closure_trigger(
    node: HtmlConstructedNodeId,
    name: HtmlSelectedOrdinaryElementName,
    trigger: &HtmlTreeTokenTrigger,
    tokenizer_run: &HtmlTokenizerRunResult,
) -> Result<(), HtmlTreeFreezeError> {
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

fn is_matching_end_tag_trigger(
    name: HtmlSelectedOrdinaryElementName,
    trigger: &HtmlTreeTokenTrigger,
    tokenizer_run: &HtmlTokenizerRunResult,
) -> bool {
    is_exact_tag_trigger(trigger, tokenizer_run, HtmlTagKind::End, &[name.interpreted()])
}

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

/// Independently replays the mixed selected-ordinary / Paragraph lifecycle.
///
/// This is intentionally not a second parser. It validates only committed
/// durable relations against retained tokenizer evidence and the session's
/// immutable final-open Paragraph checkpoint.
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
                // If a Paragraph were still open here it would no longer be
                // current after this push, contradicting the TC-S5 theorem.
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
                    return Err(HtmlTreeFreezeError::ParagraphActionSubjectIsNotParagraph(*node));
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
                    return Err(HtmlTreeFreezeError::ParagraphAuthoredInsertionTriggerMismatch {
                        node: *node,
                        token_index: action.trigger().token_index(),
                    });
                }
                inserted.push(*node);
                open_content.push(*node);
            }
            HtmlTreeActionKind::InsertedSynthesizedParagraphElement { node, cause } => {
                let Some(element) = paragraph(nodes, *node) else {
                    return Err(HtmlTreeFreezeError::ParagraphActionSubjectIsNotParagraph(*node));
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
                    || !is_exact_tag_trigger(action.trigger(), tokenizer_run, HtmlTagKind::End, &["p"])
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
            HtmlTreeActionKind::ClosedParagraphElement { node, closure } => {
                if paragraph(nodes, *node).is_none() {
                    return Err(HtmlTreeFreezeError::ParagraphActionSubjectIsNotParagraph(*node));
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
                    let next_matches = match (expected, next.kind()) {
                        (Some("p"), HtmlTreeActionKind::InsertedAuthoredParagraphElement { .. }) => {
                            true
                        }
                        (
                            Some("div"),
                            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                                name: HtmlSelectedOrdinaryElementName::Div,
                                ..
                            },
                        ) => true,
                        (
                            Some("section"),
                            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                                name: HtmlSelectedOrdinaryElementName::Section,
                                ..
                            },
                        ) => true,
                        _ => false,
                    };
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
        if inserted.iter().filter(|inserted| **inserted == node).count() != 1 {
            return Err(HtmlTreeFreezeError::ParagraphInsertionInventoryMismatch(node));
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
                .expect("non-empty invalid paragraph state"),
        ));
    }
    let replayed = replayed_paragraphs.first().copied();
    if let Some(actual) = final_open_paragraph
        && paragraph(nodes, actual).is_none()
    {
        return Err(HtmlTreeFreezeError::FinalOpenParagraphIsNotParagraph(actual));
    }
    if replayed != final_open_paragraph {
        return Err(HtmlTreeFreezeError::FinalOpenParagraphStateMismatch {
            replayed,
            actual: final_open_paragraph,
        });
    }
    Ok(())
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
    if tag.kind() != HtmlTagKind::Start || tag.name().interpreted() != "p" {
        return false;
    }
    exact_anchor(trigger.authored_boundary(), Some(tag.complete()))
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
