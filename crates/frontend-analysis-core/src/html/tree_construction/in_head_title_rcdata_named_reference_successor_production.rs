//! Production correspondence for TC-S10 — selected InHead `<title>` RCDATA +
//! Named Character-Reference causal lifecycle (Issue #394).
//!
//! Expectations here are independently authored from the accepted production
//! theorem. The tests import only production tokenizer, tree-construction,
//! durable-result, and freeze seams. They do not import, call, or copy the
//! candidate-independent #390 validation machine, do not derive any expected
//! value from production output, and use no browser, WPT, or html5lib result
//! as an oracle.
//!
//! The module is organized as falsification first: each `pf*` test names an
//! invalid implementation strategy and proves production does not use it. The
//! semantic, resource, and freeze matrices then pin the accepted behaviour.

use crate::{SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::diagnostic::{
    HtmlTokenizerDiagnosticCode, HtmlTokenizerDiagnosticContext, HtmlTokenizerDiagnosticHandling,
    HtmlTokenizerDiagnosticSubject,
};
use super::super::tokenizer::producer::{
    HtmlTokenizerSession, HtmlTokenizerSessionBoundary, tokenize,
};
use super::super::tokenizer::resource::{
    HtmlTokenizerLimits, HtmlTokenizerResource, HtmlTokenizerResourceLimit,
};
use super::super::tokenizer::result::{
    HtmlTokenizerCapability, HtmlTokenizerCapabilityAvailability, HtmlTokenizerCompletion,
    HtmlTokenizerIncompleteCause, HtmlTokenizerMode, HtmlTokenizerRunResult,
    HtmlTokenizerUnsupportedTrigger,
};
use super::driver::construct_html_document_shell;
use super::result::{
    HtmlConstructedNodeId, HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlElement,
    HtmlElementName, HtmlTreeAction, HtmlTreeActionKind, HtmlTreeCapability, HtmlTreeCompletion,
    HtmlTreeDiagnosticCode, HtmlTreeFreezeError, HtmlTreeIncompleteCause, HtmlTreeNodeKind,
    HtmlTreeRecovery, freeze,
};
use super::session::{
    DispatchOutcome, HtmlTreeSession, HtmlTreeTokenizerFeedback, InsertionMode, admit,
    token_trigger,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

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
    construct_html_document_shell(&source, limits).expect("TC-S10 production boundary")
}

fn title_id(analysis: &HtmlDocumentShellAnalysis) -> HtmlConstructedNodeId {
    title_ids(analysis)
        .first()
        .copied()
        .expect("a Title node exists")
}

fn title_ids(analysis: &HtmlDocumentShellAnalysis) -> Vec<HtmlConstructedNodeId> {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .filter(|node| {
            matches!(
                node.kind(),
                HtmlTreeNodeKind::Element(HtmlElement::Title(_))
            )
        })
        .map(|node| node.id())
        .collect()
}

/// One retained authored text contribution as `(start, end, interpreted)`.
type ContributionProjection = (usize, usize, String);

/// One durable text node as its interpreted value plus its ordered authored
/// contributions.
type TextNodeProjection = (String, Vec<ContributionProjection>);

/// The Title's child text nodes, in child order.
fn title_text_nodes(
    analysis: &HtmlDocumentShellAnalysis,
    title: HtmlConstructedNodeId,
) -> Vec<TextNodeProjection> {
    analysis
        .node(title)
        .expect("Title node")
        .children()
        .iter()
        .map(
            |child| match analysis.node(*child).expect("Title child").kind() {
                HtmlTreeNodeKind::Text(text) => (
                    text.interpreted().to_owned(),
                    text.contributions()
                        .iter()
                        .map(|contribution| {
                            (
                                contribution.source().range().start(),
                                contribution.source().range().end(),
                                contribution.interpreted().to_owned(),
                            )
                        })
                        .collect(),
                ),
                other => panic!("a Title child must be text, got {other:?}"),
            },
        )
        .collect()
}

/// The single coalesced Title text node's interpreted value and ordered
/// authored contributions.
fn title_text(analysis: &HtmlDocumentShellAnalysis) -> TextNodeProjection {
    let nodes = title_text_nodes(analysis, title_id(analysis));
    assert_eq!(nodes.len(), 1, "expected exactly one coalesced Title text");
    nodes.into_iter().next().expect("checked non-empty")
}

/// The durable subject a diagnostic relates itself to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Subject {
    InputLocation,
    EmittedToken(usize),
}

/// One tokenizer diagnostic as `(code, context, handling, subject, start, end)`.
type DiagnosticProjection = (
    HtmlTokenizerDiagnosticCode,
    HtmlTokenizerDiagnosticContext,
    HtmlTokenizerDiagnosticHandling,
    Subject,
    usize,
    usize,
);

fn tokenizer_diagnostics(analysis: &HtmlDocumentShellAnalysis) -> Vec<DiagnosticProjection> {
    analysis
        .tokenizer_run()
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            let subject = match diagnostic.subject() {
                HtmlTokenizerDiagnosticSubject::InputLocation => Subject::InputLocation,
                HtmlTokenizerDiagnosticSubject::EmittedToken { token_index } => {
                    Subject::EmittedToken(*token_index)
                }
                HtmlTokenizerDiagnosticSubject::AbandonedInput { .. } => {
                    panic!("no selected TC-S10 diagnostic abandons input")
                }
            };
            (
                diagnostic.code(),
                diagnostic.context(),
                diagnostic.handling(),
                subject,
                diagnostic.location().range().start(),
                diagnostic.location().range().end(),
            )
        })
        .collect()
}

/// Every emitted tokenizer token as `(kind, start, end, interpreted)`, where
/// `kind` is `"start"`, `"end"`, `"chars"`, or `"eof"`.
fn tokens(analysis: &HtmlDocumentShellAnalysis) -> Vec<(&'static str, usize, usize, String)> {
    analysis
        .tokenizer_run()
        .tokens()
        .iter()
        .map(|token| match token {
            HtmlToken::Tag(tag) => (
                match tag.kind() {
                    HtmlTagKind::Start => "start",
                    HtmlTagKind::End => "end",
                },
                tag.complete().range().start(),
                tag.complete().range().end(),
                tag.name().interpreted().to_owned(),
            ),
            HtmlToken::Character(character) => (
                "chars",
                character.source().range().start(),
                character.source().range().end(),
                character.interpreted().to_owned(),
            ),
            HtmlToken::EndOfFile(eof) => (
                "eof",
                eof.source().range().start(),
                eof.source().range().end(),
                String::new(),
            ),
        })
        .collect()
}

fn tokenizer_unsupported(
    analysis: &HtmlDocumentShellAnalysis,
) -> Option<(
    HtmlTokenizerCapability,
    HtmlTokenizerCapabilityAvailability,
    usize,
    usize,
)> {
    match analysis.tokenizer_run().completion() {
        HtmlTokenizerCompletion::Incomplete(
            HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported),
        ) => {
            let HtmlTokenizerUnsupportedTrigger::Input(anchor) = unsupported.trigger() else {
                panic!("a selected TC-S10 refusal is an input trigger");
            };
            Some((
                unsupported.capability(),
                unsupported.availability(),
                anchor.range().start(),
                anchor.range().end(),
            ))
        }
        _ => None,
    }
}

fn tokenizer_resource_limit(analysis: &HtmlDocumentShellAnalysis) -> HtmlTokenizerResourceLimit {
    match analysis.tokenizer_run().completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(limit)) => {
            limit.clone()
        }
        other => panic!("expected a resource-limit refusal, got {other:?}"),
    }
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

fn element_name(
    analysis: &HtmlDocumentShellAnalysis,
    id: HtmlConstructedNodeId,
) -> Option<HtmlElementName> {
    match analysis.node(id)?.kind() {
        HtmlTreeNodeKind::Element(element) => Some(element.name()),
        HtmlTreeNodeKind::Document | HtmlTreeNodeKind::Text(_) => None,
    }
}

fn action_token(
    analysis: &HtmlDocumentShellAnalysis,
    predicate: impl Fn(&HtmlTreeActionKind) -> bool,
) -> usize {
    analysis
        .actions()
        .iter()
        .find(|action| predicate(action.kind()))
        .map(|action| action.trigger().token_index())
        .expect("expected Title action")
}

fn insert_token(analysis: &HtmlDocumentShellAnalysis) -> usize {
    action_token(analysis, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::InsertedAuthoredTitleElement { .. }
        )
    })
}

fn close_token(analysis: &HtmlDocumentShellAnalysis) -> usize {
    action_token(analysis, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { .. }
        )
    })
}

