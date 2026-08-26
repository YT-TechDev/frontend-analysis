//! Production correspondence for TC-S7 — selected InBody `</body>` over the
//! open bounded `Div | Section` stack with optional current P (Issue #378).
//!
//! Every expected value here is independently authored from the accepted
//! production theorem. This module imports only production tokenizer,
//! driver/session, result, and freeze seams. It never imports, calls, projects,
//! or generates expectations from the candidate-independent validation
//! machine.

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;
use super::driver::{construct_html_document_shell, drive_token};
use super::result::{
    HtmlConstructedNodeId, HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlElementName,
    HtmlParagraphClosure, HtmlParagraphSynthesisCause,
    HtmlSelectedOrdinaryElementName, HtmlShellElementName, HtmlTreeAction, HtmlTreeActionKind,
    HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeDiagnostic, HtmlTreeDiagnosticCode,
    HtmlTreeFreezeError, HtmlTreeIncompleteCause, HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery,
    HtmlTreeTokenTrigger, freeze,
};
use super::session::{
    HtmlTreeSession, InsertionMode, TokenOutcome, admit, token_trigger,
};

type Span = (usize, usize);
type DiagnosticEvidence = (usize, SourceId, Span, HtmlTreeRecovery);

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn analyze(source: &str) -> HtmlDocumentShellAnalysis {
    analyze_with(source, 1)
}

fn analyze_with(source: &str, source_id: u64) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(source_id), source.to_owned());
    construct_html_document_shell(&source, limits()).expect("TC-S7 production boundary")
}

fn span(anchor: &SourceAnchor) -> Span {
    (anchor.range().start(), anchor.range().end())
}

fn drive_session(source_text: &str) -> (HtmlTreeSession, Vec<TokenOutcome>) {
    let source = SourceText::new(SourceId::new(1), source_text.to_owned());
    let run = tokenize(&source, limits());
    let mut session = HtmlTreeSession::new().expect("session start");
    let mut outcomes = Vec::new();
    for (token_index, token) in run.tokens().iter().enumerate() {
        let trigger = token_trigger(token, token_index);
        let admitted = match admit(token) {
            Ok(admitted) => admitted,
            Err(capability) => {
                outcomes.push(TokenOutcome::Unsupported(capability));
                break;
            }
        };
        let outcome = drive_token(&mut session, &admitted, &trigger)
            .expect("production dispatch invariant");
        outcomes.push(outcome);
        if !matches!(outcome, TokenOutcome::Consumed) {
            break;
        }
    }
    (session, outcomes)
}

fn diagnostic_evidence(
    analysis: &HtmlDocumentShellAnalysis,
    code: HtmlTreeDiagnosticCode,
) -> Vec<DiagnosticEvidence> {
    analysis
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code() == code)
        .map(|diagnostic| {
            let anchor = diagnostic
                .trigger()
                .authored_boundary()
                .expect("selected diagnostic is authored");
            (
                diagnostic.trigger().token_index(),
                anchor.source_id(),
                span(anchor),
                diagnostic.recovery(),
            )
        })
        .collect()
}

fn diagnostic_count(
    analysis: &HtmlDocumentShellAnalysis,
    code: HtmlTreeDiagnosticCode,
) -> usize {
    analysis
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code() == code)
        .count()
}

fn body_acknowledgements(
    analysis: &HtmlDocumentShellAnalysis,
) -> Vec<(usize, SourceId, Span)> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::AcknowledgedShellEndTag {
                name: HtmlShellElementName::Body,
            } => {
                let anchor = action
                    .trigger()
                    .authored_boundary()
                    .expect("body acknowledgement is authored");
                Some((
                    action.trigger().token_index(),
                    anchor.source_id(),
                    span(anchor),
                ))
            }
            _ => None,
        })
        .collect()
}

fn reprocess_count(analysis: &HtmlDocumentShellAnalysis) -> usize {
    analysis
        .actions()
        .iter()
        .filter(|action| matches!(action.kind(), HtmlTreeActionKind::ReprocessedToken))
        .count()
}

fn selected_insertions(
    analysis: &HtmlDocumentShellAnalysis,
) -> Vec<(HtmlConstructedNodeId, HtmlSelectedOrdinaryElementName, usize)> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { node, name } => {
                Some((*node, *name, action.trigger().token_index()))
            }
            _ => None,
        })
        .collect()
}

fn paragraph_insertions(
    analysis: &HtmlDocumentShellAnalysis,
) -> Vec<(HtmlConstructedNodeId, usize)> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredParagraphElement { node } => {
                Some((*node, action.trigger().token_index()))
            }
            _ => None,
        })
        .collect()
}

fn text_nodes(analysis: &HtmlDocumentShellAnalysis) -> Vec<&HtmlTreeNode> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter(|node| matches!(node.kind(), HtmlTreeNodeKind::Text(_)))
        .collect()
}

fn parent_name(
    analysis: &HtmlDocumentShellAnalysis,
    node: &HtmlTreeNode,
) -> HtmlElementName {
    let parent = analysis
        .node(node.parent().expect("text parent identity"))
        .expect("text parent resolves");
    let HtmlTreeNodeKind::Element(element) = parent.kind() else {
        panic!("text parent is an element")
    };
    element.name()
}

