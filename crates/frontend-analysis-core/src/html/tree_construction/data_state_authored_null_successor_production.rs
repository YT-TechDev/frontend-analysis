//! Production correspondence for the authored Data-state U+0000 correction
//! (Issue #884).
//!
//! Expectations here are authored by hand from the accepted theorem (WHATWG
//! HTML Data state and InBody U+0000 handling as scoped by #882, #348 and
//! #884), never from production output. The tests import only production
//! tokenizer, tree-construction and durable-result seams. They do not import,
//! call, or copy the candidate-independent validation machine in the sibling
//! `data_state_authored_null_successor_validation` module.
//!
//! Each test names the incorrect implementation it rejects.

use crate::{SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::diagnostic::{
    HtmlTokenizerDiagnosticCode as Diag, HtmlTokenizerDiagnosticContext as Context,
    HtmlTokenizerDiagnosticHandling as Handling, HtmlTokenizerDiagnosticSubject as Subject,
    HtmlTokenizerRecoveryKind,
};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerResource};
use super::super::tokenizer::result::{
    HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause, HtmlTokenizerRunResult,
};
use super::driver::construct_html_document_shell;
use super::result::{
    HtmlConstructedNodeId, HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlTextContribution,
    HtmlTextNode, HtmlTreeAction, HtmlTreeActionKind, HtmlTreeCapability, HtmlTreeCompletion,
    HtmlTreeDiagnostic, HtmlTreeDiagnosticCode, HtmlTreeFreezeError, HtmlTreeIncompleteCause,
    HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery, HtmlTreeTokenTrigger, freeze,
};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

type Limits = (usize, usize, usize, usize, usize, usize, usize);

/// source, steps, tokens, diagnostics, attributes, retained, temporary.
const AMPLE: Limits = (4_096, 32_768, 4_096, 4_096, 256, 16_384, 4_096);

fn limits_from(l: Limits) -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(l.0, l.1, l.2, l.3, l.4, l.5, l.6)
}

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), text.to_owned())
}

fn lex_with(text: &str, limits: Limits) -> HtmlTokenizerRunResult {
    tokenize(&source(text), limits_from(limits))
}

fn lex(text: &str) -> HtmlTokenizerRunResult {
    lex_with(text, AMPLE)
}

fn analyze_with(text: &str, limits: Limits) -> HtmlDocumentShellAnalysis {
    construct_html_document_shell(&source(text), limits_from(limits))
        .expect("authored Data NUL production boundary")
}

fn analyze(text: &str) -> HtmlDocumentShellAnalysis {
    analyze_with(text, AMPLE)
}

fn chars(run: &HtmlTokenizerRunResult) -> Vec<((usize, usize), String)> {
    run.tokens()
        .iter()
        .filter_map(|token| match token {
            HtmlToken::Character(character) => Some((
                (
                    character.source().range().start(),
                    character.source().range().end(),
                ),
                character.interpreted().to_owned(),
            )),
            _ => None,
        })
        .collect()
}

fn codes(run: &HtmlTokenizerRunResult) -> Vec<(Diag, (usize, usize))> {
    run.diagnostics()
        .iter()
        .map(|d| {
            (
                d.code(),
                (d.location().range().start(), d.location().range().end()),
            )
        })
        .collect()
}

fn complete(run: &HtmlTokenizerRunResult) -> bool {
    matches!(run.completion(), HtmlTokenizerCompletion::Complete)
}

/// `(resource, limit, attempted, at)` of a resource stop.
fn stop(
    run: &HtmlTokenizerRunResult,
) -> Option<(HtmlTokenizerResource, usize, usize, (usize, usize))> {
    match run.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(l)) => {
            Some((
                l.resource(),
                l.limit(),
                l.attempted(),
                (l.at().range().start(), l.at().range().end()),
            ))
        }
        _ => None,
    }
}

fn contributions(analysis: &HtmlDocumentShellAnalysis) -> Vec<((usize, usize), String)> {
    let mut out = Vec::new();
    for node in analysis.nodes_in_creation_order() {
        if let HtmlTreeNodeKind::Text(text) = node.kind() {
            for c in text.contributions() {
                out.push((
                    (c.source().range().start(), c.source().range().end()),
                    c.interpreted().to_owned(),
                ));
            }
        }
    }
    out
}

fn texts(analysis: &HtmlDocumentShellAnalysis) -> Vec<String> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter_map(|node| match node.kind() {
            HtmlTreeNodeKind::Text(text) => Some(text.interpreted().to_owned()),
            _ => None,
        })
        .collect()
}

