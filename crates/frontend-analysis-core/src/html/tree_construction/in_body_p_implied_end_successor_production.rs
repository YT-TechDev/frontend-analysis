//! Production correspondence for TC-S6 — selected `div` / `section` end tags
//! over current P with bounded non-noop implied-end handling (Issue #371).
//!
//! Expectations here are independently authored from the accepted TC-S6
//! theorem. This module never imports or calls the candidate-independent TC-S6
//! validation machine.

use crate::{SourceId, SourceText};

use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;
use super::driver::{construct_html_document_shell, drive_token};
use super::result::{
    HtmlConstructedNodeId, HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlElement,
    HtmlSelectedOrdinaryElementName, HtmlTreeAction, HtmlTreeActionKind, HtmlTreeCapability,
    HtmlTreeCompletion, HtmlTreeDiagnosticCode, HtmlTreeIncompleteCause, HtmlTreeNodeKind,
    HtmlTreeTokenTrigger, freeze,
};
use super::session::{HtmlTreeSession, TokenOutcome, admit, token_trigger};

type Span = (usize, usize);

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn analyze_with(source: &str, source_id: u64) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(source_id), source.to_owned());
    construct_html_document_shell(&source, limits()).expect("TC-S6 production boundary")
}

fn analyze(source: &str) -> HtmlDocumentShellAnalysis {
    analyze_with(source, 1)
}

fn span(trigger: &HtmlTreeTokenTrigger) -> Option<Span> {
    trigger
        .authored_boundary()
        .map(|anchor| (anchor.range().start(), anchor.range().end()))
}

fn implied_pops(
    analysis: &HtmlDocumentShellAnalysis,
) -> Vec<(
    HtmlConstructedNodeId,
    HtmlConstructedNodeId,
    usize,
    Option<Span>,
)> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { node, target } => {
                Some((
                    *node,
                    *target,
                    action.trigger().token_index(),
                    span(action.trigger()),
                ))
            }
            _ => None,
        })
        .collect()
}

fn selected_recoveries(
    analysis: &HtmlDocumentShellAnalysis,
) -> Vec<(HtmlConstructedNodeId, HtmlConstructedNodeId, usize)> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { node, target } => {
                Some((*node, *target, action.trigger().token_index()))
            }
            _ => None,
        })
        .collect()
}

fn selected_closures(
    analysis: &HtmlDocumentShellAnalysis,
) -> Vec<(
    HtmlConstructedNodeId,
    HtmlSelectedOrdinaryElementName,
    usize,
)> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::ClosedSelectedOrdinaryElement { node, name } => {
                Some((*node, *name, action.trigger().token_index()))
            }
            _ => None,
        })
        .collect()
}

fn diagnostic_count(analysis: &HtmlDocumentShellAnalysis, code: HtmlTreeDiagnosticCode) -> usize {
    analysis
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code() == code)
        .count()
}

fn paragraph_ids(analysis: &HtmlDocumentShellAnalysis) -> Vec<HtmlConstructedNodeId> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(HtmlElement::Paragraph(_)) => Some(node.id()),
            _ => None,
        })
        .collect()
}

fn selected_ids(
    analysis: &HtmlDocumentShellAnalysis,
    name: HtmlSelectedOrdinaryElementName,
) -> Vec<HtmlConstructedNodeId> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(selected))
                if selected.name() == name =>
            {
                Some(node.id())
            }
            _ => None,
        })
        .collect()
}

fn creation_position(analysis: &HtmlDocumentShellAnalysis, id: HtmlConstructedNodeId) -> usize {
    analysis
        .nodes_in_creation_order()
        .iter()
        .position(|node| node.id() == id)
        .expect("constructed identity resolves in creation order")
}

