//! Production correspondence for the selected canonical HTML DOCTYPE /
//! Initial No-Quirks successor (Issue #892).
//!
//! Expectations here are authored by hand from the accepted theorem (#348,
//! #890 and #892): byte offsets, token indexes, step counts, identities and
//! resource attempts are written out literally from the authored fixtures. They
//! are never captured from production output, and this module neither imports
//! nor calls the candidate-independent model in the sibling
//! `canonical_doctype_successor_validation` module, which stays the
//! independent oracle.
//!
//! The selected profile is exactly `<!` + ASCII-CI `DOCTYPE` + ASCII
//! whitespace+ + ASCII-CI `html` + ASCII whitespace* + `>`. Everything outside
//! it is asserted here only at the boundary this Issue owns: the unchanged
//! `MarkupDeclaration` tokenizer stop.
//!
//! # Hand-counted transition steps
//!
//! The tokenizer charges one transition per dispatched input unit. A source
//! that is only a selected DOCTYPE costs one step per input unit (a CRLF pair
//! is one unit), plus one for the reconsumed first name letter, plus one for
//! the end-of-file unit.

use crate::{SourceId, SourceText};

use super::super::token::HtmlToken;
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerResource};
use super::super::tokenizer::result::{
    HtmlTokenizerCapability, HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause,
    HtmlTokenizerRunResult, HtmlTokenizerUnsupportedTrigger,
};
use super::driver::{construct_html_document_shell, drive_token};
use super::result::{
    HtmlConstructedIdentityCounter, HtmlConstructedNodeId, HtmlDocumentShellAnalysis,
    HtmlDocumentShellParts, HtmlDocumentType, HtmlElement, HtmlShellElement, HtmlShellElementName,
    HtmlShellElementOrigin, HtmlTextContribution, HtmlTextNode, HtmlTreeAction, HtmlTreeActionKind,
    HtmlTreeCapability, HtmlTreeCompletion, HtmlTreeDiagnostic, HtmlTreeDiagnosticCode,
    HtmlTreeFreezeError, HtmlTreeIncompleteCause, HtmlTreeNode, HtmlTreeNodeKind, HtmlTreeRecovery,
    HtmlTreeTokenTrigger, freeze,
};
use super::session::{
    HtmlDocumentMode, HtmlTreeSession, InsertionMode, TokenOutcome, admit, token_trigger,
};

type Range = (usize, usize);

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn source_with(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

fn source(text: &str) -> SourceText {
    source_with(1, text)
}

fn run(text: &str) -> HtmlTokenizerRunResult {
    tokenize(&source(text), limits())
}

fn run_with(text: &str, limits: HtmlTokenizerLimits) -> HtmlTokenizerRunResult {
    tokenize(&source(text), limits)
}

fn analyze(text: &str) -> HtmlDocumentShellAnalysis {
    construct_html_document_shell(&source(text), limits()).expect("no boundary failure")
}

fn analyze_with(text: &str, limits: HtmlTokenizerLimits) -> HtmlDocumentShellAnalysis {
    construct_html_document_shell(&source(text), limits).expect("no boundary failure")
}

fn range_of(anchor: &crate::SourceAnchor) -> Range {
    (anchor.range().start(), anchor.range().end())
}

/// The tokens' kinds and complete ranges, hand-compared below.
fn shape(run: &HtmlTokenizerRunResult) -> Vec<(&'static str, Range)> {
    run.tokens()
        .iter()
        .map(|token| match token {
            HtmlToken::Character(c) => ("character", range_of(c.source())),
            HtmlToken::Doctype(d) => ("doctype", range_of(d.complete())),
            HtmlToken::Tag(t) => ("tag", range_of(t.complete())),
            HtmlToken::EndOfFile(e) => ("eof", range_of(e.source())),
        })
        .collect()
}

fn doctype(run: &HtmlTokenizerRunResult, index: usize) -> &super::super::token::HtmlDoctypeToken {
    let HtmlToken::Doctype(doctype) = &run.tokens()[index] else {
        panic!(
            "token {index} is not a Doctype token: {:?}",
            run.tokens()[index]
        )
    };
    doctype
}

fn unsupported_markup_declaration(run: &HtmlTokenizerRunResult) -> Range {
    let HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
        unsupported,
    )) = run.completion()
    else {
        panic!(
            "expected a tokenizer unsupported capability: {:?}",
            run.completion()
        )
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTokenizerCapability::MarkupDeclaration
    );
    let HtmlTokenizerUnsupportedTrigger::Input(anchor) = unsupported.trigger() else {
        panic!("MarkupDeclaration is discovered from input")
    };
    assert_eq!(
        anchor.range().start(),
        run.coverage().processed_end(),
        "an input trigger starts at the coverage boundary"
    );
    range_of(anchor)
}

// ---------------------------------------------------------------------------
// Tokenizer: accepted family
// ---------------------------------------------------------------------------

struct Accepted {
    id: &'static str,
    text: &'static str,
    /// Exact authored DOCTYPE range, hand-counted.
    complete: Range,
    /// Exact authored name range and spelling, hand-counted.
    name: Range,
    spelling: &'static str,
    /// Hand-counted input units (a CRLF pair is one unit).
    units: usize,
}

/// Every source is only the DOCTYPE, so token 1 is the end-of-file token at
/// the end of the source.
const ACCEPTED: &[Accepted] = &[
    // `<!DOCTYPE ` is 10 bytes, `html` 4, `>` 1.
    Accepted {
        id: "A01",
        text: "<!DOCTYPE html>",
        complete: (0, 15),
        name: (10, 14),
        spelling: "html",
        units: 15,
    },
    Accepted {
        id: "A02",
        text: "<!doctype html>",
        complete: (0, 15),
        name: (10, 14),
        spelling: "html",
        units: 15,
    },
    Accepted {
        id: "A03",
        text: "<!DoCtYpE HTML>",
        complete: (0, 15),
        name: (10, 14),
        spelling: "HTML",
        units: 15,
    },
    Accepted {
        id: "A04",
        text: "<!DOCTYPE\thtml>",
        complete: (0, 15),
        name: (10, 14),
        spelling: "html",
        units: 15,
    },
    Accepted {
        id: "A05",
        text: "<!DOCTYPE\nhtml>",
        complete: (0, 15),
        name: (10, 14),
        spelling: "html",
        units: 15,
    },
    Accepted {
        id: "A06",
        text: "<!DOCTYPE\u{c}html>",
        complete: (0, 15),
        name: (10, 14),
        spelling: "html",
        units: 15,
    },
    Accepted {
        id: "A07",
        text: "<!DOCTYPE\rhtml>",
        complete: (0, 15),
        name: (10, 14),
        spelling: "html",
        units: 15,
    },
    Accepted {
        id: "A08",
        text: "<!DOCTYPE hTmL>",
        complete: (0, 15),
        name: (10, 14),
        spelling: "hTmL",
        units: 15,
    },
    // One trailing space/tab before `>`: complete is 16 bytes.
    Accepted {
        id: "A09",
        text: "<!DOCTYPE html >",
        complete: (0, 16),
        name: (10, 14),
        spelling: "html",
        units: 16,
    },
    Accepted {
        id: "A10",
        text: "<!DOCTYPE html\t>",
        complete: (0, 16),
        name: (10, 14),
        spelling: "html",
        units: 16,
    },
    // `<!DOCTYPE` 9, three spaces to 12, `html` 12..16, three spaces to 19.
    Accepted {
        id: "A11",
        text: "<!DOCTYPE   html   >",
        complete: (0, 20),
        name: (12, 16),
        spelling: "html",
        units: 20,
    },
    Accepted {
        id: "A12",
        text: "<!DOCTYPE     html>",
        complete: (0, 19),
        name: (14, 18),
        spelling: "html",
        units: 19,
    },
    // CRLF is two raw bytes but one interpreted unit; ranges stay raw.
    // `<!DOCTYPE` 9, CRLF 9..11, `html` 11..15, `>` 15..16.
    Accepted {
        id: "A13",
        text: "<!DOCTYPE\r\nhtml>",
        complete: (0, 16),
        name: (11, 15),
        spelling: "html",
        units: 15,
    },
    // `html` 10..14, CRLF 14..16, `>` 16..17.
    Accepted {
        id: "A14",
        text: "<!DOCTYPE html\r\n>",
        complete: (0, 17),
        name: (10, 14),
        spelling: "html",
        units: 16,
    },
    // `<!DOCTYPE` 9, ` \r\n\t` = space 9..10, CRLF 10..12, tab 12..13, `html` 13..17, `>`.
    Accepted {
        id: "A15",
        text: "<!DOCTYPE \r\n\thtml>",
        complete: (0, 18),
        name: (13, 17),
        spelling: "html",
        units: 17,
    },
];

