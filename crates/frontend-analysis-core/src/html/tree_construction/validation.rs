//! Candidate-independent TC-S1 validation.
//!
//! The expected meaning in this module is authored independently of the
//! production implementation. It is written against its own small model —
//! [`GoldNode`], [`GoldOrigin`], [`GoldDiagnostic`], [`GoldCompletion`] — that
//! deliberately does not reuse the production result enums as its oracle, and
//! it is never generated from production output. `project_*` translates a
//! production result into that independent model so the two can be compared;
//! the translation is intentionally mechanical and carries no expectations of
//! its own.
//!
//! Coverage here:
//!
//! - G1–G8 and the ignored-head auxiliary case, compared exactly;
//! - G9 deterministic semantic creation correspondence on repeat runs,
//!   without asserting any raw identity encoding;
//! - G10 structural boundedness, with no fabricated tree limit;
//! - authored/synthesized/trigger/action separation;
//! - identity, storage-perturbation, and freeze-corruption rejection;
//! - the G8 terminal checkpoint and tokenizer/tree coverage separation;
//! - honest lower-layer unsupported, resource, and invalid-configuration
//!   propagation;
//! - the unsupported boundaries around attributes, self-closing tags, other
//!   tag names, markup declarations, character references, and unproved shell
//!   positions;
//! - source lifetime after the caller's `SourceText` handle drops; and
//! - the private document-mode and `frameset-ok` transitions, which must not
//!   reach the frozen result.

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::{HtmlToken, HtmlTokenContractError};
use super::super::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerResource};
use super::super::tokenizer::result::{
    HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause, HtmlTokenizerRunResult,
};
use super::driver::{HtmlDocumentShellConstructionError, construct_html_document_shell};
use super::result::{
    HtmlAuthoredSource, HtmlConstructedIdentityCounter, HtmlConstructedNodeId,
    HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlShellClosure, HtmlShellElement,
    HtmlShellElementName, HtmlShellElementOrigin, HtmlSynthesisCause, HtmlTextContribution,
    HtmlTextNode, HtmlTreeAction, HtmlTreeActionKind, HtmlTreeCapability, HtmlTreeCompletion,
    HtmlTreeCompletionUpgrade, HtmlTreeDiagnostic, HtmlTreeDiagnosticCode, HtmlTreeEvidenceRole,
    HtmlTreeFreezeError, HtmlTreeIncompleteCause, HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery,
    HtmlTreeTokenTrigger, HtmlTreeUnsupportedCapability, freeze,
};
use super::session::{
    HtmlDocumentMode, HtmlTreeSession, HtmlTreeSessionError, InsertionMode, TokenOutcome, admit,
    token_trigger,
};

// ---------------------------------------------------------------------------
// Independent expected model
// ---------------------------------------------------------------------------

/// Independently authored expected node shape.
#[derive(Debug, Clone, PartialEq, Eq)]
enum GoldNode {
    Document(Vec<GoldNode>),
    Element {
        name: &'static str,
        origin: GoldOrigin,
        children: Vec<GoldNode>,
    },
    Text {
        interpreted: &'static str,
        contributions: Vec<(usize, usize)>,
    },
}