fn normalized_creation_signature(
    analysis: &HtmlDocumentShellAnalysis,
) -> Vec<(String, Option<usize>)> {
    let nodes = analysis.nodes_in_creation_order();
    nodes
        .iter()
        .map(|node| {
            let meaning = match node.kind() {
                HtmlTreeNodeKind::Document => "document".to_owned(),
                HtmlTreeNodeKind::Element(element) => format!("element:{:?}", element.name()),
                HtmlTreeNodeKind::Text(text) => format!("text:{:?}", text.interpreted()),
            };
            let parent = node.parent().map(|id| {
                nodes
                    .iter()
                    .position(|candidate| candidate.id() == id)
                    .expect("parent resolves in creation order")
            });
            (meaning, parent)
        })
        .collect()
}

fn normalized_implied_pops(
    analysis: &HtmlDocumentShellAnalysis,
) -> Vec<(usize, usize, usize, Option<Span>)> {
    implied_pops(analysis)
        .into_iter()
        .map(|(node, target, token, trigger)| {
            (
                creation_position(analysis, node),
                creation_position(analysis, target),
                token,
                trigger,
            )
        })
        .collect()
}

fn normalized_recoveries(analysis: &HtmlDocumentShellAnalysis) -> Vec<(usize, usize, usize)> {
    selected_recoveries(analysis)
        .into_iter()
        .map(|(node, target, token)| {
            (
                creation_position(analysis, node),
                creation_position(analysis, target),
                token,
            )
        })
        .collect()
}

fn normalized_closures(
    analysis: &HtmlDocumentShellAnalysis,
) -> Vec<(usize, HtmlSelectedOrdinaryElementName, usize)> {
    selected_closures(analysis)
        .into_iter()
        .map(|(node, name, token)| (creation_position(analysis, node), name, token))
        .collect()
}

fn assert_unsupported_tc_s6(
    source: &str,
    capability: HtmlTreeCapability,
    refused_token: usize,
    refused_span: Span,
    committed_end: usize,
    processed_tokens: usize,
    node_count: usize,
) {
    let analysis = analyze(source);
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(unsupported)) =
        analysis.completion()
    else {
        panic!("{source:?}: expected explicit tree unsupported result")
    };
    assert_eq!(unsupported.capability(), capability, "{source:?}");
    assert_eq!(
        unsupported.trigger().token_index(),
        refused_token,
        "{source:?}"
    );
    assert_eq!(
        span(unsupported.trigger()),
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
    assert!(implied_pops(&analysis).is_empty(), "{source:?}");
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
        "{source:?}: refused token committed no diagnostic"
    );
}

fn text_parent(analysis: &HtmlDocumentShellAnalysis, interpreted: &str) -> HtmlConstructedNodeId {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .find_map(|node| match node.kind() {
            HtmlTreeNodeKind::Text(text) if text.interpreted() == interpreted => node.parent(),
            _ => None,
        })
        .expect("text parent")
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
        let admitted = admit(token).expect("fixture admitted");
        match drive_token(&mut session, &admitted, &trigger).expect("session invariant") {
            TokenOutcome::Consumed => {}
            TokenOutcome::StoppedParsing => {
                stopped = true;
                break;
            }
            TokenOutcome::Unsupported(capability) => {
                panic!("unexpected unsupported {capability:?}")
            }
        }
    }
    assert!(stopped);
    let parts = session.finish(HtmlTreeCompletion::Complete);
    (source, run, parts)
}

fn implied_action(
    parts: &HtmlDocumentShellParts,
) -> (
    usize,
    HtmlConstructedNodeId,
    HtmlConstructedNodeId,
    HtmlTreeTokenTrigger,
) {
    parts
        .actions
        .iter()
        .enumerate()
        .find_map(|(index, action)| match action.kind() {
            HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { node, target } => {
                Some((index, *node, *target, action.trigger().clone()))
            }
            _ => None,
        })
        .expect("implied P pop")
}

#[test]
fn production_correspondence_is_independent_of_candidate_validation() {
    let source = include_str!("in_body_p_implied_end_successor_production.rs");
    let forbidden = ["in_body_p_implied_end_successor_", "validation"].concat();
    assert!(!source.contains(&forbidden));
}