// ---------------------------------------------------------------------------
// PF1 — Title source may be tokenized under Data before feedback
// ---------------------------------------------------------------------------

#[test]
fn pf1_no_post_title_source_is_produced_before_the_title_feedback_is_applied() {
    let source = SourceText::new(SourceId::new(1), "<title>&amp;x</title>".to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&source, limits());

    // The Engine stops at the exact post-start-tag boundary, having produced
    // the start tag and nothing after it.
    let boundary = tokenizer.drive_to_boundary();
    assert_eq!(
        boundary,
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata)
    );
    assert_eq!(tokenizer.tokens().len(), 1);
    let HtmlToken::Tag(start) = &tokenizer.tokens()[0] else {
        panic!("the first token is the authored Title start tag");
    };
    assert_eq!(start.kind(), HtmlTagKind::Start);
    assert_eq!(start.name().interpreted(), "title");
    assert_eq!(start.complete().range().end(), 7);

    // Driving again without applying feedback produces no further tokens:
    // suspension is a real causal stop, not a hint.
    assert_eq!(
        tokenizer.drive_to_boundary(),
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata)
    );
    assert_eq!(tokenizer.tokens().len(), 1);

    // Only after the selected control is applied does later source flow — and
    // it flows as RCDATA, not Data.
    tokenizer.apply_rcdata().expect("apply selected RCDATA");
    tokenizer.drive_to_boundary();
    assert!(tokenizer.tokens().len() > 1);

    // Under the whole coordinated run the tree really did insert Title first.
    let analysis = analyze("<title>&amp;x</title>");
    assert_eq!(insert_token(&analysis), 0);
}

#[test]
fn pf1_a_title_feedback_request_cannot_be_answered_by_the_wrong_control() {
    let source = SourceText::new(SourceId::new(1), "<title>x</title>".to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&source, limits());
    assert_eq!(
        tokenizer.drive_to_boundary(),
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata)
    );
    // The RAWTEXT control is a different selected lifecycle and is refused.
    assert!(tokenizer.apply_raw_text().is_err());
    assert_eq!(tokenizer.tokens().len(), 1);
    tokenizer
        .apply_rcdata()
        .expect("the matching control applies");
}

// ---------------------------------------------------------------------------
// PF2 — RCDATA is equivalent to RAWTEXT
// ---------------------------------------------------------------------------

#[test]
fn pf2_rcdata_resolves_references_where_rawtext_keeps_them_literal() {
    let title = analyze("<title>&amp;</title>");
    assert_eq!(title_text(&title).0, "&");

    // The same authored bytes inside the selected RAWTEXT lifecycle stay
    // literal: RCDATA and RAWTEXT are not the same state.
    let style = analyze("<style>&amp;</style>");
    let style_id = style
        .nodes_in_creation_order()
        .into_iter()
        .find(|node| {
            matches!(
                node.kind(),
                HtmlTreeNodeKind::Element(HtmlElement::Style(_))
            )
        })
        .expect("Style node")
        .id();
    let style_text: String = style
        .node(style_id)
        .expect("Style node")
        .children()
        .iter()
        .map(
            |child| match style.node(*child).expect("Style child").kind() {
                HtmlTreeNodeKind::Text(text) => text.interpreted().to_owned(),
                other => panic!("Style child must be text, got {other:?}"),
            },
        )
        .collect();
    assert_eq!(style_text, "&amp;");
}

// ---------------------------------------------------------------------------
// PF3 — exact whole-name entity lookup is enough
// ---------------------------------------------------------------------------

#[test]
fn pf3_maximum_match_distinguishes_notit_from_notin() {
    // `notit` and `notit;` are not identifiers, so a whole-string lookup
    // resolves nothing. Maximum match resolves the semicolonless `not` and
    // leaves `it;` as ordinary later RCDATA input.
    let short = analyze("<title>&notit;</title>");
    let (interpreted, contributions) = title_text(&short);
    assert_eq!(interpreted, "\u{ac}it;");
    assert_eq!(
        contributions,
        vec![(7, 11, "\u{ac}".to_owned()), (11, 14, "it;".to_owned()),]
    );

    // The longer identifier wins where it exists.
    let long = analyze("<title>&notin;</title>");
    let (interpreted, contributions) = title_text(&long);
    assert_eq!(interpreted, "\u{2209}");
    assert_eq!(contributions, vec![(7, 14, "\u{2209}".to_owned())]);
}

#[test]
fn pf3_a_semicolonless_match_reports_the_missing_semicolon_at_its_own_last_scalar() {
    let analysis = analyze("<title>&notit;</title>");
    // The durable relation is the resolved Character token this diagnostic is
    // about — token 1, the `¬` emitted from `&not` — while the anchor stays
    // the matched identifier's own last authored scalar.
    assert_eq!(
        tokenizer_diagnostics(&analysis),
        vec![(
            HtmlTokenizerDiagnosticCode::MissingSemicolonAfterCharacterReference,
            HtmlTokenizerDiagnosticContext::NamedCharacterReference,
            HtmlTokenizerDiagnosticHandling::Continued,
            Subject::EmittedToken(1),
            // `&notit;` — the `t` of the matched `not`, at index 3 from the
            // `&` at index 7. Never the later unconsumed `i`, `t`, or `;`.
            10,
            11,
        )]
    );
    // The referenced token really is the resolved reference, not the tag.
    assert_eq!(tokens(&analysis)[1], ("chars", 7, 11, "\u{ac}".to_owned()));
    // MissingSemicolon was not added to the predecessor end-tag
    // emission-conditioned subset; it stays observation-conditioned.
    assert!(
        !HtmlTokenizerDiagnosticCode::MissingSemicolonAfterCharacterReference
            .is_emission_conditioned()
    );
    // A fully terminated match reports nothing.
    assert!(tokenizer_diagnostics(&analyze("<title>&notin;</title>")).is_empty());
}

// ---------------------------------------------------------------------------
// PF4 — lookahead may mutate cursor/preprocessing state
// ---------------------------------------------------------------------------

#[test]
fn pf4_maximum_match_lookahead_commits_no_early_preprocessing_observation() {
    // The control scalar sits immediately after the matched `not`, inside the
    // window a maximum-match lookahead must examine. Its preprocessing
    // diagnostic must still be observed *after* the missing-semicolon
    // diagnostic, in true source-observation order.
    let analysis = analyze("<title>&not\u{0001}</title>");
    assert_eq!(
        tokenizer_diagnostics(&analysis),
        vec![
            (
                HtmlTokenizerDiagnosticCode::MissingSemicolonAfterCharacterReference,
                HtmlTokenizerDiagnosticContext::NamedCharacterReference,
                HtmlTokenizerDiagnosticHandling::Continued,
                Subject::EmittedToken(1),
                10,
                11,
            ),
            (
                HtmlTokenizerDiagnosticCode::ControlCharacterInInputStream,
                HtmlTokenizerDiagnosticContext::InputPreprocessing,
                HtmlTokenizerDiagnosticHandling::Continued,
                Subject::InputLocation,
                11,
                12,
            ),
        ]
    );
}

#[test]
fn pf4_a_refused_named_commit_leaves_no_speculative_coverage_or_evidence() {
    // The maximum match over `amp;` is discovered, then refused before any
    // authoritative consumption. Coverage must stop at the authored `&`,
    // which the tokenizer really did consume, and never at the discovered
    // match endpoint.
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 4_096, 256, 5, 4_096);
    let analysis = analyze_with("<title>&amp;</title>", 1, constrained);
    let limit = tokenizer_resource_limit(&analysis);
    assert_eq!(
        limit.resource(),
        HtmlTokenizerResource::RetainedInterpretedBytes
    );
    assert_eq!(analysis.tokenizer_run().coverage().processed_end(), 8);
    assert_eq!(analysis.tokenizer_run().tokens().len(), 1);
    assert!(analysis.tokenizer_run().diagnostics().is_empty());
    assert_eq!(
        analysis
            .tokenizer_run()
            .usage()
            .peak_temporary_buffer_bytes(),
        0
    );
}

// ---------------------------------------------------------------------------
// PF5 — decoded text may become markup
// ---------------------------------------------------------------------------

