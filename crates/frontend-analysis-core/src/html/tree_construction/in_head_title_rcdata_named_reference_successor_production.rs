//! Production correspondence for TC-S10 — the selected InHead `<title>`
//! RCDATA + Named Character Reference causal lifecycle (Issue #394).
//!
//! Expectations here are authored from the accepted production theorem. The
//! tests import only production tokenizer, tree-construction, durable-result
//! and freeze seams. They do not import, call, or copy the candidate-
//! independent validation machine in the sibling
//! `in_head_title_rcdata_named_reference_successor_validation` module, and
//! they use no browser, WPT, or html5lib output as an expected value.
//!
//! The suite is written to *falsify* wrong implementations rather than to
//! narrate the right one. Each test names the incorrect implementation it
//! rejects.

use crate::{SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::producer::{
    HtmlTokenizerSession, HtmlTokenizerSessionBoundary, HtmlTokenizerSessionControlError, tokenize,
};
use super::super::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerResource};
use super::super::tokenizer::result::{
    HtmlCharacterReferenceContext, HtmlTokenizerCapability, HtmlTokenizerCapabilityAvailability,
    HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause, HtmlTokenizerMode,
    HtmlTokenizerRunResult, HtmlTokenizerUnsupportedTrigger,
};
use super::driver::construct_html_document_shell;
use super::result::{
    HtmlConstructedNodeId, HtmlDocumentShellAnalysis, HtmlDocumentShellParts, HtmlElement,
    HtmlElementName, HtmlTreeActionKind, HtmlTreeCapability, HtmlTreeCompletion,
    HtmlTreeDiagnosticCode, HtmlTreeFreezeError, HtmlTreeIncompleteCause, HtmlTreeNodeKind,
    HtmlTreeRecovery, freeze,
};
use super::session::{
    DispatchOutcome, HtmlTreeSession, HtmlTreeTokenizerFeedback, InsertionMode, admit,
    token_trigger,
};

// ---------------------------------------------------------------------------
// Harness
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
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .find_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(HtmlElement::Title(_)) => Some(node.id()),
            _ => None,
        })
        .expect("Title node")
}

/// The Title's interpreted text, as the frozen tree holds it.
fn title_text(analysis: &HtmlDocumentShellAnalysis) -> String {
    let title = title_id(analysis);
    analysis
        .node(title)
        .expect("Title node")
        .children()
        .iter()
        .map(
            |child| match analysis.node(*child).expect("Title child").kind() {
                HtmlTreeNodeKind::Text(text) => text.interpreted().to_owned(),
                other => panic!("a Title child must be text, got {other:?}"),
            },
        )
        .collect()
}

/// Every ordered `(authored range, interpreted)` contribution under Title.
fn title_contributions(analysis: &HtmlDocumentShellAnalysis) -> Vec<((usize, usize), String)> {
    let title = title_id(analysis);
    let mut contributions = Vec::new();
    for child in analysis.node(title).expect("Title node").children() {
        let HtmlTreeNodeKind::Text(text) = analysis.node(*child).expect("Title child").kind()
        else {
            panic!("a Title child must be text");
        };
        for contribution in text.contributions() {
            contributions.push((
                (
                    contribution.source().range().start(),
                    contribution.source().range().end(),
                ),
                contribution.interpreted().to_owned(),
            ));
        }
    }
    contributions
}

fn character_tokens(run: &HtmlTokenizerRunResult) -> Vec<((usize, usize), String)> {
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

fn tokenizer_diagnostics(
    run: &HtmlTokenizerRunResult,
) -> Vec<(
    super::super::tokenizer::diagnostic::HtmlTokenizerDiagnosticCode,
    (usize, usize),
)> {
    run.diagnostics()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.code(),
                (
                    diagnostic.location().range().start(),
                    diagnostic.location().range().end(),
                ),
            )
        })
        .collect()
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

/// One observed tokenizer unsupported stop: capability, availability, and the
/// authored span its `Input` trigger names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObservedUnsupported {
    capability: HtmlTokenizerCapability,
    availability: HtmlTokenizerCapabilityAvailability,
    trigger: Option<(usize, usize)>,
}

fn tokenizer_unsupported(run: &HtmlTokenizerRunResult) -> Option<ObservedUnsupported> {
    let HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
        unsupported,
    )) = run.completion()
    else {
        return None;
    };
    let trigger = match unsupported.trigger() {
        HtmlTokenizerUnsupportedTrigger::Input(anchor) => {
            Some((anchor.range().start(), anchor.range().end()))
        }
        HtmlTokenizerUnsupportedTrigger::EmittedToken { .. } => None,
    };
    Some(ObservedUnsupported {
        capability: unsupported.capability(),
        availability: unsupported.availability(),
        trigger,
    })
}

fn resource_stop(run: &HtmlTokenizerRunResult) -> Option<HtmlTokenizerResource> {
    match run.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(limit)) => {
            Some(limit.resource())
        }
        _ => None,
    }
}

/// A structural projection of everything the frozen result durably means,
/// excluding the caller-supplied `SourceId` so two runs over the same bytes
/// under different identities compare directly.
fn semantic_signature(analysis: &HtmlDocumentShellAnalysis) -> String {
    let mut rendered = String::new();
    for node in analysis.nodes_in_creation_order() {
        // Element identity and authored ranges, never the caller-supplied
        // `SourceId`: two runs over equal bytes must compare equal.
        let kind = match node.kind() {
            HtmlTreeNodeKind::Document => "#document".to_owned(),
            HtmlTreeNodeKind::Element(element) => format!("{:?}", element.name()),
            HtmlTreeNodeKind::Text(text) => format!("#text{:?}", text.interpreted()),
        };
        let authored = match node.authored_source() {
            Some(super::result::HtmlAuthoredSource::StartTag { complete, raw_name }) => {
                format!("{:?}{:?}", complete.range(), raw_name.range())
            }
            Some(super::result::HtmlAuthoredSource::Characters(contributions)) => contributions
                .iter()
                .map(|contribution| {
                    format!(
                        "{:?}={:?}",
                        contribution.source().range(),
                        contribution.interpreted()
                    )
                })
                .collect(),
            None => "-".to_owned(),
        };
        rendered.push_str(&format!("{:?}:{kind}:{authored};", node.id()));
    }
    rendered.push('|');
    for action in analysis.actions() {
        rendered.push_str(&format!(
            "{:?}@{}{:?};",
            action.kind(),
            action.trigger().token_index(),
            action.trigger().authored_boundary().map(|a| a.range())
        ));
    }
    rendered.push('|');
    for diagnostic in analysis.diagnostics() {
        rendered.push_str(&format!(
            "{:?}@{}{:?};",
            diagnostic.code(),
            diagnostic.trigger().token_index(),
            diagnostic.recovery()
        ));
    }
    rendered.push('|');
    for (range, interpreted) in character_tokens(analysis.tokenizer_run()) {
        rendered.push_str(&format!("{range:?}={interpreted:?};"));
    }
    rendered.push('|');
    rendered.push_str(&format!(
        "{:?}",
        tokenizer_diagnostics(analysis.tokenizer_run())
    ));
    rendered
}

// ---------------------------------------------------------------------------
// Coordination
// ---------------------------------------------------------------------------

