//! Production correspondence for TC-S9 — selected InHead `<style>` RAWTEXT
//! feedback lifecycle (Issue #388).
//!
//! Expectations in this module are independently authored from the accepted
//! production theorem. The tests import only production tokenizer,
//! tree-construction, durable-result, and freeze seams. They do not import or
//! call the candidate-independent validation transition machine and do not use
//! an external browser/parser as an oracle.

use crate::{SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::producer::{
    HtmlTokenizerSession, HtmlTokenizerSessionBoundary, tokenize,
};
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::{
    HtmlTokenizerCapability, HtmlTokenizerCapabilityAvailability, HtmlTokenizerCompletion,
    HtmlTokenizerIncompleteCause, HtmlTokenizerMode, HtmlTokenizerRunResult,
};
use super::driver::construct_html_document_shell;
use super::result::{
    HtmlConstructedNodeId, HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlElement,
    HtmlElementName, HtmlShellElementName, HtmlTreeAction, HtmlTreeActionKind, HtmlTreeCapability,
    HtmlTreeCompletion, HtmlTreeDiagnosticCode, HtmlTreeFreezeError, HtmlTreeIncompleteCause,
    HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery, freeze,
};
use super::session::{
    DispatchOutcome, HtmlTreeSession, HtmlTreeTokenizerFeedback, InsertionMode, admit, token_trigger,
};

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 4_096, 256, 16_384, 4_096)
}

fn analyze(source: &str) -> HtmlDocumentShellAnalysis {
    analyze_with(source, 1, limits())
}

fn analyze_with(
    source: &str,
    source_id: u64,
    limits: HtmlTokenizerLimits,
) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(source_id), source.to_owned());
    construct_html_document_shell(&source, limits).expect("TC-S9 production boundary")
}

fn style_id(analysis: &HtmlDocumentShellAnalysis) -> HtmlConstructedNodeId {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .find_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(HtmlElement::Style(_)) => Some(node.id()),
            _ => None,
        })
        .expect("Style node")
}

fn style_text(analysis: &HtmlDocumentShellAnalysis) -> String {
    let style = style_id(analysis);
    analysis
        .node(style)
        .expect("Style node")
        .children()
        .iter()
        .map(|child| match analysis.node(*child).expect("Style child").kind() {
            HtmlTreeNodeKind::Text(text) => text.interpreted().to_owned(),
            other => panic!("Style child must be text, got {other:?}"),
        })
        .collect()
}

fn style_action_token(
    analysis: &HtmlDocumentShellAnalysis,
    predicate: impl Fn(&HtmlTreeActionKind) -> bool,
) -> usize {
    analysis
        .actions()
        .iter()
        .find(|action| predicate(action.kind()))
        .map(|action| action.trigger().token_index())
        .expect("Style action")
}

fn style_insert_token(analysis: &HtmlDocumentShellAnalysis) -> usize {
    style_action_token(analysis, |kind| {
        matches!(kind, HtmlTreeActionKind::InsertedAuthoredStyleElement { .. })
    })
}

fn style_close_token(analysis: &HtmlDocumentShellAnalysis) -> usize {
    style_action_token(analysis, |kind| {
        matches!(kind, HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { .. })
    })
}

fn tree_unsupported(analysis: &HtmlDocumentShellAnalysis) -> Option<HtmlTreeCapability> {
    match analysis.completion() {
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => Some(unsupported.capability()),
        HtmlTreeCompletion::Complete
        | HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete) => None,
    }
}

fn semantic_signature(analysis: &HtmlDocumentShellAnalysis) -> Vec<String> {
    let mut signature = Vec::new();
    for node in analysis.nodes_in_creation_order() {
        let kind = match node.kind() {
            HtmlTreeNodeKind::Document => "document".to_owned(),
            HtmlTreeNodeKind::Element(element) => format!("element:{:?}", element.name()),
            HtmlTreeNodeKind::Text(text) => format!("text:{}", text.interpreted()),
        };
        signature.push(format!(
            "{:?}|{:?}|{:?}|{kind}",
            node.id(),
            node.parent(),
            node.children()
        ));
    }
    signature
}

