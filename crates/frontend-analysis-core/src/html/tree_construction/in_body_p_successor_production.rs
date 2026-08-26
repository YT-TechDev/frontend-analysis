//! Production correspondence for TC-S5 — Selected In-Body `p` Lifecycle with
//! Bounded Implicit Closure and Unmatched-End Synthesis (Issue #367).
//!
//! The expectations in this module are hand-authored from the accepted TC-S5
//! theorem. This module does not import, call, or project the candidate-
//! independent validation machine. Production result values are translated
//! only through small mechanical helpers, and all expected source spans,
//! action ordering, provenance, completion, and refusal values are written
//! explicitly below.

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;
use super::driver::{construct_html_document_shell, drive_token};
use super::result::{
    HtmlAuthoredSource, HtmlConstructedNodeId, HtmlDocumentShellAnalysis, HtmlDocumentShellParts,
    HtmlElement, HtmlParagraphClosure, HtmlParagraphElementOrigin, HtmlParagraphSynthesisCause,
    HtmlSelectedOrdinaryElementName, HtmlShellElementName, HtmlShellElementOrigin, HtmlTreeAction,
    HtmlTreeActionKind, HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeDiagnosticCode,
    HtmlTreeFreezeError, HtmlTreeIncompleteCause, HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery,
    HtmlTreeTokenTrigger, freeze,
};
use super::session::{HtmlTreeSession, TokenOutcome, admit, token_trigger};

type Span = (usize, usize);

#[derive(Debug, Clone, Copy)]
struct Fixture {
    id: &'static str,
    source: &'static str,
}

#[rustfmt::skip]
const FIXTURES: &[Fixture] = &[
    Fixture { id: "P1", source: "<body><p>x</p>" },
    Fixture { id: "P2", source: "<body><P>x</p>" },
    Fixture { id: "P3", source: "<body><p>a<p>b</p>" },
    Fixture { id: "P4", source: "<body><p>a<div>b</div>" },
    Fixture { id: "P5", source: "<body><p>a<section>b</section>" },
    Fixture { id: "P6", source: "<body><div><p>x</p></div>" },
    Fixture { id: "P7", source: "<body></p>" },
    Fixture { id: "P8", source: "<body><div></p>x</div>" },
    Fixture { id: "P9", source: "<body></p></p>" },
    Fixture { id: "P10", source: "<body><p>x" },
    Fixture { id: "P11", source: "<body><div><p>x" },
    Fixture { id: "P12", source: "<body><div><p></div>" },
    Fixture { id: "P13", source: "<body><section><p></section>" },
    Fixture { id: "P14", source: "<body><p id=x>" },
    Fixture { id: "P15", source: "<body><p/>" },
    Fixture { id: "P16", source: "<body></body><p>" },
    Fixture { id: "P17", source: "<body><p></body>" },
    Fixture { id: "P18", source: "<body><div>x</div><section>y</section>" },
    Fixture { id: "P19", source: "<body><div><section></div>" },
    Fixture { id: "P20", source: "<body><p>Z</p>" },
    Fixture { id: "P21", source: "<body><p></p id=x>" },
];

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .expect("canonical TC-S5 production fixture")
}

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn analyze_with(source: &str, source_id: u64) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(source_id), source.to_owned());
    construct_html_document_shell(&source, limits()).expect("TC-S5 production boundary")
}

fn analyze(source: &str) -> HtmlDocumentShellAnalysis {
    analyze_with(source, 1)
}

fn span(anchor: &SourceAnchor) -> Span {
    (anchor.range().start(), anchor.range().end())
}

fn trigger_span(trigger: &HtmlTreeTokenTrigger) -> Option<Span> {
    trigger.authored_boundary().map(span)
}

fn paragraph_nodes(analysis: &HtmlDocumentShellAnalysis) -> Vec<&HtmlTreeNode> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter(|node| {
            matches!(
                node.kind(),
                HtmlTreeNodeKind::Element(HtmlElement::Paragraph(_))
            )
        })
        .collect()
}

fn selected_nodes(
    analysis: &HtmlDocumentShellAnalysis,
    name: HtmlSelectedOrdinaryElementName,
) -> Vec<&HtmlTreeNode> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter(|node| matches!(
            node.kind(),
            HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) if selected.name() == name
        ))
        .collect()
}