#[test]
fn direct_div_section_and_mixed_case_preserve_exact_same_trigger_evidence() {
    for (source, expected_range, expected_fragment, expected_name) in [
        (
            "<body><div><p>x</div>",
            (15, 21),
            "</div>",
            HtmlSelectedOrdinaryElementName::Div,
        ),
        (
            "<body><section><p>x</section>",
            (19, 29),
            "</section>",
            HtmlSelectedOrdinaryElementName::Section,
        ),
        (
            "<body><DiV><P>x</dIv>",
            (15, 21),
            "</dIv>",
            HtmlSelectedOrdinaryElementName::Div,
        ),
    ] {
        let analysis = analyze_with(source, 17);
        assert!(analysis.is_complete(), "{source}");
        let pops = implied_pops(&analysis);
        assert_eq!(pops.len(), 1, "{source}");
        assert_eq!(pops[0].3, Some(expected_range), "{source}");
        let action = analysis
            .actions()
            .iter()
            .find(|action| {
                matches!(
                    action.kind(),
                    HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
                )
            })
            .unwrap();
        let anchor = action.trigger().authored_boundary().unwrap();
        assert_eq!(anchor.source_id(), SourceId::new(17));
        assert_eq!(anchor.fragment(), expected_fragment);
        let closures = selected_closures(&analysis);
        assert!(
            closures
                .iter()
                .any(|(_, name, token)| *name == expected_name && *token == pops[0].2)
        );
        assert_eq!(
            diagnostic_count(
                &analysis,
                HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag
            ),
            0
        );
    }
}

#[test]
fn same_trigger_order_is_implied_p_then_current_first_recovery_then_target_close() {
    let analysis = analyze("<body><div><section><div><p>x</div>");
    assert!(analysis.is_complete());
    let pop = implied_pops(&analysis);
    assert_eq!(pop.len(), 1);
    let trigger = pop[0].2;
    let related: Vec<&HtmlTreeActionKind> = analysis
        .actions()
        .iter()
        .filter(|action| action.trigger().token_index() == trigger)
        .map(HtmlTreeAction::kind)
        .filter(|kind| {
            matches!(
                kind,
                HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
                    | HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
                    | HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
            )
        })
        .collect();
    assert!(matches!(
        related[0],
        HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
    ));
    assert!(matches!(
        related.last().unwrap(),
        HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
    ));

    let analysis = analyze("<body><div><section><div><p>x</section>");
    let pop = implied_pops(&analysis);
    assert_eq!(pop.len(), 1);
    let recoveries = selected_recoveries(&analysis);
    assert_eq!(recoveries.len(), 1);
    assert_eq!(recoveries[0].1, pop[0].1);
    assert_eq!(recoveries[0].2, pop[0].2);
    assert_eq!(
        diagnostic_count(
            &analysis,
            HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag
        ),
        1
    );
}

#[test]
fn absent_target_is_resolved_before_p_mutation_and_repeated_strays_leave_p_open() {
    let analysis = analyze("<body><div><p>x</section></section></p>");
    assert!(analysis.is_complete());
    assert!(implied_pops(&analysis).is_empty());
    assert_eq!(
        diagnostic_count(
            &analysis,
            HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag
        ),
        2
    );
    assert_eq!(
        diagnostic_count(&analysis, HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag),
        0
    );
    let p = paragraph_ids(&analysis);
    assert_eq!(
        p.len(),
        1,
        "later </p> closed the authored P rather than synthesizing another"
    );
}

#[test]
fn later_unmatched_p_synthesis_remains_distinct_after_implied_pop() {
    let analysis = analyze("<body><div><p>x</div></p>");
    assert!(analysis.is_complete());
    assert_eq!(implied_pops(&analysis).len(), 1);
    assert_eq!(
        diagnostic_count(&analysis, HtmlTreeDiagnosticCode::UnmatchedParagraphEndTag),
        1
    );
    assert_eq!(paragraph_ids(&analysis).len(), 2);
    assert!(analysis.actions().iter().any(|action| matches!(
        action.kind(),
        HtmlTreeActionKind::InsertedSynthesizedParagraphElement { .. }
    )));
}

