//! Production correspondence for TC-S3 — Selected In-Body No-Attribute `div`
//! Construction (Issue #359).
//!
//! Every expectation here is hand-authored against the accepted TC-S3 theorem
//! and the exact DV1–DV14 source bytes. Nothing in this module imports,
//! derives, or replays anything from the candidate-independent oracle in
//! [`super::in_body_div_successor_validation`]: that module keeps its own
//! private machine, its own node identities, its own arena, and its own GOLD,
//! and it stays byte-for-byte unchanged. The two agree because both were
//! written from the same accepted theorem, not because either reads the other.
//!
//! The expected model below is deliberately its own small vocabulary
//! ([`ExpectedNode`], [`ExpectedAction`], [`ExpectedDiagnostic`],
//! [`ExpectedClosure`], [`ExpectedCompletion`]) rather than the production
//! result enums, so a production change cannot quietly redefine what the test
//! expects. `project_*` translates a production result into that vocabulary
//! and carries no expectations of its own.
//!
//! Closure subjects are named by the subject node's own exact authored
//! complete start-tag range, never by a raw identity encoding: that is what
//! makes "the closure names *this* element" checkable without promising any
//! identity representation.

use crate::{SourceId, SourceText};

use super::driver::construct_html_document_shell;
use super::result::{
    HtmlAuthoredSource, HtmlConstructedIdentityCounter, HtmlConstructedNodeId,
    HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlElement, HtmlSelectedOrdinaryElement,
    HtmlSelectedOrdinaryElementName, HtmlShellElement, HtmlShellElementName,
    HtmlShellElementOrigin, HtmlSynthesisCause, HtmlTreeAction, HtmlTreeActionKind,
    HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeDiagnosticCode, HtmlTreeFreezeError,
    HtmlTreeIncompleteCause, HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery,
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
    /// A shell element. `origin` is `None` for a synthesized shell element,
    /// which has no authored source at all.
    Shell {
        name: &'static str,
        origin: Option<(Span, Span)>,
        children: Vec<ExpectedNode>,
    },
    /// A selected ordinary element. It is authored-only, so its exact
    /// `(complete, raw_name)` evidence is not optional.
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
    ClosedSelectedOrdinary(&'static str),
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
    /// The end-of-file trigger has no authored extent, so no span is recorded
    /// for it and none may be fabricated.
    OpenSelectedOrdinaryElementAtEndOfFile {
        token: usize,
    },
}

/// One closure, named by the closed element's own authored start tag and the
/// exact authored end tag that triggered it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpectedClosure {
    subject_start_tag: Span,
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
    }
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
                HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag { name } => {
                    ExpectedAction::IgnoredUnmatchedSelectedOrdinaryEndTag(selected_name(*name))
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
            match diagnostic.code() {
                HtmlTreeDiagnosticCode::MissingDoctype => ExpectedDiagnostic::MissingDoctype {
                    token,
                    trigger: span(
                        diagnostic
                            .trigger()
                            .authored_boundary()
                            .expect("authored missing-DOCTYPE trigger"),
                    ),
                },
                HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag => {
                    ExpectedDiagnostic::UnmatchedSelectedOrdinaryEndTag {
                        token,
                        trigger: span(
                            diagnostic
                                .trigger()
                                .authored_boundary()
                                .expect("authored stray end-tag trigger"),
                        ),
                    }
                }
                HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile => {
                    assert!(
                        diagnostic.trigger().authored_boundary().is_none(),
                        "the end-of-file trigger must carry no authored extent"
                    );
                    ExpectedDiagnostic::OpenSelectedOrdinaryElementAtEndOfFile { token }
                }
                other => panic!("unexpected diagnostic {other:?} for a TC-S3 source"),
            }
        })
        .collect()
}

/// The recorded closures, each named by its subject's own authored start tag.
///
/// Resolving the subject through [`HtmlDocumentShellAnalysis::node`] is the
/// point: it proves the closure names a stored selected ordinary element by
/// semantic identity, and that the element's origin is still its start tag
/// rather than the end tag that closed it.
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

fn text(interpreted: &str, contributions: &[Span]) -> ExpectedNode {
    ExpectedNode::Text {
        interpreted: interpreted.to_owned(),
        contributions: contributions.to_vec(),
    }
}

/// `Document -> html(head, body(..))` with an authored `<body>` at `(0, 6)`.
///
/// Every DV source except DV13 opens with a literal `<body>`, so the shell
/// above the selected slice is the same synthesized `html`/`head` pair with an
/// authored `body`.
fn shell_with_authored_body(body_children: Vec<ExpectedNode>) -> ExpectedNode {
    ExpectedNode::Document(vec![synthesized(
        "html",
        vec![
            synthesized("head", vec![]),
            authored_shell("body", (0, 6), (1, 5), body_children),
        ],
    )])
}

/// The four predecessor actions every `<body>`-opening DV source commits
/// before its first selected token, with their exact trigger token indices.
///
/// The `<body>` start tag is token 0 and is reprocessed through
/// `BeforeHtml`, `BeforeHead`, and `InHead` before `AfterHead` consumes it.
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

/// Compares one production run against its complete hand-authored expectation.
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
        "{}: closure evidence",
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
// DV1 - simple selected start and end
// ---------------------------------------------------------------------------

#[test]
fn dv1_simple_selected_start_and_end() {
    // `<body><div></div>`
    //  0     6     11    17
    let mut actions = shell_prelude_actions();
    actions.extend([
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 2),
        (ExpectedAction::Stopped, 3),
    ]);
    check(&ExpectedRun {
        id: "DV1",
        source: "<body><div></div>",
        tree: shell_with_authored_body(vec![div((6, 11), (7, 10), vec![])]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![closure((6, 11), 2, (11, 17))],
        node_count: 5,
        committed_end: 17,
        processed_tokens: 4,
        completion: ExpectedCompletion::Complete,
    });
}

// ---------------------------------------------------------------------------
// DV2 - mixed-case raw origin and exact closing trigger
// ---------------------------------------------------------------------------

