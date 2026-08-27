//! Production correspondence for TC-S8 — selected InBody `</html>` over the
//! open bounded `Div | Section` stack with optional current P (Issue #382).
//!
//! Every expectation in this module is independently authored from the
//! accepted production theorem. It imports only production tokenizer,
//! driver/session, result, and freeze seams. It neither imports nor calls the
//! candidate-independent validation machine, and no expected value is
//! projected from that machine.

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;
use super::driver::{construct_html_document_shell, drive_token};
use super::result::{
    HtmlConstructedNodeId, HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlElementName,
    HtmlParagraphClosure, HtmlParagraphSynthesisCause, HtmlSelectedOrdinaryElementName,
    HtmlShellClosure, HtmlShellElementName, HtmlTreeAction, HtmlTreeActionKind, HtmlTreeCapability,
    HtmlTreeCompletion, HtmlTreeDiagnostic, HtmlTreeDiagnosticCode, HtmlTreeFreezeError,
    HtmlTreeIncompleteCause, HtmlTreeNodeKind, HtmlTreeRecovery, HtmlTreeTokenTrigger, freeze,
};
use super::session::{HtmlTreeSession, InsertionMode, TokenOutcome, admit, token_trigger};

type Span = (usize, usize);

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn analyze(source: &str) -> HtmlDocumentShellAnalysis {
    analyze_with(source, 1)
}

fn analyze_with(source: &str, source_id: u64) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(source_id), source.to_owned());
    construct_html_document_shell(&source, limits()).expect("TC-S8 production boundary")
}

fn span(anchor: &SourceAnchor) -> Span {
    (anchor.range().start(), anchor.range().end())
}

fn diagnostic_count(analysis: &HtmlDocumentShellAnalysis, code: HtmlTreeDiagnosticCode) -> usize {
    analysis
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code() == code)
        .count()
}

fn html_end_token(analysis: &HtmlDocumentShellAnalysis) -> usize {
    analysis
        .tokenizer_run()
        .tokens()
        .iter()
        .position(|token| {
            matches!(token, HtmlToken::Tag(tag)
                if tag.kind() == HtmlTagKind::End
                    && tag.name().interpreted() == "html"
                    && tag.attributes().is_empty()
                    && tag.self_closing_solidus().is_none())
        })
        .expect("plain Html end token")
}

fn html_phase_actions(
    analysis: &HtmlDocumentShellAnalysis,
    token_index: usize,
) -> Vec<&'static str> {
    analysis
        .actions()
        .iter()
        .filter(|action| action.trigger().token_index() == token_index)
        .map(|action| match action.kind() {
            HtmlTreeActionKind::ReprocessedToken => "reprocessed",
            HtmlTreeActionKind::AcknowledgedShellEndTag {
                name: HtmlShellElementName::Html,
            } => "acknowledged-html",
            _ => "other",
        })
        .collect()
}

fn selected_insertions(
    analysis: &HtmlDocumentShellAnalysis,
) -> Vec<(HtmlConstructedNodeId, HtmlSelectedOrdinaryElementName)> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { node, name } => {
                Some((*node, *name))
            }
            _ => None,
        })
        .collect()
}

fn paragraph_insertions(analysis: &HtmlDocumentShellAnalysis) -> Vec<HtmlConstructedNodeId> {
    analysis
        .actions()
        .iter()
        .filter_map(|action| match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredParagraphElement { node } => Some(*node),
            _ => None,
        })
        .collect()
}

fn semantic_signature(analysis: &HtmlDocumentShellAnalysis) -> Vec<String> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .map(|node| {
            format!(
                "{:?}|{:?}|{:?}|{:?}",
                node.id(),
                node.parent(),
                node.children(),
                node.kind()
            )
        })
        .collect()
}

fn forbidden_candidate_actions(
    analysis: &HtmlDocumentShellAnalysis,
    token_index: usize,
) -> Vec<&HtmlTreeActionKind> {
    analysis
        .actions()
        .iter()
        .filter(|action| {
            action.trigger().token_index() == token_index
                && !matches!(
                    action.kind(),
                    HtmlTreeActionKind::ReprocessedToken
                        | HtmlTreeActionKind::AcknowledgedShellEndTag {
                            name: HtmlShellElementName::Html,
                        }
                )
        })
        .map(HtmlTreeAction::kind)
        .collect()
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
        let outcome =
            drive_token(&mut session, &admitted, &trigger).expect("production dispatch invariant");
        outcomes.push(outcome);
        if !matches!(outcome, TokenOutcome::Consumed) {
            break;
        }
    }
    (session, outcomes)
}