#[test]
fn text_parentage_and_creation_inventory_show_pop_allocates_no_node() {
    let analysis = analyze("<body><div><p>x</div>y<section>z</section>");
    let p = paragraph_ids(&analysis)[0];
    let body = analysis
        .nodes_in_creation_order()
        .into_iter()
        .find(|node| matches!(node.kind(), HtmlTreeNodeKind::Element(HtmlElement::Shell(shell)) if shell.name() == super::result::HtmlShellElementName::Body))
        .unwrap()
        .id();
    assert_eq!(text_parent(&analysis, "x"), p);
    assert_eq!(text_parent(&analysis, "y"), body);
    assert_eq!(paragraph_ids(&analysis).len(), 1);
    assert_eq!(
        selected_ids(&analysis, HtmlSelectedOrdinaryElementName::Section).len(),
        1
    );
    assert_eq!(implied_pops(&analysis).len(), 1);
}

#[test]
fn predecessor_cells_without_selected_end_over_p_do_not_emit_tc_s6_relation() {
    for source in [
        "<body><div><section></div>",
        "<body><p>a<p>b</p>",
        "<body></p>",
    ] {
        let analysis = analyze(source);
        assert!(implied_pops(&analysis).is_empty(), "{source}");
    }
}

#[test]
fn implied_pop_trigger_is_causal_evidence_not_paragraph_origin_or_closure() {
    let analysis = analyze_with("<body><DiV><P>x</dIv>", 77);
    let p = paragraph_ids(&analysis)[0];
    let node = analysis.node(p).expect("Paragraph resolves");
    let HtmlTreeNodeKind::Element(HtmlElement::Paragraph(paragraph)) = node.kind() else {
        panic!("Paragraph node")
    };
    let super::result::HtmlParagraphElementOrigin::Authored { complete, raw_name } =
        paragraph.origin()
    else {
        panic!("authored Paragraph")
    };
    assert_eq!(complete.source_id(), SourceId::new(77));
    assert_eq!((complete.range().start(), complete.range().end()), (11, 14));
    assert_eq!((raw_name.range().start(), raw_name.range().end()), (12, 13));

    let pop = analysis
        .actions()
        .iter()
        .find(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { node, .. }
                    if *node == p
            )
        })
        .expect("TC-S6 implied pop");
    let trigger = pop
        .trigger()
        .authored_boundary()
        .expect("authored selected end");
    assert_eq!(trigger.source_id(), SourceId::new(77));
    assert_eq!((trigger.range().start(), trigger.range().end()), (15, 21));
    assert_ne!(complete.range(), trigger.range());
    assert!(
        analysis.actions().iter().all(|action| !matches!(
            action.kind(),
            HtmlTreeActionKind::ClosedParagraphElement { node, .. }
                if *node == p && action.trigger().token_index() == pop.trigger().token_index()
        )),
        "TC-S6 implied pop is not encoded as a Paragraph closure"
    );
}

#[test]
fn two_recovery_pops_are_current_first_and_share_the_implied_pop_target() {
    let analysis = analyze("<body><div><section><section><p>x</div>");
    assert!(analysis.is_complete());
    let div = selected_ids(&analysis, HtmlSelectedOrdinaryElementName::Div)[0];
    let sections = selected_ids(&analysis, HtmlSelectedOrdinaryElementName::Section);
    assert_eq!(sections.len(), 2);
    let pop = implied_pops(&analysis);
    assert_eq!(pop.len(), 1);
    assert_eq!(pop[0].1, div);

    let recoveries = selected_recoveries(&analysis);
    assert_eq!(recoveries.len(), 2);
    assert_eq!(
        recoveries[0].0, sections[1],
        "current selected element pops first"
    );
    assert_eq!(
        recoveries[1].0, sections[0],
        "next selected element pops second"
    );
    assert!(
        recoveries
            .iter()
            .all(|(_, target, token)| { *target == div && *token == pop[0].2 })
    );
    assert_eq!(
        selected_closures(&analysis),
        vec![(div, HtmlSelectedOrdinaryElementName::Div, pop[0].2)]
    );
    assert_eq!(
        diagnostic_count(
            &analysis,
            HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag
        ),
        1
    );
}

