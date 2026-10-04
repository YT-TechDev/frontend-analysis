//! Production correspondence for selected AfterAfterBody uniform
//! character-run handling (Issue #888).
//!
//! Expectations here are authored by hand from the accepted theorem (#348,
//! #886 and #888): offsets, token indexes, interpreted values, diagnostics and
//! recoveries are written out literally. They are never captured from
//! production output, and this module neither imports nor calls the
//! candidate-independent machine in the sibling
//! `after_after_body_character_successor_validation` module.
//!
//! Every source below begins with a complete shell prefix. `<body></body></html>`
//! occupies bytes `[0, 20)` as tokens 0..=2, so the character run under test
//! starts at byte 20 as token 3.

use crate::{SourceId, SourceText};

use super::super::token::HtmlToken;
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::driver::{construct_html_document_shell, drive_token};
use super::result::{
    HtmlConstructedNodeId, HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlTreeAction,
    HtmlTreeActionKind, HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeDiagnostic,
    HtmlTreeDiagnosticCode, HtmlTreeFreezeError, HtmlTreeIncompleteCause, HtmlTreeNode,
    HtmlTreeNodeKind, HtmlTreeRecovery, HtmlTreeTokenTrigger, freeze,
};
use super::session::{HtmlTreeSession, InsertionMode, TokenOutcome, admit, token_trigger};

type Range = (usize, usize);
type Trigger = (usize, Option<Range>);

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), text.to_owned())
}

fn analyze(text: &str) -> HtmlDocumentShellAnalysis {
    construct_html_document_shell(&source(text), limits()).expect("no boundary failure")
}

/// Drives the raw session token by token so the actual insertion mode, which a
/// frozen analysis never exposes, can be inspected.
fn drive(text: &str) -> (HtmlTreeSession, Vec<TokenOutcome>) {
    let run = tokenize(&source(text), limits());
    let mut session = HtmlTreeSession::new().expect("session start");
    let mut outcomes = Vec::new();
    for (index, token) in run.tokens().iter().enumerate() {
        let trigger = token_trigger(token, index);
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

fn mode(text: &str) -> InsertionMode {
    drive(text).0.insertion_mode()
}

fn character(analysis: &HtmlDocumentShellAnalysis, index: usize) -> (String, Range) {
    let HtmlToken::Character(character) = &analysis.tokenizer_run().tokens()[index] else {
        panic!("token {index} is not a Character token")
    };
    (
        character.interpreted().to_owned(),
        (
            character.source().range().start(),
            character.source().range().end(),
        ),
    )
}

fn trigger_evidence(trigger: &HtmlTreeTokenTrigger) -> Trigger {
    (
        trigger.token_index(),
        trigger
            .authored_boundary()
            .map(|anchor| (anchor.range().start(), anchor.range().end())),
    )
}

fn diagnostics_of(
    analysis: &HtmlDocumentShellAnalysis,
    code: HtmlTreeDiagnosticCode,
) -> Vec<(Trigger, HtmlTreeRecovery)> {
    analysis
        .diagnostics()
        .iter()
        .filter(|d| d.code() == code)
        .map(|d| (trigger_evidence(d.trigger()), d.recovery()))
        .collect()
}

fn after_after(analysis: &HtmlDocumentShellAnalysis) -> Vec<(Trigger, HtmlTreeRecovery)> {
    diagnostics_of(
        analysis,
        HtmlTreeDiagnosticCode::AfterAfterBodyCharacterData,
    )
}

/// `ReprocessedToken` actions for the run under test and later tokens. The
/// synthesized-shell reprocesses of token 0 (`<body>`) precede the run and are
/// not part of this theorem.
fn reprocesses(analysis: &HtmlDocumentShellAnalysis) -> Vec<Trigger> {
    analysis
        .actions()
        .iter()
        .filter(|a| a.trigger().token_index() >= 3)
        .filter(|a| matches!(a.kind(), HtmlTreeActionKind::ReprocessedToken))
        .map(|a| trigger_evidence(a.trigger()))
        .collect()
}

fn ignored_nuls(analysis: &HtmlDocumentShellAnalysis) -> Vec<Trigger> {
    analysis
        .actions()
        .iter()
        .filter(|a| matches!(a.kind(), HtmlTreeActionKind::IgnoredNullCharacterToken))
        .map(|a| trigger_evidence(a.trigger()))
        .collect()
}

fn text_actions(analysis: &HtmlDocumentShellAnalysis) -> Vec<(bool, Trigger)> {
    analysis
        .actions()
        .iter()
        .filter_map(|a| match a.kind() {
            HtmlTreeActionKind::InsertedTextNode { .. } => {
                Some((true, trigger_evidence(a.trigger())))
            }
            HtmlTreeActionKind::AppendedToTextNode { .. } => {
                Some((false, trigger_evidence(a.trigger())))
            }
            _ => None,
        })
        .collect()
}

/// Every text node as `(interpreted, contribution ranges)`.
fn texts(analysis: &HtmlDocumentShellAnalysis) -> Vec<(String, Vec<Range>)> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter_map(|node| match node.kind() {
            HtmlTreeNodeKind::Text(text) => Some((
                text.interpreted().to_owned(),
                text.contributions()
                    .iter()
                    .map(|c| (c.source().range().start(), c.source().range().end()))
                    .collect(),
            )),
            _ => None,
        })
        .collect()
}

fn unsupported(analysis: &HtmlDocumentShellAnalysis) -> Option<(HtmlTreeCapability, Trigger)> {
    match analysis.completion() {
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(u)) => {
            Some((u.capability(), trigger_evidence(u.trigger())))
        }
        _ => None,
    }
}