fn unsupported_evidence(
    analysis: &HtmlDocumentShellAnalysis,
) -> Option<(HtmlTreeCapability, usize, Option<Span>)> {
    match analysis.completion() {
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => Some((
            unsupported.capability(),
            unsupported.trigger().token_index(),
            unsupported.trigger().authored_boundary().map(span),
        )),
        HtmlTreeCompletion::Complete
        | HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete) => None,
    }
}

fn forbidden_body_end_actions(analysis: &HtmlDocumentShellAnalysis, token_index: usize) -> usize {
    analysis
        .actions()
        .iter()
        .filter(|action| {
            action.trigger().token_index() == token_index
                && matches!(
                    action.kind(),
                    HtmlTreeActionKind::InsertedAuthoredShellElement { .. }
                        | HtmlTreeActionKind::InsertedSynthesizedShellElement { .. }
                        | HtmlTreeActionKind::InsertedTextNode { .. }
                        | HtmlTreeActionKind::AppendedToTextNode { .. }
                        | HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { .. }
                        | HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
                        | HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
                        | HtmlTreeActionKind::InsertedAuthoredParagraphElement { .. }
                        | HtmlTreeActionKind::InsertedSynthesizedParagraphElement { .. }
                        | HtmlTreeActionKind::ClosedParagraphElement { .. }
                        | HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
                )
        })
        .count()
}

fn semantic_signature(analysis: &HtmlDocumentShellAnalysis) -> Vec<String> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .map(|node| {
            let kind = match node.kind() {
                HtmlTreeNodeKind::Document => "Document".to_owned(),
                HtmlTreeNodeKind::Element(element) => format!("Element({:?})", element.name()),
                HtmlTreeNodeKind::Text(text) => format!(
                    "Text({:?};{:?})",
                    text.interpreted(),
                    text.contributions()
                        .iter()
                        .map(|contribution| span(contribution.source()))
                        .collect::<Vec<_>>()
                ),
            };
            format!(
                "{:?}|{:?}|{:?}|{kind}",
                node.id(),
                node.parent(),
                node.children()
            )
        })
        .collect()
}

struct FreezeFixture {
    source: SourceText,
    run: HtmlTokenizerRunResult,
}

impl FreezeFixture {
    fn new(source: &str) -> Self {
        let source = SourceText::new(SourceId::new(73), source.to_owned());
        let run = tokenize(&source, limits());
        Self { source, run }
    }

    fn anchor(&self, start: usize, end: usize) -> SourceAnchor {
        self.source.anchor(start, end).expect("fixture range")
    }

    fn tag_trigger(&self, token_index: usize) -> HtmlTreeTokenTrigger {
        let HtmlToken::Tag(tag) = &self.run.tokens()[token_index] else {
            panic!("fixture token is a tag")
        };
        HtmlTreeTokenTrigger::authored(token_index, tag.complete().clone())
    }

    fn eof_trigger(&self) -> HtmlTreeTokenTrigger {
        let token_index = self.run.tokens().len() - 1;
        assert!(matches!(
            self.run.tokens()[token_index],
            HtmlToken::EndOfFile(_)
        ));
        HtmlTreeTokenTrigger::end_of_file(token_index)
    }
}

fn valid_parts(fixture: &FreezeFixture) -> HtmlDocumentShellParts {
    let mut session = HtmlTreeSession::new().expect("session start");
    let mut stopped = false;
    for (token_index, token) in fixture.run.tokens().iter().enumerate() {
        let admitted = admit(token).expect("freeze fixture is admitted");
        let trigger = token_trigger(token, token_index);
        match drive_token(&mut session, &admitted, &trigger).expect("fixture dispatch") {
            TokenOutcome::Consumed => {}
            TokenOutcome::StoppedParsing => {
                assert_eq!(token_index + 1, fixture.run.tokens().len());
                stopped = true;
                break;
            }
            TokenOutcome::Unsupported(capability) => {
                panic!("freeze fixture unexpectedly unsupported: {capability:?}")
            }
        }
    }
    assert!(stopped, "complete fixture reaches EOF stop");
    session.finish(HtmlTreeCompletion::Complete)
}

fn freeze_parts(
    fixture: &FreezeFixture,
    parts: HtmlDocumentShellParts,
) -> Result<HtmlDocumentShellAnalysis, HtmlTreeFreezeError> {
    freeze(&fixture.source, fixture.run.clone(), parts)
}

fn body_diagnostic(trigger: HtmlTreeTokenTrigger) -> HtmlTreeDiagnostic {
    HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements,
        trigger,
        HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements,
    )
}

fn inserted_selected_ids(parts: &HtmlDocumentShellParts) -> Vec<HtmlConstructedNodeId> {
    parts
        .actions
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { node, .. } => Some(*node),
            _ => None,
        })
        .collect()
}

fn inserted_paragraph_id(parts: &HtmlDocumentShellParts) -> HtmlConstructedNodeId {
    parts
        .actions
        .iter()
        .find_map(|action| match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredParagraphElement { node } => Some(*node),
            _ => None,
        })
        .expect("fixture Paragraph")
}