struct FreezeFixture {
    source: SourceText,
    run: HtmlTokenizerRunResult,
}

impl FreezeFixture {
    fn new(source: &str) -> Self {
        let source = SourceText::new(SourceId::new(83), source.to_owned());
        let run = tokenize(&source, limits());
        Self { source, run }
    }

    fn anchor(&self, start: usize, end: usize) -> SourceAnchor {
        self.source.anchor(start, end).expect("fixture range")
    }

    fn tag_anchor(&self, token_index: usize) -> SourceAnchor {
        let HtmlToken::Tag(tag) = &self.run.tokens()[token_index] else {
            panic!("fixture token is a tag")
        };
        tag.complete().clone()
    }

    fn tag_trigger(&self, token_index: usize) -> HtmlTreeTokenTrigger {
        HtmlTreeTokenTrigger::authored(token_index, self.tag_anchor(token_index))
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
        let admitted = admit(token).expect("freeze fixture admission");
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

fn html_diagnostic(trigger: HtmlTreeTokenTrigger) -> HtmlTreeDiagnostic {
    HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements,
        trigger,
        HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements,
    )
}

fn html_reprocess_action_index(parts: &HtmlDocumentShellParts, token_index: usize) -> usize {
    parts
        .actions
        .iter()
        .position(|action| {
            action.trigger().token_index() == token_index
                && matches!(action.kind(), HtmlTreeActionKind::ReprocessedToken)
        })
        .expect("selected Html reprocess")
}

fn html_ack_action_index(parts: &HtmlDocumentShellParts, token_index: usize) -> usize {
    parts
        .actions
        .iter()
        .position(|action| {
            action.trigger().token_index() == token_index
                && matches!(
                    action.kind(),
                    HtmlTreeActionKind::AcknowledgedShellEndTag {
                        name: HtmlShellElementName::Html,
                    }
                )
        })
        .expect("Html acknowledgement")
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

fn body_id(parts: &HtmlDocumentShellParts) -> HtmlConstructedNodeId {
    parts
        .nodes
        .iter()
        .find_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(element)
                if element.name() == HtmlElementName::Shell(HtmlShellElementName::Body) =>
            {
                Some(node.id())
            }
            _ => None,
        })
        .expect("fixture Body")
}

#[test]
fn shell_p_selected_and_deep_heterogeneous_paths_pin_cardinality_and_identity() {
    for (without_end, with_end, selected_count, paragraph_count) in [
        ("<body>", "<body></html>", 0usize, 0usize),
        ("<body><p>", "<body><p></html>", 0, 1),
        ("<body><div>", "<body><div></html>", 1, 0),
        ("<body><section>", "<body><section></html>", 1, 0),
        (
            "<body><div><section><div>",
            "<body><div><section><div></html>",
            3,
            0,
        ),
        (
            "<body><section><div><section><div><p>",
            "<body><section><div><section><div><p></html>",
            4,
            1,
        ),
    ] {
        let control = FreezeFixture::new(without_end);
        let candidate = FreezeFixture::new(with_end);
        let control_parts = valid_parts(&control);
        let candidate_parts = valid_parts(&candidate);
        assert_eq!(
            control_parts.final_open_selected_ordinary,
            candidate_parts.final_open_selected_ordinary,
            "{with_end}: exact selected identities/order"
        );
        assert_eq!(
            control_parts.final_open_paragraph, candidate_parts.final_open_paragraph,
            "{with_end}: exact P identity"
        );
        assert_eq!(
            control_parts.admitted_creation_events, candidate_parts.admitted_creation_events,
            "{with_end}: candidate admits no identity"
        );
        assert_eq!(control_parts.nodes.len(), candidate_parts.nodes.len());

        let analysis = analyze(with_end);
        assert!(analysis.is_complete(), "{with_end}");
        assert_eq!(
            diagnostic_count(
                &analysis,
                HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements,
            ),
            usize::from(selected_count != 0),
            "{with_end}"
        );
        assert_eq!(selected_insertions(&analysis).len(), selected_count);
        assert_eq!(paragraph_insertions(&analysis).len(), paragraph_count);
        let token_index = html_end_token(&analysis);
        assert_eq!(
            html_phase_actions(&analysis, token_index),
            vec!["reprocessed", "acknowledged-html"],
            "{with_end}"
        );
        assert!(forbidden_candidate_actions(&analysis, token_index).is_empty());
        assert_eq!(analysis.coverage().committed_end(), with_end.len());
        assert_eq!(
            analysis.coverage().processed_tokens(),
            analysis.tokenizer_run().tokens().len()
        );
    }
}