fn text_nodes(analysis: &HtmlDocumentShellAnalysis) -> Vec<&HtmlTreeNode> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter(|node| matches!(node.kind(), HtmlTreeNodeKind::Text(_)))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParagraphEventKind {
    InsertedAuthored,
    InsertedSynthesized(HtmlParagraphSynthesisCause),
    Closed(HtmlParagraphClosure),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParagraphEvent {
    kind: ParagraphEventKind,
    node: HtmlConstructedNodeId,
    token_index: usize,
    trigger: Option<Span>,
}

fn paragraph_events(analysis: &HtmlDocumentShellAnalysis) -> Vec<ParagraphEvent> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredParagraphElement { node } => Some(ParagraphEvent {
                kind: ParagraphEventKind::InsertedAuthored,
                node: *node,
                token_index: action.trigger().token_index(),
                trigger: trigger_span(action.trigger()),
            }),
            HtmlTreeActionKind::InsertedSynthesizedParagraphElement { node, cause } => {
                Some(ParagraphEvent {
                    kind: ParagraphEventKind::InsertedSynthesized(*cause),
                    node: *node,
                    token_index: action.trigger().token_index(),
                    trigger: trigger_span(action.trigger()),
                })
            }
            HtmlTreeActionKind::ClosedParagraphElement { node, closure } => Some(ParagraphEvent {
                kind: ParagraphEventKind::Closed(*closure),
                node: *node,
                token_index: action.trigger().token_index(),
                trigger: trigger_span(action.trigger()),
            }),
            _ => None,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DiagnosticSignature {
    code: HtmlTreeDiagnosticCode,
    recovery: HtmlTreeRecovery,
    token_index: usize,
    trigger: Option<Span>,
}

fn diagnostic_signatures(analysis: &HtmlDocumentShellAnalysis) -> Vec<DiagnosticSignature> {
    analysis
        .diagnostics()
        .iter()
        .map(|diagnostic| DiagnosticSignature {
            code: diagnostic.code(),
            recovery: diagnostic.recovery(),
            token_index: diagnostic.trigger().token_index(),
            trigger: trigger_span(diagnostic.trigger()),
        })
        .collect()
}

fn missing_doctype() -> DiagnosticSignature {
    DiagnosticSignature {
        code: HtmlTreeDiagnosticCode::MissingDoctype,
        recovery: HtmlTreeRecovery::ContinuedInQuirksDocumentMode,
        token_index: 0,
        trigger: Some((0, 6)),
    }
}

fn paragraph_authored_origin(node: &HtmlTreeNode) -> (SourceId, Span, Span) {
    let HtmlTreeNodeKind::Element(HtmlElement::Paragraph(paragraph)) = node.kind() else {
        panic!("Paragraph node")
    };
    let HtmlParagraphElementOrigin::Authored { complete, raw_name } = paragraph.origin() else {
        panic!("authored Paragraph")
    };
    (complete.source_id(), span(complete), span(raw_name))
}

fn paragraph_is_synthesized(node: &HtmlTreeNode) -> bool {
    matches!(
        node.kind(),
        HtmlTreeNodeKind::Element(HtmlElement::Paragraph(paragraph))
            if matches!(
                paragraph.origin(),
                HtmlParagraphElementOrigin::Synthesized(
                    HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag
                )
            )
    )
}

fn assert_unsupported(
    source: &str,
    capability: HtmlTreeCapability,
    refused_token: usize,
    refused_span: Span,
    committed_end: usize,
    processed_tokens: usize,
    node_count: usize,
) -> HtmlDocumentShellAnalysis {
    let analysis = analyze(source);
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(unsupported)) =
        analysis.completion()
    else {
        panic!("{source:?}: expected an explicit tree unsupported result")
    };
    assert_eq!(unsupported.capability(), capability, "{source:?}");
    assert_eq!(
        unsupported.trigger().token_index(),
        refused_token,
        "{source:?}"
    );
    assert_eq!(
        trigger_span(unsupported.trigger()),
        Some(refused_span),
        "{source:?}"
    );
    assert_eq!(
        analysis.coverage().committed_end(),
        committed_end,
        "{source:?}"
    );
    assert_eq!(
        analysis.coverage().processed_tokens(),
        processed_tokens,
        "{source:?}"
    );
    assert_eq!(analysis.node_count(), node_count, "{source:?}");
    assert!(!analysis.is_complete(), "{source:?}");

    assert!(
        analysis
            .actions()
            .iter()
            .all(|action| action.trigger().token_index() < refused_token),
        "{source:?}: refused token committed no action"
    );
    assert!(
        analysis
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.trigger().token_index() < refused_token),
        "{source:?}: refused token committed no tree diagnostic"
    );
    assert!(
        analysis
            .nodes_in_creation_order()
            .into_iter()
            .all(|node| !matches!(
                node.authored_source(),
                Some(HtmlAuthoredSource::StartTag { complete, .. }) if span(complete) == refused_span
            )),
        "{source:?}: refused token is no node origin"
    );
    analysis
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeMeaning {
    Document,
    Shell(&'static str, Option<(Span, Span)>),
    Selected(&'static str, Span, Span),
    Paragraph(Option<(Span, Span)>),
    Text(String, Vec<Span>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeSignature {
    id: HtmlConstructedNodeId,
    parent: Option<HtmlConstructedNodeId>,
    children: Vec<HtmlConstructedNodeId>,
    meaning: NodeMeaning,
}

fn node_signatures(analysis: &HtmlDocumentShellAnalysis) -> Vec<NodeSignature> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .map(|node| {
            let meaning = match node.kind() {
                HtmlTreeNodeKind::Document => NodeMeaning::Document,
                HtmlTreeNodeKind::Element(HtmlElement::Shell(shell)) => {
                    let name = match shell.name() {
                        HtmlShellElementName::Html => "html",
                        HtmlShellElementName::Head => "head",
                        HtmlShellElementName::Body => "body",
                    };
                    let origin = match shell.origin() {
                        HtmlShellElementOrigin::Authored { complete, raw_name } => {
                            Some((span(complete), span(raw_name)))
                        }
                        HtmlShellElementOrigin::Synthesized(_) => None,
                    };
                    NodeMeaning::Shell(name, origin)
                }
                HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected)) => {
                    let name = match selected.name() {
                        HtmlSelectedOrdinaryElementName::Div => "div",
                        HtmlSelectedOrdinaryElementName::Section => "section",
                    };
                    NodeMeaning::Selected(
                        name,
                        span(selected.complete()),
                        span(selected.raw_name()),
                    )
                }
                HtmlTreeNodeKind::Element(HtmlElement::Paragraph(paragraph)) => {
                    let origin = match paragraph.origin() {
                        HtmlParagraphElementOrigin::Authored { complete, raw_name } => {
                            Some((span(complete), span(raw_name)))
                        }
                        HtmlParagraphElementOrigin::Synthesized(_) => None,
                    };
                    NodeMeaning::Paragraph(origin)
                }
                HtmlTreeNodeKind::Text(text) => NodeMeaning::Text(
                    text.interpreted().to_owned(),
                    text.contributions()
                        .iter()
                        .map(|contribution| span(contribution.source()))
                        .collect(),
                ),
            };
            NodeSignature {
                id: node.id(),
                parent: node.parent(),
                children: node.children().to_vec(),
                meaning,
            }
        })
        .collect()
}

fn prepare_complete_parts(
    source_text: &str,
) -> (SourceText, HtmlTokenizerRunResult, HtmlDocumentShellParts) {
    let source = SourceText::new(SourceId::new(1), source_text.to_owned());
    let run = tokenize(&source, limits());
    assert!(!run.is_incomplete(), "fixture tokenizer completes");
    let mut session = HtmlTreeSession::new().expect("session");
    let mut stopped = false;
    for (token_index, token) in run.tokens().iter().enumerate() {
        let trigger = token_trigger(token, token_index);
        let admitted = admit(token).expect("complete fixture is admitted");
        match drive_token(&mut session, &admitted, &trigger).expect("session invariant") {
            TokenOutcome::Consumed => {}
            TokenOutcome::StoppedParsing => {
                assert_eq!(token_index + 1, run.tokens().len());
                stopped = true;
                break;
            }
            TokenOutcome::Unsupported(capability) => {
                panic!("complete fixture unexpectedly unsupported: {capability:?}")
            }
        }
    }
    assert!(stopped, "complete fixture reaches supported EOF stop");
    let parts = session.finish(HtmlTreeCompletion::Complete);
    (source, run, parts)
}

#[test]
fn production_fixture_inventory_is_exact_and_independent() {
    assert_eq!(FIXTURES.len(), 21);
    for (index, fixture) in FIXTURES.iter().enumerate() {
        assert_eq!(fixture.id, format!("P{}", index + 1));
        assert!(fixture.source.starts_with("<body>"));
    }

    let source = include_str!("in_body_p_successor_production.rs");
    let forbidden = ["in_body_p_successor_", "validation"].concat();
    assert!(
        !source.contains(&forbidden),
        "production correspondence must not import the candidate-independent machine"
    );
}

#[test]
fn p1_authored_lifecycle_text_parentage_and_exact_matching_closure() {
    let analysis = analyze_with(fixture("P1").source, 11);
    assert!(analysis.is_complete());
    assert_eq!(
        diagnostic_signatures(&analysis),
        vec![DiagnosticSignature {
            code: HtmlTreeDiagnosticCode::MissingDoctype,
            recovery: HtmlTreeRecovery::ContinuedInQuirksDocumentMode,
            token_index: 0,
            trigger: Some((0, 6)),
        }]
    );

    let p = paragraph_nodes(&analysis)[0];
    assert_eq!(
        paragraph_authored_origin(p),
        (SourceId::new(11), (6, 9), (7, 8))
    );
    let text = text_nodes(&analysis)[0];
    assert_eq!(text.parent(), Some(p.id()));
    let HtmlTreeNodeKind::Text(text) = text.kind() else {
        unreachable!()
    };
    assert_eq!(text.interpreted(), "x");
    assert_eq!(
        text.contributions()
            .iter()
            .map(|contribution| span(contribution.source()))
            .collect::<Vec<_>>(),
        vec![(9, 10)]
    );

    assert_eq!(
        paragraph_events(&analysis),
        vec![
            ParagraphEvent {
                kind: ParagraphEventKind::InsertedAuthored,
                node: p.id(),
                token_index: 1,
                trigger: Some((6, 9)),
            },
            ParagraphEvent {
                kind: ParagraphEventKind::Closed(HtmlParagraphClosure::MatchingEndTag),
                node: p.id(),
                token_index: 3,
                trigger: Some((10, 14)),
            },
        ]
    );
}

#[test]
fn p2_case_insensitive_name_keeps_exact_raw_spelling() {
    let analysis = analyze_with(fixture("P2").source, 12);
    assert!(analysis.is_complete());
    let p = paragraph_nodes(&analysis)[0];
    let HtmlTreeNodeKind::Element(HtmlElement::Paragraph(paragraph)) = p.kind() else {
        unreachable!()
    };
    let HtmlParagraphElementOrigin::Authored { complete, raw_name } = paragraph.origin() else {
        panic!("authored P")
    };
    assert_eq!(complete.source_id(), SourceId::new(12));
    assert_eq!(span(complete), (6, 9));
    assert_eq!(span(raw_name), (7, 8));
    assert_eq!(raw_name.fragment(), "P");
}

#[test]
fn p3_second_start_closes_first_before_second_identity_is_inserted() {
    let analysis = analyze(fixture("P3").source);
    assert!(analysis.is_complete());
    let ps = paragraph_nodes(&analysis);
    assert_eq!(ps.len(), 2);
    assert_ne!(ps[0].id(), ps[1].id());
    assert_eq!(ps[0].parent(), ps[1].parent());
    assert_eq!(
        paragraph_events(&analysis),
        vec![
            ParagraphEvent {
                kind: ParagraphEventKind::InsertedAuthored,
                node: ps[0].id(),
                token_index: 1,
                trigger: Some((6, 9)),
            },
            ParagraphEvent {
                kind: ParagraphEventKind::Closed(HtmlParagraphClosure::StartTriggered),
                node: ps[0].id(),
                token_index: 3,
                trigger: Some((10, 13)),
            },
            ParagraphEvent {
                kind: ParagraphEventKind::InsertedAuthored,
                node: ps[1].id(),
                token_index: 3,
                trigger: Some((10, 13)),
            },
            ParagraphEvent {
                kind: ParagraphEventKind::Closed(HtmlParagraphClosure::MatchingEndTag),
                node: ps[1].id(),
                token_index: 5,
                trigger: Some((14, 18)),
            },
        ]
    );
}

#[test]
fn p4_p5_block_starts_close_p_before_predecessor_insertion() {
    for (id, block, trigger) in [
        ("P4", HtmlSelectedOrdinaryElementName::Div, (10, 15)),
        ("P5", HtmlSelectedOrdinaryElementName::Section, (10, 19)),
    ] {
        let analysis = analyze(fixture(id).source);
        assert!(analysis.is_complete(), "{id}");
        assert_eq!(
            diagnostic_signatures(&analysis),
            vec![missing_doctype()],
            "{id}"
        );
        let p = paragraph_nodes(&analysis)[0];
        let close_index = analysis
            .actions()
            .iter()
            .position(|action| {
                matches!(
                    action.kind(),
                    HtmlTreeActionKind::ClosedParagraphElement {
                        node,
                        closure: HtmlParagraphClosure::StartTriggered,
                    } if *node == p.id()
                )
            })
            .expect("start-triggered P closure");
        let insert_index = analysis
            .actions()
            .iter()
            .position(|action| {
                matches!(
                    action.kind(),
                    HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { name, .. }
                        if *name == block
                )
            })
            .expect("block insertion");
        assert!(
            close_index < insert_index,
            "{id}: P closes before block insertion"
        );
        assert_eq!(
            trigger_span(analysis.actions()[close_index].trigger()),
            Some(trigger)
        );
        assert_eq!(
            trigger_span(analysis.actions()[insert_index].trigger()),
            Some(trigger)
        );
        assert!(analysis.actions().iter().all(|action| !matches!(
            action.kind(),
            HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
        )));
    }
}

#[test]
fn p6_matching_p_end_keeps_parent_block_separate() {
    let analysis = analyze(fixture("P6").source);
    assert!(analysis.is_complete());
    assert_eq!(diagnostic_signatures(&analysis), vec![missing_doctype()]);
    let div = selected_nodes(&analysis, HtmlSelectedOrdinaryElementName::Div)[0];
    let p = paragraph_nodes(&analysis)[0];
    assert_eq!(p.parent(), Some(div.id()));
    assert_eq!(
        paragraph_events(&analysis)
            .iter()
            .filter(|event| matches!(event.kind, ParagraphEventKind::Closed(_)))
            .copied()
            .collect::<Vec<_>>(),
        vec![ParagraphEvent {
            kind: ParagraphEventKind::Closed(HtmlParagraphClosure::MatchingEndTag),
            node: p.id(),
            token_index: 4,
            trigger: Some((15, 19)),
        }]
    );
    assert!(analysis.actions().iter().all(|action| !matches!(
        action.kind(),
        HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
    )));
}

#[test]
fn p7_unmatched_end_has_one_extra_diagnostic_source_less_synthesis_and_close() {
    let analysis = analyze_with(fixture("P7").source, 21);
    assert!(analysis.is_complete());
    assert_eq!(
        diagnostic_signatures(&analysis),
        vec![
            DiagnosticSignature {
                code: HtmlTreeDiagnosticCode::MissingDoctype,
                recovery: HtmlTreeRecovery::ContinuedInQuirksDocumentMode,
                token_index: 0,
                trigger: Some((0, 6)),
            },
            DiagnosticSignature {
                code: HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag,
                recovery: HtmlTreeRecovery::SynthesizedParagraphElementAndClosedIt,
                token_index: 1,
                trigger: Some((6, 10)),
            },
        ]
    );
    let p = paragraph_nodes(&analysis)[0];
    assert!(paragraph_is_synthesized(p));
    assert!(p.authored_source().is_none());
    assert_eq!(
        paragraph_events(&analysis),
        vec![
            ParagraphEvent {
                kind: ParagraphEventKind::InsertedSynthesized(
                    HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag,
                ),
                node: p.id(),
                token_index: 1,
                trigger: Some((6, 10)),
            },
            ParagraphEvent {
                kind: ParagraphEventKind::Closed(HtmlParagraphClosure::UnmatchedEndTagSynthesized,),
                node: p.id(),
                token_index: 1,
                trigger: Some((6, 10)),
            },
        ]
    );
}

#[test]
fn p8_synthesized_p_uses_actual_block_parent_and_following_text_returns_to_block() {
    let analysis = analyze(fixture("P8").source);
    assert!(analysis.is_complete());
    let div = selected_nodes(&analysis, HtmlSelectedOrdinaryElementName::Div)[0];
    let p = paragraph_nodes(&analysis)[0];
    assert!(paragraph_is_synthesized(p));
    assert_eq!(p.parent(), Some(div.id()));
    let text = text_nodes(&analysis)[0];
    assert_eq!(text.parent(), Some(div.id()));
    let HtmlTreeNodeKind::Text(text) = text.kind() else {
        unreachable!()
    };
    assert_eq!(text.interpreted(), "x");
    assert_eq!(
        text.contributions()
            .iter()
            .map(|contribution| span(contribution.source()))
            .collect::<Vec<_>>(),
        vec![(15, 16)]
    );
}

#[test]
fn p9_repeated_stray_ends_create_two_distinct_synthesis_closure_groups() {
    let analysis = analyze_with(fixture("P9").source, 31);
    assert!(analysis.is_complete());
    let ps = paragraph_nodes(&analysis);
    assert_eq!(ps.len(), 2);
    assert_ne!(ps[0].id(), ps[1].id());
    assert!(ps.iter().all(|p| paragraph_is_synthesized(p)));
    assert_eq!(
        diagnostic_signatures(&analysis)
            .into_iter()
            .filter(|diagnostic| diagnostic.code == HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag)
            .collect::<Vec<_>>(),
        vec![
            DiagnosticSignature {
                code: HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag,
                recovery: HtmlTreeRecovery::SynthesizedParagraphElementAndClosedIt,
                token_index: 1,
                trigger: Some((6, 10)),
            },
            DiagnosticSignature {
                code: HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag,
                recovery: HtmlTreeRecovery::SynthesizedParagraphElementAndClosedIt,
                token_index: 2,
                trigger: Some((10, 14)),
            },
        ]
    );
    assert_eq!(
        paragraph_events(&analysis),
        vec![
            ParagraphEvent {
                kind: ParagraphEventKind::InsertedSynthesized(
                    HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag,
                ),
                node: ps[0].id(),
                token_index: 1,
                trigger: Some((6, 10)),
            },
            ParagraphEvent {
                kind: ParagraphEventKind::Closed(HtmlParagraphClosure::UnmatchedEndTagSynthesized,),
                node: ps[0].id(),
                token_index: 1,
                trigger: Some((6, 10)),
            },
            ParagraphEvent {
                kind: ParagraphEventKind::InsertedSynthesized(
                    HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag,
                ),
                node: ps[1].id(),
                token_index: 2,
                trigger: Some((10, 14)),
            },
            ParagraphEvent {
                kind: ParagraphEventKind::Closed(HtmlParagraphClosure::UnmatchedEndTagSynthesized,),
                node: ps[1].id(),
                token_index: 2,
                trigger: Some((10, 14)),
            },
        ]
    );
}

#[test]
fn p10_p_only_eof_is_complete_open_and_has_no_p_specific_eof_event() {
    let analysis = analyze(fixture("P10").source);
    assert!(analysis.is_complete());
    assert_eq!(diagnostic_signatures(&analysis), vec![missing_doctype()]);
    let p = paragraph_nodes(&analysis)[0];
    assert_eq!(paragraph_authored_origin(p).1, (6, 9));
    assert_eq!(
        paragraph_events(&analysis),
        vec![ParagraphEvent {
            kind: ParagraphEventKind::InsertedAuthored,
            node: p.id(),
            token_index: 1,
            trigger: Some((6, 9)),
        }]
    );
    assert_eq!(
        analysis.coverage().committed_end(),
        fixture("P10").source.len()
    );
}

#[test]
fn p11_open_block_eof_keeps_only_the_predecessor_block_diagnostic() {
    let analysis = analyze(fixture("P11").source);
    assert!(analysis.is_complete());
    let diagnostics = diagnostic_signatures(&analysis);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0], missing_doctype());
    assert_eq!(
        diagnostics[1].code,
        HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile
    );
    assert_eq!(
        diagnostics[1].recovery,
        HtmlTreeRecovery::StoppedParsingWithOpenSelectedOrdinaryElements
    );
    assert!(
        diagnostics[1].trigger.is_none(),
        "EOF carries no fabricated authored range"
    );
    assert_eq!(paragraph_nodes(&analysis).len(), 1);
    assert!(
        paragraph_events(&analysis)
            .iter()
            .all(|event| !matches!(event.kind, ParagraphEventKind::Closed(_)))
    );
}