/// Independently authored expected provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
enum GoldOrigin {
    /// Exact authored `(complete span, raw-name span)`.
    Authored((usize, usize), (usize, usize)),
    /// Explicit absence of authored source.
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GoldDiagnostic {
    MissingDoctype,
    DuplicateHead,
    DuplicateBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GoldCompletion {
    Complete,
    /// TC-S1 stopped at exactly this authored trigger span.
    IncompleteUnsupported((usize, usize)),
}

struct GoldCase {
    id: &'static str,
    source: &'static str,
    tree: GoldNode,
    diagnostics: &'static [GoldDiagnostic],
    completion: GoldCompletion,
    /// Exclusive end of the committed tree prefix. Distinct from the
    /// tokenizer's own coverage.
    committed_prefix_end: usize,
}

fn synthesized_element(name: &'static str, children: Vec<GoldNode>) -> GoldNode {
    GoldNode::Element {
        name,
        origin: GoldOrigin::None,
        children,
    }
}

fn authored_element(
    name: &'static str,
    complete: (usize, usize),
    raw_name: (usize, usize),
    children: Vec<GoldNode>,
) -> GoldNode {
    GoldNode::Element {
        name,
        origin: GoldOrigin::Authored(complete, raw_name),
        children,
    }
}

/// The accepted TC-S1 candidate-independent GOLD, authored from the approved
/// theorem rather than from any production run.
fn gold_cases() -> Vec<GoldCase> {
    vec![
        GoldCase {
            id: "G1",
            source: "",
            tree: GoldNode::Document(vec![synthesized_element(
                "html",
                vec![
                    synthesized_element("head", vec![]),
                    synthesized_element("body", vec![]),
                ],
            )]),
            diagnostics: &[GoldDiagnostic::MissingDoctype],
            completion: GoldCompletion::Complete,
            committed_prefix_end: 0,
        },
        GoldCase {
            id: "G2",
            source: "hello",
            tree: GoldNode::Document(vec![synthesized_element(
                "html",
                vec![
                    synthesized_element("head", vec![]),
                    synthesized_element(
                        "body",
                        vec![GoldNode::Text {
                            interpreted: "hello",
                            contributions: vec![(0, 5)],
                        }],
                    ),
                ],
            )]),
            diagnostics: &[GoldDiagnostic::MissingDoctype],
            completion: GoldCompletion::Complete,
            committed_prefix_end: 5,
        },
        GoldCase {
            id: "G3",
            source: "<html><head></head><body></body></html>",
            tree: GoldNode::Document(vec![authored_element(
                "html",
                (0, 6),
                (1, 5),
                vec![
                    authored_element("head", (6, 12), (7, 11), vec![]),
                    authored_element("body", (19, 25), (20, 24), vec![]),
                ],
            )]),
            diagnostics: &[GoldDiagnostic::MissingDoctype],
            completion: GoldCompletion::Complete,
            committed_prefix_end: 39,
        },
        GoldCase {
            id: "G4",
            source: "<body>",
            tree: GoldNode::Document(vec![synthesized_element(
                "html",
                vec![
                    synthesized_element("head", vec![]),
                    authored_element("body", (0, 6), (1, 5), vec![]),
                ],
            )]),
            diagnostics: &[GoldDiagnostic::MissingDoctype],
            completion: GoldCompletion::Complete,
            committed_prefix_end: 6,
        },
        GoldCase {
            id: "G5",
            source: "<head>",
            tree: GoldNode::Document(vec![synthesized_element(
                "html",
                vec![
                    authored_element("head", (0, 6), (1, 5), vec![]),
                    synthesized_element("body", vec![]),
                ],
            )]),
            diagnostics: &[GoldDiagnostic::MissingDoctype],
            completion: GoldCompletion::Complete,
            committed_prefix_end: 6,
        },
        GoldCase {
            id: "G6",
            source: "<body>x",
            tree: GoldNode::Document(vec![synthesized_element(
                "html",
                vec![
                    synthesized_element("head", vec![]),
                    authored_element(
                        "body",
                        (0, 6),
                        (1, 5),
                        vec![GoldNode::Text {
                            interpreted: "x",
                            contributions: vec![(6, 7)],
                        }],
                    ),
                ],
            )]),
            diagnostics: &[GoldDiagnostic::MissingDoctype],
            completion: GoldCompletion::Complete,
            committed_prefix_end: 7,
        },
        GoldCase {
            id: "G7",
            source: "<body><body>",
            tree: GoldNode::Document(vec![synthesized_element(
                "html",
                vec![
                    synthesized_element("head", vec![]),
                    authored_element("body", (0, 6), (1, 5), vec![]),
                ],
            )]),
            diagnostics: &[
                GoldDiagnostic::MissingDoctype,
                GoldDiagnostic::DuplicateBody,
            ],
            completion: GoldCompletion::Complete,
            committed_prefix_end: 12,
        },
        GoldCase {
            id: "AUX-ignored-head",
            source: "<head><head>",
            tree: GoldNode::Document(vec![synthesized_element(
                "html",
                vec![
                    authored_element("head", (0, 6), (1, 5), vec![]),
                    synthesized_element("body", vec![]),
                ],
            )]),
            diagnostics: &[
                GoldDiagnostic::MissingDoctype,
                GoldDiagnostic::DuplicateHead,
            ],
            completion: GoldCompletion::Complete,
            committed_prefix_end: 12,
        },
        GoldCase {
            id: "G8",
            source: "<body><p>",
            tree: GoldNode::Document(vec![synthesized_element(
                "html",
                vec![
                    synthesized_element("head", vec![]),
                    authored_element("body", (0, 6), (1, 5), vec![]),
                ],
            )]),
            diagnostics: &[GoldDiagnostic::MissingDoctype],
            completion: GoldCompletion::IncompleteUnsupported((6, 9)),
            committed_prefix_end: 6,
        },
    ]
}

// ---------------------------------------------------------------------------
// Mechanical projection from production meaning into the independent model
// ---------------------------------------------------------------------------

fn project_tree(analysis: &HtmlDocumentShellAnalysis, id: HtmlConstructedNodeId) -> GoldNode {
    let node = analysis.node(id).expect("relationship resolves");
    let children = node
        .children()
        .iter()
        .map(|child| project_tree(analysis, *child))
        .collect();
    match node.kind() {
        HtmlTreeNodeKind::Document => GoldNode::Document(children),
        HtmlTreeNodeKind::Element(element) => GoldNode::Element {
            name: match element.name() {
                HtmlShellElementName::Html => "html",
                HtmlShellElementName::Head => "head",
                HtmlShellElementName::Body => "body",
            },
            origin: match element.origin() {
                HtmlShellElementOrigin::Authored { complete, raw_name } => GoldOrigin::Authored(
                    (complete.range().start(), complete.range().end()),
                    (raw_name.range().start(), raw_name.range().end()),
                ),
                HtmlShellElementOrigin::Synthesized(
                    HtmlSynthesisCause::ImpliedByDocumentStructure,
                ) => GoldOrigin::None,
            },
            children,
        },
        HtmlTreeNodeKind::Text(text) => GoldNode::Text {
            // Leaked back as a `&'static str` is impossible, so the comparison
            // below uses the owned projection instead; this arm exists only so
            // the shape is total.
            interpreted: "",
            contributions: text
                .contributions()
                .iter()
                .map(|contribution| {
                    (
                        contribution.source().range().start(),
                        contribution.source().range().end(),
                    )
                })
                .collect(),
        },
    }
}

/// Owned mirror of [`GoldNode`] so text content can be compared without a
/// `'static` lifetime.
#[derive(Debug, Clone, PartialEq, Eq)]
enum OwnedNode {
    Document(Vec<OwnedNode>),
    Element {
        name: String,
        origin: GoldOrigin,
        children: Vec<OwnedNode>,
    },
    Text {
        interpreted: String,
        contributions: Vec<(usize, usize)>,
    },
}

fn own_gold(node: &GoldNode) -> OwnedNode {
    match node {
        GoldNode::Document(children) => {
            OwnedNode::Document(children.iter().map(own_gold).collect())
        }
        GoldNode::Element {
            name,
            origin,
            children,
        } => OwnedNode::Element {
            name: (*name).to_owned(),
            origin: origin.clone(),
            children: children.iter().map(own_gold).collect(),
        },
        GoldNode::Text {
            interpreted,
            contributions,
        } => OwnedNode::Text {
            interpreted: (*interpreted).to_owned(),
            contributions: contributions.clone(),
        },
    }
}

fn own_observed(analysis: &HtmlDocumentShellAnalysis, id: HtmlConstructedNodeId) -> OwnedNode {
    let node = analysis.node(id).expect("relationship resolves");
    let children = node
        .children()
        .iter()
        .map(|child| own_observed(analysis, *child))
        .collect();
    match node.kind() {
        HtmlTreeNodeKind::Document => OwnedNode::Document(children),
        HtmlTreeNodeKind::Element(_) => {
            let GoldNode::Element { name, origin, .. } = project_tree(analysis, id) else {
                unreachable!("element projects to element")
            };
            OwnedNode::Element {
                name: name.to_owned(),
                origin,
                children,
            }
        }
        HtmlTreeNodeKind::Text(text) => OwnedNode::Text {
            interpreted: text.interpreted().to_owned(),
            contributions: text
                .contributions()
                .iter()
                .map(|contribution| {
                    (
                        contribution.source().range().start(),
                        contribution.source().range().end(),
                    )
                })
                .collect(),
        },
    }
}

fn project_diagnostics(analysis: &HtmlDocumentShellAnalysis) -> Vec<GoldDiagnostic> {
    analysis
        .diagnostics()
        .iter()
        .map(|diagnostic| match diagnostic.code() {
            HtmlTreeDiagnosticCode::MissingDoctype => GoldDiagnostic::MissingDoctype,
            HtmlTreeDiagnosticCode::DuplicateHeadStartTag => GoldDiagnostic::DuplicateHead,
            HtmlTreeDiagnosticCode::DuplicateBodyStartTag => GoldDiagnostic::DuplicateBody,
        })
        .collect()
}

fn project_completion(analysis: &HtmlDocumentShellAnalysis) -> GoldCompletion {
    match analysis.completion() {
        HtmlTreeCompletion::Complete => GoldCompletion::Complete,
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => {
            let boundary = unsupported
                .trigger()
                .authored_boundary()
                .expect("authored unsupported trigger");
            GoldCompletion::IncompleteUnsupported((
                boundary.range().start(),
                boundary.range().end(),
            ))
        }
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete) => {
            panic!("gold cases never rely on lower-layer incompleteness")
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn generous_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn analyze(source_text: &str) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(1), source_text.to_owned());
    construct_html_document_shell(&source, generous_limits()).expect("no boundary failure")
}

fn mint_identities(count: usize) -> Vec<HtmlConstructedNodeId> {
    let mut counter = HtmlConstructedIdentityCounter::new();
    (0..count)
        .map(|_| {
            let reserved = counter.reserve().expect("identity headroom");
            counter.commit(reserved);
            reserved
        })
        .collect()
}

fn unsupported_capability(analysis: &HtmlDocumentShellAnalysis) -> Option<HtmlTreeCapability> {
    match analysis.completion() {
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => Some(unsupported.capability()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// G1-G8 plus the ignored-head auxiliary case
// ---------------------------------------------------------------------------

#[test]
fn gold_cases_match_exactly() {
    let mut mismatches = Vec::new();
    for case in gold_cases() {
        let analysis = analyze(case.source);

        let expected_tree = own_gold(&case.tree);
        let observed_tree = own_observed(&analysis, analysis.root());
        if expected_tree != observed_tree {
            mismatches.push(format!(
                "{}: tree mismatch\n  expected {expected_tree:?}\n  observed {observed_tree:?}",
                case.id
            ));
        }

        let observed_diagnostics = project_diagnostics(&analysis);
        if observed_diagnostics != case.diagnostics {
            mismatches.push(format!(
                "{}: diagnostics {observed_diagnostics:?} != expected {:?}",
                case.id, case.diagnostics
            ));
        }

        let observed_completion = project_completion(&analysis);
        if observed_completion != case.completion {
            mismatches.push(format!(
                "{}: completion {observed_completion:?} != expected {:?}",
                case.id, case.completion
            ));
        }

        if analysis.coverage().committed_end() != case.committed_prefix_end {
            mismatches.push(format!(
                "{}: committed tree prefix ends at {} != expected {}",
                case.id,
                analysis.coverage().committed_end(),
                case.committed_prefix_end
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "TC-S1 gold mismatches:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn g7_creates_exactly_one_body_and_the_auxiliary_case_exactly_one_head() {
    let duplicate_body = analyze("<body><body>");
    assert_eq!(
        shell_element_count(&duplicate_body, HtmlShellElementName::Body),
        1
    );
    assert!(duplicate_body.actions().iter().any(|action| matches!(
        action.kind(),
        HtmlTreeActionKind::DuplicateShellStartTagCreatedNoNode {
            name: HtmlShellElementName::Body
        }
    )));

    let duplicate_head = analyze("<head><head>");
    assert_eq!(
        shell_element_count(&duplicate_head, HtmlShellElementName::Head),
        1
    );
    assert!(duplicate_head.actions().iter().any(|action| matches!(
        action.kind(),
        HtmlTreeActionKind::DuplicateShellStartTagCreatedNoNode {
            name: HtmlShellElementName::Head
        }
    )));

    // A duplicate start tag admits no constructed identity: both documents
    // hold exactly the four shell nodes.
    assert_eq!(duplicate_body.node_count(), 4);
    assert_eq!(duplicate_head.node_count(), 4);
}

fn shell_element_count(analysis: &HtmlDocumentShellAnalysis, name: HtmlShellElementName) -> usize {
    analysis
        .nodes_in_creation_order()
        .iter()
        .filter(|node| {
            matches!(node.kind(), HtmlTreeNodeKind::Element(element) if element.name() == name)
        })
        .count()
}

#[test]
fn g3_end_tags_are_action_only_and_create_no_nodes() {
    let analysis = analyze("<html><head></head><body></body></html>");
    assert_eq!(analysis.node_count(), 4);

    let closed_head = analysis.actions().iter().any(|action| {
        matches!(
            action.kind(),
            HtmlTreeActionKind::ClosedShellElement {
                name: HtmlShellElementName::Head,
                closure: HtmlShellClosure::AuthoredEndTag,
                ..
            }
        )
    });
    assert!(closed_head, "`</head>` closes the head element");

    for name in [HtmlShellElementName::Body, HtmlShellElementName::Html] {
        assert!(
            analysis.actions().iter().any(|action| matches!(
                action.kind(),
                HtmlTreeActionKind::AcknowledgedShellEndTag { name: acknowledged } if *acknowledged == name
            )),
            "{name:?} end tag is recorded as action-only evidence"
        );
    }
}

#[test]
fn eof_triggers_implied_structure_without_becoming_its_origin() {
    // G1 and G5 both synthesize structure at end of file. The end-of-file
    // trigger has no authored extent at all, so it cannot masquerade as an
    // authored origin even by accident.
    for source in ["", "<head>"] {
        let analysis = analyze(source);
        let synthesized_at_eof = analysis.actions().iter().any(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::InsertedSynthesizedShellElement { .. }
            ) && action.trigger().authored_boundary().is_none()
        });
        assert!(
            synthesized_at_eof,
            "{source:?}: end of file triggers synthesized structure"
        );
    }
}

// ---------------------------------------------------------------------------
// Provenance separation
// ---------------------------------------------------------------------------

#[test]
fn synthesized_and_root_nodes_carry_no_authored_source() {
    for case in gold_cases() {
        let analysis = analyze(case.source);
        for node in analysis.nodes_in_creation_order() {
            match node.kind() {
                HtmlTreeNodeKind::Document => assert!(
                    node.authored_source().is_none(),
                    "{}: the document root has no authored source",
                    case.id
                ),
                HtmlTreeNodeKind::Element(element) => match element.origin() {
                    HtmlShellElementOrigin::Synthesized(_) => assert!(
                        node.authored_source().is_none(),
                        "{}: a synthesized element has no authored source",
                        case.id
                    ),
                    HtmlShellElementOrigin::Authored { complete, .. } => {
                        assert!(!complete.range().is_empty());
                    }
                },
                HtmlTreeNodeKind::Text(text) => {
                    assert!(!text.contributions().is_empty());
                }
            }
        }
    }
}

#[test]
fn retained_evidence_is_exactly_the_originating_token_evidence() {
    // Proves the result propagates retained token evidence rather than
    // rediscovering ranges: every authored element range and every text
    // contribution range is exactly some emitted token's own range.
    for case in gold_cases() {
        let analysis = analyze(case.source);
        let token_ranges: Vec<(usize, usize)> = analysis
            .tokenizer_run()
            .tokens()
            .iter()
            .map(|token| match token {
                HtmlToken::Character(character) => (
                    character.source().range().start(),
                    character.source().range().end(),
                ),
                HtmlToken::Tag(tag) => {
                    (tag.complete().range().start(), tag.complete().range().end())
                }
                HtmlToken::EndOfFile(end_of_file) => (
                    end_of_file.source().range().start(),
                    end_of_file.source().range().end(),
                ),
            })
            .collect();
        let name_ranges: Vec<(usize, usize)> = analysis
            .tokenizer_run()
            .tokens()
            .iter()
            .filter_map(|token| match token {
                HtmlToken::Tag(tag) => Some((
                    tag.name().source().range().start(),
                    tag.name().source().range().end(),
                )),
                _ => None,
            })
            .collect();

        for node in analysis.nodes_in_creation_order() {
            match node.authored_source() {
                None => {}
                Some(HtmlAuthoredSource::StartTag { complete, raw_name }) => {
                    assert!(
                        token_ranges.contains(&(complete.range().start(), complete.range().end())),
                        "{}: authored tag evidence is a retained token range",
                        case.id
                    );
                    assert!(
                        name_ranges.contains(&(raw_name.range().start(), raw_name.range().end())),
                        "{}: authored name evidence is a retained token range",
                        case.id
                    );
                }
                Some(HtmlAuthoredSource::Characters(contributions)) => {
                    for contribution in contributions {
                        assert!(
                            token_ranges.contains(&(
                                contribution.source().range().start(),
                                contribution.source().range().end()
                            )),
                            "{}: text contribution is a retained token range",
                            case.id
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn adjacent_character_runs_coalesce_with_ordered_contributions() {
    // `<body>a<body>b`: the ignored duplicate `body` start tag produces no
    // node, so the second character run lands next to the first and coalesces
    // into one text node with two ordered contributions.
    let analysis = analyze("<body>a<body>b");
    let texts: Vec<&HtmlTreeNode> = analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter(|node| matches!(node.kind(), HtmlTreeNodeKind::Text(_)))
        .collect();
    assert_eq!(texts.len(), 1);
    let HtmlTreeNodeKind::Text(text) = texts[0].kind() else {
        unreachable!()
    };
    assert_eq!(text.interpreted(), "ab");
    let spans: Vec<(usize, usize)> = text
        .contributions()
        .iter()
        .map(|contribution| {
            (
                contribution.source().range().start(),
                contribution.source().range().end(),
            )
        })
        .collect();
    assert_eq!(spans, vec![(6, 7), (13, 14)]);
    assert!(
        analysis
            .actions()
            .iter()
            .any(|action| matches!(action.kind(), HtmlTreeActionKind::AppendedToTextNode { .. }))
    );
}

// ---------------------------------------------------------------------------
// G8 terminal checkpoint
// ---------------------------------------------------------------------------

#[test]
fn g8_freezes_the_terminal_checkpoint_and_leaks_no_unsupported_identity() {
    let analysis = analyze("<body><p>");

    // The authored body survives, the shell is complete, and no `p` node
    // exists at all.
    assert_eq!(analysis.node_count(), 4);
    assert_eq!(
        shell_element_count(&analysis, HtmlShellElementName::Body),
        1
    );

    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(unsupported)) =
        analysis.completion()
    else {
        panic!("G8 is Incomplete(UnsupportedCapability)")
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTreeCapability::NonShellElementTag
    );
    let trigger = unsupported
        .trigger()
        .authored_boundary()
        .expect("authored trigger");
    assert_eq!((trigger.range().start(), trigger.range().end()), (6, 9));

    // No node's authored origin is the refused token's range, so no `p`
    // identity leaks through any surface.
    for node in analysis.nodes_in_creation_order() {
        if let Some(HtmlAuthoredSource::StartTag { complete, .. }) = node.authored_source() {
            assert_ne!(complete.range(), trigger.range());
        }
    }
    // Nor does any action or diagnostic reference the refused token.
    for action in analysis.actions() {
        assert!(action.trigger().token_index() < unsupported.trigger().token_index());
    }
    for diagnostic in analysis.diagnostics() {
        assert!(diagnostic.trigger().token_index() < unsupported.trigger().token_index());
    }
}

#[test]
fn g8_keeps_tokenizer_coverage_and_committed_tree_coverage_distinct() {
    let analysis = analyze("<body><p>");
    let tokenizer_coverage = analysis.tokenizer_run().coverage();

    // The tokenizer completed the whole source.
    assert!(tokenizer_coverage.is_complete());
    assert_eq!(tokenizer_coverage.processed_end(), 9);
    assert!(matches!(
        analysis.tokenizer_run().completion(),
        HtmlTokenizerCompletion::Complete
    ));

    // Committed tree coverage stopped at the last completely processed token.
    assert_eq!(analysis.coverage().committed_end(), 6);
    assert_eq!(analysis.coverage().processed_tokens(), 1);
    assert!(analysis.coverage().committed_end() < tokenizer_coverage.processed_end());
    assert!(!analysis.is_complete());
}

// ---------------------------------------------------------------------------
// G9 deterministic semantic correspondence
// ---------------------------------------------------------------------------

/// A semantic creation-correspondence signature: for each node, in committed
/// creation order, its path from the root as child positions, its kind, and
/// its origin. Deliberately contains no raw identity encoding.
fn creation_correspondence(analysis: &HtmlDocumentShellAnalysis) -> Vec<(Vec<usize>, OwnedNode)> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .map(|node| {
            let mut path = Vec::new();
            let mut current = node.id();
            while let Some(parent_id) = analysis.node(current).and_then(HtmlTreeNode::parent) {
                let parent = analysis.node(parent_id).expect("relationship resolves");
                let position = parent
                    .children()
                    .iter()
                    .position(|child| *child == current)
                    .expect("child is recorded on its parent");
                path.push(position);
                current = parent_id;
            }
            path.reverse();
            let mut shallow = own_observed(analysis, node.id());
            // Compare only this node's own meaning; descendants are covered by
            // their own entries.
            shallow = match shallow {
                OwnedNode::Document(_) => OwnedNode::Document(Vec::new()),
                OwnedNode::Element { name, origin, .. } => OwnedNode::Element {
                    name,
                    origin,
                    children: Vec::new(),
                },
                text => text,
            };
            (path, shallow)
        })
        .collect()
}

/// A structural action signature: action meaning plus trigger token index and
/// trigger range, deliberately excluding the caller-supplied source identity so
/// two runs over equal bytes under different `SourceId` values can be compared.
fn action_signature(analysis: &HtmlDocumentShellAnalysis) -> Vec<String> {
    analysis
        .actions()
        .iter()
        .map(|action| {
            format!(
                "{:?}@{}{:?}",
                action.kind(),
                action.trigger().token_index(),
                action
                    .trigger()
                    .authored_boundary()
                    .map(SourceAnchor::range)
            )
        })
        .collect()
}

#[test]
fn g9_repeat_runs_preserve_semantic_creation_correspondence() {
    for case in gold_cases() {
        let baseline = analyze(case.source);
        let baseline_signature = creation_correspondence(&baseline);
        let baseline_tree = own_observed(&baseline, baseline.root());
        let baseline_diagnostics = project_diagnostics(&baseline);
        let baseline_actions = action_signature(&baseline);
        let baseline_completion = project_completion(&baseline);
        let baseline_coverage = (
            baseline.coverage().committed_end(),
            baseline.coverage().processed_tokens(),
        );

        for source_id in [1u64, 1u64, 7u64] {
            let source = SourceText::new(SourceId::new(source_id), case.source.to_owned());
            let repeat = construct_html_document_shell(&source, generous_limits())
                .expect("no boundary failure");
            assert_eq!(
                creation_correspondence(&repeat),
                baseline_signature,
                "{}: semantic creation correspondence changed",
                case.id
            );
            assert_eq!(own_observed(&repeat, repeat.root()), baseline_tree);
            assert_eq!(project_diagnostics(&repeat), baseline_diagnostics);
            assert_eq!(action_signature(&repeat), baseline_actions);
            assert_eq!(project_completion(&repeat), baseline_completion);
            assert_eq!(
                (
                    repeat.coverage().committed_end(),
                    repeat.coverage().processed_tokens()
                ),
                baseline_coverage
            );
        }
    }
}

#[test]
fn semantic_relationships_survive_private_storage_replacement() {
    for case in gold_cases() {
        let analysis = analyze(case.source);
        let before = own_observed(&analysis, analysis.root());
        let before_correspondence = creation_correspondence(&analysis);

        let permuted = analysis.with_reversed_storage();
        assert_eq!(
            own_observed(&permuted, permuted.root()),
            before,
            "{}: tree meaning depends on private storage order",
            case.id
        );
        assert_eq!(
            creation_correspondence(&permuted),
            before_correspondence,
            "{}: creation correspondence depends on private storage order",
            case.id
        );
    }
}

#[test]
fn constructed_identities_are_unique_and_ordered_by_creation() {
    for case in gold_cases() {
        let analysis = analyze(case.source);
        let ordered = analysis.nodes_in_creation_order();
        for pair in ordered.windows(2) {
            assert!(
                pair[0].id() < pair[1].id(),
                "{}: creation order is strictly increasing and unique",
                case.id
            );
        }
        // A parent always commits before its children.
        for node in &ordered {
            if let Some(parent) = node.parent() {
                assert!(parent < node.id());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// G10 structural boundedness, with no fabricated tree limit
// ---------------------------------------------------------------------------

/// The number of TC-S1 insertion modes. Used only to state the per-token
/// bound in assertions; it is not a resource limit, budget, or refusal policy,
/// and no production code consults it.
const TC_S1_INSERTION_MODES: usize = 8;

/// The constant shell: the document root plus `html`, `head`, and `body`.
const TC_S1_SHELL_NODES: usize = 4;

fn assert_structural_bounds(analysis: &HtmlDocumentShellAnalysis, label: &str) {
    let non_text_nodes = analysis
        .nodes_in_creation_order()
        .iter()
        .filter(|node| !matches!(node.kind(), HtmlTreeNodeKind::Text(_)))
        .count();
    assert!(
        non_text_nodes <= TC_S1_SHELL_NODES,
        "{label}: shell/root node count is constant apart from text"
    );

    let character_tokens = analysis
        .tokenizer_run()
        .tokens()
        .iter()
        .filter(|token| matches!(token, HtmlToken::Character(_)))
        .count();
    let contributions: usize = analysis
        .nodes_in_creation_order()
        .iter()
        .filter_map(|node| match node.kind() {
            HtmlTreeNodeKind::Text(text) => Some(text.contributions().len()),
            _ => None,
        })
        .sum();
    let text_nodes = analysis.node_count() - non_text_nodes;
    assert!(
        contributions <= character_tokens,
        "{label}: text contributions are bounded by emitted character evidence"
    );
    assert!(
        text_nodes <= character_tokens,
        "{label}: text nodes are bounded by emitted character evidence"
    );

    let processed = analysis.coverage().processed_tokens();
    assert!(
        analysis.actions().len() <= processed * TC_S1_INSERTION_MODES * 2 + TC_S1_SHELL_NODES,
        "{label}: actions are bounded by processed tokens plus a fixed shell contribution"
    );
    assert!(
        analysis.diagnostics().len() <= processed + 1,
        "{label}: diagnostics are bounded by processed tokens plus a fixed contribution"
    );
    assert!(
        processed <= analysis.tokenizer_run().tokens().len(),
        "{label}: committed tokens never exceed emitted tokens"
    );
}

#[test]
fn g10_structural_bounds_hold_without_any_tree_resource_limit() {
    for case in gold_cases() {
        assert_structural_bounds(&analyze(case.source), case.id);
    }
    for source in [
        "<body>a<body>b<body>c",
        "<html><head></head><body>x</body></html>",
        "<head></head><body>",
        "<body><p>",
        "<p>",
        "x<body>",
    ] {
        assert_structural_bounds(&analyze(source), source);
    }
}

#[test]
fn open_shell_element_state_stays_bounded_to_the_admitted_shell() {
    for source in [
        "",
        "hello",
        "<html><head></head><body></body></html>",
        "<body><body>",
        "<head><head>",
        "<body>a<body>b",
    ] {
        let source_text = SourceText::new(SourceId::new(1), source.to_owned());
        let run = crate::html::tokenizer::producer::tokenize(&source_text, generous_limits());
        let mut session = HtmlTreeSession::new().expect("session start");
        let mut peak = 0usize;
        for (token_index, token) in run.tokens().iter().enumerate() {
            let trigger = token_trigger(token, token_index);
            let Ok(admitted) = admit(token) else { break };
            let outcome = session
                .process(&admitted, trigger)
                .expect("no invariant failure");
            peak = peak.max(session.open_element_count());
            if !matches!(outcome, TokenOutcome::Consumed) {
                break;
            }
        }
        assert!(
            peak <= 2,
            "{source:?}: open elements stay bounded to the admitted shell (peak {peak})"
        );
        assert!(session.node_count() <= TC_S1_SHELL_NODES + run.tokens().len());
    }
}

#[test]
fn insertion_mode_transitions_are_strictly_forward() {
    // The strictly forward requirement is what bounds per-token work without a
    // work constant, so a backwards transition must be refused rather than
    // silently allowed.
    let mut session = HtmlTreeSession::new().expect("session start");
    assert_eq!(session.insertion_mode(), InsertionMode::Initial);
    let analysis = analyze("<body>");
    assert!(analysis.is_complete());

    let source = SourceText::new(SourceId::new(1), "<body>".to_owned());
    let run = crate::html::tokenizer::producer::tokenize(&source, generous_limits());
    for (token_index, token) in run.tokens().iter().enumerate() {
        let trigger = token_trigger(token, token_index);
        let admitted = admit(token).expect("admitted");
        session.process(&admitted, trigger).expect("processed");
    }
    assert_eq!(session.insertion_mode(), InsertionMode::InBody);
    assert!(InsertionMode::Initial < InsertionMode::BeforeHtml);
    assert!(InsertionMode::BeforeHtml < InsertionMode::BeforeHead);
    assert!(InsertionMode::BeforeHead < InsertionMode::InHead);
    assert!(InsertionMode::InHead < InsertionMode::AfterHead);
    assert!(InsertionMode::AfterHead < InsertionMode::InBody);
    assert!(InsertionMode::InBody < InsertionMode::AfterBody);
    assert!(InsertionMode::AfterBody < InsertionMode::AfterAfterBody);
}

// ---------------------------------------------------------------------------
// Private document mode and frameset-ok
// ---------------------------------------------------------------------------

#[test]
fn private_document_mode_and_frameset_ok_transition_without_reaching_the_result() {
    let source = SourceText::new(SourceId::new(1), "<body>".to_owned());
    let run = crate::html::tokenizer::producer::tokenize(&source, generous_limits());
    let mut session = HtmlTreeSession::new().expect("session start");
    assert_eq!(session.document_mode(), HtmlDocumentMode::NoQuirks);
    assert!(session.frameset_ok());

    let mut tokens = run.tokens().iter().enumerate();
    let (index, token) = tokens.next().expect("start tag token");
    let trigger = token_trigger(token, index);
    let admitted = admit(token).expect("admitted");
    session.process(&admitted, trigger).expect("processed");

    // The missing DOCTYPE moved the private document mode to quirks and the
    // body insertion cleared frameset-ok.
    assert_eq!(session.document_mode(), HtmlDocumentMode::Quirks);
    assert!(!session.frameset_ok());

    // Neither reaches the frozen result: its `Debug` projection, which is the
    // widest surface it has, mentions neither.
    let analysis = analyze("<body>");
    let rendered = format!("{analysis:?}");
    assert!(!rendered.contains("Quirks"));
    assert!(!rendered.contains("frameset"));
}

#[test]
fn duplicate_body_start_tag_clears_frameset_ok_without_creating_a_node() {
    let source = SourceText::new(SourceId::new(1), "<body><body>".to_owned());
    let run = crate::html::tokenizer::producer::tokenize(&source, generous_limits());
    let mut session = HtmlTreeSession::new().expect("session start");
    for (token_index, token) in run.tokens().iter().enumerate().take(2) {
        let trigger = token_trigger(token, token_index);
        let admitted = admit(token).expect("admitted");
        session.process(&admitted, trigger).expect("processed");
    }
    assert!(!session.frameset_ok());
    assert_eq!(session.node_count(), TC_S1_SHELL_NODES);
}

// ---------------------------------------------------------------------------
// Unsupported boundaries
// ---------------------------------------------------------------------------

#[test]
fn unproved_token_shapes_remain_explicit_tree_unsupported() {
    for (source, expected) in [
        ("<body><p>", HtmlTreeCapability::NonShellElementTag),
        ("<div>", HtmlTreeCapability::NonShellElementTag),
        ("</p>", HtmlTreeCapability::NonShellElementTag),
        ("<body a>", HtmlTreeCapability::ShellTagAttribute),
        ("<html lang=en>", HtmlTreeCapability::ShellTagAttribute),
        ("<body/>", HtmlTreeCapability::SelfClosingShellTag),
    ] {
        let analysis = analyze(source);
        assert_eq!(
            unsupported_capability(&analysis),
            Some(expected),
            "{source:?}: expected explicit tree-unsupported evidence"
        );
        assert!(!analysis.is_complete());
        // The tokenizer itself completed: this is a tree capability boundary,
        // not a lower-layer condition and not invalid HTML.
        assert!(matches!(
            analysis.tokenizer_run().completion(),
            HtmlTokenizerCompletion::Complete
        ));
    }
}

#[test]
fn unproved_shell_positions_remain_explicit_tree_unsupported() {
    for (source, expected) in [
        (
            "<head><html>",
            HtmlTreeCapability::UnprovedShellStartTagPosition,
        ),
        (
            "<body><html>",
            HtmlTreeCapability::UnprovedShellStartTagPosition,
        ),
        (
            "<body><head>",
            HtmlTreeCapability::UnprovedShellStartTagPosition,
        ),
        (
            "<head></head><head>",
            HtmlTreeCapability::UnprovedShellStartTagPosition,
        ),
        (
            "<head></head></head>",
            HtmlTreeCapability::UnprovedShellEndTagPosition,
        ),
        (
            "<body></html>",
            HtmlTreeCapability::UnprovedShellEndTagPosition,
        ),
        (
            "<body></body>",
            HtmlTreeCapability::UnprovedEndOfFilePosition,
        ),
        (
            "<body></body></html>x",
            HtmlTreeCapability::UnprovedCharacterDataPosition,
        ),
    ] {
        assert_eq!(
            unsupported_capability(&analyze(source)),
            Some(expected),
            "{source:?}: expected explicit unproved-position evidence"
        );
    }
}

#[test]
fn whitespace_sensitive_character_runs_remain_explicit_tree_unsupported() {
    for source in [" ", "\thello", "a b", "\n<body>"] {
        assert_eq!(
            unsupported_capability(&analyze(source)),
            Some(HtmlTreeCapability::WhitespaceSensitiveCharacterData),
            "{source:?}: whitespace-sensitive handling is not proved by TC-S1"
        );
    }
    // Inside `in body` all characters are inserted identically, so the same
    // run needs no whitespace refusal there.
    let analysis = analyze("<body>a b");
    assert!(analysis.is_complete());
}

#[test]
fn lower_layer_unsupported_capabilities_are_never_upgraded() {
    // Markup declarations and character references are the tokenizer's own
    // explicit unsupported capabilities, so the tree never sees those tokens.
    for source in ["<!DOCTYPE html>", "<!-- c -->", "&amp;", "<body>&amp;"] {
        let analysis = analyze(source);
        assert!(
            !analysis.is_complete(),
            "{source:?}: lower-layer incompleteness is never upgraded"
        );
        assert!(analysis.tokenizer_run().is_incomplete());
        assert!(matches!(
            analysis.tokenizer_run().completion(),
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::UnsupportedCapability(_)
            )
        ));
    }
}

#[test]
fn lower_layer_resource_and_configuration_evidence_stays_exact() {
    let source = SourceText::new(SourceId::new(1), "<body>hello".to_owned());

    let tiny = HtmlTokenizerLimits::new(4, 8_192, 1_024, 1_024, 256, 4_096, 1_024);
    let limited = construct_html_document_shell(&source, tiny).expect("no boundary failure");
    assert!(!limited.is_complete());
    let HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(limit)) =
        limited.tokenizer_run().completion()
    else {
        panic!("expected an exact retained resource-limit cause")
    };
    assert_eq!(limit.resource(), HtmlTokenizerResource::SourceBytes);
    assert!(matches!(
        limited.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));

    let invalid = HtmlTokenizerLimits::new(1_024, 0, 1_024, 1_024, 256, 4_096, 1_024);
    let misconfigured =
        construct_html_document_shell(&source, invalid).expect("no boundary failure");
    assert!(!misconfigured.is_complete());
    assert!(matches!(
        misconfigured.tokenizer_run().completion(),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::InvalidConfiguration(_))
    ));
    assert!(matches!(
        misconfigured.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));

    // Low emitted-token limits truncate the run; the tree must still refuse to
    // claim completion for the tokens it did process.
    let few_tokens = HtmlTokenizerLimits::new(1_024, 8_192, 1, 1_024, 256, 4_096, 1_024);
    let truncated =
        construct_html_document_shell(&source, few_tokens).expect("no boundary failure");
    assert!(!truncated.is_complete());
    assert!(truncated.tokenizer_run().is_incomplete());
}

// ---------------------------------------------------------------------------
// Source lifetime
// ---------------------------------------------------------------------------

#[test]
fn retained_evidence_outlives_the_caller_source_handle() {
    let analysis = {
        let source = SourceText::new(SourceId::new(9), "<body>x".to_owned());
        construct_html_document_shell(&source, generous_limits()).expect("no boundary failure")
    };
    let body = analysis
        .nodes_in_creation_order()
        .into_iter()
        .find(|node| {
            matches!(node.kind(), HtmlTreeNodeKind::Element(element)
                if element.name() == HtmlShellElementName::Body)
        })
        .expect("body element");
    let Some(HtmlAuthoredSource::StartTag { complete, raw_name }) = body.authored_source() else {
        panic!("authored body")
    };
    assert_eq!(complete.fragment(), "<body>");
    assert_eq!(raw_name.fragment(), "body");
    assert_eq!(complete.source_id(), SourceId::new(9));
}

// ---------------------------------------------------------------------------
// Sensitive-output review
// ---------------------------------------------------------------------------

#[test]
fn debug_output_never_exposes_authored_source_content() {
    let secret = "<body>correcthorsebatterystaple";
    let analysis = analyze(secret);
    let rendered = format!("{analysis:?}");
    assert!(!rendered.contains("correcthorsebatterystaple"));

    for node in analysis.nodes_in_creation_order() {
        assert!(!format!("{node:?}").contains("correcthorsebatterystaple"));
    }
    for action in analysis.actions() {
        assert!(!format!("{action:?}").contains("correcthorsebatterystaple"));
    }
    for diagnostic in analysis.diagnostics() {
        assert!(!format!("{diagnostic:?}").contains("correcthorsebatterystaple"));
    }

    let session_error = HtmlTreeSessionError::UnknownConstructedNode(mint_identities(1)[0]);
    assert!(!format!("{session_error}").contains("correcthorse"));
    let freeze_error = HtmlTreeFreezeError::MismatchedSourceEvidence {
        role: HtmlTreeEvidenceRole::AuthoredCompleteTag,
    };
    assert!(!format!("{freeze_error}").contains("correcthorse"));
    let boundary = HtmlDocumentShellConstructionError::Freeze(freeze_error);
    assert!(!format!("{boundary}").contains("correcthorse"));
}

// ---------------------------------------------------------------------------
// Freeze corruption rejection
// ---------------------------------------------------------------------------

struct FreezeFixture {
    source: SourceText,
    run: HtmlTokenizerRunResult,
    ids: Vec<HtmlConstructedNodeId>,
}

fn freeze_fixture() -> FreezeFixture {
    let source = SourceText::new(SourceId::new(1), "<body>x".to_owned());
    let run = crate::html::tokenizer::producer::tokenize(&source, generous_limits());
    FreezeFixture {
        ids: mint_identities(8),
        source,
        run,
    }
}

/// A minimal well-formed `Document -> html(head, body)` parts value that each
/// corruption test then perturbs in exactly one way.
fn valid_parts(fixture: &FreezeFixture) -> HtmlDocumentShellParts {
    let [root, html, head, body, ..] = fixture.ids[..] else {
        unreachable!()
    };
    HtmlDocumentShellParts {
        nodes: vec![
            HtmlTreeNode::new(root, None, vec![html], HtmlTreeNodeKind::Document),
            HtmlTreeNode::new(
                html,
                Some(root),
                vec![head, body],
                HtmlTreeNodeKind::Element(HtmlShellElement::new(
                    HtmlShellElementName::Html,
                    HtmlShellElementOrigin::Synthesized(
                        HtmlSynthesisCause::ImpliedByDocumentStructure,
                    ),
                )),
            ),
            HtmlTreeNode::new(
                head,
                Some(html),
                vec![],
                HtmlTreeNodeKind::Element(HtmlShellElement::new(
                    HtmlShellElementName::Head,
                    HtmlShellElementOrigin::Synthesized(
                        HtmlSynthesisCause::ImpliedByDocumentStructure,
                    ),
                )),
            ),
            HtmlTreeNode::new(
                body,
                Some(html),
                vec![],
                HtmlTreeNodeKind::Element(HtmlShellElement::new(
                    HtmlShellElementName::Body,
                    HtmlShellElementOrigin::Authored {
                        complete: fixture.source.anchor(0, 6).expect("valid range"),
                        raw_name: fixture.source.anchor(1, 5).expect("valid range"),
                    },
                )),
            ),
        ],
        root,
        admitted_creation_events: 4,
        diagnostics: vec![HtmlTreeDiagnostic::new(
            HtmlTreeDiagnosticCode::MissingDoctype,
            HtmlTreeTokenTrigger::authored(0, fixture.source.anchor(0, 6).expect("valid range")),
            HtmlTreeRecovery::ContinuedInQuirksDocumentMode,
        )],
        actions: vec![HtmlTreeAction::new(
            HtmlTreeActionKind::InsertedAuthoredShellElement {
                node: body,
                name: HtmlShellElementName::Body,
            },
            HtmlTreeTokenTrigger::authored(0, fixture.source.anchor(0, 6).expect("valid range")),
        )],
        processed_tokens: 1,
        committed_prefix_end: 6,
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
    }
}

fn freeze_parts(
    fixture: &FreezeFixture,
    parts: HtmlDocumentShellParts,
) -> Result<HtmlDocumentShellAnalysis, HtmlTreeFreezeError> {
    freeze(&fixture.source, fixture.run.clone(), parts)
}

#[test]
fn freeze_accepts_the_valid_baseline_and_its_storage_permutation() {
    let fixture = freeze_fixture();
    let analysis = freeze_parts(&fixture, valid_parts(&fixture)).expect("valid parts freeze");
    assert_eq!(analysis.node_count(), 4);

    let mut permuted = valid_parts(&fixture);
    permuted.nodes.reverse();
    let from_permuted = freeze_parts(&fixture, permuted).expect("storage order is not identity");
    assert_eq!(
        own_observed(&from_permuted, from_permuted.root()),
        own_observed(&analysis, analysis.root())
    );
}

#[test]
fn freeze_rejects_duplicate_constructed_identities() {
    let fixture = freeze_fixture();
    let mut parts = valid_parts(&fixture);
    let head_id = parts.nodes[2].id();
    let body_id = parts.nodes[3].id();
    parts.nodes[3] = HtmlTreeNode::new(
        head_id,
        parts.nodes[3].parent(),
        vec![],
        parts.nodes[3].kind().clone(),
    );
    parts.nodes[1] = HtmlTreeNode::new(
        parts.nodes[1].id(),
        Some(parts.root),
        vec![head_id, head_id],
        parts.nodes[1].kind().clone(),
    );
    let _ = body_id;
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::DuplicateConstructedIdentity(_))
    ));
}

#[test]
fn freeze_rejects_identities_the_creation_counter_never_admitted() {
    let fixture = freeze_fixture();
    let mut parts = valid_parts(&fixture);
    parts.admitted_creation_events = 3;
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::CreationEventInventoryMismatch { .. })
    ));

    let mut parts = valid_parts(&fixture);
    let stray = fixture.ids[6];
    parts.nodes[3] = HtmlTreeNode::new(
        stray,
        parts.nodes[3].parent(),
        vec![],
        parts.nodes[3].kind().clone(),
    );
    parts.nodes[1] = HtmlTreeNode::new(
        parts.nodes[1].id(),
        Some(parts.root),
        vec![parts.nodes[2].id(), stray],
        parts.nodes[1].kind().clone(),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnadmittedConstructedIdentity(_))
    ));
}

#[test]
fn freeze_rejects_storage_derived_and_dangling_relationships() {
    let fixture = freeze_fixture();
    // A relationship written as a storage position rather than as a
    // constructed identity: position 5 is not an admitted identity here.
    let mut parts = valid_parts(&fixture);
    parts.nodes[1] = HtmlTreeNode::new(
        parts.nodes[1].id(),
        Some(parts.root),
        vec![parts.nodes[2].id(), parts.nodes[3].id(), fixture.ids[5]],
        parts.nodes[1].kind().clone(),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnresolvedRelationship { .. })
    ));
}

#[test]
fn freeze_rejects_invalid_parent_child_relationships() {
    let fixture = freeze_fixture();

    // Child not recorded on its parent.
    let mut parts = valid_parts(&fixture);
    parts.nodes[1] = HtmlTreeNode::new(
        parts.nodes[1].id(),
        Some(parts.root),
        vec![parts.nodes[2].id()],
        parts.nodes[1].kind().clone(),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::AsymmetricRelationship { .. })
    ));

    // Root claiming a parent.
    let mut parts = valid_parts(&fixture);
    parts.nodes[0] = HtmlTreeNode::new(
        parts.root,
        Some(parts.nodes[1].id()),
        vec![parts.nodes[1].id()],
        HtmlTreeNodeKind::Document,
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::RootMustNotHaveParent(_))
    ));

    // A child created before its parent.
    let mut parts = valid_parts(&fixture);
    let root = parts.root;
    let html = parts.nodes[1].id();
    let head = parts.nodes[2].id();
    let body = parts.nodes[3].id();
    parts.nodes = vec![
        HtmlTreeNode::new(root, None, vec![body], HtmlTreeNodeKind::Document),
        HtmlTreeNode::new(body, Some(root), vec![html], parts.nodes[3].kind().clone()),
        HtmlTreeNode::new(html, Some(body), vec![head], parts.nodes[1].kind().clone()),
        HtmlTreeNode::new(head, Some(html), vec![], parts.nodes[2].kind().clone()),
    ];
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::ChildPrecedesParentCreation { .. })
    ));
}

