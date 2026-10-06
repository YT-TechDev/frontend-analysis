//! Production correspondence for the SelectedOrdinary AttributeValue character
//! reference successor (Issue #906).
//!
//! Expectations here are authored by hand from #906, the #348 focused
//! placement comment `6010078447`, and the accepted #904 semantic theorem,
//! with hand-counted source offsets. They are never produced by production
//! output, a browser, or an external parser, and this module deliberately does
//! not import the candidate-independent #904 `Machine` in the sibling
//! `in_body_selected_ordinary_attribute_value_reference_successor_validation`
//! module.
//!
//! Every fixture shares one layout so offsets are checkable by eye:
//!
//! ```text
//! <body><div id="VALUE"></div>
//! 0     6     11 14 15
//!         ^ `<` at 6, `div` 7..10, `id` 11..13, `=` 13..14,
//!           open quote 14..15, value starts at 15 (14 when unquoted)
//! ```
//!
//! Each test names the incorrect implementation it rejects.

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::{HtmlAttributeValueSyntax as TokenSyntax, HtmlTagKind, HtmlToken};
use super::super::tokenizer::diagnostic::{
    HtmlTokenizerDiagnosticCode as Diag, HtmlTokenizerDiagnosticContext as Ctx,
    HtmlTokenizerDiagnosticSubject as Subject,
};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerResource as Resource};
use super::super::tokenizer::result::{
    HtmlTokenizerCapability, HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause,
    HtmlTokenizerRunResult,
};
use super::super::tree::{
    HtmlTreeAttributeValueSyntax, HtmlTreeCompletion, HtmlTreeIncompleteCause, HtmlTreeReport,
    HtmlTreeResourceKind, HtmlTreeUnsupportedCapability, analyze_selected_document_tree,
};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

const SOURCE: u64 = 7;
const PRODUCT_SOURCE_BYTES: usize = 36_864;
const PRODUCT_STEPS: usize = 79_872;
const PRODUCT_RETAINED: usize = 49_152;

/// The fixed Product vector with the three limits a test may vary.
fn limits(steps: usize, diagnostics: usize, retained: usize) -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(
        PRODUCT_SOURCE_BYTES,
        steps,
        6_144,
        diagnostics,
        1,
        retained,
        0,
    )
}

fn product() -> HtmlTokenizerLimits {
    limits(PRODUCT_STEPS, 256, PRODUCT_RETAINED)
}

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(SOURCE), text.to_owned())
}

fn lex_with(text: &str, limits: HtmlTokenizerLimits) -> HtmlTokenizerRunResult {
    tokenize(&source(text), limits)
}

fn lex(text: &str) -> HtmlTokenizerRunResult {
    lex_with(text, product())
}

fn analyze(text: &str) -> HtmlTreeReport {
    analyze_selected_document_tree(&source(text)).expect("analysis returns a report")
}

type Span = (usize, usize);