fn action_signature(analysis: &HtmlDocumentShellAnalysis) -> Vec<String> {
    analysis
        .actions()
        .iter()
        .map(|action| {
            format!(
                "{}|{:?}",
                action.trigger().token_index(),
                action.kind()
            )
        })
        .collect()
}

#[test]
fn p1_plain_style_round_trip_is_authored_and_complete() {
    let analysis = analyze("<style></style>");
    assert!(analysis.is_complete());
    let style = style_id(&analysis);
    let node = analysis.node(style).expect("Style");
    assert_eq!(
        node.parent()
            .and_then(|id| analysis.node(id))
            .and_then(|node| match node.kind() {
                HtmlTreeNodeKind::Element(element) => Some(element.name()),
                _ => None,
            }),
        Some(HtmlElementName::Shell(HtmlShellElementName::Head))
    );
    assert_eq!(style_text(&analysis), "");
    assert!(analysis.actions().iter().any(|action| matches!(
        action.kind(),
        HtmlTreeActionKind::InsertedAuthoredStyleElement { node } if *node == style
    )));
    assert!(analysis.actions().iter().any(|action| matches!(
        action.kind(),
        HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { node } if *node == style
    )));
}

#[test]
fn p2_feedback_is_applied_before_any_post_style_source_is_produced() {
    let source = SourceText::new(
        SourceId::new(11),
        "<style><b>x</style><body>".to_owned(),
    );
    let mut tokenizer = HtmlTokenizerSession::new(&source, limits());
    assert_eq!(
        tokenizer.drive_to_boundary(),
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::RawText)
    );
    assert_eq!(tokenizer.tokens().len(), 1, "only <style> may exist pre-feedback");
    let HtmlToken::Tag(style) = &tokenizer.tokens()[0] else {
        panic!("Style start token")
    };
    assert_eq!(style.kind(), HtmlTagKind::Start);
    assert_eq!(style.name().interpreted(), "style");

    tokenizer.apply_raw_text().expect("apply RAWTEXT");
    assert_eq!(
        tokenizer.drive_to_boundary(),
        HtmlTokenizerSessionBoundary::TokenAvailable,
        "appropriate close yields before post-close source"
    );
    assert!(matches!(
        tokenizer.tokens().last(),
        Some(HtmlToken::Tag(tag)) if tag.kind() == HtmlTagKind::End
            && tag.name().interpreted() == "style"
    ));
    assert!(tokenizer.tokens().iter().all(|token| !matches!(
        token,
        HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::Start
            && tag.name().interpreted() == "body"
    )));
}

#[test]
fn p2_tag_shaped_rawtext_is_text_and_never_constructs_b_identity() {
    let analysis = analyze("<style><b>x</style>");
    assert!(analysis.is_complete());
    assert_eq!(style_text(&analysis), "<b>x");
    assert!(analysis.nodes_in_creation_order().iter().all(|node| {
        !matches!(
            node.kind(),
            HtmlTreeNodeKind::Element(element)
                if !matches!(
                    element.name(),
                    HtmlElementName::Shell(_)
                        | HtmlElementName::SelectedOrdinary(_)
                        | HtmlElementName::Paragraph
                        | HtmlElementName::Style
                )
        )
    }));
}

#[test]
fn p3_non_appropriate_style_like_end_tag_remains_rawtext() {
    let analysis = analyze("<style>x</styler>y</style>");
    assert!(analysis.is_complete());
    assert_eq!(style_text(&analysis), "x</styler>y");
    assert_eq!(
        analysis
            .actions()
            .iter()
            .filter(|action| matches!(
                action.kind(),
                HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { .. }
            ))
            .count(),
        1
    );
}