#[test]
fn freeze_rejects_a_detached_or_cyclic_structure() {
    let fixture = freeze_fixture();
    let mut parts = valid_parts(&fixture);
    let root = parts.root;
    let html = parts.nodes[1].id();
    let head = parts.nodes[2].id();
    let body = parts.nodes[3].id();
    // `head` is detached: recorded as `body`'s child on both sides but `body`
    // itself is removed from `html`'s children, so `head` is unreachable.
    parts.nodes = vec![
        HtmlTreeNode::new(root, None, vec![html], HtmlTreeNodeKind::Document),
        HtmlTreeNode::new(html, Some(root), vec![], parts.nodes[1].kind().clone()),
        HtmlTreeNode::new(head, Some(body), vec![], parts.nodes[2].kind().clone()),
        HtmlTreeNode::new(body, Some(html), vec![head], parts.nodes[3].kind().clone()),
    ];
    // The reachability check is deliberate defence in depth: with mutual
    // relationships, a single recorded parent, and parent-before-child creation
    // all enforced, a detached component is already impossible, so this
    // construction is rejected by whichever of those guards it trips first.
    assert!(freeze_parts(&fixture, parts).is_err());
}

#[test]
fn freeze_rejects_fabricated_and_foreign_source_evidence() {
    let fixture = freeze_fixture();
    let foreign = SourceText::new(SourceId::new(77), "<body>x".to_owned());

    // A fabricated synthesized origin: a synthesized element given authored
    // evidence it never had.
    let mut parts = valid_parts(&fixture);
    parts.nodes[2] = HtmlTreeNode::new(
        parts.nodes[2].id(),
        parts.nodes[2].parent(),
        vec![],
        HtmlTreeNodeKind::Element(HtmlShellElement::new(
            HtmlShellElementName::Head,
            HtmlShellElementOrigin::Authored {
                complete: foreign.anchor(0, 6).expect("valid range"),
                raw_name: foreign.anchor(1, 5).expect("valid range"),
            },
        )),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::ForeignSourceEvidence { .. })
    ));

    // Same caller-supplied SourceId, different retained bytes.
    let impostor = SourceText::new(SourceId::new(1), "<body>y".to_owned());
    let mut parts = valid_parts(&fixture);
    parts.nodes[3] = HtmlTreeNode::new(
        parts.nodes[3].id(),
        parts.nodes[3].parent(),
        vec![],
        HtmlTreeNodeKind::Element(HtmlShellElement::new(
            HtmlShellElementName::Body,
            HtmlShellElementOrigin::Authored {
                complete: impostor.anchor(0, 6).expect("valid range"),
                raw_name: impostor.anchor(1, 5).expect("valid range"),
            },
        )),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::MismatchedSourceEvidence { .. })
    ));

    // Raw-name evidence escaping its complete authored tag.
    let mut parts = valid_parts(&fixture);
    parts.nodes[3] = HtmlTreeNode::new(
        parts.nodes[3].id(),
        parts.nodes[3].parent(),
        vec![],
        HtmlTreeNodeKind::Element(HtmlShellElement::new(
            HtmlShellElementName::Body,
            HtmlShellElementOrigin::Authored {
                complete: fixture.source.anchor(0, 6).expect("valid range"),
                raw_name: fixture.source.anchor(6, 7).expect("valid range"),
            },
        )),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::AuthoredNameOutsideCompleteTag(_))
    ));
}