fn body_ack_action_index(parts: &HtmlDocumentShellParts, token_index: usize) -> usize {
    parts
        .actions
        .iter()
        .position(|action| {
            action.trigger().token_index() == token_index
                && matches!(
                    action.kind(),
                    HtmlTreeActionKind::AcknowledgedShellEndTag {
                        name: HtmlShellElementName::Body
                    }
                )
        })
        .expect("fixture body acknowledgement")
}

#[test]
fn production_correspondence_has_no_candidate_machine_dependency() {
    let source = include_str!("in_body_body_end_open_stack_successor_production.rs");
    let forbidden = ["in_body_body_end_open_stack_successor_", "validation"].concat();
    assert!(!source.contains(&forbidden));
}

#[test]
fn p_only_div_section_and_heterogeneous_body_end_have_exact_cardinality_and_zero_mutation() {
    let cases = [
        ("<body><p></body>", 0usize, 2usize, 5usize),
        ("<body><div></body>", 1, 2, 5),
        ("<body><section></body>", 1, 2, 5),
        ("<body><div><section></body>", 1, 3, 6),
        ("<body><section><div><p></body>", 1, 4, 7),
    ];
    for (source, expected_diagnostics, body_token, expected_nodes) in cases {
        let analysis = analyze(source);
        assert!(analysis.is_complete(), "{source}");
        assert_eq!(analysis.node_count(), expected_nodes, "{source}");
        assert_eq!(analysis.coverage().committed_end(), source.len(), "{source}");
        assert_eq!(
            diagnostic_count(
                &analysis,
                HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements,
            ),
            expected_diagnostics,
            "{source}"
        );
        assert_eq!(body_acknowledgements(&analysis).len(), 1, "{source}");
        assert_eq!(
            body_acknowledgements(&analysis)[0].0,
            body_token,
            "{source}"
        );
        assert_eq!(forbidden_body_end_actions(&analysis, body_token), 0, "{source}");
        assert!(analysis.actions().iter().all(|action| {
            action.trigger().token_index() != body_token
                || !matches!(
                    action.kind(),
                    HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
                        | HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
                        | HtmlTreeActionKind::ClosedParagraphElement { .. }
                        | HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
                )
        }));
    }
}

#[test]
fn body_end_preserves_exact_ordered_open_identities_and_creation_inventory() {
    for (without_end, with_end) in [
        ("<body><p>", "<body><p></body>"),
        ("<body><div>", "<body><div></body>"),
        (
            "<body><div><section><p>",
            "<body><div><section><p></body>",
        ),
    ] {
        let control = FreezeFixture::new(without_end);
        let candidate = FreezeFixture::new(with_end);
        let control_parts = valid_parts(&control);
        let candidate_parts = valid_parts(&candidate);
        assert_eq!(
            control_parts.final_open_selected_ordinary,
            candidate_parts.final_open_selected_ordinary,
            "{with_end}: exact selected identities and order"
        );
        assert_eq!(
            control_parts.final_open_paragraph,
            candidate_parts.final_open_paragraph,
            "{with_end}: exact P identity"
        );
        assert_eq!(
            control_parts.admitted_creation_events,
            candidate_parts.admitted_creation_events,
            "{with_end}: body end admits no identity"
        );
        assert_eq!(
            control_parts.nodes.len(),
            candidate_parts.nodes.len(),
            "{with_end}: body end creates no node"
        );
    }
}

#[test]
fn body_end_trigger_retains_exact_token_source_id_and_complete_authored_range() {
    let source = "<body><DiV></BoDy>";
    let analysis = analyze_with(source, 77);
    assert!(analysis.is_complete());
    assert_eq!(
        body_acknowledgements(&analysis),
        vec![(2, SourceId::new(77), (11, 18))]
    );
    assert_eq!(
        diagnostic_evidence(
            &analysis,
            HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements,
        ),
        vec![(
            2,
            SourceId::new(77),
            (11, 18),
            HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements,
        )]
    );
    let HtmlToken::Tag(tag) = &analysis.tokenizer_run().tokens()[2] else {
        panic!("body end token")
    };
    assert_eq!(tag.kind(), HtmlTagKind::End);
    assert_eq!(tag.name().interpreted(), "body");
    assert_eq!(span(tag.complete()), (11, 18));
    assert_eq!(span(tag.name().source()), (13, 17));
    assert_eq!(&source[13..17], "BoDy");
}

#[test]
fn after_body_eof_keeps_selected_and_p_open_without_in_body_selected_eof_diagnostic() {
    for source in [
        "<body><p></body>",
        "<body><div></body>",
        "<body><div><section><p></body>",
    ] {
        let analysis = analyze(source);
        assert!(analysis.is_complete(), "{source}");
        assert_eq!(
            diagnostic_count(
                &analysis,
                HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile,
            ),
            0,
            "{source}"
        );
        let (session, outcomes) = drive_session(source);
        assert_eq!(session.insertion_mode(), InsertionMode::AfterBody, "{source}");
        assert!(matches!(outcomes.last(), Some(TokenOutcome::StoppedParsing)));
    }
}