fn null_diagnostics(analysis: &HtmlDocumentShellAnalysis) -> Vec<(usize, HtmlTreeRecovery)> {
    analysis
        .diagnostics()
        .iter()
        .filter(|d| d.code() == HtmlTreeDiagnosticCode::NullCharacterInBody)
        .map(|d| (d.trigger().token_index(), d.recovery()))
        .collect()
}

fn ignored_actions(analysis: &HtmlDocumentShellAnalysis) -> Vec<usize> {
    analysis
        .actions()
        .iter()
        .filter(|a| matches!(a.kind(), HtmlTreeActionKind::IgnoredNullCharacterToken))
        .map(|a| a.trigger().token_index())
        .collect()
}

fn no_nul_or_replacement_text(analysis: &HtmlDocumentShellAnalysis) {
    for text in texts(analysis) {
        assert!(!text.contains('\0'), "no U+0000 text");
        assert!(!text.contains('\u{fffd}'), "no fabricated U+FFFD text");
    }
}

fn tree_unsupported(analysis: &HtmlDocumentShellAnalysis) -> Option<HtmlTreeCapability> {
    match analysis.completion() {
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(u)) => {
            Some(u.capability())
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tokenizer: exact authored Data U+0000
// ---------------------------------------------------------------------------

/// Falsifies: Data NUL -> U+FFFD; Data NUL claiming `Recovered`; a diagnostic
/// whose subject is an emitted token; a replacement or aggregate token.
#[test]
fn dn1_a_lone_authored_nul_is_one_exact_token_and_a_continued_diagnostic() {
    let run = lex("\0");
    assert!(complete(&run));
    assert_eq!(chars(&run), vec![((0, 1), "\0".to_owned())]);
    let [diagnostic] = run.diagnostics() else {
        panic!("exactly one diagnostic");
    };
    assert_eq!(diagnostic.code(), Diag::UnexpectedNullCharacter);
    assert_eq!(diagnostic.context(), Context::Data);
    assert_eq!(diagnostic.handling(), Handling::Continued);
    assert!(matches!(diagnostic.subject(), Subject::InputLocation));
    assert_eq!(
        (
            diagnostic.location().range().start(),
            diagnostic.location().range().end()
        ),
        (0, 1)
    );
    assert_eq!(run.usage().emitted_tokens(), 2, "NUL token + EOF");
}

/// Falsifies: a NUL merged into the ordinary Data run (one aggregate
/// `a\0b` token), or an interpreted-length endpoint reconstruction.
#[test]
fn dn2_a_nul_inside_ordinary_data_splits_the_run_with_exact_sources() {
    let run = lex("a\0b");
    assert!(complete(&run));
    assert_eq!(
        chars(&run),
        vec![
            ((0, 1), "a".to_owned()),
            ((1, 2), "\0".to_owned()),
            ((2, 3), "b".to_owned()),
        ]
    );
    assert_eq!(codes(&run), vec![(Diag::UnexpectedNullCharacter, (1, 2))]);
}

/// Falsifies: consecutive NULs collapsing into one token or losing a
/// per-NUL source span or diagnostic.
#[test]
fn dn3_consecutive_nuls_stay_separate_source_backed_tokens() {
    let run = lex("\0\0\0");
    assert!(complete(&run));
    assert_eq!(
        chars(&run),
        vec![
            ((0, 1), "\0".to_owned()),
            ((1, 2), "\0".to_owned()),
            ((2, 3), "\0".to_owned()),
        ]
    );
    assert_eq!(
        codes(&run),
        vec![
            (Diag::UnexpectedNullCharacter, (0, 1)),
            (Diag::UnexpectedNullCharacter, (1, 2)),
            (Diag::UnexpectedNullCharacter, (2, 3)),
        ]
    );
    let run = lex("x\0\0y");
    assert_eq!(
        chars(&run),
        vec![
            ((0, 1), "x".to_owned()),
            ((1, 2), "\0".to_owned()),
            ((2, 3), "\0".to_owned()),
            ((3, 4), "y".to_owned()),
        ]
    );
}

/// Falsifies: reference output and authored NUL provenance being merged, or
/// a NUL after a Character Reference losing authored-Data semantics.
#[test]
fn dn4_a_nul_after_a_reference_keeps_authored_data_semantics() {
    let run = lex("&amp;\0b");
    assert!(complete(&run));
    assert_eq!(
        chars(&run),
        vec![
            ((0, 5), "&".to_owned()),
            ((5, 6), "\0".to_owned()),
            ((6, 7), "b".to_owned()),
        ]
    );
    assert_eq!(codes(&run), vec![(Diag::UnexpectedNullCharacter, (5, 6))]);

    let run = lex("&#65;\0b");
    assert!(complete(&run));
    assert_eq!(
        chars(&run),
        vec![
            ((0, 5), "A".to_owned()),
            ((5, 6), "\0".to_owned()),
            ((6, 7), "b".to_owned()),
        ]
    );
    assert_eq!(codes(&run), vec![(Diag::UnexpectedNullCharacter, (5, 6))]);
}

/// Falsifies: Numeric zero changing to U+0000, or decoded U+FFFD being fed
/// back into authored-input handling.
#[test]
fn dn5_numeric_zero_remains_replacement_with_its_own_diagnostic() {
    for (text, end) in [("&#0;", 4), ("&#x0;", 5)] {
        let run = lex(text);
        assert!(complete(&run), "{text:?}");
        assert_eq!(
            chars(&run),
            vec![((0, end), "\u{fffd}".to_owned())],
            "{text:?}"
        );
        let observed = codes(&run);
        assert_eq!(observed.len(), 1, "{text:?}");
        assert_eq!(observed[0].0, Diag::NullCharacterReference, "{text:?}");
        assert!(
            observed
                .iter()
                .all(|(code, _)| *code != Diag::UnexpectedNullCharacter),
            "{text:?}"
        );
    }
}

/// Falsifies: the generic replacement helper being weakened or generalized
/// for non-Data contexts.
#[test]
fn dn6_tag_and_attribute_nul_keep_their_replacement_semantics() {
    let replaced = |run: &HtmlTokenizerRunResult| {
        run.diagnostics()
            .iter()
            .filter(|d| d.code() == Diag::UnexpectedNullCharacter)
            .map(|d| d.handling())
            .collect::<Vec<_>>()
    };
    let recovered =
        Handling::Recovered(HtmlTokenizerRecoveryKind::ReplacedNullWithReplacementCharacter);

    let run = lex("<a\0>");
    assert!(complete(&run));
    let Some(HtmlToken::Tag(tag)) = run.tokens().first() else {
        panic!("a start tag");
    };
    assert_eq!(tag.kind(), HtmlTagKind::Start);
    assert_eq!(tag.name().interpreted(), "a\u{fffd}");
    assert_eq!(replaced(&run), vec![recovered]);

    for text in ["<a x=\"\0\">", "<a x='\0'>", "<a x=\0>"] {
        let run = lex(text);
        assert!(complete(&run), "{text:?}");
        let Some(HtmlToken::Tag(tag)) = run.tokens().first() else {
            panic!("a start tag for {text:?}");
        };
        let [attribute] = tag.attributes() else {
            panic!("one attribute for {text:?}");
        };
        assert_eq!(attribute.interpreted_value(), "\u{fffd}", "{text:?}");
        assert_eq!(replaced(&run), vec![recovered], "{text:?}");
    }
}

/// Falsifies: RCDATA authored NUL widening to the Data correction. The
/// selected Title boundary stays unsupported with no Data NUL evidence.
#[test]
fn dn7_title_rcdata_nul_does_not_widen() {
    let analysis = analyze("<title>\0</title>");
    assert!(!analysis.is_complete());
    assert!(matches!(
        analysis.tokenizer_run().completion(),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(_))
    ));
    assert!(
        analysis
            .tokenizer_run()
            .diagnostics()
            .iter()
            .all(|d| d.code() != Diag::UnexpectedNullCharacter)
    );
    assert!(ignored_actions(&analysis).is_empty());
    assert!(null_diagnostics(&analysis).is_empty());
}

// ---------------------------------------------------------------------------
// Tokenizer resources
// ---------------------------------------------------------------------------

fn with(
    steps: Option<usize>,
    tokens: Option<usize>,
    diags: Option<usize>,
    retained: Option<usize>,
) -> Limits {
    (
        AMPLE.0,
        steps.unwrap_or(AMPLE.1),
        tokens.unwrap_or(AMPLE.2),
        diags.unwrap_or(AMPLE.3),
        AMPLE.4,
        retained.unwrap_or(AMPLE.5),
        AMPLE.6,
    )
}

fn assert_resource(
    text: &str,
    limits: Limits,
    resource: HtmlTokenizerResource,
    attempted: usize,
    tokens: usize,
    diagnostics: usize,
    label: &str,
) -> HtmlTokenizerRunResult {
    let run = lex_with(text, limits);
    let (observed, _, observed_attempted, _) =
        stop(&run).unwrap_or_else(|| panic!("{label}: resource stop"));
    assert_eq!(observed, resource, "{label}");
    assert_eq!(observed_attempted, attempted, "{label}");
    assert_eq!(run.tokens().len(), tokens, "{label}: committed tokens");
    assert_eq!(run.diagnostics().len(), diagnostics, "{label}: diagnostics");
    assert!(
        chars(&run).iter().all(|(_, t)| !t.contains('\u{fffd}')),
        "{label}: no U+FFFD"
    );
    // Every committed observation-conditioned diagnostic lies within the
    // committed processed coverage.
    for diagnostic in run.diagnostics() {
        assert!(
            diagnostic.location().range().end() <= run.coverage().processed_end(),
            "{label}: diagnostic within coverage"
        );
    }
    run
}

/// Falsifies: the NUL diagnostic or token committing before the pending
/// ordinary run closes. (A zero token or step limit is an invalid tokenizer
/// configuration, so the smallest refusals use a limit of one.)
#[test]
fn dn8_prior_run_refusal_precedes_the_nul_observation() {
    // `<a>` fills the single token slot; the pending `b` run is refused at
    // the NUL dispatch, so the NUL itself is never observed.
    let run = assert_resource(
        "<a>b\0",
        with(None, Some(1), None, None),
        HtmlTokenizerResource::EmittedTokens,
        2,
        1,
        0,
        "prior run",
    );
    assert_eq!(
        run.coverage().processed_end(),
        4,
        "pending run covered, NUL not"
    );
    assert!(chars(&run).is_empty());
}

/// Falsifies: a Data NUL observed before its transition is charged.
#[test]
fn dn9_transition_refusal_before_dispatch_commits_nothing() {
    // `a` is charged by step 1; the NUL dispatch (attempted 2) is refused, so
    // no diagnostic and no NUL token exist. The pre-existing best-effort
    // flush on stop (as in accepted RES-002) closes the pending `a` only.
    let run = assert_resource(
        "a\0",
        with(Some(1), None, None, None),
        HtmlTokenizerResource::TransitionSteps,
        2,
        1,
        0,
        "NUL dispatch refused",
    );
    assert_eq!(chars(&run), vec![((0, 1), "a".to_owned())]);
    assert_eq!(run.coverage().processed_end(), 1);
    // One step charged: the NUL is fully processed before EOF is refused.
    let run = assert_resource(
        "\0",
        with(Some(1), None, None, None),
        HtmlTokenizerResource::TransitionSteps,
        2,
        1,
        1,
        "EOF dispatch refused",
    );
    assert_eq!(chars(&run), vec![((0, 1), "\0".to_owned())]);
    assert_eq!(run.coverage().processed_end(), 1);
}

/// Falsifies: continuing silently without the mandatory parse-error
/// evidence, or fabricating a token/replacement when the diagnostic refuses.
#[test]
fn dn10_diagnostic_refusal_emits_no_token_and_no_replacement() {
    let run = assert_resource(
        "\0",
        with(None, None, Some(0), None),
        HtmlTokenizerResource::Diagnostics,
        1,
        0,
        0,
        "diagnostic refusal",
    );
    assert_eq!(run.coverage().processed_end(), 0);
    assert_eq!(stop(&run).map(|s| (s.1, s.3)), Some((0, (0, 1))));
}

/// Falsifies: the diagnostic disappearing on a later token refusal, a
/// replacement token, or lost coverage of the authored NUL.
#[test]
fn dn11_token_refusal_after_the_diagnostic_keeps_the_diagnostic() {
    // `<a>` fills the single token slot, so the NUL token itself is refused.
    let run = assert_resource(
        "<a>\0",
        with(None, Some(1), None, None),
        HtmlTokenizerResource::EmittedTokens,
        2,
        1,
        1,
        "token refusal",
    );
    assert_eq!(codes(&run), vec![(Diag::UnexpectedNullCharacter, (3, 4))]);
    let diagnostic = &run.diagnostics()[0];
    assert_eq!(diagnostic.handling(), Handling::Continued);
    assert!(matches!(diagnostic.subject(), Subject::InputLocation));
    assert_eq!(run.coverage().processed_end(), 4);
    assert!(chars(&run).is_empty(), "no U+0000 token");

    // After a closed prior run the same refusal is the NUL's own.
    let run = assert_resource(
        "a\0",
        with(None, Some(1), None, None),
        HtmlTokenizerResource::EmittedTokens,
        2,
        1,
        1,
        "token refusal after prior run",
    );
    assert_eq!(chars(&run), vec![((0, 1), "a".to_owned())]);
    assert_eq!(run.coverage().processed_end(), 2);
}

/// Falsifies: the diagnostic disappearing on a later retained refusal, and a
/// retained cost of three (U+FFFD) instead of exactly one byte.
#[test]
fn dn12_retained_refusal_after_the_diagnostic_costs_exactly_one_byte() {
    let run = assert_resource(
        "\0",
        with(None, None, None, Some(0)),
        HtmlTokenizerResource::RetainedInterpretedBytes,
        1,
        0,
        1,
        "retained refusal",
    );
    assert_eq!(codes(&run), vec![(Diag::UnexpectedNullCharacter, (0, 1))]);
    assert_eq!(run.coverage().processed_end(), 1);

    // One byte already committed by `a`: attempted is 1 + 1, never 1 + 3.
    let run = assert_resource(
        "a\0",
        with(None, None, None, Some(1)),
        HtmlTokenizerResource::RetainedInterpretedBytes,
        2,
        1,
        1,
        "retained refusal after prior run",
    );
    assert_eq!(chars(&run), vec![((0, 1), "a".to_owned())]);
}

/// Falsifies: the retained check winning when both limits are exhausted. The
/// selected order after the diagnostic is EmittedTokens, then retained bytes.
#[test]
fn dn13_simultaneous_token_and_retained_shortage_reports_emitted_tokens() {
    // After `a` closes, one token and one retained byte are committed, so the
    // NUL's token (attempted 2) and retained byte (attempted 2) both overflow.
    let run = assert_resource(
        "a\0",
        with(None, Some(1), None, Some(1)),
        HtmlTokenizerResource::EmittedTokens,
        2,
        1,
        1,
        "both exhausted",
    );
    assert_eq!(codes(&run), vec![(Diag::UnexpectedNullCharacter, (1, 2))]);
    assert_eq!(chars(&run), vec![((0, 1), "a".to_owned())]);
}

// ---------------------------------------------------------------------------
// Tree: InBody ignore
// ---------------------------------------------------------------------------

/// Falsifies: InBody inserting U+0000 or U+FFFD, a tree diagnostic without
/// the private action, an action without the diagnostic.
#[test]
fn dt1_in_body_ignores_the_exact_nul_with_one_diagnostic_and_one_action() {
    // <body>0..6 NUL 6..7
    let analysis = analyze("<body>\0</body>");
    assert!(analysis.is_complete());
    assert!(texts(&analysis).is_empty(), "no text node at all");
    assert_eq!(
        null_diagnostics(&analysis),
        vec![(1, HtmlTreeRecovery::IgnoredToken)]
    );
    assert_eq!(ignored_actions(&analysis), vec![1]);
    no_nul_or_replacement_text(&analysis);

    // An implied body is reached through the same InBody rule.
    let analysis = analyze("\0");
    assert!(analysis.is_complete());
    assert!(texts(&analysis).is_empty());
    assert_eq!(
        null_diagnostics(&analysis),
        vec![(0, HtmlTreeRecovery::IgnoredToken)]
    );
    assert_eq!(ignored_actions(&analysis), vec![0]);
    // The tokenizer diagnostic and the tree diagnostic stay distinct facts.
    assert_eq!(
        codes(analysis.tokenizer_run()),
        vec![(Diag::UnexpectedNullCharacter, (0, 1))]
    );
}

/// Falsifies: a mixed token scanned for NUL, an aggregate Character token,
/// or ignored source leaking into a text contribution.
#[test]
fn dt2_text_around_an_ignored_nul_keeps_exact_contributions() {
    // <body>0..6 a6..7 NUL7..8 b8..9
    let analysis = analyze("<body>a\0b</body>");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["ab".to_owned()]);
    assert_eq!(
        contributions(&analysis),
        vec![((6, 7), "a".to_owned()), ((8, 9), "b".to_owned())]
    );
    assert_eq!(
        null_diagnostics(&analysis),
        vec![(2, HtmlTreeRecovery::IgnoredToken)]
    );
    assert_eq!(ignored_actions(&analysis), vec![2]);
    no_nul_or_replacement_text(&analysis);

    let analysis = analyze("<body>\0\0\0");
    assert_eq!(ignored_actions(&analysis), vec![1, 2, 3]);
    assert_eq!(null_diagnostics(&analysis).len(), 3);
    assert!(texts(&analysis).is_empty());
}