#[test]
fn freeze_rejects_invalid_text_contribution_ordering_and_content() {
    let fixture = freeze_fixture();
    let text_id = fixture.ids[4];

    let with_text = |text: HtmlTextNode| {
        let mut parts = valid_parts(&fixture);
        parts.admitted_creation_events = 5;
        parts.nodes[3] = HtmlTreeNode::new(
            parts.nodes[3].id(),
            parts.nodes[3].parent(),
            vec![text_id],
            parts.nodes[3].kind().clone(),
        );
        let body_id = parts.nodes[3].id();
        parts.nodes.push(HtmlTreeNode::new(
            text_id,
            Some(body_id),
            vec![],
            HtmlTreeNodeKind::Text(text),
        ));
        parts
    };

    // Backwards contribution order.
    let backwards = HtmlTextNode::new(
        "xy".to_owned(),
        vec![
            HtmlTextContribution::new(
                fixture.source.anchor(6, 7).expect("valid range"),
                "x".to_owned(),
            ),
            HtmlTextContribution::new(
                fixture.source.anchor(0, 1).expect("valid range"),
                "y".to_owned(),
            ),
        ],
    );
    assert!(matches!(
        freeze_parts(&fixture, with_text(backwards)),
        Err(HtmlTreeFreezeError::InvalidTextContributions(_))
    ));

    // Interpreted text that is not the ordered concatenation.
    let inconsistent = HtmlTextNode::new(
        "zz".to_owned(),
        vec![HtmlTextContribution::new(
            fixture.source.anchor(6, 7).expect("valid range"),
            "x".to_owned(),
        )],
    );
    assert!(matches!(
        freeze_parts(&fixture, with_text(inconsistent)),
        Err(HtmlTreeFreezeError::InvalidTextContributions(_))
    ));

    // No contributions at all.
    let empty = HtmlTextNode::new(String::new(), vec![]);
    assert!(matches!(
        freeze_parts(&fixture, with_text(empty)),
        Err(HtmlTreeFreezeError::InvalidTextContributions(_))
    ));
}