#[test]
fn p4_mixed_case_appropriate_close_keeps_exact_authored_evidence() {
    let source = "<style>x</StYlE>";
    let analysis = analyze_with(source, 91, limits());
    assert!(analysis.is_complete());
    let token_index = style_close_token(&analysis);
    let HtmlToken::Tag(tag) = &analysis.tokenizer_run().tokens()[token_index] else {
        panic!("Style close tag")
    };
    assert_eq!(tag.kind(), HtmlTagKind::End);
    assert_eq!(tag.name().interpreted(), "style");
    assert_eq!(tag.name().source().fragment(), "StYlE");
    assert_eq!(tag.complete().fragment(), "</StYlE>");
    assert_eq!(tag.complete().source_id(), SourceId::new(91));
    let action = analysis
        .actions()
        .iter()
        .find(|action| matches!(
            action.kind(),
            HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { .. }
        ))
        .expect("Style close action");
    assert_eq!(action.trigger().token_index(), token_index);
    assert_eq!(
        action
            .trigger()
            .authored_boundary()
            .expect("authored close")
            .fragment(),
        "</StYlE>"
    );
}

#[test]
fn p5_close_returns_tokenizer_to_data_before_body_sentinel() {
    let analysis = analyze("<head><style><b>x</style><body>");
    assert!(analysis.is_complete());
    assert_eq!(style_text(&analysis), "<b>x");
    let close = style_close_token(&analysis);
    let body = analysis
        .tokenizer_run()
        .tokens()
        .iter()
        .position(|token| matches!(
            token,
            HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::Start
                && tag.name().interpreted() == "body"
        ))
        .expect("post-close Body token");
    assert!(body > close);
    assert!(analysis.nodes_in_creation_order().iter().any(|node| matches!(
        node.kind(),
        HtmlTreeNodeKind::Element(element)
            if element.name() == HtmlElementName::Shell(HtmlShellElementName::Body)
    )));
}

#[test]
fn p6_text_eof_pops_style_restores_in_head_and_reprocesses_same_eof() {
    let analysis = analyze("<style>x");
    assert!(analysis.is_complete());
    assert_eq!(style_text(&analysis), "x");
    assert!(!analysis.actions().iter().any(|action| matches!(
        action.kind(),
        HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { .. }
    )));
    let pop = analysis
        .actions()
        .iter()
        .find(|action| matches!(
            action.kind(),
            HtmlTreeActionKind::PoppedStyleElementAtEndOfFile { .. }
        ))
        .expect("Style EOF pop");
    let eof_index = pop.trigger().token_index();
    assert!(pop.trigger().authored_boundary().is_none());
    assert_eq!(
        analysis
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                diagnostic.code() == HtmlTreeDiagnosticCode::StyleEndOfFileInText
                    && diagnostic.trigger().token_index() == eof_index
                    && diagnostic.recovery()
                        == HtmlTreeRecovery::PoppedStyleAtEndOfFileAndRestoredInHead
            })
            .count(),
        1
    );
    assert!(analysis.actions().iter().any(|action| {
        action.trigger().token_index() == eof_index
            && matches!(action.kind(), HtmlTreeActionKind::ReprocessedToken)
    }));
}

#[test]
fn p7_rawtext_less_than_and_end_tag_open_fallbacks_are_character_data() {
    for (source, expected) in [
        ("<style><</style>", "<"),
        ("<style></</style>", "</"),
        ("<style></x</style>", "</x"),
    ] {
        let analysis = analyze(source);
        assert!(analysis.is_complete(), "{source}");
        assert_eq!(style_text(&analysis), expected, "{source}");
    }
}

#[test]
fn p8_source_id_perturbation_preserves_semantics_but_rebinds_evidence() {
    let first = analyze_with("<style><b>x</StYlE><body>", 7, limits());
    let second = analyze_with("<style><b>x</StYlE><body>", 700, limits());
    assert_eq!(semantic_signature(&first), semantic_signature(&second));
    assert_eq!(style_text(&first), style_text(&second));
    let HtmlToken::Tag(first_close) = &first.tokenizer_run().tokens()[style_close_token(&first)]
    else {
        panic!("first close")
    };
    let HtmlToken::Tag(second_close) = &second.tokenizer_run().tokens()[style_close_token(&second)]
    else {
        panic!("second close")
    };
    assert_eq!(first_close.name().source().fragment(), "StYlE");
    assert_eq!(second_close.name().source().fragment(), "StYlE");
    assert_ne!(first_close.complete().source_id(), second_close.complete().source_id());
}