fn tokenizer_diagnostic_count(analysis: &HtmlDocumentShellAnalysis) -> usize {
    analysis.tokenizer_run().diagnostics().len()
}

const RECOVERY: HtmlTreeRecovery = HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken;

fn no_after_body_diagnostic(analysis: &HtmlDocumentShellAnalysis) {
    assert!(
        diagnostics_of(analysis, HtmlTreeDiagnosticCode::AfterBodyCharacterData).is_empty(),
        "AfterAfterBody must never borrow the AfterBody diagnostic"
    );
}

// ---------------------------------------------------------------------------
// Whitespace: delegated, mode stays AfterAfterBody, no diagnostic/reprocess
// ---------------------------------------------------------------------------

/// Falsifies: whitespace switching mode, recording a recovery diagnostic or a
/// reprocess, or losing the authored-vs-interpreted CR/CRLF distinction.
#[test]
fn aw1_html_whitespace_delegates_in_place() {
    // (authored suffix, interpreted value, authored range)
    let cases: [(&str, &str, Range); 7] = [
        (" ", " ", (20, 21)),
        ("\t", "\t", (20, 21)),
        ("\n", "\n", (20, 21)),
        ("\u{000c}", "\u{000c}", (20, 21)),
        ("\r", "\n", (20, 21)),
        ("\r\n", "\n", (20, 22)),
        (" \t", " \t", (20, 22)),
    ];
    for (suffix, interpreted, range) in cases {
        let text = format!("<body></body></html>{suffix}");
        let analysis = analyze(&text);
        assert_eq!(character(&analysis, 3), (interpreted.to_owned(), range));
        assert!(analysis.is_complete(), "{suffix:?}");
        assert_eq!(
            texts(&analysis),
            vec![(interpreted.to_owned(), vec![range])],
            "{suffix:?}"
        );
        assert_eq!(
            text_actions(&analysis),
            vec![(true, (3, Some(range)))],
            "{suffix:?}: exactly one text insertion for the token"
        );
        assert!(after_after(&analysis).is_empty(), "{suffix:?}");
        no_after_body_diagnostic(&analysis);
        assert!(reprocesses(&analysis).is_empty(), "{suffix:?}");
        assert_eq!(analysis.coverage().committed_end(), range.1, "{suffix:?}");
        assert_eq!(analysis.node_count(), 5, "{suffix:?}: shell(4) + one text");
        assert_eq!(mode(&text), InsertionMode::AfterAfterBody, "{suffix:?}");
    }
}

// ---------------------------------------------------------------------------
// Non-whitespace: one aggregate token is one recovery unit
// ---------------------------------------------------------------------------

/// Falsifies: omitted diagnostic, per-scalar recovery for `xy`, a missing or
/// duplicated reprocess, whitespace-class leakage for NBSP / U+000B, and a
/// recovery that does not move the actual mode to InBody.
#[test]
fn an1_non_whitespace_run_is_one_recovery_unit() {
    // (suffix, interpreted, range, tokenizer diagnostics)
    let cases: [(&str, &str, Range, usize); 4] = [
        ("x", "x", (20, 21), 0),
        ("xy", "xy", (20, 22), 0),
        ("\u{00a0}", "\u{00a0}", (20, 22), 0),
        // U+000B draws one lower-layer input-stream diagnostic; it stays a
        // non-whitespace scalar and is not an HTML whitespace.
        ("\u{000b}", "\u{000b}", (20, 21), 1),
    ];
    for (suffix, interpreted, range, lower) in cases {
        let text = format!("<body></body></html>{suffix}");
        let analysis = analyze(&text);
        assert_eq!(character(&analysis, 3), (interpreted.to_owned(), range));
        assert!(analysis.is_complete(), "{suffix:?}");
        assert_eq!(
            after_after(&analysis),
            vec![((3, Some(range)), RECOVERY)],
            "{suffix:?}: exactly one distinct diagnostic"
        );
        no_after_body_diagnostic(&analysis);
        assert_eq!(
            reprocesses(&analysis),
            vec![(3, Some(range))],
            "{suffix:?}: exactly one same-trigger reprocess"
        );
        assert_eq!(
            texts(&analysis),
            vec![(interpreted.to_owned(), vec![range])],
            "{suffix:?}: one aggregate contribution"
        );
        assert_eq!(text_actions(&analysis), vec![(true, (3, Some(range)))]);
        assert_eq!(tokenizer_diagnostic_count(&analysis), lower, "{suffix:?}");
        assert_eq!(analysis.coverage().committed_end(), range.1, "{suffix:?}");
        assert_eq!(analysis.node_count(), 5, "{suffix:?}");
        assert_eq!(mode(&text), InsertionMode::InBody, "{suffix:?}");
    }
}

/// Falsifies: reprocess itself advancing committed coverage or the processed
/// token count, and a second dispatch of the same token creating identity.
#[test]
fn an2_reprocess_advances_neither_coverage_nor_token_count() {
    let analysis = analyze("<body></body></html>x");
    // `<body>`, `</body>`, `</html>`, one Character, EOF: five tokens, once each.
    assert_eq!(analysis.coverage().processed_tokens(), 5);
    assert_eq!(analysis.coverage().committed_end(), 21);
    // Token 3's own actions are exactly one reprocess and one text insertion:
    // the redispatch is not a second consumption.
    assert_eq!(
        analysis
            .actions()
            .iter()
            .filter(|a| a.trigger().token_index() == 3)
            .count(),
        2
    );
    assert_eq!(text_actions(&analysis), vec![(true, (3, Some((20, 21))))]);
}

