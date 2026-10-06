//! Production correspondence for the Data / selected Title-RCDATA Numeric
//! Character Reference successor (Issue #880).
//!
//! Expectations here are authored by hand from #880, the #348 focused
//! placement review, and the accepted #878 semantic theorem. They are never
//! produced by production output, a browser, or an external parser, and this
//! module deliberately does not import the candidate-independent validation
//! machine in the sibling `data_rcdata_numeric_reference_successor_validation`
//! module.
//!
//! Each test names the incorrect implementation it rejects.

use crate::{SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::diagnostic::HtmlTokenizerDiagnosticCode as Diag;
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerResource};
use super::super::tokenizer::result::{
    HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause, HtmlTokenizerRunResult,
};
use super::driver::construct_html_document_shell;
use super::result::{HtmlDocumentShellAnalysis, HtmlTreeDiagnosticCode, HtmlTreeNodeKind};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

const SOURCE: usize = 1 << 20;
const BIG: usize = 1 << 20;

#[derive(Clone, Copy)]
struct Lim {
    steps: usize,
    tokens: usize,
    diagnostics: usize,
    retained: usize,
}

const GENEROUS: Lim = Lim {
    steps: BIG,
    tokens: BIG,
    diagnostics: BIG,
    retained: BIG,
};

impl Lim {
    fn limits(self) -> HtmlTokenizerLimits {
        HtmlTokenizerLimits::new(
            SOURCE,
            self.steps,
            self.tokens,
            self.diagnostics,
            256,
            self.retained,
            4_096,
        )
    }
}

fn lex_with(text: &str, lim: Lim) -> HtmlTokenizerRunResult {
    let source = SourceText::new(SourceId::new(1), text.to_owned());
    tokenize(&source, lim.limits())
}

fn lex(text: &str) -> HtmlTokenizerRunResult {
    lex_with(text, GENEROUS)
}

fn analyze_with(text: &str, lim: Lim) -> HtmlDocumentShellAnalysis {
    let source = SourceText::new(SourceId::new(1), text.to_owned());
    construct_html_document_shell(&source, lim.limits()).expect("Numeric production boundary")
}

fn analyze(text: &str) -> HtmlDocumentShellAnalysis {
    analyze_with(text, GENEROUS)
}

type Chars = Vec<((usize, usize), String)>;

fn chars(run: &HtmlTokenizerRunResult) -> Chars {
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

fn tag_count(run: &HtmlTokenizerRunResult, kind: HtmlTagKind) -> usize {
    run.tokens()
        .iter()
        .filter(|token| matches!(token, HtmlToken::Tag(tag) if tag.kind() == kind))
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

fn expect(chars_expected: &[((usize, usize), &str)]) -> Chars {
    chars_expected
        .iter()
        .map(|(range, text)| (*range, (*text).to_owned()))
        .collect()
}

fn steps(run: &HtmlTokenizerRunResult) -> usize {
    run.usage().transition_steps()
}

fn processed(run: &HtmlTokenizerRunResult) -> usize {
    run.coverage().processed_end()
}

fn complete(run: &HtmlTokenizerRunResult) -> bool {
    matches!(run.completion(), HtmlTokenizerCompletion::Complete)
}

/// `(resource, limit, attempted)` of a resource-limited run.
fn refusal(run: &HtmlTokenizerRunResult) -> Option<(HtmlTokenizerResource, usize, usize)> {
    match run.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(limit)) => {
            Some((limit.resource(), limit.limit(), limit.attempted()))
        }
        _ => None,
    }
}

fn unsupported(run: &HtmlTokenizerRunResult) -> bool {
    matches!(
        run.completion(),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(_))
    )
}

/// Every committed diagnostic lies inside the committed coverage.
fn assert_diagnostics_within_coverage(run: &HtmlTokenizerRunResult, label: &str) {
    for diagnostic in run.diagnostics() {
        assert!(
            diagnostic.location().range().end() <= run.coverage().processed_end(),
            "{label}: diagnostic {:?} ends past coverage {}",
            diagnostic.code(),
            run.coverage().processed_end()
        );
    }
}

fn shift(by: usize, items: &[((usize, usize), &str)]) -> Chars {
    items
        .iter()
        .map(|((start, end), text)| ((start + by, end + by), (*text).to_owned()))
        .collect()
}

fn shift_diags(by: usize, items: &[(Diag, (usize, usize))]) -> Vec<(Diag, (usize, usize))> {
    items
        .iter()
        .map(|(code, (start, end))| (*code, (start + by, end + by)))
        .collect()
}

/// Wraps `inner` in a selected Title and returns the coordinated analysis.
/// Authored offsets inside `inner` shift by the 7-byte `<title>` start tag.
fn title(inner: &str) -> HtmlDocumentShellAnalysis {
    analyze(&format!("<title>{inner}</title>"))
}