#[test]
fn t1_the_accepted_family_emits_one_distinct_doctype_token_with_exact_evidence() {
    for case in ACCEPTED {
        let run = run(case.text);
        let len = case.text.len();
        assert!(!run.is_incomplete(), "{}", case.id);
        assert_eq!(
            shape(&run),
            vec![("doctype", case.complete), ("eof", (len, len))],
            "{}",
            case.id
        );
        let token = doctype(&run, 0);
        assert_eq!(range_of(token.complete()), case.complete, "{}", case.id);
        assert_eq!(token.complete().fragment(), case.text, "{}", case.id);
        assert_eq!(range_of(token.name().source()), case.name, "{}", case.id);
        // Authored spelling is kept apart from the interpreted selected name.
        assert_eq!(
            token.name().source().fragment(),
            case.spelling,
            "{}",
            case.id
        );
        assert_eq!(token.name().interpreted(), "html", "{}", case.id);
        assert!(run.diagnostics().is_empty(), "{}", case.id);
    }
}

#[test]
fn t2_the_accepted_family_costs_exactly_the_hand_counted_resources() {
    for case in ACCEPTED {
        let run = run(case.text);
        let usage = run.usage();
        assert_eq!(usage.source_bytes(), case.text.len(), "{}", case.id);
        // One transition per unit, plus the reconsumed `h`, plus EOF.
        assert_eq!(usage.transition_steps(), case.units + 2, "{}", case.id);
        // The DOCTYPE token and the end-of-file token.
        assert_eq!(usage.emitted_tokens(), 2, "{}", case.id);
        assert_eq!(usage.diagnostics(), 0, "{}", case.id);
        assert_eq!(usage.peak_attributes_per_tag(), 0, "{}", case.id);
        // Exactly the interpreted selected name `html`.
        assert_eq!(usage.retained_interpreted_bytes(), 4, "{}", case.id);
        assert_eq!(usage.peak_temporary_buffer_bytes(), 0, "{}", case.id);
        assert_eq!(
            run.coverage().processed_end(),
            case.text.len(),
            "{}",
            case.id
        );
    }
}

#[test]
fn t3_selected_success_needs_no_diagnostic_or_temporary_buffer_capacity() {
    // Diagnostics and TemporaryBufferBytes limits of zero are enough.
    let zero = HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 0, 256, 4_096, 0);
    for case in ACCEPTED {
        let run = run_with(case.text, zero);
        assert!(!run.is_incomplete(), "{}", case.id);
        assert_eq!(shape(&run).len(), 2, "{}", case.id);
    }
    // Exactly four retained bytes and two emitted tokens are also enough.
    let exact = HtmlTokenizerLimits::new(1_024, 17, 2, 0, 0, 4, 0);
    let run = run_with("<!DOCTYPE html>", exact);
    assert!(!run.is_incomplete());
}

#[test]
fn t4_a_doctype_followed_by_shell_tags_tokenizes_each_range_exactly() {
    // `<html>` 15..21, `</html>` 21..28.
    let run = run("<!DOCTYPE html><html></html>");
    assert_eq!(
        shape(&run),
        vec![
            ("doctype", (0, 15)),
            ("tag", (15, 21)),
            ("tag", (21, 28)),
            ("eof", (28, 28)),
        ]
    );
    // `<body>` 15..21, `</body>` 21..28, `</html>` 28..35.
    let run = self::run("<!DOCTYPE html><body></body></html>");
    assert_eq!(
        shape(&run),
        vec![
            ("doctype", (0, 15)),
            ("tag", (15, 21)),
            ("tag", (21, 28)),
            ("tag", (28, 35)),
            ("eof", (35, 35)),
        ]
    );
    assert!(!run.is_incomplete());
}

#[test]
fn t5_a_doctype_after_other_tokens_is_still_one_exact_doctype_token() {
    // Tokenizer meaning is independent of tree position. `<html>` 0..6.
    let run = run("<html><!DOCTYPE html>");
    assert_eq!(
        shape(&run),
        vec![("tag", (0, 6)), ("doctype", (6, 21)), ("eof", (21, 21)),]
    );
    let token = doctype(&run, 1);
    assert_eq!(range_of(token.name().source()), (16, 20));
}

#[test]
fn t6_crlf_and_cr_ranges_stay_raw_while_coverage_reaches_the_authored_end() {
    for (text, complete) in [
        ("<!DOCTYPE\r\nhtml>", (0, 16)),
        ("<!DOCTYPE\rhtml>", (0, 15)),
        ("<!DOCTYPE html\r\n>", (0, 17)),
    ] {
        let run = run(text);
        let token = doctype(&run, 0);
        assert_eq!(range_of(token.complete()), complete, "{text:?}");
        // The caller's text is never rewritten.
        assert_eq!(token.complete().fragment(), text, "{text:?}");
        assert_eq!(run.coverage().processed_end(), text.len(), "{text:?}");
    }
}

// ---------------------------------------------------------------------------
// Tokenizer: outside the selected profile, at the boundary this Issue owns
// ---------------------------------------------------------------------------

#[test]
fn t7_non_selected_markup_declarations_keep_the_markup_declaration_stop() {
    // Each stops at the unchanged boundary: no token, coverage rolled back to
    // the opening `<`, trigger is exactly `<!` (0..2).
    for text in [
        "<!DOCTYPE>",
        "<!DOCTYPE svg>",
        "<!DOCTYPE html PUBLIC \"x\">",
        "<!DOCTYPE html SYSTEM \"x\">",
        "<!DOCTYPE html PUBLIC \"\">",
        "<!DOCTYPE html SYSTEM \"\">",
        "<!DOCTYPE html foo>",
        "<!DOCTYPE htmlx>",
        "<!DOCTYPEhtml>",
        "<!DOCTYPE",
        "<!DOCTYPE ",
        "<!DOCTYPE html",
        "<!DOCTYPE html ",
        "<!DOCTYPE h",
        "<!DOCTYPE ht>",
        "<!doctype html x>",
        "<!DOCTYPE\u{a0}html>",
        "<!DOCTYP html>",
        "<!-- c -->",
        "<![CDATA[x]]>",
        "<!xx>",
        "<!",
    ] {
        let run = run(text);
        assert!(run.tokens().is_empty(), "{text:?}: no token may commit");
        assert_eq!(unsupported_markup_declaration(&run), (0, 2), "{text:?}");
        assert_eq!(run.coverage().processed_end(), 0, "{text:?}");
        assert_eq!(run.usage().emitted_tokens(), 0, "{text:?}");
        assert_eq!(run.usage().retained_interpreted_bytes(), 0, "{text:?}");
    }
}