#[test]
fn mixed_case_html_end_retains_semantics_index_source_range_raw_spelling_and_one_trigger() {
    let source = "<body><DiV><p>é</HtMl>";
    let analysis = analyze_with(source, 91);
    let token_index = html_end_token(&analysis);
    assert_eq!(token_index, 4);
    let HtmlToken::Tag(tag) = &analysis.tokenizer_run().tokens()[token_index] else {
        panic!("Html end tag")
    };
    assert_eq!(tag.kind(), HtmlTagKind::End);
    assert_eq!(tag.name().interpreted(), "html");
    assert_eq!(tag.complete().source_id(), SourceId::new(91));
    assert_eq!(span(tag.complete()), (16, 23));
    assert_eq!(tag.complete().fragment(), "</HtMl>");
    assert_eq!(tag.name().source().source_id(), SourceId::new(91));
    assert_eq!(span(tag.name().source()), (18, 22));
    assert_eq!(tag.name().source().fragment(), "HtMl");

    let diagnostic = analysis
        .diagnostics()
        .iter()
        .find(|diagnostic| {
            diagnostic.code() == HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements
        })
        .expect("dedicated Html diagnostic");
    assert_eq!(diagnostic.trigger().token_index(), token_index);
    assert_eq!(
        diagnostic.recovery(),
        HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements
    );
    let diagnostic_trigger = diagnostic
        .trigger()
        .authored_boundary()
        .expect("authored diagnostic trigger");
    assert_eq!(diagnostic_trigger.source_id(), tag.complete().source_id());
    assert_eq!(diagnostic_trigger.range(), tag.complete().range());
    assert_eq!(diagnostic_trigger.fragment(), tag.complete().fragment());

    let actions: Vec<&HtmlTreeAction> = analysis
        .actions()
        .iter()
        .filter(|action| action.trigger().token_index() == token_index)
        .collect();
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        actions[0].kind(),
        HtmlTreeActionKind::ReprocessedToken
    ));
    assert!(matches!(
        actions[1].kind(),
        HtmlTreeActionKind::AcknowledgedShellEndTag {
            name: HtmlShellElementName::Html,
        }
    ));
    for action in actions {
        let trigger = action
            .trigger()
            .authored_boundary()
            .expect("same retained authored token");
        assert_eq!(action.trigger().token_index(), token_index);
        assert_eq!(trigger.source_id(), SourceId::new(91));
        assert_eq!(trigger.source_id(), tag.complete().source_id());
        assert_eq!(trigger.range(), tag.complete().range());
        assert_eq!(trigger.fragment(), tag.complete().fragment());
    }
}

#[test]
fn direct_after_body_html_control_has_only_the_existing_acknowledgement() {
    let source = "<body><div></body></html>";
    let analysis = analyze(source);
    assert!(analysis.is_complete());
    assert_eq!(
        diagnostic_count(
            &analysis,
            HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements,
        ),
        0
    );
    let token_index = html_end_token(&analysis);
    assert_eq!(token_index, 3);
    assert_eq!(
        html_phase_actions(&analysis, token_index),
        vec!["acknowledged-html"]
    );
    assert_eq!(
        diagnostic_count(
            &analysis,
            HtmlTreeDiagnosticCode::BodyEndTagWithOpenSelectedOrdinaryElements,
        ),
        1
    );
}

