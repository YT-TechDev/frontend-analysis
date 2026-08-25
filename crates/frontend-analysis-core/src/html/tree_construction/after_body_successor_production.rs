//! TC-S2 selected after-body uniform character-run production correspondence.
//!
//! Focused production-conformance coverage for the accepted TC-S2 theorem
//! (Issue #355): `AfterBody` handling of one tokenizer-emitted aggregate
//! interpreted character run, partitioned into `AllHtmlWhitespace`,
//! `AllNonHtmlWhitespace`, and `Mixed`.
//!
//! Expectations here are hand-authored from the accepted theorem, not
//! generated from production output, and this module imports nothing from
//! [`super::after_body_successor_validation`]: that module remains
//! independent sibling evidence, never a production helper or an oracle for
//! these expectations. AB1-AB8 fixture bytes match the canonical byte
//! authority recorded there and in Issue #353.

use crate::{SourceId, SourceText};

use super::super::token::HtmlToken;
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::driver::{construct_html_document_shell, drive_token};
use super::result::{
    HtmlDocumentShellAnalysis, HtmlTreeActionKind, HtmlTreeCapability, HtmlTreeCompletion,
    HtmlTreeDiagnosticCode, HtmlTreeIncompleteCause, HtmlTreeNodeKind, HtmlTreeRecovery,
};
use super::session::{HtmlTreeSession, InsertionMode, TokenOutcome, admit, token_trigger};

/// A trigger's token index and its exact authored boundary range, when it has
/// one.
type TriggerEvidence = (usize, Option<(usize, usize)>);

fn generous_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn analyze(source_text: &str) -> HtmlDocumentShellAnalysis {
    analyze_with_id(source_text, 1)
}

fn analyze_with_id(source_text: &str, source_id: u64) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(source_id), source_text.to_owned());
    construct_html_document_shell(&source, generous_limits()).expect("no boundary failure")
}

/// Drives `source_text` token by token without freezing, so tests can inspect
/// the actual insertion mode a frozen [`HtmlDocumentShellAnalysis`] never
/// exposes.
fn drive_tokens(source_text: &str) -> (HtmlTreeSession, Vec<TokenOutcome>) {
    let source = SourceText::new(SourceId::new(1), source_text.to_owned());
    let run = tokenize(&source, generous_limits());
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
        let outcome = drive_token(&mut session, &admitted, &trigger).expect("no invariant failure");
        let stop = !matches!(outcome, TokenOutcome::Consumed);
        outcomes.push(outcome);
        if stop {
            break;
        }
    }
    (session, outcomes)
}

/// The exact unsupported evidence: capability, trigger token index, and
/// trigger range. A refusal's whole-token trigger is load-bearing for the
/// no-splitting/refuse-before-mutate theorem, so callers must not settle for
/// the capability alone.
fn unsupported_evidence(
    analysis: &HtmlDocumentShellAnalysis,
) -> Option<(HtmlTreeCapability, TriggerEvidence)> {
    match analysis.completion() {
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => Some((
            unsupported.capability(),
            (
                unsupported.trigger().token_index(),
                unsupported
                    .trigger()
                    .authored_boundary()
                    .map(|anchor| (anchor.range().start(), anchor.range().end())),
            ),
        )),
        _ => None,
    }
}

fn text_node_data(analysis: &HtmlDocumentShellAnalysis) -> Option<(String, Vec<(usize, usize)>)> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .find_map(|node| match node.kind() {
            HtmlTreeNodeKind::Text(text) => Some((
                text.interpreted().to_owned(),
                text.contributions()
                    .iter()
                    .map(|contribution| {
                        (
                            contribution.source().range().start(),
                            contribution.source().range().end(),
                        )
                    })
                    .collect(),
            )),
            _ => None,
        })
}

fn text_node_count(analysis: &HtmlDocumentShellAnalysis) -> usize {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter(|node| matches!(node.kind(), HtmlTreeNodeKind::Text(_)))
        .count()
}

/// Exact diagnostic evidence for one code: trigger token index, trigger
/// range, and the recorded recovery. #355 requires diagnostic code, recovery,
/// token index, and exact trigger range together — a wrong recovery value
/// must not pass by checking only the code and the trigger.
fn diagnostic_evidence(
    analysis: &HtmlDocumentShellAnalysis,
    code: HtmlTreeDiagnosticCode,
) -> Vec<(TriggerEvidence, HtmlTreeRecovery)> {
    analysis
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code() == code)
        .map(|diagnostic| {
            (
                (
                    diagnostic.trigger().token_index(),
                    diagnostic
                        .trigger()
                        .authored_boundary()
                        .map(|anchor| (anchor.range().start(), anchor.range().end())),
                ),
                diagnostic.recovery(),
            )
        })
        .collect()
}