#[test]
fn p12_p13_block_end_over_open_p_advance_to_tc_s6_support() {
    for id in ["P12", "P13"] {
        let analysis = analyze(fixture(id).source);
        assert!(analysis.is_complete(), "{id}");
        assert!(matches!(
            analysis.completion(),
            HtmlTreeCompletion::Complete
        ));
        assert!(
            analysis.actions().iter().any(|action| matches!(
                action.kind(),
                HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
            )),
            "{id}"
        );
    }
}

#[test]
fn p14_p15_p16_p21_shape_and_crossing_refusals_are_transactional() {
    assert_unsupported(
        fixture("P14").source,
        HtmlTreeCapability::ParagraphTagAttribute,
        1,
        (6, 14),
        6,
        1,
        4,
    );
    assert_unsupported(
        fixture("P15").source,
        HtmlTreeCapability::SelfClosingParagraphTag,
        1,
        (6, 10),
        6,
        1,
        4,
    );
    assert_unsupported(
        fixture("P16").source,
        HtmlTreeCapability::ParagraphTagOutsideInBody,
        2,
        (13, 16),
        13,
        2,
        4,
    );
    assert_unsupported(
        fixture("P21").source,
        HtmlTreeCapability::ParagraphTagAttribute,
        2,
        (9, 18),
        9,
        2,
        5,
    );
}