#[test]
fn pf5_decoded_syntax_is_interpreted_text_and_never_authored_markup() {
    // The decoded `<` completes a `</title>`-shaped string, but only the
    // *second*, authored `</title>` closes the element.
    let analysis = analyze("<title>&lt;/title></title>");
    let (interpreted, contributions) = title_text(&analysis);
    assert_eq!(interpreted, "</title>");
    assert_eq!(
        contributions,
        vec![(7, 11, "<".to_owned()), (11, 18, "/title>".to_owned()),]
    );
    // The authored close is the trailing one, at 18..26.
    let close = &analysis.tokenizer_run().tokens()[close_token(&analysis)];
    let HtmlToken::Tag(tag) = close else {
        panic!("the Title close is a tag token");
    };
    assert_eq!(tag.kind(), HtmlTagKind::End);
    assert_eq!(tag.complete().range().start(), 18);
    assert_eq!(tag.complete().range().end(), 26);
    assert!(analysis.is_complete());
}

// ---------------------------------------------------------------------------
// PF6 — decoded references may recursively decode
// ---------------------------------------------------------------------------

#[test]
fn pf6_decoded_output_never_re_enters_tokenizer_input() {
    // `&amp;lt;` decodes to `&` once. If decoded output re-entered input, the
    // resulting `&lt;` would decode again to `<`.
    let analysis = analyze("<title>&amp;lt;</title>");
    let (interpreted, contributions) = title_text(&analysis);
    assert_eq!(interpreted, "&lt;");
    assert_ne!(interpreted, "<");
    assert_eq!(
        contributions,
        vec![(7, 12, "&".to_owned()), (12, 15, "lt;".to_owned()),]
    );
}

// ---------------------------------------------------------------------------
// PF7 — one reference means one Unicode scalar
// ---------------------------------------------------------------------------

#[test]
fn pf7_one_reference_may_decode_to_two_scalars_under_one_contribution() {
    let analysis = analyze("<title>&acE;</title>");
    let (interpreted, contributions) = title_text(&analysis);
    assert_eq!(interpreted, "\u{223e}\u{333}");
    assert_eq!(interpreted.chars().count(), 2);
    // Exactly one authored contribution covering the exact authored bytes:
    // multi-scalar output never subdivides or fabricates source.
    assert_eq!(contributions, vec![(7, 12, "\u{223e}\u{333}".to_owned())]);
}

// ---------------------------------------------------------------------------
// PF8 — an unknown reference may be flattened with no causal boundary
// ---------------------------------------------------------------------------

#[test]
fn pf8_an_unresolved_reference_keeps_its_own_contribution_boundary() {
    let analysis = analyze("<title>&bogus;</title>");
    let (interpreted, contributions) = title_text(&analysis);
    // Final text coalesces...
    assert_eq!(interpreted, "&bogus;");
    // ...but the unresolved authored contribution and the diagnostic's own
    // semicolon stay separately inspectable.
    assert_eq!(
        contributions,
        vec![(7, 13, "&bogus".to_owned()), (13, 14, ";".to_owned()),]
    );
    assert_eq!(
        tokenizer_diagnostics(&analysis),
        vec![(
            HtmlTokenizerDiagnosticCode::UnknownNamedCharacterReference,
            HtmlTokenizerDiagnosticContext::AmbiguousAmpersand,
            HtmlTokenizerDiagnosticHandling::Continued,
            // Purely an input observation: nothing was resolved, so there is
            // no emitted token for it to be about.
            Subject::InputLocation,
            // The authored `;` observed in Ambiguous Ampersand.
            13,
            14,
        )]
    );
    assert!(analysis.is_complete());
}

#[test]
fn pf8_an_unresolved_reference_without_a_semicolon_reports_nothing() {
    // The diagnostic is caused by the `;` observation, not by the failure to
    // resolve. Without a `;` there is no unknown-name diagnostic at all.
    let analysis = analyze("<title>&bogus</title>");
    assert!(tokenizer_diagnostics(&analysis).is_empty());
    let (interpreted, contributions) = title_text(&analysis);
    assert_eq!(interpreted, "&bogus");
    assert_eq!(contributions, vec![(7, 13, "&bogus".to_owned())]);
}

// ---------------------------------------------------------------------------
// PF9 — Numeric Character References may be partially implemented incidentally
// ---------------------------------------------------------------------------

#[test]
fn pf9_the_numeric_branch_is_a_narrow_boundary_reached_after_successful_entry() {
    let analysis = analyze("<title>&#60;</title>");

    // Title admission succeeded and produced a real Title node.
    assert_eq!(title_ids(&analysis).len(), 1);
    assert_eq!(
        element_name(&analysis, title_id(&analysis)),
        Some(HtmlElementName::Title)
    );
    // RCDATA entry succeeded, and Character Reference entry was reached: the
    // authored `&` at 7..8 is committed coverage.
    assert_eq!(analysis.tokenizer_run().coverage().processed_end(), 8);
    // The `#` alone is the unsupported trigger, under the narrow Numeric
    // meaning — not "RCDATA" and not "Character Reference".
    assert_eq!(
        tokenizer_unsupported(&analysis),
        Some((
            HtmlTokenizerCapability::NumericCharacterReferenceInRcdata,
            // Unsupported, not Deferred: this bounded machine refuses the
            // Numeric branch outright. It is a different claim from the
            // standalone context-dependent boundaries, which stay Deferred
            // because tree feedback really can discharge them.
            HtmlTokenizerCapabilityAvailability::Unsupported,
            8,
            9,
        ))
    );
    assert!(!analysis.is_complete());
    // Nothing numeric was decoded.
    assert!(title_text_nodes(&analysis, title_id(&analysis)).is_empty());
}

// ---------------------------------------------------------------------------
// PF10 — RCDATA NUL recovery may be implemented incidentally
// ---------------------------------------------------------------------------

#[test]
fn pf10_an_rcdata_nul_stops_at_its_own_narrow_boundary_without_recovery() {
    let analysis = analyze("<title>ab\u{0}cd</title>");
    assert_eq!(
        tokenizer_unsupported(&analysis),
        Some((
            HtmlTokenizerCapability::RcdataNullRecovery,
            HtmlTokenizerCapabilityAvailability::Unsupported,
            9,
            10,
        ))
    );
    // Prior valid evidence survives, and coverage stops exactly at the NUL.
    assert_eq!(analysis.tokenizer_run().coverage().processed_end(), 9);
    assert_eq!(title_text(&analysis).0, "ab");
    // No U+FFFD, and no RCDATA-NUL recovery diagnostic.
    assert!(!title_text(&analysis).0.contains('\u{fffd}'));
    assert!(analysis.tokenizer_run().diagnostics().is_empty());
    assert!(!analysis.is_complete());
}

// ---------------------------------------------------------------------------
// PF11 — a Named token may commit before its diagnostic capacity is guaranteed
// ---------------------------------------------------------------------------

#[test]
fn pf11_a_named_commit_refuses_whole_when_its_required_diagnostic_cannot_be_retained() {
    // `&notit;` resolves only together with a missing-semicolon diagnostic.
    // With no diagnostic capacity the whole semantic commit must be refused:
    // no resolved token, no diagnostic, no consumed match.
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 0, 256, 16_384, 4_096);
    let analysis = analyze_with("<title>&notit;</title>", 1, constrained);
    let limit = tokenizer_resource_limit(&analysis);
    assert_eq!(limit.resource(), HtmlTokenizerResource::Diagnostics);
    assert_eq!(limit.limit(), 0);
    assert_eq!(limit.attempted(), 1);
    assert!(analysis.tokenizer_run().diagnostics().is_empty());
    // Only the Title start tag committed; no partial `¬` character token.
    assert_eq!(analysis.tokenizer_run().tokens().len(), 1);
    assert_eq!(analysis.tokenizer_run().coverage().processed_end(), 8);
    assert!(title_text_nodes(&analysis, title_id(&analysis)).is_empty());
}

#[test]
fn pf11_a_fully_terminated_reference_needs_no_diagnostic_capacity() {
    // The diagnostics preflight is required only when the semantic result
    // requires a diagnostic, so `&amp;` still resolves at zero capacity.
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 0, 256, 16_384, 4_096);
    let analysis = analyze_with("<title>&amp;</title>", 1, constrained);
    assert!(analysis.is_complete());
    assert_eq!(title_text(&analysis).0, "&");
}

// ---------------------------------------------------------------------------
// PF12 — generic RCDATA control is justified
// ---------------------------------------------------------------------------

