//! Production correspondence for TC-S4 — Selected In-Body Heterogeneous
//! `div`/`section` Block Closure Recovery (Issue #363).
//!
//! Every expectation here is hand-authored against the accepted TC-S4 theorem
//! and the exact source bytes each case names. Nothing in this module
//! imports, derives, or replays anything from the candidate-independent
//! oracle in [`super::in_body_div_section_successor_validation`]: that module
//! keeps its own private candidate machine, its own node identities, its own
//! arena, and its own HS GOLD, and it stays byte-for-byte unchanged. Some
//! source byte strings deliberately coincide, because both were written from
//! the same accepted theorem — but no expected value here was read from
//! there.
//!
//! The expected model below is its own small vocabulary ([`ExpectedNode`],
//! [`ExpectedAction`], [`ExpectedDiagnostic`], [`ExpectedClosure`],
//! [`ExpectedRecovery`], [`ExpectedCompletion`]) rather than the production
//! result enums, so a production change cannot quietly redefine what the test
//! expects. `project_*` translates a production result into that vocabulary
//! and carries no expectations of its own.
//!
//! Closure and recovery endpoints are named by each endpoint node's own exact
//! authored complete start-tag range, never by a raw identity encoding: that
//! is what makes "this relation names *this* element" checkable without
//! promising any identity representation, and it is what proves the two
//! relations stay distinct.

use crate::{SourceId, SourceText};

use super::driver::construct_html_document_shell;
use super::result::{
    HtmlAuthoredSource, HtmlConstructedIdentityCounter, HtmlConstructedNodeId,
    HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlElement, HtmlSelectedOrdinaryElement,
    HtmlSelectedOrdinaryElementName, HtmlShellElement, HtmlShellElementName,
    HtmlShellElementOrigin, HtmlSynthesisCause, HtmlTreeAction, HtmlTreeActionKind,
    HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeDiagnostic, HtmlTreeDiagnosticCode,
    HtmlTreeFreezeError, HtmlTreeIncompleteCause, HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery,
    HtmlTreeTokenTrigger, freeze,
};

use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;

// ---------------------------------------------------------------------------
// Independently hand-authored expected model
// ---------------------------------------------------------------------------

type Span = (usize, usize);

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpectedNode {
    Document(Vec<ExpectedNode>),
    Shell {
        name: &'static str,
        origin: Option<(Span, Span)>,
        children: Vec<ExpectedNode>,
    },
    /// A selected ordinary element. It is authored-only, so its exact
    /// `(complete, raw_name)` evidence is not optional — and it stays its own
    /// start tag's evidence whether the element was matched, recovery-popped,
    /// or left open.
    SelectedOrdinary {
        name: &'static str,
        complete: Span,
        raw_name: Span,
        children: Vec<ExpectedNode>,
    },
    Text {
        interpreted: String,
        contributions: Vec<Span>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpectedAction {
    InsertedAuthoredShell(&'static str),
    InsertedSynthesizedShell(&'static str),
    InsertedText,
    AppendedText,
    ClosedShellByEndTag(&'static str),
    ClosedShellByImpliedToken(&'static str),
    AcknowledgedShellEndTag(&'static str),
    DuplicateShellStartTagCreatedNoNode(&'static str),
    InsertedAuthoredSelectedOrdinary(&'static str),
    /// The target's own matching closure. Distinct from
    /// [`Self::RecoveryPoppedSelectedOrdinary`] on purpose.
    ClosedSelectedOrdinary(&'static str),
    /// One intervening element popped for an ancestor's end tag. It carries
    /// no name here precisely because it is *not* a closure of a same-named
    /// end tag; the endpoints are checked by span in [`ExpectedRecovery`].
    RecoveryPoppedSelectedOrdinary,
    IgnoredUnmatchedSelectedOrdinaryEndTag(&'static str),
    Reprocessed,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpectedDiagnostic {
    MissingDoctype {
        token: usize,
        trigger: Span,
    },
    UnmatchedSelectedOrdinaryEndTag {
        token: usize,
        trigger: Span,
    },
    MisnestedSelectedOrdinaryEndTag {
        token: usize,
        trigger: Span,
    },
    /// The end-of-file trigger has no authored extent, so no span is recorded
    /// for it and none may be fabricated.
    OpenSelectedOrdinaryElementAtEndOfFile {
        token: usize,
    },
    BodyEndTagWithOpenSelectedOrdinaryElements {
        token: usize,
        trigger: Span,
    },
    HtmlEndTagWithOpenSelectedOrdinaryElements {
        token: usize,
        trigger: Span,
    },
}

/// One matching closure, named by the closed element's own authored start tag
/// and the exact authored end tag that triggered it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpectedClosure {
    subject_start_tag: Span,
    trigger_token: usize,
    trigger: Span,
}

/// One heterogeneous recovery pop, named by the popped element's own authored
/// start tag, the target's own authored start tag, and the exact authored end
/// tag that caused it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpectedRecovery {
    subject_start_tag: Span,
    target_start_tag: Span,
    trigger_token: usize,
    trigger: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpectedCompletion {
    Complete,
    Unsupported {
        capability: HtmlTreeCapability,
        token: usize,
        trigger: Option<Span>,
    },
    LowerLayerIncomplete,
}

/// The whole hand-authored expectation for one source.
struct ExpectedRun {
    id: &'static str,
    source: &'static str,
    tree: ExpectedNode,
    actions: Vec<(ExpectedAction, usize)>,
    diagnostics: Vec<ExpectedDiagnostic>,
    closures: Vec<ExpectedClosure>,
    recoveries: Vec<ExpectedRecovery>,
    node_count: usize,
    committed_end: usize,
    processed_tokens: usize,
    completion: ExpectedCompletion,
}

// ---------------------------------------------------------------------------
// Projection from the production result into the expected vocabulary
// ---------------------------------------------------------------------------

fn generous_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn analyze_with(source_text: &str, source_id: u64) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(source_id), source_text.to_owned());
    construct_html_document_shell(&source, generous_limits()).expect("no boundary failure")
}

fn analyze(source_text: &str) -> HtmlDocumentShellAnalysis {
    analyze_with(source_text, 1)
}

fn span(anchor: &crate::SourceAnchor) -> Span {
    (anchor.range().start(), anchor.range().end())
}

fn shell_name(name: HtmlShellElementName) -> &'static str {
    match name {
        HtmlShellElementName::Html => "html",
        HtmlShellElementName::Head => "head",
        HtmlShellElementName::Body => "body",
    }
}

fn selected_name(name: HtmlSelectedOrdinaryElementName) -> &'static str {
    match name {
        HtmlSelectedOrdinaryElementName::Div => "div",
        HtmlSelectedOrdinaryElementName::Section => "section",
    }
}

/// The exact authored complete start-tag span of a selected ordinary element,
/// resolved by semantic constructed identity.
///
/// Resolving through [`HtmlDocumentShellAnalysis::node`] is the point: it
/// proves a relation names a stored selected ordinary element by identity, and
/// that the element's origin is still its own start tag rather than whatever
/// end tag ended its open lifetime.
fn selected_start_tag(analysis: &HtmlDocumentShellAnalysis, id: HtmlConstructedNodeId) -> Span {
    let node = analysis.node(id).expect("relation endpoint resolves");
    let HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) = node.kind() else {
        panic!("a selected ordinary relation endpoint must be a selected ordinary element")
    };
    span(selected.complete())
}

fn project_tree(
    analysis: &HtmlDocumentShellAnalysis,
    id: super::result::HtmlConstructedNodeId,
) -> ExpectedNode {
    let node = analysis.node(id).expect("relationship resolves");
    let children: Vec<ExpectedNode> = node
        .children()
        .iter()
        .map(|child| project_tree(analysis, *child))
        .collect();
    match node.kind() {
        HtmlTreeNodeKind::Document => ExpectedNode::Document(children),
        HtmlTreeNodeKind::Element(HtmlElement::Shell(shell)) => ExpectedNode::Shell {
            name: shell_name(shell.name()),
            origin: match shell.origin() {
                HtmlShellElementOrigin::Authored { complete, raw_name } => {
                    Some((span(complete), span(raw_name)))
                }
                HtmlShellElementOrigin::Synthesized(_) => None,
            },
            children,
        },
        HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) => {
            ExpectedNode::SelectedOrdinary {
                name: selected_name(selected.name()),
                complete: span(selected.complete()),
                raw_name: span(selected.raw_name()),
                children,
            }
        }
        HtmlTreeNodeKind::Element(HtmlElement::Paragraph(_)) => {
            panic!("TC-S4 predecessor fixtures must not construct a TC-S5 Paragraph")
        }
        HtmlTreeNodeKind::Element(HtmlElement::Style(_)) => {
            panic!("TC-S4 predecessor fixtures must not construct a TC-S9 Style")
        }
        HtmlTreeNodeKind::Element(HtmlElement::Title(_)) => {
            panic!("TC-S4 predecessor fixtures must not construct a TC-S10 Title")
        }
        HtmlTreeNodeKind::Text(text) => ExpectedNode::Text {
            interpreted: text.interpreted().to_owned(),
            contributions: text
                .contributions()
                .iter()
                .map(|contribution| span(contribution.source()))
                .collect(),
        },
    }
}

fn project_actions(analysis: &HtmlDocumentShellAnalysis) -> Vec<(ExpectedAction, usize)> {
    analysis
        .actions()
        .iter()
        .map(|action| {
            let projected = match action.kind() {
                HtmlTreeActionKind::InsertedAuthoredShellElement { name, .. } => {
                    ExpectedAction::InsertedAuthoredShell(shell_name(*name))
                }
                HtmlTreeActionKind::InsertedSynthesizedShellElement { name, .. } => {
                    ExpectedAction::InsertedSynthesizedShell(shell_name(*name))
                }
                HtmlTreeActionKind::InsertedTextNode { .. } => ExpectedAction::InsertedText,
                HtmlTreeActionKind::AppendedToTextNode { .. } => ExpectedAction::AppendedText,
                HtmlTreeActionKind::ClosedShellElement { name, closure, .. } => match closure {
                    super::result::HtmlShellClosure::AuthoredEndTag => {
                        ExpectedAction::ClosedShellByEndTag(shell_name(*name))
                    }
                    super::result::HtmlShellClosure::ImpliedByToken => {
                        ExpectedAction::ClosedShellByImpliedToken(shell_name(*name))
                    }
                },
                HtmlTreeActionKind::AcknowledgedShellEndTag { name } => {
                    ExpectedAction::AcknowledgedShellEndTag(shell_name(*name))
                }
                HtmlTreeActionKind::DuplicateShellStartTagCreatedNoNode { name } => {
                    ExpectedAction::DuplicateShellStartTagCreatedNoNode(shell_name(*name))
                }
                HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { name, .. } => {
                    ExpectedAction::InsertedAuthoredSelectedOrdinary(selected_name(*name))
                }
                HtmlTreeActionKind::ClosedSelectedOrdinaryElement { name, .. } => {
                    ExpectedAction::ClosedSelectedOrdinary(selected_name(*name))
                }
                HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. } => {
                    ExpectedAction::RecoveryPoppedSelectedOrdinary
                }
                HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag { name } => {
                    ExpectedAction::IgnoredUnmatchedSelectedOrdinaryEndTag(selected_name(*name))
                }
                HtmlTreeActionKind::InsertedAuthoredParagraphElement { .. }
                | HtmlTreeActionKind::InsertedSynthesizedParagraphElement { .. }
                | HtmlTreeActionKind::ClosedParagraphElement { .. }
                | HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. } => {
                    panic!("TC-S4 predecessor fixtures must not record a TC-S5 Paragraph action")
                }
                HtmlTreeActionKind::InsertedAuthoredStyleElement { .. }
                | HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { .. }
                | HtmlTreeActionKind::PoppedStyleElementAtEndOfFile { .. } => {
                    panic!("TC-S4 predecessor fixtures must not record a TC-S9 Style action")
                }
                HtmlTreeActionKind::InsertedAuthoredTitleElement { .. }
                | HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { .. }
                | HtmlTreeActionKind::PoppedTitleElementAtEndOfFile { .. } => {
                    panic!("TC-S4 predecessor fixtures must not record a TC-S10 Title action")
                }
                HtmlTreeActionKind::ReprocessedToken => ExpectedAction::Reprocessed,
                HtmlTreeActionKind::StoppedParsing => ExpectedAction::Stopped,
            };
            (projected, action.trigger().token_index())
        })
        .collect()
}

fn project_diagnostics(analysis: &HtmlDocumentShellAnalysis) -> Vec<ExpectedDiagnostic> {
    analysis
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            let token = diagnostic.trigger().token_index();
            let authored = |what: &str| {
                span(
                    diagnostic
                        .trigger()
                        .authored_boundary()
                        .unwrap_or_else(|| panic!("authored {what} trigger")),
                )
            };
            match diagnostic.code() {
                HtmlTreeDiagnosticCode::MissingDoctype => ExpectedDiagnostic::MissingDoctype {
                    token,
                    trigger: authored("missing-DOCTYPE"),
                },
                HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag => {
                    ExpectedDiagnostic::UnmatchedSelectedOrdinaryEndTag {
                        token,
                        trigger: authored("stray end-tag"),
                    }
                }
                HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag => {
                    assert_eq!(
                        diagnostic.recovery(),
                        HtmlTreeRecovery::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget,
                        "the misnested end tag summarizes the suffix recovery"
                    );
                    ExpectedDiagnostic::MisnestedSelectedOrdinaryEndTag {
                        token,
                        trigger: authored("misnested end-tag"),
                    }
                }
                HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile => {
                    assert!(
                        diagnostic.trigger().authored_boundary().is_none(),
                        "the end-of-file trigger must carry no authored extent"
                    );
                    ExpectedDiagnostic::OpenSelectedOrdinaryElementAtEndOfFile { token }
                }
                HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements => {
                    assert_eq!(
                        diagnostic.recovery(),
                        HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements,
                    );
                    ExpectedDiagnostic::BodyEndTagWithOpenSelectedOrdinaryElements {
                        token,
                        trigger: authored("body-end"),
                    }
                }
                HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements => {
                    assert_eq!(
                        diagnostic.recovery(),
                        HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements,
                    );
                    ExpectedDiagnostic::HtmlEndTagWithOpenSelectedOrdinaryElements {
                        token,
                        trigger: authored("html-end"),
                    }
                }
                other => panic!("unexpected diagnostic {other:?} for a TC-S4 source"),
            }
        })
        .collect()
}

/// The recorded matching closures, each named by its subject's own authored
/// start tag.
fn project_closures(analysis: &HtmlDocumentShellAnalysis) -> Vec<ExpectedClosure> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| {
            let HtmlTreeActionKind::ClosedSelectedOrdinaryElement { node, name } = action.kind()
            else {
                return None;
            };
            let subject = analysis.node(*node).expect("closure subject resolves");
            let HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) = subject.kind()
            else {
                panic!("a closure subject must be a selected ordinary element")
            };
            assert_eq!(selected.name(), *name, "closure names the subject's name");
            Some(ExpectedClosure {
                subject_start_tag: span(selected.complete()),
                trigger_token: action.trigger().token_index(),
                trigger: span(
                    action
                        .trigger()
                        .authored_boundary()
                        .expect("a closure trigger is always an authored end tag"),
                ),
            })
        })
        .collect()
}