#[test]
fn whitespace_after_body_uses_retained_current_parent_and_keeps_after_body_mode() {
    let cases = [
        ("<body></body> ", HtmlElementName::Shell(HtmlShellElementName::Body), " "),
        (
            "<body><div></body> ",
            HtmlElementName::SelectedOrdinary(HtmlSelectedOrdinaryElementName::Div),
            " ",
        ),
        (
            "<body><div><section></body> \t",
            HtmlElementName::SelectedOrdinary(HtmlSelectedOrdinaryElementName::Section),
            " \t",
        ),
        ("<body><p></body> ", HtmlElementName::Paragraph, " "),
        (
            "<body><div><p></body> ",
            HtmlElementName::Paragraph,
            " ",
        ),
    ];
    for (source, expected_parent, expected_text) in cases {
        let analysis = analyze(source);
        assert!(analysis.is_complete(), "{source:?}");
        assert_eq!(reprocess_count(&analysis), 0, "{source:?}");
        assert_eq!(
            diagnostic_count(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData),
            0,
            "{source:?}"
        );
        let text = *text_nodes(&analysis).last().expect("delegated text node");
        assert_eq!(parent_name(&analysis, text), expected_parent, "{source:?}");
        let HtmlTreeNodeKind::Text(text) = text.kind() else {
            panic!("text node")
        };
        assert_eq!(text.interpreted(), expected_text, "{source:?}");
        assert_eq!(text.contributions().len(), 1, "one aggregate contribution");
        let (session, _) = drive_session(source);
        assert_eq!(session.insertion_mode(), InsertionMode::AfterBody, "{source:?}");
    }
}

#[test]
fn non_whitespace_after_body_records_then_reprocesses_once_under_retained_parent() {
    let cases = [
        ("<body><p></body>x", HtmlElementName::Paragraph),
        (
            "<body><div></body>x",
            HtmlElementName::SelectedOrdinary(HtmlSelectedOrdinaryElementName::Div),
        ),
        (
            "<body><div><section><p></body>x",
            HtmlElementName::Paragraph,
        ),
    ];
    for (source, expected_parent) in cases {
        let analysis = analyze(source);
        assert!(analysis.is_complete(), "{source}");
        assert_eq!(reprocess_count(&analysis), 1, "{source}");
        assert_eq!(
            diagnostic_count(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData),
            1,
            "{source}"
        );
        let after_body = analysis
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.code() == HtmlTreeDiagnosticCode::AfterBodyCharacterData)
            .expect("AfterBody diagnostic");
        assert_eq!(
            after_body.recovery(),
            HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken
        );
        let reprocess_position = analysis
            .actions()
            .iter()
            .position(|action| matches!(action.kind(), HtmlTreeActionKind::ReprocessedToken))
            .expect("reprocess action");
        let text_position = analysis
            .actions()
            .iter()
            .position(|action| matches!(
                action.kind(),
                HtmlTreeActionKind::InsertedTextNode { .. }
                    | HtmlTreeActionKind::AppendedToTextNode { .. }
            ))
            .expect("text action");
        assert!(reprocess_position < text_position, "{source}");
        assert_eq!(
            analysis.actions()[reprocess_position].trigger().token_index(),
            after_body.trigger().token_index(),
            "same retained aggregate"
        );
        let text = *text_nodes(&analysis).last().expect("reprocessed text");
        assert_eq!(parent_name(&analysis, text), expected_parent, "{source}");
        let (session, _) = drive_session(source);
        assert_eq!(session.insertion_mode(), InsertionMode::InBody, "{source}");
    }
}

#[test]
fn mixed_after_body_aggregate_refuses_whole_before_any_mutation() {
    let source = "<body><div><p></body> x";
    let analysis = analyze(source);
    assert_eq!(
        unsupported_evidence(&analysis),
        Some((
            HtmlTreeCapability::WhitespaceSensitiveCharacterData,
            4,
            Some((21, 23)),
        ))
    );
    assert_eq!(analysis.coverage().committed_end(), 21);
    assert_eq!(analysis.coverage().processed_tokens(), 4);
    assert_eq!(analysis.node_count(), 6);
    assert!(text_nodes(&analysis).is_empty());
    assert_eq!(reprocess_count(&analysis), 0);
    assert_eq!(
        diagnostic_count(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData),
        0
    );
    assert_eq!(body_acknowledgements(&analysis).len(), 1);
    let (session, outcomes) = drive_session(source);
    assert_eq!(session.insertion_mode(), InsertionMode::AfterBody);
    assert_eq!(
        outcomes.last(),
        Some(&TokenOutcome::Unsupported(
            HtmlTreeCapability::WhitespaceSensitiveCharacterData,
        ))
    );
}