/// Falsifies: a tokenizer that runs past `<title>` before tree feedback, and
/// a tree that bypasses the coordinator.
#[test]
fn pf1_title_suspends_the_tokenizer_before_any_post_title_source_is_produced() {
    let source = SourceText::new(SourceId::new(1), "<title>abc</title>".to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&source, limits());

    let boundary = tokenizer.drive_to_boundary();
    assert_eq!(
        boundary,
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata)
    );
    // Exactly one token exists at the suspension: the `<title>` start tag. No
    // RCDATA text has been produced, so feedback genuinely precedes
    // post-`<title>` production rather than merely being recorded after it.
    assert_eq!(tokenizer.tokens().len(), 1);
    assert!(matches!(tokenizer.tokens()[0], HtmlToken::Tag(_)));

    // Driving again without applying feedback produces nothing new: the
    // tokenizer will not invent the mode the tree owns.
    assert_eq!(
        tokenizer.drive_to_boundary(),
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata)
    );
    assert_eq!(tokenizer.tokens().len(), 1);

    tokenizer.apply_title_rcdata().expect("selected activation");
    tokenizer.drive_to_boundary();
    assert!(tokenizer.tokens().len() > 1);
}

/// Falsifies: generic RCDATA activation, and `<textarea>` becoming eligible.
///
/// The tokenizer-side control is checked directly, without the tree, because
/// the tree gate alone would not prove the tokenizer refuses independently.
#[test]
fn pf2_direct_rcdata_activation_refuses_a_suspended_textarea() {
    let source = SourceText::new(SourceId::new(1), "<textarea>abc</textarea>".to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&source, limits());
    assert_eq!(
        tokenizer.drive_to_boundary(),
        // `textarea` carries the identical durable mode, so mode
        // classification alone is not the selected boundary.
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata)
    );
    assert_eq!(
        tokenizer.apply_title_rcdata(),
        Err(HtmlTokenizerSessionControlError::TitleRcdataActivationInvariant)
    );
    // The refusal left the suspension intact rather than half-activating.
    assert_eq!(
        tokenizer.drive_to_boundary(),
        HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata)
    );
    assert_eq!(tokenizer.tokens().len(), 1);
}

/// Falsifies: RCDATA conflated with RAWTEXT, in both directions.
#[test]
fn pf3_the_two_selected_text_mode_controls_are_not_interchangeable() {
    let title = SourceText::new(SourceId::new(1), "<title>abc</title>".to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&title, limits());
    tokenizer.drive_to_boundary();
    assert_eq!(
        tokenizer.apply_raw_text(),
        Err(
            HtmlTokenizerSessionControlError::RawTextRequestedForDifferentMode(
                HtmlTokenizerMode::Rcdata
            )
        )
    );

    let style = SourceText::new(SourceId::new(1), "<style>abc</style>".to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&style, limits());
    tokenizer.drive_to_boundary();
    assert_eq!(
        tokenizer.apply_title_rcdata(),
        Err(
            HtmlTokenizerSessionControlError::TitleRcdataRequestedForDifferentMode(
                HtmlTokenizerMode::RawText
            )
        )
    );

    // RCDATA resolves references; RAWTEXT does not. If the two modes were
    // conflated, one of these two texts would be wrong.
    assert_eq!(title_text(&analyze("<title>&amp;</title>")), "&");
    let style = analyze("<style>&amp;</style>");
    let style_id = style
        .nodes_in_creation_order()
        .into_iter()
        .find_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(HtmlElement::Style(_)) => Some(node.id()),
            _ => None,
        })
        .expect("Style node");
    let style_text: String = style
        .node(style_id)
        .expect("Style node")
        .children()
        .iter()
        .map(|child| match style.node(*child).expect("child").kind() {
            HtmlTreeNodeKind::Text(text) => text.interpreted().to_owned(),
            other => panic!("Style child must be text, got {other:?}"),
        })
        .collect();
    assert_eq!(style_text, "&amp;");
}

/// Falsifies: activation without a matching tokenizer suspension.
#[test]
fn pf4_activation_requires_the_exact_post_start_tag_suspension() {
    let source = SourceText::new(SourceId::new(1), "<title>abc</title>".to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&source, limits());
    assert_eq!(
        tokenizer.apply_title_rcdata(),
        Err(HtmlTokenizerSessionControlError::TitleRcdataRequestedWithoutSuspension)
    );

    tokenizer.drive_to_boundary();
    tokenizer.apply_title_rcdata().expect("selected activation");
    // A second activation has no suspension left to consume.
    assert_eq!(
        tokenizer.apply_title_rcdata(),
        Err(HtmlTokenizerSessionControlError::TitleRcdataRequestedWithoutSuspension)
    );
}

/// Falsifies: `<textarea>` becoming supported through the coordinated path.
#[test]
fn pf5_textarea_remains_a_coordinated_negative_control() {
    let analysis = analyze("<textarea>abc</textarea>");
    assert_eq!(
        tree_unsupported(&analysis),
        Some(HtmlTreeCapability::NonShellElementTag)
    );
    assert!(
        analysis
            .nodes_in_creation_order()
            .iter()
            .all(|node| !matches!(
                node.kind(),
                HtmlTreeNodeKind::Element(HtmlElement::Title(_))
            ))
    );
}

/// Falsifies: a Title admitted outside its selected lifecycle, or carrying
/// tree-owned attribute/self-closing shape the tokenizer must not decide.
#[test]
fn pf6_title_outside_the_selected_lifecycle_is_refused_by_the_tree() {
    assert_eq!(
        tree_unsupported(&analyze("<title lang=en>x</title>")),
        Some(HtmlTreeCapability::TitleTagAttribute)
    );
    assert_eq!(
        tree_unsupported(&analyze("<title/>")),
        Some(HtmlTreeCapability::SelfClosingTitleTag)
    );
    assert_eq!(
        tree_unsupported(&analyze("<body><title>x</title>")),
        Some(HtmlTreeCapability::TitleTagOutsideSelectedLifecycle)
    );
    // A `</title>` with no episode open is not a Text-mode close.
    assert_eq!(
        tree_unsupported(&analyze("</title>")),
        Some(HtmlTreeCapability::TitleTagOutsideSelectedLifecycle)
    );
}

// ---------------------------------------------------------------------------
// Matching
// ---------------------------------------------------------------------------

/// Falsifies: exact whole-string lookup, and shortest match.
///
/// No `notit` identifier exists. Whole-string lookup would see `notit;` and
/// resolve nothing; shortest match would resolve `not` for both inputs.
#[test]
fn pf7_matching_is_maximum_match_over_the_canonical_table() {
    let shorter = analyze("<title>&notit;</title>");
    assert_eq!(title_text(&shorter), "\u{ac}it;");
    assert_eq!(
        title_contributions(&shorter),
        vec![((7, 11), "\u{ac}".to_owned()), ((11, 14), "it;".to_owned())]
    );

    let longer = analyze("<title>&notin;</title>");
    assert_eq!(title_text(&longer), "\u{2209}");
    assert_eq!(
        title_contributions(&longer),
        vec![((7, 14), "\u{2209}".to_owned())]
    );
}

/// Falsifies: a partial, copied, whitelisted, or hand-maintained table.
///
/// The longest canonical identifier is 32 bytes, so this also exercises the
/// exact bound the owner publishes: one materialized scalar plus 31 borrowed.
#[test]
fn pf8_the_longest_canonical_identifier_resolves() {
    let analysis = analyze("<title>&CounterClockwiseContourIntegral;</title>");
    assert_eq!(title_text(&analysis), "\u{2233}");
    assert_eq!(
        title_contributions(&analysis),
        vec![((7, 40), "\u{2233}".to_owned())]
    );
    assert!(analysis.tokenizer_run().diagnostics().is_empty());
}