/// Falsifies: AfterBody special-casing NUL instead of reprocessing the same
/// token through InBody, or losing the AfterBodyCharacterData recovery.
#[test]
fn dt3_after_body_reprocesses_the_same_nul_through_in_body() {
    // <body>0..6 </body>6..13 NUL 13..14
    let analysis = analyze("<body></body>\0");
    assert!(analysis.is_complete());
    let relevant: Vec<(HtmlTreeDiagnosticCode, usize, HtmlTreeRecovery)> = analysis
        .diagnostics()
        .iter()
        .filter(|d| {
            matches!(
                d.code(),
                HtmlTreeDiagnosticCode::AfterBodyCharacterData
                    | HtmlTreeDiagnosticCode::NullCharacterInBody
            )
        })
        .map(|d| (d.code(), d.trigger().token_index(), d.recovery()))
        .collect();
    assert_eq!(
        relevant,
        vec![
            (
                HtmlTreeDiagnosticCode::AfterBodyCharacterData,
                2,
                HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken
            ),
            (
                HtmlTreeDiagnosticCode::NullCharacterInBody,
                2,
                HtmlTreeRecovery::IgnoredToken
            ),
        ]
    );
    assert_eq!(ignored_actions(&analysis), vec![2]);
    assert!(texts(&analysis).is_empty());
    assert!(
        analysis
            .actions()
            .iter()
            .any(|a| matches!(a.kind(), HtmlTreeActionKind::ReprocessedToken)
                && a.trigger().token_index() == 2),
        "the same token is reprocessed"
    );
}