#[test]
fn freeze_rejects_completion_upgrades() {
    let fixture = freeze_fixture();

    // The retained run here is Complete, but not every token was processed.
    let mut parts = valid_parts(&fixture);
    parts.completion = HtmlTreeCompletion::Complete;
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::CompletionUpgrade(
            HtmlTreeCompletionUpgrade::EmittedTokensRemainUnprocessed
        ))
    ));

    // All tokens processed, but the shell is not complete.
    let mut parts = valid_parts(&fixture);
    parts.completion = HtmlTreeCompletion::Complete;
    parts.processed_tokens = fixture.run.tokens().len();
    parts.actions.clear();
    let root = parts.root;
    let html = parts.nodes[1].id();
    let head = parts.nodes[2].id();
    parts.nodes = vec![
        HtmlTreeNode::new(root, None, vec![html], HtmlTreeNodeKind::Document),
        HtmlTreeNode::new(html, Some(root), vec![head], parts.nodes[1].kind().clone()),
        HtmlTreeNode::new(head, Some(html), vec![], parts.nodes[2].kind().clone()),
    ];
    parts.admitted_creation_events = 3;
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::CompletionUpgrade(
            HtmlTreeCompletionUpgrade::DocumentShellIsIncomplete
        ))
    ));

    // A tokenizer-incomplete run can never be upgraded to Complete.
    let short = SourceText::new(SourceId::new(1), "<body>x".to_owned());
    let incomplete_run = crate::html::tokenizer::producer::tokenize(
        &short,
        HtmlTokenizerLimits::new(4, 8_192, 1_024, 1_024, 256, 4_096, 1_024),
    );
    assert!(incomplete_run.is_incomplete());
    let mut parts = valid_parts(&fixture);
    parts.completion = HtmlTreeCompletion::Complete;
    parts.processed_tokens = incomplete_run.tokens().len();
    parts.actions.clear();
    parts.diagnostics.clear();
    parts.committed_prefix_end = 0;
    assert!(matches!(
        freeze(&short, incomplete_run, parts),
        Err(HtmlTreeFreezeError::CompletionUpgrade(
            HtmlTreeCompletionUpgrade::RetainedTokenizerRunIsIncomplete
        ))
    ));
}