#[test]
fn pf12_the_selected_discharge_is_title_specific_and_not_general_rcdata() {
    // The standalone tokenizer's context-dependent boundary is unchanged for
    // every RCDATA element, including `<title>` itself.
    for (source, expected_end) in [("<title>x", 7usize), ("<textarea>x", 10usize)] {
        let source_text = SourceText::new(SourceId::new(1), source.to_owned());
        let run = tokenize(&source_text, limits());
        let HtmlTokenizerCompletion::Incomplete(
            HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported),
        ) = run.completion()
        else {
            panic!("{source}: the standalone tokenizer still defers");
        };
        assert_eq!(
            unsupported.capability(),
            HtmlTokenizerCapability::ContextDependentTokenizerMode {
                mode: HtmlTokenizerMode::Rcdata
            }
        );
        assert_eq!(
            unsupported.availability(),
            HtmlTokenizerCapabilityAvailability::Deferred
        );
        assert_eq!(run.coverage().processed_end(), expected_end);
    }

    // Coordinated, `<textarea>` is still refused by the tree before any
    // tokenizer control is applied, so its RCDATA is never entered.
    let textarea = analyze("<textarea>x</textarea>");
    assert_eq!(
        tree_unsupported(&textarea),
        Some(HtmlTreeCapability::NonShellElementTag)
    );
    let HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
        deferred,
    )) = textarea.tokenizer_run().completion()
    else {
        panic!("textarea keeps its context-dependent tokenizer boundary");
    };
    assert_eq!(
        deferred.capability(),
        HtmlTokenizerCapability::ContextDependentTokenizerMode {
            mode: HtmlTokenizerMode::Rcdata
        },
        "textarea never reaches a selected TC-S10 RCDATA refusal"
    );
    assert!(
        textarea
            .nodes_in_creation_order()
            .iter()
            .all(|node| !matches!(
                node.kind(),
                HtmlTreeNodeKind::Element(HtmlElement::Title(_))
            ))
    );
}

#[test]
fn pf12_direct_rcdata_activation_rejects_a_suspended_textarea() {
    // The tree's `EnterRcdataForTitle` boundary is only half of the dual
    // boundary. Driving the private tokenizer session directly — the exact
    // way a future coordinator mistake would — must still be refused, because
    // `textarea` shares the durable `Rcdata` mode vocabulary with `title` and
    // a mode match is therefore not the selected boundary.
    let source = SourceText::new(SourceId::new(1), "<textarea>&amp;x</textarea>".to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&source, limits());
    assert_eq!(
        tokenizer.drive_to_boundary(),
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata),
        "textarea really does suspend under the same durable mode"
    );
    assert_eq!(tokenizer.tokens().len(), 1);

    assert!(
        tokenizer.apply_rcdata().is_err(),
        "the tokenizer refuses RCDATA activation over a suspended textarea"
    );
    // The refusal changed nothing: still suspended, still one token, and no
    // later source was produced under any lexical mode.
    assert_eq!(
        tokenizer.drive_to_boundary(),
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata)
    );
    assert_eq!(tokenizer.tokens().len(), 1);

    // The same operation over a suspended `<title>` is accepted, so the
    // refusal above is element-specific and not a disabled control.
    let title = SourceText::new(SourceId::new(1), "<title>&amp;x</title>".to_owned());
    let mut accepted = HtmlTokenizerSession::new(&title, limits());
    assert_eq!(
        accepted.drive_to_boundary(),
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata)
    );
    accepted.apply_rcdata().expect("title activates");
}

#[test]
fn pf12_title_tags_outside_the_selected_lifecycle_are_refused() {
    for (source, capability) in [
        (
            "<body><title>x</title>",
            HtmlTreeCapability::TitleTagOutsideSelectedLifecycle,
        ),
        (
            "<title id=a>x</title>",
            HtmlTreeCapability::TitleTagAttribute,
        ),
        ("<title/>", HtmlTreeCapability::SelfClosingTitleTag),
    ] {
        let analysis = analyze(source);
        assert_eq!(
            tree_unsupported(&analysis),
            Some(capability),
            "source {source:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// PF13 — Title EOF may synthesize a second EOF or a closing token
// ---------------------------------------------------------------------------

#[test]
fn pf13_title_eof_recovery_reprocesses_the_same_retained_eof_token() {
    let analysis = analyze("<title>x");

    // Exactly one EOF token exists in the whole run.
    let eof_tokens: Vec<_> = tokens(&analysis)
        .into_iter()
        .filter(|(kind, ..)| *kind == "eof")
        .collect();
    assert_eq!(eof_tokens, vec![("eof", 8, 8, String::new())]);

    // No authored close was fabricated: no end tag token exists at all.
    assert!(tokens(&analysis).iter().all(|(kind, ..)| *kind != "end"));
    assert!(analysis.actions().iter().all(|action| !matches!(
        action.kind(),
        HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { .. }
    )));

    // The pop and the redispatch share the one retained EOF token index.
    let pop_index = analysis
        .actions()
        .iter()
        .position(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::PoppedTitleElementAtEndOfFile { .. }
            )
        })
        .expect("Title EOF pop");
    let pop = &analysis.actions()[pop_index];
    let next = &analysis.actions()[pop_index + 1];
    assert!(matches!(next.kind(), HtmlTreeActionKind::ReprocessedToken));
    assert_eq!(next.trigger().token_index(), pop.trigger().token_index());
    assert!(pop.trigger().authored_boundary().is_none());

    // The recovery is diagnosed exactly once, under the Title meaning.
    let recoveries: Vec<_> = analysis
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code() == HtmlTreeDiagnosticCode::TitleEndOfFileInText)
        .map(|diagnostic| (diagnostic.recovery(), diagnostic.trigger().token_index()))
        .collect();
    assert_eq!(
        recoveries,
        vec![(
            HtmlTreeRecovery::PoppedTitleAtEndOfFileAndRestoredInHead,
            pop.trigger().token_index(),
        )]
    );

    // InHead was restored: the implied Body follows the recovered Title.
    assert!(analysis.is_complete());
    assert_eq!(title_text(&analysis).0, "x");
    assert!(
        analysis
            .nodes_in_creation_order()
            .iter()
            .any(|node| element_name(&analysis, node.id())
                == Some(HtmlElementName::Shell(
                    super::result::HtmlShellElementName::Body
                )))
    );
}

// ---------------------------------------------------------------------------
// PF14 — final coalesced text proves provenance
// ---------------------------------------------------------------------------

#[test]
fn pf14_one_coalesced_text_node_still_carries_ordered_authored_contributions() {
    let analysis = analyze("<title>a&amp;b</title>");
    let (interpreted, contributions) = title_text(&analysis);
    assert_eq!(interpreted, "a&b");
    assert_eq!(
        contributions,
        vec![
            (7, 8, "a".to_owned()),
            (8, 13, "&".to_owned()),
            (13, 14, "b".to_owned()),
        ]
    );
    // The reference's exact authored source is `&amp;`, five bytes wide, for
    // one interpreted byte: the coalesced string alone could not show that.
    let source = "<title>a&amp;b</title>";
    assert_eq!(&source[8..13], "&amp;");
}

// ---------------------------------------------------------------------------
// PF15 — TC-S10 permits broad cleanup or refactor
// ---------------------------------------------------------------------------

#[test]
fn pf15_predecessor_capabilities_and_vocabulary_are_unchanged() {
    // `HtmlCharacterReferenceContext` keeps exactly its two frozen meanings:
    // TC-S10 encodes coordinated Title semantics with its own narrow
    // capability instead of widening that predecessor vocabulary.
    let source = SourceText::new(SourceId::new(1), "&x".to_owned());
    let run = tokenize(&source, limits());
    let HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
        unsupported,
    )) = run.completion()
    else {
        panic!("the Data character-reference boundary is unchanged");
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTokenizerCapability::CharacterReference {
            context: super::super::tokenizer::result::HtmlCharacterReferenceContext::Data
        }
    );

    // TC-S9's selected Style lifecycle is unchanged end to end.
    let style = analyze("<style>a&b</style>");
    assert!(style.is_complete());
    assert!(style.tokenizer_run().diagnostics().is_empty());
}

// ---------------------------------------------------------------------------
// Semantic matrix — Title / RCDATA lifecycle
// ---------------------------------------------------------------------------

#[test]
fn empty_title_constructs_a_childless_title_under_head() {
    let analysis = analyze("<title></title>");
    let title = title_id(&analysis);
    assert!(title_text_nodes(&analysis, title).is_empty());
    assert_eq!(
        tokens(&analysis),
        vec![
            ("start", 0, 7, "title".to_owned()),
            ("end", 7, 15, "title".to_owned()),
            ("eof", 15, 15, String::new()),
        ]
    );
    assert!(analysis.is_complete());
    let parent = analysis.node(title).expect("Title").parent().expect("head");
    assert_eq!(
        element_name(&analysis, parent),
        Some(HtmlElementName::Shell(
            super::result::HtmlShellElementName::Head
        ))
    );
}