#[test]
fn t8_a_stop_after_earlier_tokens_keeps_those_tokens_and_their_coverage() {
    // `x` is committed as a Character token (0..1) before `<`; the stop's
    // trigger is `<!` at 1..3 and coverage never drops below the token.
    for text in ["x<!DOCTYPE svg>", "x<!DOCTYPE html PUBLIC \"x\">", "x<!xx>"] {
        let run = run(text);
        assert_eq!(shape(&run), vec![("character", (0, 1))], "{text:?}");
        assert_eq!(unsupported_markup_declaration(&run), (1, 3), "{text:?}");
        assert_eq!(run.coverage().processed_end(), 1, "{text:?}");
    }
}

#[test]
fn t8b_a_preprocessing_diagnostic_unit_inside_the_selected_states_keeps_the_exact_boundary() {
    // Each source reaches a different selected Doctype* state and then meets a
    // unit that raises a preprocessing diagnostic (U+0001 control character,
    // U+FDD0 / U+FFFE noncharacters). Such a unit proves the source left the
    // selected profile, so the stop must be the unchanged `<!` boundary with no
    // diagnostic committed from inside the rejected declaration.
    for text in [
        // DoctypeAfterKeyword, DoctypeBeforeName (after one and two spaces).
        "<!DOCTYPE\u{1}html>",
        "<!DOCTYPE \u{1}html>",
        "<!DOCTYPE  \u{1}html>",
        // DoctypeName.
        "<!DOCTYPE h\u{1}tml>",
        "<!DOCTYPE ht\u{fffe}ml>",
        // DoctypeAfterName.
        "<!DOCTYPE html\u{fdd0}>",
        "<!DOCTYPE html \u{1}>",
        "<!DOCTYPE html\u{1}",
        // A diagnostic unit after a CRLF separator, where CR/CRLF alone raise
        // none.
        "<!DOCTYPE\r\n\u{1}html>",
    ] {
        let run = run(text);
        assert!(run.tokens().is_empty(), "{text:?}: no token may commit");
        assert!(
            run.diagnostics().is_empty(),
            "{text:?}: no diagnostic from inside the declaration"
        );
        assert_eq!(run.usage().diagnostics(), 0, "{text:?}");
        assert_eq!(unsupported_markup_declaration(&run), (0, 2), "{text:?}");
        assert_eq!(run.coverage().processed_end(), 0, "{text:?}");
    }
}

#[test]
fn t8c_the_rollback_never_drops_below_already_committed_prefix_evidence() {
    // `x` is committed as token 0 (0..1) before `<`; the rejected declaration
    // then rolls back to exactly there and contributes no diagnostic.
    for text in [
        "x<!DOCTYPE \u{1}html>",
        "x<!DOCTYPE html\u{fdd0}>",
        "x<!DOCTYPE h\u{1}tml>",
    ] {
        let run = run(text);
        assert_eq!(shape(&run), vec![("character", (0, 1))], "{text:?}");
        assert!(run.diagnostics().is_empty(), "{text:?}");
        assert_eq!(unsupported_markup_declaration(&run), (1, 3), "{text:?}");
        assert_eq!(run.coverage().processed_end(), 1, "{text:?}");
    }
    // A diagnostic committed *before* the declaration is prior evidence and
    // stays: U+0001 at 0..1 is Data with its own diagnostic, then `<!` is 1..3.
    let run = run("\u{1}<!DOCTYPE \u{1}html>");
    assert_eq!(shape(&run), vec![("character", (0, 1))]);
    assert_eq!(run.diagnostics().len(), 1);
    assert_eq!(range_of(run.diagnostics()[0].location()), (0, 1));
    assert_eq!(unsupported_markup_declaration(&run), (1, 3));
    assert_eq!(run.coverage().processed_end(), 1);
}

#[test]
fn t8d_cr_and_crlf_are_not_preprocessing_diagnostics_in_the_selected_states() {
    for case in ACCEPTED.iter().filter(|c| c.text.contains('\r')) {
        let run = run(case.text);
        assert!(!run.is_incomplete(), "{}", case.id);
        assert!(run.diagnostics().is_empty(), "{}", case.id);
        assert_eq!(
            doctype(&run, 0).complete().fragment(),
            case.text,
            "{}",
            case.id
        );
    }
}

#[test]
fn t9_the_processing_instruction_stop_is_unchanged() {
    let run = run("<?x?>");
    let HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
        unsupported,
    )) = run.completion()
    else {
        panic!("expected unsupported")
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTokenizerCapability::ProcessingInstruction
    );
}

// ---------------------------------------------------------------------------
// Tokenizer: resource atomicity
// ---------------------------------------------------------------------------

fn assert_resource_stop(
    run: &HtmlTokenizerRunResult,
    resource: HtmlTokenizerResource,
    limit: usize,
    attempted: usize,
) {
    let HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(refusal)) =
        run.completion()
    else {
        panic!("expected a resource refusal: {:?}", run.completion())
    };
    assert_eq!(refusal.resource(), resource);
    assert_eq!(refusal.limit(), limit);
    assert_eq!(refusal.attempted(), attempted);
}

#[test]
fn t10_emitted_token_refusal_commits_no_doctype_token() {
    // `x` is token 0; the DOCTYPE would be the second emitted token.
    let tight = HtmlTokenizerLimits::new(1_024, 8_192, 1, 1_024, 256, 4_096, 1_024);
    let run = run_with("x<!DOCTYPE html>", tight);
    assert_resource_stop(&run, HtmlTokenizerResource::EmittedTokens, 1, 2);
    assert_eq!(shape(&run), vec![("character", (0, 1))]);
    assert_eq!(run.usage().emitted_tokens(), 1);
    // Only the Character token's one byte is retained: the selected name is
    // never partially committed.
    assert_eq!(run.usage().retained_interpreted_bytes(), 1);
    assert!(
        !run.tokens()
            .iter()
            .any(|t| matches!(t, HtmlToken::Doctype(_)))
    );

    // With a single emission available the DOCTYPE commits and the EOF is the
    // refused emission instead.
    let run = run_with("<!DOCTYPE html>", tight);
    assert_resource_stop(&run, HtmlTokenizerResource::EmittedTokens, 1, 2);
    assert_eq!(shape(&run), vec![("doctype", (0, 15))]);
    assert_eq!(run.usage().retained_interpreted_bytes(), 4);
}

#[test]
fn t11_retained_byte_refusal_commits_no_selected_name_evidence() {
    for retained in 0..4 {
        let tight = HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, retained, 1_024);
        let run = run_with("<!DOCTYPE html>", tight);
        assert_resource_stop(
            &run,
            HtmlTokenizerResource::RetainedInterpretedBytes,
            retained,
            4,
        );
        assert!(run.tokens().is_empty(), "limit {retained}");
        assert_eq!(
            run.usage().retained_interpreted_bytes(),
            0,
            "limit {retained}"
        );
        assert_eq!(run.usage().emitted_tokens(), 0, "limit {retained}");
    }
    // A prior token's committed bytes count: 1 (`x`) + 4 attempted = 5 > 4.
    let tight = HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4, 1_024);
    let run = run_with("x<!DOCTYPE html>", tight);
    assert_resource_stop(&run, HtmlTokenizerResource::RetainedInterpretedBytes, 4, 5);
    assert_eq!(shape(&run), vec![("character", (0, 1))]);
    assert_eq!(run.usage().retained_interpreted_bytes(), 1);
}