/// The character tokens that lie strictly between `<title>` and `</title>`.
fn title_chars(analysis: &HtmlDocumentShellAnalysis, inner_len: usize) -> Chars {
    chars(analysis.tokenizer_run())
        .into_iter()
        .filter(|((start, end), _)| *start >= 7 && *end <= 7 + inner_len)
        .collect()
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

fn contributions(analysis: &HtmlDocumentShellAnalysis) -> Chars {
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

fn tree_codes(analysis: &HtmlDocumentShellAnalysis) -> Vec<HtmlTreeDiagnosticCode> {
    analysis
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code())
        .collect()
}

const MISSING: Diag = Diag::MissingSemicolonAfterCharacterReference;
const ABSENCE: Diag = Diag::AbsenceOfDigitsInNumericCharacterReference;
const NULL: Diag = Diag::NullCharacterReference;
const OUTSIDE: Diag = Diag::CharacterReferenceOutsideUnicodeRange;
const SURROGATE: Diag = Diag::SurrogateCharacterReference;
const NONCHAR: Diag = Diag::NoncharacterCharacterReference;
const CONTROL: Diag = Diag::ControlCharacterReference;
const INPUT_CONTROL: Diag = Diag::ControlCharacterInInputStream;

// ---------------------------------------------------------------------------
// NR1 - the lexical matrix, in both return contexts
// ---------------------------------------------------------------------------

type Case = (
    &'static str,
    &'static [((usize, usize), &'static str)],
    &'static [(Diag, (usize, usize))],
);

const MATRIX: &[Case] = &[
    ("&#65;", &[((0, 5), "A")], &[]),
    ("&#65", &[((0, 4), "A")], &[(MISSING, (4, 4))]),
    ("&#x41;", &[((0, 6), "A")], &[]),
    ("&#X41;", &[((0, 6), "A")], &[]),
    ("&#x41", &[((0, 5), "A")], &[(MISSING, (5, 5))]),
    ("&#X41", &[((0, 5), "A")], &[(MISSING, (5, 5))]),
    ("&#xaB;", &[((0, 6), "\u{ab}")], &[]),
    // No digits: the literal prefix is flushed and the offending unit is
    // reconsumed by the return state as its own run.
    (
        "&#;",
        &[((0, 2), "&#"), ((2, 3), ";")],
        &[(ABSENCE, (2, 3))],
    ),
    (
        "&#A",
        &[((0, 2), "&#"), ((2, 3), "A")],
        &[(ABSENCE, (2, 3))],
    ),
    (
        "&#x;",
        &[((0, 3), "&#x"), ((3, 4), ";")],
        &[(ABSENCE, (3, 4))],
    ),
    (
        "&#xZ",
        &[((0, 3), "&#x"), ((3, 4), "Z")],
        &[(ABSENCE, (3, 4))],
    ),
    (
        "&#X;",
        &[((0, 3), "&#X"), ((3, 4), ";")],
        &[(ABSENCE, (3, 4))],
    ),
    ("&#", &[((0, 2), "&#")], &[(ABSENCE, (2, 2))]),
    ("&#x", &[((0, 3), "&#x")], &[(ABSENCE, (3, 3))]),
    // Numeric End mappings.
    ("&#0;", &[((0, 4), "\u{fffd}")], &[(NULL, (3, 4))]),
    ("&#xD800;", &[((0, 8), "\u{fffd}")], &[(SURROGATE, (7, 8))]),
    (
        "&#1114112;",
        &[((0, 10), "\u{fffd}")],
        &[(OUTSIDE, (9, 10))],
    ),
    ("&#xFDD0;", &[((0, 8), "\u{fdd0}")], &[(NONCHAR, (7, 8))]),
    ("&#128;", &[((0, 6), "\u{20ac}")], &[(CONTROL, (5, 6))]),
    ("&#x80;", &[((0, 6), "\u{20ac}")], &[(CONTROL, (5, 6))]),
    // CR is a control reference with no override: the decoded value is CR,
    // never normalized to LF.
    ("&#13;", &[((0, 5), "\r")], &[(CONTROL, (4, 5))]),
    ("&#9;", &[((0, 4), "\t")], &[]),
    // Semicolonless End: missing-semicolon first, then End, both at the
    // terminator, and the terminator stays outside the reference span.
    (
        "&#0x",
        &[((0, 3), "\u{fffd}"), ((3, 4), "x")],
        &[(MISSING, (3, 4)), (NULL, (3, 4))],
    ),
    (
        "&#0",
        &[((0, 3), "\u{fffd}")],
        &[(MISSING, (3, 3)), (NULL, (3, 3))],
    ),
    (
        "&#65x",
        &[((0, 4), "A"), ((4, 5), "x")],
        &[(MISSING, (4, 5))],
    ),
    (
        "&#65a;",
        &[((0, 4), "A"), ((4, 6), "a;")],
        &[(MISSING, (4, 5))],
    ),
    (
        "&#x41g",
        &[((0, 5), "A"), ((5, 6), "g")],
        &[(MISSING, (5, 6))],
    ),
    // Range boundary: U+10FFFF is representable (a noncharacter); one more is
    // outside the range.
    (
        "&#1114111;",
        &[((0, 10), "\u{10ffff}")],
        &[(NONCHAR, (9, 10))],
    ),
    (
        "&#x10FFFF;",
        &[((0, 10), "\u{10ffff}")],
        &[(NONCHAR, (9, 10))],
    ),
    (
        "&#x110000;",
        &[((0, 10), "\u{fffd}")],
        &[(OUTSIDE, (9, 10))],
    ),
    // No Named lookup after the Numeric branch, and no recursive decoding.
    (
        "&#65amp;",
        &[((0, 4), "A"), ((4, 8), "amp;")],
        &[(MISSING, (4, 5))],
    ),
    ("&#38;amp;", &[((0, 5), "&"), ((5, 9), "amp;")], &[]),
    ("&#38;#65;", &[((0, 5), "&"), ((5, 9), "#65;")], &[]),
    (
        "a&#38;b",
        &[((0, 1), "a"), ((1, 6), "&"), ((6, 7), "b")],
        &[],
    ),
    (
        "&amp;&#65;&lt;",
        &[((0, 5), "&"), ((5, 10), "A"), ((10, 14), "<")],
        &[],
    ),
    // C1 holes keep their own scalar (control, no remap); every other control
    // is a control reference.
    ("&#129;", &[((0, 6), "\u{81}")], &[(CONTROL, (5, 6))]),
    ("&#1;", &[((0, 4), "\u{1}")], &[(CONTROL, (3, 4))]),
    ("&#x7F;", &[((0, 6), "\u{7f}")], &[(CONTROL, (5, 6))]),
    ("&#10;", &[((0, 5), "\n")], &[]),
    ("&#12;", &[((0, 5), "\u{c}")], &[]),
    ("&#32;", &[((0, 5), " ")], &[]),
    ("&#xDFFF;", &[((0, 8), "\u{fffd}")], &[(SURROGATE, (7, 8))]),
    ("&#x1F600;", &[((0, 9), "\u{1f600}")], &[]),
];