/// Falsifies: multi-scalar truncation, and a decoded value split across more
/// than one authored origin.
#[test]
fn pf9_a_two_scalar_reference_keeps_one_authored_origin() {
    let analysis = analyze("<title>&acE;</title>");
    assert_eq!(title_text(&analysis), "\u{223e}\u{333}");
    assert_eq!(title_text(&analysis).chars().count(), 2);
    assert_eq!(
        title_contributions(&analysis),
        vec![((7, 12), "\u{223e}\u{333}".to_owned())]
    );
    // Exactly one character token carries both scalars.
    assert_eq!(
        character_tokens(analysis.tokenizer_run()),
        vec![((7, 12), "\u{223e}\u{333}".to_owned())]
    );
}

/// Falsifies: recursive decoding, and decoded output re-entering the
/// tokenizer as markup.
#[test]
fn pf10_decoded_output_is_never_fed_back_as_tokenizer_input() {
    // A decoded `<` must not open a tag, and the authored `</title>` after it
    // must still be the close.
    let markup = analyze("<title>&lt;/title></title>");
    assert_eq!(title_text(&markup), "</title>");
    assert_eq!(
        title_contributions(&markup),
        vec![((7, 11), "<".to_owned()), ((11, 18), "/title>".to_owned())]
    );

    // A decoded `&` must not begin a second reference: `&amp;amp;` is `&`
    // followed by the literal text `amp;`.
    let recursive = analyze("<title>&amp;amp;</title>");
    assert_eq!(title_text(&recursive), "&amp;");
    assert_eq!(
        title_contributions(&recursive),
        vec![((7, 12), "&".to_owned()), ((12, 16), "amp;".to_owned())]
    );
}

// ---------------------------------------------------------------------------
// Ambiguous ampersand and diagnostics
// ---------------------------------------------------------------------------

/// Falsifies: silently consuming the `;` of an unresolved name, and losing
/// the authored text of an unresolved run.
#[test]
fn pf11_an_unresolved_name_preserves_authored_text_and_reconsumes_its_semicolon() {
    let analysis = analyze("<title>&nope;x</title>");
    assert_eq!(title_text(&analysis), "&nope;x");
    // The unresolved candidate closes at its own boundary. The authored `;`
    // is never consumed as part of a nonexistent entity: it stays authored
    // input, is reconsumed in RCDATA, and belongs to the *following*
    // contribution. Collapsing these two into one `(7, 14)` range would erase
    // the boundary the selected lifecycle is defined by.
    assert_eq!(
        title_contributions(&analysis),
        vec![((7, 12), "&nope".to_owned()), ((12, 14), ";x".to_owned())]
    );
    assert_eq!(
        tokenizer_diagnostics(analysis.tokenizer_run()),
        vec![(
            super::super::tokenizer::diagnostic::HtmlTokenizerDiagnosticCode::
                UnknownNamedCharacterReference,
            // Anchored at the authored `;` itself.
            (12, 13)
        )]
    );

    // Without a `;`, the ambiguous run carries no diagnostic at all, and the
    // delimiter still belongs to the following contribution.
    let unterminated = analyze("<title>&nope </title>");
    assert_eq!(title_text(&unterminated), "&nope ");
    assert_eq!(
        title_contributions(&unterminated),
        vec![((7, 12), "&nope".to_owned()), ((12, 13), " ".to_owned())]
    );
    assert!(unterminated.tokenizer_run().diagnostics().is_empty());

    // Ordinary RCDATA text observed before the `&` is prior evidence and is
    // its own contribution, so the unresolved candidate keeps its own
    // authored origin exactly as a resolved reference does.
    let preceded = analyze("<title>abc&bogus;</title>");
    assert_eq!(
        title_contributions(&preceded),
        vec![
            ((7, 10), "abc".to_owned()),
            ((10, 16), "&bogus".to_owned()),
            ((16, 17), ";".to_owned()),
        ]
    );
}

/// Falsifies: an unresolved candidate whose emission refusal takes the
/// authored delimiter, or the prior text, down with it.
#[test]
fn pf11b_an_unresolved_candidate_refuses_transactionally() {
    // Room for the `<title>` tag and the prior `abc` run only, so closing the
    // unresolved candidate is refused.
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 2, 4_096, 256, 16_384, 4_096);
    let analysis = analyze_with("<title>abc&bogus;</title>", 1, constrained);
    let run = analysis.tokenizer_run();
    assert_eq!(
        resource_stop(run),
        Some(HtmlTokenizerResource::EmittedTokens)
    );
    // Prior evidence survives, and no token merges the candidate with the
    // authored delimiter.
    assert_eq!(character_tokens(run), vec![((7, 10), "abc".to_owned())]);
    // The observation-conditioned diagnostic was recorded *before* the
    // refused close, so it survives it. Recording it also commits its own
    // observed location to the processed prefix, which is the established
    // engine rule for every diagnostic — hence coverage ends at the `;`.
    assert_eq!(
        tokenizer_diagnostics(run),
        vec![(
            super::super::tokenizer::diagnostic::HtmlTokenizerDiagnosticCode::
                UnknownNamedCharacterReference,
            (16, 17)
        )]
    );
    assert_eq!(run.coverage().processed_end(), 17);

    // The same refusal with no diagnostic in play: the candidate's own
    // emission is what is refused, and the authored delimiter is untouched,
    // so coverage stops exactly at the candidate's boundary.
    let analysis = analyze_with("<title>abc&bogus </title>", 1, constrained);
    let run = analysis.tokenizer_run();
    assert_eq!(
        resource_stop(run),
        Some(HtmlTokenizerResource::EmittedTokens)
    );
    assert_eq!(character_tokens(run), vec![((7, 10), "abc".to_owned())]);
    assert!(run.diagnostics().is_empty());
    assert_eq!(run.coverage().processed_end(), 16);
}

/// Falsifies: a missing-semicolon diagnostic anchored anywhere but the final
/// authored scalar of the matched name, or related to the wrong token.
#[test]
fn pf12_missing_semicolon_is_anchored_at_the_matched_name_and_names_its_token() {
    use super::super::tokenizer::diagnostic::{
        HtmlTokenizerDiagnosticCode, HtmlTokenizerDiagnosticSubject,
    };

    let analysis = analyze("<title>a&amp b</title>");
    assert_eq!(title_text(&analysis), "a& b");
    let run = analysis.tokenizer_run();
    assert_eq!(run.diagnostics().len(), 1);
    let diagnostic = &run.diagnostics()[0];
    assert_eq!(
        diagnostic.code(),
        HtmlTokenizerDiagnosticCode::MissingSemicolonAfterCharacterReference
    );
    // `&amp` spans 8..12, so its final authored scalar is 11..12.
    assert_eq!(
        (
            diagnostic.location().range().start(),
            diagnostic.location().range().end()
        ),
        (11, 12)
    );
    let HtmlTokenizerDiagnosticSubject::EmittedToken { token_index } = diagnostic.subject() else {
        panic!("a resolved reference's diagnostic names its emitted token");
    };
    let HtmlToken::Character(character) = &run.tokens()[*token_index] else {
        panic!("the referenced token is the resolved character token");
    };
    assert_eq!(character.interpreted(), "&");
    assert_eq!(
        (
            character.source().range().start(),
            character.source().range().end()
        ),
        (8, 12)
    );
    // Observation-conditioned, so it may not be an emission-conditioned code.
    assert!(!diagnostic.code().is_emission_conditioned());
}