#[test]
fn selected_html_end_has_zero_lifecycle_recovery_synthesis_text_and_identity_delta() {
    for (without_end, with_end) in [
        ("<body><p>", "<body><p></html>"),
        ("<body><div>", "<body><div></html>"),
        (
            "<body><div><section><div><p>",
            "<body><div><section><div><p></html>",
        ),
    ] {
        let control = analyze(without_end);
        let candidate = analyze(with_end);
        assert_eq!(semantic_signature(&control), semantic_signature(&candidate));
        assert_eq!(
            selected_insertions(&control),
            selected_insertions(&candidate)
        );
        assert_eq!(
            paragraph_insertions(&control),
            paragraph_insertions(&candidate)
        );
        let token_index = html_end_token(&candidate);
        assert!(forbidden_candidate_actions(&candidate, token_index).is_empty());
        assert!(candidate.actions().iter().all(|action| {
            action.trigger().token_index() != token_index
                || !matches!(
                    action.kind(),
                    HtmlTreeActionKind::ClosedSelectedOrdinaryElement { .. }
                        | HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag { .. }
                        | HtmlTreeActionKind::IgnoredUnmatchedSelectedOrdinaryEndTag { .. }
                        | HtmlTreeActionKind::ClosedParagraphElement { .. }
                        | HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag { .. }
                        | HtmlTreeActionKind::InsertedSynthesizedParagraphElement { .. }
                        | HtmlTreeActionKind::ClosedShellElement { .. }
                        | HtmlTreeActionKind::InsertedTextNode { .. }
                        | HtmlTreeActionKind::AppendedToTextNode { .. }
                )
        }));
    }
}

#[test]
fn after_after_body_eof_retains_stack_and_never_fabricates_in_body_eof_diagnostic() {
    for source in [
        "<body></html>",
        "<body><p></html>",
        "<body><div></html>",
        "<body><div><section><p></html>",
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
        assert_eq!(session.insertion_mode(), InsertionMode::AfterAfterBody);
        assert!(matches!(
            outcomes.last(),
            Some(TokenOutcome::StoppedParsing)
        ));
    }
}

#[test]
fn attributes_and_self_closing_html_end_are_refused_transactionally() {
    for (source, capability) in [
        (
            "<body><div><p></html id=x>",
            HtmlTreeCapability::ShellTagAttribute,
        ),
        (
            "<body><div><p></html/>",
            HtmlTreeCapability::SelfClosingShellTag,
        ),
    ] {
        let candidate_start = source.find("</html").expect("candidate start");
        let analysis = analyze(source);
        assert_eq!(
            unsupported_evidence(&analysis),
            Some((capability, 3, Some((candidate_start, source.len())))),
            "{source}"
        );
        assert_eq!(analysis.coverage().processed_tokens(), 3);
        assert_eq!(analysis.coverage().committed_end(), candidate_start);
        assert!(
            analysis
                .actions()
                .iter()
                .all(|action| action.trigger().token_index() != 3)
        );
        assert_eq!(
            semantic_signature(&analysis),
            semantic_signature(&analyze("<body><div><p>"))
        );
    }
}

#[test]
fn duplicate_body_start_and_unrelated_shell_firewalls_remain_closed() {
    for (source, capability, token, start) in [
        (
            "<body><div><body>",
            HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement,
            2usize,
            11usize,
        ),
        (
            "<body><p><body>",
            HtmlTreeCapability::ShellTagWithOpenParagraphElement,
            2,
            9,
        ),
        (
            "<body><section></head>",
            HtmlTreeCapability::ShellTagWithOpenSelectedOrdinaryElement,
            2,
            15,
        ),
    ] {
        let analysis = analyze(source);
        assert_eq!(
            unsupported_evidence(&analysis),
            Some((capability, token, Some((start, source.len())))),
            "{source}"
        );
        assert_eq!(analysis.coverage().committed_end(), start);
        assert_eq!(analysis.coverage().processed_tokens(), token);
    }
}

#[test]
fn committed_coverage_processed_tokens_and_later_identity_control_are_exact() {
    let supported_source = "<body><div></html>";
    let supported = analyze(supported_source);
    assert!(supported.is_complete());
    assert_eq!(supported.coverage().committed_end(), supported_source.len());
    assert_eq!(supported.coverage().processed_tokens(), 4);

    let later_source = "<body><div></html><section>";
    let later = analyze(later_source);
    assert_eq!(
        unsupported_evidence(&later),
        Some((
            HtmlTreeCapability::SelectedOrdinaryTagOutsideInBody,
            3,
            Some((18, later_source.len())),
        ))
    );
    assert_eq!(later.coverage().committed_end(), 18);
    assert_eq!(later.coverage().processed_tokens(), 3);
    assert_eq!(semantic_signature(&later), semantic_signature(&supported));
    assert_eq!(selected_insertions(&later), selected_insertions(&supported));
}