#[test]
fn p9_repeat_runs_are_deterministic() {
    let first = analyze_with("<style>x</styler>y</style><body><div>z", 55, limits());
    let second = analyze_with("<style>x</styler>y</style><body><div>z", 55, limits());
    assert_eq!(semantic_signature(&first), semantic_signature(&second));
    assert_eq!(action_signature(&first), action_signature(&second));
    assert_eq!(
        first
            .diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.code(), diagnostic.trigger().token_index()))
            .collect::<Vec<_>>(),
        second
            .diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.code(), diagnostic.trigger().token_index()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn p10_excluded_style_shapes_refuse_transactionally() {
    for (source, expected) in [
        ("<style a>x</style>", HtmlTreeCapability::StyleTagAttribute),
        ("<style/>x", HtmlTreeCapability::SelfClosingStyleTag),
    ] {
        let analysis = analyze(source);
        assert_eq!(tree_unsupported(&analysis), Some(expected), "{source}");
        assert!(analysis.nodes_in_creation_order().iter().all(|node| !matches!(
            node.kind(),
            HtmlTreeNodeKind::Element(HtmlElement::Style(_))
        )));
        assert!(analysis.actions().iter().all(|action| !matches!(
            action.kind(),
            HtmlTreeActionKind::InsertedAuthoredStyleElement { .. }
                | HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { .. }
                | HtmlTreeActionKind::PoppedStyleElementAtEndOfFile { .. }
        )));
    }
}

#[test]
fn p11_batch_tokenizer_keeps_predecessor_deferred_rawtext_boundary() {
    let source = SourceText::new(SourceId::new(1), "<style><b>x</style>".to_owned());
    let run = tokenize(&source, limits());
    assert_eq!(run.tokens().len(), 1);
    match run.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => {
            assert_eq!(
                unsupported.capability(),
                HtmlTokenizerCapability::ContextDependentTokenizerMode {
                    mode: HtmlTokenizerMode::RawText,
                }
            );
            assert_eq!(
                unsupported.availability(),
                HtmlTokenizerCapabilityAvailability::Deferred
            );
        }
        other => panic!("batch RAWTEXT boundary changed: {other:?}"),
    }
}

#[test]
fn p12_tree_stop_does_not_guess_or_apply_future_style_feedback() {
    let analysis = analyze("<unknown><style>x</style>");
    assert_eq!(tree_unsupported(&analysis), Some(HtmlTreeCapability::NonShellElementTag));
    assert!(analysis.nodes_in_creation_order().iter().all(|node| !matches!(
        node.kind(),
        HtmlTreeNodeKind::Element(HtmlElement::Style(_))
    )));
    match analysis.tokenizer_run().completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => assert_eq!(
            unsupported.capability(),
            HtmlTokenizerCapability::ContextDependentTokenizerMode {
                mode: HtmlTokenizerMode::RawText,
            }
        ),
        other => panic!("tree stop must retain tokenizer evidence: {other:?}"),
    }
}

#[test]
fn p13_lower_layer_incomplete_is_never_upgraded_to_complete() {
    let constrained = HtmlTokenizerLimits::new(4_096, 1, 4_096, 4_096, 256, 16_384, 4_096);
    let analysis = analyze_with("<style>xxxxxxxxxxxxxxxx</style>", 1, constrained);
    assert!(!analysis.is_complete());
    assert!(analysis.tokenizer_run().is_incomplete());
    assert!(matches!(
        analysis.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));
}