/// The recorded heterogeneous recovery pops, each named by both endpoints'
/// own authored start tags.
fn project_recoveries(analysis: &HtmlDocumentShellAnalysis) -> Vec<ExpectedRecovery> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| {
            let HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { node, target } =
                action.kind()
            else {
                return None;
            };
            assert_ne!(node, target, "a recovery never targets its own subject");
            Some(ExpectedRecovery {
                subject_start_tag: selected_start_tag(analysis, *node),
                target_start_tag: selected_start_tag(analysis, *target),
                trigger_token: action.trigger().token_index(),
                trigger: span(
                    action
                        .trigger()
                        .authored_boundary()
                        .expect("a recovery trigger is always an authored end tag"),
                ),
            })
        })
        .collect()
}

fn project_completion(analysis: &HtmlDocumentShellAnalysis) -> ExpectedCompletion {
    match analysis.completion() {
        HtmlTreeCompletion::Complete => ExpectedCompletion::Complete,
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete) => {
            ExpectedCompletion::LowerLayerIncomplete
        }
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => ExpectedCompletion::Unsupported {
            capability: unsupported.capability(),
            token: unsupported.trigger().token_index(),
            trigger: unsupported.trigger().authored_boundary().map(span),
        },
    }
}

// ---------------------------------------------------------------------------
// Hand-authored expectation helpers
// ---------------------------------------------------------------------------

fn synthesized(name: &'static str, children: Vec<ExpectedNode>) -> ExpectedNode {
    ExpectedNode::Shell {
        name,
        origin: None,
        children,
    }
}

fn authored_shell(
    name: &'static str,
    complete: Span,
    raw_name: Span,
    children: Vec<ExpectedNode>,
) -> ExpectedNode {
    ExpectedNode::Shell {
        name,
        origin: Some((complete, raw_name)),
        children,
    }
}

fn div(complete: Span, raw_name: Span, children: Vec<ExpectedNode>) -> ExpectedNode {
    ExpectedNode::SelectedOrdinary {
        name: "div",
        complete,
        raw_name,
        children,
    }
}

fn section(complete: Span, raw_name: Span, children: Vec<ExpectedNode>) -> ExpectedNode {
    ExpectedNode::SelectedOrdinary {
        name: "section",
        complete,
        raw_name,
        children,
    }
}

fn text(interpreted: &str, contributions: &[Span]) -> ExpectedNode {
    ExpectedNode::Text {
        interpreted: interpreted.to_owned(),
        contributions: contributions.to_vec(),
    }
}

/// `Document -> html(head, body(..))` with an authored `<body>` at `(0, 6)`.
fn shell_with_authored_body(body_children: Vec<ExpectedNode>) -> ExpectedNode {
    ExpectedNode::Document(vec![synthesized(
        "html",
        vec![
            synthesized("head", vec![]),
            authored_shell("body", (0, 6), (1, 5), body_children),
        ],
    )])
}

/// The four predecessor actions every `<body>`-opening source commits before
/// its first selected token, with their exact trigger token indices.
fn shell_prelude_actions() -> Vec<(ExpectedAction, usize)> {
    vec![
        (ExpectedAction::Reprocessed, 0),
        (ExpectedAction::InsertedSynthesizedShell("html"), 0),
        (ExpectedAction::Reprocessed, 0),
        (ExpectedAction::InsertedSynthesizedShell("head"), 0),
        (ExpectedAction::Reprocessed, 0),
        (ExpectedAction::ClosedShellByImpliedToken("head"), 0),
        (ExpectedAction::Reprocessed, 0),
        (ExpectedAction::InsertedAuthoredShell("body"), 0),
    ]
}

fn missing_doctype_at(token: usize, trigger: Span) -> ExpectedDiagnostic {
    ExpectedDiagnostic::MissingDoctype { token, trigger }
}

fn closure(subject_start_tag: Span, trigger_token: usize, trigger: Span) -> ExpectedClosure {
    ExpectedClosure {
        subject_start_tag,
        trigger_token,
        trigger,
    }
}

fn recovery(
    subject_start_tag: Span,
    target_start_tag: Span,
    trigger_token: usize,
    trigger: Span,
) -> ExpectedRecovery {
    ExpectedRecovery {
        subject_start_tag,
        target_start_tag,
        trigger_token,
        trigger,
    }
}

/// Compares one production run against its complete hand-authored
/// expectation.
fn check(expected: &ExpectedRun) {
    let analysis = analyze(expected.source);
    assert_eq!(
        project_tree(&analysis, analysis.root()),
        expected.tree,
        "{}: constructed tree",
        expected.id
    );
    assert_eq!(
        project_actions(&analysis),
        expected.actions,
        "{}: committed actions and dispositions",
        expected.id
    );
    assert_eq!(
        project_diagnostics(&analysis),
        expected.diagnostics,
        "{}: parse diagnostics",
        expected.id
    );
    assert_eq!(
        project_closures(&analysis),
        expected.closures,
        "{}: matching closure evidence",
        expected.id
    );
    assert_eq!(
        project_recoveries(&analysis),
        expected.recoveries,
        "{}: heterogeneous recovery evidence",
        expected.id
    );
    assert_eq!(
        analysis.node_count(),
        expected.node_count,
        "{}: admitted constructed identities",
        expected.id
    );
    assert_eq!(
        analysis.coverage().committed_end(),
        expected.committed_end,
        "{}: committed tree coverage",
        expected.id
    );
    assert_eq!(
        analysis.coverage().processed_tokens(),
        expected.processed_tokens,
        "{}: processed tokens",
        expected.id
    );
    assert_eq!(
        project_completion(&analysis),
        expected.completion,
        "{}: effective completion",
        expected.id
    );
}

// ---------------------------------------------------------------------------
// PS1 — a `section` is constructed and matched exactly like the accepted `div`
// ---------------------------------------------------------------------------

fn ps1() -> ExpectedRun {
    // `<body><section></section>`
    //  0      6        15       25
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 2),
        (ExpectedAction::Stopped, 3),
    ]);
    ExpectedRun {
        id: "PS1",
        source: "<body><section></section>",
        tree: shell_with_authored_body(vec![section((6, 15), (7, 14), vec![])]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![closure((6, 15), 2, (15, 25))],
        recoveries: vec![],
        node_count: 5,
        committed_end: 25,
        processed_tokens: 4,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps1_a_section_is_authored_and_matched_like_the_predecessor_div() {
    check(&ps1());
}

#[test]
fn ps1_a_current_target_end_records_no_recovery_and_no_misnested_diagnostic() {
    // The load-bearing negative half of the theorem: the accepted TC-S3 shape
    // must not acquire recovery semantics merely because the domain grew.
    for source in [
        "<body><section></section>",
        "<body><div></div>",
        "<body><section><section></section></section>",
        "<body><div><div></div></div>",
        "<body><section><div></div></section>",
    ] {
        let analysis = analyze(source);
        assert!(
            project_recoveries(&analysis).is_empty(),
            "{source:?}: a current-target end recovers nothing"
        );
        assert!(
            !project_diagnostics(&analysis)
                .iter()
                .any(|diagnostic| matches!(
                    diagnostic,
                    ExpectedDiagnostic::MisnestedSelectedOrdinaryEndTag { .. }
                )),
            "{source:?}: a current-target end is not misnested"
        );
    }
}

// ---------------------------------------------------------------------------
// PS2 — exact raw mixed-case `section` provenance
// ---------------------------------------------------------------------------

fn ps2() -> ExpectedRun {
    // `<body><SeCtIoN>x</sEcTiOn>`
    //  0      6       15 16      26
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (ExpectedAction::InsertedText, 2),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 3),
        (ExpectedAction::Stopped, 4),
    ]);
    ExpectedRun {
        id: "PS2",
        source: "<body><SeCtIoN>x</sEcTiOn>",
        tree: shell_with_authored_body(vec![section(
            (6, 15),
            (7, 14),
            vec![text("x", &[(15, 16)])],
        )]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![closure((6, 15), 3, (16, 26))],
        recoveries: vec![],
        node_count: 6,
        committed_end: 26,
        processed_tokens: 5,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps2_mixed_case_section_keeps_its_exact_authored_spelling() {
    check(&ps2());
    // The interpreted closed name is `Section` while the retained raw name is
    // the authored `SeCtIoN`; the two are separate evidence and neither is
    // derived from the other.
    let analysis = analyze("<body><SeCtIoN>x</sEcTiOn>");
    let source = "<body><SeCtIoN>x</sEcTiOn>";
    let raw = analysis
        .nodes_in_creation_order()
        .into_iter()
        .find_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) => {
                Some(span(selected.raw_name()))
            }
            _ => None,
        })
        .expect("one selected ordinary element");
    assert_eq!(&source[raw.0..raw.1], "SeCtIoN");
}

// ---------------------------------------------------------------------------
// PS3 — nested same-name `section`
// ---------------------------------------------------------------------------

