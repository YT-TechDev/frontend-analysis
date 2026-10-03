//! Production correspondence for the Data-state Named Character Reference
//! successor (Issue #876).
//!
//! Expectations here are authored from the accepted theorem (WHATWG named
//! character reference semantics as scoped by #348 / #874 / #876), never from
//! production output. The tests import only production tokenizer,
//! tree-construction and durable-result seams. They do not import, call, or
//! copy the candidate-independent validation machine in the sibling
//! `data_state_named_reference_successor_validation` module.
//!
//! Each test names the incorrect implementation it rejects.

use crate::{SourceId, SourceText};

use super::super::token::HtmlToken;
use super::super::tokenizer::diagnostic::HtmlTokenizerDiagnosticCode as Diag;
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerResource};
use super::super::tokenizer::result::{
    HtmlCharacterReferenceContext, HtmlTokenizerCapability, HtmlTokenizerCapabilityAvailability,
    HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause, HtmlTokenizerRunResult,
    HtmlTokenizerUnsupportedTrigger,
};
use super::driver::construct_html_document_shell;
use super::result::{
    HtmlDocumentShellAnalysis, HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeIncompleteCause,
    HtmlTreeNodeKind,
};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 4_096, 256, 16_384, 4_096)
}

fn lex_with(text: &str, limits: HtmlTokenizerLimits) -> HtmlTokenizerRunResult {
    let source = SourceText::new(SourceId::new(1), text.to_owned());
    tokenize(&source, limits)
}

fn lex(text: &str) -> HtmlTokenizerRunResult {
    lex_with(text, limits())
}

fn analyze_with(text: &str, limits: HtmlTokenizerLimits) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(1), text.to_owned());
    construct_html_document_shell(&source, limits).expect("Data Named production boundary")
}

fn analyze(text: &str) -> HtmlDocumentShellAnalysis {
    analyze_with(text, limits())
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

fn tag_count(run: &HtmlTokenizerRunResult) -> usize {
    run.tokens()
        .iter()
        .filter(|token| matches!(token, HtmlToken::Tag(_)))
        .count()
}

fn diags(run: &HtmlTokenizerRunResult) -> Vec<(Diag, (usize, usize))> {
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

fn expect_chars(text: &str, expected: &[((usize, usize), &str)]) -> HtmlTokenizerRunResult {
    let run = lex(text);
    assert!(
        matches!(run.completion(), HtmlTokenizerCompletion::Complete),
        "{text:?} completes"
    );
    let expected: Vec<_> = expected
        .iter()
        .map(|(range, interpreted)| (*range, (*interpreted).to_owned()))
        .collect();
    assert_eq!(chars(&run), expected, "{text:?}");
    run
}

type ObservedUnsupported = (
    HtmlTokenizerCapability,
    HtmlTokenizerCapabilityAvailability,
    Option<(usize, usize)>,
);

fn unsupported(run: &HtmlTokenizerRunResult) -> Option<ObservedUnsupported> {
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
    Some((
        unsupported.capability(),
        unsupported.availability(),
        trigger,
    ))
}

fn resource_stop(run: &HtmlTokenizerRunResult) -> Option<HtmlTokenizerResource> {
    match run.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(limit)) => {
            Some(limit.resource())
        }
        _ => None,
    }
}