/// Falsifies: speculative lookahead that preprocesses future input.
///
/// If discovery materialized the scalar that ends the match, the control
/// character's preprocessing diagnostic would be recorded *before* the
/// missing-semicolon diagnostic that authored order requires first.
#[test]
fn pf13_missing_semicolon_precedes_a_later_preprocessing_diagnostic() {
    use super::super::tokenizer::diagnostic::HtmlTokenizerDiagnosticCode;

    let analysis = analyze("<title>&not\u{1}z</title>");
    assert_eq!(
        tokenizer_diagnostics(analysis.tokenizer_run()),
        vec![
            (
                HtmlTokenizerDiagnosticCode::MissingSemicolonAfterCharacterReference,
                (10, 11)
            ),
            (
                HtmlTokenizerDiagnosticCode::ControlCharacterInInputStream,
                (11, 12)
            ),
        ]
    );
    assert_eq!(title_text(&analysis), "\u{ac}\u{1}z");
}

/// Falsifies: an `&` that cannot begin a reference being lost or mistreated.
#[test]
fn pf14_an_ampersand_that_begins_no_reference_stays_ordinary_text() {
    assert_eq!(title_text(&analyze("<title>&</title>")), "&");
    assert_eq!(title_text(&analyze("<title>& x</title>")), "& x");
    assert_eq!(title_text(&analyze("<title>&&amp;</title>")), "&&");
    // An `&` immediately before the appropriate close is still text.
    assert_eq!(
        title_contributions(&analyze("<title>a&</title>")),
        vec![((7, 9), "a&".to_owned())]
    );
}

// ---------------------------------------------------------------------------
// Retained unsupported boundaries
// ---------------------------------------------------------------------------

/// Falsifies: incidental Numeric Character Reference support, and a numeric
/// refusal that reports the wrong availability or trigger.
#[test]
fn pf15_numeric_character_references_are_refused_at_the_authored_hash() {
    let analysis = analyze("<title>&#38;</title>");
    assert_eq!(
        tokenizer_unsupported(analysis.tokenizer_run()),
        Some(ObservedUnsupported {
            capability: HtmlTokenizerCapability::NumericCharacterReferenceInRcdata,
            // `Unsupported`, not `Deferred`: no tree feedback can discharge it.
            availability: HtmlTokenizerCapabilityAvailability::Unsupported,
            trigger: Some((8, 9)),
        })
    );
    // The authored `&` that caused entry is committed; nothing beyond it is.
    assert_eq!(analysis.tokenizer_run().coverage().processed_end(), 8);
    assert!(character_tokens(analysis.tokenizer_run()).is_empty());
    assert!(analysis.tokenizer_run().diagnostics().is_empty());
}

/// Falsifies: incidental RCDATA NUL recovery, or a NUL replacement claimed
/// without support.
#[test]
fn pf16_rcdata_nul_is_refused_without_replacement_or_recovery() {
    let analysis = analyze("<title>a\u{0}b</title>");
    assert_eq!(
        tokenizer_unsupported(analysis.tokenizer_run()),
        Some(ObservedUnsupported {
            capability: HtmlTokenizerCapability::RcdataNullRecovery,
            availability: HtmlTokenizerCapabilityAvailability::Unsupported,
            trigger: Some((8, 9)),
        })
    );
    // The NUL scalar is not committed and no replacement is claimed.
    assert_eq!(analysis.tokenizer_run().coverage().processed_end(), 8);
    assert_eq!(
        character_tokens(analysis.tokenizer_run()),
        vec![((7, 8), "a".to_owned())]
    );
    assert!(
        !character_tokens(analysis.tokenizer_run())
            .iter()
            .any(|(_, interpreted)| interpreted.contains('\u{fffd}'))
    );
    assert!(analysis.tokenizer_run().diagnostics().is_empty());
}

/// Falsifies: general Character Reference support leaking out of the selected
/// RCDATA slice, and the standalone tokenizer silently gaining capability.
#[test]
fn pf17_standalone_and_general_boundaries_remain_exactly_as_deferred_as_before() {
    let expect_deferred = |text: &str, capability: HtmlTokenizerCapability| {
        let source = SourceText::new(SourceId::new(1), text.to_owned());
        let run = tokenize(&source, limits());
        assert_eq!(
            tokenizer_unsupported(&run)
                .map(|observed| (observed.capability, observed.availability)),
            Some((capability, HtmlTokenizerCapabilityAvailability::Deferred)),
            "standalone {text:?}"
        );
    };
    // Only coordinated Title discharges the selected RCDATA boundary; driven
    // standalone, `<title>` is exactly as deferred as before TC-S10.
    expect_deferred(
        "<title>x",
        HtmlTokenizerCapability::ContextDependentTokenizerMode {
            mode: HtmlTokenizerMode::Rcdata,
        },
    );
    expect_deferred(
        "<textarea>x",
        HtmlTokenizerCapability::ContextDependentTokenizerMode {
            mode: HtmlTokenizerMode::Rcdata,
        },
    );
    expect_deferred(
        "&amp;",
        HtmlTokenizerCapability::CharacterReference {
            context: HtmlCharacterReferenceContext::Data,
        },
    );
    expect_deferred(
        "<p id=\"&amp;\">",
        HtmlTokenizerCapability::CharacterReference {
            context: HtmlCharacterReferenceContext::AttributeValue,
        },
    );
    expect_deferred(
        "<style>x",
        HtmlTokenizerCapability::ContextDependentTokenizerMode {
            mode: HtmlTokenizerMode::RawText,
        },
    );
}

// ---------------------------------------------------------------------------
// Appropriate close and EOF
// ---------------------------------------------------------------------------

/// Falsifies: the tree lexically rediscovering the close, a fabricated close
/// token, and a non-appropriate `</title>`-shaped run ending the episode.
#[test]
fn pf18_the_appropriate_close_is_the_tokenizer_s_own_emitted_end_tag() {
    let analysis = analyze("<title>a</TiTlE>");
    let run = analysis.tokenizer_run();
    let close_index = analysis
        .actions()
        .iter()
        .find(|action| {
            matches!(
                action.kind(),
                HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { .. }
            )
        })
        .map(|action| action.trigger().token_index())
        .expect("an authored Title close");
    let HtmlToken::Tag(tag) = &run.tokens()[close_index] else {
        panic!("the close trigger is the emitted end-tag token");
    };
    assert_eq!(tag.kind(), HtmlTagKind::End);
    assert_eq!(tag.name().interpreted(), "title");
    // ASCII-case-insensitive appropriate matching, with authored spelling kept.
    assert_eq!(tag.name().source().fragment(), "TiTlE");
    assert_eq!(title_text(&analysis), "a");

    // A `</titl>` is not appropriate: it stays RCDATA text.
    let inappropriate = analyze("<title>a</titl></title>");
    assert_eq!(title_text(&inappropriate), "a</titl>");

    // A `</titlex>` prefix is not appropriate either.
    let longer = analyze("<title>a</titlex></title>");
    assert_eq!(title_text(&longer), "a</titlex>");
}