/// Falsifies: the AfterAfterBody recovery replacing the InBody NUL ignore
/// fact, or a recovered authored NUL becoming text. The earlier "AfterAfterBody
/// is not widened" control is superseded by #888.
#[test]
fn dt4_after_after_body_recovery_composes_with_the_in_body_nul_ignore() {
    let analysis = analyze("<body></body></html>\0");
    assert!(analysis.is_complete());
    assert_eq!(ignored_actions(&analysis), vec![3]);
    assert_eq!(
        null_diagnostics(&analysis),
        vec![(3, HtmlTreeRecovery::IgnoredToken)]
    );
    let recoveries: Vec<_> = analysis
        .diagnostics()
        .iter()
        .filter(|d| d.code() == HtmlTreeDiagnosticCode::AfterAfterBodyCharacterData)
        .map(|d| (d.trigger().token_index(), d.recovery()))
        .collect();
    assert_eq!(
        recoveries,
        vec![(3, HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken)]
    );
    assert!(texts(&analysis).is_empty());
    no_nul_or_replacement_text(&analysis);
}

/// Falsifies: decoded Numeric U+FFFD being treated as authored NUL, or
/// Numeric zero changing to U+0000, in the selected tree.
#[test]
fn dt5_numeric_zero_is_text_not_an_ignored_nul() {
    for text in ["<body>&#0;", "<body>&#x0;"] {
        let analysis = analyze(text);
        assert!(analysis.is_complete(), "{text:?}");
        assert_eq!(texts(&analysis), vec!["\u{fffd}".to_owned()], "{text:?}");
        assert!(ignored_actions(&analysis).is_empty(), "{text:?}");
        assert!(null_diagnostics(&analysis).is_empty(), "{text:?}");
        assert!(
            analysis
                .tokenizer_run()
                .diagnostics()
                .iter()
                .any(|d| d.code() == Diag::NullCharacterReference),
            "{text:?}"
        );
    }
    let analysis = analyze("<body>&#65;\0b");
    assert_eq!(texts(&analysis), vec!["Ab".to_owned()]);
    assert_eq!(ignored_actions(&analysis).len(), 1);
}