#[test]
fn p17_body_end_preserves_the_current_p_without_a_selected_open_diagnostic() {
    let analysis = analyze(fixture("P17").source);
    assert!(analysis.is_complete());
    assert_eq!(analysis.node_count(), 5);
    assert_eq!(analysis.coverage().committed_end(), 16);
    assert_eq!(analysis.coverage().processed_tokens(), 4);
    assert_eq!(
        diagnostic_count(
            &analysis,
            HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements,
        ),
        0,
        "P alone is allowed by the bounded body-end stack check"
    );
    assert!(analysis.actions().iter().any(|action| matches!(
        action.kind(),
        HtmlTreeActionKind::AcknowledgedShellEndTag {
            name: HtmlShellElementName::Body,
        } if action.trigger().token_index() == 2
    )));
    assert!(analysis.actions().iter().all(|action| {
        action.trigger().token_index() != 2
            || !matches!(
                action.kind(),
                HtmlTreeActionKind::ClosedParagraphElement { .. }
                    | HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
            )
    }));
}

#[test]
fn p18_predecessor_div_section_lifecycle_has_exact_zero_p_delta() {
    let analysis = analyze(fixture("P18").source);
    assert!(analysis.is_complete());
    assert_eq!(diagnostic_signatures(&analysis), vec![missing_doctype()]);
    assert!(paragraph_nodes(&analysis).is_empty());
    assert!(paragraph_events(&analysis).is_empty());
    assert_eq!(
        selected_nodes(&analysis, HtmlSelectedOrdinaryElementName::Div).len(),
        1
    );
    assert_eq!(
        selected_nodes(&analysis, HtmlSelectedOrdinaryElementName::Section).len(),
        1
    );
    assert!(analysis.actions().iter().all(|action| !matches!(
        action.kind(),
        HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
    )));
}