#[test]
fn t12_transition_step_refusal_at_every_point_leaves_no_doctype_evidence() {
    // Seventeen steps complete `<!DOCTYPE html>` (15 units + reconsumed `h` +
    // EOF). Every smaller budget refuses exactly one past the budget.
    for budget in 1..17 {
        let tight = HtmlTokenizerLimits::new(1_024, budget, 1_024, 1_024, 256, 4_096, 1_024);
        let run = run_with("<!DOCTYPE html>", tight);
        assert_resource_stop(
            &run,
            HtmlTokenizerResource::TransitionSteps,
            budget,
            budget + 1,
        );
        assert_eq!(run.usage().transition_steps(), budget, "budget {budget}");
        // Sixteen steps reach the EOF unit after the DOCTYPE committed; any
        // smaller budget refuses before the DOCTYPE token may commit.
        if budget < 16 {
            assert!(run.tokens().is_empty(), "budget {budget}");
            assert_eq!(
                run.usage().retained_interpreted_bytes(),
                0,
                "budget {budget}"
            );
            assert_eq!(run.usage().emitted_tokens(), 0, "budget {budget}");
        } else {
            assert_eq!(shape(&run), vec![("doctype", (0, 15))], "budget {budget}");
        }
    }
    let exact = HtmlTokenizerLimits::new(1_024, 17, 1_024, 1_024, 256, 4_096, 1_024);
    assert!(!run_with("<!DOCTYPE html>", exact).is_incomplete());
}

#[test]
fn t13_source_bytes_preflight_still_refuses_before_any_processing() {
    let tight = HtmlTokenizerLimits::new(14, 8_192, 1_024, 1_024, 256, 4_096, 1_024);
    let run = run_with("<!DOCTYPE html>", tight);
    assert_resource_stop(&run, HtmlTokenizerResource::SourceBytes, 14, 15);
    assert!(run.tokens().is_empty());
    assert_eq!(run.usage().transition_steps(), 0);
    let exact = HtmlTokenizerLimits::new(15, 8_192, 1_024, 1_024, 256, 4_096, 1_024);
    assert!(!run_with("<!DOCTYPE html>", exact).is_incomplete());
}

// ---------------------------------------------------------------------------
// Tokenizer: bounded mutation sweep against an independent byte matcher
// ---------------------------------------------------------------------------

/// A tiny hand-written matcher for exactly the selected profile, independent
/// of the tokenizer. It returns the DOCTYPE's end and the name range when
/// `bytes` begin with a selected DOCTYPE.
fn selected_prefix(bytes: &[u8]) -> Option<(usize, Range)> {
    let ws = |b: u8| matches!(b, b'\t' | b'\n' | 0x0c | b'\r' | b' ');
    let ci = |at: usize, word: &str| {
        bytes
            .get(at..at + word.len())
            .is_some_and(|slice| slice.eq_ignore_ascii_case(word.as_bytes()))
    };
    if !bytes.starts_with(b"<!") || !ci(2, "doctype") {
        return None;
    }
    let mut at = 9;
    if !bytes.get(at).copied().is_some_and(ws) {
        return None;
    }
    while bytes.get(at).copied().is_some_and(ws) {
        at += 1;
    }
    if !ci(at, "html") {
        return None;
    }
    let name = (at, at + 4);
    at += 4;
    while bytes.get(at).copied().is_some_and(ws) {
        at += 1;
    }
    (bytes.get(at) == Some(&b'>')).then_some((at + 1, name))
}

#[test]
fn t25_bounded_mutation_sweep_agrees_with_the_independent_matcher() {
    let alphabet: &[char] = &[
        'x', '>', ' ', '\n', '\r', '\t', '\u{c}', '\0', '\u{1}', '!', '<', '/', '"', '\u{a0}',
        '\u{fdd0}', 'h', 'D', 'l',
    ];
    let mut sources: Vec<String> = ACCEPTED.iter().map(|case| case.text.to_owned()).collect();
    sources.extend([
        "<!DOCTYPE html>".to_owned(),
        "<!DOCTYPE html><html>".to_owned(),
        "<!DOCTYPE html PUBLIC \"x\">".to_owned(),
    ]);
    let mut checked = 0usize;
    for base in &sources {
        let chars: Vec<char> = base.chars().collect();
        let mut candidates: Vec<String> = vec![base.clone()];
        for cut in 0..=chars.len() {
            candidates.push(chars[..cut].iter().collect());
        }
        for position in 0..chars.len() {
            let mut deleted = chars.clone();
            deleted.remove(position);
            candidates.push(deleted.iter().collect());
        }
        for position in 0..=chars.len() {
            for &inserted in alphabet {
                let mut with = chars.clone();
                with.insert(position, inserted);
                candidates.push(with.iter().collect());
                if position < chars.len() {
                    let mut replaced = chars.clone();
                    replaced[position] = inserted;
                    candidates.push(replaced.iter().collect());
                }
            }
        }
        for text in candidates {
            // Tokenizing never panics: every run satisfies its own contract.
            let run = run(&text);
            checked += 1;

            // Boundary purity: when the leading declaration is not a selected
            // DOCTYPE, the very first `<!` is the unchanged MarkupDeclaration
            // stop and nothing from inside the rejected declaration is
            // committed, whatever unit (including diagnostic-raising ones) the
            // declaration contains.
            if text.starts_with("<!") && selected_prefix(text.as_bytes()).is_none() {
                assert!(run.tokens().is_empty(), "{text:?}");
                assert!(run.diagnostics().is_empty(), "{text:?}");
                assert_eq!(run.coverage().processed_end(), 0, "{text:?}");
                assert_eq!(unsupported_markup_declaration(&run), (0, 2), "{text:?}");
            }
            // Soundness: every DOCTYPE token is an exact selected DOCTYPE.
            for token in run.tokens() {
                if let HtmlToken::Doctype(doctype) = token {
                    let fragment = doctype.complete().fragment();
                    let (end, name) = selected_prefix(fragment.as_bytes())
                        .unwrap_or_else(|| panic!("{text:?}: unselected DOCTYPE {fragment:?}"));
                    assert_eq!(end, fragment.len(), "{text:?}");
                    let start = doctype.complete().range().start();
                    assert_eq!(
                        range_of(doctype.name().source()),
                        (start + name.0, start + name.1),
                        "{text:?}"
                    );
                    assert_eq!(doctype.name().interpreted(), "html", "{text:?}");
                }
            }
            // Completeness at the start of the source.
            if let Some((end, name)) = selected_prefix(text.as_bytes()) {
                let token = doctype(&run, 0);
                assert_eq!(range_of(token.complete()), (0, end), "{text:?}");
                assert_eq!(range_of(token.name().source()), name, "{text:?}");
            }
        }
    }
    assert!(
        checked > 5_000,
        "the sweep covers a meaningful set: {checked}"
    );
}

// ---------------------------------------------------------------------------
// Tree: Initial theorem
// ---------------------------------------------------------------------------

/// Drives only the first token of a run through the raw session so the
/// private insertion mode and document mode a frozen result never exposes can
/// be inspected.
fn session_after_first_token(text: &str) -> (HtmlTreeSession, TokenOutcome) {
    let run = run(text);
    let mut session = HtmlTreeSession::new().expect("session start");
    let token = &run.tokens()[0];
    let trigger = token_trigger(token, 0);
    let admitted = admit(token).expect("admitted");
    let outcome = drive_token(&mut session, &admitted, &trigger).expect("no invariant failure");
    (session, outcome)
}