/// Falsifies: a second tokenizer EOF, fabricated authored closing evidence,
/// and a tree that fails to restore InHead.
#[test]
fn pf19_eof_in_title_text_pops_and_reprocesses_the_same_retained_token() {
    let analysis = analyze("<title>abc");
    let run = analysis.tokenizer_run();

    // Exactly one EOF token, and it is last.
    let eof_indices: Vec<usize> = run
        .tokens()
        .iter()
        .enumerate()
        .filter(|(_, token)| matches!(token, HtmlToken::EndOfFile(_)))
        .map(|(index, _)| index)
        .collect();
    assert_eq!(eof_indices.len(), 1);
    assert_eq!(eof_indices[0] + 1, run.tokens().len());

    // No authored closing evidence was fabricated: no end-tag token exists.
    assert!(!run.tokens().iter().any(|token| matches!(
        token,
        HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::End
    )));

    let eof = eof_indices[0];
    let kinds: Vec<&HtmlTreeActionKind> = analysis
        .actions()
        .iter()
        .filter(|action| action.trigger().token_index() == eof)
        .map(|action| action.kind())
        .collect();
    // The pop is immediately followed by the redispatch of the same token.
    let pop = kinds
        .iter()
        .position(|kind| {
            matches!(
                kind,
                HtmlTreeActionKind::PoppedTitleElementAtEndOfFile { .. }
            )
        })
        .expect("a Title EOF pop");
    assert!(matches!(
        kinds[pop + 1],
        HtmlTreeActionKind::ReprocessedToken
    ));
    // Restoring InHead is what lets the retained EOF then imply the head close.
    assert!(
        kinds
            .iter()
            .any(|kind| matches!(kind, HtmlTreeActionKind::ClosedShellElement { .. }))
    );

    let diagnostics: Vec<(HtmlTreeDiagnosticCode, HtmlTreeRecovery)> = analysis
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.trigger().token_index() == eof)
        .map(|diagnostic| (diagnostic.code(), diagnostic.recovery()))
        .collect();
    assert_eq!(
        diagnostics,
        vec![(
            HtmlTreeDiagnosticCode::TitleEndOfFileInText,
            HtmlTreeRecovery::PoppedTitleAtEndOfFileAndRestoredInHead
        )]
    );
    assert_eq!(title_text(&analysis), "abc");
}

/// Falsifies: an EOF reached mid-reference silently resolving or fabricating.
#[test]
fn pf20_eof_inside_an_unterminated_reference_resolves_only_what_matched() {
    // `&am` completes no identifier, so the run is ordinary text.
    let ambiguous = analyze("<title>&am");
    assert_eq!(title_text(&ambiguous), "&am");
    assert!(ambiguous.tokenizer_run().diagnostics().is_empty());

    // `&amp` is a complete semicolonless identifier even at EOF.
    let resolved = analyze("<title>&amp");
    assert_eq!(title_text(&resolved), "&");
    assert_eq!(
        title_contributions(&resolved),
        vec![((7, 11), "&".to_owned())]
    );
}

// ---------------------------------------------------------------------------
// Provenance and determinism
// ---------------------------------------------------------------------------

/// Falsifies: coalesced text standing in for provenance, and `SourceId` loss.
#[test]
fn pf21_provenance_survives_coalescing_and_source_identity() {
    let text = "<title>a&notin;b&amp c</title>";
    let analysis = analyze(text);
    assert_eq!(title_text(&analysis), "a\u{2209}b& c");
    assert_eq!(
        title_contributions(&analysis),
        vec![
            ((7, 8), "a".to_owned()),
            ((8, 15), "\u{2209}".to_owned()),
            ((15, 16), "b".to_owned()),
            ((16, 20), "&".to_owned()),
            ((20, 22), " c".to_owned()),
        ]
    );

    // Every retained anchor revalidates against the exact authored bytes and
    // keeps the caller-supplied identity.
    for source_id in [1u64, 7u64] {
        let source = SourceText::new(SourceId::new(source_id), text.to_owned());
        let analysis =
            construct_html_document_shell(&source, limits()).expect("production boundary");
        let title = title_id(&analysis);
        for child in analysis.node(title).expect("Title").children() {
            let HtmlTreeNodeKind::Text(node) = analysis.node(*child).expect("child").kind() else {
                panic!("a Title child must be text");
            };
            for contribution in node.contributions() {
                assert_eq!(contribution.source().source_id(), source.id());
                let range = contribution.source().range();
                assert_eq!(
                    &text[range.start()..range.end()],
                    contribution.source().fragment()
                );
            }
        }
    }
}