#[test]
fn p19_tc_s4_heterogeneous_recovery_remains_separate_from_p() {
    let analysis = analyze(fixture("P19").source);
    assert!(analysis.is_complete());
    assert!(paragraph_nodes(&analysis).is_empty());
    assert!(paragraph_events(&analysis).is_empty());
    let diagnostics = diagnostic_signatures(&analysis);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0], missing_doctype());
    assert_eq!(
        diagnostics[1].code,
        HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag
    );
    assert_eq!(
        diagnostics[1].recovery,
        HtmlTreeRecovery::PoppedInterveningSelectedOrdinaryElementsAndClosedTarget
    );
    assert_eq!(diagnostics[1].token_index, 3);
    assert_eq!(diagnostics[1].trigger, Some((20, 26)));

    let recovery = analysis
        .actions()
        .iter()
        .find(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
            )
        })
        .expect("TC-S4 recovery action");
    assert_eq!(recovery.trigger().token_index(), 3);
    assert_eq!(trigger_span(recovery.trigger()), Some((20, 26)));
}

#[test]
fn p20_source_id_changes_provenance_not_semantic_creation_identity() {
    let first = analyze_with(fixture("P20").source, 7);
    let second = analyze_with(fixture("P20").source, 9_999);
    assert!(first.is_complete());
    assert!(second.is_complete());
    let a = paragraph_nodes(&first)[0];
    let b = paragraph_nodes(&second)[0];
    assert_eq!(a.id(), b.id());
    let (a_source, a_complete, a_raw) = paragraph_authored_origin(a);
    let (b_source, b_complete, b_raw) = paragraph_authored_origin(b);
    assert_ne!(a_source, b_source);
    assert_eq!(a_complete, b_complete);
    assert_eq!(a_raw, b_raw);
    assert_eq!(a_complete, (6, 9));
    assert_eq!(a_raw, (7, 8));
}