#[test]
fn later_matching_p_and_selected_ends_close_the_original_retained_identities() {
    let p = analyze("<body><p></body>x</p>");
    let p_insertions = paragraph_insertions(&p);
    assert_eq!(p_insertions.len(), 1);
    let p_id = p_insertions[0].0;
    let p_closures: Vec<(HtmlConstructedNodeId, usize)> = p
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::ClosedParagraphElement {
                node,
                closure: HtmlParagraphClosure::MatchingEndTag,
            } => Some((*node, action.trigger().token_index())),
            _ => None,
        })
        .collect();
    assert_eq!(p_closures, vec![(p_id, 4)]);
    assert_eq!(forbidden_body_end_actions(&p, 2), 0);

    let selected = analyze("<body><div></body>x</div>");
    let selected_insertions = selected_insertions(&selected);
    assert_eq!(selected_insertions.len(), 1);
    assert_eq!(
        selected_insertions[0].1,
        HtmlSelectedOrdinaryElementName::Div
    );
    let selected_id = selected_insertions[0].0;
    let selected_closures: Vec<(HtmlConstructedNodeId, usize)> = selected
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::ClosedSelectedOrdinaryElement { node, .. } => {
                Some((*node, action.trigger().token_index()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(selected_closures, vec![(selected_id, 4)]);
    assert_eq!(forbidden_body_end_actions(&selected, 2), 0);
}

#[test]
fn repeated_body_end_after_non_whitespace_recovery_reuses_the_same_open_stack() {
    let source = "<body><div></body>x</body>";
    let analysis = analyze(source);
    assert!(analysis.is_complete());
    assert_eq!(
        body_acknowledgements(&analysis)
            .iter()
            .map(|(token, _, _)| *token)
            .collect::<Vec<_>>(),
        vec![2, 4]
    );
    assert_eq!(
        diagnostic_evidence(
            &analysis,
            HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements,
        )
        .iter()
        .map(|(token, _, _, recovery)| (*token, *recovery))
        .collect::<Vec<_>>(),
        vec![
            (
                2,
                HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements,
            ),
            (
                4,
                HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements,
            ),
        ]
    );
    assert_eq!(reprocess_count(&analysis), 1);
    assert_eq!(selected_insertions(&analysis).len(), 1);
    assert!(analysis.actions().iter().all(|action| !matches!(
        action.kind(),
        HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
            | HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
    )));
    let (session, _) = drive_session(source);
    assert_eq!(session.insertion_mode(), InsertionMode::AfterBody);
}

#[test]
fn body_end_shape_and_other_shell_crossings_keep_the_broad_firewalls() {
    for (source, capability, token, trigger) in [
        (
            "<body><p></body id=x>",
            HtmlTreeCapability::ShellTagAttribute,
            2usize,
            (9usize, 21usize),
        ),
        (
            "<body><div></body/>",
            HtmlTreeCapability::SelfClosingShellTag,
            2,
            (11, 19),
        ),
        (
            "<body><p></html>",
            HtmlTreeCapability::ShellTagWithOpenParagraphElement,
            2,
            (9, 16),
        ),
        (
            "<body><div></html>",
            HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement,
            2,
            (11, 18),
        ),
        (
            "<body><p><body>",
            HtmlTreeCapability::ShellTagWithOpenParagraphElement,
            2,
            (9, 15),
        ),
        (
            "<body><div><body>",
            HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement,
            2,
            (11, 17),
        ),
    ] {
        let analysis = analyze(source);
        assert_eq!(
            unsupported_evidence(&analysis),
            Some((capability, token, Some(trigger))),
            "{source}"
        );
        assert!(body_acknowledgements(&analysis).is_empty(), "{source}");
        assert_eq!(analysis.coverage().processed_tokens(), token, "{source}");
        assert_eq!(analysis.coverage().committed_end(), trigger.0, "{source}");
    }
}

#[test]
fn committed_coverage_processed_tokens_and_identity_non_allocation_are_exact() {
    let supported = analyze("<body><div></body>x");
    assert!(supported.is_complete());
    assert_eq!(supported.coverage().committed_end(), 19);
    assert_eq!(supported.coverage().processed_tokens(), 5);

    let refused = analyze("<body><div><p></body> x");
    assert_eq!(refused.coverage().committed_end(), 21);
    assert_eq!(refused.coverage().processed_tokens(), 4);

    for (without_end, with_end) in [
        ("<body><p>", "<body><p></body>"),
        ("<body><div>", "<body><div></body>"),
        (
            "<body><div><section><p>",
            "<body><div><section><p></body>",
        ),
    ] {
        let control = analyze(without_end);
        let candidate = analyze(with_end);
        assert_eq!(control.node_count(), candidate.node_count(), "{with_end}");
        assert_eq!(
            selected_insertions(&control)
                .iter()
                .map(|(node, _, _)| *node)
                .collect::<Vec<_>>(),
            selected_insertions(&candidate)
                .iter()
                .map(|(node, _, _)| *node)
                .collect::<Vec<_>>(),
            "{with_end}"
        );
        assert_eq!(
            paragraph_insertions(&control),
            paragraph_insertions(&candidate),
            "{with_end}"
        );
    }
}

#[test]
fn freeze_rejects_body_end_diagnostic_absence_presence_and_duplicate_corruption() {
    let p_only = FreezeFixture::new("<body><p></body>");
    let mut parts = valid_parts(&p_only);
    parts.diagnostics.push(body_diagnostic(p_only.tag_trigger(2)));
    assert!(matches!(
        freeze_parts(&p_only, parts),
        Err(HtmlTreeFreezeError::BodyEndDiagnosticCardinalityMismatch {
            token_index: 2,
            selected_open: 0,
            diagnostics: 1,
        })
    ));

    let selected = FreezeFixture::new("<body><div></body>");
    let mut parts = valid_parts(&selected);
    parts.diagnostics.retain(|diagnostic| {
        diagnostic.code()
            != HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements
    });
    assert!(matches!(
        freeze_parts(&selected, parts),
        Err(HtmlTreeFreezeError::BodyEndDiagnosticCardinalityMismatch {
            token_index: 2,
            selected_open: 1,
            diagnostics: 0,
        })
    ));

    let mut parts = valid_parts(&selected);
    parts
        .diagnostics
        .push(body_diagnostic(selected.tag_trigger(2)));
    assert!(matches!(
        freeze_parts(&selected, parts),
        Err(HtmlTreeFreezeError::BodyEndDiagnosticCardinalityMismatch {
            token_index: 2,
            selected_open: 1,
            diagnostics: 2,
        })
    ));
}

#[test]
fn freeze_rejects_body_end_diagnostic_token_source_range_recovery_and_phase_corruption() {
    let fixture = FreezeFixture::new("<body><div></body>");

    let mut wrong_token = valid_parts(&fixture);
    let diagnostic_index = wrong_token
        .diagnostics
        .iter()
        .position(|diagnostic| {
            diagnostic.code()
                == HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements
        })
        .expect("TC-S7 diagnostic");
    wrong_token.diagnostics[diagnostic_index] = body_diagnostic(fixture.tag_trigger(1));
    assert!(matches!(
        freeze_parts(&fixture, wrong_token),
        Err(HtmlTreeFreezeError::BodyEndDiagnosticCardinalityMismatch {
            token_index: 2,
            ..
        })
    ));

    let mut wrong_range = valid_parts(&fixture);
    wrong_range.diagnostics[diagnostic_index] = body_diagnostic(
        HtmlTreeTokenTrigger::authored(2, fixture.anchor(12, 18)),
    );
    assert!(matches!(
        freeze_parts(&fixture, wrong_range),
        Err(HtmlTreeFreezeError::BodyEndDiagnosticTriggerOrRecoveryMismatch {
            token_index: 2,
        })
    ));

    let foreign_source = SourceText::new(
        SourceId::new(74),
        "<body><div></body>".to_owned(),
    );
    let mut wrong_source = valid_parts(&fixture);
    wrong_source.diagnostics[diagnostic_index] = body_diagnostic(
        HtmlTreeTokenTrigger::authored(
            2,
            foreign_source.anchor(11, 18).expect("foreign body end"),
        ),
    );
    assert!(matches!(
        freeze_parts(&fixture, wrong_source),
        Err(HtmlTreeFreezeError::BodyEndDiagnosticTriggerOrRecoveryMismatch {
            token_index: 2,
        })
    ));

    let mut wrong_recovery = valid_parts(&fixture);
    wrong_recovery.diagnostics[diagnostic_index] = HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements,
        fixture.tag_trigger(2),
        HtmlTreeRecovery::IgnoredToken,
    );
    assert!(matches!(
        freeze_parts(&fixture, wrong_recovery),
        Err(HtmlTreeFreezeError::BodyEndDiagnosticTriggerOrRecoveryMismatch {
            token_index: 2,
        })
    ));

    // No later diagnostic may claim the already-consumed body-end phase.
    let mut wrong_phase = valid_parts(&fixture);
    wrong_phase.diagnostics.push(HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::MissingDoctype,
        fixture.tag_trigger(2),
        HtmlTreeRecovery::ContinuedInQuirksDocumentMode,
    ));
    assert!(matches!(
        freeze_parts(&fixture, wrong_phase),
        Err(HtmlTreeFreezeError::BodyEndDiagnosticTriggerOrRecoveryMismatch {
            token_index: 2,
        })
    ));
}

