//! Public selected HTML document-construction consumer facade (#864, ADR 0011).
//!
//! This is the single intentional public analysis boundary over the existing
//! crate-private HTML tree-construction pipeline:
//!
//! ```text
//! &SourceText
//!   -> fixed private HtmlTokenizerLimits
//!   -> existing construct_html_document_shell
//!   -> validated HtmlDocumentShellAnalysis
//!   -> fallible owned HtmlTreeReport
//! ```
//!
//! The facade composes the existing producer under one fixed private document
//! profile (ordinary document, scripting disabled, tokenizer initial state
//! Data, no fragment/context mode). The execution envelope below is Core
//! consumer policy approved in Issue #864; it is not caller-configurable and
//! not public API.
//!
//! Every report value is projected from evidence already retained by the
//! producer. No source text is searched, rescanned, retokenized, reparsed, or
//! otherwise reconstructed here, and node identity, relationships, and order
//! are never derived from storage position.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

use super::token::{HtmlAttributeValueSyntax, HtmlToken};
use super::tokenizer::diagnostic as tokenizer_diagnostic;
use super::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerResource};
use super::tokenizer::result as tokenizer_result;
use super::tree_construction::driver::construct_html_document_shell;
use super::tree_construction::result as tree_result;

#[cfg(test)]
mod tests;

// Issue #864 fixed HTML tokenizer resource vector, in constructor field
// order. Execution policy only; not HTML semantics or public API.
const SOURCE_BYTES: usize = 36_864;
const TRANSITION_STEPS: usize = 79_872;
const EMITTED_TOKENS: usize = 6_144;
const TOKENIZER_DIAGNOSTICS: usize = 256;
const ATTRIBUTES_PER_TAG: usize = 1;
const RETAINED_INTERPRETED_BYTES: usize = 49_152;
const TEMPORARY_BUFFER_BYTES: usize = 0;

const fn fixed_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(
        SOURCE_BYTES,
        TRANSITION_STEPS,
        EMITTED_TOKENS,
        TOKENIZER_DIAGNOSTICS,
        ATTRIBUTES_PER_TAG,
        RETAINED_INTERPRETED_BYTES,
        TEMPORARY_BUFFER_BYTES,
    )
}

/// Analyzes one HTML source under the fixed selected document-construction
/// profile and returns an owned report.
///
/// Incomplete, unsupported, resource-refused, and diagnostic outcomes are
/// reported through [`HtmlTreeReport`]. An `Err` is only a returned Core
/// boundary failure.
pub fn analyze_selected_document_tree(
    source: &SourceText,
) -> Result<HtmlTreeReport, HtmlTreeCoreFailure> {
    analyze(source, fixed_limits())
}

/// The single composition path. The public entry point always supplies the
/// fixed private vector; validation alone supplies other limits.
fn analyze(
    source: &SourceText,
    limits: HtmlTokenizerLimits,
) -> Result<HtmlTreeReport, HtmlTreeCoreFailure> {
    let analysis = construct_html_document_shell(source, limits)
        .map_err(|_| HtmlTreeCoreFailure::InternalFailure)?;
    project(source.id(), &analysis)
}

/// An owned, immutable selected document-construction report.
#[derive(Debug, Clone)]
pub struct HtmlTreeReport {
    source_id: SourceId,
    root: HtmlTreeNodeId,
    /// Ascending by constructed creation identity. Position carries no
    /// meaning; lookup is by identity.
    nodes: Vec<HtmlTreeNode>,
    completion: HtmlTreeCompletion,
    coverage: HtmlTreeCoverage,
    tokenizer_diagnostics: Vec<HtmlTokenizerDiagnostic>,
    tree_diagnostics: Vec<HtmlTreeDiagnostic>,
    /// Validated encounter order. Position carries no meaning beyond that
    /// order.
    selected_ordinary_relations: Vec<HtmlTreeSelectedOrdinaryRelation>,
    /// Validated encounter order of the committed attributed selected
    /// ordinary start tags. Position carries no meaning beyond that order.
    selected_ordinary_attributes: Vec<HtmlTreeSelectedOrdinaryAttribute>,
    /// Validated Paragraph encounter order. A separate slice from the
    /// selected ordinary relations: the two are not one timeline.
    paragraph_relations: Vec<HtmlTreeParagraphRelation>,
}

impl HtmlTreeReport {
    pub fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub fn root(&self) -> HtmlTreeNodeId {
        self.root
    }

    /// Nodes in deterministic constructed-identity order. Final tree order is
    /// the explicit parent/child relationships, not this order.
    pub fn nodes(&self) -> &[HtmlTreeNode] {
        &self.nodes
    }

    pub fn node(&self, id: HtmlTreeNodeId) -> Option<&HtmlTreeNode> {
        self.nodes
            .binary_search_by_key(&id, HtmlTreeNode::id)
            .ok()
            .map(|index| &self.nodes[index])
    }

    pub fn completion(&self) -> &HtmlTreeCompletion {
        &self.completion
    }

    pub fn coverage(&self) -> &HtmlTreeCoverage {
        &self.coverage
    }

    pub fn tokenizer_diagnostics(&self) -> &[HtmlTokenizerDiagnostic] {
        &self.tokenizer_diagnostics
    }

    pub fn tree_diagnostics(&self) -> &[HtmlTreeDiagnostic] {
        &self.tree_diagnostics
    }

    /// Validated close/recovery relations of the selected ordinary elements
    /// (`div`, `section`, `article`, `aside`, `footer`, `header`, `main`,
    /// `nav`), in validated encounter order. For one authored end tag the
    /// current-to-target recovery pops precede the target's matching close.
    ///
    /// Absence of a relation means only that no validated relation of this
    /// selected domain is projected for a node. It does not mean the node is
    /// open at end of file, never closed, implicitly closed, or the target of
    /// an unmatched end tag, and in an incomplete report it says nothing about
    /// unprocessed input. [`Self::completion`] and [`Self::coverage`] remain
    /// the only completeness boundary.
    pub fn selected_ordinary_relations(&self) -> &[HtmlTreeSelectedOrdinaryRelation] {
        &self.selected_ordinary_relations
    }

    /// The one authored attribute of each committed selected ordinary element
    /// (`div`, `section`, `article`, `aside`, `footer`, `header`, `main`,
    /// `nav`) whose complete consumed start tag carried one, in validated
    /// encounter order. At most one record exists per node.
    ///
    /// For a node the caller already knows is a committed selected ordinary
    /// element of this report, a record means its complete consumed start tag
    /// had that one authored attribute, and no record means that start tag had
    /// zero attributes. That reading does not apply to unknown identities,
    /// other element families, synthesized nodes, or input that never
    /// completed an admitted start tag. [`Self::completion`] and
    /// [`Self::coverage`] remain the only completeness boundary; a record
    /// committed before a later stop stays valid.
    pub fn selected_ordinary_attributes(&self) -> &[HtmlTreeSelectedOrdinaryAttribute] {
        &self.selected_ordinary_attributes
    }