fn span(anchor: &SourceAnchor) -> Span {
    assert_eq!(anchor.source_id(), SourceId::new(SOURCE), "SourceId");
    (anchor.range().start(), anchor.range().end())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Syntax {
    Double {
        equals: Span,
        open: Span,
        value: Span,
        close: Span,
    },
    Single {
        equals: Span,
        open: Span,
        value: Span,
        close: Span,
    },
    Unquoted {
        equals: Span,
        value: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Attr {
    complete: Span,
    name: Span,
    syntax: Syntax,
    interpreted_name: String,
    interpreted_value: String,
}

fn token_attribute(run: &HtmlTokenizerRunResult, token_index: usize) -> Attr {
    let Some(HtmlToken::Tag(tag)) = run.tokens().get(token_index) else {
        panic!("token {token_index} is not a tag: {:?}", run.tokens());
    };
    assert_eq!(tag.attributes().len(), 1, "exactly one authored attribute");
    let attribute = &tag.attributes()[0];
    let syntax = match attribute.value_syntax() {
        TokenSyntax::DoubleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => Syntax::Double {
            equals: span(equals),
            open: span(open_quote),
            value: span(value),
            close: span(close_quote),
        },
        TokenSyntax::SingleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => Syntax::Single {
            equals: span(equals),
            open: span(open_quote),
            value: span(value),
            close: span(close_quote),
        },
        TokenSyntax::Unquoted { equals, value } => Syntax::Unquoted {
            equals: span(equals),
            value: span(value),
        },
        other => panic!("unexpected value syntax {other:?}"),
    };
    Attr {
        complete: span(attribute.complete()),
        name: span(attribute.name().source()),
        syntax,
        interpreted_name: attribute.name().interpreted().to_owned(),
        interpreted_value: attribute.interpreted_value().to_owned(),
    }
}

fn report_attribute(report: &HtmlTreeReport) -> Attr {
    let rows = report.selected_ordinary_attributes();
    assert_eq!(rows.len(), 1, "one projected attribute row");
    let row = &rows[0];
    let syntax = match row.value_syntax() {
        HtmlTreeAttributeValueSyntax::DoubleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => Syntax::Double {
            equals: span(equals),
            open: span(open_quote),
            value: span(value),
            close: span(close_quote),
        },
        HtmlTreeAttributeValueSyntax::SingleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => Syntax::Single {
            equals: span(equals),
            open: span(open_quote),
            value: span(value),
            close: span(close_quote),
        },
        HtmlTreeAttributeValueSyntax::Unquoted { equals, value } => Syntax::Unquoted {
            equals: span(equals),
            value: span(value),
        },
        other => panic!("unexpected projected syntax {other:?}"),
    };
    Attr {
        complete: span(row.complete()),
        name: span(row.authored_name()),
        syntax,
        interpreted_name: row.interpreted_name().to_owned(),
        interpreted_value: row.interpreted_value().to_owned(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quote {
    Double,
    Single,
    None,
}

const QUOTES: [Quote; 3] = [Quote::Double, Quote::Single, Quote::None];

/// `<body><div id=VALUE></div>` in the given authored syntax. When
/// `closed` is false the end tag is omitted, which keeps the retained-byte
/// arithmetic of the resource fixtures to the start tag alone.
fn fixture(quote: Quote, raw: &str, closed: bool) -> String {
    let tail = if closed { "</div>" } else { "" };
    match quote {
        Quote::Double => format!("<body><div id=\"{raw}\">{tail}"),
        Quote::Single => format!("<body><div id='{raw}'>{tail}"),
        Quote::None => format!("<body><div id={raw}>{tail}"),
    }
}

/// Where the authored value starts in [`fixture`].
fn value_start(quote: Quote) -> usize {
    match quote {
        Quote::Double | Quote::Single => 15,
        Quote::None => 14,
    }
}

/// The exact expected attribute evidence for an authored value of `len` bytes.
fn expected_attr(quote: Quote, len: usize, interpreted: &str) -> Attr {
    let (complete_end, syntax) = match quote {
        Quote::Double => (
            16 + len,
            Syntax::Double {
                equals: (13, 14),
                open: (14, 15),
                value: (15, 15 + len),
                close: (15 + len, 16 + len),
            },
        ),
        Quote::Single => (
            16 + len,
            Syntax::Single {
                equals: (13, 14),
                open: (14, 15),
                value: (15, 15 + len),
                close: (15 + len, 16 + len),
            },
        ),
        Quote::None => (
            14 + len,
            Syntax::Unquoted {
                equals: (13, 14),
                value: (14, 14 + len),
            },
        ),
    };
    Attr {
        complete: (11, complete_end),
        name: (11, 13),
        syntax,
        interpreted_name: "id".to_owned(),
        interpreted_value: interpreted.to_owned(),
    }
}

type DiagRow = (Diag, Ctx, Span);

fn diag_rows(run: &HtmlTokenizerRunResult) -> Vec<DiagRow> {
    run.diagnostics()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.code(),
                diagnostic.context(),
                span(diagnostic.location()),
            )
        })
        .collect()
}

fn complete(run: &HtmlTokenizerRunResult) -> bool {
    matches!(run.completion(), HtmlTokenizerCompletion::Complete)
}

/// `(resource, limit, attempted, anchor start)` of a resource-limited run.
fn refusal(run: &HtmlTokenizerRunResult) -> Option<(Resource, usize, usize, usize)> {
    match run.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(limit)) => {
            Some((
                limit.resource(),
                limit.limit(),
                limit.attempted(),
                limit.at().range().start(),
            ))
        }
        _ => None,
    }
}

fn tag_count(run: &HtmlTokenizerRunResult, kind: HtmlTagKind) -> usize {
    run.tokens()
        .iter()
        .filter(|token| matches!(token, HtmlToken::Tag(tag) if tag.kind() == kind))
        .count()
}

fn tree_capability(report: &HtmlTreeReport) -> Option<HtmlTreeUnsupportedCapability> {
    match report.completion() {
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(unsupported)) => {
            Some(unsupported.capability())
        }
        _ => None,
    }
}

fn is_complete_tree(report: &HtmlTreeReport) -> bool {
    matches!(report.completion(), HtmlTreeCompletion::Complete)
}

// ---------------------------------------------------------------------------
// The reference matrix
// ---------------------------------------------------------------------------

const NAMED: Ctx = Ctx::NamedCharacterReference;
const AMBIGUOUS: Ctx = Ctx::AmbiguousAmpersand;
const DECIMAL: Ctx = Ctx::DecimalCharacterReference;
const HEX: Ctx = Ctx::HexadecimalCharacterReference;
const END: Ctx = Ctx::NumericCharacterReferenceEnd;
const NUMERIC: Ctx = Ctx::NumericCharacterReference;
const HEX_START: Ctx = Ctx::HexadecimalCharacterReferenceStart;

/// `(raw authored value, expected interpreted value, expected diagnostics as
/// (code, context, start, end) relative to the first authored value byte)`.
/// Every raw value is free of whitespace, quotes, `<`, `=`, backtick and `>`,
/// so the same row is valid in all three authored syntaxes.
type Row = (
    &'static str,
    &'static str,
    &'static [(Diag, Ctx, usize, usize)],
);