// ---------------------------------------------------------------------------
// Mixed: whole-token refusal before mutation
// ---------------------------------------------------------------------------

/// Falsifies: splitting the aggregate, a prefix/suffix extraction, a committed
/// diagnostic or mode change before refusal, and any text/identity leakage.
#[test]
fn am1_mixed_aggregate_is_refused_whole_before_mutation() {
    let cases: [(&str, &str, Range); 4] = [
        (" x", " x", (20, 22)),
        ("x ", "x ", (20, 22)),
        ("\r\nx", "\nx", (20, 23)),
        ("x y", "x y", (20, 23)),
    ];
    for (suffix, interpreted, range) in cases {
        let text = format!("<body></body></html>{suffix}");
        let analysis = analyze(&text);
        assert_eq!(character(&analysis, 3), (interpreted.to_owned(), range));
        assert_eq!(
            unsupported(&analysis),
            Some((
                HtmlTreeCapability::WhitespaceSensitiveCharacterData,
                (3, Some(range))
            )),
            "{suffix:?}: whole token is the trigger"
        );
        assert!(!analysis.is_complete());
        assert!(texts(&analysis).is_empty(), "{suffix:?}");
        assert!(text_actions(&analysis).is_empty(), "{suffix:?}");
        assert!(after_after(&analysis).is_empty(), "{suffix:?}");
        no_after_body_diagnostic(&analysis);
        assert!(reprocesses(&analysis).is_empty(), "{suffix:?}");
        assert_eq!(analysis.node_count(), 4, "{suffix:?}: no new identity");
        assert_eq!(analysis.coverage().committed_end(), 20, "{suffix:?}");
        let (session, outcomes) = drive(&text);
        assert!(matches!(
            outcomes.last(),
            Some(TokenOutcome::Unsupported(
                HtmlTreeCapability::WhitespaceSensitiveCharacterData
            ))
        ));
        assert_eq!(
            session.insertion_mode(),
            InsertionMode::AfterAfterBody,
            "{suffix:?}: refused before the mode could change"
        );
    }
}

// ---------------------------------------------------------------------------
// NUL composition with the accepted #885 authored Data U+0000 theorem
// ---------------------------------------------------------------------------

fn nul_diagnostics(analysis: &HtmlDocumentShellAnalysis) -> Vec<(Trigger, HtmlTreeRecovery)> {
    diagnostics_of(analysis, HtmlTreeDiagnosticCode::NullCharacterInBody)
}

fn assert_no_nul_or_replacement_text(analysis: &HtmlDocumentShellAnalysis) {
    for (text, _) in texts(analysis) {
        assert!(!text.contains('\0'), "no U+0000 tree text");
        assert!(!text.contains('\u{fffd}'), "no fabricated U+FFFD tree text");
    }
}

/// Falsifies: `</html>\0` becoming text or U+FFFD, or the AfterAfterBody
/// recovery replacing the InBody NUL ignore facts.
#[test]
fn az1_authored_nul_recovers_then_is_ignored_in_body() {
    let analysis = analyze("<body></body></html>\0");
    assert_eq!(character(&analysis, 3), ("\0".to_owned(), (20, 21)));
    assert!(analysis.is_complete());
    // Tokenizer layer: exactly one authored-NUL diagnostic, nothing else.
    assert_eq!(tokenizer_diagnostic_count(&analysis), 1);
    assert_eq!(
        after_after(&analysis),
        vec![((3, Some((20, 21))), RECOVERY)]
    );
    assert_eq!(reprocesses(&analysis), vec![(3, Some((20, 21)))]);
    assert_eq!(
        nul_diagnostics(&analysis),
        vec![((3, Some((20, 21))), HtmlTreeRecovery::IgnoredToken)]
    );
    assert_eq!(ignored_nuls(&analysis), vec![(3, Some((20, 21)))]);
    assert!(text_actions(&analysis).is_empty());
    assert!(texts(&analysis).is_empty());
    assert_no_nul_or_replacement_text(&analysis);
    assert_eq!(analysis.node_count(), 4, "ignored NUL admits no identity");
    assert_eq!(analysis.coverage().committed_end(), 21);
    assert_eq!(mode("<body></body></html>\0"), InsertionMode::InBody);
}

/// Falsifies: the second NUL requiring or fabricating a second AfterAfterBody
/// recovery, or the second NUL escaping its own ignore decision.
#[test]
fn az2_second_nul_is_ignored_without_a_second_recovery() {
    let analysis = analyze("<body></body></html>\0\0");
    assert_eq!(analysis.tokenizer_run().diagnostics().len(), 2);
    assert_eq!(
        after_after(&analysis),
        vec![((3, Some((20, 21))), RECOVERY)],
        "only the first NUL recovers"
    );
    assert_eq!(reprocesses(&analysis), vec![(3, Some((20, 21)))]);
    assert_eq!(
        nul_diagnostics(&analysis),
        vec![
            ((3, Some((20, 21))), HtmlTreeRecovery::IgnoredToken),
            ((4, Some((21, 22))), HtmlTreeRecovery::IgnoredToken)
        ]
    );
    assert_eq!(
        ignored_nuls(&analysis),
        vec![(3, Some((20, 21))), (4, Some((21, 22)))]
    );
    assert!(texts(&analysis).is_empty());
    assert_no_nul_or_replacement_text(&analysis);
    assert!(analysis.is_complete());
    assert_eq!(analysis.coverage().committed_end(), 22);
}