#[test]
fn p15_representative_tc_s1_through_s8_control_remains_complete() {
    let analysis = analyze("<body><div>x</div></body></html>");
    assert!(analysis.is_complete());
    assert!(analysis.nodes_in_creation_order().iter().all(|node| !matches!(
        node.kind(),
        HtmlTreeNodeKind::Element(HtmlElement::Style(_))
    )));
    assert!(analysis.nodes_in_creation_order().iter().any(|node| matches!(
        node.kind(),
        HtmlTreeNodeKind::Element(element)
            if matches!(element.name(), HtmlElementName::SelectedOrdinary(_))
    )));
}

struct PartsFixture {
    source: SourceText,
    run: HtmlTokenizerRunResult,
    parts: HtmlDocumentShellParts,
}

fn coordinated_parts(source_text: &str) -> PartsFixture {
    coordinated_parts_with(source_text, limits())
}

fn coordinated_parts_with(source_text: &str, limits: HtmlTokenizerLimits) -> PartsFixture {
    let source = SourceText::new(SourceId::new(401), source_text.to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&source, limits);
    let mut session = HtmlTreeSession::new().expect("session");
    let mut next_token = 0usize;
    let mut entry_tokens = Vec::new();
    let mut close_tokens = Vec::new();
    let mut stopped = false;

    let run = 'produce: loop {
        let boundary = tokenizer.drive_to_boundary();
        let produced = tokenizer.tokens().len();

        while next_token < produced {
            let token = &tokenizer.tokens()[next_token];
            let admitted = admit(token).expect("valid fixture admission");
            let trigger = token_trigger(token, next_token);
            let mut evaluated = Vec::<InsertionMode>::new();

            loop {
                let mode = session.insertion_mode();
                assert!(!evaluated.contains(&mode), "same-token insertion-mode cycle");
                evaluated.push(mode);
                match session.dispatch(&admitted, &trigger).expect("dispatch") {
                    DispatchOutcome::Consumed => {
                        next_token += 1;
                        break;
                    }
                    DispatchOutcome::ReprocessSameToken => {}
                    DispatchOutcome::StoppedParsing => {
                        next_token += 1;
                        stopped = true;
                        break;
                    }
                    DispatchOutcome::Unsupported(capability) => {
                        panic!("fixture unexpectedly unsupported: {capability:?}")
                    }
                    DispatchOutcome::TokenizerFeedbackRequested(feedback) => {
                        assert_eq!(feedback, HtmlTreeTokenizerFeedback::EnterRawText);
                        assert_eq!(next_token + 1, produced);
                        assert_eq!(
                            boundary,
                            HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::RawText)
                        );
                        tokenizer.apply_raw_text().expect("apply RAWTEXT");
                        entry_tokens.push(next_token);
                        session
                            .acknowledge_tokenizer_feedback(feedback)
                            .expect("ack feedback");
                        next_token += 1;
                        continue 'produce;
                    }
                }
            }
            if stopped {
                break;
            }
        }

        if stopped {
            break 'produce tokenizer.finish_batch_compatible();
        }

        match boundary {
            HtmlTokenizerSessionBoundary::TokenAvailable => {
                close_tokens.push(produced.checked_sub(1).expect("close token"));
            }
            HtmlTokenizerSessionBoundary::Suspended(mode) => {
                panic!("fixture suspension without tree feedback: {mode:?}")
            }
            HtmlTokenizerSessionBoundary::Terminal => {
                break 'produce tokenizer.into_result().expect("terminal result")
            }
        }
    };

    let completion = if matches!(run.completion(), HtmlTokenizerCompletion::Complete) && stopped {
        HtmlTreeCompletion::Complete
    } else {
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    };
    let mut parts = session.finish(completion);
    parts.coordinated_raw_text_entry_tokens = entry_tokens;
    parts.coordinated_raw_text_close_tokens = close_tokens;
    PartsFixture { source, run, parts }
}

fn freeze_fixture(fixture: &PartsFixture, parts: HtmlDocumentShellParts) -> Result<HtmlDocumentShellAnalysis, HtmlTreeFreezeError> {
    freeze(&fixture.source, fixture.run.clone(), parts)
}