#[test]
fn tag_shaped_rcdata_stays_interpreted_text() {
    let analysis = analyze("<title><b>x</title>");
    let (interpreted, contributions) = title_text(&analysis);
    assert_eq!(interpreted, "<b>x");
    assert_eq!(contributions, vec![(7, 11, "<b>x".to_owned())]);
    assert!(analysis.is_complete());
}

#[test]
fn a_non_appropriate_closing_tag_stays_interpreted_text() {
    let analysis = analyze("<title>x</titler>y</title>");
    let (interpreted, contributions) = title_text(&analysis);
    assert_eq!(interpreted, "x</titler>y");
    assert_eq!(contributions, vec![(7, 18, "x</titler>y".to_owned())]);
    // The authored close is the trailing one, and its exact raw spelling and
    // range remain available.
    let close = &analysis.tokenizer_run().tokens()[close_token(&analysis)];
    let HtmlToken::Tag(tag) = close else {
        panic!("the Title close is a tag token");
    };
    assert_eq!(tag.complete().fragment(), "</title>");
    assert_eq!(
        (tag.complete().range().start(), tag.complete().range().end()),
        (18, 26)
    );
}

#[test]
fn a_mixed_case_appropriate_close_succeeds_and_keeps_its_raw_spelling() {
    let analysis = analyze("<title>x</TiTlE>");
    assert!(analysis.is_complete());
    let close = &analysis.tokenizer_run().tokens()[close_token(&analysis)];
    let HtmlToken::Tag(tag) = close else {
        panic!("the Title close is a tag token");
    };
    // Interpreted name is ASCII-lowercased; the authored spelling is exact.
    assert_eq!(tag.name().interpreted(), "title");
    assert_eq!(tag.name().source().fragment(), "TiTlE");
    assert_eq!(tag.complete().fragment(), "</TiTlE>");
}

#[test]
fn the_tokenizer_returns_to_data_before_post_title_source() {
    let analysis = analyze("<head><title>x</title><body>");
    assert_eq!(
        tokens(&analysis),
        vec![
            ("start", 0, 6, "head".to_owned()),
            ("start", 6, 13, "title".to_owned()),
            ("chars", 13, 14, "x".to_owned()),
            ("end", 14, 22, "title".to_owned()),
            // Produced under Data: a real Body start tag, not RCDATA text.
            ("start", 22, 28, "body".to_owned()),
            ("eof", 28, 28, String::new()),
        ]
    );
    assert!(analysis.is_complete());
}

#[test]
fn repeated_title_episodes_each_run_their_own_lifecycle() {
    let analysis = analyze("<title>a</title><title>b</title>");
    let titles = title_ids(&analysis);
    assert_eq!(titles.len(), 2);
    assert_eq!(
        title_text_nodes(&analysis, titles[0])
            .into_iter()
            .map(|(interpreted, _)| interpreted)
            .collect::<Vec<_>>(),
        vec!["a".to_owned()]
    );
    assert_eq!(
        title_text_nodes(&analysis, titles[1])
            .into_iter()
            .map(|(interpreted, _)| interpreted)
            .collect::<Vec<_>>(),
        vec!["b".to_owned()]
    );
    let inserts: Vec<_> = analysis
        .actions()
        .iter()
        .filter(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::InsertedAuthoredTitleElement { .. }
            )
        })
        .map(|action| action.trigger().token_index())
        .collect();
    let closes: Vec<_> = analysis
        .actions()
        .iter()
        .filter(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { .. }
            )
        })
        .map(|action| action.trigger().token_index())
        .collect();
    assert_eq!(inserts, vec![0, 3]);
    assert_eq!(closes, vec![2, 5]);
    assert!(analysis.is_complete());
}

#[test]
fn an_isolated_ampersand_stays_literal_text() {
    let analysis = analyze("<title>&</title>");
    let (interpreted, contributions) = title_text(&analysis);
    assert_eq!(interpreted, "&");
    assert_eq!(contributions, vec![(7, 8, "&".to_owned())]);
    assert!(tokenizer_diagnostics(&analysis).is_empty());
}

#[test]
fn transition_steps_are_not_reinterpreted_as_consumed_bytes() {
    // Both runs have exactly the same dispatch sequence: `<title>`, one `&`,
    // one Character Reference dispatch, one Named Character Reference
    // dispatch (which internally consumes the whole selected match), then
    // `</title>` and EOF. Only the matched identifier's byte length differs —
    // six bytes for `notin;` against four for `amp;`. If internal
    // consumption became one transition per consumed byte, the two counts
    // would differ by two.
    let long = analyze("<title>&notin;</title>");
    let short = analyze("<title>&amp;</title>");
    assert_eq!(
        long.tokenizer_run().usage().transition_steps(),
        short.tokenizer_run().usage().transition_steps(),
        "matched identifier bytes must not each become a transition"
    );
    // The Ambiguous Ampersand path does dispatch per unit, as the accepted
    // step definition requires.
    let ambiguous = analyze("<title>&bogus;</title>");
    assert!(
        ambiguous.tokenizer_run().usage().transition_steps()
            > short.tokenizer_run().usage().transition_steps()
    );
}

// ---------------------------------------------------------------------------
// Resource matrix
// ---------------------------------------------------------------------------

#[test]
fn a_named_commit_refuses_retained_interpreted_bytes_without_partial_effect() {
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 4_096, 256, 5, 4_096);
    let analysis = analyze_with("<title>&amp;</title>", 1, constrained);
    let limit = tokenizer_resource_limit(&analysis);
    assert_eq!(
        limit.resource(),
        HtmlTokenizerResource::RetainedInterpretedBytes
    );
    assert_eq!(limit.limit(), 5);
    assert_eq!(limit.attempted(), 6);
    assert_eq!(analysis.tokenizer_run().tokens().len(), 1);
    assert!(analysis.tokenizer_run().diagnostics().is_empty());
    assert_eq!(analysis.tokenizer_run().coverage().processed_end(), 8);
    assert!(title_text_nodes(&analysis, title_id(&analysis)).is_empty());
}

#[test]
fn a_named_commit_refuses_emitted_tokens_without_partial_effect() {
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 1, 4_096, 256, 16_384, 4_096);
    let analysis = analyze_with("<title>&amp;</title>", 1, constrained);
    let limit = tokenizer_resource_limit(&analysis);
    assert_eq!(limit.resource(), HtmlTokenizerResource::EmittedTokens);
    assert_eq!(limit.limit(), 1);
    assert_eq!(limit.attempted(), 2);
    assert_eq!(analysis.tokenizer_run().tokens().len(), 1);
    assert!(analysis.tokenizer_run().diagnostics().is_empty());
    assert_eq!(analysis.tokenizer_run().coverage().processed_end(), 8);
}

#[test]
fn the_first_named_commit_refusal_is_deterministic_in_the_accepted_order() {
    // All three capacities are exhausted at once. The accepted preflight
    // order is RetainedInterpretedBytes -> EmittedTokens -> Diagnostics, so
    // the first one always wins, deterministically across repeats.
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 1, 0, 256, 5, 4_096);
    for source_id in [1u64, 2u64, 3u64] {
        let analysis = analyze_with("<title>&notit;</title>", source_id, constrained);
        assert_eq!(
            tokenizer_resource_limit(&analysis).resource(),
            HtmlTokenizerResource::RetainedInterpretedBytes
        );
    }
    // With only retained bytes available, the next boundary is EmittedTokens.
    let tokens_only = HtmlTokenizerLimits::new(4_096, 32_768, 1, 0, 256, 16_384, 4_096);
    assert_eq!(
        tokenizer_resource_limit(&analyze_with("<title>&notit;</title>", 1, tokens_only))
            .resource(),
        HtmlTokenizerResource::EmittedTokens
    );
    // With both available, Diagnostics is the last one to refuse.
    let diagnostics_only = HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 0, 256, 16_384, 4_096);
    assert_eq!(
        tokenizer_resource_limit(&analyze_with("<title>&notit;</title>", 1, diagnostics_only))
            .resource(),
        HtmlTokenizerResource::Diagnostics
    );
}

#[test]
fn prior_valid_evidence_survives_a_named_refusal() {
    // Text committed before the reference stays exactly as it was.
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 2, 4_096, 256, 16_384, 4_096);
    let analysis = analyze_with("<title>ab&amp;</title>", 1, constrained);
    assert_eq!(
        tokenizer_resource_limit(&analysis).resource(),
        HtmlTokenizerResource::EmittedTokens
    );
    assert_eq!(title_text(&analysis).0, "ab");
    assert_eq!(analysis.tokenizer_run().coverage().processed_end(), 10);
}