#[test]
fn dv2_mixed_case_raw_origin_and_exact_closing_trigger() {
    // `<body><DiV>x</dIv>`
    //  0     6    11 12   18
    let mut actions = shell_prelude_actions();
    actions.extend([
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1),
        (ExpectedAction::InsertedText, 2),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 3),
        (ExpectedAction::Stopped, 4),
    ]);
    check(&ExpectedRun {
        id: "DV2",
        source: "<body><DiV>x</dIv>",
        // The interpreted name is `div` while the retained raw-name evidence
        // is the exact authored `DiV` spelling at (7, 10).
        tree: shell_with_authored_body(vec![div((6, 11), (7, 10), vec![text("x", &[(11, 12)])])]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        // The differently-spelled `</dIv>` is the closure trigger and nothing
        // else: the node's origin stays the `<DiV>` start tag above.
        closures: vec![closure((6, 11), 3, (12, 18))],
        node_count: 6,
        committed_end: 18,
        processed_tokens: 5,
        completion: ExpectedCompletion::Complete,
    });
}

#[test]
fn dv2_raw_name_evidence_is_the_exact_authored_spelling() {
    let source = SourceText::new(SourceId::new(1), "<body><DiV>x</dIv>".to_owned());
    let analysis =
        construct_html_document_shell(&source, generous_limits()).expect("no boundary failure");

    let selected = analysis
        .nodes_in_creation_order()
        .into_iter()
        .find_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) => Some(selected),
            _ => None,
        })
        .expect("one selected ordinary element");

    assert_eq!(selected.name(), HtmlSelectedOrdinaryElementName::Div);
    assert_eq!(selected.raw_name().fragment(), "DiV");
    assert_eq!(selected.complete().fragment(), "<DiV>");

    // The closure trigger retains the differently-spelled end tag exactly.
    let closure_trigger = analysis
        .actions()
        .iter()
        .find_map(|action| match action.kind() {
            HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. } => Some(action.trigger()),
            _ => None,
        })
        .expect("one closure");
    assert_eq!(
        closure_trigger
            .authored_boundary()
            .expect("authored end tag")
            .fragment(),
        "</dIv>"
    );
}

// ---------------------------------------------------------------------------
// DV3 - nested identities, parentage, LIFO closure, storage independence
// ---------------------------------------------------------------------------

#[test]
fn dv3_nested_identities_parentage_and_lifo_closure() {
    // `<body><div><div>x</div></div>`
    //  0     6    11   16 17    23   29
    let mut actions = shell_prelude_actions();
    actions.extend([
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 2),
        (ExpectedAction::InsertedText, 3),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 4),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 5),
        (ExpectedAction::Stopped, 6),
    ]);
    check(&ExpectedRun {
        id: "DV3",
        source: "<body><div><div>x</div></div>",
        tree: shell_with_authored_body(vec![div(
            (6, 11),
            (7, 10),
            vec![div((11, 16), (12, 15), vec![text("x", &[(16, 17)])])],
        )]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        // Innermost first: the inner `<div>` at (11, 16) closes on the first
        // `</div>`, the outer one at (6, 11) on the second.
        closures: vec![
            closure((11, 16), 4, (17, 23)),
            closure((6, 11), 5, (23, 29)),
        ],
        node_count: 7,
        committed_end: 29,
        processed_tokens: 7,
        completion: ExpectedCompletion::Complete,
    });
}

#[test]
fn dv3_relationships_survive_private_storage_replacement() {
    let analysis = analyze("<body><div><div>x</div></div>");
    let baseline = project_tree(&analysis, analysis.root());
    let closures = project_closures(&analysis);

    let permuted = analysis.clone().with_reversed_storage();
    assert_eq!(
        project_tree(&permuted, permuted.root()),
        baseline,
        "nesting and parentage are identity-based, not storage-derived"
    );
    assert_eq!(
        project_closures(&permuted),
        closures,
        "closure subjects resolve by identity, not by storage position"
    );
}

// ---------------------------------------------------------------------------
// DV4 - sibling placement and distinct creation order
// ---------------------------------------------------------------------------