fn ps3() -> ExpectedRun {
    // `<body><section><section></section></section>`
    //  0      6       15       24       34         44
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            2,
        ),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 3),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 4),
        (ExpectedAction::Stopped, 5),
    ]);
    ExpectedRun {
        id: "PS3",
        source: "<body><section><section></section></section>",
        tree: shell_with_authored_body(vec![section(
            (6, 15),
            (7, 14),
            vec![section((15, 24), (16, 23), vec![])],
        )]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        // Innermost first: each end tag matches its own nearest target, which
        // is the current node in both cases.
        closures: vec![
            closure((15, 24), 3, (24, 34)),
            closure((6, 15), 4, (34, 44)),
        ],
        recoveries: vec![],
        node_count: 6,
        committed_end: 44,
        processed_tokens: 6,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps3_nested_sections_close_innermost_first_without_recovery() {
    check(&ps3());
}

// ---------------------------------------------------------------------------
// PS4 — the load-bearing heterogeneous case: `section -> div -> </section>`
// ---------------------------------------------------------------------------

fn ps4() -> ExpectedRun {
    // `<body><section><div></section>`
    //  0      6       15    20       30
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 2),
        (ExpectedAction::RecoveryPoppedSelectedOrdinary, 3),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 3),
        (ExpectedAction::Stopped, 4),
    ]);
    ExpectedRun {
        id: "PS4",
        source: "<body><section><div></section>",
        // Recovery pops the open-element stack; it does not reparent. The
        // `div` stays exactly where it was constructed.
        tree: shell_with_authored_body(vec![section(
            (6, 15),
            (7, 14),
            vec![div((15, 20), (16, 19), vec![])],
        )]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::MisnestedSelectedOrdinaryEndTag {
                token: 3,
                trigger: (20, 30),
            },
        ],
        // Exactly one matching closure, and it names the `section`.
        closures: vec![closure((6, 15), 3, (20, 30))],
        // Exactly one recovery pop, and it names the `div` as subject and the
        // `section` as target — under the same authored `</section>`.
        recoveries: vec![recovery((15, 20), (6, 15), 3, (20, 30))],
        node_count: 6,
        committed_end: 30,
        processed_tokens: 5,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps4_a_misnested_end_tag_recovers_the_suffix_and_closes_its_target() {
    check(&ps4());
}

#[test]
fn ps4_the_recovered_div_receives_no_fabricated_matching_closure() {
    // The whole point of the two relations: no matching end tag caused the
    // `div` to be removed, so it must never appear as a closure subject —
    // whether or not a `</div>` appears later in some other source.
    let analysis = analyze("<body><section><div></section>");
    let closures = project_closures(&analysis);
    assert_eq!(closures.len(), 1);
    assert_eq!(closures[0].subject_start_tag, (6, 15), "the target closes");
    assert!(
        !closures
            .iter()
            .any(|closed| closed.subject_start_tag == (15, 20)),
        "an intervening element never receives a matching closure"
    );

    let recoveries = project_recoveries(&analysis);
    assert_eq!(recoveries.len(), 1);
    assert_eq!(recoveries[0].subject_start_tag, (15, 20));
    assert!(
        !recoveries
            .iter()
            .any(|popped| popped.subject_start_tag == (6, 15)),
        "the target is closed, never recovery-popped"
    );
}

#[test]
fn ps4_one_authored_end_tag_carries_both_relations_and_authors_neither_node() {
    let analysis = analyze("<body><section><div></section>");
    let closures = project_closures(&analysis);
    let recoveries = project_recoveries(&analysis);
    // One trigger, two distinct relations, one diagnostic.
    assert_eq!(closures[0].trigger, (20, 30));
    assert_eq!(recoveries[0].trigger, (20, 30));
    assert_eq!(closures[0].trigger_token, recoveries[0].trigger_token);

    // And that trigger is the authored origin of nothing.
    for node in analysis.nodes_in_creation_order() {
        if let Some(HtmlAuthoredSource::StartTag { complete, .. }) = node.authored_source() {
            assert_ne!(
                span(complete),
                (20, 30),
                "an end tag is never a node's authored origin"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// PS5 — the mirrored heterogeneous case: `div -> section -> </div>`
// ---------------------------------------------------------------------------

fn ps5() -> ExpectedRun {
    // `<body><div><section></div>`
    //  0      6    11       20    26
    let mut actions = shell_prelude_actions();
    actions.extend([
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1),
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            2,
        ),
        (ExpectedAction::RecoveryPoppedSelectedOrdinary, 3),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 3),
        (ExpectedAction::Stopped, 4),
    ]);
    ExpectedRun {
        id: "PS5",
        source: "<body><div><section></div>",
        tree: shell_with_authored_body(vec![div(
            (6, 11),
            (7, 10),
            vec![section((11, 20), (12, 19), vec![])],
        )]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::MisnestedSelectedOrdinaryEndTag {
                token: 3,
                trigger: (20, 26),
            },
        ],
        closures: vec![closure((6, 11), 3, (20, 26))],
        recoveries: vec![recovery((11, 20), (6, 11), 3, (20, 26))],
        node_count: 6,
        committed_end: 26,
        processed_tokens: 5,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps5_the_recovery_relation_is_symmetric_across_the_closed_domain() {
    check(&ps5());
}

// ---------------------------------------------------------------------------
// PS6 — several intervening elements, popped current-first
// ---------------------------------------------------------------------------

fn ps6() -> ExpectedRun {
    // `<body><section><div><div></section>`
    //  0      6       15    20    25       35
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 2),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 3),
        (ExpectedAction::RecoveryPoppedSelectedOrdinary, 4),
        (ExpectedAction::RecoveryPoppedSelectedOrdinary, 4),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 4),
        (ExpectedAction::Stopped, 5),
    ]);
    ExpectedRun {
        id: "PS6",
        source: "<body><section><div><div></section>",
        tree: shell_with_authored_body(vec![section(
            (6, 15),
            (7, 14),
            vec![div(
                (15, 20),
                (16, 19),
                vec![div((20, 25), (21, 24), vec![])],
            )],
        )]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            // Exactly one, however many pops.
            ExpectedDiagnostic::MisnestedSelectedOrdinaryEndTag {
                token: 4,
                trigger: (25, 35),
            },
        ],
        closures: vec![closure((6, 15), 4, (25, 35))],
        // Current-first: the innermost `div` (20, 25) leaves first.
        recoveries: vec![
            recovery((20, 25), (6, 15), 4, (25, 35)),
            recovery((15, 20), (6, 15), 4, (25, 35)),
        ],
        node_count: 7,
        committed_end: 35,
        processed_tokens: 6,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps6_multiple_intervening_elements_pop_current_first_under_one_diagnostic() {
    check(&ps6());
}

// ---------------------------------------------------------------------------
// PS7 — the target is the *nearest* same-name element, not the outermost
// ---------------------------------------------------------------------------

fn ps7() -> ExpectedRun {
    // `<body><section><section><div></section></section>`
    //  0      6       15       24    29       39         49
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            2,
        ),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 3),
        (ExpectedAction::RecoveryPoppedSelectedOrdinary, 4),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 4),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 5),
        (ExpectedAction::Stopped, 6),
    ]);
    ExpectedRun {
        id: "PS7",
        source: "<body><section><section><div></section></section>",
        tree: shell_with_authored_body(vec![section(
            (6, 15),
            (7, 14),
            vec![section(
                (15, 24),
                (16, 23),
                vec![div((24, 29), (25, 28), vec![])],
            )],
        )]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::MisnestedSelectedOrdinaryEndTag {
                token: 4,
                trigger: (29, 39),
            },
        ],
        // The first `</section>` closes the *inner* section (15, 24); the
        // second then closes the outer one as an ordinary current-target end.
        closures: vec![
            closure((15, 24), 4, (29, 39)),
            closure((6, 15), 5, (39, 49)),
        ],
        recoveries: vec![recovery((24, 29), (15, 24), 4, (29, 39))],
        node_count: 7,
        committed_end: 49,
        processed_tokens: 7,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps7_the_recovery_target_is_the_nearest_same_name_element() {
    check(&ps7());
    let analysis = analyze("<body><section><section><div></section></section>");
    assert_eq!(
        project_recoveries(&analysis)[0].target_start_tag,
        (15, 24),
        "the nearest open `section`, not the outermost, is the target"
    );
}

// ---------------------------------------------------------------------------
// PS8 / PS9 — unmatched selected ends stay the accepted ignored cell
// ---------------------------------------------------------------------------

fn ps8() -> ExpectedRun {
    // `<body></section>`
    //  0      6         16
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::IgnoredUnmatchedSelectedOrdinaryEndTag("section"),
            1,
        ),
        (ExpectedAction::Stopped, 2),
    ]);
    ExpectedRun {
        id: "PS8",
        source: "<body></section>",
        tree: shell_with_authored_body(vec![]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::UnmatchedSelectedOrdinaryEndTag {
                token: 1,
                trigger: (6, 16),
            },
        ],
        closures: vec![],
        recoveries: vec![],
        node_count: 4,
        committed_end: 16,
        processed_tokens: 3,
        completion: ExpectedCompletion::Complete,
    }
}

fn ps9() -> ExpectedRun {
    // `<body><section></div>` — a `div` end with only a `section` open is
    // absent-target, not misnested: names must match to select a target.
    //  0      6       15     21
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (
            ExpectedAction::IgnoredUnmatchedSelectedOrdinaryEndTag("div"),
            2,
        ),
        (ExpectedAction::Stopped, 3),
    ]);
    ExpectedRun {
        id: "PS9",
        source: "<body><section></div>",
        tree: shell_with_authored_body(vec![section((6, 15), (7, 14), vec![])]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::UnmatchedSelectedOrdinaryEndTag {
                token: 2,
                trigger: (15, 21),
            },
            ExpectedDiagnostic::OpenSelectedOrdinaryElementAtEndOfFile { token: 3 },
        ],
        closures: vec![],
        recoveries: vec![],
        node_count: 5,
        committed_end: 21,
        processed_tokens: 4,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps8_an_unmatched_section_end_is_a_diagnostic_and_an_ignored_disposition() {
    check(&ps8());
}

#[test]
fn ps9_a_differently_named_end_selects_no_target_and_recovers_nothing() {
    check(&ps9());
}

#[test]
fn an_absent_target_leaves_the_selected_state_exactly_as_it_was() {
    for source in ["<body></section>", "<body></div>", "<body><section></div>"] {
        let analysis = analyze(source);
        assert!(project_recoveries(&analysis).is_empty(), "{source:?}");
        assert!(project_closures(&analysis).is_empty(), "{source:?}");
        assert!(
            analysis.is_complete(),
            "{source:?}: a diagnostic is not incompleteness"
        );
    }
}

// ---------------------------------------------------------------------------
// PS10 — a selected element reopens normally after a recovery
// ---------------------------------------------------------------------------

fn ps10() -> ExpectedRun {
    // `<body><section><div></section><div></div>`
    //  0      6       15    20       30    35    41
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 2),
        (ExpectedAction::RecoveryPoppedSelectedOrdinary, 3),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 3),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 4),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 5),
        (ExpectedAction::Stopped, 6),
    ]);
    ExpectedRun {
        id: "PS10",
        source: "<body><section><div></section><div></div>",
        // The second `div` is a `body` sibling of the `section`, because the
        // committed stack change moved the insertion point.
        tree: shell_with_authored_body(vec![
            section((6, 15), (7, 14), vec![div((15, 20), (16, 19), vec![])]),
            div((30, 35), (31, 34), vec![]),
        ]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::MisnestedSelectedOrdinaryEndTag {
                token: 3,
                trigger: (20, 30),
            },
        ],
        closures: vec![
            closure((6, 15), 3, (20, 30)),
            closure((30, 35), 5, (35, 41)),
        ],
        recoveries: vec![recovery((15, 20), (6, 15), 3, (20, 30))],
        node_count: 7,
        committed_end: 41,
        processed_tokens: 7,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps10_construction_continues_normally_after_a_recovery() {
    check(&ps10());
}

// ---------------------------------------------------------------------------
// PS11 — text parentage follows the committed selected-stack change
// ---------------------------------------------------------------------------

fn ps11() -> ExpectedRun {
    // `<body><section>a<div>b</section>c`
    //  0      6       15 16   21 22     32 33
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (ExpectedAction::InsertedText, 2),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 3),
        (ExpectedAction::InsertedText, 4),
        (ExpectedAction::RecoveryPoppedSelectedOrdinary, 5),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 5),
        (ExpectedAction::InsertedText, 6),
        (ExpectedAction::Stopped, 7),
    ]);
    ExpectedRun {
        id: "PS11",
        source: "<body><section>a<div>b</section>c",
        tree: shell_with_authored_body(vec![
            section(
                (6, 15),
                (7, 14),
                vec![
                    text("a", &[(15, 16)]),
                    div((16, 21), (17, 20), vec![text("b", &[(21, 22)])]),
                ],
            ),
            text("c", &[(32, 33)]),
        ]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::MisnestedSelectedOrdinaryEndTag {
                token: 5,
                trigger: (22, 32),
            },
        ],
        closures: vec![closure((6, 15), 5, (22, 32))],
        recoveries: vec![recovery((16, 21), (6, 15), 5, (22, 32))],
        node_count: 9,
        committed_end: 33,
        processed_tokens: 8,
        completion: ExpectedCompletion::Complete,
    }
}

fn ps12() -> ExpectedRun {
    // `<body><section>a</div>b</section>` — the ignored `</div>` splits the
    // authored character runs, so the existing per-parent coalescing path is
    // exercised unchanged inside a `section`.
    //  0      6       15 16     22 23     33
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (ExpectedAction::InsertedText, 2),
        (
            ExpectedAction::IgnoredUnmatchedSelectedOrdinaryEndTag("div"),
            3,
        ),
        (ExpectedAction::AppendedText, 4),
        (ExpectedAction::ClosedSelectedOrdinary("section"), 5),
        (ExpectedAction::Stopped, 6),
    ]);
    ExpectedRun {
        id: "PS12",
        source: "<body><section>a</div>b</section>",
        tree: shell_with_authored_body(vec![section(
            (6, 15),
            (7, 14),
            // One text node, two exact ordered contributions, one identity.
            vec![text("ab", &[(15, 16), (22, 23)])],
        )]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::UnmatchedSelectedOrdinaryEndTag {
                token: 3,
                trigger: (16, 22),
            },
        ],
        closures: vec![closure((6, 15), 5, (23, 33))],
        recoveries: vec![],
        node_count: 6,
        committed_end: 33,
        processed_tokens: 7,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps11_text_parentage_follows_the_committed_recovery() {
    check(&ps11());
}

#[test]
fn ps12_text_coalescing_inside_a_section_reuses_the_existing_path() {
    check(&ps12());
}

// ---------------------------------------------------------------------------
// PS13 — end of file with a mixed open selected suffix
// ---------------------------------------------------------------------------

fn ps13() -> ExpectedRun {
    // `<body><section><div>`
    //  0      6       15    20
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        ),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 2),
        (ExpectedAction::Stopped, 3),
    ]);
    ExpectedRun {
        id: "PS13",
        source: "<body><section><div>",
        tree: shell_with_authored_body(vec![section(
            (6, 15),
            (7, 14),
            vec![div((15, 20), (16, 19), vec![])],
        )]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::OpenSelectedOrdinaryElementAtEndOfFile { token: 3 },
        ],
        // Nothing is popped, closed, recovered, or synthesized at end of file.
        closures: vec![],
        recoveries: vec![],
        node_count: 6,
        committed_end: 20,
        processed_tokens: 4,
        completion: ExpectedCompletion::Complete,
    }
}