fn style_node_id(parts: &HtmlDocumentShellParts) -> HtmlConstructedNodeId {
    parts
        .actions
        .iter()
        .find_map(|action| match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredStyleElement { node } => Some(*node),
            _ => None,
        })
        .expect("fixture Style")
}

fn action_index(parts: &HtmlDocumentShellParts, predicate: impl Fn(&HtmlTreeActionKind) -> bool) -> usize {
    parts
        .actions
        .iter()
        .position(|action| predicate(action.kind()))
        .expect("fixture action")
}

#[test]
fn p14_freeze_rejects_outstanding_feedback_and_missing_coordination() {
    let fixture = coordinated_parts("<style>x</style>");

    let mut pending = coordinated_parts("<style>x</style>").parts;
    pending.pending_tokenizer_feedback = true;
    assert!(matches!(
        freeze_fixture(&fixture, pending),
        Err(HtmlTreeFreezeError::OutstandingTokenizerFeedback)
    ));

    let mut missing_entry = coordinated_parts("<style>x</style>").parts;
    missing_entry.coordinated_raw_text_entry_tokens.clear();
    assert!(matches!(
        freeze_fixture(&fixture, missing_entry),
        Err(HtmlTreeFreezeError::StyleCoordinationEntryMismatch { .. })
    ));

    let mut missing_close = coordinated_parts("<style>x</style>").parts;
    missing_close.coordinated_raw_text_close_tokens.clear();
    assert!(matches!(
        freeze_fixture(&fixture, missing_close),
        Err(HtmlTreeFreezeError::StyleCoordinationCloseMismatch { .. })
    ));
}

#[test]
fn p14_freeze_rejects_wrong_terminal_tree_state_and_close_trigger() {
    let fixture = coordinated_parts("<style>x</style>");

    let mut text_mode = coordinated_parts("<style>x</style>").parts;
    text_mode.final_style_text_mode_active = true;
    assert!(matches!(
        freeze_fixture(&fixture, text_mode),
        Err(HtmlTreeFreezeError::FinalStyleStateMismatch)
    ));

    let mut retained_original = coordinated_parts("<style>x</style>").parts;
    retained_original.final_style_original_in_head_retained = true;
    assert!(matches!(
        freeze_fixture(&fixture, retained_original),
        Err(HtmlTreeFreezeError::FinalStyleStateMismatch)
    ));

    let mut claimed_open = coordinated_parts("<style>x</style>").parts;
    claimed_open.final_open_style = Some(style_node_id(&claimed_open));
    claimed_open.final_style_text_mode_active = true;
    claimed_open.final_style_original_in_head_retained = true;
    assert!(freeze_fixture(&fixture, claimed_open).is_err());

    let mut wrong_close = coordinated_parts("<style>x</style>").parts;
    let insert = action_index(&wrong_close, |kind| matches!(
        kind,
        HtmlTreeActionKind::InsertedAuthoredStyleElement { .. }
    ));
    let close = action_index(&wrong_close, |kind| matches!(
        kind,
        HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { .. }
    ));
    let style = style_node_id(&wrong_close);
    let trigger = wrong_close.actions[insert].trigger().clone();
    wrong_close.actions[close] = HtmlTreeAction::new(
        HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { node: style },
        trigger,
    );
    assert!(matches!(
        freeze_fixture(&fixture, wrong_close),
        Err(HtmlTreeFreezeError::StyleAuthoredCloseTriggerMismatch { .. })
            | Err(HtmlTreeFreezeError::InvalidTokenProgression { .. })
    ));
}