/// Falsifies: any lexical, mapping, anchor, or authored-span defect, in the
/// Data return context.
#[test]
fn nr1_data_lexical_matrix() {
    for (text, expected, expected_diags) in MATRIX {
        let run = lex(text);
        assert!(complete(&run), "{text:?} completes");
        assert_eq!(chars(&run), expect(expected), "{text:?} tokens");
        assert_eq!(diags(&run), expected_diags.to_vec(), "{text:?} diagnostics");
        assert_diagnostics_within_coverage(&run, text);
        assert_eq!(run.usage().peak_temporary_buffer_bytes(), 0, "{text:?}");
        assert!(!unsupported(&run), "{text:?}");
    }
}

/// Falsifies: the same matrix under the selected Title/RCDATA return owner.
#[test]
fn nr2_rcdata_lexical_matrix() {
    for (text, expected, expected_diags) in MATRIX {
        let analysis = title(text);
        let run = analysis.tokenizer_run();
        assert!(complete(run), "{text:?} completes inside Title");
        assert_eq!(
            title_chars(&analysis, text.len()),
            shift(7, expected),
            "{text:?} tokens"
        );
        // A zero-width EOF site becomes the `<` of the appropriate `</title>`
        // in this wrapper: the terminator is one authored unit wide there.
        let expected_sites: Vec<_> = expected_diags
            .iter()
            .map(|(code, (start, end))| {
                if start == end {
                    (*code, (start + 7, end + 7 + 1))
                } else {
                    (*code, (start + 7, end + 7))
                }
            })
            .collect();
        assert_eq!(diags(run), expected_sites, "{text:?} diagnostics");
        assert_diagnostics_within_coverage(run, text);
        assert_eq!(run.usage().peak_temporary_buffer_bytes(), 0, "{text:?}");
        assert!(analysis.is_complete(), "{text:?}");
    }
}

// ---------------------------------------------------------------------------
// NR3 - long runs: value-driven, never length-driven
// ---------------------------------------------------------------------------