// ---------------------------------------------------------------------------
// Dense NUL boundary under the fixed Product resource vector
// ---------------------------------------------------------------------------

/// Falsifies: a new tree resource dimension, a 257th token, upgraded lower
/// layer incompleteness, or NUL/U+FFFD text from the ignored NULs.
#[test]
fn dt6_dense_nul_is_bounded_by_the_tokenizer_diagnostic_limit() {
    // The fixed Product vector (tree.rs): source 36_864, steps 79_872,
    // tokens 6_144, diagnostics 256, attributes 1, retained 49_152, temp 0.
    let product: Limits = (36_864, 79_872, 6_144, 256, 1, 49_152, 0);
    let analysis = analyze_with(&format!("<body>{}", "\0".repeat(257)), product);

    let run = analysis.tokenizer_run();
    let (resource, limit, attempted, at) = stop(run).expect("tokenizer resource stop");
    assert_eq!(resource, HtmlTokenizerResource::Diagnostics);
    assert_eq!((limit, attempted), (256, 257));
    assert_eq!(at, (262, 263), "exact authored 257th NUL");

    assert_eq!(run.diagnostics().len(), 256);
    assert_eq!(chars(run).len(), 256, "no 257th U+0000 token");
    assert!(chars(run).iter().all(|(_, t)| t == "\0"));
    assert_eq!(ignored_actions(&analysis).len(), 256);
    assert_eq!(null_diagnostics(&analysis).len(), 256);
    assert!(texts(&analysis).is_empty());
    no_nul_or_replacement_text(&analysis);
    assert!(matches!(
        analysis.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));
}