const MATRIX: &[Row] = &[
    // Named, with a semicolon.
    ("&amp;", "&", &[]),
    ("&lt;b&gt;", "<b>", &[]),
    ("&copy;x", "\u{a9}x", &[]),
    // Maximum match: `notin;` wins over the shorter `not`.
    ("&notin;", "\u{2209}", &[]),
    // One canonical identifier decoding to two scalars.
    ("&NotEqualTilde;", "\u{2242}\u{338}", &[]),
    // Prefix, reference, suffix.
    ("a&lt;b", "a<b", &[]),
    // Several references in the same one attribute.
    ("&lt;&amp;&#x41;&gt;", "<&A>", &[]),
    // Semicolonless and resolving: decoded, and the diagnostic is reported at
    // the last matched byte.
    (
        "&amp",
        "&",
        &[(Diag::MissingSemicolonAfterCharacterReference, NAMED, 3, 4)],
    ),
    // Semicolonless but followed by an ASCII alphanumeric: literal, silent.
    ("&notit;", "&notit;", &[]),
    ("&ampx", "&ampx", &[]),
    ("&amp1", "&amp1", &[]),
    // No Named match: ambiguous ampersand.
    (
        "&zz;",
        "&zz;",
        &[(Diag::UnknownNamedCharacterReference, AMBIGUOUS, 3, 4)],
    ),
    ("&zz", "&zz", &[]),
    // `&` followed by nothing that starts a reference.
    ("&;", "&;", &[]),
    ("a&", "a&", &[]),
    // Numeric.
    ("&#65;", "A", &[]),
    ("&#x41;", "A", &[]),
    ("&#X1f600;", "\u{1f600}", &[]),
    (
        "&#65",
        "A",
        &[(Diag::MissingSemicolonAfterCharacterReference, DECIMAL, 4, 5)],
    ),
    (
        "&#x41",
        "A",
        &[(Diag::MissingSemicolonAfterCharacterReference, HEX, 5, 6)],
    ),
    // Numeric recovery.
    (
        "&#0;",
        "\u{fffd}",
        &[(Diag::NullCharacterReference, END, 3, 4)],
    ),
    (
        "&#0",
        "\u{fffd}",
        &[
            (Diag::MissingSemicolonAfterCharacterReference, DECIMAL, 3, 4),
            (Diag::NullCharacterReference, END, 3, 4),
        ],
    ),
    (
        "&#xD800;",
        "\u{fffd}",
        &[(Diag::SurrogateCharacterReference, END, 7, 8)],
    ),
    (
        "&#x110000;",
        "\u{fffd}",
        &[(Diag::CharacterReferenceOutsideUnicodeRange, END, 9, 10)],
    ),
    // Twenty nines saturate; a wrapping accumulator would not stay out of range.
    (
        "&#99999999999999999999;",
        "\u{fffd}",
        &[(Diag::CharacterReferenceOutsideUnicodeRange, END, 22, 23)],
    ),
    (
        "&#x80;",
        "\u{20ac}",
        &[(Diag::ControlCharacterReference, END, 5, 6)],
    ),
    (
        "&#x81;",
        "\u{81}",
        &[(Diag::ControlCharacterReference, END, 5, 6)],
    ),
    (
        "&#x9F;",
        "\u{178}",
        &[(Diag::ControlCharacterReference, END, 5, 6)],
    ),
    (
        "&#13;",
        "\r",
        &[(Diag::ControlCharacterReference, END, 4, 5)],
    ),
    (
        "&#xFDD0;",
        "\u{fdd0}",
        &[(Diag::NoncharacterCharacterReference, END, 7, 8)],
    ),
    // No digit: the recognized prefix stays literal.
    (
        "&#;",
        "&#;",
        &[(
            Diag::AbsenceOfDigitsInNumericCharacterReference,
            NUMERIC,
            2,
            3,
        )],
    ),
    (
        "&#",
        "&#",
        &[(
            Diag::AbsenceOfDigitsInNumericCharacterReference,
            NUMERIC,
            2,
            3,
        )],
    ),
    (
        "&#x;",
        "&#x;",
        &[(
            Diag::AbsenceOfDigitsInNumericCharacterReference,
            HEX_START,
            3,
            4,
        )],
    ),
    (
        "&#Xz",
        "&#Xz",
        &[(
            Diag::AbsenceOfDigitsInNumericCharacterReference,
            HEX_START,
            3,
            4,
        )],
    ),
];

fn expected_diags(quote: Quote, rows: &[(Diag, Ctx, usize, usize)]) -> Vec<DiagRow> {
    let base = value_start(quote);
    rows.iter()
        .map(|&(code, context, start, end)| (code, context, (base + start, base + end)))
        .collect()
}

/// Checks one authored value end to end: tokenizer evidence, ordered
/// diagnostics and their subject, the report projection, and the unchanged
/// shape of everything around the attribute.
fn check_value(
    quote: Quote,
    raw: &str,
    interpreted: &str,
    diags: &[(Diag, Ctx, usize, usize)],
    label: &str,
) {
    let text = fixture(quote, raw, true);
    let run = lex(&text);
    assert!(complete(&run), "{label}: {text:?} {:?}", run.completion());
    assert_eq!(
        run.tokens().len(),
        4,
        "{label}: body start, div start, div end, end of file"
    );
    assert_eq!(
        token_attribute(&run, 1),
        expected_attr(quote, raw.len(), interpreted),
        "{label}: {text:?}"
    );
    assert_eq!(diag_rows(&run), expected_diags(quote, diags), "{label}");
    for diagnostic in run.diagnostics() {
        assert!(
            matches!(
                diagnostic.subject(),
                Subject::EmittedToken { token_index: 1 }
            ),
            "{label}: a reference diagnostic belongs to the div start tag, got {:?}",
            diagnostic.subject()
        );
    }
    assert!(
        run.tokens()
            .iter()
            .all(|token| !matches!(token, HtmlToken::Character(_))),
        "{label}: reference output never becomes a document Character token"
    );

    let report = analyze(&text);
    assert!(
        is_complete_tree(&report),
        "{label}: {:?}",
        report.completion()
    );
    assert_eq!(
        report_attribute(&report),
        expected_attr(quote, raw.len(), interpreted),
        "{label}: projected row"
    );
    assert_eq!(
        report.selected_ordinary_attributes()[0].node().value(),
        4,
        "{label}: the attribute belongs to the constructed div"
    );
    assert_eq!(
        report.tokenizer_diagnostics().len(),
        diags.len(),
        "{label}: tokenizer diagnostics stay tokenizer-owned"
    );
}

/// Falsifies: a wrong AttributeValue return variant, output routed to
/// Character tokens, a missing exception, shortest-match, a missing diagnostic,
/// a swallowed delimiter, partial multi-scalar output, numeric wrap, a missing
/// C1 remap, and a corrupted authored value endpoint — in each of the three
/// originating syntaxes.
#[test]
fn av1_reference_matrix_matches_hand_authored_expectations_in_all_three_syntaxes() {
    for quote in QUOTES {
        for &(raw, interpreted, diags) in MATRIX {
            check_value(quote, raw, interpreted, diags, &format!("{quote:?}"));
        }
    }
}