/// Falsifies: overflow inferred from digit count (a long leading-zero run is
/// not an overflow), and a wrapping accumulator.
#[test]
fn nr3_long_runs_are_decided_by_value_not_length() {
    let zeros = "0".repeat(5_000);
    for (text, expected, expected_diags) in [
        (format!("&#{zeros}65;"), "A", Vec::new()),
        (format!("&#x{zeros}41;"), "A", Vec::new()),
        (format!("&#{zeros}0;"), "\u{fffd}", vec![NULL]),
    ] {
        let run = lex(&text);
        assert!(complete(&run));
        let end = text.len();
        assert_eq!(chars(&run), vec![((0, end), expected.to_owned())]);
        assert_eq!(
            diags(&run)
                .iter()
                .map(|(code, _)| *code)
                .collect::<Vec<_>>(),
            expected_diags
        );
    }

    // Overflowing runs saturate and stay outside the range, however long and
    // whatever a wrapping 32-bit accumulator would have produced: `4294967361`
    // wraps to 65 ('A') in 32 bits, `x100000000000000041` wraps to 0x41.
    for text in [
        format!("&#{};", "9".repeat(5_000)),
        format!("&#x{};", "F".repeat(5_000)),
        "&#4294967361;".to_owned(),
        "&#x100000041;".to_owned(),
        "&#x10000000000000041;".to_owned(),
        format!("&#1{};", "0".repeat(5_000)),
    ] {
        let run = lex(&text);
        assert!(complete(&run), "{}", &text[..text.len().min(24)]);
        assert_eq!(
            chars(&run),
            vec![((0, text.len()), "\u{fffd}".to_owned())],
            "{}",
            &text[..text.len().min(24)]
        );
        assert_eq!(diags(&run), vec![(OUTSIDE, (text.len() - 1, text.len()))]);
    }
}

/// Falsifies: a digit run compressed into one transition, and an uncounted or
/// double-counted Numeric End.
#[test]
fn nr4_transition_steps_follow_the_per_state_dispatch_formula() {
    // Hand derivation. `&#65;`: Data(&) CharRef(#) Numeric(6) Decimal(6,
    // reconsume) Decimal(5) Decimal(;) End Data(EOF) = 8.
    for (text, expected) in [
        ("&#65;", 8),
        // EOF is examined by Decimal, End follows, then Data(EOF).
        ("&#65", 8),
        // Hex Start adds one examination.
        ("&#x41;", 9),
        // The terminator is examined by Decimal, End, then again by Data
        // (reconsume), then Data(EOF).
        ("&#65x", 9),
        ("&#;", 5),
        ("&#xZ", 6),
        ("&#", 4),
    ] {
        assert_eq!(steps(&lex(text)), expected, "{text:?}");
    }
    // A run of `n` digits costs `n` Decimal examinations plus the first digit's
    // reconsume: `&#` + n digits + `;` = 6 + n steps in total.
    for digits in [1usize, 2, 10, 100, 1_000, 5_000] {
        let text = format!("&#{};", "7".repeat(digits));
        assert_eq!(steps(&lex(&text)), 6 + digits, "{digits} digits");
    }
}

// ---------------------------------------------------------------------------
// NR5 - decoded output is never tokenizer input
// ---------------------------------------------------------------------------

/// Falsifies: decoded `<` retokenized as markup, decoded `&` decoded again,
/// and decoded CR re-run through input preprocessing.
#[test]
fn nr5_decoded_output_is_never_authored_syntax() {
    let run = lex("&#60;/body>");
    assert_eq!(chars(&run), expect(&[((0, 5), "<"), ((5, 11), "/body>")]));
    assert_eq!(tag_count(&run, HtmlTagKind::End), 0);
    assert_eq!(tag_count(&run, HtmlTagKind::Start), 0);
    // The same bytes authored literally are an end tag.
    assert_eq!(tag_count(&lex("</body>"), HtmlTagKind::End), 1);

    let run = lex("&#60;b>");
    assert_eq!(tag_count(&run, HtmlTagKind::Start), 0);

    let run = lex("&#38;amp;");
    assert_eq!(chars(&run), expect(&[((0, 5), "&"), ((5, 9), "amp;")]));

    // Decoded CR stays CR and is not normalized; an authored CR is LF.
    let run = lex("&#13;");
    assert_eq!(chars(&run), expect(&[((0, 5), "\r")]));
    assert_eq!(diags(&run), vec![(CONTROL, (4, 5))]);
    let authored = lex("\r");
    assert_eq!(chars(&authored), expect(&[((0, 1), "\n")]));

    // Inside Title the decoded `</title>` is text, not the appropriate close.
    let analysis = analyze("<title>&#60;/title></title>");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["</title>".to_owned()]);
}

// ---------------------------------------------------------------------------
// NR6 - return-state ownership
// ---------------------------------------------------------------------------

/// Falsifies: a collapsed return owner. Data resumes Data (a following `<b>`
/// is a tag); RCDATA resumes RCDATA (the same bytes stay text).
#[test]
fn nr6_return_owners_are_not_collapsed() {
    let run = lex("&#65;<b>");
    assert_eq!(tag_count(&run, HtmlTagKind::Start), 1);
    assert_eq!(chars(&run), expect(&[((0, 5), "A")]));

    let run = lex("&#65<b>");
    assert_eq!(tag_count(&run, HtmlTagKind::Start), 1);
    assert_eq!(diags(&run), vec![(MISSING, (4, 5))]);

    let analysis = title("&#65;<b>");
    assert!(analysis.is_complete());
    assert_eq!(tag_count(analysis.tokenizer_run(), HtmlTagKind::Start), 1);
    assert_eq!(texts(&analysis), vec!["A<b>".to_owned()]);

    let analysis = title("&#65<b>");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["A<b>".to_owned()]);
    // The terminator `<` follows the reference without being part of it.
    assert_eq!(diags(analysis.tokenizer_run()), vec![(MISSING, (11, 12))]);

    // The semicolonless RCDATA terminator may be the appropriate close.
    let analysis = title("&#65");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["A".to_owned()]);
    assert_eq!(diags(analysis.tokenizer_run()), vec![(MISSING, (11, 12))]);

    // Authored Data NUL (exact U+0000, no replacement) and RCDATA NUL stay
    // separate after a Numeric reference.
    let run = lex("&#65;\0");
    assert!(complete(&run));
    assert_eq!(chars(&run), expect(&[((0, 5), "A"), ((5, 6), "\0")]));
    let analysis = title("&#65;\0");
    assert!(
        unsupported(analysis.tokenizer_run()),
        "RCDATA NUL stays refused"
    );
}