#[test]
fn freeze_rejects_non_body_and_duplicate_body_acknowledgement_corruption() {
    let fixture = FreezeFixture::new("<body><div></body>");
    let mut non_body = valid_parts(&fixture);
    let action_index = body_ack_action_index(&non_body, 2);
    non_body.actions[action_index] = HtmlTreeAction::new(
        HtmlTreeActionKind::AcknowledgedShellEndTag {
            name: HtmlShellElementName::Body,
        },
        fixture.tag_trigger(1),
    );
    assert!(matches!(
        freeze_parts(&fixture, non_body),
        Err(HtmlTreeFreezeError::BodyEndAcknowledgementTriggerMismatch {
            token_index: 1,
        })
    ));

    let mut duplicate = valid_parts(&fixture);
    let action_index = body_ack_action_index(&duplicate, 2);
    duplicate.actions.insert(
        action_index + 1,
        HtmlTreeAction::new(
            HtmlTreeActionKind::AcknowledgedShellEndTag {
                name: HtmlShellElementName::Body,
            },
            fixture.tag_trigger(2),
        ),
    );
    assert!(matches!(
        freeze_parts(&fixture, duplicate),
        Err(HtmlTreeFreezeError::DuplicateBodyEndAcknowledgement {
            token_index: 2,
        })
    ));
}