fn reprocess_triggers(analysis: &HtmlDocumentShellAnalysis) -> Vec<TriggerEvidence> {
    analysis
        .actions()
        .iter()
        .filter(|action| matches!(action.kind(), HtmlTreeActionKind::ReprocessedToken))
        .map(|action| {
            (
                action.trigger().token_index(),
                action
                    .trigger()
                    .authored_boundary()
                    .map(|anchor| (anchor.range().start(), anchor.range().end())),
            )
        })
        .collect()
}

/// The exact ordered `ReprocessedToken` trigger ranges recorded for one
/// token. #355 requires exact reprocess-action trigger indexes/ranges, not
/// only a cardinality count, so an empty or wrong-range action must not pass
/// a count-only check.
fn reprocess_trigger_ranges_for_token(
    analysis: &HtmlDocumentShellAnalysis,
    token_index: usize,
) -> Vec<Option<(usize, usize)>> {
    reprocess_triggers(analysis)
        .into_iter()
        .filter(|(index, _)| *index == token_index)
        .map(|(_, range)| range)
        .collect()
}

fn character_token_shape(
    analysis: &HtmlDocumentShellAnalysis,
    token_index: usize,
) -> (String, (usize, usize)) {
    let HtmlToken::Character(character) = &analysis.tokenizer_run().tokens()[token_index] else {
        panic!("token {token_index} is not a character token")
    };
    (
        character.interpreted().to_owned(),
        (
            character.source().range().start(),
            character.source().range().end(),
        ),
    )
}