/// Falsifies: a quoted state returning to the unquoted state after a
/// reference, and the originating state inferred from anything but the
/// recorded return variant.
#[test]
fn av2_each_quoted_state_returns_to_itself_and_keeps_its_other_quote() {
    // After `&amp;` a double-quoted value goes on: the space and `'` are
    // ordinary value bytes, which an unquoted return state would not allow.
    let run = lex("<body><div id=\"&amp; x'y\"></div>");
    assert!(complete(&run));
    assert_eq!(
        token_attribute(&run, 1),
        expected_attr(Quote::Double, 9, "& x'y")
    );
    assert!(run.diagnostics().is_empty());

    let run = lex("<body><div id='&amp; x\"y'></div>");
    assert!(complete(&run));
    assert_eq!(
        token_attribute(&run, 1),
        expected_attr(Quote::Single, 9, "& x\"y")
    );
    assert!(run.diagnostics().is_empty());

    // The other quote is the delimiter only of its own state.
    let run = lex("<body><div id=\"&amp'\"></div>");
    assert!(complete(&run));
    assert_eq!(
        token_attribute(&run, 1),
        expected_attr(Quote::Double, 5, "&'"),
        "semicolonless `&amp` before `'` inside double quotes"
    );
    assert_eq!(
        diag_rows(&run),
        vec![(
            Diag::MissingSemicolonAfterCharacterReference,
            NAMED,
            (18, 19)
        )]
    );
}

/// Falsifies: a swallowed delimiter and an unquoted return state that does
/// not end the value at the exact authored terminator.
#[test]
fn av3_unquoted_references_end_at_the_exact_terminator_and_never_swallow_it() {
    // `>` terminator after a reference.
    let run = lex("<body><div id=&lt;></div>");
    assert!(complete(&run));
    assert_eq!(token_attribute(&run, 1), expected_attr(Quote::None, 4, "<"));
    let Some(HtmlToken::Tag(tag)) = run.tokens().get(1) else {
        panic!("div start tag");
    };
    assert_eq!(span(tag.complete()), (6, 19), "the `>` closes the tag");

    // Whitespace terminator after a semicolonless reference: the space is not
    // consumed by the reference, and the tag still closes afterwards.
    let run = lex("<body><div id=&amp ></div>");
    assert!(complete(&run));
    assert_eq!(token_attribute(&run, 1), expected_attr(Quote::None, 4, "&"));
    assert_eq!(
        diag_rows(&run),
        vec![(
            Diag::MissingSemicolonAfterCharacterReference,
            NAMED,
            (17, 18)
        )]
    );
}

/// Falsifies: the unquoted-attribute rules being lost after a blocked
/// literal, and the `=` follower being consumed with the literal.
#[test]
fn av4_blocked_by_equals_stays_literal_and_leaves_the_equals_to_its_own_state() {
    for quote in [Quote::Double, Quote::Single] {
        let text = fixture(quote, "&amp=1", true);
        let run = lex(&text);
        assert!(complete(&run), "{quote:?}");
        assert_eq!(
            token_attribute(&run, 1),
            expected_attr(quote, 6, "&amp=1"),
            "{quote:?}"
        );
        assert!(run.diagnostics().is_empty(), "{quote:?}: no diagnostic");
    }
    // Unquoted, the `=` is then judged by the originating unquoted state.
    let run = lex("<body><div id=&amp=1></div>");
    assert!(complete(&run));
    assert_eq!(
        token_attribute(&run, 1),
        expected_attr(Quote::None, 6, "&amp=1")
    );
    assert_eq!(
        diag_rows(&run),
        vec![(
            Diag::UnexpectedCharacterInUnquotedAttributeValue,
            Ctx::AttributeValueUnquoted,
            (18, 19)
        )]
    );
}

/// Falsifies: the attribute-only exception applied outside attributes, or
/// omitted: the same `&notit;` is literal in an attribute and resolved in Data.
#[test]
fn av5_the_semicolonless_exception_is_attribute_only() {
    let run = lex("&notit;<div id=\"&notit;\">");
    assert!(complete(&run));
    let characters: Vec<(Span, String)> = run
        .tokens()
        .iter()
        .filter_map(|token| match token {
            HtmlToken::Character(character) => {
                Some((span(character.source()), character.interpreted().to_owned()))
            }
            _ => None,
        })
        .collect();
    // Data resolves the maximum match `not` and leaves `it;` as ordinary text.
    assert_eq!(
        characters,
        vec![((0, 4), "\u{ac}".to_owned()), ((4, 7), "it;".to_owned())]
    );
    // The attribute keeps the very same spelling literally.
    // `&notit;` is 0..7, so `<div id="` starts at 7: `id` 12..14, `=` 14..15,
    // open quote 15..16, value 16..23, close quote 23..24.
    assert_eq!(
        token_attribute(&run, 2),
        Attr {
            complete: (12, 24),
            name: (12, 14),
            syntax: Syntax::Double {
                equals: (14, 15),
                open: (15, 16),
                value: (16, 23),
                close: (23, 24),
            },
            interpreted_name: "id".to_owned(),
            interpreted_value: "&notit;".to_owned(),
        }
    );
    // Data raises the missing-semicolon diagnostic for the resolved `not`;
    // the blocked attribute match raises nothing.
    assert_eq!(
        diag_rows(&run),
        vec![(Diag::MissingSemicolonAfterCharacterReference, NAMED, (3, 4))]
    );
}