// ---------------------------------------------------------------------------
// NR7 - exactly-once preprocessing of the reconsumed terminator
// ---------------------------------------------------------------------------

/// Falsifies: input preprocessing repeated when the terminator is reconsumed
/// in the return state, and diagnostics ordered before their cause.
#[test]
fn nr7_terminator_is_preprocessed_exactly_once() {
    type TerminatorCase = (&'static str, (usize, usize), &'static [Diag], &'static str);
    let cases: &[TerminatorCase] = &[
        // text, terminator range, ordered diagnostics, interpreted terminator
        ("&#65\u{1}", (4, 5), &[INPUT_CONTROL, MISSING], "\u{1}"),
        ("&#65\u{7f}", (4, 5), &[INPUT_CONTROL, MISSING], "\u{7f}"),
        ("&#65\u{85}", (4, 6), &[INPUT_CONTROL, MISSING], "\u{85}"),
        ("&#65\r", (4, 5), &[MISSING], "\n"),
        ("&#65\r\n", (4, 6), &[MISSING], "\n"),
        ("&#65 ", (4, 5), &[MISSING], " "),
    ];
    for (text, (start, end), codes, interpreted) in cases {
        let run = lex(text);
        assert!(complete(&run), "{text:?}");
        assert_eq!(
            chars(&run),
            expect(&[((0, 4), "A"), ((*start, *end), interpreted)]),
            "{text:?}"
        );
        assert_eq!(
            diags(&run),
            codes
                .iter()
                .map(|code| (*code, (*start, *end)))
                .collect::<Vec<_>>(),
            "{text:?}: one diagnostic per cause, none repeated by the reconsume"
        );
        assert_diagnostics_within_coverage(&run, text);
    }

    // The same theorem under the RCDATA owner (shifted by `<title>`).
    for (text, (start, end), codes, interpreted) in cases {
        let analysis = title(text);
        let run = analysis.tokenizer_run();
        assert!(complete(run), "{text:?}");
        assert_eq!(
            diags(run),
            codes
                .iter()
                .map(|code| (*code, (*start + 7, *end + 7)))
                .collect::<Vec<_>>(),
            "{text:?} in RCDATA"
        );
        let inner = title_chars(&analysis, text.len());
        assert_eq!(
            inner,
            shift(7, &[((0, 4), "A"), ((*start, *end), interpreted)]),
            "{text:?} in RCDATA"
        );
    }
}

// ---------------------------------------------------------------------------
// NR8 - tree and Product cases
// ---------------------------------------------------------------------------

/// Falsifies: authored-spelling-based placement and lost source-backed
/// contribution ordering.
#[test]
fn nr8_tree_cases_follow_decoded_scalars_with_ordered_contributions() {
    let analysis = analyze("<body>a&#38;b</body>");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["a&b".to_owned()]);
    assert_eq!(
        contributions(&analysis),
        expect(&[((6, 7), "a"), ((7, 12), "&"), ((12, 13), "b")])
    );

    let analysis = analyze("<body>&#9;</body>");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["\t".to_owned()]);
    assert_eq!(contributions(&analysis), expect(&[((6, 10), "\t")]));

    // After `</body>`: decoded LF is whitespace (no after-body diagnostic);
    // decoded U+00A0 is not whitespace (after-body diagnostic), whatever the
    // authored spelling.
    let analysis = analyze("<body></body>&#10;");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["\n".to_owned()]);
    assert!(!tree_codes(&analysis).contains(&HtmlTreeDiagnosticCode::AfterBodyCharacterData));

    let analysis = analyze("<body></body>&#160;");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["\u{a0}".to_owned()]);
    assert!(tree_codes(&analysis).contains(&HtmlTreeDiagnosticCode::AfterBodyCharacterData));

    // The decoded scalar, not the spelling, decides: a literal LF and `&#10;`
    // behave alike, a literal NBSP and `&#160;` behave alike.
    assert_eq!(
        tree_codes(&analyze("<body></body>\n")),
        tree_codes(&analyze("<body></body>&#10;"))
    );
    assert_eq!(
        tree_codes(&analyze("<body></body>\u{a0}")),
        tree_codes(&analyze("<body></body>&#160;"))
    );

    let analysis = analyze("<title>&#38;</title>");
    assert!(analysis.is_complete());
    assert_eq!(texts(&analysis), vec!["&".to_owned()]);
    assert_eq!(contributions(&analysis), expect(&[((7, 12), "&")]));
}