/// Falsifies: a trailing NUL re-entering AfterAfterBody after an ordinary
/// recovery, or the ordinary character losing its text.
#[test]
fn az3_ordinary_then_nul_recovers_once_and_ignores_the_nul() {
    let analysis = analyze("<body></body></html>x\0");
    assert_eq!(
        after_after(&analysis),
        vec![((3, Some((20, 21))), RECOVERY)]
    );
    assert_eq!(reprocesses(&analysis), vec![(3, Some((20, 21)))]);
    assert_eq!(texts(&analysis), vec![("x".to_owned(), vec![(20, 21)])]);
    assert_eq!(
        nul_diagnostics(&analysis),
        vec![((4, Some((21, 22))), HtmlTreeRecovery::IgnoredToken)]
    );
    assert_eq!(ignored_nuls(&analysis), vec![(4, Some((21, 22)))]);
    assert_no_nul_or_replacement_text(&analysis);
    assert!(analysis.is_complete());
}

/// Falsifies: whitespace-then-NUL treating the whitespace as recovery, or the
/// NUL dispatching under the wrong mode.
#[test]
fn az4_whitespace_then_nul_delegates_then_recovers_the_nul() {
    let analysis = analyze("<body></body></html> \0");
    assert_eq!(
        after_after(&analysis),
        vec![((4, Some((21, 22))), RECOVERY)],
        "only the NUL token recovers"
    );
    assert_eq!(reprocesses(&analysis), vec![(4, Some((21, 22)))]);
    assert_eq!(texts(&analysis), vec![(" ".to_owned(), vec![(20, 21)])]);
    assert_eq!(text_actions(&analysis), vec![(true, (3, Some((20, 21))))]);
    assert_eq!(ignored_nuls(&analysis), vec![(4, Some((21, 22)))]);
    assert_no_nul_or_replacement_text(&analysis);
    assert!(analysis.is_complete());
}

/// Falsifies: `\0x` treating the NUL and `x` as one token, or `x` triggering a
/// second AfterAfterBody recovery after the NUL already moved the mode.
#[test]
fn az5_nul_then_ordinary_recovers_once_and_inserts_the_ordinary_text() {
    let analysis = analyze("<body></body></html>\0x");
    assert_eq!(
        after_after(&analysis),
        vec![((3, Some((20, 21))), RECOVERY)]
    );
    assert_eq!(reprocesses(&analysis), vec![(3, Some((20, 21)))]);
    assert_eq!(ignored_nuls(&analysis), vec![(3, Some((20, 21)))]);
    assert_eq!(texts(&analysis), vec![("x".to_owned(), vec![(21, 22)])]);
    assert_eq!(text_actions(&analysis), vec![(true, (4, Some((21, 22))))]);
    assert_no_nul_or_replacement_text(&analysis);
    assert!(analysis.is_complete());
}

/// Falsifies: widening to numeric, RCDATA, tag or attribute NUL semantics.
#[test]
fn az6_numeric_zero_and_rcdata_nul_remain_outside_the_authored_data_path() {
    let analysis = analyze("<body></body></html>&#0;");
    assert_eq!(character(&analysis, 3), ("\u{fffd}".to_owned(), (20, 24)));
    assert_eq!(
        after_after(&analysis),
        vec![((3, Some((20, 24))), RECOVERY)]
    );
    assert!(ignored_nuls(&analysis).is_empty());
    assert_eq!(
        texts(&analysis),
        vec![("\u{fffd}".to_owned(), vec![(20, 24)])],
        "numeric zero stays decoded U+FFFD text"
    );
}

// ---------------------------------------------------------------------------
// Composition and identity
// ---------------------------------------------------------------------------

/// Falsifies: AfterAfterBody whitespace failing to coalesce with prior text, or
/// minting identity for a coalesced append.
#[test]
fn ac1_prior_text_coalesces_with_after_after_body_whitespace() {
    let analysis = analyze("<body>a</body></html> ");
    assert_eq!(character(&analysis, 4), (" ".to_owned(), (21, 22)));
    assert_eq!(
        texts(&analysis),
        vec![("a ".to_owned(), vec![(6, 7), (21, 22)])]
    );
    assert_eq!(
        text_actions(&analysis),
        vec![(true, (1, Some((6, 7)))), (false, (4, Some((21, 22))))]
    );
    assert!(after_after(&analysis).is_empty());
    assert!(reprocesses(&analysis).is_empty());
    assert_eq!(analysis.node_count(), 5, "append admits no identity");
    assert_eq!(
        mode("<body>a</body></html> "),
        InsertionMode::AfterAfterBody
    );
}