#[test]
fn p14_freeze_rejects_fabricated_eof_close_missing_eof_evidence_and_wrong_redispatch() {
    let fixture = coordinated_parts("<style>x");

    let mut fabricated_close = coordinated_parts("<style>x").parts;
    let pop = action_index(&fabricated_close, |kind| matches!(
        kind,
        HtmlTreeActionKind::PoppedStyleElementAtEndOfFile { .. }
    ));
    let style = style_node_id(&fabricated_close);
    let trigger = fabricated_close.actions[pop].trigger().clone();
    fabricated_close.actions[pop] = HtmlTreeAction::new(
        HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { node: style },
        trigger,
    );
    assert!(freeze_fixture(&fixture, fabricated_close).is_err());

    let mut missing_diagnostic = coordinated_parts("<style>x").parts;
    missing_diagnostic
        .diagnostics
        .retain(|diagnostic| diagnostic.code() != HtmlTreeDiagnosticCode::StyleEndOfFileInText);
    assert!(matches!(
        freeze_fixture(&fixture, missing_diagnostic),
        Err(HtmlTreeFreezeError::StyleEndOfFileDiagnosticMismatch { .. })
    ));

    let mut missing_reprocess = coordinated_parts("<style>x").parts;
    let eof = missing_reprocess.actions[pop].trigger().token_index();
    missing_reprocess.actions.retain(|action| {
        !(action.trigger().token_index() == eof
            && matches!(action.kind(), HtmlTreeActionKind::ReprocessedToken))
    });
    assert!(matches!(
        freeze_fixture(&fixture, missing_reprocess),
        Err(HtmlTreeFreezeError::StyleEndOfFileRedispatchMismatch { .. })
    ));
}

#[test]
fn p14_freeze_rejects_corrupt_rawtext_contribution_and_chronology() {
    let fixture = coordinated_parts("<style>x</style>");

    let mut duplicate_text = coordinated_parts("<style>x</style>").parts;
    let text_index = duplicate_text
        .nodes
        .iter()
        .position(|node| matches!(node.kind(), HtmlTreeNodeKind::Text(_)))
        .expect("text node");
    let old = duplicate_text.nodes[text_index].clone();
    let HtmlTreeNodeKind::Text(text) = old.kind() else {
        unreachable!()
    };
    let contribution = text.contributions()[0].clone();
    duplicate_text.nodes[text_index] = HtmlTreeNode::new(
        old.id(),
        old.parent(),
        old.children().to_vec(),
        HtmlTreeNodeKind::Text(super::result::HtmlTextNode::new(
            text.interpreted().to_owned(),
            vec![contribution.clone(), contribution],
        )),
    );
    assert!(matches!(
        freeze_fixture(&fixture, duplicate_text),
        Err(HtmlTreeFreezeError::InvalidTextContributions(_))
    ));

    let mut chronology = coordinated_parts("<style>x</style>").parts;
    let insert = action_index(&chronology, |kind| matches!(
        kind,
        HtmlTreeActionKind::InsertedAuthoredStyleElement { .. }
    ));
    let close = action_index(&chronology, |kind| matches!(
        kind,
        HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { .. }
    ));
    chronology.actions.swap(insert, close);
    assert!(freeze_fixture(&fixture, chronology).is_err());
}

#[test]
fn p14_complete_claim_cannot_upgrade_a_lower_layer_resource_stop() {
    let constrained = HtmlTokenizerLimits::new(4_096, 1, 4_096, 4_096, 256, 16_384, 4_096);
    let fixture = coordinated_parts_with("<body>abcdef", constrained);
    assert!(fixture.run.is_incomplete());
    let mut parts = fixture.parts;
    parts.completion = HtmlTreeCompletion::Complete;
    assert!(matches!(
        freeze(&fixture.source, fixture.run.clone(), parts),
        Err(HtmlTreeFreezeError::CompletionUpgrade(_))
    ));
}

#[test]
fn negative_space_has_no_general_rcdata_script_or_plaintext_entry() {
    for source in ["<title>x</title>", "<textarea>x</textarea>", "<script>x</script>"] {
        let analysis = analyze(source);
        assert_eq!(tree_unsupported(&analysis), Some(HtmlTreeCapability::NonShellElementTag));
        assert!(analysis.nodes_in_creation_order().iter().all(|node| !matches!(
            node.kind(),
            HtmlTreeNodeKind::Element(HtmlElement::Style(_))
        )));
    }
}