#[test]
fn t14_initial_consumes_the_selected_doctype_once_and_stays_no_quirks() {
    let fresh = HtmlTreeSession::new().expect("session start");
    assert_eq!(fresh.insertion_mode(), InsertionMode::Initial);
    assert_eq!(fresh.document_mode(), HtmlDocumentMode::NoQuirks);

    for case in ACCEPTED {
        let (session, outcome) = session_after_first_token(case.text);
        assert_eq!(outcome, TokenOutcome::Consumed, "{}", case.id);
        assert_eq!(
            session.insertion_mode(),
            InsertionMode::BeforeHtml,
            "{}",
            case.id
        );
        assert_eq!(
            session.document_mode(),
            HtmlDocumentMode::NoQuirks,
            "{}",
            case.id
        );
        // Document root plus exactly one DocumentType; nothing is open.
        assert_eq!(session.node_count(), 2, "{}", case.id);
        assert_eq!(session.open_element_count(), 0, "{}", case.id);
    }
}

#[test]
fn t15_the_missing_doctype_predecessor_is_unchanged_and_observably_distinct() {
    let (session, outcome) = session_after_first_token("<body>");
    assert_eq!(outcome, TokenOutcome::Consumed);
    // `<body>` reprocesses through BeforeHtml/BeforeHead/AfterHead into InBody.
    assert_eq!(session.document_mode(), HtmlDocumentMode::Quirks);
    assert_eq!(session.insertion_mode(), InsertionMode::InBody);

    let analysis = analyze("<body>");
    let [diagnostic, ..] = analysis.diagnostics() else {
        panic!("expected the missing-doctype diagnostic")
    };
    assert_eq!(diagnostic.code(), HtmlTreeDiagnosticCode::MissingDoctype);
    assert_eq!(
        diagnostic.recovery(),
        HtmlTreeRecovery::ContinuedInQuirksDocumentMode
    );
    assert_eq!(diagnostic.trigger().token_index(), 0);
    assert!(
        analysis
            .actions()
            .iter()
            .any(|a| a.trigger().token_index() == 0
                && matches!(a.kind(), HtmlTreeActionKind::ReprocessedToken))
    );
    assert!(
        !analysis
            .nodes_in_creation_order()
            .iter()
            .any(|n| matches!(n.kind(), HtmlTreeNodeKind::DocumentType(_)))
    );
}

fn node(analysis: &HtmlDocumentShellAnalysis, id: u32) -> &HtmlTreeNode {
    analysis
        .nodes_in_creation_order()
        .into_iter()
        .find(|node| node.id().creation_ordinal() == id)
        .unwrap_or_else(|| panic!("node #{id} exists"))
}

fn ids(node: &HtmlTreeNode) -> Vec<u32> {
    node.children()
        .iter()
        .map(|c| c.creation_ordinal())
        .collect()
}

fn document_type(node: &HtmlTreeNode) -> &HtmlDocumentType {
    let HtmlTreeNodeKind::DocumentType(doctype) = node.kind() else {
        panic!("{:?} is not a DocumentType", node.kind())
    };
    doctype
}

#[test]
fn t16_selected_doctype_with_an_implied_shell_builds_the_explicit_structure() {
    let analysis = analyze("<!DOCTYPE html>");
    assert!(analysis.is_complete());

    // Document(#0) -> [DocumentType(#1), html(#2)]; html -> [head(#3), body(#4)].
    assert_eq!(analysis.node_count(), 5);
    let document = node(&analysis, 0);
    assert!(matches!(document.kind(), HtmlTreeNodeKind::Document));
    assert_eq!(ids(document), [1, 2]);

    let doctype_node = node(&analysis, 1);
    assert_eq!(doctype_node.parent().map(|p| p.creation_ordinal()), Some(0));
    assert!(doctype_node.children().is_empty());
    let doctype = document_type(doctype_node);
    assert_eq!(range_of(doctype.complete()), (0, 15));
    assert_eq!(range_of(doctype.authored_name()), (10, 14));

    let html = node(&analysis, 2);
    assert_eq!(html.parent().map(|p| p.creation_ordinal()), Some(0));
    assert_eq!(ids(html), [3, 4]);

    // Both token 0 (DOCTYPE) and token 1 (EOF) are processed; coverage reaches
    // the DOCTYPE's authored end.
    assert_eq!(analysis.coverage().processed_tokens(), 2);
    assert_eq!(analysis.coverage().committed_end(), 15);
    // No MissingDoctype and no tree diagnostic at all.
    assert!(analysis.diagnostics().is_empty());
    assert!(analysis.tokenizer_run().diagnostics().is_empty());
}

#[test]
fn t17_the_doctype_action_is_exact_single_and_never_reprocessed() {
    let analysis = analyze("<!DOCTYPE html>");
    let on_token_0: Vec<&HtmlTreeAction> = analysis
        .actions()
        .iter()
        .filter(|a| a.trigger().token_index() == 0)
        .collect();
    let [action] = on_token_0.as_slice() else {
        panic!("token 0 produces exactly one action: {on_token_0:?}")
    };
    let HtmlTreeActionKind::InsertedAuthoredDocumentType { node } = action.kind() else {
        panic!("expected the DocumentType insertion: {:?}", action.kind())
    };
    assert_eq!(node.creation_ordinal(), 1);
    assert_eq!(
        action.trigger().authored_boundary().map(range_of),
        Some((0, 15))
    );
    // The implied shell is created by the end-of-file token (token 1), not by
    // reprocessing the DOCTYPE.
    assert!(
        analysis
            .actions()
            .iter()
            .filter(|a| matches!(a.kind(), HtmlTreeActionKind::ReprocessedToken))
            .all(|a| a.trigger().token_index() == 1)
    );
}

#[test]
fn t18_explicit_shell_after_the_doctype_places_the_doctype_before_html() {
    // `<html>` 15..21 `<head>` 21..27 `</head>` 27..34 `<body>` 34..40
    // `</body>` 40..47 `</html>` 47..54.
    let text = "<!DOCTYPE html><html><head></head><body></body></html>";
    let analysis = analyze(text);
    assert!(analysis.is_complete());
    let document = node(&analysis, 0);
    assert_eq!(ids(document), [1, 2]);
    assert!(matches!(
        node(&analysis, 1).kind(),
        HtmlTreeNodeKind::DocumentType(_)
    ));
    assert!(matches!(
        node(&analysis, 2).kind(),
        HtmlTreeNodeKind::Element(HtmlElement::Shell(shell))
            if shell.name() == HtmlShellElementName::Html
    ));
    assert_eq!(ids(node(&analysis, 2)), [3, 4]);
    // Stored order and creation identity never decide placement.
    assert_eq!(analysis.node_count(), 5);
}

#[test]
fn t19_identity_is_creation_order_independent_of_source_identity() {
    for id in [1_u64, 7, 99] {
        let analysis = construct_html_document_shell(&source_with(id, "<!DOCTYPE html>"), limits())
            .expect("no boundary failure");
        let ordinals: Vec<u32> = analysis
            .nodes_in_creation_order()
            .iter()
            .map(|n| n.id().creation_ordinal())
            .collect();
        assert_eq!(ordinals, [0, 1, 2, 3, 4], "source id {id}");
        assert_eq!(
            range_of(document_type(node(&analysis, 1)).complete()),
            (0, 15)
        );
    }
}

#[test]
fn t20_mixed_case_spelling_is_retained_exactly() {
    let analysis = analyze("<!DoCtYpE HTML>");
    let doctype = document_type(node(&analysis, 1));
    assert_eq!(doctype.complete().fragment(), "<!DoCtYpE HTML>");
    assert_eq!(doctype.authored_name().fragment(), "HTML");
}

// ---------------------------------------------------------------------------
// Tree: non-Initial boundary
// ---------------------------------------------------------------------------