/// Falsifies: the missing-semicolon diagnostic emitted for a blocked match,
/// and a diagnostic subject that leaves the tag.
#[test]
fn av6_a_diagnosed_reference_inside_one_attribute_is_ordered_by_source() {
    // `&amp` resolves (a `&` follows), then `&zz;` is unknown.
    let run = lex("<body><div id=\"&amp&zz;\"></div>");
    assert!(complete(&run));
    assert_eq!(
        token_attribute(&run, 1),
        expected_attr(Quote::Double, 8, "&&zz;")
    );
    assert_eq!(
        diag_rows(&run),
        vec![
            (
                Diag::MissingSemicolonAfterCharacterReference,
                NAMED,
                (18, 19)
            ),
            (Diag::UnknownNamedCharacterReference, AMBIGUOUS, (22, 23)),
        ]
    );
}

// ---------------------------------------------------------------------------
// Authored evidence and the numeric / raw-NUL distinction
// ---------------------------------------------------------------------------

/// Falsifies: any attribute anchor, value syntax, or SourceId derived from
/// the decoded value, and an authored value that is not the exact source.
#[test]
fn av7_authored_anchors_are_exact_and_independent_of_decoding() {
    // `<body>`0..6 `<div id="a&amp;b">`6..24: `a&amp;b` is 7 bytes, 15..22.
    let text = "<body><div id=\"a&amp;b\"></div>";
    let run = lex(text);
    assert_eq!(
        token_attribute(&run, 1),
        Attr {
            complete: (11, 23),
            name: (11, 13),
            syntax: Syntax::Double {
                equals: (13, 14),
                open: (14, 15),
                value: (15, 22),
                close: (22, 23),
            },
            interpreted_name: "id".to_owned(),
            interpreted_value: "a&b".to_owned(),
        },
        "decoded length 3, authored length 7"
    );
    let report = analyze(text);
    assert!(is_complete_tree(&report));
    let row = &report.selected_ordinary_attributes()[0];
    assert_eq!(row.interpreted_value(), "a&b");
    let HtmlTreeAttributeValueSyntax::DoubleQuoted { value, .. } = row.value_syntax() else {
        panic!("double quoted syntax");
    };
    assert_eq!(
        value.fragment(),
        "a&amp;b",
        "the authored value is not decoded"
    );
    assert_eq!(row.complete().fragment(), "id=\"a&amp;b\"");
    assert_eq!(row.authored_name().fragment(), "id");
    assert_eq!(row.complete().source_id(), SourceId::new(SOURCE));
    assert_eq!(report.source_id(), SourceId::new(SOURCE));
}

/// Falsifies: raw authored U+0000 conflated with numeric zero. Both interpret
/// as U+FFFD, through different sources and different diagnostics, and the
/// authored source keeps both spellings.
#[test]
fn av8_raw_nul_and_numeric_zero_stay_distinct() {
    // NUL 15..16, `&#0;` 16..20, close quote 20..21.
    let text = "<body><div id=\"\0&#0;\"></div>";
    let run = lex(text);
    assert!(complete(&run));
    assert_eq!(
        token_attribute(&run, 1),
        expected_attr(Quote::Double, 5, "\u{fffd}\u{fffd}")
    );
    assert_eq!(
        diag_rows(&run),
        vec![
            (
                Diag::UnexpectedNullCharacter,
                Ctx::AttributeValueDoubleQuoted,
                (15, 16)
            ),
            (Diag::NullCharacterReference, END, (19, 20)),
        ]
    );
    let report = analyze(text);
    let row = &report.selected_ordinary_attributes()[0];
    let HtmlTreeAttributeValueSyntax::DoubleQuoted { value, .. } = row.value_syntax() else {
        panic!("double quoted syntax");
    };
    assert_eq!(value.fragment(), "\0&#0;");
    assert_eq!(row.interpreted_value(), "\u{fffd}\u{fffd}");
}

// ---------------------------------------------------------------------------
// Resource and preflight
// ---------------------------------------------------------------------------