// ---------------------------------------------------------------------------
// Freeze theorem: ignored-NUL evidence cannot be fabricated
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

    fn anchor(&self, start: usize, end: usize) -> crate::SourceAnchor {
        source(self.text).anchor(start, end).expect("valid range")
    }

    fn token_trigger(&self, index: usize) -> HtmlTreeTokenTrigger {
        let boundary = match &self.analysis.tokenizer_run().tokens()[index] {
            HtmlToken::Character(c) => c.source().clone(),
            HtmlToken::Tag(t) => t.complete().clone(),
            HtmlToken::Doctype(d) => d.complete().clone(),
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
        let run = lex(self.text);
        freeze(&source(self.text), run, parts).map(|_| ())
    }
}

fn is_ignored(action: &HtmlTreeAction) -> bool {
    matches!(action.kind(), HtmlTreeActionKind::IgnoredNullCharacterToken)
}

fn is_null_diagnostic(diagnostic: &HtmlTreeDiagnostic) -> bool {
    diagnostic.code() == HtmlTreeDiagnosticCode::NullCharacterInBody
}

/// The unmodified reconstruction freezes, so every rejection below is caused
/// by the mutation alone.
#[test]
fn df0_the_unmutated_reconstruction_freezes() {
    for text in ["<body>a\0b", "<body>\0", "<body>\0\0"] {
        let frozen = Frozen::new(text);
        assert_eq!(frozen.freeze(frozen.parts()), Ok(()), "{text:?}");
    }
}