#[test]
fn ps13_end_of_file_fabricates_no_closure_and_no_recovery() {
    check(&ps13());
    let analysis = analyze("<body><section><div>");
    for action in analysis.actions() {
        assert!(
            !matches!(
                action.kind(),
                HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
                    | HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
            ),
            "end of file must not end a selected element's open lifetime"
        );
    }
}

// ---------------------------------------------------------------------------
// PS14-PS17 — refusal is transactional across the grown domain
// ---------------------------------------------------------------------------

/// The parts of a refusal expectation that vary between cases. Every refusal
/// shares the same shell prelude, the same single missing-DOCTYPE diagnostic,
/// and — the point of these cases — no closure and no recovery at all.
struct RefusalCase {
    id: &'static str,
    source: &'static str,
    body_children: Vec<ExpectedNode>,
    tail: Vec<(ExpectedAction, usize)>,
    node_count: usize,
    committed_end: usize,
    processed_tokens: usize,
    completion: ExpectedCompletion,
}

fn refusal_run(case: RefusalCase) -> ExpectedRun {
    let mut actions = shell_prelude_actions();
    actions.extend(case.tail);
    ExpectedRun {
        id: case.id,
        source: case.source,
        tree: shell_with_authored_body(case.body_children),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![],
        recoveries: vec![],
        node_count: case.node_count,
        committed_end: case.committed_end,
        processed_tokens: case.processed_tokens,
        completion: case.completion,
    }
}

#[test]
fn ps14_an_attributed_section_start_tag_refuses_transactionally() {
    check(&refusal_run(RefusalCase {
        id: "PS14",
        source: "<body><section id=x>",
        body_children: vec![],
        tail: vec![],
        node_count: 4,
        committed_end: 6,
        processed_tokens: 1,
        completion: ExpectedCompletion::Unsupported {
            capability: HtmlTreeCapability::SelectedOrdinaryTagAttribute,
            token: 1,
            trigger: Some((6, 20)),
        },
    }));
}

#[test]
fn ps15_a_self_closing_section_start_tag_refuses_transactionally() {
    check(&refusal_run(RefusalCase {
        id: "PS15",
        source: "<body><section/>",
        body_children: vec![],
        tail: vec![],
        node_count: 4,
        committed_end: 6,
        processed_tokens: 1,
        completion: ExpectedCompletion::Unsupported {
            capability: HtmlTreeCapability::SelfClosingSelectedOrdinaryTag,
            token: 1,
            trigger: Some((6, 16)),
        },
    }));
}

#[test]
fn ps16_a_section_outside_in_body_refuses_transactionally() {
    check(&refusal_run(RefusalCase {
        id: "PS16",
        source: "<body></body><section>",
        body_children: vec![],
        tail: vec![(ExpectedAction::AcknowledgedShellEndTag("body"), 1)],
        node_count: 4,
        committed_end: 13,
        processed_tokens: 2,
        completion: ExpectedCompletion::Unsupported {
            capability: HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
            token: 2,
            trigger: Some((13, 22)),
        },
    }));
}

#[test]
fn ps17_a_body_close_over_an_open_section_advances_only_the_tc_s7_cell() {
    let analysis = analyze("<body><section></body>");
    assert!(analysis.is_complete());
    assert_eq!(analysis.node_count(), 5);
    assert_eq!(analysis.coverage().committed_end(), 22);
    assert_eq!(analysis.coverage().processed_tokens(), 4);
    assert_eq!(
        project_diagnostics(&analysis)
            .into_iter()
            .filter(|diagnostic| matches!(
                diagnostic,
                ExpectedDiagnostic::BodyEndTagWithOpenSelectedOrdinaryElements { .. }
            ))
            .collect::<Vec<_>>(),
        vec![
            ExpectedDiagnostic::BodyEndTagWithOpenSelectedOrdinaryElements {
                token: 2,
                trigger: (15, 22),
            },
        ]
    );
    assert!(project_closures(&analysis).is_empty());
    assert!(project_recoveries(&analysis).is_empty());
    assert!(analysis.actions().iter().any(|action| matches!(
        action.kind(),
        HtmlTreeActionKind::AcknowledgedShellEndTag {
            name: HtmlShellElementName::Body,
        } if action.trigger().token_index() == 2
    )));
}

#[test]
fn body_and_html_end_advance_their_exact_mixed_selected_stack_cells() {
    for source in [
        "<body><section></body>",
        "<body><section><div></body>",
        "<body><div><section></body>",
    ] {
        let analysis = analyze(source);
        assert!(analysis.is_complete(), "{source:?}");
        assert_eq!(
            analysis
                .diagnostics()
                .iter()
                .filter(|diagnostic| diagnostic.code()
                    == HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements)
                .count(),
            1,
            "{source:?}"
        );
        assert!(project_recoveries(&analysis).is_empty());
        assert!(project_closures(&analysis).is_empty());
    }

    let source = "<body><section><div></html>";
    let analysis = analyze(source);
    assert_eq!(project_completion(&analysis), ExpectedCompletion::Complete);
    assert_eq!(analysis.coverage().committed_end(), source.len());
    assert_eq!(analysis.coverage().processed_tokens(), 5);
    assert_eq!(
        project_diagnostics(&analysis)
            .into_iter()
            .filter(|diagnostic| matches!(
                diagnostic,
                ExpectedDiagnostic::HtmlEndTagWithOpenSelectedOrdinaryElements { .. }
            ))
            .collect::<Vec<_>>(),
        vec![
            ExpectedDiagnostic::HtmlEndTagWithOpenSelectedOrdinaryElements {
                token: 3,
                trigger: (20, source.len()),
            },
        ]
    );
    assert_eq!(
        project_actions(&analysis)
            .into_iter()
            .filter(|(_, token)| *token == 3)
            .collect::<Vec<_>>(),
        vec![
            (ExpectedAction::Reprocessed, 3),
            (ExpectedAction::AcknowledgedShellEndTag("html"), 3),
        ]
    );
    assert!(project_recoveries(&analysis).is_empty());
    assert!(project_closures(&analysis).is_empty());
}

#[test]
fn ps18_lower_layer_incompleteness_is_never_upgraded() {
    check(&refusal_run(RefusalCase {
        id: "PS18",
        source: "<body><section>&amp;",
        body_children: vec![section((6, 15), (7, 14), vec![])],
        tail: vec![(
            ExpectedAction::InsertedAuthoredSelectedOrdinary("section"),
            1,
        )],
        node_count: 5,
        committed_end: 15,
        processed_tokens: 2,
        completion: ExpectedCompletion::LowerLayerIncomplete,
    }));
}

// ---------------------------------------------------------------------------
// Identity, source binding, and storage independence
// ---------------------------------------------------------------------------

/// Every TC-S4 source this module reasons about, so the invariant families
/// below can be stated once over all of them.
fn all_sources() -> Vec<&'static str> {
    vec![
        "<body><section></section>",
        "<body><SeCtIoN>x</sEcTiOn>",
        "<body><section><section></section></section>",
        "<body><section><div></section>",
        "<body><div><section></div>",
        "<body><section><div><div></section>",
        "<body><section><section><div></section></section>",
        "<body></section>",
        "<body><section></div>",
        "<body><section><div></section><div></div>",
        "<body><section>a<div>b</section>c",
        "<body><section>a</div>b</section>",
        "<body><section><div>",
        "<body><SeCtIoN><DiV></SeCtIoN>",
        "<body><div><section><div></div></section></div>",
        "<body><section><div></section></section>",
    ]
}

#[test]
fn semantic_meaning_is_identical_under_differing_source_ids() {
    for source in all_sources() {
        // Distinct, non-zero, non-sequential source identities over the exact
        // same bytes must not change one bit of durable meaning.
        let first = analyze_with(source, 7);
        let second = analyze_with(source, 4_294_967_311);
        assert_eq!(
            project_tree(&first, first.root()),
            project_tree(&second, second.root()),
            "{source:?}: tree"
        );
        assert_eq!(
            project_actions(&first),
            project_actions(&second),
            "{source:?}: actions"
        );
        assert_eq!(
            project_diagnostics(&first),
            project_diagnostics(&second),
            "{source:?}: diagnostics"
        );
        assert_eq!(
            project_closures(&first),
            project_closures(&second),
            "{source:?}: closures"
        );
        assert_eq!(
            project_recoveries(&first),
            project_recoveries(&second),
            "{source:?}: recoveries"
        );
        assert_eq!(
            project_completion(&first),
            project_completion(&second),
            "{source:?}: completion"
        );
    }
}

#[test]
fn relations_survive_private_storage_replacement() {
    for source in all_sources() {
        let analysis = analyze(source);
        let expected_tree = project_tree(&analysis, analysis.root());
        let expected_closures = project_closures(&analysis);
        let expected_recoveries = project_recoveries(&analysis);

        // Reversing private node storage changes no identity and no
        // relationship, so nothing above may notice.
        let perturbed = analyze(source).with_reversed_storage();
        assert_eq!(
            project_tree(&perturbed, perturbed.root()),
            expected_tree,
            "{source:?}: tree survives storage replacement"
        );
        assert_eq!(
            project_closures(&perturbed),
            expected_closures,
            "{source:?}: closures survive storage replacement"
        );
        assert_eq!(
            project_recoveries(&perturbed),
            expected_recoveries,
            "{source:?}: recoveries survive storage replacement"
        );
        assert_eq!(perturbed.node_count(), analysis.node_count());
    }
}

#[test]
fn identity_admission_counts_only_committed_creation_events() {
    // A start tag admits exactly one identity; a recovery pop, a matching
    // closure, a misnested diagnostic, an ignored end, an append, end of
    // file, and a refusal admit none.
    for (source, expected_nodes) in [
        // root + html + head + body = 4 structural nodes in every case.
        ("<body><section></section>", 5usize),
        ("<body><section><div></section>", 6),
        ("<body><section><div><div></section>", 7),
        ("<body><section><section><div></section></section>", 7),
        ("<body></section>", 4),
        ("<body><section>a</div>b</section>", 6),
        ("<body><section><div></section><div></div>", 7),
        ("<body><section id=x>", 4),
        ("<body><section/>", 4),
    ] {
        let analysis = analyze(source);
        assert_eq!(analysis.node_count(), expected_nodes, "{source:?}");
        // Identity is gap-free committed creation order, checked without
        // asserting any raw encoding.
        let ordered = analysis.nodes_in_creation_order();
        assert_eq!(ordered.len(), expected_nodes, "{source:?}");
        for window in ordered.windows(2) {
            assert!(window[0].id() < window[1].id(), "{source:?}");
        }
    }
}