    /// Validated close and implied-pop relations of the Paragraph (`p`)
    /// elements, in the Paragraph slice's own validated encounter order. For
    /// one authored selected end tag the Paragraph implied pop is reported
    /// here, while the selected ordinary recovery pops and matching close of
    /// that same tag are reported by [`Self::selected_ordinary_relations`].
    /// Positions in the two slices are not a shared timeline and no
    /// cross-slice chronology is defined.
    ///
    /// Absence of a relation means only that no validated relation of this
    /// selected Paragraph relation domain is projected. It does not mean a
    /// Paragraph is open at end of file, never closed, synthesized,
    /// unmatched, unsupported, unprocessed, or that the report is complete.
    /// [`Self::completion`] and [`Self::coverage`] remain the only
    /// completeness boundary; a relation committed before a later stop stays
    /// valid.
    pub fn paragraph_relations(&self) -> &[HtmlTreeParagraphRelation] {
        &self.paragraph_relations
    }
}

/// The one authored attribute of one committed selected ordinary element.
///
/// Authored evidence (anchors, syntax) and tokenizer-interpreted values are
/// distinct: for example raw U+0000 stays in the anchored source while the
/// interpreted value carries U+FFFD. Interpreted values are the tokenizer's
/// retained values, never recomputed here. Every anchor preserves its
/// [`SourceId`].
#[derive(Debug, Clone)]
pub struct HtmlTreeSelectedOrdinaryAttribute {
    node: HtmlTreeNodeId,
    complete: SourceAnchor,
    authored_name: SourceAnchor,
    value_syntax: HtmlTreeAttributeValueSyntax,
    interpreted_name: String,
    interpreted_value: String,
}

impl HtmlTreeSelectedOrdinaryAttribute {
    /// The report-local identity of the constructed selected ordinary element
    /// whose consumed start tag carried this attribute.
    pub fn node(&self) -> HtmlTreeNodeId {
        self.node
    }

    /// The exact complete authored attribute.
    pub fn complete(&self) -> &SourceAnchor {
        &self.complete
    }

    /// The exact authored attribute-name spelling.
    pub fn authored_name(&self) -> &SourceAnchor {
        &self.authored_name
    }

    pub fn value_syntax(&self) -> &HtmlTreeAttributeValueSyntax {
        &self.value_syntax
    }

    /// The tokenizer-retained normalized attribute name.
    pub fn interpreted_name(&self) -> &str {
        &self.interpreted_name
    }

    /// The tokenizer-retained interpreted attribute value. Empty for
    /// `Missing`, `MissingAfterEquals`, and explicit empty quoted values
    /// alike; the syntax, not this value, distinguishes them.
    pub fn interpreted_value(&self) -> &str {
        &self.interpreted_value
    }
}

/// The exact authored value syntax of one attribute.
///
/// This enum is intentionally closed and exhaustive: the five forms are the
/// whole validated authored-syntax domain of the selected attribute
/// contract. It is deliberately not `#[non_exhaustive]`; a further authored
/// syntax form would need its own evidence and compatibility authority.
#[derive(Debug, Clone)]
pub enum HtmlTreeAttributeValueSyntax {
    /// No `=` and no value.
    Missing,
    /// `=` with no value before the end of the attribute.
    MissingAfterEquals {
        equals: SourceAnchor,
        value_boundary: SourceAnchor,
    },
    Unquoted {
        equals: SourceAnchor,
        value: SourceAnchor,
    },
    DoubleQuoted {
        equals: SourceAnchor,
        open_quote: SourceAnchor,
        value: SourceAnchor,
        close_quote: SourceAnchor,
    },
    SingleQuoted {
        equals: SourceAnchor,
        open_quote: SourceAnchor,
        value: SourceAnchor,
        close_quote: SourceAnchor,
    },
}

/// How a selected ordinary element left the open-element state.
///
/// Subject and target are report-local identities resolvable through
/// [`HtmlTreeReport::node`]; names are read from the resolved nodes. The two
/// meanings are distinct durable facts and are never collapsed.
#[derive(Debug, Clone)]
pub enum HtmlTreeSelectedOrdinaryRelation {
    /// The element's own authored end tag closed it.
    MatchingClose {
        node: HtmlTreeNodeId,
        /// The exact complete authored end tag.
        trigger: SourceAnchor,
    },
    /// No matching end tag of its own caused this pop: an authored end tag of
    /// the enclosing `target` element removed it.
    RecoveryPopByAncestorEndTag {
        node: HtmlTreeNodeId,
        /// The constructed ancestor the authored end tag was aimed at.
        target: HtmlTreeNodeId,
        /// The exact complete authored end tag of `target`.
        trigger: SourceAnchor,
    },
}

/// How a Paragraph element left the open-element state.
///
/// Nodes are report-local identities resolvable through
/// [`HtmlTreeReport::node`]; names are read from the resolved nodes. The four
/// meanings are distinct durable facts and are never collapsed. Every trigger
/// is exact authored source evidence. This enum is intentionally closed: the
/// four forms are the whole validated Paragraph relation domain.
#[derive(Debug, Clone)]
pub enum HtmlTreeParagraphRelation {
    /// The Paragraph's own authored `</p>` closed it.
    MatchingClose {
        node: HtmlTreeNodeId,
        /// The exact complete authored end tag.
        trigger: SourceAnchor,
    },
    /// An authored start tag closed the open Paragraph and inserted a new
    /// element in the same step.
    StartTriggeredClose {
        /// The Paragraph that was closed.
        node: HtmlTreeNodeId,
        /// The Paragraph or selected ordinary element the same start tag
        /// inserted.
        inserted: HtmlTreeNodeId,
        /// The exact complete authored start tag.
        trigger: SourceAnchor,
    },
    /// An unmatched authored `</p>` caused the synthesized Paragraph `node`
    /// to be created and closed. The synthesized insertion itself and the
    /// parse-error classification stay with the node's provenance and the
    /// tree diagnostics.
    SynthesizedCloseByUnmatchedEndTag {
        node: HtmlTreeNodeId,
        /// The exact complete authored unmatched end tag.
        trigger: SourceAnchor,
    },
    /// No `</p>` of its own caused this pop: the authored end tag of the
    /// selected ordinary `target` element removed the open Paragraph.
    ImpliedPopBySelectedOrdinaryEndTag {
        node: HtmlTreeNodeId,
        /// The constructed selected ordinary element the end tag was aimed at.
        target: HtmlTreeNodeId,
        /// The exact complete authored end tag of `target`.
        trigger: SourceAnchor,
    },
}

/// A result-local constructed creation identity.
///
/// The numeric value is the committed semantic creation ordinal within one
/// report. It has no cross-run, cross-edit, cross-revision, storage-index,
/// pointer, source-range, or browser meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HtmlTreeNodeId(u32);

impl HtmlTreeNodeId {
    pub fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct HtmlTreeNode {
    id: HtmlTreeNodeId,
    parent: Option<HtmlTreeNodeId>,
    children: Vec<HtmlTreeNodeId>,
    kind: HtmlTreeNodeKind,
}

impl HtmlTreeNode {
    pub fn id(&self) -> HtmlTreeNodeId {
        self.id
    }