fn mismatch(actions: &[usize], diagnostics: &[usize]) -> HtmlTreeFreezeError {
    HtmlTreeFreezeError::IgnoredNullCharacterDiagnosticMismatch {
        actions: actions.to_vec(),
        diagnostics: diagnostics.to_vec(),
    }
}

/// Falsifies: a freeze that accepts a missing or orphaned half of the
/// action/diagnostic pair.
#[test]
fn df1_missing_and_orphan_halves_are_rejected() {
    let frozen = Frozen::new("<body>a\0b");
    // 1. missing action (the diagnostic is now an orphan).
    let mut parts = frozen.parts();
    parts.actions.retain(|a| !is_ignored(a));
    assert_eq!(frozen.freeze(parts), Err(mismatch(&[], &[2])));
    // 2. missing diagnostic.
    let mut parts = frozen.parts();
    parts.diagnostics.retain(|d| !is_null_diagnostic(d));
    assert_eq!(frozen.freeze(parts), Err(mismatch(&[2], &[])));
    // 3. orphan diagnostic on a different ordinary token.
    let mut parts = frozen.parts();
    parts.diagnostics.push(HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::NullCharacterInBody,
        frozen.token_trigger(3),
        HtmlTreeRecovery::IgnoredToken,
    ));
    assert_eq!(frozen.freeze(parts), Err(mismatch(&[2], &[2, 3])));
}

/// Falsifies: a freeze that accepts a repeated ignore decision.
#[test]
fn df2_duplicates_are_rejected() {
    let frozen = Frozen::new("<body>a\0b");
    // 4. duplicate action.
    let mut parts = frozen.parts();
    let action = parts
        .actions
        .iter()
        .find(|a| is_ignored(a))
        .cloned()
        .unwrap();
    let at = parts.actions.iter().position(is_ignored).unwrap();
    parts.actions.insert(at, action);
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DuplicateIgnoredNullCharacterDecision { token_index: 2 })
    );
    // 5. duplicate diagnostic.
    let mut parts = frozen.parts();
    let diagnostic = parts
        .diagnostics
        .iter()
        .find(|d| is_null_diagnostic(d))
        .cloned()
        .unwrap();
    parts.diagnostics.push(diagnostic);
    assert_eq!(frozen.freeze(parts), Err(mismatch(&[2], &[2, 2])));
}