#[test]
fn source_id_and_private_storage_perturbations_preserve_semantics_not_evidence_binding() {
    let source = "<body><section><div><p></HtMl>";
    let first = analyze_with(source, 101);
    let second = analyze_with(source, 202);
    assert_eq!(first.node_count(), second.node_count());
    assert_eq!(selected_insertions(&first), selected_insertions(&second));
    assert_eq!(paragraph_insertions(&first), paragraph_insertions(&second));
    let first_token = html_end_token(&first);
    let second_token = html_end_token(&second);
    assert_eq!(first_token, second_token);
    for (analysis, expected) in [(&first, SourceId::new(101)), (&second, SourceId::new(202))] {
        for action in analysis
            .actions()
            .iter()
            .filter(|action| action.trigger().token_index() == first_token)
        {
            assert_eq!(
                action
                    .trigger()
                    .authored_boundary()
                    .expect("authored Html trigger")
                    .source_id(),
                expected
            );
        }
    }

    let fixture = FreezeFixture::new(source);
    let mut parts = valid_parts(&fixture);
    parts.nodes.reverse();
    let reversed = freeze_parts(&fixture, parts).expect("storage-independent freeze");
    let baseline = analyze_with(source, 83);
    assert_eq!(semantic_signature(&baseline), semantic_signature(&reversed));
    assert_eq!(
        semantic_signature(&baseline),
        semantic_signature(&baseline.clone().with_reversed_storage())
    );
}