#[test]
fn semantic_meaning_survives_private_storage_reversal() {
    for id in ["P3", "P7", "P8", "P9", "P19"] {
        let analysis = analyze(fixture(id).source);
        let expected_nodes = node_signatures(&analysis);
        let expected_events = paragraph_events(&analysis);
        let expected_diagnostics = diagnostic_signatures(&analysis);
        let expected_coverage = (
            analysis.coverage().committed_end(),
            analysis.coverage().processed_tokens(),
        );
        let reversed = analysis.clone().with_reversed_storage();
        assert_eq!(node_signatures(&reversed), expected_nodes, "{id}");
        assert_eq!(paragraph_events(&reversed), expected_events, "{id}");
        assert_eq!(
            diagnostic_signatures(&reversed),
            expected_diagnostics,
            "{id}"
        );
        assert_eq!(
            (
                reversed.coverage().committed_end(),
                reversed.coverage().processed_tokens(),
            ),
            expected_coverage,
            "{id}"
        );
    }
}

#[test]
fn lower_layer_incompleteness_is_never_upgraded() {
    let source = SourceText::new(SourceId::new(1), "<body><p>xxxxxxxx".to_owned());
    let result =
        construct_html_document_shell(&source, HtmlTokenizerLimits::new(1, 1, 1, 1, 1, 1, 1))
            .expect("lower-layer incomplete remains an ordinary analysis result");
    assert!(result.tokenizer_run().is_incomplete());
    assert!(!result.is_complete());
    assert!(matches!(
        result.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));
}