/// A minimal semantic-creation signature: for each node in committed creation
/// order, its kind and content. Deliberately contains no raw identity
/// encoding, so cross-run and cross-`SourceId` comparisons stay meaningful.
fn creation_signature(analysis: &HtmlDocumentShellAnalysis) -> Vec<String> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .map(|node| match node.kind() {
            HtmlTreeNodeKind::Document => "Document".to_owned(),
            HtmlTreeNodeKind::Element(element) => format!("Element({:?})", element.name()),
            HtmlTreeNodeKind::Text(text) => format!("Text({:?})", text.interpreted()),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// AB1: AfterBody + AllHtmlWhitespace
// ---------------------------------------------------------------------------

#[test]
fn ab1_all_html_whitespace_after_body_delegates_without_mode_change_or_reprocess() {
    let analysis = analyze("<body></body> ");

    assert_eq!(
        text_node_data(&analysis),
        Some((" ".to_owned(), vec![(13, 14)]))
    );
    assert!(
        diagnostic_evidence(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData).is_empty()
    );
    assert!(reprocess_trigger_ranges_for_token(&analysis, 2).is_empty());
    assert!(analysis.is_complete());
    assert_eq!(analysis.coverage().committed_end(), 14);
    // The shell (4) plus exactly one text node: no extra identity admitted.
    assert_eq!(analysis.node_count(), 5);

    // The actual insertion mode stays `AfterBody`: a frozen analysis never
    // exposes it, so this drives the raw session directly.
    let (session, outcomes) = drive_tokens("<body></body> ");
    assert!(matches!(
        outcomes.last(),
        Some(TokenOutcome::StoppedParsing)
    ));
    assert_eq!(session.insertion_mode(), InsertionMode::AfterBody);
}

// ---------------------------------------------------------------------------
// AB2: AfterBody + AllNonHtmlWhitespace
// ---------------------------------------------------------------------------

#[test]
fn ab2_all_non_html_whitespace_after_body_recovers_into_in_body() {
    let analysis = analyze("<body></body>x");

    assert_eq!(
        text_node_data(&analysis),
        Some(("x".to_owned(), vec![(13, 14)]))
    );
    assert_eq!(
        diagnostic_evidence(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData),
        vec![(
            (2, Some((13, 14))),
            HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken
        )]
    );
    assert_eq!(
        reprocess_trigger_ranges_for_token(&analysis, 2),
        vec![Some((13, 14))]
    );
    assert!(analysis.is_complete());
    assert_eq!(analysis.coverage().committed_end(), 14);
    assert_eq!(analysis.node_count(), 5);

    // The accepted recovery transition itself: the actual insertion mode
    // moves `AfterBody -> InBody` for this token, which a frozen analysis
    // never exposes.
    let (session, outcomes) = drive_tokens("<body></body>x");
    assert!(matches!(
        outcomes.last(),
        Some(TokenOutcome::StoppedParsing)
    ));
    assert_eq!(session.insertion_mode(), InsertionMode::InBody);
}

// ---------------------------------------------------------------------------
// AB3: coalescing across an action-only `</body>`
// ---------------------------------------------------------------------------

#[test]
fn ab3_recovered_text_coalesces_across_an_action_only_end_tag() {
    let analysis = analyze("<body>a</body>b");

    assert_eq!(
        text_node_data(&analysis),
        Some(("ab".to_owned(), vec![(6, 7), (14, 15)]))
    );
    assert_eq!(text_node_count(&analysis), 1);
    assert_eq!(
        diagnostic_evidence(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData),
        vec![(
            (3, Some((14, 15))),
            HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken
        )]
    );
    assert_eq!(
        reprocess_trigger_ranges_for_token(&analysis, 3),
        vec![Some((14, 15))]
    );
    assert!(analysis.is_complete());
    assert_eq!(analysis.coverage().committed_end(), 15);
    // Shell (4) plus exactly one text node identity, even though the text
    // node received two ordered contributions.
    assert_eq!(analysis.node_count(), 5);
}

// ---------------------------------------------------------------------------
// AB4: two bounded recovery cycles across two different character tokens
// ---------------------------------------------------------------------------

#[test]
fn ab4_two_distinct_character_tokens_each_recover_without_a_same_token_cycle() {
    let analysis = analyze("<body></body>x</body>y");

    assert_eq!(
        text_node_data(&analysis),
        Some(("xy".to_owned(), vec![(13, 14), (21, 22)]))
    );
    assert_eq!(text_node_count(&analysis), 1);
    assert_eq!(
        diagnostic_evidence(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData),
        vec![
            (
                (2, Some((13, 14))),
                HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken
            ),
            (
                (4, Some((21, 22))),
                HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken
            )
        ]
    );
    // Each recovery belongs to its own token: exactly one reprocess, at the
    // exact expected trigger range, per token, never two for the same token.
    assert_eq!(
        reprocess_trigger_ranges_for_token(&analysis, 2),
        vec![Some((13, 14))]
    );
    assert_eq!(
        reprocess_trigger_ranges_for_token(&analysis, 4),
        vec![Some((21, 22))]
    );
    assert!(analysis.is_complete());
    assert_eq!(analysis.coverage().committed_end(), 22);
    assert_eq!(analysis.node_count(), 5);
}

// ---------------------------------------------------------------------------
// AB5: one aggregate multi-character whitespace run
// ---------------------------------------------------------------------------

#[test]
fn ab5_aggregate_whitespace_run_delegates_as_one_observation() {
    // The tokenizer emits one aggregate Character token for the run, not one
    // token per character.
    let analysis = analyze("<body></body> \t");
    assert_eq!(
        character_token_shape(&analysis, 2),
        (" \t".to_owned(), (13, 15))
    );

    assert_eq!(
        text_node_data(&analysis),
        Some((" \t".to_owned(), vec![(13, 15)]))
    );
    assert!(
        diagnostic_evidence(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData).is_empty()
    );
    assert!(reprocess_trigger_ranges_for_token(&analysis, 2).is_empty());
    assert!(analysis.is_complete());
    assert_eq!(analysis.coverage().committed_end(), 15);
    assert_eq!(analysis.node_count(), 5);

    let (session, _) = drive_tokens("<body></body> \t");
    assert_eq!(session.insertion_mode(), InsertionMode::AfterBody);
}

// ---------------------------------------------------------------------------
// AB6: mixed aggregate run refused whole, before mutation
// ---------------------------------------------------------------------------

#[test]
fn ab6_mixed_aggregate_run_is_refused_whole_before_mutation() {
    let analysis = analyze("<body></body> x");
    assert_eq!(
        character_token_shape(&analysis, 2),
        (" x".to_owned(), (13, 15))
    );

    // The unsupported trigger is the whole aggregate token: token index 2,
    // range [13,15) — never a split prefix or a fabricated sub-anchor.
    assert_eq!(
        unsupported_evidence(&analysis),
        Some((
            HtmlTreeCapability::WhitespaceSensitiveCharacterData,
            (2, Some((13, 15)))
        ))
    );
    assert!(!analysis.is_complete());
    // Refused before mutation: no text, no contribution, no identity beyond
    // the shell, and committed coverage stops at the previous token.
    assert!(text_node_data(&analysis).is_none());
    assert_eq!(analysis.node_count(), 4);
    assert_eq!(analysis.coverage().committed_end(), 13);
    assert!(
        diagnostic_evidence(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData).is_empty()
    );
    assert!(reprocess_trigger_ranges_for_token(&analysis, 2).is_empty());

    // The actual insertion mode never changed: the refusal happened before
    // any mutation, including the mode.
    let (session, outcomes) = drive_tokens("<body></body> x");
    assert!(matches!(
        outcomes.last(),
        Some(TokenOutcome::Unsupported(
            HtmlTreeCapability::WhitespaceSensitiveCharacterData
        ))
    ));
    assert_eq!(session.insertion_mode(), InsertionMode::AfterBody);
}

// ---------------------------------------------------------------------------
// AB7: AfterAfterBody character data remains unsupported (negative pin)
// ---------------------------------------------------------------------------

#[test]
fn ab7_after_after_body_character_data_remains_unsupported() {
    let analysis = analyze("<body></body></html>x");

    assert_eq!(
        character_token_shape(&analysis, 3),
        ("x".to_owned(), (20, 21))
    );
    assert_eq!(
        unsupported_evidence(&analysis),
        Some((
            HtmlTreeCapability::UnprovedCharacterDataPosition,
            (3, Some((20, 21)))
        ))
    );
    assert!(!analysis.is_complete());
    assert!(text_node_data(&analysis).is_none());
    assert_eq!(analysis.node_count(), 4);
    assert_eq!(analysis.coverage().committed_end(), 20);
    assert!(
        diagnostic_evidence(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData).is_empty()
    );
    assert!(reprocess_trigger_ranges_for_token(&analysis, 3).is_empty());

    let (session, _) = drive_tokens("<body></body></html>x");
    assert_eq!(session.insertion_mode(), InsertionMode::AfterAfterBody);
}

// ---------------------------------------------------------------------------
// AB8: one aggregate non-whitespace run is one recovery unit
// ---------------------------------------------------------------------------

#[test]
fn ab8_aggregate_non_whitespace_run_is_one_recovery_unit_not_per_character() {
    let analysis = analyze("<body></body>xy");
    assert_eq!(
        character_token_shape(&analysis, 2),
        ("xy".to_owned(), (13, 15))
    );

    assert_eq!(
        text_node_data(&analysis),
        Some(("xy".to_owned(), vec![(13, 15)]))
    );
    // Exactly one diagnostic and one reprocess for the whole two-character
    // run: no per-character multiplication.
    assert_eq!(
        diagnostic_evidence(&analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData),
        vec![(
            (2, Some((13, 15))),
            HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken
        )]
    );
    assert_eq!(
        reprocess_trigger_ranges_for_token(&analysis, 2),
        vec![Some((13, 15))]
    );
    assert!(analysis.is_complete());
    assert_eq!(analysis.coverage().committed_end(), 15);
    assert_eq!(analysis.node_count(), 5);
}

// ---------------------------------------------------------------------------
// Required additional production properties
// ---------------------------------------------------------------------------

#[test]
fn semantic_correspondence_is_deterministic_across_repeats_and_source_ids() {
    for source_text in [
        "<body></body> ",
        "<body></body>x",
        "<body>a</body>b",
        "<body></body>x</body>y",
        "<body></body> \t",
        "<body></body>xy",
    ] {
        let baseline = analyze_with_id(source_text, 1);
        let baseline_signature = creation_signature(&baseline);
        let baseline_coverage = baseline.coverage().committed_end();
        for source_id in [1u64, 1u64, 7u64] {
            let repeat = analyze_with_id(source_text, source_id);
            assert_eq!(
                creation_signature(&repeat),
                baseline_signature,
                "{source_text:?}: semantic creation correspondence changed"
            );
            assert_eq!(repeat.coverage().committed_end(), baseline_coverage);
            assert!(repeat.is_complete());
        }
    }
}

#[test]
fn lower_layer_incompleteness_is_never_upgraded_for_a_recovered_run() {
    let source = SourceText::new(SourceId::new(1), "<body></body>x".to_owned());
    let tiny = HtmlTokenizerLimits::new(1_024, 8_192, 3, 1_024, 256, 4_096, 1_024);
    let truncated = construct_html_document_shell(&source, tiny).expect("no boundary failure");
    assert!(truncated.tokenizer_run().is_incomplete());
    assert!(!truncated.is_complete());
    assert!(matches!(
        truncated.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));
}