    pub fn parent(&self) -> Option<HtmlTreeNodeId> {
        self.parent
    }

    /// Final constructed child order.
    pub fn children(&self) -> &[HtmlTreeNodeId] {
        &self.children
    }

    pub fn kind(&self) -> &HtmlTreeNodeKind {
        &self.kind
    }
}

#[derive(Debug, Clone)]
pub enum HtmlTreeNodeKind {
    Document,
    DocumentType(HtmlTreeDocumentType),
    Element(HtmlTreeElement),
    Text(HtmlTreeText),
}

/// The one selected canonical constructed DocumentType.
///
/// The type itself is the closed selected meaning: interpreted name `html`,
/// public identifier Missing, system identifier Missing, force-quirks Off.
/// Only the exact authored evidence is exposed; the authored spelling (case)
/// of the name is retained by [`Self::authored_name`]. No document-mode,
/// identifier, or force-quirks accessor exists.
#[derive(Debug, Clone)]
pub struct HtmlTreeDocumentType {
    complete: SourceAnchor,
    authored_name: SourceAnchor,
}

impl HtmlTreeDocumentType {
    /// The exact authored `<!DOCTYPE ...>` range.
    pub fn complete(&self) -> &SourceAnchor {
        &self.complete
    }

    /// The exact authored spelling of the DOCTYPE name.
    pub fn authored_name(&self) -> &SourceAnchor {
        &self.authored_name
    }
}

#[derive(Debug, Clone)]
pub struct HtmlTreeElement {
    name: HtmlTreeElementName,
    provenance: HtmlTreeNodeProvenance,
}

impl HtmlTreeElement {
    pub fn name(&self) -> HtmlTreeElementName {
        self.name
    }