#[test]
fn an_unknown_name_diagnostic_survives_a_refused_unresolved_run_emission() {
    // `<title>&bogus;</title>`: the authored `;` observation is complete on
    // its own, so the `UnknownNamedCharacterReference` diagnostic commits
    // before the unresolved `&bogus` run is flushed. With room for only the
    // Title start tag, that flush is refused — and the already-valid
    // observation-conditioned diagnostic must still be retained.
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 1, 4_096, 256, 16_384, 4_096);
    let analysis = analyze_with("<title>&bogus;</title>", 1, constrained);

    let limit = tokenizer_resource_limit(&analysis);
    assert_eq!(limit.resource(), HtmlTokenizerResource::EmittedTokens);
    assert_eq!(limit.limit(), 1);
    assert_eq!(limit.attempted(), 2);

    // The diagnostic survived the later refusal, at its exact authored `;`.
    assert_eq!(
        tokenizer_diagnostics(&analysis),
        vec![(
            HtmlTokenizerDiagnosticCode::UnknownNamedCharacterReference,
            HtmlTokenizerDiagnosticContext::AmbiguousAmpersand,
            HtmlTokenizerDiagnosticHandling::Continued,
            Subject::InputLocation,
            13,
            14,
        )]
    );
    // Only the Title start tag committed: the unresolved run really was
    // refused, so this is not the ordinary success path in disguise.
    assert_eq!(analysis.tokenizer_run().tokens().len(), 1);
    assert!(title_text_nodes(&analysis, title_id(&analysis)).is_empty());
    // Coverage still includes the observed `;` that caused the diagnostic.
    assert_eq!(analysis.tokenizer_run().coverage().processed_end(), 14);
    assert!(!analysis.is_complete());
}

#[test]
fn the_selected_matcher_retains_no_temporary_buffer_bytes() {
    for source in [
        "<title>&amp;</title>",
        "<title>&notit;</title>",
        "<title>&notin;</title>",
        "<title>&acE;</title>",
        "<title>&bogus;</title>",
        "<title>&CounterClockwiseContourIntegral;</title>",
    ] {
        let analysis = analyze(source);
        assert_eq!(
            analysis
                .tokenizer_run()
                .usage()
                .peak_temporary_buffer_bytes(),
            0,
            "source {source:?}"
        );
    }
    // A zero temporary-buffer budget therefore never refuses the matcher.
    let zero_buffer = HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 4_096, 256, 16_384, 0);
    let analysis = analyze_with("<title>&notit;</title>", 1, zero_buffer);
    assert!(analysis.is_complete());
    assert_eq!(title_text(&analysis).0, "\u{ac}it;");
}

// ---------------------------------------------------------------------------
// Freeze / durable-result corruption matrix
// ---------------------------------------------------------------------------

struct PartsFixture {
    source: SourceText,
    run: HtmlTokenizerRunResult,
    parts: HtmlDocumentShellParts,
}

/// Drives the same production seams the coordinator uses, but keeps the parts
/// so a test can corrupt exactly one durable fact before freezing.
fn coordinated_parts(source_text: &str) -> PartsFixture {
    let source = SourceText::new(SourceId::new(410), source_text.to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&source, limits());
    let mut session = HtmlTreeSession::new().expect("session");
    let mut next_token = 0usize;
    let mut raw_text_entries = Vec::new();
    let mut raw_text_closes = Vec::new();
    let mut rcdata_entries = Vec::new();
    let mut rcdata_closes = Vec::new();
    let mut live: Option<HtmlTreeTokenizerFeedback> = None;
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
                assert!(
                    !evaluated.contains(&mode),
                    "same-token insertion-mode cycle"
                );
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
                        assert_eq!(next_token + 1, produced);
                        match feedback {
                            HtmlTreeTokenizerFeedback::EnterRawText => {
                                assert_eq!(
                                    boundary,
                                    HtmlTokenizerSessionBoundary::Suspended(
                                        HtmlTokenizerMode::RawText
                                    )
                                );
                                tokenizer.apply_raw_text().expect("apply RAWTEXT");
                                raw_text_entries.push(next_token);
                            }
                            HtmlTreeTokenizerFeedback::EnterRcdataForTitle => {
                                assert_eq!(
                                    boundary,
                                    HtmlTokenizerSessionBoundary::Suspended(
                                        HtmlTokenizerMode::Rcdata
                                    )
                                );
                                tokenizer.apply_rcdata().expect("apply RCDATA");
                                rcdata_entries.push(next_token);
                            }
                        }
                        live = Some(feedback);
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
                let close = produced.checked_sub(1).expect("close token");
                match live.take() {
                    Some(HtmlTreeTokenizerFeedback::EnterRawText) => raw_text_closes.push(close),
                    Some(HtmlTreeTokenizerFeedback::EnterRcdataForTitle) => {
                        rcdata_closes.push(close)
                    }
                    None => panic!("close without a live episode"),
                }
            }
            HtmlTokenizerSessionBoundary::Suspended(mode) => {
                panic!("fixture suspension without tree feedback: {mode:?}")
            }
            HtmlTokenizerSessionBoundary::Terminal => {
                break 'produce tokenizer.into_result().expect("terminal result");
            }
        }
    };

    let completion = if matches!(run.completion(), HtmlTokenizerCompletion::Complete) && stopped {
        HtmlTreeCompletion::Complete
    } else {
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    };
    let mut parts = session.finish(completion);
    parts.coordinated_raw_text_entry_tokens = raw_text_entries;
    parts.coordinated_raw_text_close_tokens = raw_text_closes;
    parts.coordinated_rcdata_entry_tokens = rcdata_entries;
    parts.coordinated_rcdata_close_tokens = rcdata_closes;
    PartsFixture { source, run, parts }
}

fn freeze_fixture(
    fixture: &PartsFixture,
    parts: HtmlDocumentShellParts,
) -> Result<HtmlDocumentShellAnalysis, HtmlTreeFreezeError> {
    freeze(&fixture.source, fixture.run.clone(), parts)
}

fn title_node_id(parts: &HtmlDocumentShellParts) -> HtmlConstructedNodeId {
    parts
        .actions
        .iter()
        .find_map(|action| match action.kind() {
            HtmlTreeActionKind::InsertedAuthoredTitleElement { node } => Some(*node),
            _ => None,
        })
        .expect("fixture Title")
}

fn last_action_index(
    parts: &HtmlDocumentShellParts,
    predicate: impl Fn(&HtmlTreeActionKind) -> bool,
) -> usize {
    parts
        .actions
        .iter()
        .rposition(|action| predicate(action.kind()))
        .expect("fixture action")
}

fn action_index(
    parts: &HtmlDocumentShellParts,
    predicate: impl Fn(&HtmlTreeActionKind) -> bool,
) -> usize {
    parts
        .actions
        .iter()
        .position(|action| predicate(action.kind()))
        .expect("fixture action")
}

const CLOSED: &str = "<title>x</title>";
const OPEN_AT_EOF: &str = "<title>x";
/// A run whose tokenizer refuses the Numeric branch while the Title is still
/// open, so the freeze layer sees a genuinely open Text-mode element.
const OPEN_AT_REFUSAL: &str = "<title>&#60;</title>";

#[test]
fn freeze_rejects_pending_feedback_and_missing_coordination() {
    let fixture = coordinated_parts(CLOSED);
    assert!(freeze_fixture(&fixture, coordinated_parts(CLOSED).parts).is_ok());

    // (1) pending Title tokenizer feedback at freeze
    let mut pending = coordinated_parts(CLOSED).parts;
    pending.pending_tokenizer_feedback = true;
    assert!(matches!(
        freeze_fixture(&fixture, pending),
        Err(HtmlTreeFreezeError::OutstandingTokenizerFeedback)
    ));

    // (2) feedback acknowledged without a recorded tokenizer RCDATA entry
    let mut no_entry = coordinated_parts(CLOSED).parts;
    no_entry.coordinated_rcdata_entry_tokens.clear();
    assert!(matches!(
        freeze_fixture(&fixture, no_entry),
        Err(HtmlTreeFreezeError::TitleCoordinationEntryMismatch { .. })
    ));

    // (3) a claimed authored close with no coordinated close boundary
    let mut no_close = coordinated_parts(CLOSED).parts;
    no_close.coordinated_rcdata_close_tokens.clear();
    assert!(matches!(
        freeze_fixture(&fixture, no_close),
        Err(HtmlTreeFreezeError::TitleCoordinationCloseMismatch { .. })
    ));

    // (17) a terminal result with unresolved coordinator feedback
    let mut terminal = coordinated_parts(CLOSED).parts;
    terminal.pending_tokenizer_feedback = true;
    terminal.completion = HtmlTreeCompletion::Complete;
    assert!(matches!(
        freeze_fixture(&fixture, terminal),
        Err(HtmlTreeFreezeError::OutstandingTokenizerFeedback)
    ));
}