/// Retained refusals. `(source, retained limit, attempted, refusal site,
/// retained bytes still owned, committed value kept)`. The base owned before
/// the value is 4 (`body`) + 3 (`div`) + 2 (`id`) = 9.
type Retained = (&'static str, usize, usize, usize, usize);

const RETAINED_REFUSALS: &[Retained] = &[
    ("<body><div id=\"&amp;\">", 9, 10, 16, 9),
    // Five decoded bytes: not the fifteen authored ones, not the two scalars.
    ("<body><div id=\"&NotEqualTilde;\">", 13, 14, 16, 9),
    ("<body><div id=\"&NotEqualTilde;\">", 11, 14, 16, 9),
    // An earlier reference stays committed; the next one is refused whole.
    ("<body><div id=\"a&amp;&amp;\">", 11, 12, 22, 11),
    ("<body><div id=\"&#x1F600;\">", 12, 13, 23, 9),
    ("<body><div id=\"&#x1F600\">", 12, 13, 23, 9),
    ("<body><div id=\"&#\">", 10, 11, 17, 9),
    ("<body><div id=\"&notit;\">", 12, 13, 16, 9),
    ("<body><div id=\"ab\">", 10, 11, 16, 10),
];

/// The same sources at exactly their final retained size.
const RETAINED_FITS: &[(&str, usize)] = &[
    ("<body><div id=\"&amp;\">", 10),
    ("<body><div id=\"&NotEqualTilde;\">", 14),
    ("<body><div id=\"a&amp;&amp;\">", 12),
    ("<body><div id=\"&#x1F600;\">", 13),
    ("<body><div id=\"&#\">", 11),
];

/// Falsifies: retained cost taken from the authored length or the scalar
/// count, consuming before the refusal is known, a partially appended
/// multi-scalar value, and coverage past the refused effect.
#[test]
fn av9_retained_refusal_leaves_no_partial_reference_effect() {
    for &(text, limit, attempted, site, owned) in RETAINED_REFUSALS {
        let run = lex_with(text, limits(PRODUCT_STEPS, 256, limit));
        assert_eq!(
            refusal(&run),
            Some((Resource::RetainedInterpretedBytes, limit, attempted, site)),
            "{text:?} @ {limit}"
        );
        assert_eq!(
            run.usage().retained_interpreted_bytes(),
            owned,
            "{text:?} @ {limit}: nothing of the refused effect is retained"
        );
        assert_eq!(
            run.coverage().processed_end(),
            site,
            "{text:?} @ {limit}: the refused reference stays unconsumed"
        );
        assert_eq!(
            tag_count(&run, HtmlTagKind::Start),
            1,
            "{text:?}: only <body> was emitted"
        );
        assert!(run.diagnostics().is_empty(), "{text:?}");
    }
    for &(text, limit) in RETAINED_FITS {
        let run = lex_with(text, limits(PRODUCT_STEPS, 256, limit));
        assert!(complete(&run), "{text:?} @ {limit}: {:?}", run.completion());
    }
}

/// Every retained limit below the final size of the whole matrix refuses
/// with the run owning exactly the bytes committed before the refused effect:
/// the owned bytes always sit on a commit boundary of the interpreted value.
#[test]
fn av10_every_retained_limit_refuses_on_a_commit_boundary() {
    for &(raw, interpreted, _) in MATRIX {
        let text = fixture(Quote::Double, raw, false);
        let full = lex_with(&text, limits(PRODUCT_STEPS, 256, PRODUCT_RETAINED));
        assert!(complete(&full), "{raw:?}");
        let final_size = full.usage().retained_interpreted_bytes();
        assert_eq!(final_size, 9 + interpreted.len(), "{raw:?}: decoded cost");
        for limit in 9..final_size {
            let run = lex_with(&text, limits(PRODUCT_STEPS, 256, limit));
            let (resource, observed_limit, attempted, _) =
                refusal(&run).unwrap_or_else(|| panic!("{raw:?} @ {limit} must be refused"));
            assert_eq!(resource, Resource::RetainedInterpretedBytes, "{raw:?}");
            assert_eq!(observed_limit, limit, "{raw:?}");
            assert!(
                attempted > limit && attempted <= final_size,
                "{raw:?} @ {limit}"
            );
            let owned = run.usage().retained_interpreted_bytes();
            assert!(owned <= limit, "{raw:?} @ {limit}: owns {owned}");
            // The owned value prefix is a prefix of the decoded value at a
            // character boundary, never half of one reference's output.
            let kept = &interpreted.as_bytes()[..owned - 9];
            assert!(
                std::str::from_utf8(kept).is_ok(),
                "{raw:?} @ {limit}: split a decoded scalar"
            );
        }
        let at_final = lex_with(&text, limits(PRODUCT_STEPS, 256, final_size));
        assert!(complete(&at_final), "{raw:?} @ final size");
    }
}

/// `(source, diagnostics limit, attempted, refusal site, processed end,
/// diagnostics kept, retained bytes owned)`.
type DiagnosticsRefusal = (&'static str, usize, usize, usize, usize, usize, usize);

const DIAGNOSTICS_REFUSALS: &[DiagnosticsRefusal] = &[
    // Named: the diagnostic is prepared with the output it explains.
    ("<body><div id=\"&amp\">", 0, 1, 16, 16, 0, 9),
    // Numeric: refused at the terminal unit, before `;`/the terminator and
    // before any output.
    ("<body><div id=\"&#65\">", 0, 1, 19, 19, 0, 9),
    ("<body><div id=\"&#0;\">", 0, 1, 18, 18, 0, 9),
    // The semicolonless observation and the End recovery are one atomic
    // effect: the first would fit alone, yet neither is committed.
    ("<body><div id=\"&#0\">", 1, 2, 18, 18, 0, 9),
    // An unknown name is per-unit by nature: the candidate stays, and the `;`
    // observation is the refused effect.
    ("<body><div id=\"&zz;\">", 0, 1, 18, 18, 0, 12),
];

/// Falsifies: output committed without its diagnostic, a diagnostic committed
/// without its output, and the numeric observation split from its End effect.
#[test]
fn av11_diagnostics_refusal_leaves_no_partial_reference_effect() {
    for &(text, limit, attempted, site, processed, kept, owned) in DIAGNOSTICS_REFUSALS {
        let run = lex_with(text, limits(PRODUCT_STEPS, limit, PRODUCT_RETAINED));
        assert_eq!(
            refusal(&run),
            Some((Resource::Diagnostics, limit, attempted, site)),
            "{text:?} @ diagnostics {limit}"
        );
        assert_eq!(run.coverage().processed_end(), processed, "{text:?}");
        assert_eq!(run.diagnostics().len(), kept, "{text:?}");
        assert_eq!(run.usage().retained_interpreted_bytes(), owned, "{text:?}");
    }
    // The same sources complete once every diagnostic fits.
    for (text, limit) in [
        ("<body><div id=\"&amp\">", 1),
        ("<body><div id=\"&#65\">", 1),
        ("<body><div id=\"&#0;\">", 1),
        ("<body><div id=\"&#0\">", 2),
        ("<body><div id=\"&zz;\">", 1),
    ] {
        let run = lex_with(text, limits(PRODUCT_STEPS, limit, PRODUCT_RETAINED));
        assert!(complete(&run), "{text:?} @ {limit}: {:?}", run.completion());
        assert_eq!(run.diagnostics().len(), limit, "{text:?}");
    }
}

/// Steps spent before the first value byte of `<body><div id="`, counted by
/// hand under the existing per-state formula (one step per dispatched unit,
/// plus one for each reconsumed dispatch):
///
/// ```text
/// <body>     `<` 1, `b` 2 (reconsumed into TagName), `o` `d` `y` 3, `>` 1 = 7
/// <div id="  `<` 1, `d` 2, `i` `v` 2, space 1, `i` 2 (reconsumed into
///            AttributeName), `d` 1, `=` 1, `"` 1                          = 11
/// ```
const PREFIX_STEPS: usize = 18;

/// Existing TransitionSteps accounting applied to the newly reachable states:
/// one step per dispatched unit, one per reconsumed dispatch, one for the
/// input-free Numeric End, and one transition for a whole Named operation.
///
/// After the value, `">` costs 2 and end of file 1.
/// `&amp;`: `&`, the reference-start dispatch, one Named operation = 3, which
/// is two more than the single step of an ordinary one-byte value.
/// `&#65;`: `&`, `#`, radix selection, `6`, `5`, `;`, End = 7, six more.
#[test]
fn av12_transition_steps_follow_the_existing_per_state_formula() {
    let run = lex("<body><div id=\"&amp;\">");
    assert!(complete(&run));
    assert_eq!(run.usage().transition_steps(), PREFIX_STEPS + 3 + 2 + 1);

    let run = lex("<body><div id=\"&#65;\">");
    assert!(complete(&run));
    assert_eq!(run.usage().transition_steps(), PREFIX_STEPS + 7 + 2 + 1);

    // A one-byte ordinary value is the baseline: one step.
    let run = lex("<body><div id=\"x\">");
    assert_eq!(run.usage().transition_steps(), PREFIX_STEPS + 1 + 2 + 1);
}

/// Falsifies: a Named operation that costs one step per matched byte, and a
/// refusal that consumes the identifier it could not afford.
#[test]
fn av13_step_refusals_keep_honest_partial_progress() {
    let steps_refusal = |run: &HtmlTokenizerRunResult| {
        refusal(run).map(|(resource, limit, attempted, _)| (resource, limit, attempted))
    };
    let text = "<body><div id=\"&amp;\">";
    // Step 19 is `&`, 20 the reference start, 21 the whole Named operation.
    let run = lex_with(text, limits(PREFIX_STEPS + 2, 256, PRODUCT_RETAINED));
    assert_eq!(
        steps_refusal(&run),
        Some((
            Resource::TransitionSteps,
            PREFIX_STEPS + 2,
            PREFIX_STEPS + 3
        ))
    );
    assert_eq!(run.coverage().processed_end(), 16, "identifier unconsumed");
    assert_eq!(run.usage().retained_interpreted_bytes(), 9);

    // With one more step the reference is fully committed and step 22 (`"`)
    // is refused.
    let run = lex_with(text, limits(PREFIX_STEPS + 3, 256, PRODUCT_RETAINED));
    assert_eq!(
        steps_refusal(&run),
        Some((
            Resource::TransitionSteps,
            PREFIX_STEPS + 3,
            PREFIX_STEPS + 4
        ))
    );
    assert_eq!(run.coverage().processed_end(), 20);
    assert_eq!(
        run.usage().retained_interpreted_bytes(),
        10,
        "`&` committed"
    );

    // Numeric: `&` 19, `#` 20, radix 21, `6` 22, `5` 23, `;` 24, End 25.
    let text = "<body><div id=\"&#65;\">";
    let run = lex_with(text, limits(PREFIX_STEPS + 5, 256, PRODUCT_RETAINED));
    assert_eq!(
        steps_refusal(&run),
        Some((
            Resource::TransitionSteps,
            PREFIX_STEPS + 5,
            PREFIX_STEPS + 6
        ))
    );
    assert_eq!(run.coverage().processed_end(), 19, "`;` unconsumed");
    let run = lex_with(text, limits(PREFIX_STEPS + 6, 256, PRODUCT_RETAINED));
    assert_eq!(
        steps_refusal(&run),
        Some((
            Resource::TransitionSteps,
            PREFIX_STEPS + 6,
            PREFIX_STEPS + 7
        ))
    );
    assert_eq!(run.coverage().processed_end(), 20, "`;` committed progress");
    assert_eq!(
        run.usage().retained_interpreted_bytes(),
        9,
        "End never ran, so nothing was appended"
    );
}

/// The fixed Product resource vector gained no dimension and no temporary
/// buffer: the Named and numeric paths borrow canonical data and keep a fixed
/// accumulator.
#[test]
fn av14_no_new_resource_dimension_and_zero_temporary_buffer() {
    for &(raw, _, _) in MATRIX {
        let run = lex(&fixture(Quote::Double, raw, true));
        assert_eq!(run.usage().peak_temporary_buffer_bytes(), 0, "{raw:?}");
        assert_eq!(run.limits(), product(), "{raw:?}");
        assert_eq!(run.usage().peak_attributes_per_tag(), 1, "{raw:?}");
    }
}

// ---------------------------------------------------------------------------
// Tree boundaries (all unchanged)
// ---------------------------------------------------------------------------

/// Falsifies: reference support relaxing the one-attribute Product profile.
#[test]
fn av15_a_second_attribute_remains_the_attributes_per_tag_resource_limit() {
    let text = "<body><div id=\"&amp;\" class=x></div>";
    let run = lex(text);
    assert_eq!(
        refusal(&run).map(|(resource, limit, attempted, _)| (resource, limit, attempted)),
        Some((Resource::AttributesPerTag, 1, 2))
    );
    let report = analyze(text);
    let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::ResourceLimited(limit)) =
        report.completion()
    else {
        panic!("expected resource limited, got {:?}", report.completion());
    };
    assert_eq!(limit.kind(), HtmlTreeResourceKind::AttributesPerTag);
    assert_eq!((limit.limit(), limit.attempted()), (1, 2));
    assert!(report.selected_ordinary_attributes().is_empty());
    assert_eq!(report.nodes().len(), 4, "no selected node was constructed");
}

/// Falsifies: the attribute refusal / self-closing precedence changing because
/// the attribute value now contains a reference.
#[test]
fn av16_attributed_self_closing_keeps_the_attribute_refusal_precedence() {
    for text in [
        "<body><div id=\"&amp;\"/>",
        "<body><nav a=&lt; />",
        "<body><main a='&#65;'/>",
    ] {
        let report = analyze(text);
        assert!(
            matches!(
                report.completion(),
                HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(u))
                    if u.capability() == HtmlTreeUnsupportedCapability::SelectedOrdinaryTagAttribute
            ),
            "{text:?}: {:?}",
            report.completion()
        );
        assert!(report.selected_ordinary_attributes().is_empty(), "{text:?}");
        assert_eq!(report.nodes().len(), 4, "{text:?}");
    }
}