#[test]
fn a_recovered_elements_origin_is_never_an_end_tag_or_a_trigger() {
    for source in all_sources() {
        let analysis = analyze(source);
        let trigger_spans: Vec<Span> = analysis
            .actions()
            .iter()
            .filter(|action| {
                matches!(
                    action.kind(),
                    HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
                        | HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
                )
            })
            .filter_map(|action| action.trigger().authored_boundary().map(span))
            .collect();
        for node in analysis.nodes_in_creation_order() {
            let Some(HtmlAuthoredSource::StartTag { complete, .. }) = node.authored_source() else {
                continue;
            };
            assert!(
                !trigger_spans.contains(&span(complete)),
                "{source:?}: a closure or recovery trigger is never an authored origin"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The proved cells, and only the proved cells
// ---------------------------------------------------------------------------

#[test]
fn predecessor_capability_meanings_are_unchanged_and_apply_to_section() {
    // The accepted selected capability vocabulary covers `section` exactly as
    // it covers `div`; no new capability was needed and no old one moved.
    for (source, expected) in [
        (
            "<body><section id=x>",
            HtmlTreeCapability::SelectedOrdinaryTagAttribute,
        ),
        (
            "<body><div id=x>",
            HtmlTreeCapability::SelectedOrdinaryTagAttribute,
        ),
        (
            "<body><section/>",
            HtmlTreeCapability::SelfClosingSelectedOrdinaryTag,
        ),
        (
            "<body><div/>",
            HtmlTreeCapability::SelfClosingSelectedOrdinaryTag,
        ),
        (
            "<section>",
            HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
        ),
        (
            "<div>",
            HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
        ),
        (
            "</section>",
            HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
        ),
        // Non-P names outside the selected-ordinary and Paragraph domains
        // keep the frozen unproved-name meaning.
        ("<body><span>", HtmlTreeCapability::NonShellElementTag),
        ("<body><article>", HtmlTreeCapability::NonShellElementTag),
        ("<body></article>", HtmlTreeCapability::NonShellElementTag),
        ("<body a>", HtmlTreeCapability::ShellTagAttribute),
        ("<body/>", HtmlTreeCapability::SelfClosingShellTag),
    ] {
        let analysis = analyze(source);
        let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) = analysis.completion()
        else {
            panic!("{source:?}: expected explicit tree-unsupported evidence");
        };
        assert_eq!(unsupported.capability(), expected, "{source:?}");
    }
}

#[test]
fn the_selected_domain_is_closed_at_div_and_section() {
    // Anything outside the dedicated Paragraph successor that merely looks
    // like a block element stays outside the selected ordinary domain.
    for name in [
        "span", "article", "aside", "main", "nav", "header", "footer", "sections", "divs", "sec",
        "SECTIONS",
    ] {
        let source = format!("<body><{name}>");
        let analysis = analyze_with(&source, 1);
        let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) = analysis.completion()
        else {
            panic!("{source:?}: expected explicit tree-unsupported evidence");
        };
        assert_eq!(
            unsupported.capability(),
            HtmlTreeCapability::NonShellElementTag,
            "{source:?}"
        );
    }

    let paragraph = analyze("<body><p>");
    assert!(paragraph.is_complete());
    assert!(
        paragraph
            .nodes_in_creation_order()
            .into_iter()
            .any(|node| matches!(
                node.kind(),
                HtmlTreeNodeKind::Element(HtmlElement::Paragraph(_))
            ))
    );
    assert_eq!(
        paragraph
            .nodes_in_creation_order()
            .into_iter()
            .filter(|node| matches!(
                node.kind(),
                HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(_))
            ))
            .count(),
        0,
        "Paragraph support is a separate domain, not a third selected-ordinary name"
    );
}

// ---------------------------------------------------------------------------
// Bounded generated sequences
// ---------------------------------------------------------------------------

/// Builds `<body>` followed by the given selected start tags and then one end
/// tag, so a whole family of nesting shapes can be checked against the
/// theorem without a fixture per shape.
fn selected_sequence(starts: &[&str], end: &str) -> String {
    let mut source = String::from("<body>");
    for start in starts {
        source.push('<');
        source.push_str(start);
        source.push('>');
    }
    source.push_str("</");
    source.push_str(end);
    source.push('>');
    source
}

#[test]
fn bounded_generated_heterogeneous_sequences_hold_the_theorem() {
    let names = ["div", "section"];
    let mut checked = 0usize;
    for depth in 1..=4usize {
        // Every `{div, section}` word of this depth, then each possible end.
        for encoded in 0..2usize.pow(u32::try_from(depth).expect("small depth")) {
            let starts: Vec<&str> = (0..depth)
                .map(|position| names[(encoded >> position) & 1])
                .collect();
            for end in names {
                let source = selected_sequence(&starts, end);
                let analysis = analyze_with(&source, 3);
                assert!(analysis.is_complete(), "{source:?}");

                let recoveries = project_recoveries(&analysis);
                let closures = project_closures(&analysis);
                let misnested = project_diagnostics(&analysis)
                    .into_iter()
                    .filter(|diagnostic| {
                        matches!(
                            diagnostic,
                            ExpectedDiagnostic::MisnestedSelectedOrdinaryEndTag { .. }
                        )
                    })
                    .count();

                // The nearest same-name target, computed independently here
                // from the authored word rather than from production.
                let nearest = starts.iter().rposition(|start| *start == end);
                match nearest {
                    None => {
                        assert!(recoveries.is_empty(), "{source:?}");
                        assert!(closures.is_empty(), "{source:?}");
                        assert_eq!(misnested, 0, "{source:?}");
                    }
                    Some(position) => {
                        let expected_pops = depth - 1 - position;
                        assert_eq!(recoveries.len(), expected_pops, "{source:?}");
                        assert_eq!(closures.len(), 1, "{source:?}");
                        assert_eq!(misnested, usize::from(expected_pops > 0), "{source:?}");
                        // One target for the whole group, and the pops are
                        // strictly current-first: each subject starts later in
                        // the source than the next.
                        for popped in &recoveries {
                            assert_eq!(
                                popped.target_start_tag, closures[0].subject_start_tag,
                                "{source:?}"
                            );
                            assert_eq!(popped.trigger, closures[0].trigger, "{source:?}");
                            assert_ne!(popped.subject_start_tag, popped.target_start_tag);
                        }
                        for window in recoveries.windows(2) {
                            assert!(
                                window[0].subject_start_tag.0 > window[1].subject_start_tag.0,
                                "{source:?}: recovery order is current-first"
                            );
                        }
                        // No element is both recovery-popped and closed.
                        for popped in &recoveries {
                            assert!(
                                !closures
                                    .iter()
                                    .any(|closed| closed.subject_start_tag
                                        == popped.subject_start_tag),
                                "{source:?}: the two relations stay disjoint"
                            );
                        }
                    }
                }
                checked += 1;
            }
        }
    }
    // A structural sanity check on the generator itself, not a budget.
    assert_eq!(checked, (2 + 4 + 8 + 16) * 2);
}

// ---------------------------------------------------------------------------
// The lifecycle theorem is checked at freeze, not merely satisfied
// ---------------------------------------------------------------------------
//
// Every fixture below is built by hand rather than by running production, so
// these tests prove that `freeze` itself rejects the corruption — not that the
// session happens never to produce it. Each one perturbs exactly one thing.

struct LifecycleFixture {
    source: SourceText,
    run: HtmlTokenizerRunResult,
    ids: Vec<HtmlConstructedNodeId>,
}

impl LifecycleFixture {
    fn anchor(&self, start: usize, end: usize) -> crate::SourceAnchor {
        self.source.anchor(start, end).expect("valid range")
    }
}

fn lifecycle_fixture(source_text: &str, identities: usize) -> LifecycleFixture {
    let source = SourceText::new(SourceId::new(1), source_text.to_owned());
    let run = tokenize(&source, generous_limits());
    let mut counter = HtmlConstructedIdentityCounter::new();
    let ids = (0..identities)
        .map(|_| {
            let reserved = counter.reserve().expect("identity headroom");
            counter.commit(reserved);
            reserved
        })
        .collect();
    LifecycleFixture { source, run, ids }
}

fn synthesized_shell_kind(name: HtmlShellElementName) -> HtmlTreeNodeKind {
    HtmlTreeNodeKind::Element(HtmlElement::Shell(HtmlShellElement::new(
        name,
        HtmlShellElementOrigin::Synthesized(HtmlSynthesisCause::ImpliedByDocumentStructure),
    )))
}

fn selected_kind(
    fixture: &LifecycleFixture,
    name: HtmlSelectedOrdinaryElementName,
    complete: Span,
    raw_name: Span,
) -> HtmlTreeNodeKind {
    HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(
        HtmlSelectedOrdinaryElement::new(
            name,
            fixture.anchor(complete.0, complete.1),
            fixture.anchor(raw_name.0, raw_name.1),
        ),
    ))
}

/// The `Document -> html(head, body)` prefix every fixture below shares, with
/// an authored `<body>` at `(0, 6)` and `body`'s children supplied by the
/// caller.
fn shell_prefix_nodes(
    fixture: &LifecycleFixture,
    body_children: Vec<HtmlConstructedNodeId>,
) -> Vec<HtmlTreeNode> {
    let [root, html, head, body] = fixture.ids[..4] else {
        panic!("at least four minted identities")
    };
    vec![
        HtmlTreeNode::new(root, None, vec![html], HtmlTreeNodeKind::Document),
        HtmlTreeNode::new(
            html,
            Some(root),
            vec![head, body],
            synthesized_shell_kind(HtmlShellElementName::Html),
        ),
        HtmlTreeNode::new(
            head,
            Some(html),
            vec![],
            synthesized_shell_kind(HtmlShellElementName::Head),
        ),
        HtmlTreeNode::new(
            body,
            Some(html),
            body_children,
            HtmlTreeNodeKind::Element(HtmlElement::Shell(HtmlShellElement::new(
                HtmlShellElementName::Body,
                HtmlShellElementOrigin::Authored {
                    complete: fixture.anchor(0, 6),
                    raw_name: fixture.anchor(1, 5),
                },
            ))),
        ),
    ]
}

fn insertion(
    node: HtmlConstructedNodeId,
    name: HtmlSelectedOrdinaryElementName,
    token: usize,
    trigger: crate::SourceAnchor,
) -> HtmlTreeAction {
    HtmlTreeAction::new(
        HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { node, name },
        HtmlTreeTokenTrigger::authored(token, trigger),
    )
}

fn recovery_pop(
    node: HtmlConstructedNodeId,
    target: HtmlConstructedNodeId,
    token: usize,
    trigger: crate::SourceAnchor,
) -> HtmlTreeAction {
    HtmlTreeAction::new(
        HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { node, target },
        HtmlTreeTokenTrigger::authored(token, trigger),
    )
}

fn matching_closure(
    node: HtmlConstructedNodeId,
    name: HtmlSelectedOrdinaryElementName,
    token: usize,
    trigger: crate::SourceAnchor,
) -> HtmlTreeAction {
    HtmlTreeAction::new(
        HtmlTreeActionKind::ClosedSelectedOrdinaryElement { node, name },
        HtmlTreeTokenTrigger::authored(token, trigger),
    )
}

fn misnested_diagnostic(token: usize, trigger: crate::SourceAnchor) -> HtmlTreeDiagnostic {
    HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag,
        HtmlTreeTokenTrigger::authored(token, trigger),
        HtmlTreeRecovery::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget,
    )
}

fn freeze_parts(
    fixture: &LifecycleFixture,
    parts: HtmlDocumentShellParts,
) -> Result<HtmlDocumentShellAnalysis, HtmlTreeFreezeError> {
    freeze(&fixture.source, fixture.run.clone(), parts)
}

// --- Fixture A: `<body><section><div></section></div>` ----------------------
//
// One intervening pop. The trailing, deliberately unprocessed `</div>` at
// token 4 exists so corruption tests have a real, retained, differently-named
// end tag available as false evidence.

const FIXTURE_A_SOURCE: &str = "<body><section><div></section></div>";

fn fixture_a() -> LifecycleFixture {
    lifecycle_fixture(FIXTURE_A_SOURCE, 6)
}

fn fixture_a_parts(fixture: &LifecycleFixture) -> HtmlDocumentShellParts {
    let [.., section_id, div_id] = fixture.ids[..] else {
        panic!("six minted identities")
    };
    let body = fixture.ids[3];
    let mut nodes = shell_prefix_nodes(fixture, vec![section_id]);
    nodes.push(HtmlTreeNode::new(
        section_id,
        Some(body),
        vec![div_id],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Section,
            (6, 15),
            (7, 14),
        ),
    ));
    nodes.push(HtmlTreeNode::new(
        div_id,
        Some(section_id),
        vec![],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Div,
            (15, 20),
            (16, 19),
        ),
    ));

    HtmlDocumentShellParts {
        nodes,
        root: fixture.ids[0],
        admitted_creation_events: 6,
        diagnostics: vec![misnested_diagnostic(3, fixture.anchor(20, 30))],
        actions: vec![
            insertion(
                section_id,
                HtmlSelectedOrdinaryElementName::Section,
                1,
                fixture.anchor(6, 15),
            ),
            insertion(
                div_id,
                HtmlSelectedOrdinaryElementName::Div,
                2,
                fixture.anchor(15, 20),
            ),
            recovery_pop(div_id, section_id, 3, fixture.anchor(20, 30)),
            matching_closure(
                section_id,
                HtmlSelectedOrdinaryElementName::Section,
                3,
                fixture.anchor(20, 30),
            ),
        ],
        processed_tokens: 4,
        committed_prefix_end: 30,
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
        final_open_selected_ordinary: Vec::new(),
        final_open_paragraph: None,
        final_open_style: None,
        final_open_title: None,
        final_text_mode_active: false,
        final_original_insertion_mode_retained: false,
        pending_tokenizer_feedback: false,
        coordinated_raw_text_entry_tokens: Vec::new(),
        coordinated_raw_text_close_tokens: Vec::new(),
        coordinated_rcdata_entry_tokens: Vec::new(),
        coordinated_rcdata_close_tokens: Vec::new(),
    }
}

// --- Fixture B: `<body><section><div><div></section>` ------------------------
//
// Two intervening pops, so ordering, skipping, and duplication are visible.

fn fixture_b() -> LifecycleFixture {
    lifecycle_fixture("<body><section><div><div></section>", 7)
}