/// Every ordered `(authored range, interpreted)` text contribution in the
/// frozen tree, in creation order.
fn contributions(analysis: &HtmlDocumentShellAnalysis) -> Vec<((usize, usize), String)> {
    let mut out = Vec::new();
    for node in analysis.nodes_in_creation_order() {
        if let HtmlTreeNodeKind::Text(text) = node.kind() {
            for contribution in text.contributions() {
                out.push((
                    (
                        contribution.source().range().start(),
                        contribution.source().range().end(),
                    ),
                    contribution.interpreted().to_owned(),
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

fn tree_unsupported(analysis: &HtmlDocumentShellAnalysis) -> Option<HtmlTreeCapability> {
    match analysis.completion() {
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => Some(unsupported.capability()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Lexical families
// ---------------------------------------------------------------------------

/// Falsifies: a Data `&` that still stops as the broad Deferred capability,
/// and a bare or `;`-adjacent `&` that is swallowed or reinterpreted.
#[test]
fn ds1_a_bare_ampersand_and_ampersand_semicolon_are_ordinary_text() {
    let run = expect_chars("&", &[((0, 1), "&")]);
    assert!(diags(&run).is_empty());
    let run = expect_chars("&;", &[((0, 2), "&;")]);
    assert!(diags(&run).is_empty());
}

/// Falsifies: a diagnostic fabricated at end of input without an authored
/// `;`, and an unresolved candidate that loses its authored text.
#[test]
fn ds2_unresolved_candidates_keep_authored_text_and_condition_the_diagnostic_on_semicolon() {
    let run = expect_chars("&x", &[((0, 2), "&x")]);
    assert!(diags(&run).is_empty());

    let run = expect_chars("&x;", &[((0, 2), "&x"), ((2, 3), ";")]);
    assert_eq!(
        diags(&run),
        vec![(Diag::UnknownNamedCharacterReference, (2, 3))]
    );

    let run = expect_chars("&bogus;", &[((0, 6), "&bogus"), ((6, 7), ";")]);
    assert_eq!(
        diags(&run),
        vec![(Diag::UnknownNamedCharacterReference, (6, 7))]
    );

    let run = expect_chars("&bogus", &[((0, 6), "&bogus")]);
    assert!(diags(&run).is_empty());
}

/// Falsifies: exact-only matching, a lost missing-semicolon diagnostic, and
/// the AttributeValue historical exception leaking into Data.
#[test]
fn ds3_semicolon_and_semicolonless_named_references_resolve_by_data_rules() {
    let run = expect_chars("&amp;", &[((0, 5), "&")]);
    assert!(diags(&run).is_empty());

    let run = expect_chars("&amp", &[((0, 4), "&")]);
    assert_eq!(
        diags(&run),
        vec![(Diag::MissingSemicolonAfterCharacterReference, (3, 4))]
    );

    // In Data a semicolonless reference followed by `=` still resolves; the
    // AttributeValue exception (which would keep `&amp=` literal) must not
    // apply.
    let run = expect_chars("&amp=", &[((0, 4), "&"), ((4, 5), "=")]);
    assert_eq!(
        diags(&run),
        vec![(Diag::MissingSemicolonAfterCharacterReference, (3, 4))]
    );

    // Maximum semicolonless prefix: `amp`, then the literal `x;`.
    let run = expect_chars("&ampx;", &[((0, 4), "&"), ((4, 6), "x;")]);
    assert_eq!(
        diags(&run),
        vec![(Diag::MissingSemicolonAfterCharacterReference, (3, 4))]
    );
}

/// Falsifies: a non-maximal match (`&not` + `it;` versus `&notin;`).
#[test]
fn ds4_maximum_match_is_preserved() {
    let run = expect_chars("&notit;", &[((0, 4), "\u{ac}"), ((4, 7), "it;")]);
    assert_eq!(
        diags(&run),
        vec![(Diag::MissingSemicolonAfterCharacterReference, (3, 4))]
    );
    let run = expect_chars("&notin;", &[((0, 7), "\u{2209}")]);
    assert!(diags(&run).is_empty());
}

/// Falsifies: fabricated source subdivision of one multi-scalar reference.
#[test]
fn ds5_one_authored_reference_is_one_origin_with_multi_scalar_output() {
    let run = expect_chars("&NotEqualTilde;", &[((0, 15), "\u{2242}\u{338}")]);
    assert!(diags(&run).is_empty());
}

/// Falsifies: recursive decoding and decoded markup re-entering the tokenizer.
#[test]
fn ds6_decoded_output_is_never_authored_syntax() {
    let run = expect_chars("&amp;lt;", &[((0, 5), "&"), ((5, 8), "lt;")]);
    assert_eq!(tag_count(&run), 0);

    let run = expect_chars("&lt;/body>", &[((0, 4), "<"), ((4, 10), "/body>")]);
    assert_eq!(tag_count(&run), 0, "decoded `</body>` is not an end tag");

    // The same bytes authored literally are an end tag; the decoded form is
    // distinguishable from it.
    assert_eq!(tag_count(&lex("</body>")), 1);
}

/// Falsifies: the Named branch swallowing `#`, the retired Numeric-in-Data
/// refusal returning, and the broad Deferred Data capability. Numeric
/// references are supported by #880; the Named matcher is not consulted.
#[test]
fn ds7_numeric_in_data_is_supported_and_never_a_named_lookup() {
    let run = expect_chars("&#65;", &[((0, 5), "A")]);
    assert!(unsupported(&run).is_none());
    assert!(diags(&run).is_empty());

    // `amp;` after a Numeric reference is ordinary text, not a Named lookup.
    expect_chars("&#65amp;", &[((0, 4), "A"), ((4, 8), "amp;")]);
}

/// Falsifies: lookahead that consumes or diagnoses scalars it only peeks at.
/// (U+0001, not Data U+0000, is the selected challenge.)
#[test]
fn ds8_lookahead_is_non_committing_over_a_control_scalar() {
    let run = lex("&not\u{1}it;");
    assert_eq!(chars(&run)[0], ((0, 4), "\u{ac}".to_owned()));
    // The control scalar is diagnosed exactly once, at its own authored
    // location, after the missing-semicolon diagnostic.
    assert_eq!(
        diags(&run),
        vec![
            (Diag::MissingSemicolonAfterCharacterReference, (3, 4)),
            (Diag::ControlCharacterInInputStream, (4, 5)),
        ]
    );
    assert_eq!(
        chars(&run)
            .into_iter()
            .skip(1)
            .map(|(_, text)| text)
            .collect::<String>(),
        "\u{1}it;"
    );
}

// ---------------------------------------------------------------------------
// Data / RCDATA return-state ownership
// ---------------------------------------------------------------------------

/// Falsifies: Data resolution returning to RCDATA (the `<b>` would be text)
/// and RCDATA resolution returning to Data (the `<b>` would become a tag).
#[test]
fn ds9_data_and_rcdata_return_to_their_own_states() {
    // Data: after the reference `<b>` is markup again.
    let data = lex("&amp;<b>");
    assert_eq!(tag_count(&data), 1);

    // RCDATA: after the reference `<b>` stays text until the appropriate end.
    let analysis = analyze("<title>&amp;<b></title>");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["&<b>".to_owned()]);

    // Unresolved candidates also close into their own return state.
    assert_eq!(tag_count(&lex("&bogus<b>")), 1);
    let analysis = analyze("<title>&bogus<b></title>");
    assert_eq!(texts(&analysis), vec!["&bogus<b>".to_owned()]);
}

/// Falsifies: the retired Numeric-in-Data/RCDATA refusals returning, and a
/// return state collapsed so Numeric output lands in the wrong context.
#[test]
fn ds10_numeric_returns_to_its_own_state_in_both_contexts() {
    let title = analyze("<title>&#65;</title>");
    assert!(title.is_complete());
    assert!(unsupported(title.tokenizer_run()).is_none());
    assert_eq!(texts(&title), vec!["A".to_owned()]);

    let body = analyze("<body>&#65;");
    assert!(body.is_complete());
    assert!(unsupported(body.tokenizer_run()).is_none());
    assert_eq!(texts(&body), vec!["A".to_owned()]);
}

/// Falsifies: AttributeValue support or its historical exception arriving
/// through the shared states.
#[test]
fn ds11_attribute_value_character_references_remain_deferred() {
    for text in ["<p id=\"&amp;\">", "<a x=&x>"] {
        let run = lex(text);
        let observed = unsupported(&run).unwrap_or_else(|| panic!("{text:?} unsupported"));
        assert_eq!(
            observed.0,
            HtmlTokenizerCapability::CharacterReference {
                context: HtmlCharacterReferenceContext::AttributeValue,
            }
        );
        assert_eq!(observed.1, HtmlTokenizerCapabilityAvailability::Deferred);
    }
}

/// Falsifies: the Data NUL contradiction being silently altered here. The
/// existing separate defect is unchanged: Data NUL still takes the
/// pre-existing recovery path, and a NUL after a reference behaves the same.
#[test]
fn ds12_data_nul_behavior_is_unchanged() {
    let plain = lex("a\0b");
    let after_reference = lex("&amp;\0b");
    assert_eq!(
        chars(&plain)
            .into_iter()
            .map(|(_, text)| text)
            .collect::<String>(),
        "a\u{fffd}b"
    );
    assert_eq!(
        chars(&after_reference)
            .into_iter()
            .map(|(_, text)| text)
            .collect::<String>(),
        "&\u{fffd}b"
    );
    assert_eq!(diags(&plain).len(), diags(&after_reference).len());
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Falsifies: a matcher that retains a copied candidate buffer.
#[test]
fn ds13_temporary_buffer_bytes_stay_zero() {
    for text in [
        "&CounterClockwiseContourIntegral;",
        "&notit;&notin;&acE;&nope;&amp x",
        "&nomatchatallhereatall",
        "&#65;",
    ] {
        let run = lex(text);
        assert_eq!(run.usage().peak_temporary_buffer_bytes(), 0, "{text:?}");
    }
}

/// Falsifies: a reference committed past its retained-byte preflight.
#[test]
fn ds14_a_retained_byte_refusal_commits_no_part_of_the_reference() {
    // `abc` (3) + `&acE;` decodes to 5 more > 6.
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 4_096, 256, 6, 4_096);
    let run = lex_with("abc&acE;", constrained);
    assert_eq!(
        resource_stop(&run),
        Some(HtmlTokenizerResource::RetainedInterpretedBytes)
    );
    assert_eq!(chars(&run), vec![((0, 3), "abc".to_owned())]);
    assert_eq!(run.coverage().processed_end(), 4);
    assert!(diags(&run).is_empty());
}

/// Falsifies: a reference emitted past its token-capacity preflight.
#[test]
fn ds15_an_emitted_token_refusal_commits_no_part_of_the_reference() {
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 1, 4_096, 256, 16_384, 4_096);
    let run = lex_with("abc&acE;", constrained);
    assert_eq!(
        resource_stop(&run),
        Some(HtmlTokenizerResource::EmittedTokens)
    );
    assert_eq!(chars(&run), vec![((0, 3), "abc".to_owned())]);
    assert_eq!(run.coverage().processed_end(), 4);
}

/// Falsifies: a resolved reference committed without capacity for its
/// required missing-semicolon diagnostic.
#[test]
fn ds16_a_diagnostic_refusal_commits_no_part_of_the_reference() {
    let constrained = HtmlTokenizerLimits::new(4_096, 32_768, 4_096, 0, 256, 16_384, 4_096);
    let run = lex_with("&not", constrained);
    assert_eq!(
        resource_stop(&run),
        Some(HtmlTokenizerResource::Diagnostics)
    );
    assert!(chars(&run).is_empty());
    assert!(diags(&run).is_empty());
    assert_eq!(run.coverage().processed_end(), 1);

    // A match needing no diagnostic is unaffected by the same zero budget.
    let clean = lex_with("&amp;", constrained);
    assert_eq!(chars(&clean), vec![((0, 5), "&".to_owned())]);
}

/// Falsifies: source consumed before the transaction is ready. Whatever
/// transition-step budget is chosen, committed coverage never lands strictly
/// inside a matched identifier.
#[test]
fn ds17_coverage_never_lands_inside_a_matched_identifier() {
    let text = "abc&notin;d";
    for steps in 0..40 {
        let constrained = HtmlTokenizerLimits::new(4_096, steps, 4_096, 4_096, 256, 16_384, 4_096);
        let run = lex_with(text, constrained);
        let end = run.coverage().processed_end();
        assert!(
            !(5..10).contains(&end),
            "steps {steps}: coverage {end} is inside the matched identifier"
        );
        if resource_stop(&run).is_some() && end < 10 {
            assert!(
                chars(&run)
                    .iter()
                    .all(|(_, text)| !text.contains('\u{2209}')),
                "steps {steps}: a refused reference left partial output"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Product / tree
// ---------------------------------------------------------------------------

/// Falsifies: lost authored/interpreted separation in the tree.
#[test]
fn dt1_body_text_keeps_ordered_authored_contributions() {
    let analysis = analyze("<body>a&amp;b</body>");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["a&b".to_owned()]);
    assert_eq!(
        contributions(&analysis),
        vec![
            ((6, 7), "a".to_owned()),
            ((7, 12), "&".to_owned()),
            ((12, 13), "b".to_owned()),
        ]
    );
}

/// Falsifies: source subdivision of a multi-scalar reference in the tree.
#[test]
fn dt2_multi_scalar_reference_is_one_tree_contribution() {
    let analysis = analyze("<body>&NotEqualTilde;</body>");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["\u{2242}\u{338}".to_owned()]);
    assert_eq!(
        contributions(&analysis),
        vec![((6, 21), "\u{2242}\u{338}".to_owned())]
    );
}

/// Falsifies: tree whitespace decisions taken from authored spelling rather
/// than interpreted values: `&Tab;` / `&NewLine;` are whitespace and
/// `&nbsp;` is not, so each must behave exactly like the same value authored
/// literally.
#[test]
fn dt3_tree_decisions_use_interpreted_values_not_authored_entity_spelling() {
    let same = |entity: &str, literal: &str| {
        let by_entity = analyze(entity);
        let by_literal = analyze(literal);
        assert_eq!(
            by_entity.is_complete(),
            by_literal.is_complete(),
            "{entity:?} vs {literal:?}: completion"
        );
        assert_eq!(
            tree_unsupported(&by_entity),
            tree_unsupported(&by_literal),
            "{entity:?} vs {literal:?}: tree capability"
        );
        assert_eq!(
            texts(&by_entity),
            texts(&by_literal),
            "{entity:?} vs {literal:?}: text"
        );
        assert_eq!(
            by_entity.node_count(),
            by_literal.node_count(),
            "{entity:?} vs {literal:?}: identities"
        );
    };
    same("<body>&Tab;</body>", "<body>\t</body>");
    same("<body></body>&NewLine;", "<body></body>\n");
    same("<body></body>&nbsp;", "<body></body>\u{a0}");

    let tab = analyze("<body>&Tab;</body>");
    assert!(tab.is_complete());
    assert_eq!(texts(&tab), vec!["\t".to_owned()]);
}

/// Falsifies: early whitespace-sensitive tree positions widening.
#[test]
fn dt4_early_unsupported_tree_positions_remain_unsupported() {
    let same = |entity: &str, literal: &str| {
        let by_entity = analyze(entity);
        let by_literal = analyze(literal);
        assert_eq!(
            by_entity.is_complete(),
            by_literal.is_complete(),
            "{entity:?}"
        );
        assert_eq!(
            tree_unsupported(&by_entity),
            tree_unsupported(&by_literal),
            "{entity:?}"
        );
    };
    same("&NotEqualTilde;", "\u{2242}\u{338}");
    same("&nbsp;<body>", "\u{a0}<body>");
    same("&Tab;<body>", "\t<body>");
}