/// Falsifies: a freeze that accepts an ignore decision whose trigger is not
/// the exact authored U+0000 Character token.
#[test]
fn df3_wrong_non_nul_and_non_character_triggers_are_rejected() {
    // 6. action pointing at the wrong (ordinary Character) token.
    let frozen = Frozen::new("<body>a\0b");
    for wrong in [1, 3] {
        let mut parts = frozen.parts();
        let trigger = frozen.token_trigger(wrong);
        let action = parts.actions.iter_mut().find(|a| is_ignored(a)).unwrap();
        *action = HtmlTreeAction::new(HtmlTreeActionKind::IgnoredNullCharacterToken, trigger);
        assert_eq!(
            frozen.freeze(parts),
            Err(
                HtmlTreeFreezeError::IgnoredNullCharacterTriggerIsNotExactNull {
                    token_index: wrong
                }
            ),
            "token {wrong}"
        );
    }
    // 7. diagnostic pointing at the wrong token.
    let mut parts = frozen.parts();
    let trigger = frozen.token_trigger(1);
    let diagnostic = parts
        .diagnostics
        .iter_mut()
        .find(|d| is_null_diagnostic(d))
        .unwrap();
    *diagnostic = HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::NullCharacterInBody,
        trigger,
        HtmlTreeRecovery::IgnoredToken,
    );
    assert_eq!(frozen.freeze(parts), Err(mismatch(&[2], &[1])));
    // 8. action pointing at a non-Character (tag) token.
    let frozen = Frozen::new("<body>\0");
    let mut parts = frozen.parts();
    let trigger = frozen.token_trigger(0);
    let action = parts.actions.iter_mut().find(|a| is_ignored(a)).unwrap();
    *action = HtmlTreeAction::new(HtmlTreeActionKind::IgnoredNullCharacterToken, trigger);
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::IgnoredNullCharacterTriggerIsNotExactNull { token_index: 0 })
    );
    // An action whose boundary is not the token's exact authored evidence.
    let mut parts = frozen.parts();
    let action = parts.actions.iter_mut().find(|a| is_ignored(a)).unwrap();
    *action = HtmlTreeAction::new(
        HtmlTreeActionKind::IgnoredNullCharacterToken,
        HtmlTreeTokenTrigger::authored(1, frozen.anchor(6, 7).clone()),
    );
    // (anchor 6..7 is the NUL's own range, so this is the unmutated shape.)
    assert_eq!(frozen.freeze(parts), Ok(()));
    let mut parts = frozen.parts();
    let action = parts.actions.iter_mut().find(|a| is_ignored(a)).unwrap();
    *action = HtmlTreeAction::new(
        HtmlTreeActionKind::IgnoredNullCharacterToken,
        HtmlTreeTokenTrigger::authored(1, frozen.anchor(0, 7)),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::IgnoredNullCharacterTriggerIsNotExactNull { token_index: 1 })
    );
}

/// Falsifies: a freeze that accepts a non-IgnoredToken recovery.
#[test]
fn df4_wrong_recovery_is_rejected() {
    let frozen = Frozen::new("<body>a\0b");
    let mut parts = frozen.parts();
    let trigger = frozen.token_trigger(2);
    let diagnostic = parts
        .diagnostics
        .iter_mut()
        .find(|d| is_null_diagnostic(d))
        .unwrap();
    *diagnostic = HtmlTreeDiagnostic::new(
        HtmlTreeDiagnosticCode::NullCharacterInBody,
        trigger,
        HtmlTreeRecovery::SwitchedToInBodyAndReprocessedSameToken,
    );
    assert_eq!(frozen.freeze(parts), Err(mismatch(&[2], &[2])));
}

/// Falsifies: a freeze that accepts the ignored NUL (or a fabricated U+FFFD
/// in its place) as a final text contribution.
#[test]
fn df5_ignored_nul_leaking_into_text_is_rejected() {
    let frozen = Frozen::new("<body>a\0b");
    for leaked in ["\0", "\u{fffd}"] {
        let mut parts = frozen.parts();
        let text_id: HtmlConstructedNodeId = parts
            .nodes
            .iter()
            .find(|n| matches!(n.kind(), HtmlTreeNodeKind::Text(_)))
            .map(HtmlTreeNode::id)
            .expect("text node");
        for node in &mut parts.nodes {
            if node.id() != text_id {
                continue;
            }
            let rebuilt = HtmlTextNode::new(
                format!("a{leaked}b"),
                vec![
                    HtmlTextContribution::new(frozen.anchor(6, 7), "a".to_owned()),
                    HtmlTextContribution::new(frozen.anchor(7, 8), leaked.to_owned()),
                    HtmlTextContribution::new(frozen.anchor(8, 9), "b".to_owned()),
                ],
            );
            *node = HtmlTreeNode::new(
                node.id(),
                node.parent(),
                node.children().to_vec(),
                HtmlTreeNodeKind::Text(rebuilt),
            );
        }
        assert_eq!(
            frozen.freeze(parts),
            Err(HtmlTreeFreezeError::IgnoredNullCharacterLeakedIntoText {
                token_index: 2,
                node: text_id
            }),
            "{leaked:?}"
        );
    }
}