// ---------------------------------------------------------------------------
// NR9 - resources
// ---------------------------------------------------------------------------

fn steps_only(steps: usize) -> Lim {
    Lim { steps, ..GENEROUS }
}

/// Falsifies: a Numeric reference made atomic across its digit run, a lost
/// committed prefix, a prematurely entered End, and an upgraded higher layer.
#[test]
fn nr9_steps_refusal_inside_a_digit_run_keeps_honest_partial_progress() {
    // Steps 1..5 commit: `&`, `#`, Numeric(1), Decimal(1), Decimal(2). The
    // sixth (Decimal(3)) is refused with `&#12` already consumed.
    let run = lex_with("&#1234;", steps_only(5));
    assert_eq!(
        refusal(&run),
        Some((HtmlTokenizerResource::TransitionSteps, 5, 6))
    );
    assert_eq!(steps(&run), 5);
    assert_eq!(processed(&run), 4);
    assert!(run.tokens().is_empty(), "no decoded output");
    assert!(run.diagnostics().is_empty(), "End was never entered");

    let analysis = analyze_with("<body>&#1234;", steps_only(5 + 6 + 1));
    assert!(!analysis.is_complete(), "the higher layer stays incomplete");
    assert!(texts(&analysis).is_empty());
}

/// Falsifies: End charged zero steps, folded into the previous transition, or
/// charged twice.
#[test]
fn nr10_numeric_end_costs_exactly_one_step() {
    // `&#65;` has 8 steps: End is the 7th.
    let run = lex_with("&#65;", steps_only(6));
    assert_eq!(
        refusal(&run),
        Some((HtmlTokenizerResource::TransitionSteps, 6, 7))
    );
    assert_eq!(steps(&run), 6);
    assert!(run.tokens().is_empty(), "End refused: no decoded output");
    assert!(
        run.diagnostics().is_empty(),
        "End refused: no End diagnostic"
    );
    // The `;` was examined and consumed by a committed transition.
    assert_eq!(processed(&run), 5);

    let run = lex_with("&#65;", steps_only(7));
    assert_eq!(
        refusal(&run),
        Some((HtmlTokenizerResource::TransitionSteps, 7, 8))
    );
    assert_eq!(steps(&run), 7);
    assert_eq!(chars(&run), expect(&[((0, 5), "A")]), "End committed");
    assert_eq!(processed(&run), 5);

    let run = lex_with("&#65;", steps_only(8));
    assert!(complete(&run));
    assert_eq!(steps(&run), 8);
}

/// Falsifies: `;` coverage lost when End later refuses, and the semicolonless
/// terminator's prior evidence lost when End refuses.
#[test]
fn nr11_end_step_refusal_preserves_prior_committed_evidence() {
    // Semicolon: coverage includes `;` (0..5), nothing else exists.
    let run = lex_with("&#65;", steps_only(6));
    assert_eq!(processed(&run), 5);
    assert!(run.tokens().is_empty() && run.diagnostics().is_empty());

    // Semicolonless: the missing-semicolon observation at `x` survives; the
    // reference output and any End diagnostic do not; the terminator is not
    // reconsumed because End never succeeded.
    let run = lex_with("&#65x", steps_only(6));
    assert_eq!(
        refusal(&run),
        Some((HtmlTokenizerResource::TransitionSteps, 6, 7))
    );
    assert_eq!(diags(&run), vec![(MISSING, (4, 5))]);
    assert!(run.tokens().is_empty());
    assert_eq!(processed(&run), 5);
    assert_diagnostics_within_coverage(&run, "&#65x");

    // EOF terminator: a zero-width site.
    let run = lex_with("&#65", steps_only(6));
    assert_eq!(diags(&run), vec![(MISSING, (4, 4))]);
    assert!(run.tokens().is_empty());
    assert_eq!(processed(&run), 4);

    // End diagnostic never fabricated for a refused End (`&#0` would carry one).
    // `&#0x`: Data(&) CharRef(#) Numeric(0) Decimal(0) Decimal(x) = 5, End = 6.
    let run = lex_with("&#0x", steps_only(5));
    assert_eq!(diags(&run), vec![(MISSING, (3, 4))]);
    assert!(run.tokens().is_empty());
}