#[test]
fn dv4_sibling_placement_and_distinct_creation_order() {
    // `<body><div></div><div></div>`
    //  0     6    11     17   22    28
    let mut actions = shell_prelude_actions();
    actions.extend([
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 2),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 3),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 4),
        (ExpectedAction::Stopped, 5),
    ]);
    check(&ExpectedRun {
        id: "DV4",
        source: "<body><div></div><div></div>",
        tree: shell_with_authored_body(vec![
            div((6, 11), (7, 10), vec![]),
            div((17, 22), (18, 21), vec![]),
        ]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![
            closure((6, 11), 2, (11, 17)),
            closure((17, 22), 4, (22, 28)),
        ],
        node_count: 6,
        committed_end: 28,
        processed_tokens: 6,
        completion: ExpectedCompletion::Complete,
    });

    // The two siblings are distinct creation events in authored order.
    let analysis = analyze("<body><div></div><div></div>");
    let selected: Vec<Span> = analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) => {
                Some(span(selected.complete()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(selected, vec![(6, 11), (17, 22)]);
}

// ---------------------------------------------------------------------------
// DV5 - stray end tag: one diagnostic, one ignored disposition
// ---------------------------------------------------------------------------

#[test]
fn dv5_stray_end_tag_is_a_diagnostic_and_an_ignored_disposition() {
    // `<body></div>`
    //  0     6      12
    let mut actions = shell_prelude_actions();
    actions.extend([
        (
            ExpectedAction::IgnoredUnmatchedSelectedOrdinaryEndTag("div"),
            1,
        ),
        (ExpectedAction::Stopped, 2),
    ]);
    check(&ExpectedRun {
        id: "DV5",
        source: "<body></div>",
        // The tree is exactly the predecessor shell: nothing was created.
        tree: shell_with_authored_body(vec![]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            ExpectedDiagnostic::UnmatchedSelectedOrdinaryEndTag {
                token: 1,
                trigger: (6, 12),
            },
        ],
        // No closure: nothing was open, so nothing was closed.
        closures: vec![],
        // No identity was admitted for the ignored token.
        node_count: 4,
        // Progress still advanced past the ignored token.
        committed_end: 12,
        processed_tokens: 3,
        // A stray-end diagnostic does not force incompleteness.
        completion: ExpectedCompletion::Complete,
    });
}

#[test]
fn dv5_the_stray_end_tag_recovery_is_the_ignored_token_recovery() {
    let analysis = analyze("<body></div>");
    let recovery = analysis
        .diagnostics()
        .iter()
        .find(|diagnostic| {
            diagnostic.code() == HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag
        })
        .expect("one unmatched-end diagnostic")
        .recovery();
    assert_eq!(recovery, HtmlTreeRecovery::IgnoredToken);
}

// ---------------------------------------------------------------------------
// DV6 - end of file with an open selected element
// ---------------------------------------------------------------------------

#[test]
fn dv6_end_of_file_with_an_open_selected_element_fabricates_no_closure() {
    // `<body><div>x`
    //  0     6    11 12
    let mut actions = shell_prelude_actions();
    actions.extend([
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1),
        (ExpectedAction::InsertedText, 2),
        (ExpectedAction::Stopped, 3),
    ]);
    check(&ExpectedRun {
        id: "DV6",
        source: "<body><div>x",
        // The `div` is still in the tree, unpopped and unclosed.
        tree: shell_with_authored_body(vec![div((6, 11), (7, 10), vec![text("x", &[(11, 12)])])]),
        actions,
        diagnostics: vec![
            missing_doctype_at(0, (0, 6)),
            // The end-of-file trigger has no authored extent, so the
            // projection above asserts none is fabricated for it.
            ExpectedDiagnostic::OpenSelectedOrdinaryElementAtEndOfFile { token: 3 },
        ],
        // No synthesized close, no fabricated end-tag anchor, no closure.
        closures: vec![],
        node_count: 6,
        committed_end: 12,
        processed_tokens: 4,
        // An open-element end-of-file diagnostic does not force incompleteness.
        completion: ExpectedCompletion::Complete,
    });
}

#[test]
fn dv6_the_open_element_recovery_is_the_stopped_with_open_elements_recovery() {
    let analysis = analyze("<body><div>x");
    let recovery = analysis
        .diagnostics()
        .iter()
        .find(|diagnostic| {
            diagnostic.code() == HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile
        })
        .expect("one open-element end-of-file diagnostic")
        .recovery();
    assert_eq!(
        recovery,
        HtmlTreeRecovery::StoppedParsingWithOpenSelectedOrdinaryElements
    );

    // No action anywhere in the run closes a selected ordinary element.
    assert!(
        !analysis.actions().iter().any(|action| matches!(
            action.kind(),
            HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
        )),
        "end of file must fabricate no closure"
    );
}

// ---------------------------------------------------------------------------
// DV7 - parent-sensitive text insertion and coalescing
// ---------------------------------------------------------------------------

#[test]
fn dv7_text_insertion_is_parent_sensitive_and_coalesces_per_parent() {
    // `<body><div>a<div>b</div>c</div>`
    //  0     6    11 12   17 18     24 25   31
    let mut actions = shell_prelude_actions();
    actions.extend([
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1),
        (ExpectedAction::InsertedText, 2),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 3),
        (ExpectedAction::InsertedText, 4),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 5),
        // `c` is a *new* text node, not an append: the last child of the
        // outer `div` is the inner element, not a text node.
        (ExpectedAction::InsertedText, 6),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 7),
        (ExpectedAction::Stopped, 8),
    ]);
    check(&ExpectedRun {
        id: "DV7",
        source: "<body><div>a<div>b</div>c</div>",
        tree: shell_with_authored_body(vec![div(
            (6, 11),
            (7, 10),
            vec![
                text("a", &[(11, 12)]),
                div((12, 17), (13, 16), vec![text("b", &[(17, 18)])]),
                text("c", &[(24, 25)]),
            ],
        )]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![
            closure((12, 17), 5, (18, 24)),
            closure((6, 11), 7, (25, 31)),
        ],
        node_count: 9,
        committed_end: 31,
        processed_tokens: 9,
        completion: ExpectedCompletion::Complete,
    });
}

#[test]
fn the_existing_coalescing_path_is_reused_unchanged_beside_a_selected_element() {
    // `<body>a<body>b<div></div>`
    //  0     6 7     13 14   19    25
    //        a=(6,7)  b=(13,14)  div=(14,19) raw=(15,18) end=(19,25)
    //
    // The ignored duplicate `body` start tag produces no node, so the second
    // character run lands next to the first and coalesces into one text node
    // with two exact ordered contributions. The append admits no identity, and
    // the selected element that follows is a separate creation event.
    let analysis = analyze("<body>a<body>b<div></div>");
    assert_eq!(
        project_tree(&analysis, analysis.root()),
        shell_with_authored_body(vec![
            text("ab", &[(6, 7), (13, 14)]),
            div((14, 19), (15, 18), vec![]),
        ])
    );

    let mut actions = shell_prelude_actions();
    actions.extend([
        (ExpectedAction::InsertedText, 1),
        (
            ExpectedAction::DuplicateShellStartTagCreatedNoNode("body"),
            2,
        ),
        // An append, not a second text node.
        (ExpectedAction::AppendedText, 3),
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 4),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 5),
        (ExpectedAction::Stopped, 6),
    ]);
    assert_eq!(project_actions(&analysis), actions);

    // Document, html, head, body, one text node, one `div`: the append and the
    // ignored duplicate admitted nothing.
    assert_eq!(analysis.node_count(), 6);
    assert!(analysis.is_complete());
}

// ---------------------------------------------------------------------------
// DV8a / DV8b - transactional refusal before any mutation
// ---------------------------------------------------------------------------

#[test]
fn dv8a_an_attributed_selected_start_tag_refuses_transactionally() {
    // `<body><div id=x>`
    //  0     6          16
    check(&ExpectedRun {
        id: "DV8a",
        source: "<body><div id=x>",
        // The predecessor shell only: the refused token created nothing.
        tree: shell_with_authored_body(vec![]),
        // Exactly the predecessor prelude: the refused token committed no
        // action of any kind, not even a disposition.
        actions: shell_prelude_actions(),
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![],
        node_count: 4,
        // Coverage stops at the end of the last committed token.
        committed_end: 6,
        processed_tokens: 1,
        completion: ExpectedCompletion::Unsupported {
            capability: HtmlTreeCapability::AdmittedTagAttribute,
            token: 1,
            trigger: Some((6, 16)),
        },
    });
}

#[test]
fn dv8b_a_self_closing_selected_start_tag_refuses_transactionally() {
    // `<body><div/>`
    //  0     6      12
    check(&ExpectedRun {
        id: "DV8b",
        source: "<body><div/>",
        tree: shell_with_authored_body(vec![]),
        actions: shell_prelude_actions(),
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![],
        node_count: 4,
        committed_end: 6,
        processed_tokens: 1,
        completion: ExpectedCompletion::Unsupported {
            capability: HtmlTreeCapability::SelfClosingAdmittedTag,
            token: 1,
            trigger: Some((6, 12)),
        },
    });
}