#[test]
fn freeze_rejects_impossible_text_mode_final_state() {
    let fixture = coordinated_parts(CLOSED);

    // (4) tree Text mode active in a claimed terminal Title result
    let mut text_mode = coordinated_parts(CLOSED).parts;
    text_mode.final_text_mode_active = true;
    assert!(matches!(
        freeze_fixture(&fixture, text_mode),
        Err(HtmlTreeFreezeError::FinalTextModeStateMismatch)
    ));

    // (5) Text mode without the retained original insertion mode. The Title
    // really is open here, so only the retained-original fact is corrupted.
    let refused = coordinated_parts(OPEN_AT_REFUSAL);
    assert!(freeze_fixture(&refused, coordinated_parts(OPEN_AT_REFUSAL).parts).is_ok());
    let mut missing_original = coordinated_parts(OPEN_AT_REFUSAL).parts;
    assert!(missing_original.final_open_title.is_some());
    assert!(missing_original.final_text_mode_active);
    missing_original.final_original_insertion_mode_retained = false;
    assert!(matches!(
        freeze_fixture(&refused, missing_original),
        Err(HtmlTreeFreezeError::FinalTextModeStateMismatch)
    ));

    // ...and the same for a genuinely open Title claiming Text mode is off.
    let mut inactive = coordinated_parts(OPEN_AT_REFUSAL).parts;
    inactive.final_text_mode_active = false;
    assert!(matches!(
        freeze_fixture(&refused, inactive),
        Err(HtmlTreeFreezeError::FinalTextModeStateMismatch)
    ));

    let open = coordinated_parts(OPEN_AT_EOF);

    // (6) Title marked open after its authored close
    let mut still_open = coordinated_parts(CLOSED).parts;
    still_open.final_open_title = Some(title_node_id(&still_open));
    still_open.final_text_mode_active = true;
    still_open.final_original_insertion_mode_retained = true;
    assert!(matches!(
        freeze_fixture(&fixture, still_open),
        Err(HtmlTreeFreezeError::FinalTitleStateMismatch)
    ));

    // (7) Title marked closed with no authored close and no EOF recovery
    let mut closed_without_cause = coordinated_parts(OPEN_AT_EOF).parts;
    let pop = action_index(&closed_without_cause, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::PoppedTitleElementAtEndOfFile { .. }
        )
    });
    closed_without_cause.actions.remove(pop);
    assert!(freeze_fixture(&open, closed_without_cause).is_err());

    // (18) TC-S9's Style Text-mode state still validates through the same
    // shared final fields.
    let style = coordinated_parts("<style>x</style>");
    assert!(freeze_fixture(&style, coordinated_parts("<style>x</style>").parts).is_ok());
    let mut style_broken = coordinated_parts("<style>x</style>").parts;
    style_broken.final_text_mode_active = true;
    assert!(matches!(
        freeze_fixture(&style, style_broken),
        Err(HtmlTreeFreezeError::FinalTextModeStateMismatch)
    ));

    // Style and Title can never both be the open Text-mode element. Claiming
    // the open Title in the Style slot is refused by the Style domain's own
    // identity check first; the shared invariant additionally rules the
    // combination out for any pair that got past both domain replays.
    let mut concurrent = coordinated_parts(OPEN_AT_REFUSAL).parts;
    concurrent.final_open_style = concurrent.final_open_title;
    assert!(matches!(
        freeze_fixture(&refused, concurrent),
        Err(HtmlTreeFreezeError::FinalOpenStyleIsNotStyle(_))
    ));
}

#[test]
fn freeze_rejects_wrong_title_origin_and_close_relationships() {
    let fixture = coordinated_parts(CLOSED);

    // (10) wrong Title start origin
    let mut wrong_start = coordinated_parts(CLOSED).parts;
    let title = title_node_id(&wrong_start);
    let insert = action_index(&wrong_start, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::InsertedAuthoredTitleElement { .. }
        )
    });
    let close_trigger = wrong_start.actions[action_index(&wrong_start, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { .. }
        )
    })]
    .trigger()
    .clone();
    wrong_start.actions[insert] = HtmlTreeAction::new(
        HtmlTreeActionKind::InsertedAuthoredTitleElement { node: title },
        close_trigger,
    );
    // Rejected as an impossible durable relationship: the Title's authored
    // origin cannot be a token that follows its own text and close.
    assert!(freeze_fixture(&fixture, wrong_start).is_err());

    // (11) wrong appropriate closing token/source relationship
    let mut wrong_close = coordinated_parts(CLOSED).parts;
    let title = title_node_id(&wrong_close);
    let text_trigger = wrong_close.actions[last_action_index(&wrong_close, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::InsertedTextNode { .. }
                | HtmlTreeActionKind::AppendedToTextNode { .. }
        )
    })]
    .trigger()
    .clone();
    let close = action_index(&wrong_close, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { .. }
        )
    });
    wrong_close.actions[close] = HtmlTreeAction::new(
        HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { node: title },
        text_trigger,
    );
    assert!(matches!(
        freeze_fixture(&fixture, wrong_close),
        Err(HtmlTreeFreezeError::TitleAuthoredCloseTriggerMismatch { .. })
    ));

    // A Title action whose subject is not a Title node at all.
    let mut wrong_subject = coordinated_parts(CLOSED).parts;
    let root = wrong_subject.root;
    let insert = action_index(&wrong_subject, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::InsertedAuthoredTitleElement { .. }
        )
    });
    let trigger = wrong_subject.actions[insert].trigger().clone();
    wrong_subject.actions[insert] = HtmlTreeAction::new(
        HtmlTreeActionKind::InsertedAuthoredTitleElement { node: root },
        trigger,
    );
    assert!(matches!(
        freeze_fixture(&fixture, wrong_subject),
        Err(HtmlTreeFreezeError::TitleActionSubjectIsNotTitle(_))
    ));

    // (16) a complete tree result over an incomplete tokenizer result.
    let numeric = coordinated_parts("<title>a</title>");
    let mut upgraded = coordinated_parts("<title>a</title>").parts;
    upgraded.processed_tokens = 0;
    assert!(freeze_fixture(&numeric, upgraded).is_err());
}

#[test]
fn freeze_rejects_broken_title_eof_recovery_evidence() {
    let open = coordinated_parts(OPEN_AT_EOF);
    assert!(freeze_fixture(&open, coordinated_parts(OPEN_AT_EOF).parts).is_ok());

    // (8) a fabricated authored closing anchor at EOF: the EOF pop replaced
    // by an authored-close action.
    let mut fabricated = coordinated_parts(OPEN_AT_EOF).parts;
    let title = title_node_id(&fabricated);
    let pop = action_index(&fabricated, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::PoppedTitleElementAtEndOfFile { .. }
        )
    });
    let trigger = fabricated.actions[pop].trigger().clone();
    fabricated.actions[pop] = HtmlTreeAction::new(
        HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { node: title },
        trigger,
    );
    assert!(matches!(
        freeze_fixture(&open, fabricated),
        Err(HtmlTreeFreezeError::TitleAuthoredCloseTriggerMismatch { .. })
    ));

    // (9) conflicting authored-close and EOF-close causes for one Title.
    let mut conflicting = coordinated_parts(OPEN_AT_EOF).parts;
    let title = title_node_id(&conflicting);
    let pop = action_index(&conflicting, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::PoppedTitleElementAtEndOfFile { .. }
        )
    });
    let trigger = conflicting.actions[pop].trigger().clone();
    conflicting.actions.insert(
        pop + 1,
        HtmlTreeAction::new(
            HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { node: title },
            trigger,
        ),
    );
    assert!(freeze_fixture(&open, conflicting).is_err());

    // The EOF recovery diagnostic is required, and must be the Title one.
    let mut no_diagnostic = coordinated_parts(OPEN_AT_EOF).parts;
    no_diagnostic
        .diagnostics
        .retain(|diagnostic| diagnostic.code() != HtmlTreeDiagnosticCode::TitleEndOfFileInText);
    assert!(matches!(
        freeze_fixture(&open, no_diagnostic),
        Err(HtmlTreeFreezeError::TitleEndOfFileDiagnosticMismatch { .. })
    ));

    // The same retained EOF token must be redispatched right after the pop.
    let mut no_redispatch = coordinated_parts(OPEN_AT_EOF).parts;
    let pop = action_index(&no_redispatch, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::PoppedTitleElementAtEndOfFile { .. }
        )
    });
    no_redispatch.actions.remove(pop + 1);
    assert!(matches!(
        freeze_fixture(&open, no_redispatch),
        Err(HtmlTreeFreezeError::TitleEndOfFileRedispatchMismatch { .. })
    ));

    // A Title EOF diagnostic with no matching pop is an orphan.
    let closed = coordinated_parts(CLOSED);
    let mut orphan = coordinated_parts(CLOSED).parts;
    orphan
        .diagnostics
        .push(super::result::HtmlTreeDiagnostic::new(
            HtmlTreeDiagnosticCode::TitleEndOfFileInText,
            orphan.actions[0].trigger().clone(),
            HtmlTreeRecovery::PoppedTitleAtEndOfFileAndRestoredInHead,
        ));
    assert!(matches!(
        freeze_fixture(&closed, orphan),
        Err(HtmlTreeFreezeError::OrphanTitleEndOfFileDiagnostic { .. })
    ));
}