/// Falsifies: a recovered non-whitespace token failing to coalesce, or
/// consuming new identity for the append.
#[test]
fn ac2_prior_text_coalesces_with_recovered_non_whitespace() {
    let analysis = analyze("<body>x</body></html>y");
    assert_eq!(
        texts(&analysis),
        vec![("xy".to_owned(), vec![(6, 7), (21, 22)])]
    );
    assert_eq!(
        text_actions(&analysis),
        vec![(true, (1, Some((6, 7)))), (false, (4, Some((21, 22))))]
    );
    assert_eq!(
        after_after(&analysis),
        vec![((4, Some((21, 22))), RECOVERY)]
    );
    assert_eq!(reprocesses(&analysis), vec![(4, Some((21, 22)))]);
    assert_eq!(analysis.node_count(), 5);
    assert_eq!(analysis.coverage().committed_end(), 22);
    assert!(analysis.is_complete());
}

/// Falsifies: a recovered ignored NUL admitting identity or disturbing the
/// prior committed text.
#[test]
fn ac3_ignored_nul_admits_no_identity_after_prior_text() {
    let analysis = analyze("<body>a</body></html>\0");
    assert_eq!(texts(&analysis), vec![("a".to_owned(), vec![(6, 7)])]);
    assert_eq!(ignored_nuls(&analysis), vec![(4, Some((21, 22)))]);
    assert_eq!(analysis.node_count(), 5);
    assert_no_nul_or_replacement_text(&analysis);
    assert!(analysis.is_complete());
}

/// Falsifies: mixed refusal rewriting the committed prefix, or committing any
/// part of the refused token.
#[test]
fn ac4_mixed_refusal_preserves_the_committed_prefix() {
    let analysis = analyze("<body>a</body></html> x");
    assert_eq!(character(&analysis, 4), (" x".to_owned(), (21, 23)));
    assert_eq!(
        unsupported(&analysis),
        Some((
            HtmlTreeCapability::WhitespaceSensitiveCharacterData,
            (4, Some((21, 23)))
        ))
    );
    assert_eq!(texts(&analysis), vec![("a".to_owned(), vec![(6, 7)])]);
    assert_eq!(analysis.coverage().committed_end(), 21);
    assert!(after_after(&analysis).is_empty());
    assert!(reprocesses(&analysis).is_empty());
}

/// Falsifies: a later distinct token cycling through the same-token
/// redispatch guard after recovery (`y` then `z` are two aggregate runs only
/// when separated; adjacent ordinary characters are one token).
#[test]
fn ac5_adjacent_ordinary_characters_are_one_token_and_later_tokens_do_not_cycle() {
    let analysis = analyze("<body>x</body></html>yz");
    assert_eq!(character(&analysis, 4), ("yz".to_owned(), (21, 23)));
    assert_eq!(after_after(&analysis).len(), 1);
    assert_eq!(reprocesses(&analysis), vec![(4, Some((21, 23)))]);
    assert_eq!(
        texts(&analysis),
        vec![("xyz".to_owned(), vec![(6, 7), (21, 23)])]
    );
    assert!(analysis.is_complete());
}

/// Falsifies: AfterAfterBody text ignoring the preserved open-element parent
/// left by an html end tag over open selected content.
#[test]
fn ac6_preserved_open_element_is_the_parent_for_whitespace_and_recovered_text() {
    // `<body>` 0..6, `<div>` 6..11, `</html>` 11..18, then the run at 18.
    for (suffix, interpreted, range) in [(" ", " ", (18, 19)), ("x", "x", (18, 19))] {
        let text = format!("<body><div></html>{suffix}");
        // Already frozen by the production driver, so this exercises the
        // durable replay with the real preserved open stack.
        let analysis = &analyze(&text);
        assert!(analysis.is_complete(), "{suffix:?}");
        assert_eq!(
            texts(analysis),
            vec![(interpreted.to_owned(), vec![range])],
            "{suffix:?}"
        );
        let text_id = analysis
            .nodes_in_creation_order()
            .into_iter()
            .find(|n| matches!(n.kind(), HtmlTreeNodeKind::Text(_)))
            .expect("text node");
        let parent = text_id.parent().expect("text parent");
        let parent_node = analysis
            .nodes_in_creation_order()
            .into_iter()
            .find(|n| n.id() == parent)
            .expect("parent node");
        assert!(
            matches!(parent_node.kind(), HtmlTreeNodeKind::Element(_)),
            "{suffix:?}"
        );
        assert_ne!(
            parent_node.parent(),
            None,
            "{suffix:?}: the open `div`, not the document, is the target"
        );
        assert_eq!(
            analysis.nodes_in_creation_order().len(),
            6,
            "{suffix:?}: shell(4) + div + one text"
        );
    }
}

// ---------------------------------------------------------------------------
// Negative controls
// ---------------------------------------------------------------------------

/// Falsifies: EOF semantics changing at AfterAfterBody.
#[test]
fn ax1_eof_after_html_end_still_stops_parsing() {
    let analysis = analyze("<body></body></html>");
    assert!(analysis.is_complete());
    assert!(after_after(&analysis).is_empty());
    assert!(reprocesses(&analysis).is_empty());
    let (session, outcomes) = drive("<body></body></html>");
    assert!(matches!(
        outcomes.last(),
        Some(TokenOutcome::StoppedParsing)
    ));
    assert_eq!(session.insertion_mode(), InsertionMode::AfterAfterBody);
}

/// Falsifies: tag, Comment, DOCTYPE or markup-declaration widening at
/// AfterAfterBody.
#[test]
fn ax2_tags_comments_and_declarations_remain_unsupported() {
    for text in [
        "<body></body></html><p>",
        "<body></body></html></p>",
        "<body></body></html><html>",
        "<body></body></html><!--c-->",
        "<body></body></html><!DOCTYPE html>",
        "<body></body></html><?pi>",
    ] {
        let analysis = analyze(text);
        assert!(!analysis.is_complete(), "{text:?}");
        assert!(after_after(&analysis).is_empty(), "{text:?}");
        assert!(reprocesses(&analysis).is_empty(), "{text:?}");
        assert!(texts(&analysis).is_empty(), "{text:?}");
    }
}