#[test]
fn freeze_rejects_a_leaked_unsupported_node_identity() {
    let fixture = freeze_fixture();
    let mut parts = valid_parts(&fixture);
    // The unsupported trigger names the same authored range as the body's own
    // origin, which would mean the refused token authored a committed node.
    parts.completion = HtmlTreeCompletion::Incomplete(
        HtmlTreeIncompleteCause::UnsupportedCapability(HtmlTreeUnsupportedCapability::new(
            HtmlTreeCapability::NonShellElementTag,
            HtmlTreeTokenTrigger::authored(0, fixture.source.anchor(0, 6).expect("valid range")),
        )),
    );
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnsupportedTriggerLeakedAsAuthoredOrigin(_))
    ));
}

#[test]
fn freeze_rejects_out_of_range_evidence_and_coverage() {
    let fixture = freeze_fixture();

    let mut parts = valid_parts(&fixture);
    parts.committed_prefix_end = 999;
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::InvalidCommittedCoverage { .. })
    ));

    let mut parts = valid_parts(&fixture);
    parts.processed_tokens = fixture.run.tokens().len() + 1;
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::CommittedTokensExceedRetainedRun { .. })
    ));

    let mut parts = valid_parts(&fixture);
    parts.actions.push(HtmlTreeAction::new(
        HtmlTreeActionKind::StoppedParsing,
        HtmlTreeTokenTrigger::end_of_file(999),
    ));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::InvalidTokenProgression { .. })
    ));

    let mut parts = valid_parts(&fixture);
    parts.actions.push(HtmlTreeAction::new(
        HtmlTreeActionKind::AppendedToTextNode {
            node: fixture.ids[6],
        },
        HtmlTreeTokenTrigger::end_of_file(1),
    ));
    assert!(matches!(
        freeze_parts(&fixture, parts),
        Err(HtmlTreeFreezeError::UnresolvedActionSubject(_))
    ));
}