#[test]
fn lower_layer_incompleteness_is_never_upgraded() {
    let source = SourceText::new(SourceId::new(1), "<body><div></html>x".to_owned());
    let analysis = construct_html_document_shell(
        &source,
        HtmlTokenizerLimits::new(1_024, 8_192, 3, 1_024, 256, 4_096, 1_024),
    )
    .expect("tree boundary");
    assert!(analysis.tokenizer_run().is_incomplete());
    assert!(!analysis.is_complete());
    assert!(matches!(
        analysis.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));
    assert_eq!(analysis.coverage().processed_tokens(), 3);
    assert_eq!(analysis.coverage().committed_end(), 18);
    assert_eq!(
        diagnostic_count(
            &analysis,
            HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements,
        ),
        1
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GeneratedBlock {
    Div,
    Section,
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

fn generated_source(blocks: &[GeneratedBlock], paragraph: bool) -> String {
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
    source.push_str("</html>");
    source
}

#[test]
fn generated_div_section_stacks_through_depth_four_times_optional_p_match_closed_form() {
    // Four is test coverage only. Production reads no depth value and gains no
    // resource dimension or normative stack limit from this generator.
    for blocks in generated_block_sequences(4) {
        for paragraph in [false, true] {
            let source = generated_source(&blocks, paragraph);
            let analysis = analyze_with(&source, 31);
            assert!(analysis.is_complete(), "{source}");
            assert_eq!(analysis.coverage().committed_end(), source.len());
            assert_eq!(
                analysis.coverage().processed_tokens(),
                analysis.tokenizer_run().tokens().len()
            );
            assert_eq!(
                analysis.node_count(),
                4 + blocks.len() + usize::from(paragraph)
            );
            assert_eq!(selected_insertions(&analysis).len(), blocks.len());
            assert_eq!(
                paragraph_insertions(&analysis).len(),
                usize::from(paragraph)
            );
            assert_eq!(
                diagnostic_count(
                    &analysis,
                    HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements,
                ),
                usize::from(!blocks.is_empty())
            );
            let token_index = html_end_token(&analysis);
            assert_eq!(
                html_phase_actions(&analysis, token_index),
                vec!["reprocessed", "acknowledged-html"]
            );
            assert!(forbidden_candidate_actions(&analysis, token_index).is_empty());
            let names: Vec<HtmlSelectedOrdinaryElementName> = selected_insertions(&analysis)
                .into_iter()
                .map(|(_, name)| name)
                .collect();
            assert_eq!(
                names,
                blocks
                    .iter()
                    .map(|block| match block {
                        GeneratedBlock::Div => HtmlSelectedOrdinaryElementName::Div,
                        GeneratedBlock::Section => HtmlSelectedOrdinaryElementName::Section,
                    })
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn freeze_rejects_missing_duplicate_and_wrong_selected_phase_actions() {
    let fixture = FreezeFixture::new("<body><div><section><p></html>");
    let html_token = 4;

    let mut missing_reprocess = valid_parts(&fixture);
    let index = html_reprocess_action_index(&missing_reprocess, html_token);
    missing_reprocess.actions.remove(index);
    assert!(matches!(
        freeze_parts(&fixture, missing_reprocess),
        Err(HtmlTreeFreezeError::MissingHtmlEndReprocess { token_index: 4 })
    ));

    let mut duplicate_reprocess = valid_parts(&fixture);
    let index = html_reprocess_action_index(&duplicate_reprocess, html_token);
    duplicate_reprocess.actions.insert(
        index + 1,
        HtmlTreeAction::new(
            HtmlTreeActionKind::ReprocessedToken,
            fixture.tag_trigger(html_token),
        ),
    );
    assert!(matches!(
        freeze_parts(&fixture, duplicate_reprocess),
        Err(HtmlTreeFreezeError::DuplicateHtmlEndReprocess { token_index: 4 })
    ));

    let mut wrong_token = valid_parts(&fixture);
    let index = html_reprocess_action_index(&wrong_token, html_token);
    wrong_token.actions[index] =
        HtmlTreeAction::new(HtmlTreeActionKind::ReprocessedToken, fixture.tag_trigger(3));
    assert!(matches!(
        freeze_parts(&fixture, wrong_token),
        Err(HtmlTreeFreezeError::HtmlEndReprocessTriggerMismatch { token_index: 3 })
    ));

    let mut wrong_range = valid_parts(&fixture);
    let index = html_reprocess_action_index(&wrong_range, html_token);
    wrong_range.actions[index] = HtmlTreeAction::new(
        HtmlTreeActionKind::ReprocessedToken,
        HtmlTreeTokenTrigger::authored(html_token, fixture.tag_anchor(3)),
    );
    assert!(matches!(
        freeze_parts(&fixture, wrong_range),
        Err(HtmlTreeFreezeError::HtmlEndReprocessTriggerMismatch { token_index: 4 })
    ));

    let mut missing_ack = valid_parts(&fixture);
    let index = html_ack_action_index(&missing_ack, html_token);
    missing_ack.actions.remove(index);
    assert!(matches!(
        freeze_parts(&fixture, missing_ack),
        Err(HtmlTreeFreezeError::MissingHtmlEndAcknowledgement { token_index: 4 })
    ));

    let mut forged_ack = valid_parts(&fixture);
    let index = html_ack_action_index(&forged_ack, html_token);
    forged_ack.actions[index] = HtmlTreeAction::new(
        HtmlTreeActionKind::AcknowledgedShellEndTag {
            name: HtmlShellElementName::Html,
        },
        HtmlTreeTokenTrigger::authored(html_token, fixture.tag_anchor(3)),
    );
    assert!(matches!(
        freeze_parts(&fixture, forged_ack),
        Err(HtmlTreeFreezeError::HtmlEndAcknowledgementTriggerMismatch { token_index: 4 })
    ));

    let mut duplicate_ack = valid_parts(&fixture);
    let index = html_ack_action_index(&duplicate_ack, html_token);
    duplicate_ack.actions.insert(
        index + 1,
        HtmlTreeAction::new(
            HtmlTreeActionKind::AcknowledgedShellEndTag {
                name: HtmlShellElementName::Html,
            },
            fixture.tag_trigger(html_token),
        ),
    );
    assert!(matches!(
        freeze_parts(&fixture, duplicate_ack),
        Err(HtmlTreeFreezeError::DuplicateHtmlEndAcknowledgement { token_index: 4 })
    ));
}

#[test]
fn freeze_rejects_html_diagnostic_cardinality_trigger_range_source_recovery_and_orphaning() {
    let selected = FreezeFixture::new("<body><div><section><p></html>");
    let html_token = 4;

    let mut missing = valid_parts(&selected);
    missing.diagnostics.retain(|diagnostic| {
        diagnostic.code() != HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements
    });
    assert!(matches!(
        freeze_parts(&selected, missing),
        Err(HtmlTreeFreezeError::HtmlEndDiagnosticCardinalityMismatch {
            token_index: 4,
            selected_open: 2,
            diagnostics: 0,
        })
    ));

    let mut duplicate = valid_parts(&selected);
    duplicate
        .diagnostics
        .push(html_diagnostic(selected.tag_trigger(html_token)));
    assert!(matches!(
        freeze_parts(&selected, duplicate),
        Err(HtmlTreeFreezeError::HtmlEndDiagnosticCardinalityMismatch {
            token_index: 4,
            selected_open: 2,
            diagnostics: 2,
        })
    ));

    let p_only = FreezeFixture::new("<body><p></html>");
    let mut unexpected = valid_parts(&p_only);
    unexpected
        .diagnostics
        .push(html_diagnostic(p_only.tag_trigger(2)));
    assert!(matches!(
        freeze_parts(&p_only, unexpected),
        Err(HtmlTreeFreezeError::HtmlEndDiagnosticCardinalityMismatch {
            token_index: 2,
            selected_open: 0,
            diagnostics: 1,
        })
    ));

    let diagnostic_index = valid_parts(&selected)
        .diagnostics
        .iter()
        .position(|diagnostic| {
            diagnostic.code() == HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements
        })
        .expect("TC-S8 diagnostic");

    let mut wrong_token = valid_parts(&selected);
    wrong_token.diagnostics[diagnostic_index] = html_diagnostic(selected.tag_trigger(3));
    assert!(matches!(
        freeze_parts(&selected, wrong_token),
        Err(HtmlTreeFreezeError::HtmlEndDiagnosticCardinalityMismatch { token_index: 4, .. })
    ));

    let mut wrong_range = valid_parts(&selected);
    wrong_range.diagnostics[diagnostic_index] = html_diagnostic(HtmlTreeTokenTrigger::authored(
        html_token,
        selected.tag_anchor(3),
    ));
    assert!(matches!(
        freeze_parts(&selected, wrong_range),
        Err(HtmlTreeFreezeError::HtmlEndDiagnosticTriggerOrRecoveryMismatch { token_index: 4 })
    ));

    let foreign_source = SourceText::new(
        SourceId::new(84),
        "<body><div><section><p></html>".to_owned(),
    );
    let mut wrong_source = valid_parts(&selected);
    wrong_source.diagnostics[diagnostic_index] = html_diagnostic(HtmlTreeTokenTrigger::authored(
        html_token,
        foreign_source
            .anchor(23, 30)
            .expect("foreign Html-end range"),
    ));
    assert!(matches!(
        freeze_parts(&selected, wrong_source),
        Err(HtmlTreeFreezeError::HtmlEndDiagnosticTriggerOrRecoveryMismatch { token_index: 4 })
    ));

    let mut wrong_recovery = valid_parts(&selected);
    wrong_recovery.diagnostics[diagnostic_index] = HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements,
        selected.tag_trigger(html_token),
        HtmlTreeRecovery::IgnoredToken,
    );
    assert!(matches!(
        freeze_parts(&selected, wrong_recovery),
        Err(HtmlTreeFreezeError::HtmlEndDiagnosticTriggerOrRecoveryMismatch { token_index: 4 })
    ));

    let direct = FreezeFixture::new("<body><div></body></html>");
    let mut orphan = valid_parts(&direct);
    orphan
        .diagnostics
        .push(html_diagnostic(direct.tag_trigger(3)));
    assert!(matches!(
        freeze_parts(&direct, orphan),
        Err(HtmlTreeFreezeError::OrphanHtmlEndDiagnostic { token_index: 3 })
    ));
}

#[test]
fn freeze_rejects_every_same_trigger_lifecycle_mutation_and_identity_admission() {
    let fixture = FreezeFixture::new("<body><div><section><p></html>");
    let baseline = valid_parts(&fixture);
    let selected = inserted_selected_ids(&baseline);
    let paragraph = inserted_paragraph_id(&baseline);
    let body = body_id(&baseline);
    let html_token = 4;

    let corruptions = [
        HtmlTreeActionKind::ClosedParagraphElement {
            node: paragraph,
            closure: HtmlParagraphClosure::MatchingEndTag,
        },
        HtmlTreeActionKind::PoppedParagraphElementBySelectedOrdinaryEndTag {
            node: paragraph,
            target: selected[1],
        },
        HtmlTreeActionKind::InsertedSynthesizedParagraphElement {
            node: paragraph,
            cause: HtmlParagraphSynthesisCause::UnmatchedParagraphEndTag,
        },
        HtmlTreeActionKind::ClosedSelectedOrdinaryElement {
            node: selected[1],
            name: HtmlSelectedOrdinaryElementName::Section,
        },
        HtmlTreeActionKind::PoppedSelectedOrdinaryElementByAncestorEndTag {
            node: selected[1],
            target: selected[0],
        },
        HtmlTreeActionKind::ClosedShellElement {
            node: body,
            name: HtmlShellElementName::Body,
            closure: HtmlShellClosure::AuthoredEndTag,
        },
        HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
            node: selected[1],
            name: HtmlSelectedOrdinaryElementName::Section,
        },
        HtmlTreeActionKind::InsertedTextNode { node: paragraph },
        HtmlTreeActionKind::AppendedToTextNode { node: paragraph },
    ];
    for corruption in corruptions {
        let mut parts = valid_parts(&fixture);
        let reprocess = html_reprocess_action_index(&parts, html_token);
        parts.actions.insert(
            reprocess + 1,
            HtmlTreeAction::new(corruption, fixture.tag_trigger(html_token)),
        );
        assert!(freeze_parts(&fixture, parts).is_err());
    }

    for corruption in [
        HtmlTreeActionKind::ClosedShellElement {
            node: body,
            name: HtmlShellElementName::Body,
            closure: HtmlShellClosure::AuthoredEndTag,
        },
        HtmlTreeActionKind::InsertedTextNode { node: paragraph },
    ] {
        let mut parts = valid_parts(&fixture);
        let reprocess = html_reprocess_action_index(&parts, html_token);
        parts.actions.insert(
            reprocess,
            HtmlTreeAction::new(corruption, fixture.tag_trigger(html_token)),
        );
        assert!(matches!(
            freeze_parts(&fixture, parts),
            Err(HtmlTreeFreezeError::HtmlEndSameTriggerMutation { token_index: 4 })
        ));
    }

    let mut admitted_identity = valid_parts(&fixture);
    admitted_identity.admitted_creation_events += 1;
    assert!(matches!(
        freeze_parts(&fixture, admitted_identity),
        Err(HtmlTreeFreezeError::CreationEventInventoryMismatch { .. })
    ));
}

#[test]
fn freeze_rejects_final_open_identity_and_after_after_eof_diagnostic_corruption() {
    let fixture = FreezeFixture::new("<body><div><section><p></html>");

    let mut selected_checkpoint = valid_parts(&fixture);
    selected_checkpoint.final_open_selected_ordinary.remove(0);
    assert!(freeze_parts(&fixture, selected_checkpoint).is_err());

    let mut paragraph_checkpoint = valid_parts(&fixture);
    paragraph_checkpoint.final_open_paragraph = None;
    assert!(freeze_parts(&fixture, paragraph_checkpoint).is_err());

    let mut eof_diagnostic = valid_parts(&fixture);
    eof_diagnostic.diagnostics.push(HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::OpenSelectedOrdinaryElementAtEndOfFile,
        fixture.eof_trigger(),
        HtmlTreeRecovery::StoppedParsingWithOpenSelectedOrdinaryElements,
    ));
    assert!(matches!(
        freeze_parts(&fixture, eof_diagnostic),
        Err(HtmlTreeFreezeError::BodyEndAfterBodyEofDiagnosticMismatch { token_index: 5 })
    ));
}

#[test]
fn predecessor_construction_reprocesses_are_not_naively_counted_as_tc_s8() {
    let source = "</html>";
    let analysis = analyze(source);
    assert!(analysis.is_complete());
    let token_index = html_end_token(&analysis);
    let reprocesses = analysis
        .actions()
        .iter()
        .filter(|action| {
            action.trigger().token_index() == token_index
                && matches!(action.kind(), HtmlTreeActionKind::ReprocessedToken)
        })
        .count();
    assert!(
        reprocesses > 1,
        "predecessor shell construction reprocesses exist"
    );
    assert_eq!(
        analysis
            .actions()
            .iter()
            .filter(|action| {
                action.trigger().token_index() == token_index
                    && matches!(
                        action.kind(),
                        HtmlTreeActionKind::AcknowledgedShellEndTag {
                            name: HtmlShellElementName::Html,
                        }
                    )
            })
            .count(),
        1
    );
    assert_eq!(
        diagnostic_count(
            &analysis,
            HtmlTreeDiagnosticCode::HtmlEndTagWithOpenSelectedOrdinaryElements,
        ),
        0
    );
}

#[test]
fn production_correspondence_has_no_candidate_machine_dependency() {
    let source = include_str!("in_body_html_end_open_stack_successor_production.rs");
    let validation_module = ["in_body_html_end_open_stack_successor_", "validation"].concat();
    assert!(
        !source
            .lines()
            .any(|line| line.trim_start().starts_with("use ") && line.contains(&validation_module))
    );
    assert!(
        !source
            .lines()
            .any(|line| line.contains("include_str!") && line.contains(&validation_module))
    );
}