/// Falsifies: lower-layer incompleteness being upgraded to tree Complete for
/// an AfterAfterBody run.
#[test]
fn ax3_lower_layer_incompleteness_is_never_upgraded() {
    for text in ["<body></body></html>x", "<body></body></html> "] {
        let tiny = HtmlTokenizerLimits::new(1_024, 8_192, 3, 1_024, 256, 4_096, 1_024);
        let truncated =
            construct_html_document_shell(&source(text), tiny).expect("no boundary failure");
        assert!(truncated.tokenizer_run().is_incomplete(), "{text:?}");
        assert!(!truncated.is_complete(), "{text:?}");
        assert!(matches!(
            truncated.completion(),
            HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
        ));
    }
}

/// Falsifies: semantic output depending on the source identity.
#[test]
fn ax4_correspondence_is_deterministic_across_repeats_and_source_ids() {
    let baseline = texts(&analyze("<body>x</body></html>y"));
    for id in [1_u64, 7, 99] {
        let text = "<body>x</body></html>y";
        let again = construct_html_document_shell(
            &SourceText::new(SourceId::new(id), text.to_owned()),
            limits(),
        )
        .expect("no boundary failure");
        assert_eq!(texts(&again), baseline);
        assert!(again.is_complete());
    }
}

// ---------------------------------------------------------------------------
// Durable freeze sealing: malformed AfterAfterBody evidence is rejected
// ---------------------------------------------------------------------------

struct Frozen {
    text: &'static str,
    analysis: HtmlDocumentShellAnalysis,
}

impl Frozen {
    fn new(text: &'static str) -> Self {
        Self {
            text,
            analysis: analyze(text),
        }
    }

    fn trigger(&self, index: usize) -> HtmlTreeTokenTrigger {
        let boundary = match &self.analysis.tokenizer_run().tokens()[index] {
            HtmlToken::Character(c) => c.source().clone(),
            HtmlToken::Tag(t) => t.complete().clone(),
            HtmlToken::EndOfFile(_) => panic!("end-of-file has no authored boundary"),
        };
        HtmlTreeTokenTrigger::authored(index, boundary)
    }

    fn parts(&self) -> HtmlDocumentShellParts {
        let a = &self.analysis;
        let nodes: Vec<HtmlTreeNode> = a.nodes_in_storage_order().cloned().collect();
        HtmlDocumentShellParts {
            admitted_creation_events: u32::try_from(nodes.len()).expect("small"),
            nodes,
            root: a.root(),
            diagnostics: a.diagnostics().to_vec(),
            actions: a.actions().to_vec(),
            processed_tokens: a.coverage().processed_tokens(),
            committed_prefix_end: a.coverage().committed_end(),
            completion: a.completion().clone(),
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

    fn freeze(&self, parts: HtmlDocumentShellParts) -> Result<(), HtmlTreeFreezeError> {
        freeze(
            &source(self.text),
            tokenize(&source(self.text), limits()),
            parts,
        )
        .map(|_| ())
    }
}

fn is_after_after(d: &HtmlTreeDiagnostic) -> bool {
    d.code() == HtmlTreeDiagnosticCode::AfterAfterBodyCharacterData
}

/// The recovery reprocess of the run under test; token 0's synthesized-shell
/// reprocesses are a predecessor theorem and are left untouched.
fn is_reprocess(a: &HtmlTreeAction) -> bool {
    a.trigger().token_index() >= 3 && matches!(a.kind(), HtmlTreeActionKind::ReprocessedToken)
}

fn is_text(a: &HtmlTreeAction) -> bool {
    matches!(
        a.kind(),
        HtmlTreeActionKind::InsertedTextNode { .. } | HtmlTreeActionKind::AppendedToTextNode { .. }
    )
}

fn recovery_mismatch(token_index: usize) -> HtmlTreeFreezeError {
    HtmlTreeFreezeError::AfterAfterBodyCharacterRecoveryMismatch { token_index }
}

fn consumption_mismatch(token_index: usize) -> HtmlTreeFreezeError {
    HtmlTreeFreezeError::AfterAfterBodyCharacterConsumptionMismatch { token_index }
}

/// The unmutated reconstructions freeze, so each rejection below is caused by
/// the mutation alone.
#[test]
fn afz0_the_unmutated_reconstructions_freeze() {
    for text in [
        "<body></body></html>x",
        "<body></body></html>xy",
        "<body></body></html> ",
        "<body></body></html>\0",
        "<body></body></html>\0\0",
        "<body></body></html>x\0",
        "<body></body></html> \0",
        "<body></body></html>\0x",
        "<body>x</body></html>y",
    ] {
        let frozen = Frozen::new(text);
        assert_eq!(frozen.freeze(frozen.parts()), Ok(()), "{text:?}");
    }
}

/// Falsifies: a freeze accepting a missing, duplicated, mis-triggered or
/// mis-recovered AfterAfterBody diagnostic.
#[test]
fn afz1_diagnostic_corruptions_are_rejected() {
    let frozen = Frozen::new("<body></body></html>x");

    let mut parts = frozen.parts();
    parts.diagnostics.retain(|d| !is_after_after(d));
    assert_eq!(frozen.freeze(parts), Err(recovery_mismatch(3)), "missing");

    let mut parts = frozen.parts();
    let duplicate = parts
        .diagnostics
        .iter()
        .find(|d| is_after_after(d))
        .cloned()
        .unwrap();
    parts.diagnostics.push(duplicate);
    assert_eq!(frozen.freeze(parts), Err(recovery_mismatch(3)), "duplicate");

    let mut parts = frozen.parts();
    let at = parts.diagnostics.iter().position(is_after_after).unwrap();
    parts.diagnostics[at] = HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::AfterAfterBodyCharacterData,
        frozen.trigger(2),
        RECOVERY,
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(recovery_mismatch(3)),
        "wrong diagnostic trigger"
    );

    for wrong in [
        HtmlTreeRecovery::IgnoredToken,
        HtmlTreeRecovery::SwitchedToAfterBodyPreservingOpenElements,
    ] {
        let mut parts = frozen.parts();
        let at = parts.diagnostics.iter().position(is_after_after).unwrap();
        parts.diagnostics[at] = HtmlTreeDiagnostic::new(
            HtmlTreeDiagnosticCode::AfterAfterBodyCharacterData,
            frozen.trigger(3),
            wrong,
        );
        assert_eq!(
            frozen.freeze(parts),
            Err(recovery_mismatch(3)),
            "wrong recovery {wrong:?}"
        );
    }

    // The AfterBody diagnostic is not an acceptable substitute.
    let mut parts = frozen.parts();
    let at = parts.diagnostics.iter().position(is_after_after).unwrap();
    parts.diagnostics[at] = HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::AfterBodyCharacterData,
        frozen.trigger(3),
        RECOVERY,
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(recovery_mismatch(3)),
        "AfterBody diagnostic substituted"
    );
}