/// Falsifies: non-deterministic replay across repeats and source identities.
#[test]
fn pf22_the_selected_lifecycle_replays_deterministically() {
    for text in [
        "<title>&notit;&notin;&acE;</title>",
        "<title>&nope;&amp x</title>",
        "<title>abc",
        "<title></title>",
    ] {
        let baseline = semantic_signature(&analyze_with(text, 1, limits()));
        for source_id in [1u64, 2u64, 9u64] {
            assert_eq!(
                semantic_signature(&analyze_with(text, source_id, limits())),
                baseline,
                "{text:?} under SourceId {source_id}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Falsifies: a matcher that retains a copied candidate buffer.
#[test]
fn pf23_the_selected_matcher_keeps_temporary_buffer_bytes_at_zero() {
    for text in [
        "<title>&CounterClockwiseContourIntegral;</title>",
        "<title>&notit;&notin;&acE;&nope;&amp x</title>",
        "<title>&nomatchatallhereatall",
    ] {
        let analysis = analyze(text);
        assert_eq!(
            analysis
                .tokenizer_run()
                .usage()
                .peak_temporary_buffer_bytes(),
            0,
            "{text:?}"
        );
        assert_eq!(
            analysis
                .tokenizer_run()
                .limits()
                .max_temporary_buffer_bytes(),
            limits().max_temporary_buffer_bytes()
        );
    }
}

/// Falsifies: a reference committed past its retained-byte preflight, and a
/// refusal that destroys earlier valid evidence.
#[test]
fn pf24_a_retained_byte_refusal_commits_no_part_of_the_reference() {
    // `title` (5) + `abc` (3) = 8 committed; `&acE;` decodes to 5 more.
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 4_096, 256, 12, 4_096);
    let analysis = analyze_with("<title>abc&acE;</title>", 1, constrained);
    let run = analysis.tokenizer_run();
    assert_eq!(
        resource_stop(run),
        Some(HtmlTokenizerResource::RetainedInterpretedBytes)
    );
    // Prior valid evidence survives; no partial or whole reference committed.
    assert_eq!(character_tokens(run), vec![((7, 10), "abc".to_owned())]);
    // Only the authored `&` was consumed beyond the surviving run: no scalar
    // of the identifier itself is inside committed coverage.
    assert_eq!(run.coverage().processed_end(), 11);
    assert!(run.diagnostics().is_empty());
}

/// Falsifies: a reference emitted past its own token-capacity preflight, and
/// a refusal that loses the single-increment attempt shape the frozen run
/// contract requires.
#[test]
fn pf25_an_emitted_token_refusal_commits_no_part_of_the_reference() {
    // Room for the `<title>` tag and the pending run, but not the reference.
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 2, 4_096, 256, 16_384, 4_096);
    let analysis = analyze_with("<title>abc&acE;</title>", 1, constrained);
    let run = analysis.tokenizer_run();
    assert_eq!(
        resource_stop(run),
        Some(HtmlTokenizerResource::EmittedTokens)
    );
    // The pending run survives as prior valid evidence; the reference does not
    // exist, and no identifier scalar is committed.
    assert_eq!(character_tokens(run), vec![((7, 10), "abc".to_owned())]);
    assert_eq!(run.coverage().processed_end(), 11);

    // Room for the `<title>` tag alone, so even the prior run cannot emit. The
    // refusal must still be a valid one-token attempt, or the frozen run would
    // fail its own contract instead of reporting honestly.
    let tighter = HtmlTokenizerLimits::new(4_096, 32_768, 1, 4_096, 256, 16_384, 4_096);
    let analysis = analyze_with("<title>abc&acE;</title>", 1, tighter);
    let run = analysis.tokenizer_run();
    assert_eq!(
        resource_stop(run),
        Some(HtmlTokenizerResource::EmittedTokens)
    );
    assert!(character_tokens(run).is_empty());
    assert_eq!(run.usage().emitted_tokens(), 1);
}

/// Falsifies: a resolved reference committed without capacity for the
/// diagnostic its result requires.
#[test]
fn pf26_a_diagnostic_refusal_commits_no_part_of_the_reference() {
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 0, 256, 16_384, 4_096);
    let analysis = analyze_with("<title>&not</title>", 1, constrained);
    let run = analysis.tokenizer_run();
    assert_eq!(resource_stop(run), Some(HtmlTokenizerResource::Diagnostics));
    assert!(character_tokens(run).is_empty());
    assert!(run.diagnostics().is_empty());
    // Coverage stops right after the authored `&`.
    assert_eq!(run.coverage().processed_end(), 8);

    // A match needing no diagnostic is unaffected by the same zero budget.
    let clean = analyze_with("<title>&amp;</title>", 1, constrained);
    assert_eq!(title_text(&clean), "&");
}

/// Falsifies: any selected authored source consumed before every semantic
/// candidate and preflight is ready.
///
/// The selected Named transaction is ordered preflight -> construct -> consume
/// -> infallible commit, so committed coverage can never land *strictly
/// inside* a matched identifier: either the transaction refused, and coverage
/// stops at the authored `&` that caused entry with no reference evidence at
/// all, or it committed, and the identifier is whole. This sweeps every
/// selected preflight boundary rather than trusting any single one.
#[test]
fn pf26b_no_selected_source_is_consumed_before_the_transaction_is_ready() {
    struct Case {
        text: &'static str,
        ampersand_end: usize,
        identifier_end: usize,
        reference_start: usize,
        needs_diagnostic: bool,
    }
    let cases = [
        Case {
            text: "<title>abc&acE;</title>",
            ampersand_end: 11,
            identifier_end: 15,
            reference_start: 10,
            needs_diagnostic: false,
        },
        Case {
            // Semicolonless, so the transaction additionally reserves and
            // constructs a missing-semicolon diagnostic before consuming.
            text: "<title>&not</title>",
            ampersand_end: 8,
            identifier_end: 11,
            reference_start: 7,
            needs_diagnostic: true,
        },
    ];

    let mut refusals = 0usize;
    let mut commits = 0usize;
    for case in &cases {
        for retained in [0usize, 1, 5, 6, 7, 8, 9, 10, 11, 12, 4_096] {
            for tokens in [0usize, 1, 2, 3, 4_096] {
                for diagnostics in [0usize, 1, 4_096] {
                    let limits = HtmlTokenizerLimits::new(
                        4_096,
                        32_768,
                        tokens,
                        diagnostics,
                        256,
                        retained,
                        4_096,
                    );
                    let analysis = analyze_with(case.text, 1, limits);
                    let run = analysis.tokenizer_run();
                    let covered = run.coverage().processed_end();
                    let label = format!(
                        "{:?} retained={retained} tokens={tokens} diagnostics={diagnostics}",
                        case.text
                    );
                    assert!(
                        covered <= case.ampersand_end || covered >= case.identifier_end,
                        "{label}: coverage {covered} landed inside the matched identifier"
                    );

                    let reference = character_tokens(run)
                        .into_iter()
                        .find(|(range, _)| range.0 == case.reference_start);
                    if covered <= case.ampersand_end {
                        refusals += 1;
                        // Refused: no reference evidence of any kind exists.
                        assert!(
                            reference.is_none(),
                            "{label}: a refused reference still emitted a token"
                        );
                        if case.needs_diagnostic {
                            assert!(
                                run.diagnostics().is_empty(),
                                "{label}: a refused reference still recorded its diagnostic"
                            );
                        }
                    } else if let Some((range, _)) = reference {
                        commits += 1;
                        // Committed: whole, never partial.
                        assert_eq!(
                            range,
                            (case.reference_start, case.identifier_end),
                            "{label}: a committed reference is not its whole authored span"
                        );
                        // And complete: the diagnostic the result requires was
                        // reserved and constructed before consumption, so a
                        // committed reference can never be missing it.
                        if case.needs_diagnostic {
                            assert!(
                                !run.diagnostics().is_empty(),
                                "{label}: a committed reference lost its required diagnostic"
                            );
                        }
                    }
                }
            }
        }
    }
    assert!(refusals > 0 && commits > 0, "the sweep must exercise both");
}

/// Falsifies: charging one outer transition per authored scalar of a matched
/// identifier.
///
/// The selected Named operation is one transition-level step: bounded
/// discovery plus bounded matched-source consumption. Its cost must therefore
/// be independent of how long the resolved identifier is. Per-scalar charging
/// makes these three diverge by exactly the identifier lengths.
#[test]
fn pf27_a_resolved_identifier_costs_one_transition_level_operation() {
    let short = analyze("<title>&gt;</title>");
    let medium = analyze("<title>&notin;</title>");
    let longest = analyze("<title>&CounterClockwiseContourIntegral;</title>");

    // Identifier lengths of 3, 6 and 32 authored bytes.
    assert_eq!(short.tokenizer_run().usage().source_bytes(), 19);
    assert_eq!(medium.tokenizer_run().usage().source_bytes(), 22);
    assert_eq!(longest.tokenizer_run().usage().source_bytes(), 48);

    let steps = short.tokenizer_run().usage().transition_steps();
    assert_eq!(medium.tokenizer_run().usage().transition_steps(), steps);
    assert_eq!(longest.tokenizer_run().usage().transition_steps(), steps);

    // Deterministic across repeats.
    for text in [
        "<title>&gt;</title>",
        "<title>&notin;</title>",
        "<title>&CounterClockwiseContourIntegral;</title>",
    ] {
        assert_eq!(
            analyze(text).tokenizer_run().usage().transition_steps(),
            analyze(text).tokenizer_run().usage().transition_steps()
        );
    }
}

/// Falsifies: a `TransitionSteps` refusal that exposes authored identifier
/// scalars inside committed coverage with no evidence to explain them.
///
/// This is the FA400-01 reproducer. On the rejected head, a step budget of 14
/// over `<title>abc&notin;</title>` covered through `<title>abc&no` while only
/// `abc` had token evidence. Because the whole selected identifier is now one
/// transition-level operation, every budget either stops before the identifier
/// begins or commits it whole.
#[test]
fn pf27b_a_transition_step_refusal_never_splits_a_matched_identifier() {
    let text = "<title>abc&notin;</title>";
    // `&` ends at 11; the identifier spans 11..17.
    const AMPERSAND_END: usize = 11;
    const IDENTIFIER_END: usize = 17;

    for steps in 1..=40 {
        let limits = HtmlTokenizerLimits::new(4_096, steps, 4_096, 4_096, 256, 16_384, 4_096);
        let analysis = analyze_with(text, 1, limits);
        let run = analysis.tokenizer_run();
        let covered = run.coverage().processed_end();
        assert!(
            covered <= AMPERSAND_END || covered >= IDENTIFIER_END,
            "step budget {steps} covered {covered}, splitting the matched identifier"
        );
        // Whenever coverage passed the identifier, its resolved evidence
        // exists; whenever it did not, no part of the identifier is claimed.
        let resolved = character_tokens(run)
            .into_iter()
            .any(|(range, _)| range == (10, IDENTIFIER_END));
        assert_eq!(
            resolved,
            covered >= IDENTIFIER_END,
            "step budget {steps} disagrees about the resolved reference at coverage {covered}"
        );
    }
}

// ---------------------------------------------------------------------------
// Freeze
// ---------------------------------------------------------------------------

struct PartsFixture {
    source: SourceText,
    run: HtmlTokenizerRunResult,
    parts: HtmlDocumentShellParts,
}

/// Drives the production coordination seam directly so freeze can be attacked
/// with corrupted coordination facts.
fn coordinated_parts(source_text: &str) -> PartsFixture {
    let source = SourceText::new(SourceId::new(410), source_text.to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&source, limits());
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
                        assert_eq!(feedback, HtmlTreeTokenizerFeedback::EnterRcdataForTitle);
                        assert_eq!(next_token + 1, produced);
                        assert_eq!(
                            boundary,
                            HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::Rcdata)
                        );
                        tokenizer.apply_title_rcdata().expect("apply Title RCDATA");
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
    parts.coordinated_rcdata_entry_tokens = entry_tokens;
    parts.coordinated_rcdata_close_tokens = close_tokens;
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

/// The honest fixture freezes.
#[test]
fn pf28_the_selected_coordinated_lifecycle_freezes() {
    for text in [
        "<title>&notin;</title>",
        "<title></title>",
        "<title>&nope; text</title>",
    ] {
        let fixture = coordinated_parts(text);
        let parts = coordinated_parts(text).parts;
        assert!(freeze_fixture(&fixture, parts).is_ok(), "{text:?}");
    }
}

/// Falsifies: invalid RCDATA episode accounting surviving the freeze.
#[test]
fn pf29_freeze_rejects_corrupted_rcdata_coordination_facts() {
    let fixture = coordinated_parts("<title>x</title>");

    let mut missing_entry = coordinated_parts("<title>x</title>").parts;
    missing_entry.coordinated_rcdata_entry_tokens.clear();
    assert!(matches!(
        freeze_fixture(&fixture, missing_entry),
        Err(HtmlTreeFreezeError::TitleCoordinationEntryMismatch { .. })
    ));

    let mut missing_close = coordinated_parts("<title>x</title>").parts;
    missing_close.coordinated_rcdata_close_tokens.clear();
    assert!(matches!(
        freeze_fixture(&fixture, missing_close),
        Err(HtmlTreeFreezeError::TitleCoordinationCloseMismatch { .. })
    ));

    // A Title episode may not be accounted as a RAWTEXT one.
    let mut wrong_domain = coordinated_parts("<title>x</title>").parts;
    wrong_domain.coordinated_raw_text_entry_tokens =
        wrong_domain.coordinated_rcdata_entry_tokens.clone();
    assert!(matches!(
        freeze_fixture(&fixture, wrong_domain),
        Err(HtmlTreeFreezeError::StyleCoordinationEntryMismatch { .. })
    ));

    let mut pending = coordinated_parts("<title>x</title>").parts;
    pending.pending_tokenizer_feedback = true;
    assert!(matches!(
        freeze_fixture(&fixture, pending),
        Err(HtmlTreeFreezeError::OutstandingTokenizerFeedback)
    ));
}

/// Falsifies: mismatched final insertion-mode facts surviving the freeze.
#[test]
fn pf30_freeze_rejects_wrong_terminal_text_mode_state() {
    let fixture = coordinated_parts("<title>x</title>");

    let mut text_mode = coordinated_parts("<title>x</title>").parts;
    text_mode.final_text_mode_active = true;
    assert!(matches!(
        freeze_fixture(&fixture, text_mode),
        Err(HtmlTreeFreezeError::FinalTextModeStateMismatch)
    ));

    let mut retained = coordinated_parts("<title>x</title>").parts;
    retained.final_original_insertion_mode_retained = true;
    assert!(matches!(
        freeze_fixture(&fixture, retained),
        Err(HtmlTreeFreezeError::FinalTextModeStateMismatch)
    ));

    let mut claimed_open = coordinated_parts("<title>x</title>").parts;
    claimed_open.final_open_title = Some(title_node_id(&claimed_open));
    assert!(matches!(
        freeze_fixture(&fixture, claimed_open),
        Err(HtmlTreeFreezeError::FinalTitleStateMismatch)
    ));

    // A Title may not be claimed open as a Style, and vice versa.
    let mut cross_domain = coordinated_parts("<title>x</title>").parts;
    cross_domain.final_open_style = Some(title_node_id(&cross_domain));
    assert!(matches!(
        freeze_fixture(&fixture, cross_domain),
        Err(HtmlTreeFreezeError::FinalOpenStyleIsNotStyle(_))
    ));
}

/// Falsifies: a replayed action chronology that no longer matches its tokens.
#[test]
fn pf31_freeze_rejects_replayed_lifecycle_corruption() {
    let fixture = coordinated_parts("<title>x</title>");

    let mut chronology = coordinated_parts("<title>x</title>").parts;
    let insert = action_index(&chronology, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::InsertedAuthoredTitleElement { .. }
        )
    });
    let close = action_index(&chronology, |kind| {
        matches!(
            kind,
            HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { .. }
        )
    });
    chronology.actions.swap(insert, close);
    assert!(freeze_fixture(&fixture, chronology).is_err());

    // A Title whose episode never closed cannot be a Complete document. The
    // Numeric boundary leaves exactly that state: the tokenizer refused while
    // the tree still holds the Title open in Text.
    let open_fixture = coordinated_parts("<title>&#38;");
    assert!(open_fixture.parts.final_open_title.is_some());
    assert!(open_fixture.parts.final_text_mode_active);
    let mut complete = coordinated_parts("<title>&#38;").parts;
    complete.completion = HtmlTreeCompletion::Complete;
    assert!(matches!(
        freeze_fixture(&open_fixture, complete),
        Err(HtmlTreeFreezeError::CompleteTitleStateMismatch)
    ));
}