fn fixture_b_parts(fixture: &LifecycleFixture) -> HtmlDocumentShellParts {
    let [_, _, _, body, section_id, outer_div, inner_div] = fixture.ids[..] else {
        panic!("seven minted identities")
    };
    let mut nodes = shell_prefix_nodes(fixture, vec![section_id]);
    nodes.push(HtmlTreeNode::new(
        section_id,
        Some(body),
        vec![outer_div],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Section,
            (6, 15),
            (7, 14),
        ),
    ));
    nodes.push(HtmlTreeNode::new(
        outer_div,
        Some(section_id),
        vec![inner_div],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Div,
            (15, 20),
            (16, 19),
        ),
    ));
    nodes.push(HtmlTreeNode::new(
        inner_div,
        Some(outer_div),
        vec![],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Div,
            (20, 25),
            (21, 24),
        ),
    ));

    HtmlDocumentShellParts {
        nodes,
        root: fixture.ids[0],
        admitted_creation_events: 7,
        diagnostics: vec![misnested_diagnostic(4, fixture.anchor(25, 35))],
        actions: vec![
            insertion(
                section_id,
                HtmlSelectedOrdinaryElementName::Section,
                1,
                fixture.anchor(6, 15),
            ),
            insertion(
                outer_div,
                HtmlSelectedOrdinaryElementName::Div,
                2,
                fixture.anchor(15, 20),
            ),
            insertion(
                inner_div,
                HtmlSelectedOrdinaryElementName::Div,
                3,
                fixture.anchor(20, 25),
            ),
            recovery_pop(inner_div, section_id, 4, fixture.anchor(25, 35)),
            recovery_pop(outer_div, section_id, 4, fixture.anchor(25, 35)),
            matching_closure(
                section_id,
                HtmlSelectedOrdinaryElementName::Section,
                4,
                fixture.anchor(25, 35),
            ),
        ],
        processed_tokens: 5,
        committed_prefix_end: 35,
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
        final_open_selected_ordinary: Vec::new(),
        final_open_paragraph: None,
        final_open_style: None,
        final_open_title: None,
        final_text_mode_active: false,
        final_original_insertion_mode_retained: false,
        pending_tokenizer_feedback: false,
        coordinated_raw_text_entry_tokens: Vec::new(),
        coordinated_raw_text_close_tokens: Vec::new(),
        coordinated_rcdata_entry_tokens: Vec::new(),
        coordinated_rcdata_close_tokens: Vec::new(),
    }
}

// --- Fixture C: `<body><section><section><div></section>` --------------------
//
// Two same-name candidates, so "nearest" is a real question the recorded
// target field could get wrong while still naming a plausible element.

fn fixture_c() -> LifecycleFixture {
    lifecycle_fixture("<body><section><section><div></section>", 7)
}

fn fixture_c_parts(fixture: &LifecycleFixture) -> HtmlDocumentShellParts {
    let [_, _, _, body, outer_section, inner_section, div_id] = fixture.ids[..] else {
        panic!("seven minted identities")
    };
    let mut nodes = shell_prefix_nodes(fixture, vec![outer_section]);
    nodes.push(HtmlTreeNode::new(
        outer_section,
        Some(body),
        vec![inner_section],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Section,
            (6, 15),
            (7, 14),
        ),
    ));
    nodes.push(HtmlTreeNode::new(
        inner_section,
        Some(outer_section),
        vec![div_id],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Section,
            (15, 24),
            (16, 23),
        ),
    ));
    nodes.push(HtmlTreeNode::new(
        div_id,
        Some(inner_section),
        vec![],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Div,
            (24, 29),
            (25, 28),
        ),
    ));

    HtmlDocumentShellParts {
        nodes,
        root: fixture.ids[0],
        admitted_creation_events: 7,
        diagnostics: vec![misnested_diagnostic(4, fixture.anchor(29, 39))],
        actions: vec![
            insertion(
                outer_section,
                HtmlSelectedOrdinaryElementName::Section,
                1,
                fixture.anchor(6, 15),
            ),
            insertion(
                inner_section,
                HtmlSelectedOrdinaryElementName::Section,
                2,
                fixture.anchor(15, 24),
            ),
            insertion(
                div_id,
                HtmlSelectedOrdinaryElementName::Div,
                3,
                fixture.anchor(24, 29),
            ),
            recovery_pop(div_id, inner_section, 4, fixture.anchor(29, 39)),
            matching_closure(
                inner_section,
                HtmlSelectedOrdinaryElementName::Section,
                4,
                fixture.anchor(29, 39),
            ),
        ],
        processed_tokens: 5,
        committed_prefix_end: 39,
        // The outer `section` is still open at hand-off.
        final_open_selected_ordinary: vec![outer_section],
        final_open_paragraph: None,
        final_open_style: None,
        final_open_title: None,
        final_text_mode_active: false,
        final_original_insertion_mode_retained: false,
        pending_tokenizer_feedback: false,
        coordinated_raw_text_entry_tokens: Vec::new(),
        coordinated_raw_text_close_tokens: Vec::new(),
        coordinated_rcdata_entry_tokens: Vec::new(),
        coordinated_rcdata_close_tokens: Vec::new(),
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
    }
}

// --- Positive controls ------------------------------------------------------

#[test]
fn freeze_accepts_the_valid_single_recovery_baseline() {
    let fixture = fixture_a();
    let analysis = freeze_parts(&fixture, fixture_a_parts(&fixture)).expect("valid parts freeze");
    assert_eq!(analysis.node_count(), 6);
    assert_eq!(project_recoveries(&analysis).len(), 1);
    assert_eq!(project_closures(&analysis).len(), 1);
}

#[test]
fn freeze_accepts_the_valid_multiple_recovery_baseline() {
    let fixture = fixture_b();
    let analysis = freeze_parts(&fixture, fixture_b_parts(&fixture)).expect("valid parts freeze");
    assert_eq!(project_recoveries(&analysis).len(), 2);
    assert_eq!(project_closures(&analysis).len(), 1);
}

#[test]
fn freeze_accepts_the_valid_nested_target_baseline() {
    let fixture = fixture_c();
    let analysis = freeze_parts(&fixture, fixture_c_parts(&fixture)).expect("valid parts freeze");
    assert_eq!(project_recoveries(&analysis)[0].target_start_tag, (15, 24));
}

#[test]
fn a_valid_frozen_recovery_survives_storage_perturbation() {
    let fixture = fixture_b();
    let analysis = freeze_parts(&fixture, fixture_b_parts(&fixture)).expect("valid parts freeze");
    let expected = project_recoveries(&analysis);
    let perturbed = freeze_parts(&fixture, fixture_b_parts(&fixture))
        .expect("valid parts freeze")
        .with_reversed_storage();
    assert_eq!(project_recoveries(&perturbed), expected);
}

// --- Recovery endpoint corruption ------------------------------------------

#[test]
fn freeze_rejects_a_recovery_target_that_is_not_the_nearest_same_name_element() {
    // S3: the recorded target is a real, open, correctly-named `section` — but
    // not the nearest one. Freeze recomputes "nearest" from its own replay
    // instead of trusting the field, so this cannot pass.
    let fixture = fixture_c();
    let mut parts = fixture_c_parts(&fixture);
    let outer_section = fixture.ids[4];
    let div_id = fixture.ids[6];
    parts.actions[3] = recovery_pop(div_id, outer_section, 4, fixture.anchor(29, 39));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::RecoveryTargetIsNotNearestMatchingSelectedOrdinary(_))
    ));
}

#[test]
fn freeze_rejects_a_recovery_target_that_is_not_a_selected_element() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let body = fixture.ids[3];
    let div_id = fixture.ids[5];
    parts.actions[2] = recovery_pop(div_id, body, 3, fixture.anchor(20, 30));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::RecoveryTargetIsNotSelectedOrdinaryElement(_))
    ));
}

#[test]
fn freeze_rejects_a_recovery_subject_that_is_not_a_selected_element() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let body = fixture.ids[3];
    let section_id = fixture.ids[4];
    parts.actions[2] = recovery_pop(body, section_id, 3, fixture.anchor(20, 30));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::RecoverySubjectIsNotSelectedOrdinaryElement(_))
    ));
}

#[test]
fn freeze_rejects_a_self_targeting_recovery() {
    // S2 in its bluntest form: the target must be closed, never popped.
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let section_id = fixture.ids[4];
    parts.actions[2] = recovery_pop(section_id, section_id, 3, fixture.anchor(20, 30));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::SelfTargetingSelectedOrdinaryRecovery(
            _
        ))
    ));
}

#[test]
fn freeze_rejects_an_unresolved_recovery_endpoint() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let section_id = fixture.ids[4];
    // An identity the stored inventory does not contain at all.
    let mut counter = HtmlConstructedIdentityCounter::new();
    for _ in 0..7 {
        let reserved = counter.reserve().expect("identity headroom");
        counter.commit(reserved);
    }
    let stranger = counter.reserve().expect("identity headroom");
    parts.actions[2] = recovery_pop(stranger, section_id, 3, fixture.anchor(20, 30));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnresolvedActionSubject(_))
    ));
}

// --- Recovery ordering corruption ------------------------------------------

#[test]
fn freeze_rejects_a_reversed_recovery_order() {
    // S6: popping the outer `div` while the inner one is still current.
    let fixture = fixture_b();
    let mut parts = fixture_b_parts(&fixture);
    parts.actions.swap(3, 4);
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::NonLifoSelectedOrdinaryRecovery(_))
    ));
}

#[test]
fn freeze_rejects_a_skipped_intervening_element() {
    // S5: only one of the two intervening elements is accounted for.
    let fixture = fixture_b();
    let mut parts = fixture_b_parts(&fixture);
    parts.actions.remove(3);
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::NonLifoSelectedOrdinaryRecovery(_))
    ));
}

#[test]
fn freeze_rejects_a_non_current_recovery_subject() {
    // S4: the recorded subject is open, selected, and part of the real suffix
    // — but it is not the current node when the pop is claimed.
    let fixture = fixture_b();
    let mut parts = fixture_b_parts(&fixture);
    let section_id = fixture.ids[4];
    let outer_div = fixture.ids[5];
    parts.actions[3] = recovery_pop(outer_div, section_id, 4, fixture.anchor(25, 35));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::NonLifoSelectedOrdinaryRecovery(_))
    ));
}

#[test]
fn freeze_rejects_a_duplicated_recovery() {
    // S7: the same element popped twice under one end tag.
    let fixture = fixture_b();
    let mut parts = fixture_b_parts(&fixture);
    let section_id = fixture.ids[4];
    let inner_div = fixture.ids[6];
    parts.actions.insert(
        4,
        recovery_pop(inner_div, section_id, 4, fixture.anchor(25, 35)),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::NonLifoSelectedOrdinaryRecovery(_))
    ));
}

#[test]
fn freeze_rejects_an_extra_recovery_after_the_group_closed() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let section_id = fixture.ids[4];
    let div_id = fixture.ids[5];
    parts
        .actions
        .push(recovery_pop(div_id, section_id, 3, fixture.anchor(20, 30)));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::RecoveryTargetIsNotNearestMatchingSelectedOrdinary(_))
    ));
}

// --- Recovery group integrity ----------------------------------------------

#[test]
fn freeze_rejects_a_recovery_with_no_target_closure() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts.actions.pop();
    // The `section` is then still open, so the snapshot must say so too, or
    // the final-open check would be what fires instead of the group check.
    parts.final_open_selected_ordinary = vec![fixture.ids[4]];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnterminatedSelectedOrdinaryRecovery(_))
    ));
}

#[test]
fn freeze_rejects_an_unrelated_action_inside_a_recovery_group() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts.actions.insert(
        3,
        HtmlTreeAction::new(
            HtmlTreeActionKind::StoppedParsing,
            HtmlTreeTokenTrigger::authored(3, fixture.anchor(20, 30)),
        ),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnterminatedSelectedOrdinaryRecovery(_))
    ));
}

#[test]
fn freeze_rejects_a_group_terminated_by_the_wrong_closure() {
    // The group's own target is never closed; a different selected element is
    // closed under its trigger instead.
    let fixture = fixture_c();
    let mut parts = fixture_c_parts(&fixture);
    let outer_section = fixture.ids[4];
    parts.actions[4] = matching_closure(
        outer_section,
        HtmlSelectedOrdinaryElementName::Section,
        4,
        fixture.anchor(29, 39),
    );
    parts.final_open_selected_ordinary = Vec::new();
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::SelectedOrdinaryRecoveryClosureMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_a_non_current_target_closed_without_its_recovery() {
    // S5's mirror: the closure alone, with the intervening `div` unaccounted
    // for. The target is not current, so the closure cannot stand.
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts.actions.remove(2);
    parts.diagnostics = Vec::new();
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::NonLifoSelectedOrdinaryClosure(_))
    ));
}

// --- Trigger corruption -----------------------------------------------------

#[test]
fn freeze_rejects_a_recovery_triggered_by_a_start_tag() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let section_id = fixture.ids[4];
    let div_id = fixture.ids[5];
    // Token 2 is the `<div>` start tag: a real retained token whose authored
    // boundary revalidates, in non-decreasing token order, and which is still
    // not an end tag.
    parts.actions[2] = recovery_pop(div_id, section_id, 2, fixture.anchor(15, 20));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::RecoveryTriggerIsNotMatchingTargetEndTag { .. })
    ));
}