// ---------------------------------------------------------------------------
// DV9 - a selected tag outside `in body` stays refused
// ---------------------------------------------------------------------------

#[test]
fn dv9_a_selected_start_tag_in_after_body_remains_refused() {
    // `<body></body><div>`
    //  0     6       13    18
    let mut actions = shell_prelude_actions();
    actions.push((ExpectedAction::AcknowledgedShellEndTag("body"), 1));
    check(&ExpectedRun {
        id: "DV9",
        source: "<body></body><div>",
        tree: shell_with_authored_body(vec![]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![],
        node_count: 4,
        committed_end: 13,
        processed_tokens: 2,
        completion: ExpectedCompletion::Unsupported {
            capability: HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
            token: 2,
            trigger: Some((13, 18)),
        },
    });
}

#[test]
fn a_selected_tag_before_the_shell_walk_refuses_before_any_effect() {
    // A selected tag as the very first token must refuse before the
    // missing-DOCTYPE recovery, the shell walk, the mode change, coverage, or
    // any identity: the whole result is the bare Document root.
    for source in ["<div>", "</div>", "<DIV>"] {
        let analysis = analyze(source);
        assert_eq!(
            project_tree(&analysis, analysis.root()),
            ExpectedNode::Document(vec![]),
            "{source:?}: no shell element may be created"
        );
        assert_eq!(analysis.node_count(), 1, "{source:?}: root only");
        assert!(
            analysis.diagnostics().is_empty(),
            "{source:?}: no missing-DOCTYPE recovery may be recorded"
        );
        assert!(analysis.actions().is_empty(), "{source:?}: no action");
        assert_eq!(analysis.coverage().committed_end(), 0, "{source:?}");
        assert_eq!(analysis.coverage().processed_tokens(), 0, "{source:?}");
        assert_eq!(
            project_completion(&analysis),
            ExpectedCompletion::Unsupported {
                capability: HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
                token: 0,
                trigger: Some((0, source.len())),
            },
            "{source:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// DV10 - a shell interaction over an open selected element commits nothing
// ---------------------------------------------------------------------------

#[test]
fn dv10_body_close_with_an_open_selected_element_commits_no_partial_mutation() {
    // `<body><div></body>`
    //  0     6    11      18
    let mut actions = shell_prelude_actions();
    actions.push((ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1));
    check(&ExpectedRun {
        id: "DV10",
        source: "<body><div></body>",
        // The `div` created before the refused token stays exactly as it was.
        tree: shell_with_authored_body(vec![div((6, 11), (7, 10), vec![])]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        // No closure, and no acknowledged body end tag: the refused `</body>`
        // committed no part of the body close.
        closures: vec![],
        node_count: 5,
        committed_end: 11,
        processed_tokens: 2,
        completion: ExpectedCompletion::Unsupported {
            capability: HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement,
            token: 2,
            trigger: Some((11, 18)),
        },
    });
}

// ---------------------------------------------------------------------------
// DV11 - `<p>` remains unsupported
// ---------------------------------------------------------------------------

#[test]
fn dv11_p_remains_unsupported() {
    // `<body><p>`
    //  0     6   9
    check(&ExpectedRun {
        id: "DV11",
        source: "<body><p>",
        tree: shell_with_authored_body(vec![]),
        actions: shell_prelude_actions(),
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![],
        node_count: 4,
        committed_end: 6,
        processed_tokens: 1,
        completion: ExpectedCompletion::Unsupported {
            capability: HtmlTreeCapability::UnprovedElementTag,
            token: 1,
            trigger: Some((6, 9)),
        },
    });
}

// ---------------------------------------------------------------------------
// DV12 - lower-layer incompleteness is never upgraded
// ---------------------------------------------------------------------------

#[test]
fn dv12_lower_layer_incompleteness_is_never_upgraded() {
    // `<body><div>&amp;` — the tokenizer refuses the character reference, so
    // the tree layer simply runs out of tokens with the `div` still open.
    let mut actions = shell_prelude_actions();
    actions.push((ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1));
    check(&ExpectedRun {
        id: "DV12",
        source: "<body><div>&amp;",
        tree: shell_with_authored_body(vec![div((6, 11), (7, 10), vec![])]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![],
        node_count: 5,
        committed_end: 11,
        processed_tokens: 2,
        completion: ExpectedCompletion::LowerLayerIncomplete,
    });

    let analysis = analyze("<body><div>&amp;");
    assert!(
        analysis.tokenizer_run().is_incomplete(),
        "the lower layer is the incomplete one"
    );
    assert!(!analysis.is_complete());
}

// ---------------------------------------------------------------------------
// DV13 - the synthesized-shell predecessor path
// ---------------------------------------------------------------------------

#[test]
fn dv13_a_synthesized_shell_predecessor_path_reaches_the_selected_cells() {
    // `x<div></div>`
    //  0 1    6     12
    check(&ExpectedRun {
        id: "DV13",
        source: "x<div></div>",
        tree: ExpectedNode::Document(vec![synthesized(
            "html",
            vec![
                synthesized("head", vec![]),
                // Every shell element here is synthesized: `x` triggered the
                // implied structure but authored none of it.
                synthesized(
                    "body",
                    vec![text("x", &[(0, 1)]), div((1, 6), (2, 5), vec![])],
                ),
            ],
        )]),
        actions: vec![
            (ExpectedAction::Reprocessed, 0),
            (ExpectedAction::InsertedSynthesizedShell("html"), 0),
            (ExpectedAction::Reprocessed, 0),
            (ExpectedAction::InsertedSynthesizedShell("head"), 0),
            (ExpectedAction::Reprocessed, 0),
            (ExpectedAction::ClosedShellByImpliedToken("head"), 0),
            (ExpectedAction::Reprocessed, 0),
            (ExpectedAction::InsertedSynthesizedShell("body"), 0),
            (ExpectedAction::Reprocessed, 0),
            (ExpectedAction::InsertedText, 0),
            (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1),
            (ExpectedAction::ClosedSelectedOrdinary("div"), 2),
            (ExpectedAction::Stopped, 3),
        ],
        // The missing-DOCTYPE trigger is the `x` character run, not a tag.
        diagnostics: vec![missing_doctype_at(0, (0, 1))],
        closures: vec![closure((1, 6), 2, (6, 12))],
        node_count: 6,
        committed_end: 12,
        processed_tokens: 4,
        completion: ExpectedCompletion::Complete,
    });
}

// ---------------------------------------------------------------------------
// DV14 - a closed selected element then the existing body-close path
// ---------------------------------------------------------------------------

#[test]
fn dv14_a_closed_selected_element_preserves_the_predecessor_body_close() {
    // `<body><div></div></body>`
    //  0     6    11     17     24
    let mut actions = shell_prelude_actions();
    actions.extend([
        (ExpectedAction::InsertedAuthoredSelectedOrdinary("div"), 1),
        (ExpectedAction::ClosedSelectedOrdinary("div"), 2),
        // Once `k` is back to 0 the existing body-close cell proceeds exactly
        // as the predecessor proves it.
        (ExpectedAction::AcknowledgedShellEndTag("body"), 3),
        (ExpectedAction::Stopped, 4),
    ]);
    check(&ExpectedRun {
        id: "DV14",
        source: "<body><div></div></body>",
        tree: shell_with_authored_body(vec![div((6, 11), (7, 10), vec![])]),
        actions,
        diagnostics: vec![missing_doctype_at(0, (0, 6))],
        closures: vec![closure((6, 11), 2, (11, 17))],
        node_count: 5,
        committed_end: 24,
        processed_tokens: 5,
        completion: ExpectedCompletion::Complete,
    });
}

// ---------------------------------------------------------------------------
// Bounded generated selected sequences
// ---------------------------------------------------------------------------

/// Balanced `<div>` nesting to depth `k`, wrapped in an authored `<body>`.
fn balanced_nesting(depth: usize) -> String {
    let mut source = String::from("<body>");
    for _ in 0..depth {
        source.push_str("<div>");
    }
    for _ in 0..depth {
        source.push_str("</div>");
    }
    source
}

#[test]
fn bounded_generated_selected_sequences_hold_the_theorem() {
    // Deliberately a bounded enumeration, not a resource probe: nothing here
    // asserts a maximum nesting depth, and the subsystem defines none.
    for depth in 0..12usize {
        let source_text = balanced_nesting(depth);
        let analysis = analyze(&source_text);
        let label = format!("depth {depth}");

        assert!(
            analysis.is_complete(),
            "{label}: balanced nesting completes"
        );
        assert_eq!(
            analysis.coverage().committed_end(),
            source_text.len(),
            "{label}: full committed coverage"
        );
        // Document + html + head + body + one node per `div`.
        assert_eq!(analysis.node_count(), 4 + depth, "{label}: identity count");

        let closures = project_closures(&analysis);
        assert_eq!(closures.len(), depth, "{label}: one closure per element");

        // Closure order is innermost-first, and each names a distinct element.
        let mut subjects: Vec<Span> = closures.iter().map(|c| c.subject_start_tag).collect();
        let unique = {
            let mut sorted = subjects.clone();
            sorted.sort_unstable();
            sorted.dedup();
            sorted.len()
        };
        assert_eq!(unique, depth, "{label}: each element closes exactly once");
        subjects.reverse();
        assert!(
            subjects.windows(2).all(|pair| pair[0] < pair[1]),
            "{label}: closures are innermost-first over the selected slice"
        );
    }
}

#[test]
fn bounded_generated_stray_and_open_sequences_stay_honest() {
    for depth in 1..8usize {
        // `k` opens with no closes: exactly one end-of-file diagnostic,
        // regardless of how many elements are open.
        let open_only = format!("<body>{}", "<div>".repeat(depth));
        let analysis = analyze(&open_only);
        let diagnostics = project_diagnostics(&analysis);
        assert_eq!(
            diagnostics
                .iter()
                .filter(|diagnostic| matches!(
                    diagnostic,
                    ExpectedDiagnostic::OpenSelectedOrdinaryElementAtEndOfFile { .. }
                ))
                .count(),
            1,
            "depth {depth}: exactly one open-element end-of-file diagnostic"
        );
        assert!(project_closures(&analysis).is_empty(), "depth {depth}");
        assert!(analysis.is_complete(), "depth {depth}");

        // `k` stray closes with nothing open: one diagnostic each, no node.
        let stray_only = format!("<body>{}", "</div>".repeat(depth));
        let analysis = analyze(&stray_only);
        assert_eq!(
            project_diagnostics(&analysis)
                .iter()
                .filter(|diagnostic| matches!(
                    diagnostic,
                    ExpectedDiagnostic::UnmatchedSelectedOrdinaryEndTag { .. }
                ))
                .count(),
            depth,
            "depth {depth}: one diagnostic per stray end tag"
        );
        assert_eq!(analysis.node_count(), 4, "depth {depth}: no node created");
        assert!(analysis.is_complete(), "depth {depth}");
    }
}

// ---------------------------------------------------------------------------
// Identity, provenance, and cross-`SourceId` invariance
// ---------------------------------------------------------------------------

#[test]
fn semantic_meaning_is_identical_under_differing_source_ids() {
    for source_text in [
        "<body><div></div>",
        "<body><DiV>x</dIv>",
        "<body><div><div>x</div></div>",
        "<body></div>",
        "<body><div>x",
        "<body><div></body>",
        "x<div></div>",
        "<body><div></div></body>",
    ] {
        let baseline = analyze_with(source_text, 1);
        let baseline_tree = project_tree(&baseline, baseline.root());
        let baseline_actions = project_actions(&baseline);
        let baseline_diagnostics = project_diagnostics(&baseline);
        let baseline_closures = project_closures(&baseline);
        let baseline_completion = project_completion(&baseline);

        for source_id in [2u64, 7u64, 4_096u64] {
            let repeat = analyze_with(source_text, source_id);
            assert_eq!(
                project_tree(&repeat, repeat.root()),
                baseline_tree,
                "{source_text:?} under SourceId {source_id}"
            );
            assert_eq!(project_actions(&repeat), baseline_actions);
            assert_eq!(project_diagnostics(&repeat), baseline_diagnostics);
            assert_eq!(project_closures(&repeat), baseline_closures);
            assert_eq!(project_completion(&repeat), baseline_completion);

            // Retained evidence is bound to the caller's own source identity.
            for node in repeat.nodes_in_creation_order() {
                if let Some(HtmlAuthoredSource::StartTag { complete, raw_name }) =
                    node.authored_source()
                {
                    assert_eq!(complete.source_id(), SourceId::new(source_id));
                    assert_eq!(raw_name.source_id(), SourceId::new(source_id));
                }
            }
        }
    }
}

#[test]
fn identity_admission_is_gap_free_across_the_selected_domain() {
    // Each pair is (source, expected admitted creation events). Selected
    // starts and new text nodes admit; ends, diagnostics, dispositions,
    // appends, end of file, and refusals admit nothing and leave no gap.
    for (source_text, expected) in [
        ("<body>", 4usize),
        ("<body><div>", 5),
        ("<body><div></div>", 5),
        ("<body></div>", 4),
        ("<body></div></div></div>", 4),
        ("<body><div>x", 6),
        ("<body><div>x</div>", 6),
        ("<body><div id=x>", 4),
        ("<body><div/>", 4),
        ("<body></body><div>", 4),
        ("<body><div></body>", 5),
    ] {
        let analysis = analyze(source_text);
        assert_eq!(
            analysis.node_count(),
            expected,
            "{source_text:?}: admitted creation events"
        );
        // Gap-free: creation order is dense and strictly increasing, which the
        // frozen inventory check already required, so a refused or ignored
        // action cannot have consumed an ordinal.
        let ordered = analysis.nodes_in_creation_order();
        assert_eq!(ordered.len(), expected, "{source_text:?}");
        assert!(
            ordered.windows(2).all(|pair| pair[0].id() < pair[1].id()),
            "{source_text:?}: identities are strictly creation-ordered"
        );
    }
}

#[test]
fn a_selected_elements_origin_is_never_an_end_tag_or_a_trigger() {
    for source_text in [
        "<body><div></div>",
        "<body><DiV>x</dIv>",
        "<body><div><div>x</div></div>",
        "<body><div>a<div>b</div>c</div>",
    ] {
        let analysis = analyze(source_text);

        // Every closure trigger range is disjoint from every node origin.
        let origins: Vec<Span> = analysis
            .nodes_in_creation_order()
            .into_iter()
            .filter_map(|node| match node.authored_source() {
                Some(HtmlAuthoredSource::StartTag { complete, .. }) => Some(span(complete)),
                _ => None,
            })
            .collect();
        for closure in project_closures(&analysis) {
            assert!(
                !origins.contains(&closure.trigger),
                "{source_text:?}: an end-tag trigger became a node origin"
            );
            assert!(
                origins.contains(&closure.subject_start_tag),
                "{source_text:?}: a closure subject lost its authored origin"
            );
        }

        // Every selected ordinary origin is a start tag whose retained bytes
        // begin with `<` and not with `</`.
        for node in analysis.nodes_in_creation_order() {
            let HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) = node.kind()
            else {
                continue;
            };
            assert!(
                !selected.complete().fragment().starts_with("</"),
                "{source_text:?}: an end tag was retained as an authored origin"
            );
            let complete = selected.complete().range();
            let raw_name = selected.raw_name().range();
            assert!(
                complete.start() <= raw_name.start() && raw_name.end() <= complete.end(),
                "{source_text:?}: raw-name evidence escapes its complete tag"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The selected support is exactly the proved cells, and nothing more
// ---------------------------------------------------------------------------

#[test]
fn selected_support_appears_only_in_the_proved_cells() {
    // Every input here reaches a `div` outside the proved selected cells and
    // must still be refused, with nothing created for the refused token. This
    // is the negative-space evidence that this implementation opened exactly
    // the accepted cells and no others.
    for (source_text, expected) in [
        // Outside the actual `in body` mode.
        (
            "<div>",
            HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
        ),
        (
            "</div>",
            HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
        ),
        (
            "<body></body><div>",
            HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
        ),
        (
            "<body></body></div>",
            HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
        ),
        (
            "<body></body></html><div>",
            HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
        ),
        // Unsupported selected syntax.
        ("<body><div id=x>", HtmlTreeCapability::AdmittedTagAttribute),
        ("<body><div/>", HtmlTreeCapability::SelfClosingAdmittedTag),
        (
            "<body><div></div id=x>",
            HtmlTreeCapability::AdmittedTagAttribute,
        ),
        // Shell interaction over an open selected element.
        (
            "<body><div></body>",
            HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement,
        ),
        (
            "<body><div><body>",
            HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement,
        ),
        (
            "<body><div></html>",
            HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement,
        ),
        // Names outside both closed domains stay unproved.
        ("<body><p>", HtmlTreeCapability::UnprovedElementTag),
        ("<body><div><p>", HtmlTreeCapability::UnprovedElementTag),
        ("<body><span>", HtmlTreeCapability::UnprovedElementTag),
    ] {
        let analysis = analyze(source_text);
        let ExpectedCompletion::Unsupported {
            capability,
            token,
            trigger,
        } = project_completion(&analysis)
        else {
            panic!("{source_text:?}: expected an explicit unsupported stop")
        };
        assert_eq!(capability, expected, "{source_text:?}: exact capability");
        assert!(!analysis.is_complete(), "{source_text:?}");

        // The refused token is the first token *after* the committed prefix,
        // and the whole refused token is the trigger. Nothing between the two
        // was partially committed.
        assert_eq!(
            token,
            analysis.coverage().processed_tokens(),
            "{source_text:?}: the refused token is the next unprocessed one"
        );
        let trigger = trigger.expect("a refused tag always has an authored extent");
        assert_eq!(
            analysis.coverage().committed_end(),
            trigger.0,
            "{source_text:?}: committed coverage stops exactly at the refused token"
        );

        // The refused token never became any node's authored origin.
        assert!(
            analysis
                .nodes_in_creation_order()
                .into_iter()
                .all(|node| !matches!(
                    node.authored_source(),
                    Some(HtmlAuthoredSource::StartTag { complete, .. })
                        if span(complete) == trigger
                )),
            "{source_text:?}: the refused trigger leaked as an authored origin"
        );
    }
}

// ---------------------------------------------------------------------------
// Predecessor behaviour is unchanged
// ---------------------------------------------------------------------------

#[test]
fn tc_s1_and_tc_s2_predecessor_behaviour_is_unchanged() {
    // Predecessor sources that contain no `div` must behave exactly as their
    // own accepted GOLD requires. Their own validation modules assert that
    // GOLD in full; this is the TC-S3-side guard that the selected layer did
    // not leak into them.
    for source_text in [
        "<body>",
        "<body></body>",
        "<body></body></html>",
        "<body>x",
        "<body>x</body>",
        "<body></body> ",
        "<body></body>x",
        "x",
    ] {
        let analysis = analyze(source_text);

        // No selected ordinary element, action, or diagnostic anywhere.
        assert!(
            analysis
                .nodes_in_creation_order()
                .into_iter()
                .all(|node| !matches!(
                    node.kind(),
                    HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(_))
                )),
            "{source_text:?}: a selected ordinary element appeared"
        );
        assert!(
            project_closures(&analysis).is_empty(),
            "{source_text:?}: a closure appeared"
        );
        assert!(
            analysis.diagnostics().iter().all(|diagnostic| !matches!(
                diagnostic.code(),
                HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag
                    | HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile
            )),
            "{source_text:?}: a selected ordinary diagnostic appeared"
        );
        assert!(
            analysis.actions().iter().all(|action| !matches!(
                action.kind(),
                HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { .. }
                    | HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
                    | HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag { .. }
            )),
            "{source_text:?}: a selected ordinary action appeared"
        );
    }
}

#[test]
fn shell_element_names_stay_shell_only() {
    // The shell domain must never be stretched to carry the selected ordinary
    // meaning: in a source with both, every `html`/`head`/`body` node is a
    // shell element and every `div` node is a selected ordinary element.
    let analysis = analyze("<body><div><div>x</div></div></body>");
    let mut shell = 0usize;
    let mut selected = 0usize;
    for node in analysis.nodes_in_creation_order() {
        match node.kind() {
            HtmlTreeNodeKind::Element(HtmlElement::Shell(element)) => {
                assert!(matches!(
                    element.name(),
                    HtmlShellElementName::Html
                        | HtmlShellElementName::Head
                        | HtmlShellElementName::Body
                ));
                shell += 1;
            }
            HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(element)) => {
                assert_eq!(element.name(), HtmlSelectedOrdinaryElementName::Div);
                selected += 1;
            }
            _ => {}
        }
    }
    assert_eq!(shell, 3, "exactly html, head, and body are shell elements");
    assert_eq!(selected, 2, "both `div` nodes are selected ordinary");
}

// ---------------------------------------------------------------------------
// The closure-evidence theorem is checked at freeze, not merely satisfied
// ---------------------------------------------------------------------------

/// A well-formed `Document -> html(head, body(div))` parts value whose single
/// closure each test below then perturbs in exactly one way.
///
/// Built by hand rather than by running production, so these tests prove that
/// [`freeze`] itself rejects the corruption — not that the session happens
/// never to produce it.
struct ClosureFixture {
    source: SourceText,
    run: HtmlTokenizerRunResult,
    ids: Vec<HtmlConstructedNodeId>,
}

fn closure_fixture() -> ClosureFixture {
    let source = SourceText::new(SourceId::new(1), "<body><div></div>".to_owned());
    let run = tokenize(&source, generous_limits());
    let mut counter = HtmlConstructedIdentityCounter::new();
    let ids = (0..5)
        .map(|_| {
            let reserved = counter.reserve().expect("identity headroom");
            counter.commit(reserved);
            reserved
        })
        .collect();
    ClosureFixture { source, run, ids }
}

fn closure_parts(fixture: &ClosureFixture) -> HtmlDocumentShellParts {
    let [root, html, head, body, selected] = fixture.ids[..] else {
        panic!("five minted identities")
    };
    let anchor = |start: usize, end: usize| fixture.source.anchor(start, end).expect("valid range");
    let synthesized_shell = |name| {
        HtmlTreeNodeKind::Element(HtmlElement::Shell(HtmlShellElement::new(
            name,
            HtmlShellElementOrigin::Synthesized(HtmlSynthesisCause::ImpliedByDocumentStructure),
        )))
    };

    HtmlDocumentShellParts {
        nodes: vec![
            HtmlTreeNode::new(root, None, vec![html], HtmlTreeNodeKind::Document),
            HtmlTreeNode::new(
                html,
                Some(root),
                vec![head, body],
                synthesized_shell(HtmlShellElementName::Html),
            ),
            HtmlTreeNode::new(
                head,
                Some(html),
                vec![],
                synthesized_shell(HtmlShellElementName::Head),
            ),
            HtmlTreeNode::new(
                body,
                Some(html),
                vec![selected],
                HtmlTreeNodeKind::Element(HtmlElement::Shell(HtmlShellElement::new(
                    HtmlShellElementName::Body,
                    HtmlShellElementOrigin::Authored {
                        complete: anchor(0, 6),
                        raw_name: anchor(1, 5),
                    },
                ))),
            ),
            HtmlTreeNode::new(
                selected,
                Some(body),
                vec![],
                HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(
                    HtmlSelectedOrdinaryElement::new(
                        HtmlSelectedOrdinaryElementName::Div,
                        anchor(6, 11),
                        anchor(7, 10),
                    ),
                )),
            ),
        ],
        root,
        admitted_creation_events: 5,
        diagnostics: vec![],
        actions: vec![
            HtmlTreeAction::new(
                HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                    node: selected,
                    name: HtmlSelectedOrdinaryElementName::Div,
                },
                HtmlTreeTokenTrigger::authored(1, anchor(6, 11)),
            ),
            HtmlTreeAction::new(
                HtmlTreeActionKind::ClosedSelectedOrdinaryElement {
                    node: selected,
                    name: HtmlSelectedOrdinaryElementName::Div,
                },
                HtmlTreeTokenTrigger::authored(2, anchor(11, 17)),
            ),
        ],
        processed_tokens: 3,
        committed_prefix_end: 17,
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
    }
}

fn freeze_closure_parts(
    fixture: &ClosureFixture,
    parts: HtmlDocumentShellParts,
) -> Result<HtmlDocumentShellAnalysis, HtmlTreeFreezeError> {
    freeze(&fixture.source, fixture.run.clone(), parts)
}

#[test]
fn freeze_accepts_the_valid_closure_baseline() {
    let fixture = closure_fixture();
    let analysis =
        freeze_closure_parts(&fixture, closure_parts(&fixture)).expect("valid parts freeze");
    assert_eq!(analysis.node_count(), 5);
    assert_eq!(project_closures(&analysis).len(), 1);
}

#[test]
fn freeze_rejects_a_closure_subject_that_is_not_the_selected_ordinary_element() {
    let fixture = closure_fixture();
    let mut parts = closure_parts(&fixture);
    let body = fixture.ids[3];
    // Name the shell `body` node as the closed selected ordinary element.
    parts.actions[1] = HtmlTreeAction::new(
        HtmlTreeActionKind::ClosedSelectedOrdinaryElement {
            node: body,
            name: HtmlSelectedOrdinaryElementName::Div,
        },
        parts.actions[1].trigger().clone(),
    );
    assert_eq!(
        freeze_closure_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::ClosureSubjectIsNotTheSelectedOrdinaryElement {
            node: body,
            name: HtmlSelectedOrdinaryElementName::Div,
        }
    );
}

#[test]
fn freeze_rejects_a_closure_fabricated_at_end_of_file() {
    let fixture = closure_fixture();
    let mut parts = closure_parts(&fixture);
    let selected = fixture.ids[4];
    // Re-trigger the closure from the end-of-file token, which has no
    // authored extent and may therefore never be closure evidence.
    parts.actions[1] = HtmlTreeAction::new(
        HtmlTreeActionKind::ClosedSelectedOrdinaryElement {
            node: selected,
            name: HtmlSelectedOrdinaryElementName::Div,
        },
        HtmlTreeTokenTrigger::end_of_file(3),
    );
    assert_eq!(
        freeze_closure_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::FabricatedSelectedOrdinaryClosure(selected)
    );
}

#[test]
fn freeze_rejects_a_closure_recorded_twice_for_one_element() {
    let fixture = closure_fixture();
    let mut parts = closure_parts(&fixture);
    let selected = fixture.ids[4];
    let duplicate = parts.actions[1].clone();
    parts.actions.push(duplicate);
    assert_eq!(
        freeze_closure_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::NonLifoSelectedOrdinaryClosure(selected)
    );
}

#[test]
fn freeze_rejects_a_closure_that_was_never_opened() {
    let fixture = closure_fixture();
    let mut parts = closure_parts(&fixture);
    // Drop the insertion action but keep the closure: the closure now names an
    // element that the committed action stream never opened.
    parts.actions.remove(0);
    assert_eq!(
        freeze_closure_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::NonLifoSelectedOrdinaryClosure(fixture.ids[4])
    );
}

#[test]
fn freeze_rejects_a_closure_that_is_not_stack_consistent() {
    // Two nested selected elements closed outermost-first: stack-inconsistent
    // for the selected slice, so freeze must reject it.
    let source = SourceText::new(SourceId::new(1), "<body><div><div></div></div>".to_owned());
    let run = tokenize(&source, generous_limits());
    let mut counter = HtmlConstructedIdentityCounter::new();
    let ids: Vec<HtmlConstructedNodeId> = (0..6)
        .map(|_| {
            let reserved = counter.reserve().expect("identity headroom");
            counter.commit(reserved);
            reserved
        })
        .collect();
    let [root, html, head, body, outer, inner] = ids[..] else {
        panic!("six minted identities")
    };
    let anchor = |start: usize, end: usize| source.anchor(start, end).expect("valid range");
    let synthesized_shell = |name| {
        HtmlTreeNodeKind::Element(HtmlElement::Shell(HtmlShellElement::new(
            name,
            HtmlShellElementOrigin::Synthesized(HtmlSynthesisCause::ImpliedByDocumentStructure),
        )))
    };
    let selected_element = |complete: (usize, usize), raw: (usize, usize)| {
        HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(
            HtmlSelectedOrdinaryElement::new(
                HtmlSelectedOrdinaryElementName::Div,
                anchor(complete.0, complete.1),
                anchor(raw.0, raw.1),
            ),
        ))
    };
    let insertion = |node, token, complete: (usize, usize)| {
        HtmlTreeAction::new(
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                node,
                name: HtmlSelectedOrdinaryElementName::Div,
            },
            HtmlTreeTokenTrigger::authored(token, anchor(complete.0, complete.1)),
        )
    };
    let closing = |node, token, complete: (usize, usize)| {
        HtmlTreeAction::new(
            HtmlTreeActionKind::ClosedSelectedOrdinaryElement {
                node,
                name: HtmlSelectedOrdinaryElementName::Div,
            },
            HtmlTreeTokenTrigger::authored(token, anchor(complete.0, complete.1)),
        )
    };

    let parts = HtmlDocumentShellParts {
        nodes: vec![
            HtmlTreeNode::new(root, None, vec![html], HtmlTreeNodeKind::Document),
            HtmlTreeNode::new(
                html,
                Some(root),
                vec![head, body],
                synthesized_shell(HtmlShellElementName::Html),
            ),
            HtmlTreeNode::new(
                head,
                Some(html),
                vec![],
                synthesized_shell(HtmlShellElementName::Head),
            ),
            HtmlTreeNode::new(
                body,
                Some(html),
                vec![outer],
                HtmlTreeNodeKind::Element(HtmlElement::Shell(HtmlShellElement::new(
                    HtmlShellElementName::Body,
                    HtmlShellElementOrigin::Authored {
                        complete: anchor(0, 6),
                        raw_name: anchor(1, 5),
                    },
                ))),
            ),
            HtmlTreeNode::new(
                outer,
                Some(body),
                vec![inner],
                selected_element((6, 11), (7, 10)),
            ),
            HtmlTreeNode::new(
                inner,
                Some(outer),
                vec![],
                selected_element((11, 16), (12, 15)),
            ),
        ],
        root,
        admitted_creation_events: 6,
        diagnostics: vec![],
        // Outermost-first: `outer` is closed while `inner` is still open.
        actions: vec![
            insertion(outer, 1, (6, 11)),
            insertion(inner, 2, (11, 16)),
            closing(outer, 3, (16, 22)),
            closing(inner, 4, (22, 28)),
        ],
        processed_tokens: 5,
        committed_prefix_end: 28,
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
    };

    assert_eq!(
        freeze(&source, run, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::NonLifoSelectedOrdinaryClosure(outer)
    );
}

#[test]
fn freeze_accepts_a_valid_end_of_file_open_selected_state() {
    // The accepted end-of-file branch leaves the selected element open with no
    // closure at all. Freeze must accept exactly that, without any mutable
    // open-element state travelling into the frozen result.
    let fixture = closure_fixture();
    let mut parts = closure_parts(&fixture);
    parts.actions.pop();
    let analysis = freeze_closure_parts(&fixture, parts).expect("an open selected state freezes");
    assert_eq!(analysis.node_count(), 5);
    assert!(project_closures(&analysis).is_empty());
}