#[test]
fn implied_pop_and_unmatched_selected_ignore_allocate_no_creation_event() {
    let implied = analyze("<body><div><p>x</div><section></section>");
    let explicit = analyze("<body><div><p>x</p></div><section></section>");
    assert_eq!(
        normalized_creation_signature(&implied),
        normalized_creation_signature(&explicit),
        "replacing an implied P pop with an explicit P close changes no creation event"
    );
    let implied_section = selected_ids(&implied, HtmlSelectedOrdinaryElementName::Section)[0];
    let explicit_section = selected_ids(&explicit, HtmlSelectedOrdinaryElementName::Section)[0];
    assert_eq!(
        creation_position(&implied, implied_section),
        creation_position(&explicit, explicit_section)
    );

    let ignored = analyze("<body><div><p>x</section><section></section>");
    let control = analyze("<body><div><p>x<section></section>");
    assert_eq!(
        diagnostic_count(
            &ignored,
            HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag
        ),
        1
    );
    assert_eq!(
        normalized_creation_signature(&ignored),
        normalized_creation_signature(&control),
        "ignored selected end consumes no creation event"
    );
    let ignored_section = selected_ids(&ignored, HtmlSelectedOrdinaryElementName::Section)[0];
    let control_section = selected_ids(&control, HtmlSelectedOrdinaryElementName::Section)[0];
    assert_eq!(
        creation_position(&ignored, ignored_section),
        creation_position(&control, control_section)
    );
}

#[test]
fn semantic_relation_endpoints_and_order_are_source_id_independent() {
    let source = "<body><div><section><p>x</div>";
    let first = analyze_with(source, 41);
    let second = analyze_with(source, 99);
    assert_eq!(
        normalized_creation_signature(&first),
        normalized_creation_signature(&second)
    );
    assert_eq!(
        normalized_implied_pops(&first),
        normalized_implied_pops(&second)
    );
    assert_eq!(
        normalized_recoveries(&first),
        normalized_recoveries(&second)
    );
    assert_eq!(normalized_closures(&first), normalized_closures(&second));

    let first_trigger = first
        .actions()
        .iter()
        .find(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
            )
        })
        .unwrap()
        .trigger()
        .authored_boundary()
        .unwrap();
    let second_trigger = second
        .actions()
        .iter()
        .find(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
            )
        })
        .unwrap()
        .trigger()
        .authored_boundary()
        .unwrap();
    assert_eq!(first_trigger.source_id(), SourceId::new(41));
    assert_eq!(second_trigger.source_id(), SourceId::new(99));
}

#[test]
fn shape_phase_and_shell_crossing_refuse_before_tc_s6_mutation() {
    assert_unsupported_tc_s6(
        "<body><div><p>x</div id=x>",
        HtmlTreeCapability::SelectedOrdinaryTagAttribute,
        4,
        (15, 26),
        15,
        4,
        7,
    );
    assert_unsupported_tc_s6(
        "<body><div><p>x</div/>",
        HtmlTreeCapability::SelfClosingSelectedOrdinaryTag,
        4,
        (15, 22),
        15,
        4,
        7,
    );
    assert_unsupported_tc_s6(
        "<body></body></div>",
        HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
        2,
        (13, 19),
        13,
        2,
        4,
    );
    assert_unsupported_tc_s6(
        "<body><p></body>",
        HtmlTreeCapability::ShellTagWithOpenParagraphElement,
        2,
        (9, 16),
        9,
        2,
        5,
    );
}

#[test]
fn lower_layer_incompleteness_is_never_upgraded() {
    let source = SourceText::new(SourceId::new(1), "<body><div><p>x</div>".to_owned());
    let tiny = HtmlTokenizerLimits::new(4, 8_192, 1_024, 1_024, 256, 4_096, 1_024);
    let analysis = construct_html_document_shell(&source, tiny).expect("tree boundary");
    assert!(matches!(
        analysis.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));
    assert!(!analysis.is_complete());
}