    pub fn provenance(&self) -> &HtmlTreeNodeProvenance {
        &self.provenance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTreeElementName {
    Html,
    Head,
    Body,
    Div,
    Section,
    Article,
    Aside,
    Footer,
    Header,
    Main,
    Nav,
    Paragraph,
    Style,
    Title,
}

#[derive(Debug, Clone)]
pub enum HtmlTreeNodeProvenance {
    /// Semantic absence (for example the Document root), never an unknown or
    /// sentinel source range.
    None,
    AuthoredStartTag {
        complete: SourceAnchor,
        raw_name: SourceAnchor,
    },
    Synthesized(HtmlTreeSynthesisCause),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTreeSynthesisCause {
    ImpliedByDocumentStructure,
    UnmatchedParagraphEndTag,
}

#[derive(Debug, Clone)]
pub struct HtmlTreeText {
    interpreted: String,
    contributions: Vec<HtmlTreeTextContribution>,
}

impl HtmlTreeText {
    pub fn interpreted(&self) -> &str {
        &self.interpreted
    }

    /// Ordered authored contributions; distinct from the final interpreted
    /// value.
    pub fn contributions(&self) -> &[HtmlTreeTextContribution] {
        &self.contributions
    }
}

#[derive(Debug, Clone)]
pub struct HtmlTreeTextContribution {
    source: SourceAnchor,
    interpreted: String,
}

impl HtmlTreeTextContribution {
    pub fn source(&self) -> &SourceAnchor {
        &self.source
    }

    pub fn interpreted(&self) -> &str {
        &self.interpreted
    }
}

#[derive(Debug, Clone)]
pub enum HtmlTreeCompletion {
    Complete,
    Incomplete(HtmlTreeIncompleteCause),
}

#[derive(Debug, Clone)]
pub enum HtmlTreeIncompleteCause {
    TreeUnsupported(HtmlTreeUnsupported),
    TokenizerUnsupported(HtmlTokenizerUnsupported),
    ResourceLimited(HtmlTreeResourceLimit),
}

#[derive(Debug, Clone)]
pub struct HtmlTreeUnsupported {
    capability: HtmlTreeUnsupportedCapability,
    trigger: Option<SourceAnchor>,
}

impl HtmlTreeUnsupported {
    pub fn capability(&self) -> HtmlTreeUnsupportedCapability {
        self.capability
    }

    pub fn trigger(&self) -> Option<&SourceAnchor> {
        self.trigger.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTreeUnsupportedCapability {
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
    ParagraphTagOutsideInBody,
    ParagraphTagAttribute,
    SelfClosingParagraphTag,
    ShellTagWithOpenParagraphElement,
    StyleTagAttribute,
    SelfClosingStyleTag,
    StyleTagOutsideSelectedLifecycle,
    TitleTagAttribute,
    SelfClosingTitleTag,
    TitleTagOutsideSelectedLifecycle,
    DoctypeOutsideInitial,
}

#[derive(Debug, Clone)]
pub struct HtmlTokenizerUnsupported {
    capability: HtmlTokenizerUnsupportedCapability,
    availability: HtmlTokenizerCapabilityAvailability,
    trigger: SourceAnchor,
}

impl HtmlTokenizerUnsupported {
    pub fn capability(&self) -> HtmlTokenizerUnsupportedCapability {
        self.capability
    }

    pub fn availability(&self) -> HtmlTokenizerCapabilityAvailability {
        self.availability
    }

    pub fn trigger(&self) -> &SourceAnchor {
        &self.trigger
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTokenizerCapabilityAvailability {
    Deferred,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTokenizerUnsupportedCapability {
    CharacterReference {
        context: HtmlCharacterReferenceContext,
    },
    MarkupDeclaration,
    ProcessingInstruction,
    BogusCommentRecovery,
    ContextDependentTokenizerMode {
        mode: HtmlTokenizerMode,
    },
    TreeConstructionControlledState,
    ForeignContentControl,
    NumericCharacterReferenceInData,
    NumericCharacterReferenceInRcdata,
    RcdataNullRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlCharacterReferenceContext {
    Data,
    AttributeValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTokenizerMode {
    Rcdata,
    RawText,
    ScriptData,
    Plaintext,
    Noscript,
}

#[derive(Debug, Clone)]
pub struct HtmlTreeResourceLimit {
    kind: HtmlTreeResourceKind,
    limit: usize,
    attempted: usize,
    at: SourceAnchor,
}

impl HtmlTreeResourceLimit {
    pub fn kind(&self) -> HtmlTreeResourceKind {
        self.kind
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn attempted(&self) -> usize {
        self.attempted
    }

    pub fn at(&self) -> &SourceAnchor {
        &self.at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTreeResourceKind {
    SourceBytes,
    TransitionSteps,
    EmittedTokens,
    TokenizerDiagnostics,
    AttributesPerTag,
    RetainedInterpretedBytes,
    TemporaryBufferBytes,
}

#[derive(Debug, Clone)]
pub struct HtmlTreeCoverage {
    committed_prefix: SourceAnchor,
    processed_tokens: usize,
}

impl HtmlTreeCoverage {
    pub fn committed_prefix(&self) -> &SourceAnchor {
        &self.committed_prefix
    }

    pub fn processed_tokens(&self) -> usize {
        self.processed_tokens
    }
}

#[derive(Debug, Clone)]
pub struct HtmlTokenizerDiagnostic {
    code: HtmlTokenizerDiagnosticCode,
    location: SourceAnchor,
}

impl HtmlTokenizerDiagnostic {
    pub fn code(&self) -> HtmlTokenizerDiagnosticCode {
        self.code
    }

    pub fn location(&self) -> &SourceAnchor {
        &self.location
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTokenizerDiagnosticCode {
    NoncharacterInInputStream,
    ControlCharacterInInputStream,
    UnexpectedNullCharacter,
    EofBeforeTagName,
    InvalidFirstCharacterOfTagName,
    UnexpectedQuestionMarkInsteadOfTagName,
    MissingEndTagName,
    EofInTag,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedCharacterInAttributeName,
    MissingAttributeValue,
    UnexpectedCharacterInUnquotedAttributeValue,
    MissingWhitespaceBetweenAttributes,
    UnexpectedSolidusInTag,
    DuplicateAttribute,
    EndTagWithAttributes,
    EndTagWithTrailingSolidus,
    MissingSemicolonAfterCharacterReference,
    UnknownNamedCharacterReference,
    AbsenceOfDigitsInNumericCharacterReference,
    NullCharacterReference,
    CharacterReferenceOutsideUnicodeRange,
    SurrogateCharacterReference,
    NoncharacterCharacterReference,
    ControlCharacterReference,
}

#[derive(Debug, Clone)]
pub struct HtmlTreeDiagnostic {
    code: HtmlTreeDiagnosticCode,
    recovery: HtmlTreeRecovery,
    trigger: Option<SourceAnchor>,
}

impl HtmlTreeDiagnostic {
    pub fn code(&self) -> HtmlTreeDiagnosticCode {
        self.code
    }

    pub fn recovery(&self) -> HtmlTreeRecovery {
        self.recovery
    }

    /// The trigger token's authored boundary when it has one; `None` is
    /// semantic absence (for example an end-of-file trigger), never a
    /// fabricated anchor.
    pub fn trigger(&self) -> Option<&SourceAnchor> {
        self.trigger.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTreeDiagnosticCode {
    MissingDoctype,
    DuplicateHeadStartTag,
    DuplicateBodyStartTag,
    AfterBodyCharacterData,
    AfterAfterBodyCharacterData,
    NullCharacterInBody,
    BodyEndTagWithOpenSelectedOrdinaryElements,
    HtmlEndTagWithOpenSelectedOrdinaryElements,
    UnmatchedSelectedOrdinaryEndTag,
    MisnestedSelectedOrdinaryEndTag,
    OpenSelectedOrdinaryElementAtEndOfFile,
    UnmatchedParagraphEndTag,
    StyleEndOfFileInText,
    TitleEndOfFileInText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTreeRecovery {
    ContinuedInQuirksDocumentMode,
    DuplicateShellStartTagProducedNoNode,
    SwitchedToInBodyAndReprocessedSameToken,
    SwitchedToAfterBodyPreservingOpenElements,
    IgnoredToken,
    PoppedInterveningSelectedOrdinaryElementsAndClosedTarget,
    StoppedParsingWithOpenSelectedOrdinaryElements,
    SynthesizedParagraphElementAndClosedIt,
    PoppedStyleAtEndOfFileAndRestoredInHead,
    PoppedTitleAtEndOfFileAndRestoredInHead,
}

/// A returned Core boundary failure. Never an ordinary analysis outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTreeCoreFailure {
    /// Reserving the owned public projection failed. This is never a
    /// tokenizer resource refusal.
    ProjectionResourceExhausted,
    InternalFailure,
}

impl fmt::Display for HtmlTreeCoreFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProjectionResourceExhausted => formatter.write_str(
                "HTML tree analysis returned a Core failure: report projection resources exhausted",
            ),
            Self::InternalFailure => {
                formatter.write_str("HTML tree analysis returned a Core internal failure")
            }
        }
    }
}

impl Error for HtmlTreeCoreFailure {}

fn exhausted(_: TryReserveError) -> HtmlTreeCoreFailure {
    HtmlTreeCoreFailure::ProjectionResourceExhausted
}

// ---------------------------------------------------------------------------
// Fallible owned projection
// ---------------------------------------------------------------------------

fn vec_with_capacity<T>(capacity: usize) -> Result<Vec<T>, HtmlTreeCoreFailure> {
    let mut values = Vec::new();
    values.try_reserve_exact(capacity).map_err(exhausted)?;
    Ok(values)
}

fn owned_str(value: &str) -> Result<String, HtmlTreeCoreFailure> {
    let mut owned = String::new();
    owned.try_reserve_exact(value.len()).map_err(exhausted)?;
    owned.push_str(value);
    Ok(owned)
}

fn project(
    source_id: SourceId,
    analysis: &tree_result::HtmlDocumentShellAnalysis,
) -> Result<HtmlTreeReport, HtmlTreeCoreFailure> {
    let completion = project_completion(analysis)?;

    // Reserve the ordering scratch fallibly too: the producer's own
    // `nodes_in_creation_order` allocates infallibly. Constructed identities
    // are unique, so the allocation-free unstable sort fully determines order.
    let mut internal_nodes = vec_with_capacity(analysis.node_count())?;
    internal_nodes.extend(analysis.nodes_in_storage_order());
    internal_nodes.sort_unstable_by_key(|node| node.id());
    let mut nodes = vec_with_capacity(internal_nodes.len())?;
    for node in internal_nodes {
        nodes.push(project_node(node)?);
    }

    let run = analysis.tokenizer_run();
    let mut tokenizer_diagnostics = vec_with_capacity(run.diagnostics().len())?;
    for diagnostic in run.diagnostics() {
        tokenizer_diagnostics.push(HtmlTokenizerDiagnostic {
            code: map_tokenizer_diagnostic_code(diagnostic.code()),
            location: diagnostic.location().clone(),
        });
    }

    let mut tree_diagnostics = vec_with_capacity(analysis.diagnostics().len())?;
    for diagnostic in analysis.diagnostics() {
        tree_diagnostics.push(HtmlTreeDiagnostic {
            code: map_tree_diagnostic_code(diagnostic.code()),
            recovery: map_recovery(diagnostic.recovery()),
            trigger: diagnostic.trigger().authored_boundary().cloned(),
        });
    }

    let coverage = HtmlTreeCoverage {
        committed_prefix: analysis.coverage().committed_prefix().clone(),
        processed_tokens: analysis.coverage().processed_tokens(),
    };

    let selected_ordinary_relations = project_selected_ordinary_relations(analysis)?;
    let selected_ordinary_attributes = project_selected_ordinary_attributes(analysis)?;
    let paragraph_relations = project_paragraph_relations(analysis)?;

    Ok(HtmlTreeReport {
        source_id,
        root: node_id(analysis.root()),
        nodes,
        completion,
        coverage,
        tokenizer_diagnostics,
        tree_diagnostics,
        selected_ordinary_relations,
        selected_ordinary_attributes,
        paragraph_relations,
    })
}

/// Projects the attribute of each already freeze-validated selected ordinary
/// insertion: node identity -> its insertion action -> the exact trigger
/// token -> that token's tokenizer-owned attribute evidence. Freeze has
/// proved the action/token/node correspondence; nothing is searched,
/// rescanned, or reconstructed, and no name or tree position is consulted. A
/// token that cannot be resolved after freeze fails closed.
fn project_selected_ordinary_attributes(
    analysis: &tree_result::HtmlDocumentShellAnalysis,
) -> Result<Vec<HtmlTreeSelectedOrdinaryAttribute>, HtmlTreeCoreFailure> {
    use tree_result::HtmlTreeActionKind as Action;

    let tokens = analysis.tokenizer_run().tokens();
    let attributed = |action: &tree_result::HtmlTreeAction| -> Result<_, HtmlTreeCoreFailure> {
        let Action::InsertedAuthoredSelectedOrdinaryElement { node, .. } = action.kind() else {
            return Ok(None);
        };
        let Some(HtmlToken::Tag(tag)) = tokens.get(action.trigger().token_index()) else {
            return Err(HtmlTreeCoreFailure::InternalFailure);
        };
        Ok(tag.attributes().first().map(|attribute| (*node, attribute)))
    };
    let mut count = 0usize;
    for action in analysis.actions() {
        if attributed(action)?.is_some() {
            count += 1;
        }
    }
    let mut records = vec_with_capacity(count)?;
    for action in analysis.actions() {
        let Some((node, attribute)) = attributed(action)? else {
            continue;
        };
        records.push(HtmlTreeSelectedOrdinaryAttribute {
            node: node_id(node),
            complete: attribute.complete().clone(),
            authored_name: attribute.name().source().clone(),
            value_syntax: project_attribute_value_syntax(attribute.value_syntax()),
            interpreted_name: owned_str(attribute.name().interpreted())?,
            interpreted_value: owned_str(attribute.interpreted_value())?,
        });
    }
    Ok(records)
}

fn project_attribute_value_syntax(
    syntax: &HtmlAttributeValueSyntax,
) -> HtmlTreeAttributeValueSyntax {
    match syntax {
        HtmlAttributeValueSyntax::Missing => HtmlTreeAttributeValueSyntax::Missing,
        HtmlAttributeValueSyntax::MissingAfterEquals {
            equals,
            value_boundary,
        } => HtmlTreeAttributeValueSyntax::MissingAfterEquals {
            equals: equals.clone(),
            value_boundary: value_boundary.clone(),
        },
        HtmlAttributeValueSyntax::Unquoted { equals, value } => {
            HtmlTreeAttributeValueSyntax::Unquoted {
                equals: equals.clone(),
                value: value.clone(),
            }
        }
        HtmlAttributeValueSyntax::DoubleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => HtmlTreeAttributeValueSyntax::DoubleQuoted {
            equals: equals.clone(),
            open_quote: open_quote.clone(),
            value: value.clone(),
            close_quote: close_quote.clone(),
        },
        HtmlAttributeValueSyntax::SingleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => HtmlTreeAttributeValueSyntax::SingleQuoted {
            equals: equals.clone(),
            open_quote: open_quote.clone(),
            value: value.clone(),
            close_quote: close_quote.clone(),
        },
    }
}

/// Projects the already freeze-validated selected ordinary close and
/// recovery actions, preserving their encounter order. Nothing is
/// recomputed: the freeze has proved subject, target, nearest-target
/// selection, trigger and order. A missing authored trigger cannot occur
/// after freeze and fails closed rather than inventing evidence.
fn project_selected_ordinary_relations(
    analysis: &tree_result::HtmlDocumentShellAnalysis,
) -> Result<Vec<HtmlTreeSelectedOrdinaryRelation>, HtmlTreeCoreFailure> {
    use tree_result::HtmlTreeActionKind as Action;

    let is_relation = |action: &&tree_result::HtmlTreeAction| {
        matches!(
            action.kind(),
            Action::ClosedSelectedOrdinaryElement { .. }
                | Action::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
        )
    };
    let count = analysis.actions().iter().filter(is_relation).count();
    let mut relations = vec_with_capacity(count)?;
    for action in analysis.actions().iter().filter(is_relation) {
        let trigger = action
            .trigger()
            .authored_boundary()
            .cloned()
            .ok_or(HtmlTreeCoreFailure::InternalFailure)?;
        relations.push(match action.kind() {
            Action::ClosedSelectedOrdinaryElement { node, .. } => {
                HtmlTreeSelectedOrdinaryRelation::MatchingClose {
                    node: node_id(*node),
                    trigger,
                }
            }
            Action::PoppedSelectedOrdinaryElementByAncestorEndTag { node, target } => {
                HtmlTreeSelectedOrdinaryRelation::RecoveryPopByAncestorEndTag {
                    node: node_id(*node),
                    target: node_id(*target),
                    trigger,
                }
            }
            _ => return Err(HtmlTreeCoreFailure::InternalFailure),
        });
    }
    Ok(relations)
}

/// Projects the already freeze-validated Paragraph close and implied-pop
/// actions, preserving their encounter order. Nothing is recomputed: freeze
/// has proved subject, closure kind, trigger shape, the synthesized node's
/// own paired close, the implied-pop target, and that a start-triggered close
/// is immediately followed by the insertion of the new element by the same
/// trigger token. That following action supplies `inserted`; no source, name,
/// creation order, or tree position is consulted. A missing authored trigger
/// or insertion cannot occur after freeze and fails closed.
fn project_paragraph_relations(
    analysis: &tree_result::HtmlDocumentShellAnalysis,
) -> Result<Vec<HtmlTreeParagraphRelation>, HtmlTreeCoreFailure> {
    use tree_result::HtmlTreeActionKind as Action;

    let actions = analysis.actions();
    let is_relation = |action: &&tree_result::HtmlTreeAction| {
        matches!(
            action.kind(),
            Action::ClosedParagraphElement { .. }
                | Action::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
        )
    };
    let count = actions.iter().filter(is_relation).count();
    let mut relations = vec_with_capacity(count)?;
    for (index, action) in actions.iter().enumerate() {
        if !is_relation(&action) {
            continue;
        }
        let trigger = action
            .trigger()
            .authored_boundary()
            .cloned()
            .ok_or(HtmlTreeCoreFailure::InternalFailure)?;
        relations.push(match action.kind() {
            Action::ClosedParagraphElement { node, closure } => match closure {
                tree_result::HtmlParagraphClosure::MatchingEndTag => {
                    HtmlTreeParagraphRelation::MatchingClose {
                        node: node_id(*node),
                        trigger,
                    }
                }
                tree_result::HtmlParagraphClosure::StartTriggered => {
                    let next = actions
                        .get(index + 1)
                        .ok_or(HtmlTreeCoreFailure::InternalFailure)?;
                    if next.trigger().token_index() != action.trigger().token_index() {
                        return Err(HtmlTreeCoreFailure::InternalFailure);
                    }
                    let inserted = match next.kind() {
                        Action::InsertedAuthoredParagraphElement { node }
                        | Action::InsertedAuthoredSelectedOrdinaryElement { node, .. } => *node,
                        _ => return Err(HtmlTreeCoreFailure::InternalFailure),
                    };
                    HtmlTreeParagraphRelation::StartTriggeredClose {
                        node: node_id(*node),
                        inserted: node_id(inserted),
                        trigger,
                    }
                }
                tree_result::HtmlParagraphClosure::UnmatchedEndTagSynthesized => {
                    HtmlTreeParagraphRelation::SynthesizedCloseByUnmatchedEndTag {
                        node: node_id(*node),
                        trigger,
                    }
                }
            },
            Action::PoppedParagraphElementBySelectedOrdinaryEndTag { node, target } => {
                HtmlTreeParagraphRelation::ImpliedPopBySelectedOrdinaryEndTag {
                    node: node_id(*node),
                    target: node_id(*target),
                    trigger,
                }
            }
            _ => return Err(HtmlTreeCoreFailure::InternalFailure),
        });
    }
    Ok(relations)
}

fn node_id(id: tree_result::HtmlConstructedNodeId) -> HtmlTreeNodeId {
    HtmlTreeNodeId(id.creation_ordinal())
}

fn project_node(node: &tree_result::HtmlTreeNode) -> Result<HtmlTreeNode, HtmlTreeCoreFailure> {
    let mut children = vec_with_capacity(node.children().len())?;
    children.extend(node.children().iter().copied().map(node_id));
    let kind = match node.kind() {
        tree_result::HtmlTreeNodeKind::Document => HtmlTreeNodeKind::Document,
        tree_result::HtmlTreeNodeKind::DocumentType(doctype) => {
            HtmlTreeNodeKind::DocumentType(HtmlTreeDocumentType {
                complete: doctype.complete().clone(),
                authored_name: doctype.authored_name().clone(),
            })
        }
        tree_result::HtmlTreeNodeKind::Element(element) => {
            HtmlTreeNodeKind::Element(project_element(element))
        }
        tree_result::HtmlTreeNodeKind::Text(text) => {
            let mut contributions = vec_with_capacity(text.contributions().len())?;
            for contribution in text.contributions() {
                contributions.push(HtmlTreeTextContribution {
                    source: contribution.source().clone(),
                    interpreted: owned_str(contribution.interpreted())?,
                });
            }
            HtmlTreeNodeKind::Text(HtmlTreeText {
                interpreted: owned_str(text.interpreted())?,
                contributions,
            })
        }
    };
    Ok(HtmlTreeNode {
        id: node_id(node.id()),
        parent: node.parent().map(node_id),
        children,
        kind,
    })
}

fn authored(complete: &SourceAnchor, raw_name: &SourceAnchor) -> HtmlTreeNodeProvenance {
    HtmlTreeNodeProvenance::AuthoredStartTag {
        complete: complete.clone(),
        raw_name: raw_name.clone(),
    }
}

fn project_element(element: &tree_result::HtmlElement) -> HtmlTreeElement {
    let (name, provenance) = match element {
        tree_result::HtmlElement::Shell(shell) => {
            let name = match shell.name() {
                tree_result::HtmlShellElementName::Html => HtmlTreeElementName::Html,
                tree_result::HtmlShellElementName::Head => HtmlTreeElementName::Head,
                tree_result::HtmlShellElementName::Body => HtmlTreeElementName::Body,
            };
            let provenance = match shell.origin() {
                tree_result::HtmlShellElementOrigin::Authored { complete, raw_name } => {
                    authored(complete, raw_name)
                }
                tree_result::HtmlShellElementOrigin::Synthesized(
                    tree_result::HtmlSynthesisCause::ImpliedByDocumentStructure,
                ) => HtmlTreeNodeProvenance::Synthesized(
                    HtmlTreeSynthesisCause::ImpliedByDocumentStructure,
                ),
            };
            (name, provenance)
        }
        tree_result::HtmlElement::SelectedOrdinary(selected) => {
            let name = match selected.name() {
                tree_result::HtmlSelectedOrdinaryElementName::Div => HtmlTreeElementName::Div,
                tree_result::HtmlSelectedOrdinaryElementName::Section => {
                    HtmlTreeElementName::Section
                }
                tree_result::HtmlSelectedOrdinaryElementName::Article => {
                    HtmlTreeElementName::Article
                }
                tree_result::HtmlSelectedOrdinaryElementName::Aside => HtmlTreeElementName::Aside,
                tree_result::HtmlSelectedOrdinaryElementName::Footer => HtmlTreeElementName::Footer,
                tree_result::HtmlSelectedOrdinaryElementName::Header => HtmlTreeElementName::Header,
                tree_result::HtmlSelectedOrdinaryElementName::Main => HtmlTreeElementName::Main,
                tree_result::HtmlSelectedOrdinaryElementName::Nav => HtmlTreeElementName::Nav,
            };
            (name, authored(selected.complete(), selected.raw_name()))
        }
        tree_result::HtmlElement::Paragraph(paragraph) => {
            let provenance = match paragraph.origin() {
                tree_result::HtmlParagraphElementOrigin::Authored { complete, raw_name } => {
                    authored(complete, raw_name)
                }
                tree_result::HtmlParagraphElementOrigin::Synthesized(
                    tree_result::HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag,
                ) => HtmlTreeNodeProvenance::Synthesized(
                    HtmlTreeSynthesisCause::UnmatchedParagraphEndTag,
                ),
            };
            (HtmlTreeElementName::Paragraph, provenance)
        }
        tree_result::HtmlElement::Style(style) => (
            HtmlTreeElementName::Style,
            authored(style.complete(), style.raw_name()),
        ),
        tree_result::HtmlElement::Title(title) => (
            HtmlTreeElementName::Title,
            authored(title.complete(), title.raw_name()),
        ),
    };
    HtmlTreeElement { name, provenance }
}

/// Maps the internal completion without ever upgrading lower-layer
/// incompletion. Anything that cannot be represented without inventing a
/// classification fails closed.
fn project_completion(
    analysis: &tree_result::HtmlDocumentShellAnalysis,
) -> Result<HtmlTreeCompletion, HtmlTreeCoreFailure> {
    match analysis.completion() {
        tree_result::HtmlTreeCompletion::Complete => Ok(HtmlTreeCompletion::Complete),
        tree_result::HtmlTreeCompletion::Incomplete(
            tree_result::HtmlTreeIncompleteCause::UnsupportedCapability(unsupported),
        ) => Ok(HtmlTreeCompletion::Incomplete(
            HtmlTreeIncompleteCause::TreeUnsupported(HtmlTreeUnsupported {
                capability: map_tree_capability(unsupported.capability()),
                trigger: unsupported.trigger().authored_boundary().cloned(),
            }),
        )),
        tree_result::HtmlTreeCompletion::Incomplete(
            tree_result::HtmlTreeIncompleteCause::LowerLayerIncomplete,
        ) => match analysis.tokenizer_run().completion() {
            tokenizer_result::HtmlTokenizerCompletion::Incomplete(
                tokenizer_result::HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported),
            ) => {
                let trigger = match unsupported.trigger() {
                    tokenizer_result::HtmlTokenizerUnsupportedTrigger::Input(anchor)
                    | tokenizer_result::HtmlTokenizerUnsupportedTrigger::EmittedToken {
                        boundary: anchor,
                        ..
                    } => anchor.clone(),
                };
                Ok(HtmlTreeCompletion::Incomplete(
                    HtmlTreeIncompleteCause::TokenizerUnsupported(HtmlTokenizerUnsupported {
                        capability: map_tokenizer_capability(unsupported.capability()),
                        availability: match unsupported.availability() {
                            tokenizer_result::HtmlTokenizerCapabilityAvailability::Deferred => {
                                HtmlTokenizerCapabilityAvailability::Deferred
                            }
                            tokenizer_result::HtmlTokenizerCapabilityAvailability::Unsupported => {
                                HtmlTokenizerCapabilityAvailability::Unsupported
                            }
                        },
                        trigger,
                    }),
                ))
            }
            tokenizer_result::HtmlTokenizerCompletion::Incomplete(
                tokenizer_result::HtmlTokenizerIncompleteCause::ResourceLimit(limit),
            ) => Ok(HtmlTreeCompletion::Incomplete(
                HtmlTreeIncompleteCause::ResourceLimited(project_resource_limit(limit)),
            )),
            tokenizer_result::HtmlTokenizerCompletion::Incomplete(
                tokenizer_result::HtmlTokenizerIncompleteCause::InvalidConfiguration(_)
                | tokenizer_result::HtmlTokenizerIncompleteCause::InternalInvariantFailure(_),
            ) => Err(HtmlTreeCoreFailure::InternalFailure),
            // The tree stopped with a complete tokenizer run: the approved
            // completion mapping has no representation for this, and it must
            // not be upgraded to Complete.
            tokenizer_result::HtmlTokenizerCompletion::Complete => {
                Err(HtmlTreeCoreFailure::InternalFailure)
            }
        },
    }
}

fn project_resource_limit(
    limit: &super::tokenizer::resource::HtmlTokenizerResourceLimit,
) -> HtmlTreeResourceLimit {
    HtmlTreeResourceLimit {
        kind: map_resource(limit.resource()),
        limit: limit.limit(),
        attempted: limit.attempted(),
        at: limit.at().clone(),
    }
}

fn map_resource(resource: HtmlTokenizerResource) -> HtmlTreeResourceKind {
    match resource {
        HtmlTokenizerResource::SourceBytes => HtmlTreeResourceKind::SourceBytes,
        HtmlTokenizerResource::TransitionSteps => HtmlTreeResourceKind::TransitionSteps,
        HtmlTokenizerResource::EmittedTokens => HtmlTreeResourceKind::EmittedTokens,
        HtmlTokenizerResource::Diagnostics => HtmlTreeResourceKind::TokenizerDiagnostics,
        HtmlTokenizerResource::AttributesPerTag => HtmlTreeResourceKind::AttributesPerTag,
        HtmlTokenizerResource::RetainedInterpretedBytes => {
            HtmlTreeResourceKind::RetainedInterpretedBytes
        }
        HtmlTokenizerResource::TemporaryBufferBytes => HtmlTreeResourceKind::TemporaryBufferBytes,
    }
}

fn map_tree_capability(
    capability: tree_result::HtmlTreeCapability,
) -> HtmlTreeUnsupportedCapability {
    use HtmlTreeUnsupportedCapability as Public;
    use tree_result::HtmlTreeCapability as Internal;
    match capability {
        Internal::NonShellElementTag => Public::NonShellElementTag,
        Internal::ShellTagAttribute => Public::ShellTagAttribute,
        Internal::SelfClosingShellTag => Public::SelfClosingShellTag,
        Internal::WhitespaceSensitiveCharacterData => Public::WhitespaceSensitiveCharacterData,
        Internal::UnprovedCharacterDataPosition => Public::UnprovedCharacterDataPosition,
        Internal::UnprovedShellStartTagPosition => Public::UnprovedShellStartTagPosition,
        Internal::UnprovedShellEndTagPosition => Public::UnprovedShellEndTagPosition,
        Internal::SelectedOrdinaryTagOutsideInBody => Public::SelectedOrdinaryTagOutsideInBody,
        Internal::ShellTagWithOpenSelectedOrdinaryElement => {
            Public::ShellTagWithOpenSelectedOrdinaryElement
        }
        Internal::SelectedOrdinaryTagAttribute => Public::SelectedOrdinaryTagAttribute,
        Internal::SelfClosingSelectedOrdinaryTag => Public::SelfClosingSelectedOrdinaryTag,
        Internal::ParagraphTagOutsideInBody => Public::ParagraphTagOutsideInBody,
        Internal::ParagraphTagAttribute => Public::ParagraphTagAttribute,
        Internal::SelfClosingParagraphTag => Public::SelfClosingParagraphTag,
        Internal::ShellTagWithOpenParagraphElement => Public::ShellTagWithOpenParagraphElement,
        Internal::StyleTagAttribute => Public::StyleTagAttribute,
        Internal::SelfClosingStyleTag => Public::SelfClosingStyleTag,
        Internal::StyleTagOutsideSelectedLifecycle => Public::StyleTagOutsideSelectedLifecycle,
        Internal::TitleTagAttribute => Public::TitleTagAttribute,
        Internal::SelfClosingTitleTag => Public::SelfClosingTitleTag,
        Internal::TitleTagOutsideSelectedLifecycle => Public::TitleTagOutsideSelectedLifecycle,
        Internal::DoctypeOutsideInitial => Public::DoctypeOutsideInitial,
    }
}

fn map_tokenizer_capability(
    capability: tokenizer_result::HtmlTokenizerCapability,
) -> HtmlTokenizerUnsupportedCapability {
    use HtmlTokenizerUnsupportedCapability as Public;
    use tokenizer_result::HtmlTokenizerCapability as Internal;
    match capability {
        Internal::CharacterReference { context } => Public::CharacterReference {
            context: match context {
                tokenizer_result::HtmlCharacterReferenceContext::Data => {
                    HtmlCharacterReferenceContext::Data
                }
                tokenizer_result::HtmlCharacterReferenceContext::AttributeValue => {
                    HtmlCharacterReferenceContext::AttributeValue
                }
            },
        },
        Internal::MarkupDeclaration => Public::MarkupDeclaration,
        Internal::ProcessingInstruction => Public::ProcessingInstruction,
        Internal::BogusCommentRecovery => Public::BogusCommentRecovery,
        Internal::ContextDependentTokenizerMode { mode } => Public::ContextDependentTokenizerMode {
            mode: match mode {
                tokenizer_result::HtmlTokenizerMode::Rcdata => HtmlTokenizerMode::Rcdata,
                tokenizer_result::HtmlTokenizerMode::RawText => HtmlTokenizerMode::RawText,
                tokenizer_result::HtmlTokenizerMode::ScriptData => HtmlTokenizerMode::ScriptData,
                tokenizer_result::HtmlTokenizerMode::Plaintext => HtmlTokenizerMode::Plaintext,
                tokenizer_result::HtmlTokenizerMode::Noscript => HtmlTokenizerMode::Noscript,
            },
        },
        Internal::TreeConstructionControlledState => Public::TreeConstructionControlledState,
        Internal::ForeignContentControl => Public::ForeignContentControl,
        Internal::NumericCharacterReferenceInData => Public::NumericCharacterReferenceInData,
        Internal::NumericCharacterReferenceInRcdata => Public::NumericCharacterReferenceInRcdata,
        Internal::RcdataNullRecovery => Public::RcdataNullRecovery,
    }
}

fn map_tokenizer_diagnostic_code(
    code: tokenizer_diagnostic::HtmlTokenizerDiagnosticCode,
) -> HtmlTokenizerDiagnosticCode {
    use HtmlTokenizerDiagnosticCode as Public;
    use tokenizer_diagnostic::HtmlTokenizerDiagnosticCode as Internal;
    match code {
        Internal::NoncharacterInInputStream => Public::NoncharacterInInputStream,
        Internal::ControlCharacterInInputStream => Public::ControlCharacterInInputStream,
        Internal::UnexpectedNullCharacter => Public::UnexpectedNullCharacter,
        Internal::EofBeforeTagName => Public::EofBeforeTagName,
        Internal::InvalidFirstCharacterOfTagName => Public::InvalidFirstCharacterOfTagName,
        Internal::UnexpectedQuestionMarkInsteadOfTagName => {
            Public::UnexpectedQuestionMarkInsteadOfTagName
        }
        Internal::MissingEndTagName => Public::MissingEndTagName,
        Internal::EofInTag => Public::EofInTag,
        Internal::UnexpectedEqualsSignBeforeAttributeName => {
            Public::UnexpectedEqualsSignBeforeAttributeName
        }
        Internal::UnexpectedCharacterInAttributeName => Public::UnexpectedCharacterInAttributeName,
        Internal::MissingAttributeValue => Public::MissingAttributeValue,
        Internal::UnexpectedCharacterInUnquotedAttributeValue => {
            Public::UnexpectedCharacterInUnquotedAttributeValue
        }
        Internal::MissingWhitespaceBetweenAttributes => Public::MissingWhitespaceBetweenAttributes,
        Internal::UnexpectedSolidusInTag => Public::UnexpectedSolidusInTag,
        Internal::DuplicateAttribute => Public::DuplicateAttribute,
        Internal::EndTagWithAttributes => Public::EndTagWithAttributes,
        Internal::EndTagWithTrailingSolidus => Public::EndTagWithTrailingSolidus,
        Internal::MissingSemicolonAfterCharacterReference => {
            Public::MissingSemicolonAfterCharacterReference
        }
        Internal::UnknownNamedCharacterReference => Public::UnknownNamedCharacterReference,
        Internal::AbsenceOfDigitsInNumericCharacterReference => {
            Public::AbsenceOfDigitsInNumericCharacterReference
        }
        Internal::NullCharacterReference => Public::NullCharacterReference,
        Internal::CharacterReferenceOutsideUnicodeRange => {
            Public::CharacterReferenceOutsideUnicodeRange
        }
        Internal::SurrogateCharacterReference => Public::SurrogateCharacterReference,
        Internal::NoncharacterCharacterReference => Public::NoncharacterCharacterReference,
        Internal::ControlCharacterReference => Public::ControlCharacterReference,
    }
}

fn map_tree_diagnostic_code(code: tree_result::HtmlTreeDiagnosticCode) -> HtmlTreeDiagnosticCode {
    use HtmlTreeDiagnosticCode as Public;
    use tree_result::HtmlTreeDiagnosticCode as Internal;
    match code {
        Internal::MissingDoctype => Public::MissingDoctype,
        Internal::DuplicateHeadStartTag => Public::DuplicateHeadStartTag,
        Internal::DuplicateBodyStartTag => Public::DuplicateBodyStartTag,
        Internal::AfterBodyCharacterData => Public::AfterBodyCharacterData,
        Internal::AfterAfterBodyCharacterData => Public::AfterAfterBodyCharacterData,
        Internal::NullCharacterInBody => Public::NullCharacterInBody,
        Internal::BodyEndTagWithOpenSelectedOrdinaryElements => {
            Public::BodyEndTagWithOpenSelectedOrdinaryElements
        }
        Internal::HtmlEndTagWithOpenSelectedOrdinaryElements => {
            Public::HtmlEndTagWithOpenSelectedOrdinaryElements
        }
        Internal::UnmatchedSelectedOrdinaryEndTag => Public::UnmatchedSelectedOrdinaryEndTag,
        Internal::MisnestedSelectedOrdinaryEndTag => Public::MisnestedSelectedOrdinaryEndTag,
        Internal::OpenSelectedOrdinaryElementAtEndOfFile => {
            Public::OpenSelectedOrdinaryElementAtEndOfFile
        }
        Internal::UnmatchedParagraphEndTag => Public::UnmatchedParagraphEndTag,
        Internal::StyleEndOfFileInText => Public::StyleEndOfFileInText,
        Internal::TitleEndOfFileInText => Public::TitleEndOfFileInText,
    }
}

fn map_recovery(recovery: tree_result::HtmlTreeRecovery) -> HtmlTreeRecovery {
    use HtmlTreeRecovery as Public;
    use tree_result::HtmlTreeRecovery as Internal;
    match recovery {
        Internal::ContinuedInQuirksDocumentMode => Public::ContinuedInQuirksDocumentMode,
        Internal::DuplicateShellStartTagProducedNoNode => {
            Public::DuplicateShellStartTagProducedNoNode
        }
        Internal::SwitchedToInBodyAndReprocessedSameToken => {
            Public::SwitchedToInBodyAndReprocessedSameToken
        }
        Internal::SwitchedToAfterBodyPreservingOpenElements => {
            Public::SwitchedToAfterBodyPreservingOpenElements
        }
        Internal::IgnoredToken => Public::IgnoredToken,
        Internal::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget => {
            Public::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget
        }
        Internal::StoppedParsingWithOpenSelectedOrdinaryElements => {
            Public::StoppedParsingWithOpenSelectedOrdinaryElements
        }
        Internal::SynthesizedParagraphElementAndClosedIt => {
            Public::SynthesizedParagraphElementAndClosedIt
        }
        Internal::PoppedStyleAtEndOfFileAndRestoredInHead => {
            Public::PoppedStyleAtEndOfFileAndRestoredInHead
        }
        Internal::PoppedTitleAtEndOfFileAndRestoredInHead => {
            Public::PoppedTitleAtEndOfFileAndRestoredInHead
        }
    }
}