fn assert_doctype_outside_initial(
    text: &str,
    trigger: Range,
    document_types: usize,
    refused_token: usize,
    nodes: usize,
) {
    let analysis = analyze(text);
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(unsupported)) =
        analysis.completion()
    else {
        panic!(
            "{text:?}: expected tree unsupported: {:?}",
            analysis.completion()
        )
    };
    assert_eq!(
        unsupported.capability(),
        HtmlTreeCapability::DoctypeOutsideInitial,
        "{text:?}"
    );
    assert_eq!(
        unsupported.trigger().authored_boundary().map(range_of),
        Some(trigger),
        "{text:?}"
    );
    assert_eq!(
        unsupported.trigger().token_index(),
        refused_token,
        "{text:?}"
    );
    let constructed = analysis
        .nodes_in_creation_order()
        .iter()
        .filter(|n| matches!(n.kind(), HtmlTreeNodeKind::DocumentType(_)))
        .count();
    assert_eq!(constructed, document_types, "{text:?}");
    // Refusal precedes any mutation: the refused token created no node, no
    // synthesized shell, no action, no reprocess and no diagnostic.
    assert_eq!(analysis.node_count(), nodes, "{text:?}");
    assert!(
        analysis
            .actions()
            .iter()
            .all(|a| a.trigger().token_index() < refused_token),
        "{text:?}"
    );
    assert!(
        analysis
            .diagnostics()
            .iter()
            .all(|d| d.trigger().token_index() < refused_token),
        "{text:?}"
    );
    // The refused token is not committed: coverage ends at its start.
    assert_eq!(analysis.coverage().committed_end(), trigger.0, "{text:?}");
    assert_eq!(
        analysis.coverage().processed_tokens(),
        refused_token,
        "{text:?}"
    );
    assert!(!analysis.is_complete(), "{text:?}");
}

#[test]
fn t21_a_second_doctype_constructs_no_second_document_type() {
    // First DOCTYPE 0..15; the second one is 15..30.
    assert_doctype_outside_initial("<!DOCTYPE html><!DOCTYPE html>", (15, 30), 1, 1, 2);
}

#[test]
fn t22_a_doctype_after_html_start_is_refused_after_missing_doctype() {
    // `<html>` 0..6; Initial reprocessed it with MissingDoctype; the DOCTYPE is
    // 6..21 in BeforeHead.
    assert_doctype_outside_initial("<html><!DOCTYPE html>", (6, 21), 0, 1, 2);
    let analysis = analyze("<html><!DOCTYPE html>");
    assert!(
        analysis
            .diagnostics()
            .iter()
            .any(|d| d.code() == HtmlTreeDiagnosticCode::MissingDoctype)
    );
}

#[test]
fn t23_a_doctype_after_the_closed_document_is_refused() {
    // `<body></body></html>` is 20 bytes; the DOCTYPE is 20..35 in
    // AfterAfterBody.
    assert_doctype_outside_initial("<body></body></html><!DOCTYPE html>", (20, 35), 0, 3, 4);
}

// ---------------------------------------------------------------------------
// Lower-layer incompleteness is never upgraded
// ---------------------------------------------------------------------------

#[test]
fn t24_lower_layer_refusals_are_never_upgraded_to_tree_success() {
    // Unsupported tokenizer boundary: no DocumentType, not complete.
    let unsupported = analyze("<!DOCTYPE svg>");
    assert!(!unsupported.is_complete());
    assert!(matches!(
        unsupported.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));
    assert!(
        !unsupported
            .nodes_in_creation_order()
            .iter()
            .any(|n| matches!(n.kind(), HtmlTreeNodeKind::DocumentType(_)))
    );

    // The DOCTYPE commits but the EOF emission is refused: the DocumentType is
    // honest committed meaning, yet the result is incomplete.
    let one_token = HtmlTokenizerLimits::new(1_024, 8_192, 1, 1_024, 256, 4_096, 1_024);
    let truncated = analyze_with("<!DOCTYPE html>", one_token);
    assert!(!truncated.is_complete());
    assert!(matches!(
        truncated.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
    ));
    assert!(truncated.tokenizer_run().is_incomplete());

    // A refused DOCTYPE emission leaves no DocumentType at all.
    let refused = analyze_with("x<!DOCTYPE html>", one_token);
    assert!(!refused.is_complete());
    assert!(
        !refused
            .nodes_in_creation_order()
            .iter()
            .any(|n| matches!(n.kind(), HtmlTreeNodeKind::DocumentType(_)))
    );
}

// ---------------------------------------------------------------------------
// Freeze sealing: malformed DocumentType evidence is rejected
// ---------------------------------------------------------------------------

struct Frozen {
    text: &'static str,
    limits: HtmlTokenizerLimits,
    analysis: HtmlDocumentShellAnalysis,
}

impl Frozen {
    fn new(text: &'static str) -> Self {
        Self::with_limits(text, limits())
    }

    fn with_limits(text: &'static str, limits: HtmlTokenizerLimits) -> Self {
        Self {
            text,
            limits,
            analysis: analyze_with(text, limits),
        }
    }

    fn trigger(&self, index: usize) -> HtmlTreeTokenTrigger {
        let boundary = match &self.analysis.tokenizer_run().tokens()[index] {
            HtmlToken::Character(c) => c.source().clone(),
            HtmlToken::Doctype(d) => d.complete().clone(),
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
            tokenize(&source(self.text), self.limits),
            parts,
        )
        .map(|_| ())
    }

    fn anchor(&self, start: usize, end: usize) -> crate::SourceAnchor {
        source(self.text).anchor(start, end).expect("valid anchor")
    }
}

const BASE: &str = "<!DOCTYPE html><html></html>";

fn id(ordinal: u32) -> HtmlConstructedNodeId {
    let mut counter = HtmlConstructedIdentityCounter::new();
    let mut found = None;
    for _ in 0..=ordinal {
        let reserved = counter.reserve().expect("headroom");
        counter.commit(reserved);
        found = Some(reserved);
    }
    found.expect("minted")
}

fn doctype_node_position(parts: &HtmlDocumentShellParts) -> usize {
    parts
        .nodes
        .iter()
        .position(|n| matches!(n.kind(), HtmlTreeNodeKind::DocumentType(_)))
        .expect("a DocumentType node")
}

fn replace_node(
    parts: &mut HtmlDocumentShellParts,
    position: usize,
    parent: Option<HtmlConstructedNodeId>,
    children: Option<Vec<HtmlConstructedNodeId>>,
    kind: Option<HtmlTreeNodeKind>,
) {
    let old = parts.nodes[position].clone();
    parts.nodes[position] = HtmlTreeNode::new(
        old.id(),
        parent.or(old.parent()),
        children.unwrap_or_else(|| old.children().to_vec()),
        kind.unwrap_or_else(|| old.kind().clone()),
    );
}

#[test]
fn fz0_the_unmutated_reconstructions_freeze() {
    for text in [
        "<!DOCTYPE html>",
        "<!doctype html>",
        "<!DOCTYPE html><html></html>",
        "<!DOCTYPE html><html><head></head><body></body></html>",
        "<!DOCTYPE html><!DOCTYPE html>",
        "<html><!DOCTYPE html>",
        "<body></body></html><!DOCTYPE html>",
        "<body>",
    ] {
        let frozen = Frozen::new(text);
        assert_eq!(frozen.freeze(frozen.parts()), Ok(()), "{text:?}");
    }
}