// ---------------------------------------------------------------------------
// Predecessor regression controls
// ---------------------------------------------------------------------------

/// Falsifies: the TC-S10 Title lifecycle being satisfiable by TC-S9 Style
/// facts, in the direction the sibling TC-S9 test does not cover.
#[test]
fn pf36_style_facts_cannot_satisfy_the_title_lifecycle() {
    let fixture = coordinated_parts("<title>x</title>");

    // A Title node is not a Style.
    let mut claimed_style = coordinated_parts("<title>x</title>").parts;
    claimed_style.final_open_style = Some(title_node_id(&claimed_style));
    assert!(matches!(
        freeze_fixture(&fixture, claimed_style),
        Err(HtmlTreeFreezeError::FinalOpenStyleIsNotStyle(_))
    ));

    // An RCDATA episode's coordination is not a RAWTEXT episode's.
    let mut cross_coordinated = coordinated_parts("<title>x</title>").parts;
    cross_coordinated.coordinated_raw_text_entry_tokens =
        cross_coordinated.coordinated_rcdata_entry_tokens.clone();
    assert!(matches!(
        freeze_fixture(&fixture, cross_coordinated),
        Err(HtmlTreeFreezeError::StyleCoordinationEntryMismatch { .. })
    ));

    let mut cross_close = coordinated_parts("<title>x</title>").parts;
    cross_close.coordinated_raw_text_close_tokens =
        cross_close.coordinated_rcdata_close_tokens.clone();
    assert!(matches!(
        freeze_fixture(&fixture, cross_close),
        Err(HtmlTreeFreezeError::StyleCoordinationCloseMismatch { .. })
    ));

    // A Title EOF episode's diagnostic is Title's own: it can never be
    // matched by the Style replay, which would leave it orphaned there.
    let eof_fixture = coordinated_parts("<title>x");
    assert!(freeze_fixture(&eof_fixture, coordinated_parts("<title>x").parts).is_ok());
    assert!(
        eof_fixture
            .parts
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code() == HtmlTreeDiagnosticCode::TitleEndOfFileInText)
    );

    assert!(freeze_fixture(&fixture, coordinated_parts("<title>x</title>").parts).is_ok());
}