/// Falsifies: the terminator's preprocessing diagnostic lost or reordered when
/// the next transition is refused (the accepted #878 remediation theorem).
#[test]
fn nr12_preprocessing_survives_a_refused_terminator_step() {
    let run = lex_with("&#65\u{1}", steps_only(5));
    assert_eq!(
        refusal(&run),
        Some((HtmlTokenizerResource::TransitionSteps, 5, 6))
    );
    assert_eq!(diags(&run), vec![(INPUT_CONTROL, (4, 5))]);
    assert_eq!(processed(&run), 5, "coverage includes the U+0001 site");
    assert!(run.tokens().is_empty(), "no Numeric output");
    assert!(
        !diags(&run).iter().any(|(code, _)| *code == MISSING),
        "the terminator transition never started"
    );
    assert_diagnostics_within_coverage(&run, "&#65<U+0001>");
}

/// Falsifies: a terminator transition that runs although its preprocessing
/// diagnostic was refused.
#[test]
fn nr13_preprocessing_diagnostic_refusal_prevents_the_terminator_transition() {
    let limits = Lim {
        diagnostics: 0,
        ..GENEROUS
    };
    let run = lex_with("&#65\u{1}", limits);
    assert_eq!(
        refusal(&run),
        Some((HtmlTokenizerResource::Diagnostics, 0, 1))
    );
    assert!(run.diagnostics().is_empty());
    assert!(run.tokens().is_empty());
    assert_eq!(steps(&run), 5, "the terminator transition never started");
    assert_eq!(processed(&run), 4, "coverage excludes the refused site");
}

/// Falsifies: an End diagnostic committed without End output, and End output
/// committed without its required End diagnostic.
#[test]
fn nr14_end_diagnostic_and_token_commit_together_or_not_at_all() {
    // Diagnostic capacity: the End step committed (6), then the compound
    // effect refused: no decoded character, no diagnostic.
    let no_diag = Lim {
        diagnostics: 0,
        ..GENEROUS
    };
    let run = lex_with("&#0;", no_diag);
    assert_eq!(
        refusal(&run),
        Some((HtmlTokenizerResource::Diagnostics, 0, 1))
    );
    assert_eq!(steps(&run), 6);
    assert!(run.tokens().is_empty() && run.diagnostics().is_empty());
    assert_eq!(processed(&run), 4);

    // Semicolonless: the earlier missing-semicolon diagnostic is prior
    // evidence; the End diagnostic and output are absent.
    let one_diag = Lim {
        diagnostics: 1,
        ..GENEROUS
    };
    let run = lex_with("&#0x", one_diag);
    assert_eq!(
        refusal(&run),
        Some((HtmlTokenizerResource::Diagnostics, 1, 2))
    );
    assert_eq!(steps(&run), 6);
    assert_eq!(diags(&run), vec![(MISSING, (3, 4))]);
    assert!(run.tokens().is_empty());

    // Token capacity. A zero token limit is an invalid configuration, so a
    // prior run flushed at `#` consumes the whole budget and the reference's
    // own token is the refused second one.
    let one_token = Lim {
        tokens: 1,
        ..GENEROUS
    };
    for text in ["a&#0;", "a&#65;", "a&#128;", "a&#0x"] {
        let run = lex_with(text, one_token);
        assert_eq!(
            refusal(&run),
            Some((HtmlTokenizerResource::EmittedTokens, 1, 2)),
            "{text:?}"
        );
        assert_eq!(chars(&run), expect(&[((0, 1), "a")]), "{text:?}");
        assert!(
            diags(&run).iter().all(|(code, _)| *code == MISSING),
            "{text:?}: no End diagnostic without the End output"
        );
    }
}

/// Falsifies: partial finalization on a retained-byte refusal, and a wrong
/// per-output byte cost.
#[test]
fn nr15_retained_byte_refusal_commits_no_output_or_diagnostic() {
    // (source, bytes of the decoded output)
    for (text, bytes) in [
        ("&#65;", 1usize),
        ("&#0;", 3),
        ("&#128;", 3),
        ("&#x1F600;", 4),
    ] {
        let tight = Lim {
            retained: bytes - 1,
            ..GENEROUS
        };
        let run = lex_with(text, tight);
        assert_eq!(
            refusal(&run),
            Some((
                HtmlTokenizerResource::RetainedInterpretedBytes,
                bytes - 1,
                bytes
            )),
            "{text:?}"
        );
        assert!(run.tokens().is_empty(), "{text:?}");
        assert!(run.diagnostics().is_empty(), "{text:?}");

        let exact = Lim {
            retained: bytes,
            ..GENEROUS
        };
        let run = lex_with(text, exact);
        assert!(complete(&run), "{text:?} fits exactly");
        assert_eq!(run.usage().retained_interpreted_bytes(), bytes, "{text:?}");
    }
}