#[test]
fn private_storage_reversal_preserves_tc_s6_meaning() {
    let (source, run, mut parts) = prepare_complete_parts("<body><div><section><p>x</div>");
    parts.nodes.reverse();
    let analysis = freeze(&source, run, parts).expect("identity-based freeze");
    assert_eq!(implied_pops(&analysis).len(), 1);
    assert_eq!(selected_recoveries(&analysis).len(), 1);
}

fn generated_blocks(
    depth: usize,
    current: &mut Vec<HtmlSelectedOrdinaryElementName>,
    out: &mut Vec<Vec<HtmlSelectedOrdinaryElementName>>,
) {
    out.push(current.clone());
    if depth == 0 {
        return;
    }
    for name in [
        HtmlSelectedOrdinaryElementName::Div,
        HtmlSelectedOrdinaryElementName::Section,
    ] {
        current.push(name);
        generated_blocks(depth - 1, current, out);
        current.pop();
    }
}

fn tag(name: HtmlSelectedOrdinaryElementName) -> &'static str {
    match name {
        HtmlSelectedOrdinaryElementName::Div => "div",
        HtmlSelectedOrdinaryElementName::Section => "section",
    }
}

#[test]
fn generated_bounded_stacks_match_independent_closed_form_counts() {
    let mut stacks = Vec::new();
    generated_blocks(4, &mut Vec::new(), &mut stacks);
    for blocks in stacks {
        for end in [
            HtmlSelectedOrdinaryElementName::Div,
            HtmlSelectedOrdinaryElementName::Section,
        ] {
            let mut source = String::from("<body>");
            for block in &blocks {
                source.push('<');
                source.push_str(tag(*block));
                source.push('>');
            }
            source.push_str("<p>x</");
            source.push_str(tag(end));
            source.push('>');
            let analysis = analyze(&source);
            let target = blocks.iter().rposition(|name| *name == end);
            match target {
                None => {
                    assert_eq!(implied_pops(&analysis).len(), 0, "{source}");
                    assert_eq!(selected_recoveries(&analysis).len(), 0, "{source}");
                    assert_eq!(
                        diagnostic_count(
                            &analysis,
                            HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag
                        ),
                        1,
                        "{source}"
                    );
                }
                Some(position) => {
                    let expected_recoveries = blocks.len() - position - 1;
                    assert_eq!(implied_pops(&analysis).len(), 1, "{source}");
                    assert_eq!(
                        selected_recoveries(&analysis).len(),
                        expected_recoveries,
                        "{source}"
                    );
                    assert_eq!(
                        diagnostic_count(
                            &analysis,
                            HtmlTreeDiagnosticCode::UnmatchedSelectedOrdinaryEndTag
                        ),
                        0,
                        "{source}"
                    );
                    assert_eq!(
                        diagnostic_count(
                            &analysis,
                            HtmlTreeDiagnosticCode::MisnestedSelectedOrdinaryEndTag
                        ),
                        usize::from(expected_recoveries > 0),
                        "{source}"
                    );
                }
            }
        }
    }
}

fn assert_freeze_rejects(source_text: &str, mutate: impl FnOnce(&mut HtmlDocumentShellParts)) {
    let (source, run, mut parts) = prepare_complete_parts(source_text);
    mutate(&mut parts);
    assert!(freeze(&source, run, parts).is_err());
}

#[test]
fn freeze_rejects_missing_duplicate_and_reordered_implied_pop_relation() {
    let source = "<body><div><p>x</div>";
    assert_freeze_rejects(source, |parts| {
        let (index, ..) = implied_action(parts);
        parts.actions.remove(index);
    });
    assert_freeze_rejects(source, |parts| {
        let (index, ..) = implied_action(parts);
        let duplicate = parts.actions[index].clone();
        parts.actions.insert(index + 1, duplicate);
    });
    assert_freeze_rejects(source, |parts| {
        let (index, ..) = implied_action(parts);
        parts.actions.swap(index, index + 1);
    });
}