/// Replaces the Title's text node with one whose retained contributions were
/// rewritten by `mutate`, leaving every other durable fact alone.
fn with_rewritten_title_text(
    source_text: &str,
    mutate: impl FnOnce(&mut Vec<super::result::HtmlTextContribution>, &mut String, &SourceText),
) -> HtmlDocumentShellParts {
    let source = SourceText::new(SourceId::new(410), source_text.to_owned());
    let mut parts = coordinated_parts(source_text).parts;
    let title = title_node_id(&parts);
    let index = parts
        .nodes
        .iter()
        .position(|node| {
            node.parent() == Some(title) && matches!(node.kind(), HtmlTreeNodeKind::Text(_))
        })
        .expect("Title text node");
    let node = &parts.nodes[index];
    let HtmlTreeNodeKind::Text(text) = node.kind() else {
        unreachable!("selected by kind above");
    };
    let mut contributions = text.contributions().to_vec();
    let mut interpreted = text.interpreted().to_owned();
    mutate(&mut contributions, &mut interpreted, &source);
    parts.nodes[index] = super::result::HtmlTreeNode::new(
        node.id(),
        node.parent(),
        node.children().to_vec(),
        HtmlTreeNodeKind::Text(super::result::HtmlTextNode::new(interpreted, contributions)),
    );
    parts
}

#[test]
fn freeze_rejects_corrupted_title_text_provenance() {
    let fixture = coordinated_parts("<title>a&amp;b</title>");
    assert!(freeze_fixture(&fixture, coordinated_parts("<title>a&amp;b</title>").parts).is_ok());

    // (12) contribution source order mismatch.
    let reordered = with_rewritten_title_text("<title>a&amp;b</title>", |contributions, _, _| {
        assert_eq!(contributions.len(), 3);
        contributions.reverse();
    });
    assert!(matches!(
        freeze_fixture(&fixture, reordered),
        Err(HtmlTreeFreezeError::InvalidTextContributions(_))
    ));

    // (13) overlapping/fabricated source subdivision for a two-scalar entity.
    // The one authored `&acE;` contribution must never be split into invented
    // per-scalar spans.
    let multi = coordinated_parts("<title>&acE;</title>");
    assert!(freeze_fixture(&multi, coordinated_parts("<title>&acE;</title>").parts).is_ok());
    let split = with_rewritten_title_text("<title>&acE;</title>", |contributions, _, _| {
        assert_eq!(contributions.len(), 1);
        let source = contributions[0].source().clone();
        let whole = source.clone();
        contributions.clear();
        contributions.push(super::result::HtmlTextContribution::new(
            source,
            "\u{223e}".to_owned(),
        ));
        // A second contribution over the *same* authored bytes: a fabricated
        // subdivision that overlaps the first.
        contributions.push(super::result::HtmlTextContribution::new(
            whole,
            "\u{333}".to_owned(),
        ));
    });
    assert!(matches!(
        freeze_fixture(&multi, split),
        Err(HtmlTreeFreezeError::InvalidTextContributions(_))
    ));

    // (15) the unresolved `&bogus` contribution merged with the diagnostic's
    // own semicolon origin, so no character token matches either any more.
    let bogus = coordinated_parts("<title>&bogus;</title>");
    assert!(freeze_fixture(&bogus, coordinated_parts("<title>&bogus;</title>").parts).is_ok());
    let merged = with_rewritten_title_text("<title>&bogus;</title>", |contributions, _, source| {
        assert_eq!(contributions.len(), 2);
        let start = contributions[0].source().range().start();
        let end = contributions[1].source().range().end();
        let anchor = source.anchor(start, end).expect("merged span");
        contributions.clear();
        contributions.push(super::result::HtmlTextContribution::new(
            anchor,
            "&bogus;".to_owned(),
        ));
    });
    assert!(freeze_fixture(&bogus, merged).is_err());
}

#[test]
fn freeze_rejects_a_decoded_close_presented_as_the_authored_close() {
    // (14) `<title>&lt;/title></title>` — the decoded `</title>` is a
    // character token. Retargeting the close action onto it must not freeze.
    let source = "<title>&lt;/title></title>";
    let fixture = coordinated_parts(source);
    assert!(freeze_fixture(&fixture, coordinated_parts(source).parts).is_ok());

    let mut decoded_close = coordinated_parts(source).parts;
    let title = title_node_id(&decoded_close);
    let text_trigger = decoded_close.actions[last_action_index(&decoded_close, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::InsertedTextNode { .. }
                | HtmlTreeActionKind::AppendedToTextNode { .. }
        )
    })]
    .trigger()
    .clone();
    let close = action_index(&decoded_close, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { .. }
        )
    });
    decoded_close.actions[close] = HtmlTreeAction::new(
        HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { node: title },
        text_trigger,
    );
    assert!(matches!(
        freeze_fixture(&fixture, decoded_close),
        Err(HtmlTreeFreezeError::TitleAuthoredCloseTriggerMismatch { .. })
    ));
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn the_selected_lifecycle_is_deterministic_across_repeats_and_identities() {
    for source in [
        "<title>a&amp;b</title>",
        "<title>&notit;</title>",
        "<title>&bogus;</title>",
        "<title>&acE;</title>",
        "<title>x",
        "<title>&#60;</title>",
        "<title>a\u{0}b</title>",
    ] {
        let baseline = signature(&analyze_with(source, 1, limits()));
        for source_id in [1u64, 2u64, 7u64] {
            assert_eq!(
                signature(&analyze_with(source, source_id, limits())),
                baseline,
                "source {source:?} under SourceId {source_id}"
            );
        }
    }
}

/// A structural signature that excludes the caller-supplied `SourceId` so two
/// runs over equal bytes under different identities compare directly.
fn signature(analysis: &HtmlDocumentShellAnalysis) -> String {
    let mut rendered = String::new();
    for node in analysis.nodes_in_creation_order() {
        rendered.push_str(&format!(
            "{:?}|{:?}|{:?}|",
            node.id(),
            node.parent(),
            node.children()
        ));
        match node.kind() {
            HtmlTreeNodeKind::Document => rendered.push_str("document"),
            HtmlTreeNodeKind::Element(element) => {
                rendered.push_str(&format!("element:{:?}", element.name()))
            }
            HtmlTreeNodeKind::Text(text) => rendered.push_str(&format!(
                "text:{}:{:?}",
                text.interpreted(),
                text.contributions()
                    .iter()
                    .map(|contribution| contribution.source().range())
                    .collect::<Vec<_>>()
            )),
        }
        rendered.push(';');
    }
    rendered.push('|');
    for action in analysis.actions() {
        rendered.push_str(&format!(
            "{:?}@{};",
            action.kind(),
            action.trigger().token_index()
        ));
    }
    rendered.push('|');
    for diagnostic in analysis.tokenizer_run().diagnostics() {
        rendered.push_str(&format!(
            "{:?}@{:?}:{:?};",
            diagnostic.code(),
            diagnostic.location().range(),
            diagnostic.context()
        ));
    }
    rendered.push('|');
    rendered.push_str(&format!(
        "{}:{}|{:?}",
        analysis.coverage().committed_end(),
        analysis.coverage().processed_tokens(),
        analysis.completion()
    ));
    rendered
}