/// Falsifies: a later unrelated stop erasing already committed node and
/// attribute evidence.
#[test]
fn av17_a_later_unsupported_stop_keeps_the_committed_decoded_attribute() {
    // `<div id="&amp;">` is 6..22 and commits; `<span>` then stops the run.
    let text = "<body><div id=\"&amp;\"><span>";
    let report = analyze(text);
    assert!(matches!(
        report.completion(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(_))
    ));
    assert!(!is_complete_tree(&report));
    assert_eq!(report.selected_ordinary_attributes().len(), 1);
    assert_eq!(
        report_attribute(&report),
        expected_attr(Quote::Double, 5, "&")
    );
    assert_eq!(report.selected_ordinary_attributes()[0].node().value(), 4);
}

/// Falsifies: tokenizer element-family coupling. The tokenizer is family
/// neutral, so a reference-bearing attribute on an excluded family now
/// completes its tag and reaches that family's existing tree refusal; it no
/// longer stops at the tokenizer character-reference boundary, and no node or
/// attribute row is projected for it.
#[test]
fn av18_excluded_families_complete_the_tokenizer_then_hit_their_existing_tree_refusal() {
    let families = [
        (
            "<body><p id=\"&amp;\">",
            HtmlTreeUnsupportedCapability::ParagraphTagAttribute,
        ),
        (
            "<body><style id=\"&amp;\">",
            HtmlTreeUnsupportedCapability::StyleTagAttribute,
        ),
        (
            "<body><title id=\"&amp;\">",
            HtmlTreeUnsupportedCapability::TitleTagAttribute,
        ),
        (
            "<html lang=\"&amp;\">",
            HtmlTreeUnsupportedCapability::ShellTagAttribute,
        ),
        (
            "<body><span id=\"&amp;\">",
            HtmlTreeUnsupportedCapability::NonShellElementTag,
        ),
    ];
    for (text, capability) in families {
        let report = analyze(text);
        assert!(
            matches!(
                report.completion(),
                HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::TreeUnsupported(u))
                    if u.capability() == capability
            ),
            "{text:?}: {:?}",
            report.completion()
        );
        assert!(report.selected_ordinary_attributes().is_empty(), "{text:?}");
        // The refusal is exactly the one the same family gives a plain value.
        let control = analyze(&text.replace("&amp;", "x"));
        assert_eq!(
            tree_capability(&control),
            tree_capability(&report),
            "{text:?}: unchanged tree refusal"
        );

        // The old lower-layer stop is gone: the standalone tokenizer completes
        // the tag instead of stopping at `CharacterReference(AttributeValue)`.
        let run = lex(text);
        let stopped_at_reference = matches!(
            run.completion(),
            HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported))
                if matches!(unsupported.capability(), HtmlTokenizerCapability::CharacterReference { .. })
        );
        assert!(!stopped_at_reference, "{text:?}: {:?}", run.completion());
    }
    // The tokenizer completes the attributed paragraph tag with the decoded
    // value, without knowing anything about paragraphs.
    let run = lex("<p id=\"&amp;\"></p>");
    assert!(complete(&run), "{:?}", run.completion());
    let Some(HtmlToken::Tag(tag)) = run.tokens().first() else {
        panic!("p start tag");
    };
    assert_eq!(tag.name().interpreted(), "p");
    assert_eq!(tag.attributes()[0].interpreted_value(), "&");
}