#[test]
fn freeze_rejects_paragraph_final_open_corruption() {
    let (source, run, mut parts) = prepare_complete_parts(fixture("P10").source);
    assert!(parts.final_open_paragraph.is_some());
    parts.final_open_paragraph = None;
    assert!(matches!(
        freeze(&source, run, parts),
        Err(HtmlTreeFreezeError::FinalOpenParagraphStateMismatch { .. })
    ));

    let (source, run, mut parts) = prepare_complete_parts(fixture("P1").source);
    let body = parts
        .nodes
        .iter()
        .find(|node| {
            matches!(
                node.kind(),
                HtmlTreeNodeKind::Element(HtmlElement::Shell(shell))
                    if shell.name() == HtmlShellElementName::Body
            )
        })
        .expect("body")
        .id();
    parts.final_open_paragraph = Some(body);
    assert_eq!(
        freeze(&source, run, parts).expect_err("non-P final-open subject"),
        HtmlTreeFreezeError::FinalOpenParagraphIsNotParagraph(body)
    );
}

#[test]
fn freeze_rejects_unmatched_p_diagnostic_and_synthesis_closure_corruption() {
    let (source, run, mut parts) = prepare_complete_parts(fixture("P7").source);
    parts
        .diagnostics
        .retain(|diagnostic| diagnostic.code() != HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag);
    assert!(matches!(
        freeze(&source, run, parts),
        Err(HtmlTreeFreezeError::UnmatchedParagraphDiagnosticMismatch { .. })
    ));

    let (source, run, mut parts) = prepare_complete_parts(fixture("P7").source);
    parts.actions.retain(|action| {
        !matches!(
            action.kind(),
            HtmlTreeActionKind::ClosedParagraphElement {
                closure: HtmlParagraphClosure::UnmatchedEndTagSynthesized,
                ..
            }
        )
    });
    assert!(matches!(
        freeze(&source, run, parts),
        Err(HtmlTreeFreezeError::ParagraphSynthesisClosureMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_duplicate_p_insertion_and_wrong_matching_close_trigger() {
    let (source, run, mut parts) = prepare_complete_parts(fixture("P1").source);
    let insertion_index = parts
        .actions
        .iter()
        .position(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::InsertedAuthoredParagraphElement { .. }
            )
        })
        .expect("P insertion");
    let duplicate = parts.actions[insertion_index].clone();
    parts.actions.insert(insertion_index + 1, duplicate);
    assert!(matches!(
        freeze(&source, run, parts),
        Err(HtmlTreeFreezeError::DuplicateParagraphInsertion(_))
    ));

    let (source, run, mut parts) = prepare_complete_parts(fixture("P1").source);
    let closure_index = parts
        .actions
        .iter()
        .position(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::ClosedParagraphElement {
                    closure: HtmlParagraphClosure::MatchingEndTag,
                    ..
                }
            )
        })
        .expect("matching P closure");
    let node = match parts.actions[closure_index].kind() {
        HtmlTreeActionKind::ClosedParagraphElement { node, .. } => *node,
        _ => unreachable!(),
    };
    parts.actions[closure_index] = HtmlTreeAction::new(
        HtmlTreeActionKind::ClosedParagraphElement {
            node,
            closure: HtmlParagraphClosure::MatchingEndTag,
        },
        token_trigger(&run.tokens()[2], 2),
    );
    assert!(matches!(
        freeze(&source, run, parts),
        Err(HtmlTreeFreezeError::ParagraphClosureTriggerMismatch {
            node: rejected,
            token_index: 2,
        }) if rejected == node
    ));
}