#[test]
fn fz1_a_document_type_stored_as_an_element_or_text_is_rejected() {
    let frozen = Frozen::new(BASE);
    let ordinal = frozen.parts().nodes[doctype_node_position(&frozen.parts())].id();

    let mut parts = frozen.parts();
    let position = doctype_node_position(&parts);
    let element = HtmlElement::Shell(HtmlShellElement::new(
        HtmlShellElementName::Html,
        HtmlShellElementOrigin::Authored {
            complete: frozen.anchor(0, 15),
            raw_name: frozen.anchor(10, 14),
        },
    ));
    replace_node(
        &mut parts,
        position,
        None,
        None,
        Some(HtmlTreeNodeKind::Element(element)),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeActionSubjectIsNotDocumentType(
            ordinal
        ))
    );

    let mut parts = frozen.parts();
    let position = doctype_node_position(&parts);
    let text = HtmlTextNode::new(
        "x".to_owned(),
        vec![HtmlTextContribution::new(
            frozen.anchor(0, 15),
            "x".to_owned(),
        )],
    );
    replace_node(
        &mut parts,
        position,
        None,
        None,
        Some(HtmlTreeNodeKind::Text(text)),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeActionSubjectIsNotDocumentType(
            ordinal
        ))
    );
}

#[test]
fn fz2_wrong_parent_and_child_relationships_are_rejected() {
    let frozen = Frozen::new(BASE);
    let doctype_id = id(1);
    let html_id = id(2);

    // Parent claims `html` while the Document still lists the DocumentType.
    let mut parts = frozen.parts();
    let position = doctype_node_position(&parts);
    replace_node(&mut parts, position, Some(html_id), None, None);
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::AsymmetricRelationship {
            parent: id(0),
            child: doctype_id
        })
    );

    // A DocumentType given a child.
    let mut parts = frozen.parts();
    let position = doctype_node_position(&parts);
    replace_node(&mut parts, position, None, Some(vec![html_id]), None);
    assert!(frozen.freeze(parts).is_err());

    // The Document no longer lists its DocumentType child.
    let mut parts = frozen.parts();
    let root = parts.root;
    let position = parts.nodes.iter().position(|n| n.id() == root).unwrap();
    replace_node(&mut parts, position, None, Some(vec![html_id]), None);
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::AsymmetricRelationship {
            parent: id(0),
            child: doctype_id
        })
    );
}

#[test]
fn fz3_wrong_final_order_is_rejected() {
    let frozen = Frozen::new(BASE);
    let mut parts = frozen.parts();
    let root = parts.root;
    let position = parts.nodes.iter().position(|n| n.id() == root).unwrap();
    replace_node(&mut parts, position, None, Some(vec![id(2), id(1)]), None);
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DocumentTypeIsNotFirstDocumentChild(
            id(1)
        ))
    );
}

#[test]
fn fz4_a_document_type_whose_parent_is_not_the_document_is_rejected() {
    // Hand-built shape Document -> html -> DocumentType, which is structurally
    // valid (the parent was created first) but is not the selected placement.
    let frozen = Frozen::new(BASE);
    let original = frozen.parts();
    let html_kind = original
        .nodes
        .iter()
        .find(|n| matches!(n.kind(), HtmlTreeNodeKind::Element(_)))
        .expect("html")
        .kind()
        .clone();
    let doctype_kind = original.nodes[doctype_node_position(&original)]
        .kind()
        .clone();
    let mut parts = frozen.parts();
    parts.nodes = vec![
        HtmlTreeNode::new(id(0), None, vec![id(1)], HtmlTreeNodeKind::Document),
        HtmlTreeNode::new(id(1), Some(id(0)), vec![id(2)], html_kind),
        HtmlTreeNode::new(id(2), Some(id(1)), Vec::new(), doctype_kind),
    ];
    parts.admitted_creation_events = 3;
    parts.actions.clear();
    parts.diagnostics.clear();
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DocumentTypeParentIsNotDocument(id(2)))
    );
}

#[test]
fn fz5_identity_inventory_corruptions_are_rejected() {
    let frozen = Frozen::new(BASE);

    let mut parts = frozen.parts();
    let duplicate = parts.nodes[doctype_node_position(&parts)].clone();
    parts.nodes.push(duplicate);
    parts.admitted_creation_events += 1;
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DuplicateConstructedIdentity(id(1)))
    );

    let mut parts = frozen.parts();
    parts.admitted_creation_events -= 1;
    assert!(matches!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::CreationEventInventoryMismatch { .. })
    ));

    // An identity beyond the admitted inventory.
    let mut parts = frozen.parts();
    let position = doctype_node_position(&parts);
    let old = parts.nodes[position].clone();
    parts.nodes[position] = HtmlTreeNode::new(id(9), old.parent(), Vec::new(), old.kind().clone());
    assert!(matches!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::UnadmittedConstructedIdentity(_))
    ));
}

#[test]
fn fz6_duplicate_missing_and_misdirected_insertion_actions_are_rejected() {
    let frozen = Frozen::new(BASE);
    let doctype_id = id(1);

    let is_insertion = |a: &HtmlTreeAction| {
        matches!(
            a.kind(),
            HtmlTreeActionKind::InsertedAuthoredDocumentType { .. }
        )
    };

    // Duplicate insertion for one node (and one token).
    let mut parts = frozen.parts();
    let position = parts.actions.iter().position(is_insertion).unwrap();
    let duplicate = parts.actions[position].clone();
    parts.actions.insert(position, duplicate);
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DuplicateDoctypeInsertion(doctype_id))
    );

    // A node without its insertion action.
    let mut parts = frozen.parts();
    parts.actions.retain(|a| !is_insertion(a));
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeInsertionInventoryMismatch(
            doctype_id
        ))
    );

    // An action whose subject is not a DocumentType.
    let mut parts = frozen.parts();
    let position = parts.actions.iter().position(is_insertion).unwrap();
    let trigger = parts.actions[position].trigger().clone();
    parts.actions[position] = HtmlTreeAction::new(
        HtmlTreeActionKind::InsertedAuthoredDocumentType { node: id(2) },
        trigger,
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeActionSubjectIsNotDocumentType(
            id(2)
        ))
    );

    // An action whose subject does not exist.
    let mut parts = frozen.parts();
    let position = parts.actions.iter().position(is_insertion).unwrap();
    let trigger = parts.actions[position].trigger().clone();
    parts.actions[position] = HtmlTreeAction::new(
        HtmlTreeActionKind::InsertedAuthoredDocumentType { node: id(40) },
        trigger,
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::UnresolvedActionSubject(id(40)))
    );

    // Two actions for the same token resolving to different nodes cannot both
    // be selected insertions; the second reuses token 0.
    let mut parts = frozen.parts();
    let position = parts.actions.iter().position(is_insertion).unwrap();
    let trigger = parts.actions[position].trigger().clone();
    parts.actions.insert(
        position + 1,
        HtmlTreeAction::new(
            HtmlTreeActionKind::InsertedAuthoredDocumentType { node: id(2) },
            trigger,
        ),
    );
    assert!(frozen.freeze(parts).is_err());
}

#[test]
fn fz7_an_insertion_triggered_by_a_non_doctype_token_is_rejected() {
    let frozen = Frozen::new(BASE);
    let mut parts = frozen.parts();
    let position = parts
        .actions
        .iter()
        .position(|a| {
            matches!(
                a.kind(),
                HtmlTreeActionKind::InsertedAuthoredDocumentType { .. }
            )
        })
        .unwrap();
    // Token 1 is the `<html>` tag.
    parts.actions[position] = HtmlTreeAction::new(
        HtmlTreeActionKind::InsertedAuthoredDocumentType { node: id(1) },
        frozen.trigger(1),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeInsertionTriggerMismatch {
            node: id(1),
            token_index: 1
        })
    );
}