/// Falsifies: AttributeValue support leaking through the shared states into
/// Data/RCDATA routing, and Data/RCDATA text no longer reaching Character
/// tokens after an attribute reference.
#[test]
fn av19_data_and_rcdata_routing_is_unchanged_around_an_attribute_reference() {
    let run = lex("&amp;<div id=\"&amp;\">&lt;");
    assert!(complete(&run), "{:?}", run.completion());
    let kinds: Vec<String> = run
        .tokens()
        .iter()
        .map(|token| match token {
            HtmlToken::Character(character) => {
                format!(
                    "char {:?} {:?}",
                    span(character.source()),
                    character.interpreted()
                )
            }
            HtmlToken::Tag(tag) => format!("tag {}", tag.name().interpreted()),
            HtmlToken::EndOfFile(_) => "eof".to_owned(),
            other => format!("{other:?}"),
        })
        .collect();
    assert_eq!(
        kinds,
        vec![
            "char (0, 5) \"&\"".to_owned(),
            "tag div".to_owned(),
            "char (21, 25) \"<\"".to_owned(),
            "eof".to_owned(),
        ]
    );
    assert_eq!(
        token_attribute(&run, 1).interpreted_value,
        "&",
        "the attribute reference is not a Character token"
    );

    // Coordinated Title RCDATA keeps resolving its own references.
    let report = analyze("<title>a&amp;b</title><body><div id=\"&lt;\">&amp;</div>");
    assert!(is_complete_tree(&report), "{:?}", report.completion());
    assert_eq!(
        report.selected_ordinary_attributes()[0].interpreted_value(),
        "<"
    );
}