#[test]
fn freeze_rejects_a_recovery_triggered_at_end_of_file() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let section_id = fixture.ids[4];
    let div_id = fixture.ids[5];
    parts.actions[2] = HtmlTreeAction::new(
        HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag {
            node: div_id,
            target: section_id,
        },
        HtmlTreeTokenTrigger::end_of_file(5),
    );
    // Drop the closure so recorded token order stays non-decreasing and the
    // trigger check is what this case actually exercises.
    parts.actions.pop();
    parts.final_open_selected_ordinary = vec![section_id];
    parts.processed_tokens = 6;
    parts.committed_prefix_end = 36;
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::RecoveryTriggerIsNotMatchingTargetEndTag { .. })
    ));
}

#[test]
fn freeze_rejects_a_recovery_triggered_by_an_unrelated_selected_end_tag() {
    // The retained `</div>` at token 4 is a real, valid, authored selected end
    // tag — just not the target's. Name equality is checked, not mere
    // end-tag-ness.
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let section_id = fixture.ids[4];
    let div_id = fixture.ids[5];
    parts.actions[2] = recovery_pop(div_id, section_id, 4, fixture.anchor(30, 36));
    parts.actions.pop();
    parts.final_open_selected_ordinary = vec![section_id];
    parts.processed_tokens = 5;
    parts.committed_prefix_end = 36;
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::RecoveryTriggerIsNotMatchingTargetEndTag { .. })
    ));
}

#[test]
fn freeze_rejects_a_recovery_whose_range_is_not_the_retained_end_tag_evidence() {
    // S8: a range that still revalidates against the source, but is not that
    // token's own complete-tag evidence.
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let section_id = fixture.ids[4];
    let div_id = fixture.ids[5];
    parts.actions[2] = recovery_pop(div_id, section_id, 3, fixture.anchor(20, 29));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::RecoveryTriggerIsNotMatchingTargetEndTag { .. })
    ));
}

#[test]
fn freeze_rejects_recovery_evidence_bound_to_a_foreign_source() {
    // S8: identical bytes, different `SourceId`. Retained evidence is bound to
    // exactly one source and may not be swapped for a look-alike.
    let fixture = fixture_a();
    let foreign = SourceText::new(SourceId::new(99), FIXTURE_A_SOURCE.to_owned());
    let mut parts = fixture_a_parts(&fixture);
    let section_id = fixture.ids[4];
    let div_id = fixture.ids[5];
    parts.actions[2] = recovery_pop(
        div_id,
        section_id,
        3,
        foreign.anchor(20, 30).expect("valid range"),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::ForeignSourceEvidence { .. })
    ));
}

#[test]
fn freeze_rejects_a_recovery_and_closure_that_disagree_on_the_trigger() {
    // One end tag, one group: the closure cannot belong to a different token
    // than the pops it terminates.
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let section_id = fixture.ids[4];
    parts.actions[3] = matching_closure(
        section_id,
        HtmlSelectedOrdinaryElementName::Section,
        4,
        fixture.anchor(30, 36),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::ClosureTriggerIsNotTheMatchingEndTag { .. })
    ));
}

// --- Fabricated closure on a recovered element ------------------------------

#[test]
fn freeze_rejects_a_fabricated_closure_on_a_recovery_popped_element() {
    // S2: the `</div>` at token 4 is a real retained end tag whose name even
    // matches the popped element — but that element's open lifetime already
    // ended with a recovery pop, so it can never be closed.
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let div_id = fixture.ids[5];
    parts.actions.push(matching_closure(
        div_id,
        HtmlSelectedOrdinaryElementName::Div,
        4,
        fixture.anchor(30, 36),
    ));
    parts.processed_tokens = 5;
    parts.committed_prefix_end = 36;
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::NonLifoSelectedOrdinaryClosure(_))
    ));
}

#[test]
fn freeze_rejects_a_closure_recorded_for_an_intervening_element_instead_of_the_target() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    let div_id = fixture.ids[5];
    parts.actions[3] = matching_closure(
        div_id,
        HtmlSelectedOrdinaryElementName::Div,
        3,
        fixture.anchor(20, 30),
    );
    parts.final_open_selected_ordinary = vec![fixture.ids[4]];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::ClosureTriggerIsNotTheMatchingEndTag { .. })
    ));
}

// --- Diagnostic corruption --------------------------------------------------

#[test]
fn freeze_rejects_a_missing_misnested_diagnostic() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts.diagnostics = Vec::new();
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::SelectedOrdinaryRecoveryDiagnosticMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_a_duplicated_misnested_diagnostic() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts
        .diagnostics
        .push(misnested_diagnostic(3, fixture.anchor(20, 30)));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::SelectedOrdinaryRecoveryDiagnosticMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_a_misnested_diagnostic_with_the_wrong_trigger() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts.diagnostics = vec![misnested_diagnostic(1, fixture.anchor(6, 15))];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::SelectedOrdinaryRecoveryDiagnosticMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_a_misnested_diagnostic_with_the_wrong_recovery_summary() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts.diagnostics = vec![HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag,
        HtmlTreeTokenTrigger::authored(3, fixture.anchor(20, 30)),
        HtmlTreeRecovery::IgnoredToken,
    )];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::SelectedOrdinaryRecoveryDiagnosticMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_a_misnested_diagnostic_with_no_recovery_at_all() {
    // A current-target closure carrying a misnested diagnostic: the diagnostic
    // claims a recovery that the action stream never committed.
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    // Drop the `div` insertion and the recovery, leaving `section` current.
    parts.actions.remove(2);
    parts.actions.remove(1);
    parts.nodes.retain(|node| node.id() != fixture.ids[5]);
    parts.admitted_creation_events = 5;
    let section_id = fixture.ids[4];
    let section = parts
        .nodes
        .iter_mut()
        .find(|node| node.id() == section_id)
        .expect("the section is stored");
    *section = HtmlTreeNode::new(
        section_id,
        Some(fixture.ids[3]),
        vec![],
        selected_kind(
            &fixture,
            HtmlSelectedOrdinaryElementName::Section,
            (6, 15),
            (7, 14),
        ),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::SelectedOrdinaryRecoveryDiagnosticMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_an_unmatched_end_action_with_no_matching_diagnostic() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts.actions.push(HtmlTreeAction::new(
        HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag {
            name: HtmlSelectedOrdinaryElementName::Div,
        },
        HtmlTreeTokenTrigger::authored(4, fixture.anchor(30, 36)),
    ));
    parts.processed_tokens = 5;
    parts.committed_prefix_end = 36;
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnmatchedSelectedOrdinaryEndTagDiagnosticMismatch { .. })
    ));
}

// --- Final-open snapshot corruption -----------------------------------------

#[test]
fn freeze_rejects_a_final_open_snapshot_that_disagrees_with_the_replay() {
    // S9: the session claims the `section` is still open; the action stream
    // says it was closed.
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts.final_open_selected_ordinary = vec![fixture.ids[4]];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::FinalOpenSelectedOrdinaryStateMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_a_final_open_snapshot_that_drops_a_still_open_element() {
    let fixture = fixture_c();
    let mut parts = fixture_c_parts(&fixture);
    parts.final_open_selected_ordinary = Vec::new();
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::FinalOpenSelectedOrdinaryStateMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_a_final_open_entry_that_is_not_a_selected_element() {
    let fixture = fixture_c();
    let mut parts = fixture_c_parts(&fixture);
    parts.final_open_selected_ordinary = vec![fixture.ids[3]];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::FinalOpenSelectedOrdinaryIsNotASelectedElement(_))
    ));
}

#[test]
fn freeze_rejects_a_misnested_diagnostic_whose_boundary_is_not_the_trigger_evidence() {
    // The recorded token index is the group's own, and the recorded range
    // still revalidates against the source — but it is not that token's
    // complete authored evidence, so it is not the trigger it claims to be.
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts.diagnostics = vec![misnested_diagnostic(3, fixture.anchor(20, 29))];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::SelectedOrdinaryRecoveryDiagnosticMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_a_misnested_diagnostic_triggered_at_end_of_file() {
    let fixture = fixture_a();
    let mut parts = fixture_a_parts(&fixture);
    parts.diagnostics = vec![HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag,
        HtmlTreeTokenTrigger::end_of_file(3),
        HtmlTreeRecovery::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget,
    )];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::SelectedOrdinaryRecoveryDiagnosticMismatch { .. })
    ));
}

// ---------------------------------------------------------------------------
// One retained selected end token spends exactly one terminal decision
// ---------------------------------------------------------------------------
//
// A recovery group may carry many ordered pops before its one closure, but the
// dispatch ends there. A replay that spends the same retained end token again
// — to close a second, further-out same-name ancestor — describes semantics no
// single dispatch can produce, and each of its two groups looks individually
// valid, so only an explicit per-token ledger rejects it.

/// `<body><section><div><section><div></section>`, where the one authored
/// `</section>` legitimately recovers exactly one `div` and closes the *inner*
/// `section`, leaving the outer `section` and its `div` open.
fn fixture_e() -> LifecycleFixture {
    lifecycle_fixture("<body><section><div><section><div></section>", 8)
}

fn fixture_e_parts(fixture: &LifecycleFixture) -> HtmlDocumentShellParts {
    let [
        _,
        _,
        _,
        body,
        outer_section,
        outer_div,
        inner_section,
        inner_div,
    ] = fixture.ids[..]
    else {
        panic!("eight minted identities")
    };
    let mut nodes = shell_prefix_nodes(fixture, vec![outer_section]);
    for (id, parent, children, name, complete, raw_name) in [
        (
            outer_section,
            body,
            vec![outer_div],
            HtmlSelectedOrdinaryElementName::Section,
            (6, 15),
            (7, 14),
        ),
        (
            outer_div,
            outer_section,
            vec![inner_section],
            HtmlSelectedOrdinaryElementName::Div,
            (15, 20),
            (16, 19),
        ),
        (
            inner_section,
            outer_div,
            vec![inner_div],
            HtmlSelectedOrdinaryElementName::Section,
            (20, 29),
            (21, 28),
        ),
        (
            inner_div,
            inner_section,
            vec![],
            HtmlSelectedOrdinaryElementName::Div,
            (29, 34),
            (30, 33),
        ),
    ] {
        nodes.push(HtmlTreeNode::new(
            id,
            Some(parent),
            children,
            selected_kind(fixture, name, complete, raw_name),
        ));
    }

    HtmlDocumentShellParts {
        nodes,
        root: fixture.ids[0],
        admitted_creation_events: 8,
        diagnostics: vec![misnested_diagnostic(5, fixture.anchor(34, 44))],
        actions: vec![
            insertion(
                outer_section,
                HtmlSelectedOrdinaryElementName::Section,
                1,
                fixture.anchor(6, 15),
            ),
            insertion(
                outer_div,
                HtmlSelectedOrdinaryElementName::Div,
                2,
                fixture.anchor(15, 20),
            ),
            insertion(
                inner_section,
                HtmlSelectedOrdinaryElementName::Section,
                3,
                fixture.anchor(20, 29),
            ),
            insertion(
                inner_div,
                HtmlSelectedOrdinaryElementName::Div,
                4,
                fixture.anchor(29, 34),
            ),
            recovery_pop(inner_div, inner_section, 5, fixture.anchor(34, 44)),
            matching_closure(
                inner_section,
                HtmlSelectedOrdinaryElementName::Section,
                5,
                fixture.anchor(34, 44),
            ),
        ],
        processed_tokens: 6,
        committed_prefix_end: 44,
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
        final_open_selected_ordinary: vec![outer_section, outer_div],
        final_open_paragraph: None,
        final_open_style: None,
        final_open_title: None,
        final_text_mode_active: false,
        final_original_insertion_mode_retained: false,
        pending_tokenizer_feedback: false,
        coordinated_raw_text_entry_tokens: Vec::new(),
        coordinated_raw_text_close_tokens: Vec::new(),
        coordinated_rcdata_entry_tokens: Vec::new(),
        coordinated_rcdata_close_tokens: Vec::new(),
    }
}

#[test]
fn freeze_accepts_one_group_leaving_a_further_out_same_name_ancestor_open() {
    let fixture = fixture_e();
    let analysis = freeze_parts(&fixture, fixture_e_parts(&fixture)).expect("valid parts freeze");
    assert_eq!(project_recoveries(&analysis).len(), 1);
    assert_eq!(project_closures(&analysis).len(), 1);
    assert_eq!(
        project_closures(&analysis)[0].subject_start_tag,
        (20, 29),
        "the inner section is the nearest target"
    );
}

#[test]
fn freeze_rejects_one_end_token_spending_a_second_recovery_group_and_closure() {
    // Both forged groups are individually well-formed: correct nearest target
    // recomputed from the post-first-group stack, current-first pop, matching
    // trigger, and a paired misnested diagnostic. Only the fact that one
    // retained token cannot terminate twice rejects this.
    let fixture = fixture_e();
    let mut parts = fixture_e_parts(&fixture);
    let [_, _, _, _, outer_section, outer_div, ..] = fixture.ids[..] else {
        panic!("eight minted identities")
    };
    parts.actions.push(recovery_pop(
        outer_div,
        outer_section,
        5,
        fixture.anchor(34, 44),
    ));
    parts.actions.push(matching_closure(
        outer_section,
        HtmlSelectedOrdinaryElementName::Section,
        5,
        fixture.anchor(34, 44),
    ));
    parts
        .diagnostics
        .push(misnested_diagnostic(5, fixture.anchor(34, 44)));
    parts.final_open_selected_ordinary = Vec::new();
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::DuplicateSelectedOrdinaryEndTokenDecision { .. })
    ));
}