/// Falsifies: one shared lexical episode standing in for both selected
/// tokenizer lifecycles.
///
/// RAWTEXT and RCDATA are separate lexical episodes with separate retained
/// start tags and separate appropriate-close markers. Neither control may
/// open the other's episode, and neither may open a second episode while one
/// is already running.
#[test]
fn pf37_the_two_lexical_episodes_are_separately_owned() {
    // The RAWTEXT control cannot open a suspended Title, and vice versa.
    let title = SourceText::new(SourceId::new(1), "<title>abc</title>".to_owned());
    let mut tokenizer = HtmlTokenizerSession::new(&title, limits());
    tokenizer.drive_to_boundary();
    assert!(tokenizer.apply_raw_text().is_err());
    // The refusal left the RCDATA suspension intact and opened nothing.
    tokenizer.apply_title_rcdata().expect("selected activation");

    // With an episode already running, neither control may open another.
    assert!(tokenizer.apply_raw_text().is_err());
    assert!(tokenizer.apply_title_rcdata().is_err());

    // An appropriate close belongs to the episode that opened it: a
    // `</style>` inside a Title episode is text, never a close, and the
    // authored `</title>` still closes it.
    let mixed = analyze("<title>a</style>b</title>");
    assert_eq!(title_text(&mixed), "a</style>b");
    assert!(mixed.actions().iter().any(|action| matches!(
        action.kind(),
        HtmlTreeActionKind::ClosedTitleElementByAuthoredEndTag { .. }
    )));

    // And symmetrically, a `</title>` inside a Style episode is text.
    let style = analyze("<style>a</title>b</style>");
    let style_id = style
        .nodes_in_creation_order()
        .into_iter()
        .find_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(HtmlElement::Style(_)) => Some(node.id()),
            _ => None,
        })
        .expect("Style node");
    let style_text: String = style
        .node(style_id)
        .expect("Style node")
        .children()
        .iter()
        .map(|child| match style.node(*child).expect("child").kind() {
            HtmlTreeNodeKind::Text(text) => text.interpreted().to_owned(),
            other => panic!("a Style child must be text, got {other:?}"),
        })
        .collect();
    assert_eq!(style_text, "a</title>b");
}

/// Falsifies: TC-S10 disturbing the accepted TC-S9 Style lifecycle.
#[test]
fn pf32_the_tc_s9_style_lifecycle_is_unchanged() {
    let analysis = analyze("<style>a<b</style>");
    let names: Vec<HtmlElementName> = analysis
        .nodes_in_creation_order()
        .iter()
        .filter_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(element) => Some(element.name()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&HtmlElementName::Style));
    assert!(!names.contains(&HtmlElementName::Title));
    assert!(analysis.tokenizer_run().diagnostics().is_empty());
    assert!(analysis.actions().iter().any(|action| matches!(
        action.kind(),
        HtmlTreeActionKind::ClosedStyleElementByAuthoredEndTag { .. }
    )));
}

/// Falsifies: the two selected Text-mode domains merging into one.
#[test]
fn pf33_style_and_title_remain_separate_element_domains() {
    let title = analyze("<title>x</title>");
    assert!(title.nodes_in_creation_order().iter().all(|node| !matches!(
        node.kind(),
        HtmlTreeNodeKind::Element(HtmlElement::Style(_))
    )));
    let style = analyze("<style>x</style>");
    assert!(style.nodes_in_creation_order().iter().all(|node| !matches!(
        node.kind(),
        HtmlTreeNodeKind::Element(HtmlElement::Title(_))
    )));

    // Two selected Text elements can never be open at once, so a second
    // selected start tag inside an open episode is not admissible text.
    let nested = analyze("<title>a<style>b</style></title>");
    // `<style>` inside RCDATA is ordinary text, never a second episode.
    assert_eq!(title_text(&nested), "a<style>b</style>");
}

/// Falsifies: a hostile resource configuration reaching an ordinary Core
/// panic, or assembling a frozen result that fails its own contract.
///
/// No `catch_unwind`: a production panic on any combination fails naturally,
/// and both the tokenizer run assembly and the tree freeze validate
/// themselves. This is the pressure that found the reference emission's
/// refusal shape, so it stays permanent.
#[test]
fn pf35_hostile_limits_never_panic_and_always_assemble_a_valid_result() {
    let texts = [
        "<title>&notin;</title>",
        "<title>&notit;</title>",
        "<title>abc&acE;def</title>",
        "<title>&nope;x</title>",
        "<title>&amp b</title>",
        "<title>&CounterClockwiseContourIntegral;</title>",
        "<title>&#38;</title>",
        "<title>a\u{0}b</title>",
        "<title>&not\u{1}z</title>",
        "<title>abc",
        "<title>&amp",
        "<title>&lt;/title></title>",
        "<textarea>&amp;</textarea>",
        "<style>&amp;</style>",
    ];
    for text in texts {
        for tokens in [0usize, 1, 2, 3, 5, 64] {
            for diagnostics in [0usize, 1, 64] {
                for retained in [0usize, 1, 8, 12, 13, 4_096] {
                    for steps in [0usize, 1, 10, 12, 14, 4_096] {
                        let limits = HtmlTokenizerLimits::new(
                            4_096,
                            steps,
                            tokens,
                            diagnostics,
                            256,
                            retained,
                            4_096,
                        );
                        let source = SourceText::new(SourceId::new(1), text.to_owned());
                        construct_html_document_shell(&source, limits)
                            .expect("a hostile configuration still reaches a valid boundary");
                        let _ = tokenize(&source, limits);
                    }
                }
            }
        }
    }
}

/// Falsifies: TC-S10 changing predecessor shell construction.
#[test]
fn pf34_predecessor_shell_construction_is_unchanged() {
    let analysis = analyze("<html><head></head><body><p>x</p></body></html>");
    assert!(tree_unsupported(&analysis).is_none());
    assert!(
        analysis
            .nodes_in_creation_order()
            .iter()
            .all(|node| !matches!(
                node.kind(),
                HtmlTreeNodeKind::Element(HtmlElement::Title(_))
            ))
    );
}