#[test]
fn fz8_wrong_complete_or_name_anchors_are_rejected() {
    let frozen = Frozen::new(BASE);

    // Complete anchor shifted but still containing the name: evidence no
    // longer equals the exact DOCTYPE token.
    let mut parts = frozen.parts();
    let position = doctype_node_position(&parts);
    replace_node(
        &mut parts,
        position,
        None,
        None,
        Some(HtmlTreeNodeKind::DocumentType(HtmlDocumentType::new(
            frozen.anchor(0, 14),
            frozen.anchor(10, 14),
        ))),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeEvidenceMismatch(id(1)))
    );

    // Name anchor inside the complete range but not the token's name.
    let mut parts = frozen.parts();
    let position = doctype_node_position(&parts);
    replace_node(
        &mut parts,
        position,
        None,
        None,
        Some(HtmlTreeNodeKind::DocumentType(HtmlDocumentType::new(
            frozen.anchor(0, 15),
            frozen.anchor(2, 9),
        ))),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeEvidenceMismatch(id(1)))
    );

    // Name outside the complete range.
    let mut parts = frozen.parts();
    let position = doctype_node_position(&parts);
    replace_node(
        &mut parts,
        position,
        None,
        None,
        Some(HtmlTreeNodeKind::DocumentType(HtmlDocumentType::new(
            frozen.anchor(0, 15),
            frozen.anchor(15, 21),
        ))),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::AuthoredDoctypeNameOutsideCompleteDoctype(id(1)))
    );

    // Foreign source evidence.
    let foreign = SourceText::new(SourceId::new(9), BASE.to_owned());
    let mut parts = frozen.parts();
    let position = doctype_node_position(&parts);
    replace_node(
        &mut parts,
        position,
        None,
        None,
        Some(HtmlTreeNodeKind::DocumentType(HtmlDocumentType::new(
            foreign.anchor(0, 15).unwrap(),
            foreign.anchor(10, 14).unwrap(),
        ))),
    );
    assert!(matches!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::ForeignSourceEvidence { .. })
    ));
}

#[test]
fn fz9_a_selected_doctype_cannot_pair_with_missing_doctype_or_reprocess() {
    let frozen = Frozen::new(BASE);

    let mut parts = frozen.parts();
    parts.diagnostics.insert(
        0,
        HtmlTreeDiagnostic::new(
            HtmlTreeDiagnosticCode::MissingDoctype,
            frozen.trigger(0),
            HtmlTreeRecovery::ContinuedInQuirksDocumentMode,
        ),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeTokenPairedWithMissingDoctype { token_index: 0 })
    );

    // A MissingDoctype for any other token cannot coexist with a DocumentType
    // either: Initial leaves exactly once.
    let mut parts = frozen.parts();
    parts.diagnostics.insert(
        0,
        HtmlTreeDiagnostic::new(
            HtmlTreeDiagnosticCode::MissingDoctype,
            frozen.trigger(1),
            HtmlTreeRecovery::ContinuedInQuirksDocumentMode,
        ),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeCoexistsWithMissingDoctype(id(
            1
        )))
    );

    let mut parts = frozen.parts();
    let position = parts
        .actions
        .iter()
        .position(|a| {
            matches!(
                a.kind(),
                HtmlTreeActionKind::InsertedAuthoredDocumentType { .. }
            )
        })
        .unwrap();
    parts.actions.insert(
        position + 1,
        HtmlTreeAction::new(HtmlTreeActionKind::ReprocessedToken, frozen.trigger(0)),
    );
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeTokenReprocessed { token_index: 0 })
    );
}

#[test]
fn fz10_a_document_type_constructed_outside_initial_is_rejected() {
    // Fabricated: a lone DocumentType built from the DOCTYPE at token 1 of a
    // run whose token 0 is `<html>`. Initial can only ever consume token 0.
    let frozen = Frozen::new("<html><!DOCTYPE html>");
    let (complete, name) = {
        let HtmlToken::Doctype(token) = &frozen.analysis.tokenizer_run().tokens()[1] else {
            panic!("token 1 is the DOCTYPE")
        };
        (token.complete().clone(), token.name().source().clone())
    };
    let mut parts = frozen.parts();
    parts.nodes = vec![
        HtmlTreeNode::new(id(0), None, vec![id(1)], HtmlTreeNodeKind::Document),
        HtmlTreeNode::new(
            id(1),
            Some(id(0)),
            Vec::new(),
            HtmlTreeNodeKind::DocumentType(HtmlDocumentType::new(complete, name)),
        ),
    ];
    parts.admitted_creation_events = 2;
    parts.diagnostics.clear();
    parts.actions = vec![HtmlTreeAction::new(
        HtmlTreeActionKind::InsertedAuthoredDocumentType { node: id(1) },
        frozen.trigger(1),
    )];
    parts.processed_tokens = 2;
    parts.committed_prefix_end = 21;
    parts.completion =
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete);
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeConstructedOutsideInitial {
            node: id(1),
            token_index: 1
        })
    );
}

#[test]
fn fz11_a_committed_doctype_token_without_a_node_is_rejected() {
    // Fabricated: the DOCTYPE token is committed (two processed tokens) but
    // neither a DocumentType node nor its insertion action exists.
    let frozen = Frozen::new("<!DOCTYPE html>");
    let original = frozen.parts();
    let html_kind = original
        .nodes
        .iter()
        .find(|n| matches!(n.kind(), HtmlTreeNodeKind::Element(_)))
        .expect("html")
        .kind()
        .clone();
    let mut parts = frozen.parts();
    parts.nodes = vec![
        HtmlTreeNode::new(id(0), None, vec![id(1)], HtmlTreeNodeKind::Document),
        HtmlTreeNode::new(id(1), Some(id(0)), Vec::new(), html_kind),
    ];
    parts.admitted_creation_events = 2;
    parts.actions.clear();
    parts.diagnostics.clear();
    parts.completion =
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete);
    assert_eq!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::DoctypeTokenWithoutInsertion { token_index: 0 })
    );
}

#[test]
fn fz12_an_unsupported_trigger_cannot_become_a_document_type_origin() {
    // Second DOCTYPE 15..30 is the unsupported trigger; fabricate a DocumentType
    // whose complete evidence is that exact range.
    let frozen = Frozen::new("<!DOCTYPE html><!DOCTYPE html>");
    let mut parts = frozen.parts();
    let position = doctype_node_position(&parts);
    replace_node(
        &mut parts,
        position,
        None,
        None,
        Some(HtmlTreeNodeKind::DocumentType(HtmlDocumentType::new(
            frozen.anchor(15, 30),
            frozen.anchor(25, 29),
        ))),
    );
    // The mutated evidence is caught as not equal to the token it claims.
    assert!(frozen.freeze(parts).is_err());
}

#[test]
fn fz13_lower_layer_incompleteness_is_never_upgraded_to_complete() {
    let one_token = HtmlTokenizerLimits::new(1_024, 8_192, 1, 1_024, 256, 4_096, 1_024);
    let frozen = Frozen::with_limits("<!DOCTYPE html>", one_token);
    assert_eq!(frozen.freeze(frozen.parts()), Ok(()));
    let mut parts = frozen.parts();
    parts.completion = HtmlTreeCompletion::Complete;
    assert!(matches!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::CompletionUpgrade(_))
    ));

    // A complete tokenizer run whose tree stopped early is not upgraded either.
    let frozen = Frozen::new("<!DOCTYPE html><!DOCTYPE html>");
    let mut parts = frozen.parts();
    parts.completion = HtmlTreeCompletion::Complete;
    assert!(matches!(
        frozen.freeze(parts),
        Err(HtmlTreeFreezeError::CompletionUpgrade(_))
    ));
}