/// Falsifies: a freeze that lets an AfterAfterBody Character episode carry the
/// predecessor AfterBody diagnostic *in addition to* the correct evidence. This
/// is distinct from the substitution mutation in `afz1`.
#[test]
fn afz1b_additive_after_body_diagnostic_is_rejected() {
    // Recovered non-whitespace: the valid AfterAfterBody diagnostic stays.
    let frozen = Frozen::new("<body></body></html>x");
    let mut parts = frozen.parts();
    assert_eq!(
        parts
            .diagnostics
            .iter()
            .filter(|d| is_after_after(d))
            .count(),
        1
    );
    parts.diagnostics.push(HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::AfterBodyCharacterData,
        frozen.trigger(3),
        RECOVERY,
    ));
    assert_eq!(
        frozen.freeze(parts),
        Err(recovery_mismatch(3)),
        "recovered non-whitespace with an extra AfterBody diagnostic"
    );

    // Whitespace delegation: otherwise valid (one text action, no reprocess,
    // no AfterAfterBody diagnostic).
    let frozen = Frozen::new("<body></body></html> ");
    let mut parts = frozen.parts();
    assert!(!parts.diagnostics.iter().any(is_after_after));
    parts.diagnostics.push(HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::AfterBodyCharacterData,
        frozen.trigger(3),
        RECOVERY,
    ));
    assert_eq!(
        frozen.freeze(parts),
        Err(consumption_mismatch(3)),
        "whitespace with an AfterBody diagnostic"
    );
}

/// Falsifies: a freeze accepting a missing, duplicated or mis-attached
/// reprocess.
#[test]
fn afz2_reprocess_corruptions_are_rejected() {
    let frozen = Frozen::new("<body></body></html>x");

    // Missing reprocess: the token's text is then consumed in place as if it
    // were whitespace, which the whitespace theorem rejects.
    let mut parts = frozen.parts();
    parts.actions.retain(|a| !is_reprocess(a));
    assert_eq!(
        frozen.freeze(parts),
        Err(consumption_mismatch(3)),
        "missing reprocess"
    );

    // Duplicate same-trigger reprocess.
    let mut parts = frozen.parts();
    let at = parts.actions.iter().position(is_reprocess).unwrap();
    let duplicate = parts.actions[at].clone();
    parts.actions.insert(at, duplicate);
    assert_eq!(
        frozen.freeze(parts),
        Err(recovery_mismatch(3)),
        "duplicate reprocess"
    );

    // Reprocess attached to the wrong token (the preceding `</html>`).
    let mut parts = frozen.parts();
    let at = parts.actions.iter().position(is_reprocess).unwrap();
    parts.actions[at] =
        HtmlTreeAction::new(HtmlTreeActionKind::ReprocessedToken, frozen.trigger(2));
    assert_eq!(
        frozen.freeze(parts),
        Err(recovery_mismatch(2)),
        "reprocess on the wrong token"
    );
}