/// Falsifies: no-digits recovery that commits the diagnostic without the
/// literal prefix, or the prefix without the diagnostic.
#[test]
fn nr16_no_digits_recovery_is_atomic() {
    for text in ["&#;", "&#xZ", "&#A", "&#"] {
        let prefix = if text.starts_with("&#x") { 3 } else { 2 };

        let no_diag = Lim {
            diagnostics: 0,
            ..GENEROUS
        };
        let run = lex_with(text, no_diag);
        assert_eq!(
            refusal(&run),
            Some((HtmlTokenizerResource::Diagnostics, 0, 1)),
            "{text:?}"
        );
        assert!(
            run.tokens().is_empty(),
            "{text:?}: no prefix without its diagnostic"
        );
        assert!(run.diagnostics().is_empty(), "{text:?}");
        assert_eq!(processed(&run), prefix, "{text:?}");

        // A prior run flushed at `#` uses the whole one-token budget, so the
        // prefix token is the refused second token.
        let one_token = Lim {
            tokens: 1,
            ..GENEROUS
        };
        let prefixed = format!("a{text}");
        let run = lex_with(&prefixed, one_token);
        assert_eq!(
            refusal(&run),
            Some((HtmlTokenizerResource::EmittedTokens, 1, 2)),
            "{prefixed:?}"
        );
        assert_eq!(chars(&run), expect(&[((0, 1), "a")]), "{prefixed:?}");
        assert!(
            run.diagnostics().is_empty(),
            "{prefixed:?}: no diagnostic without its prefix"
        );

        let tight = Lim {
            retained: prefix - 1,
            ..GENEROUS
        };
        let run = lex_with(text, tight);
        assert_eq!(
            refusal(&run),
            Some((
                HtmlTokenizerResource::RetainedInterpretedBytes,
                prefix - 1,
                prefix
            )),
            "{text:?}"
        );
        assert!(
            run.tokens().is_empty() && run.diagnostics().is_empty(),
            "{text:?}"
        );
    }
}

/// Falsifies: Numeric breaking the existing resource model or leaving a
/// temporary buffer.
#[test]
fn nr17_no_new_resource_dimension_and_zero_temporary_buffer() {
    assert_eq!(HtmlTokenizerResource::ALL.len(), 7);
    for text in [
        "&#65;",
        "&#x41",
        "&#;",
        "&#0;",
        "&#99999999999999999999;",
        "&#xZ",
    ] {
        let run = lex(text);
        assert_eq!(run.usage().peak_temporary_buffer_bytes(), 0, "{text:?}");
    }
}

/// Falsifies: any committed Numeric diagnostic ending past processed coverage,
/// on complete and refused runs alike.
#[test]
fn nr18_every_numeric_diagnostic_is_inside_coverage() {
    let sources = [
        "&#;",
        "&#A",
        "&#x;",
        "&#xZ",
        "&#",
        "&#x",
        "&#0;",
        "&#0x",
        "&#0",
        "&#xD800;",
        "&#1114112;",
        "&#xFDD0;",
        "&#128;",
        "&#13;",
        "&#65x",
        "&#65",
        "&#65\u{1}",
        "&#65\r\n",
        "&#xFDD0\u{85}",
    ];
    for text in sources {
        let run = lex(text);
        assert_diagnostics_within_coverage(&run, text);
        for limit in 0..=run.usage().transition_steps() {
            let refused = lex_with(text, steps_only(limit));
            assert_diagnostics_within_coverage(&refused, text);
        }
        for limit in 0..4 {
            let refused = lex_with(
                text,
                Lim {
                    diagnostics: limit,
                    ..GENEROUS
                },
            );
            assert_diagnostics_within_coverage(&refused, text);
        }
        // Nondecreasing starts: the diagnostic contract.
        let starts: Vec<usize> = diags(&run).iter().map(|(_, (start, _))| *start).collect();
        assert!(starts.windows(2).all(|pair| pair[0] <= pair[1]), "{text:?}");
    }
}

/// Falsifies: the reference span derived from coverage, which a semicolonless
/// terminator diagnostic advances past the reference.
#[test]
fn nr19_reference_span_ends_at_the_last_consumed_prefix_unit() {
    for (text, end) in [
        ("&#65x", 4usize),
        ("&#65\u{1}", 4),
        ("&#x41g", 5),
        ("&#0x", 3),
        ("&#65 ", 4),
    ] {
        let run = lex(text);
        let (range, _) = chars(&run).remove(0);
        assert_eq!(
            range,
            (0, end),
            "{text:?}: the terminator is not part of it"
        );
        // The terminator advanced coverage past the reference, as designed.
        assert!(processed(&run) >= end);
    }
    // With a semicolon the span includes it.
    assert_eq!(chars(&lex("&#65;")).remove(0).0, (0, 5));
}

/// Falsifies: the retired Numeric-unsupported refusals returning for the
/// selected paths.
#[test]
fn nr20_boundaries_after_numeric_support() {
    for text in ["&#65;", "<body>&#65;", "<title>&#65;</title>"] {
        let source = SourceText::new(SourceId::new(1), text.to_owned());
        let run = tokenize(&source, GENEROUS.limits());
        if text.starts_with("<title>") {
            // Standalone, `<title>` is exactly as deferred as before; only the
            // coordinated analysis selects RCDATA.
            assert!(unsupported(&run));
            continue;
        }
        assert!(complete(&run), "{text:?}");
    }
}