/// `<body><section><section></section>`, where the one authored `</section>`
/// legitimately closes the inner `section` as a current target.
fn fixture_h() -> LifecycleFixture {
    lifecycle_fixture("<body><section><section></section>", 6)
}

fn fixture_h_parts(fixture: &LifecycleFixture) -> HtmlDocumentShellParts {
    let [_, _, _, body, outer_section, inner_section] = fixture.ids[..] else {
        panic!("six minted identities")
    };
    let mut nodes = shell_prefix_nodes(fixture, vec![outer_section]);
    nodes.push(HtmlTreeNode::new(
        outer_section,
        Some(body),
        vec![inner_section],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Section,
            (6, 15),
            (7, 14),
        ),
    ));
    nodes.push(HtmlTreeNode::new(
        inner_section,
        Some(outer_section),
        vec![],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Section,
            (15, 24),
            (16, 23),
        ),
    ));

    HtmlDocumentShellParts {
        nodes,
        root: fixture.ids[0],
        admitted_creation_events: 6,
        diagnostics: vec![],
        actions: vec![
            insertion(
                outer_section,
                HtmlSelectedOrdinaryElementName::Section,
                1,
                fixture.anchor(6, 15),
            ),
            insertion(
                inner_section,
                HtmlSelectedOrdinaryElementName::Section,
                2,
                fixture.anchor(15, 24),
            ),
            matching_closure(
                inner_section,
                HtmlSelectedOrdinaryElementName::Section,
                3,
                fixture.anchor(24, 34),
            ),
        ],
        processed_tokens: 4,
        committed_prefix_end: 34,
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
        final_open_selected_ordinary: vec![outer_section],
        final_open_paragraph: None,
        final_open_style: None,
        final_open_title: None,
        final_text_mode_active: false,
        final_original_insertion_mode_retained: false,
        pending_tokenizer_feedback: false,
        coordinated_raw_text_entry_tokens: Vec::new(),
        coordinated_raw_text_close_tokens: Vec::new(),
        coordinated_rcdata_entry_tokens: Vec::new(),
        coordinated_rcdata_close_tokens: Vec::new(),
    }
}

#[test]
fn freeze_accepts_the_valid_current_target_closure_baseline() {
    let fixture = fixture_h();
    let analysis = freeze_parts(&fixture, fixture_h_parts(&fixture)).expect("valid parts freeze");
    assert_eq!(project_closures(&analysis).len(), 1);
    assert!(project_recoveries(&analysis).is_empty());
}

#[test]
fn freeze_rejects_one_end_token_spending_a_second_current_target_closure() {
    // The same ledger rule with no recovery involved at all: after the inner
    // `section` is closed, the outer one *is* the current node, so a second
    // closure under the same retained token is stack-consistent, uniquely
    // subjected, and correctly triggered. Only the per-token ledger rejects
    // it.
    let fixture = fixture_h();
    let mut parts = fixture_h_parts(&fixture);
    let outer_section = fixture.ids[4];
    parts.actions.push(matching_closure(
        outer_section,
        HtmlSelectedOrdinaryElementName::Section,
        3,
        fixture.anchor(24, 34),
    ));
    parts.final_open_selected_ordinary = Vec::new();
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::DuplicateSelectedOrdinaryEndTokenDecision { .. })
    ));
}

// ---------------------------------------------------------------------------
// Unmatched selected-end evidence is proved, not merely paired by token index
// ---------------------------------------------------------------------------

/// `<body></section><section>`: one ignored unmatched `</section>` before any
/// selected element is open, then an ordinary `section` left open at the end.
fn fixture_f() -> LifecycleFixture {
    lifecycle_fixture("<body></section><section>", 5)
}

fn ignored_unmatched_end(
    name: HtmlSelectedOrdinaryElementName,
    token: usize,
    trigger: crate::SourceAnchor,
) -> HtmlTreeAction {
    HtmlTreeAction::new(
        HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag { name },
        HtmlTreeTokenTrigger::authored(token, trigger),
    )
}

fn unmatched_diagnostic(token: usize, trigger: crate::SourceAnchor) -> HtmlTreeDiagnostic {
    HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag,
        HtmlTreeTokenTrigger::authored(token, trigger),
        HtmlTreeRecovery::IgnoredToken,
    )
}

fn fixture_f_parts(fixture: &LifecycleFixture) -> HtmlDocumentShellParts {
    let [_, _, _, body, section_id] = fixture.ids[..] else {
        panic!("five minted identities")
    };
    let mut nodes = shell_prefix_nodes(fixture, vec![section_id]);
    nodes.push(HtmlTreeNode::new(
        section_id,
        Some(body),
        vec![],
        selected_kind(
            fixture,
            HtmlSelectedOrdinaryElementName::Section,
            (16, 25),
            (17, 24),
        ),
    ));

    HtmlDocumentShellParts {
        nodes,
        root: fixture.ids[0],
        admitted_creation_events: 5,
        diagnostics: vec![unmatched_diagnostic(1, fixture.anchor(6, 16))],
        actions: vec![
            ignored_unmatched_end(
                HtmlSelectedOrdinaryElementName::Section,
                1,
                fixture.anchor(6, 16),
            ),
            insertion(
                section_id,
                HtmlSelectedOrdinaryElementName::Section,
                2,
                fixture.anchor(16, 25),
            ),
        ],
        processed_tokens: 3,
        committed_prefix_end: 25,
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
        final_open_selected_ordinary: vec![section_id],
        final_open_paragraph: None,
        final_open_style: None,
        final_open_title: None,
        final_text_mode_active: false,
        final_original_insertion_mode_retained: false,
        pending_tokenizer_feedback: false,
        coordinated_raw_text_entry_tokens: Vec::new(),
        coordinated_raw_text_close_tokens: Vec::new(),
        coordinated_rcdata_entry_tokens: Vec::new(),
        coordinated_rcdata_close_tokens: Vec::new(),
    }
}

#[test]
fn freeze_accepts_the_valid_unmatched_end_baseline() {
    let fixture = fixture_f();
    let analysis = freeze_parts(&fixture, fixture_f_parts(&fixture)).expect("valid parts freeze");
    assert!(project_closures(&analysis).is_empty());
    assert!(project_recoveries(&analysis).is_empty());
}

#[test]
fn freeze_rejects_an_unmatched_end_triggered_by_a_start_tag() {
    let fixture = fixture_f();
    let mut parts = fixture_f_parts(&fixture);
    // Token 2 is the `<section>` start tag: real, retained, authored, and in
    // non-decreasing order — but not an end tag.
    parts.actions[0] = ignored_unmatched_end(
        HtmlSelectedOrdinaryElementName::Section,
        2,
        fixture.anchor(16, 25),
    );
    parts.diagnostics = vec![unmatched_diagnostic(2, fixture.anchor(16, 25))];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnmatchedSelectedOrdinaryEndTriggerIsNotTheMatchingEndTag { .. })
    ));
}

#[test]
fn freeze_rejects_an_unmatched_end_whose_recorded_name_is_not_the_trigger_name() {
    let fixture = fixture_f();
    let mut parts = fixture_f_parts(&fixture);
    parts.actions[0] = ignored_unmatched_end(
        HtmlSelectedOrdinaryElementName::Div,
        1,
        fixture.anchor(6, 16),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnmatchedSelectedOrdinaryEndTriggerIsNotTheMatchingEndTag { .. })
    ));
}

#[test]
fn freeze_rejects_an_unmatched_end_whose_range_is_not_the_retained_evidence() {
    let fixture = fixture_f();
    let mut parts = fixture_f_parts(&fixture);
    parts.actions[0] = ignored_unmatched_end(
        HtmlSelectedOrdinaryElementName::Section,
        1,
        fixture.anchor(6, 15),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnmatchedSelectedOrdinaryEndTriggerIsNotTheMatchingEndTag { .. })
    ));
}

#[test]
fn freeze_rejects_unmatched_end_evidence_bound_to_a_foreign_source() {
    let fixture = fixture_f();
    let foreign = SourceText::new(SourceId::new(99), "<body></section><section>".to_owned());
    let mut parts = fixture_f_parts(&fixture);
    parts.actions[0] = ignored_unmatched_end(
        HtmlSelectedOrdinaryElementName::Section,
        1,
        foreign.anchor(6, 16).expect("valid range"),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::ForeignSourceEvidence { .. })
    ));
}

#[test]
fn freeze_rejects_an_unmatched_diagnostic_with_the_wrong_recovery_summary() {
    let fixture = fixture_f();
    let mut parts = fixture_f_parts(&fixture);
    parts.diagnostics = vec![HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag,
        HtmlTreeTokenTrigger::authored(1, fixture.anchor(6, 16)),
        HtmlTreeRecovery::StoppedParsingWithOpenSelectedOrdinaryElements,
    )];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnmatchedSelectedOrdinaryEndTagDiagnosticMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_an_unmatched_diagnostic_whose_boundary_is_not_the_trigger_evidence() {
    let fixture = fixture_f();
    let mut parts = fixture_f_parts(&fixture);
    parts.diagnostics = vec![unmatched_diagnostic(1, fixture.anchor(6, 15))];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnmatchedSelectedOrdinaryEndTagDiagnosticMismatch { .. })
    ));
}

/// `<body><section></section>`, forged so the authored `</section>` is claimed
/// to be unmatched while its own target is still open.
fn fixture_g() -> LifecycleFixture {
    lifecycle_fixture("<body><section></section>", 5)
}

#[test]
fn freeze_rejects_an_unmatched_end_while_a_same_name_target_is_open() {
    let fixture = fixture_g();
    let [_, _, _, body, section_id] = fixture.ids[..] else {
        panic!("five minted identities")
    };
    let mut nodes = shell_prefix_nodes(&fixture, vec![section_id]);
    nodes.push(HtmlTreeNode::new(
        section_id,
        Some(body),
        vec![],
        selected_kind(
            &fixture,
            HtmlSelectedOrdinaryElementName::Section,
            (6, 15),
            (7, 14),
        ),
    ));
    let parts = HtmlDocumentShellParts {
        nodes,
        root: fixture.ids[0],
        admitted_creation_events: 5,
        diagnostics: vec![unmatched_diagnostic(2, fixture.anchor(15, 25))],
        actions: vec![
            insertion(
                section_id,
                HtmlSelectedOrdinaryElementName::Section,
                1,
                fixture.anchor(6, 15),
            ),
            // The trigger really is the retained `</section>` and the recorded
            // name really is `Section` — but that element is open, so the
            // ignored cell is not the cell that applies.
            ignored_unmatched_end(
                HtmlSelectedOrdinaryElementName::Section,
                2,
                fixture.anchor(15, 25),
            ),
        ],
        processed_tokens: 3,
        committed_prefix_end: 25,
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
        final_open_selected_ordinary: vec![section_id],
        final_open_paragraph: None,
        final_open_style: None,
        final_open_title: None,
        final_text_mode_active: false,
        final_original_insertion_mode_retained: false,
        pending_tokenizer_feedback: false,
        coordinated_raw_text_entry_tokens: Vec::new(),
        coordinated_raw_text_close_tokens: Vec::new(),
        coordinated_rcdata_entry_tokens: Vec::new(),
        coordinated_rcdata_close_tokens: Vec::new(),
    };
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnmatchedSelectedOrdinaryEndTagWithOpenTarget(_))
    ));
}

// ---------------------------------------------------------------------------
// A recovery-popped element's later same-name end tag is unmatched, not a
// closure
// ---------------------------------------------------------------------------

#[test]
fn a_later_same_name_end_tag_after_a_recovery_pop_is_unmatched() {
    // `<body><section><div></section></div>` really does contain a `</div>`
    // for the recovery-popped `div`. The durable statement is not that no such
    // end tag exists, but that no matching end tag *caused* the pop and that
    // no closure relation may be fabricated for it. The later `</div>` is an
    // ordinary unmatched end: it closes nothing and recovers nothing.
    let analysis = analyze("<body><section><div></section></div>");
    let closures = project_closures(&analysis);
    assert_eq!(closures.len(), 1, "only the section is ever closed");
    assert_eq!(closures[0].subject_start_tag, (6, 15));
    assert_eq!(project_recoveries(&analysis).len(), 1);
    assert_eq!(
        project_diagnostics(&analysis),
        vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::MisnestedSelectedOrdinaryEndTag {
                token: 3,
                trigger: (20, 30),
            },
            ExpectedDiagnostic::UnmatchedSelectedOrdinaryEndTag {
                token: 4,
                trigger: (30, 36),
            },
        ]
    );
    assert!(
        project_actions(&analysis).contains(&(
            ExpectedAction::IgnoredUnmatchedSelectedOrdinaryEndTag("div"),
            4
        )),
        "the later `</div>` is ignored, not a closure"
    );
    assert!(analysis.is_complete());
}