#[test]
fn freeze_rejects_every_same_body_trigger_p_selected_synthesis_and_creation_mutation() {
    let fixture = FreezeFixture::new("<body><div><p></body>");
    let baseline = valid_parts(&fixture);
    let selected = inserted_selected_ids(&baseline)[0];
    let paragraph = inserted_paragraph_id(&baseline);
    for corruption in [
        HtmlTreeActionKind::ClosedParagraphElement {
            node: paragraph,
            closure: HtmlParagraphClosure::MatchingEndTag,
        },
        HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag {
            node: paragraph,
            target: selected,
        },
        HtmlTreeActionKind::ClosedSelectedOrdinaryElement {
            node: selected,
            name: HtmlSelectedOrdinaryElementName::Div,
        },
        HtmlTreeActionKind::InsertedSynthesizedParagraphElement {
            node: paragraph,
            cause: HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag,
        },
        HtmlTreeActionKind::InsertedTextNode { node: paragraph },
    ] {
        let mut parts = valid_parts(&fixture);
        let action_index = body_ack_action_index(&parts, 3);
        parts.actions.insert(
            action_index + 1,
            HtmlTreeAction::new(corruption, fixture.tag_trigger(3)),
        );
        assert!(freeze_parts(&fixture, parts).is_err());
    }

    let recovery_fixture = FreezeFixture::new("<body><div><section></body>");
    let mut recovery = valid_parts(&recovery_fixture);
    let selected = inserted_selected_ids(&recovery);
    let action_index = body_ack_action_index(&recovery, 3);
    recovery.actions.insert(
        action_index + 1,
        HtmlTreeAction::new(
            HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag {
                node: selected[1],
                target: selected[0],
            },
            recovery_fixture.tag_trigger(3),
        ),
    );
    assert!(freeze_parts(&recovery_fixture, recovery).is_err());

    let mut allocated = valid_parts(&fixture);
    allocated.admitted_creation_events += 1;
    assert!(matches!(
        freeze_parts(&fixture, allocated),
        Err(HtmlTreeFreezeError::CreationEventInventoryMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_after_body_eof_selected_diagnostic_and_final_open_checkpoint_corruption() {
    let selected_fixture = FreezeFixture::new("<body><div></body>");
    let mut eof_diagnostic = valid_parts(&selected_fixture);
    eof_diagnostic.diagnostics.push(HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile,
        selected_fixture.eof_trigger(),
        HtmlTreeRecovery::StoppedParsingWithOpenSelectedOrdinaryElements,
    ));
    assert!(matches!(
        freeze_parts(&selected_fixture, eof_diagnostic),
        Err(HtmlTreeFreezeError::BodyEndAfterBodyEofDiagnosticMismatch {
            token_index: 3,
        })
    ));

    let mut selected_checkpoint = valid_parts(&selected_fixture);
    selected_checkpoint.final_open_selected_ordinary.clear();
    assert!(matches!(
        freeze_parts(&selected_fixture, selected_checkpoint),
        Err(HtmlTreeFreezeError::FinalOpenSelectedOrdinaryStateMismatch { .. })
    ));

    let paragraph_fixture = FreezeFixture::new("<body><p></body>");
    let mut paragraph_checkpoint = valid_parts(&paragraph_fixture);
    paragraph_checkpoint.final_open_paragraph = None;
    assert!(matches!(
        freeze_parts(&paragraph_fixture, paragraph_checkpoint),
        Err(HtmlTreeFreezeError::FinalOpenParagraphStateMismatch { .. })
    ));
}

#[test]
fn lower_layer_incompleteness_is_never_upgraded() {
    let source = SourceText::new(
        SourceId::new(1),
        "<body><div><p></body>x".to_owned(),
    );
    let analysis = construct_html_document_shell(
        &source,
        HtmlTokenizerLimits::new(1_024, 8_192, 4, 1_024, 256, 4_096, 1_024),
    )
    .expect("tree boundary");
    assert!(analysis.tokenizer_run().is_incomplete());
    assert!(!analysis.is_complete());
    assert!(matches!(
        analysis.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));
    assert_eq!(body_acknowledgements(&analysis).len(), 1);
    assert_eq!(analysis.coverage().processed_tokens(), 4);
    assert_eq!(analysis.coverage().committed_end(), 21);
}

#[test]
fn private_storage_permutation_preserves_tc_s7_semantic_identity_and_evidence() {
    for source in [
        "<body><p></body> ",
        "<body><div><section><p></body>x</div>",
        "<body><div></body></html>",
    ] {
        let baseline = analyze_with(source, 42);
        let permuted = baseline.clone().with_reversed_storage();
        assert_eq!(semantic_signature(&baseline), semantic_signature(&permuted), "{source}");
        assert_eq!(
            format!("{:?}", baseline.actions()),
            format!("{:?}", permuted.actions()),
            "{source}"
        );
        assert_eq!(
            format!("{:?}", baseline.diagnostics()),
            format!("{:?}", permuted.diagnostics()),
            "{source}"
        );
        assert_eq!(baseline.coverage().committed_end(), permuted.coverage().committed_end());
        assert_eq!(baseline.coverage().processed_tokens(), permuted.coverage().processed_tokens());
        assert_eq!(baseline.is_complete(), permuted.is_complete());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GeneratedBlock {
    Div,
    Section,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GeneratedSuccessor {
    Eof,
    Whitespace,
    NonWhitespace,
}

fn generated_block_sequences(max_test_depth: usize) -> Vec<Vec<GeneratedBlock>> {
    let mut sequences = vec![Vec::new()];
    for depth in 1..=max_test_depth {
        for mask in 0..(1usize << depth) {
            let mut sequence = Vec::with_capacity(depth);
            for bit in 0..depth {
                sequence.push(if mask & (1usize << bit) == 0 {
                    GeneratedBlock::Div
                } else {
                    GeneratedBlock::Section
                });
            }
            sequences.push(sequence);
        }
    }
    sequences
}

fn generated_source(
    blocks: &[GeneratedBlock],
    paragraph: bool,
    successor: GeneratedSuccessor,
) -> String {
    let mut source = String::from("<body>");
    for block in blocks {
        source.push_str(match block {
            GeneratedBlock::Div => "<div>",
            GeneratedBlock::Section => "<section>",
        });
    }
    if paragraph {
        source.push_str("<p>");
    }
    source.push_str("</body>");
    match successor {
        GeneratedSuccessor::Eof => {}
        GeneratedSuccessor::Whitespace => source.push(' '),
        GeneratedSuccessor::NonWhitespace => source.push('x'),
    }
    source
}

fn generated_expected_parent(
    blocks: &[GeneratedBlock],
    paragraph: bool,
) -> HtmlElementName {
    if paragraph {
        HtmlElementName::Paragraph
    } else {
        match blocks.last() {
            Some(GeneratedBlock::Div) => {
                HtmlElementName::SelectedOrdinary(HtmlSelectedOrdinaryElementName::Div)
            }
            Some(GeneratedBlock::Section) => {
                HtmlElementName::SelectedOrdinary(HtmlSelectedOrdinaryElementName::Section)
            }
            None => HtmlElementName::Shell(HtmlShellElementName::Body),
        }
    }
}

fn generated_expected_diagnostics(
    blocks: &[GeneratedBlock],
    successor: GeneratedSuccessor,
) -> Vec<HtmlTreeDiagnosticCode> {
    let mut diagnostics = vec![HtmlTreeDiagnosticCode::MissingDoctype];
    if !blocks.is_empty() {
        diagnostics.push(HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements);
    }
    if successor == GeneratedSuccessor::NonWhitespace {
        diagnostics.push(HtmlTreeDiagnosticCode::AfterBodyCharacterData);
        if !blocks.is_empty() {
            diagnostics.push(HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile);
        }
    }
    diagnostics
}

#[test]
fn generated_bounded_stack_successors_match_an_independent_closed_form_model() {
    // Depth four is test generation only. It is not read by production and
    // introduces no parser resource dimension or normative stack constant.
    for blocks in generated_block_sequences(4) {
        for paragraph in [false, true] {
            for successor in [
                GeneratedSuccessor::Eof,
                GeneratedSuccessor::Whitespace,
                GeneratedSuccessor::NonWhitespace,
            ] {
                let source = generated_source(&blocks, paragraph, successor);
                let analysis = analyze_with(&source, 9);
                assert!(analysis.is_complete(), "{source}");
                assert_eq!(analysis.coverage().committed_end(), source.len(), "{source}");
                assert_eq!(
                    analysis.coverage().processed_tokens(),
                    analysis.tokenizer_run().tokens().len(),
                    "{source}"
                );
                let has_text = successor != GeneratedSuccessor::Eof;
                assert_eq!(
                    analysis.node_count(),
                    4 + blocks.len() + usize::from(paragraph) + usize::from(has_text),
                    "{source}"
                );
                assert_eq!(
                    analysis
                        .diagnostics()
                        .iter()
                        .map(HtmlTreeDiagnostic::code)
                        .collect::<Vec<_>>(),
                    generated_expected_diagnostics(&blocks, successor),
                    "{source}"
                );
                assert_eq!(
                    reprocess_count(&analysis),
                    usize::from(successor == GeneratedSuccessor::NonWhitespace),
                    "{source}"
                );
                assert_eq!(body_acknowledgements(&analysis).len(), 1, "{source}");
                let body_token = body_acknowledgements(&analysis)[0].0;
                assert_eq!(forbidden_body_end_actions(&analysis, body_token), 0, "{source}");

                if has_text {
                    let text = *text_nodes(&analysis).last().expect("generated text");
                    assert_eq!(
                        parent_name(&analysis, text),
                        generated_expected_parent(&blocks, paragraph),
                        "{source}"
                    );
                } else {
                    assert!(text_nodes(&analysis).is_empty(), "{source}");
                }

                let (session, _) = drive_session(&source);
                let expected_mode = match successor {
                    GeneratedSuccessor::Eof | GeneratedSuccessor::Whitespace => {
                        InsertionMode::AfterBody
                    }
                    GeneratedSuccessor::NonWhitespace => InsertionMode::InBody,
                };
                assert_eq!(session.insertion_mode(), expected_mode, "{source}");
            }
        }
    }
}