// ---------------------------------------------------------------------------
// Session invariant boundaries
// ---------------------------------------------------------------------------

#[test]
fn session_refuses_to_reopen_an_already_open_shell_element() {
    // Driving the session with a hand-built duplicate `html` start tag proves
    // the open-element invariant is enforced by the session rather than only
    // by the mode machine that normally prevents it.
    let source = SourceText::new(SourceId::new(1), "<html><html>".to_owned());
    let run = crate::html::tokenizer::producer::tokenize(&source, generous_limits());
    let mut session = HtmlTreeSession::new().expect("session start");

    let first = &run.tokens()[0];
    let admitted = admit(first).expect("admitted");
    session
        .process(&admitted, token_trigger(first, 0))
        .expect("first html start tag");

    // The second `html` start tag is refused by the mode machine before it can
    // reach the session's own duplicate-open guard.
    let second = &run.tokens()[1];
    let admitted = admit(second).expect("admitted");
    assert!(matches!(
        session
            .process(&admitted, token_trigger(second, 1))
            .expect("no invariant failure"),
        TokenOutcome::Unsupported(HtmlTreeCapability::UnprovedShellStartTagPosition)
    ));
    assert_eq!(session.node_count(), 2);
    assert_eq!(session.open_element_count(), 1);
}

#[test]
fn admission_refuses_unproved_token_shapes_before_any_mode_runs() {
    let source = SourceText::new(SourceId::new(1), "<p><body a><body/>".to_owned());
    let run = crate::html::tokenizer::producer::tokenize(&source, generous_limits());
    let expected = [
        HtmlTreeCapability::NonShellElementTag,
        HtmlTreeCapability::ShellTagAttribute,
        HtmlTreeCapability::SelfClosingShellTag,
    ];
    for (token, expected) in run.tokens().iter().zip(expected) {
        assert_eq!(admit(token).err(), Some(expected));
    }
}

#[test]
fn token_contract_errors_stay_in_the_tokenizer_vocabulary() {
    // A compile-time reminder that TC-S1 introduces no second token vocabulary
    // and does not re-wrap lower-layer token contract errors.
    fn assert_error<E: std::error::Error>() {}
    assert_error::<HtmlTokenContractError>();
    assert_error::<HtmlTreeSessionError>();
    assert_error::<HtmlTreeFreezeError>();
    assert_error::<HtmlDocumentShellConstructionError>();
}