#[test]
fn freeze_rejects_wrong_subject_target_trigger_and_final_open_state() {
    let source = "<body><div><section><p>x</div>";
    assert_freeze_rejects(source, |parts| {
        let (index, _node, target, trigger) = implied_action(parts);
        parts.actions[index] = HtmlTreeAction::new(
            HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag {
                node: target,
                target,
            },
            trigger,
        );
    });
    assert_freeze_rejects(source, |parts| {
        let (index, node, _target, trigger) = implied_action(parts);
        let wrong_target = parts
            .actions
            .iter()
            .find_map(|action| match action.kind() {
                HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                    node,
                    name: HtmlSelectedOrdinaryElementName::Section,
                } => Some(*node),
                _ => None,
            })
            .expect("section");
        parts.actions[index] = HtmlTreeAction::new(
            HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag {
                node,
                target: wrong_target,
            },
            trigger,
        );
    });
    assert_freeze_rejects(source, |parts| {
        let (index, node, target, _trigger) = implied_action(parts);
        let wrong_trigger = parts
            .actions
            .iter()
            .find(|action| {
                matches!(
                    action.kind(),
                    HtmlTreeActionKind::InsertedAuthoredParagraphElement { .. }
                )
            })
            .unwrap()
            .trigger()
            .clone();
        parts.actions[index] = HtmlTreeAction::new(
            HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { node, target },
            wrong_trigger,
        );
    });
    assert_freeze_rejects(source, |parts| {
        let (_, node, _, _) = implied_action(parts);
        parts.final_open_paragraph = Some(node);
    });
}

#[test]
fn freeze_rejects_non_nearest_closed_paragraph_and_target_absent_fabrications() {
    assert_freeze_rejects("<body><div><div><p>x</div>", |parts| {
        let (index, node, _target, trigger) = implied_action(parts);
        let divs: Vec<HtmlConstructedNodeId> = parts
            .actions
            .iter()
            .filter_map(|action| match action.kind() {
                HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                    node,
                    name: HtmlSelectedOrdinaryElementName::Div,
                } => Some(*node),
                _ => None,
            })
            .collect();
        parts.actions[index] = HtmlTreeAction::new(
            HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag {
                node,
                target: divs[0],
            },
            trigger,
        );
    });

    assert_freeze_rejects("<body><div><p></p></div>", |parts| {
        let p = parts
            .actions
            .iter()
            .find_map(|action| match action.kind() {
                HtmlTreeActionKind::InsertedAuthoredParagraphElement { node } => Some(*node),
                _ => None,
            })
            .unwrap();
        let (close_index, target, trigger) = parts
            .actions
            .iter()
            .enumerate()
            .find_map(|(index, action)| match action.kind() {
                HtmlTreeActionKind::ClosedSelectedOrdinaryElement {
                    node,
                    name: HtmlSelectedOrdinaryElementName::Div,
                } => Some((index, *node, action.trigger().clone())),
                _ => None,
            })
            .unwrap();
        parts.actions.insert(
            close_index,
            HtmlTreeAction::new(
                HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag {
                    node: p,
                    target,
                },
                trigger,
            ),
        );
    });

    assert_freeze_rejects("<body><div></div><p>x</div></p>", |parts| {
        let p = parts
            .actions
            .iter()
            .find_map(|action| match action.kind() {
                HtmlTreeActionKind::InsertedAuthoredParagraphElement { node } => Some(*node),
                _ => None,
            })
            .unwrap();
        let old_div = parts
            .actions
            .iter()
            .find_map(|action| match action.kind() {
                HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                    node,
                    name: HtmlSelectedOrdinaryElementName::Div,
                } => Some(*node),
                _ => None,
            })
            .unwrap();
        let (ignore_index, trigger) = parts
            .actions
            .iter()
            .enumerate()
            .find_map(|(index, action)| match action.kind() {
                HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag {
                    name: HtmlSelectedOrdinaryElementName::Div,
                } => Some((index, action.trigger().clone())),
                _ => None,
            })
            .unwrap();
        parts.actions.insert(
            ignore_index,
            HtmlTreeAction::new(
                HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag {
                    node: p,
                    target: old_div,
                },
                trigger,
            ),
        );
    });
}