/// Falsifies: a freeze accepting an ordinary recovery whose text is missing,
/// duplicated, or placed under the wrong parent.
#[test]
fn afz3_recovered_text_corruptions_are_rejected() {
    let frozen = Frozen::new("<body></body></html>x");

    let mut parts = frozen.parts();
    parts.actions.retain(|a| !is_text(a));
    assert!(frozen.freeze(parts).is_err(), "recovered text missing");

    let mut parts = frozen.parts();
    let at = parts.actions.iter().position(is_text).unwrap();
    let duplicate = parts.actions[at].clone();
    parts.actions.insert(at + 1, duplicate);
    assert_eq!(
        frozen.freeze(parts),
        Err(recovery_mismatch(3)),
        "duplicate recovered text action"
    );

    // Wrong parent: move the text node under `html` (children kept consistent).
    let mut parts = frozen.parts();
    let text_id = parts
        .nodes
        .iter()
        .find(|n| matches!(n.kind(), HtmlTreeNodeKind::Text(_)))
        .map(HtmlTreeNode::id)
        .unwrap();
    let old_parent = parts
        .nodes
        .iter()
        .find(|n| n.id() == text_id)
        .and_then(HtmlTreeNode::parent)
        .unwrap();
    let new_parent: HtmlConstructedNodeId = parts
        .nodes
        .iter()
        .find(|n| n.children().contains(&old_parent))
        .map(HtmlTreeNode::id)
        .unwrap();
    for node in &mut parts.nodes {
        let mut children = node.children().to_vec();
        let mut parent = node.parent();
        if node.id() == old_parent {
            children.retain(|c| *c != text_id);
        }
        if node.id() == new_parent {
            children.push(text_id);
        }
        if node.id() == text_id {
            parent = Some(new_parent);
        }
        *node = HtmlTreeNode::new(node.id(), parent, children, node.kind().clone());
    }
    assert_eq!(
        frozen.freeze(parts),
        Err(recovery_mismatch(3)),
        "recovered text under the wrong parent"
    );
}

/// Falsifies: whitespace being allowed to carry recovery evidence, or a
/// whitespace token losing its single text consumption.
#[test]
fn afz4_whitespace_corruptions_are_rejected() {
    let frozen = Frozen::new("<body></body></html> ");

    let mut parts = frozen.parts();
    parts.diagnostics.push(HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::AfterAfterBodyCharacterData,
        frozen.trigger(3),
        RECOVERY,
    ));
    assert_eq!(
        frozen.freeze(parts),
        Err(consumption_mismatch(3)),
        "whitespace with a recovery diagnostic"
    );

    let mut parts = frozen.parts();
    let at = parts.actions.iter().position(is_text).unwrap();
    parts.actions.insert(
        at,
        HtmlTreeAction::new(HtmlTreeActionKind::ReprocessedToken, frozen.trigger(3)),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(recovery_mismatch(3)),
        "whitespace with a reprocess"
    );

    let mut parts = frozen.parts();
    parts.actions.retain(|a| !is_text(a));
    assert_eq!(
        frozen.freeze(parts),
        Err(consumption_mismatch(3)),
        "whitespace text action missing"
    );

    let mut parts = frozen.parts();
    let at = parts.actions.iter().position(is_text).unwrap();
    let duplicate = parts.actions[at].clone();
    parts.actions.insert(at + 1, duplicate);
    assert_eq!(
        frozen.freeze(parts),
        Err(consumption_mismatch(3)),
        "whitespace text action duplicated"
    );
}

/// Falsifies: a freeze that lets the AfterAfterBody recovery stand in for the
/// InBody NUL ignore facts, or that lets a recovered NUL leak text.
#[test]
fn afz5_recovered_nul_corruptions_are_rejected() {
    let frozen = Frozen::new("<body></body></html>\0");

    // Missing ignored-NUL action (its diagnostic remains).
    let mut parts = frozen.parts();
    parts
        .actions
        .retain(|a| !matches!(a.kind(), HtmlTreeActionKind::IgnoredNullCharacterToken));
    assert!(frozen.freeze(parts).is_err(), "ignore action missing");

    // Missing InBody NUL diagnostic (the action remains).
    let mut parts = frozen.parts();
    parts
        .diagnostics
        .retain(|d| d.code() != HtmlTreeDiagnosticCode::NullCharacterInBody);
    assert!(frozen.freeze(parts).is_err(), "NUL diagnostic missing");

    // The recovery diagnostic alone must not be accepted as the ignore.
    let mut parts = frozen.parts();
    parts
        .actions
        .retain(|a| !matches!(a.kind(), HtmlTreeActionKind::IgnoredNullCharacterToken));
    parts
        .diagnostics
        .retain(|d| d.code() != HtmlTreeDiagnosticCode::NullCharacterInBody);
    assert_eq!(
        frozen.freeze(parts),
        Err(recovery_mismatch(3)),
        "recovery without any ignore decision"
    );

    // A recovered NUL that leaks a text action instead of being ignored.
    let ordinary = Frozen::new("<body></body></html>x");
    let mut parts = frozen.parts();
    let text_action = ordinary
        .parts()
        .actions
        .into_iter()
        .find(is_text)
        .expect("text action");
    let at = parts
        .actions
        .iter()
        .position(|a| matches!(a.kind(), HtmlTreeActionKind::IgnoredNullCharacterToken))
        .unwrap();
    parts.actions.insert(at, text_action);
    assert!(frozen.freeze(parts).is_err(), "recovered NUL leaking text");
}

/// Falsifies: a stray recovery diagnostic on a token that never recovered
/// (for example the second NUL of `\0\0`).
#[test]
fn afz6_second_nul_cannot_carry_a_second_recovery() {
    let frozen = Frozen::new("<body></body></html>\0\0");
    let mut parts = frozen.parts();
    parts.diagnostics.push(HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::AfterAfterBodyCharacterData,
        frozen.trigger(4),
        RECOVERY,
    ));
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::OrphanAfterAfterBodyCharacterDiagnostic { token_index: 4 })
    );
}
