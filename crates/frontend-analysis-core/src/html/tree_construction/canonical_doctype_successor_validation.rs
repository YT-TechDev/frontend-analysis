//! Candidate-independent validation of the *proposed* successor theorem
//! "Selected Canonical HTML DOCTYPE / Initial No-Quirks Recognition"
//! (Issue #890; selection record: #348 comment `5979233108`).
//!
//! This module is **validation only**. It changes no production behavior.
//! Production is expected to remain unchanged and still stops canonical
//! DOCTYPE at the tokenizer `MarkupDeclaration` boundary. Nothing here
//! authorizes production placement or implementation, and no name in this file
//! is production placement authority.
//!
//! # Normative basis (WHATWG HTML, pin `a5e15011a00ddefd648c29e4d27734f3e7ff821f`)
//!
//! * Markup declaration open: an ASCII case-insensitive `DOCTYPE` after `<!`.
//! * DOCTYPE state: ASCII whitespace -> before DOCTYPE name; `>` reconsume;
//!   anything else is a `missing-whitespace-before-doctype-name` parse error
//!   and reconsumes in before DOCTYPE name. **Consequence:** `<!DOCTYPEhtml>`
//!   *is* a DOCTYPE token (name `html`) in the Standard, but it carries a
//!   tokenizer parse error. It is therefore outside the selected *canonical*
//!   profile (a profile that carries no tokenizer parse error), not "not a
//!   DOCTYPE". html5lib-tests corroborates that lineage (challenge evidence,
//!   not an independent vote: it is derived from the same Standard).
//! * Initial insertion mode, DOCTYPE token: create a DocumentType (missing
//!   identifiers become the empty string *in the DOM node*), append it to the
//!   Document when it has no DocumentType/element child, apply the quirks
//!   conditions (force-quirks on, name not `html`, identifier tables), then
//!   switch to "before html". The token is not reprocessed.
//! * Initial insertion mode, anything else: parse error (MissingDoctype),
//!   quirks mode (when the parser can change the mode flag), switch to
//!   "before html" and reprocess the token.
//! * Before html: a DOCTYPE token is a parse error and is ignored. That is
//!   general non-Initial recovery and is deliberately **not** modelled here:
//!   any DOCTYPE outside Initial is `OutsideSelectedTreeProfile`.
//!
//! Parse configuration assumed: a non-`iframe srcdoc` document whose parser
//! can change the mode flag. The `srcdoc`/cannot-change-mode configurations
//! are not selected.
//!
//! # Independence boundary
//!
//! Expected meaning is hand-authored in [`gold`] from the theorem above. This
//! module imports no production token, tree-construction driver/session/result,
//! or tokenizer-diagnostic code. The only generic project primitives used are
//! `SourceText`/`SourceAnchor`/`SourceId`, and the batch tokenizer is used in
//! exactly one place: [`current_production_stops_at_markup_declaration`],
//! which observes the existing boundary without defining any expectation.
//!
//! Two independent statements meet and must agree:
//!
//! 1. the test-private model ([`lex`] + [`Machine`] + the projections), which
//!    owns its own recognizer and captures source evidence *at recognition*;
//!    and
//! 2. [`gold`], hand-authored expected observations with handwritten byte
//!    offsets that are separately checked against the literal fixture bytes.
//!
//! # Deliberately partial model
//!
//! Only the cells the fixtures traverse are modelled (Initial, BeforeHtml,
//! BeforeHead, plus a DOCTYPE-only cell for later modes). Everything else stops
//! as `ModelBoundary`, a property of this oracle and not of production. This
//! module is not a second HTML parser, and the abstract [`Budget`] is a
//! semantic atomicity device, not a proposed production resource dimension.
//!
//! # Constructed identity (test-private)
//!
//! The representation comparison uses an opaque, result-scoped identity (see
//! the `identity` module). It is not derived from source identity, authored
//! range, final placement or document mode, and nothing here claims that an
//! identity is comparable, equal or stable across independent results; ids
//! from different result scopes are explicitly not comparable. The selected
//! slice has no placement-changing recovery, so the model does not exercise
//! the architecture's "identity survives recovery" invariant. It only avoids
//! precluding it, because identity never reads placement. This is a validation
//! vocabulary, not a production encoding or placement.

use std::collections::BTreeSet;

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::{
    HtmlTokenizerCapability, HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause,
};

// ---------------------------------------------------------------------------
// 1. Test-private semantic vocabulary (not production types)
// ---------------------------------------------------------------------------

type ByteRange = (usize, usize);

/// Source identity plus authored byte range. Authored evidence only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span {
    source: u64,
    range: ByteRange,
}

/// `Missing` is not an authored empty identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Identifier {
    Missing,
    Authored { span: Span, value: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForceQuirks {
    Off,
    On,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DoctypeToken {
    complete: Span,
    name_source: Span,
    /// Exact authored spelling of the name, captured at recognition.
    name_spelling: String,
    interpreted_name: String,
    public: Identifier,
    system: Identifier,
    force_quirks: ForceQuirks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagName {
    Html,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Doctype(DoctypeToken),
    StartTag {
        name: TagName,
        span: Span,
    },
    EndTag {
        name: TagName,
        span: Span,
    },
    /// Never produced by the model's recognizer; exists so a text-masquerade
    /// countermodel can be expressed.
    Character {
        span: Span,
        interpreted: String,
    },
    EndOfFile {
        at: usize,
    },
}

impl Token {
    /// Retained source anchors this token owns once committed.
    fn anchors(&self) -> usize {
        match self {
            Self::Doctype(_) => 2,
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Broader {
    NameNotHtml,
    PublicKeyword,
    SystemKeyword,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Malformed {
    MissingName,
    TruncatedAtEof,
    AfterNameJunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Markup {
    Comment,
    Cdata,
    ProcessingInstruction,
    General,
}

/// Lexical boundaries around the selected profile. None of these claims a
/// production semantic and none claims "invalid HTML".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Boundary {
    /// A DOCTYPE with a well-formed name that is wider than the selected
    /// profile.
    BroaderDoctype(Broader),
    /// A DOCTYPE-shaped input whose end is not the selected profile.
    MalformedSelectedForm(Malformed),
    /// The Standard still produces a DOCTYPE token, but with a tokenizer
    /// parse error (`missing-whitespace-before-doctype-name`). The selected
    /// canonical profile has none.
    RequiresTokenizerParseError,
    OtherMarkup(Markup),
    OutsideModelledToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Resource {
    Tokens,
    Anchors,
    Diagnostics,
}

/// Abstract atomicity device only. Not a proposed production dimension.
#[derive(Debug, Clone, Copy)]
struct Budget {
    tokens: usize,
    anchors: usize,
    diagnostics: usize,
}

impl Budget {
    const GENEROUS: Self = Self {
        tokens: 1_024,
        anchors: 1_024,
        diagnostics: 1_024,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InBody,
    AfterBody,
    AfterAfterBody,
}

const ALL_MODES: [Mode; 6] = [
    Mode::Initial,
    Mode::BeforeHtml,
    Mode::BeforeHead,
    Mode::InBody,
    Mode::AfterBody,
    Mode::AfterAfterBody,
];

/// Only the states needed by the selected theorem. `LimitedQuirks` is
/// deliberately absent: the selected profile never needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocMode {
    NoQuirks,
    Quirks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Consumed {
        next: Mode,
    },
    Reprocess {
        next: Mode,
    },
    /// A DOCTYPE token outside Initial: not selected, no recovery invented.
    OutsideSelectedTreeProfile,
    ModelBoundary,
    ResourceRefused(Resource),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Step {
    mode: Mode,
    outcome: Outcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Dispatch {
    token_index: usize,
    steps: Vec<Step>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagCode {
    MissingDoctype,
    /// Tree-level DOCTYPE parse error (non-`html` name or present identifier).
    InitialDoctypeParseError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Diag {
    code: DiagCode,
    token_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ElementOrigin {
    Authored(Span),
    /// Implied by the before-html "anything else" rule.
    SynthesizedByBeforeHtml,
}

/// The normative constructed Document children (WHATWG), before any durable
/// representation is chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Child {
    DocumentType(DoctypeToken),
    Html(ElementOrigin),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Completion {
    /// Lower layer complete and the model reached its first unmodelled cell.
    CheckpointReached {
        stopped_at: Option<usize>,
    },
    Boundary(Boundary),
    ResourceLimited(Resource),
    TreeRefused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    tokens: Vec<Token>,
    dispatches: Vec<Dispatch>,
    diagnostics: Vec<Diag>,
    doc_mode: DocMode,
    mode: Mode,
    children: Vec<Child>,
    retained_anchors: usize,
    completion: Completion,
}

/// One independently falsifiable semantic dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Dim {
    Acceptance,
    TokenKind,
    CompleteRange,
    NameRange,
    SourceIdentity,
    NameSpelling,
    InterpretedName,
    PublicIdentifier,
    SystemIdentifier,
    ForceQuirks,
    OtherTokens,
    Dispatch,
    Diagnostics,
    DocumentMode,
    ModeTransition,
    Constructed,
    RetainedEvidence,
    Completion,
}

// ---------------------------------------------------------------------------
// 2. Fixtures (escaped byte literals; rendered text is never byte authority)
// ---------------------------------------------------------------------------

const SOURCE_ID: u64 = 7;

#[derive(Debug, Clone, Copy)]
enum Tail {
    /// Nothing follows the DOCTYPE.
    None,
    /// `<html></html>` follows.
    HtmlPair { start: ByteRange, end: ByteRange },
    /// `<body></body></html>` follows.
    BodyShell {
        body: ByteRange,
        body_end: ByteRange,
        html_end: ByteRange,
    },
}

/// One accepted canonical fixture with handwritten expected offsets.
struct Accepted {
    id: &'static str,
    bytes: &'static [u8],
    complete: ByteRange,
    name: ByteRange,
    /// Exact authored spelling of the name.
    spelling: &'static str,
    tail: Tail,
    /// Interpreted whitespace units (CR and CRLF are one unit each) between
    /// the keyword and the name, to pin authored != interpreted.
    leading_whitespace_authored: usize,
    leading_whitespace_interpreted: usize,
}

const fn acc(
    id: &'static str,
    bytes: &'static [u8],
    complete: ByteRange,
    name: ByteRange,
    spelling: &'static str,
) -> Accepted {
    Accepted {
        id,
        bytes,
        complete,
        name,
        spelling,
        tail: Tail::None,
        leading_whitespace_authored: name.0 - 9,
        leading_whitespace_interpreted: name.0 - 9,
    }
}

const ACCEPTED: &[Accepted] = &[
    // --- keyword and name casing ---
    acc("A01", b"<!DOCTYPE html>", (0, 15), (10, 14), "html"),
    acc("A02", b"<!doctype html>", (0, 15), (10, 14), "html"),
    acc("A03", b"<!DoCtYpE HTML>", (0, 15), (10, 14), "HTML"),
    acc("A04", b"<!dOcTyPe hTmL>", (0, 15), (10, 14), "hTmL"),
    acc("A05", b"<!DOCTYPE HTML>", (0, 15), (10, 14), "HTML"),
    // --- each whitespace class between keyword and name ---
    acc("W01", b"<!DOCTYPE\x09html>", (0, 15), (10, 14), "html"),
    acc("W02", b"<!DOCTYPE\x0ahtml>", (0, 15), (10, 14), "html"),
    acc("W03", b"<!DOCTYPE\x0chtml>", (0, 15), (10, 14), "html"),
    acc("W04", b"<!DOCTYPE\x0dhtml>", (0, 15), (10, 14), "html"),
    // Authored CRLF is two source bytes and one interpreted LF.
    Accepted {
        id: "W05",
        bytes: b"<!DOCTYPE\x0d\x0ahtml>",
        complete: (0, 16),
        name: (11, 15),
        spelling: "html",
        tail: Tail::None,
        leading_whitespace_authored: 2,
        leading_whitespace_interpreted: 1,
    },
    acc("W06", b"<!DOCTYPE     html>", (0, 19), (14, 18), "html"),
    acc(
        "W07",
        b"<!DOCTYPE\x20\x09\x0a\x0c\x0d\x20html>",
        (0, 20),
        (15, 19),
        "html",
    ),
    // --- each whitespace class immediately before `>` ---
    acc("T01", b"<!DOCTYPE html >", (0, 16), (10, 14), "html"),
    acc("T02", b"<!DOCTYPE html\x09>", (0, 16), (10, 14), "html"),
    acc("T03", b"<!DOCTYPE html\x0a>", (0, 16), (10, 14), "html"),
    acc("T04", b"<!DOCTYPE html\x0c>", (0, 16), (10, 14), "html"),
    acc("T05", b"<!DOCTYPE html\x0d>", (0, 16), (10, 14), "html"),
    acc("T06", b"<!DOCTYPE html\x0d\x0a>", (0, 17), (10, 14), "html"),
    acc("T07", b"<!DOCTYPE html   >", (0, 18), (10, 14), "html"),
    acc("T08", b"<!DOCTYPE   html   >", (0, 20), (12, 16), "html"),
    acc(
        "T09",
        b"<!DOCTYPE\x20html\x20\x09\x0a\x0c\x0d\x20>",
        (0, 21),
        (10, 14),
        "html",
    ),
    // --- composition with selected document-shell inputs ---
    Accepted {
        id: "C01",
        bytes: b"<!DOCTYPE html><html></html>",
        complete: (0, 15),
        name: (10, 14),
        spelling: "html",
        tail: Tail::HtmlPair {
            start: (15, 21),
            end: (21, 28),
        },
        leading_whitespace_authored: 1,
        leading_whitespace_interpreted: 1,
    },
    Accepted {
        id: "C02",
        bytes: b"<!DOCTYPE html><body></body></html>",
        complete: (0, 15),
        name: (10, 14),
        spelling: "html",
        tail: Tail::BodyShell {
            body: (15, 21),
            body_end: (21, 28),
            html_end: (28, 35),
        },
        leading_whitespace_authored: 1,
        leading_whitespace_interpreted: 1,
    },
];

/// Required byte content at handwritten ranges, independent of the model.
fn expected_fragments(accepted: &Accepted) -> Vec<(ByteRange, &'static [u8])> {
    let mut v: Vec<(ByteRange, &'static [u8])> = vec![
        (accepted.name, accepted.spelling.as_bytes()),
        ((accepted.complete.0, accepted.complete.0 + 2), b"<!"),
        ((accepted.complete.1 - 1, accepted.complete.1), b">"),
    ];
    match accepted.tail {
        Tail::None => {}
        Tail::HtmlPair { start, end } => {
            v.push((start, b"<html>"));
            v.push((end, b"</html>"));
        }
        Tail::BodyShell {
            body,
            body_end,
            html_end,
        } => {
            v.push((body, b"<body>"));
            v.push((body_end, b"</body>"));
            v.push((html_end, b"</html>"));
        }
    }
    v
}

fn accepted(id: &str) -> &'static Accepted {
    ACCEPTED
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("unknown accepted fixture {id}"))
}

fn text_of(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("fixture bytes are UTF-8")
}

/// Standalone outside-profile fixtures with a handwritten classification.
const NEGATIVES: &[(&str, &[u8], Boundary)] = &[
    (
        "X01",
        b"<!DOCTYPE>",
        Boundary::MalformedSelectedForm(Malformed::MissingName),
    ),
    (
        "X02",
        b"<!DOCTYPE svg>",
        Boundary::BroaderDoctype(Broader::NameNotHtml),
    ),
    (
        "X03",
        b"<!DOCTYPE html PUBLIC \"x\">",
        Boundary::BroaderDoctype(Broader::PublicKeyword),
    ),
    (
        "X04",
        b"<!DOCTYPE html SYSTEM \"x\">",
        Boundary::BroaderDoctype(Broader::SystemKeyword),
    ),
    (
        "X05",
        b"<!DOCTYPE html PUBLIC \"\">",
        Boundary::BroaderDoctype(Broader::PublicKeyword),
    ),
    (
        "X06",
        b"<!DOCTYPE html SYSTEM \"\">",
        Boundary::BroaderDoctype(Broader::SystemKeyword),
    ),
    (
        "X07",
        b"<!DOCTYPE html foo>",
        Boundary::MalformedSelectedForm(Malformed::AfterNameJunk),
    ),
    (
        "X08",
        b"<!DOCTYPEhtml>",
        Boundary::RequiresTokenizerParseError,
    ),
    (
        "X09",
        b"<!DOCTYPE",
        Boundary::MalformedSelectedForm(Malformed::TruncatedAtEof),
    ),
    (
        "X10",
        b"<!DOCTYPE html",
        Boundary::MalformedSelectedForm(Malformed::TruncatedAtEof),
    ),
    (
        "X11",
        b"<!doctype html x>",
        Boundary::MalformedSelectedForm(Malformed::AfterNameJunk),
    ),
    (
        "X12",
        b"<!DOCTYPE >",
        Boundary::MalformedSelectedForm(Malformed::MissingName),
    ),
    (
        "X13",
        b"<!DOCTYPE ",
        Boundary::MalformedSelectedForm(Malformed::TruncatedAtEof),
    ),
    (
        "X14",
        b"<!DOCTYPE htmlx>",
        Boundary::BroaderDoctype(Broader::NameNotHtml),
    ),
    (
        "X15",
        b"<!DOCTYPE htm>",
        Boundary::BroaderDoctype(Broader::NameNotHtml),
    ),
    // Non-ASCII and non-HTML whitespace never qualifies as selected whitespace.
    // VT is ASCII but is not HTML whitespace.
    (
        "U01",
        b"<!DOCTYPE\x0bhtml>",
        Boundary::RequiresTokenizerParseError,
    ),
    // NBSP U+00A0.
    (
        "U02",
        b"<!DOCTYPE\xc2\xa0html>",
        Boundary::RequiresTokenizerParseError,
    ),
    // EM SPACE U+2003.
    (
        "U03",
        b"<!DOCTYPE\xe2\x80\x83html>",
        Boundary::RequiresTokenizerParseError,
    ),
    // NEL U+0085.
    (
        "U04",
        b"<!DOCTYPE\xc2\x85html>",
        Boundary::RequiresTokenizerParseError,
    ),
    // Trailing non-selected whitespace becomes part of the name.
    (
        "U05",
        b"<!DOCTYPE html\xc2\xa0>",
        Boundary::BroaderDoctype(Broader::NameNotHtml),
    ),
    (
        "U06",
        b"<!DOCTYPE html\x0b>",
        Boundary::BroaderDoctype(Broader::NameNotHtml),
    ),
    // Other markup sharing the `<!` / `<?` region.
    ("M01", b"<!-- c -->", Boundary::OtherMarkup(Markup::Comment)),
    (
        "M02",
        b"<![CDATA[x]]>",
        Boundary::OtherMarkup(Markup::Cdata),
    ),
    ("M03", b"<!xx>", Boundary::OtherMarkup(Markup::General)),
    (
        "M04",
        b"<?x?>",
        Boundary::OtherMarkup(Markup::ProcessingInstruction),
    ),
];

fn negative(id: &str) -> (&'static [u8], Boundary) {
    NEGATIVES
        .iter()
        .find(|(candidate, _, _)| *candidate == id)
        .map(|(_, bytes, boundary)| (*bytes, *boundary))
        .unwrap_or_else(|| panic!("unknown negative fixture {id}"))
}

// ---------------------------------------------------------------------------
// 3. Independent recognizer (owns source evidence at recognition time)
// ---------------------------------------------------------------------------

/// Deliberately wrong models used to prove each dimension load-bearing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutation {
    // token identity
    DoctypeAsStartTag,
    DoctypeAsText,
    // authored evidence
    NameAnchorOnKeyword,
    NameAnchorOnWhitespace,
    CompleteTruncated,
    CompleteShifted,
    SourceIdCorrupted,
    NameSpellingLowercased,
    // post-hoc reconstruction
    NameStartFromFixedOffset,
    NameFromCaseSensitiveSearch,
    CompleteEndFromInterpretedLength,
    RetokenizedSubstring,
    // meaning
    InterpretedNameSvg,
    ForceQuirksOn,
    PublicFabricatedEmpty,
    SystemFabricatedEmpty,
    // lexical widening
    AcceptPublicSystem,
    AcceptMissingWhitespace,
    AcceptUnicodeWhitespace,
    AliasMarkup(Markup),
    // tree
    MissingDoctypePollution,
    QuirksOnCanonical,
    ReprocessCanonical,
    DropDurableEvidence,
    AcceptNonInitial,
    // monotonicity / atomicity
    UpgradeIncomplete,
    PartialAnchorCommit,
    AdvanceModeOnRefusal,
}

fn is_html_whitespace(byte: u8) -> bool {
    matches!(byte, 0x09 | 0x0a | 0x0c | 0x0d | 0x20)
}

/// Bytes consumed as whitespace at `at`: selected ASCII whitespace only,
/// unless the Unicode-whitespace countermodel is active.
fn whitespace_len(source: &str, at: usize, unicode: bool) -> usize {
    let bytes = source.as_bytes();
    if at >= bytes.len() {
        return 0;
    }
    if is_html_whitespace(bytes[at]) {
        return 1;
    }
    if unicode
        && source.is_char_boundary(at)
        && let Some(character) = source[at..].chars().next()
        && character.is_whitespace()
    {
        return character.len_utf8();
    }
    0
}

fn eq_ignore_ascii_case(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(l, r)| l.eq_ignore_ascii_case(r))
}

fn starts_with_ci(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.len() >= needle.len() && eq_ignore_ascii_case(&haystack[..needle.len()], needle)
}

enum Recognized {
    Canonical { name: ByteRange, end: usize },
    Boundary(Boundary),
}

/// Single forward scan from the `<` of `<!DOCTYPE`. Offsets are captured here
/// and are never recomputed afterwards.
fn recognize_doctype(source: &str, lt: usize, mutation: Option<Mutation>) -> Recognized {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let unicode = mutation == Some(Mutation::AcceptUnicodeWhitespace);
    let mut at = lt + 9; // `<!DOCTYPE`
    if at >= len {
        return Recognized::Boundary(Boundary::MalformedSelectedForm(Malformed::TruncatedAtEof));
    }
    let first = whitespace_len(source, at, unicode);
    if first > 0 {
        while at < len {
            let step = whitespace_len(source, at, unicode);
            if step == 0 {
                break;
            }
            at += step;
        }
    } else if bytes[at] == b'>' {
        return Recognized::Boundary(Boundary::MalformedSelectedForm(Malformed::MissingName));
    } else if mutation != Some(Mutation::AcceptMissingWhitespace) {
        return Recognized::Boundary(Boundary::RequiresTokenizerParseError);
    }
    if at >= len {
        return Recognized::Boundary(Boundary::MalformedSelectedForm(Malformed::TruncatedAtEof));
    }
    if bytes[at] == b'>' {
        return Recognized::Boundary(Boundary::MalformedSelectedForm(Malformed::MissingName));
    }
    let name_start = at;
    while at < len && whitespace_len(source, at, unicode) == 0 && bytes[at] != b'>' {
        at += 1;
    }
    if at >= len {
        return Recognized::Boundary(Boundary::MalformedSelectedForm(Malformed::TruncatedAtEof));
    }
    let name_end = at;
    if !eq_ignore_ascii_case(&bytes[name_start..name_end], b"html") {
        return Recognized::Boundary(Boundary::BroaderDoctype(Broader::NameNotHtml));
    }
    while at < len {
        let step = whitespace_len(source, at, unicode);
        if step == 0 {
            break;
        }
        at += step;
    }
    if at >= len {
        return Recognized::Boundary(Boundary::MalformedSelectedForm(Malformed::TruncatedAtEof));
    }
    if bytes[at] == b'>' {
        return Recognized::Canonical {
            name: (name_start, name_end),
            end: at + 1,
        };
    }
    let rest = &bytes[at..];
    let broader = if starts_with_ci(rest, b"PUBLIC") {
        Some(Broader::PublicKeyword)
    } else if starts_with_ci(rest, b"SYSTEM") {
        Some(Broader::SystemKeyword)
    } else {
        None
    };
    match broader {
        Some(kind) => Recognized::Boundary(Boundary::BroaderDoctype(kind)),
        None => Recognized::Boundary(Boundary::MalformedSelectedForm(Malformed::AfterNameJunk)),
    }
}

fn span_of(anchor: &SourceAnchor) -> Span {
    Span {
        source: anchor.source_id().value(),
        range: (anchor.range().start(), anchor.range().end()),
    }
}

/// Builds the canonical token from anchors captured at recognition. The
/// spelling and interpreted name are read from the evidence just captured,
/// never recovered from the source afterwards.
fn canonical_token(source: &SourceText, complete: ByteRange, name: ByteRange) -> DoctypeToken {
    let complete_anchor = source
        .anchor(complete.0, complete.1)
        .expect("recognized complete range is a valid anchor");
    let name_anchor = source
        .anchor(name.0, name.1)
        .expect("recognized name range is a valid anchor");
    DoctypeToken {
        complete: span_of(&complete_anchor),
        name_source: span_of(&name_anchor),
        name_spelling: name_anchor.fragment().to_owned(),
        interpreted_name: name_anchor.fragment().to_ascii_lowercase(),
        public: Identifier::Missing,
        system: Identifier::Missing,
        force_quirks: ForceQuirks::Off,
    }
}

/// Interpreted (preprocessed) form: CRLF and lone CR become one LF each.
fn interpret_newlines(authored: &str) -> String {
    authored.replace("\r\n", "\n").replace('\r', "\n")
}

fn apply_token_mutation(source: &SourceText, token: &mut DoctypeToken, mutation: Option<Mutation>) {
    let Some(mutation) = mutation else { return };
    match mutation {
        Mutation::NameAnchorOnKeyword => token.name_source.range = (2, 9),
        Mutation::NameAnchorOnWhitespace => token.name_source.range = (9, 10),
        Mutation::CompleteTruncated => token.complete.range.1 -= 1,
        Mutation::CompleteShifted => {
            token.complete.range.0 += 1;
            token.complete.range.1 += 1;
        }
        Mutation::SourceIdCorrupted => token.name_source.source += 1,
        Mutation::NameSpellingLowercased => {
            token.name_spelling = token.name_spelling.to_ascii_lowercase();
        }
        Mutation::NameStartFromFixedOffset => {
            // Assumes exactly one whitespace byte after the keyword.
            token.name_source.range = (10, 14);
        }
        Mutation::NameFromCaseSensitiveSearch => {
            let found = source.as_str().find("html");
            token.name_source.range = found.map_or((0, 0), |start| (start, start + 4));
        }
        Mutation::CompleteEndFromInterpretedLength => {
            let authored = &source.as_str()[token.complete.range.0..token.complete.range.1];
            token.complete.range.1 = token.complete.range.0 + interpret_newlines(authored).len();
        }
        Mutation::RetokenizedSubstring => {
            let substring = &source.as_str()[token.complete.range.0..token.complete.range.1];
            let fresh = SourceText::new(SourceId::new(SOURCE_ID + 1_000), substring.to_owned());
            // The re-lexed copy is a different source identity.
            token.complete.source = fresh.id().value();
            token.name_source.source = fresh.id().value();
        }
        Mutation::InterpretedNameSvg => token.interpreted_name = "svg".to_owned(),
        Mutation::ForceQuirksOn => token.force_quirks = ForceQuirks::On,
        Mutation::PublicFabricatedEmpty => {
            let at = token.name_source.range.1;
            token.public = Identifier::Authored {
                span: Span {
                    source: token.name_source.source,
                    range: (at, at),
                },
                value: String::new(),
            };
        }
        Mutation::SystemFabricatedEmpty => {
            let at = token.name_source.range.1;
            token.system = Identifier::Authored {
                span: Span {
                    source: token.name_source.source,
                    range: (at, at),
                },
                value: String::new(),
            };
        }
        _ => {}
    }
}

enum LexEnd {
    Complete,
    Boundary(Boundary),
    Resource(Resource),
}

struct Lexed {
    tokens: Vec<Token>,
    end: LexEnd,
    retained_anchors: usize,
}

fn first_greater_than(source: &str, from: usize) -> Option<usize> {
    source.as_bytes()[from..]
        .iter()
        .position(|byte| *byte == b'>')
        .map(|offset| from + offset)
}

/// Commits one token under the preflight -> prepare -> commit ordering. A
/// refusal happens before any evidence is retained.
fn commit(
    lexed: &mut Lexed,
    budget: &mut Budget,
    token: Token,
    mutation: Option<Mutation>,
) -> Result<(), Resource> {
    if budget.tokens == 0 {
        return Err(Resource::Tokens);
    }
    let needed = token.anchors();
    if budget.anchors < needed {
        if mutation == Some(Mutation::PartialAnchorCommit) && needed > 1 {
            // Countermodel: retains the first anchor before discovering the
            // second cannot be afforded.
            lexed.retained_anchors += 1;
        }
        return Err(Resource::Anchors);
    }
    budget.tokens -= 1;
    budget.anchors -= needed;
    lexed.retained_anchors += needed;
    lexed.tokens.push(token);
    Ok(())
}

fn lex(source: &SourceText, budget: &mut Budget, mutation: Option<Mutation>) -> Lexed {
    let text = source.as_str();
    let bytes = text.as_bytes();
    let mut lexed = Lexed {
        tokens: Vec::new(),
        end: LexEnd::Complete,
        retained_anchors: 0,
    };
    let mut at = 0;
    loop {
        if at >= bytes.len() {
            let eof = Token::EndOfFile { at };
            if let Err(resource) = commit(&mut lexed, budget, eof, mutation) {
                lexed.end = LexEnd::Resource(resource);
            }
            return lexed;
        }
        let (token, next) = match next_token(source, at, mutation) {
            Ok(pair) => pair,
            Err(boundary) => {
                lexed.end = LexEnd::Boundary(boundary);
                return lexed;
            }
        };
        if let Err(resource) = commit(&mut lexed, budget, token, mutation) {
            lexed.end = LexEnd::Resource(resource);
            return lexed;
        }
        at = next;
    }
}

fn tag_span(source: &SourceText, range: ByteRange) -> Span {
    span_of(
        &source
            .anchor(range.0, range.1)
            .expect("recognized tag range is a valid anchor"),
    )
}

fn next_token(
    source: &SourceText,
    at: usize,
    mutation: Option<Mutation>,
) -> Result<(Token, usize), Boundary> {
    let text = source.as_str();
    let bytes = text.as_bytes();
    if bytes[at] != b'<' || at + 1 >= bytes.len() {
        return Err(Boundary::OutsideModelledToken);
    }
    match bytes[at + 1] {
        b'!' => {
            let rest = &bytes[at + 2..];
            if starts_with_ci(rest, b"DOCTYPE") {
                match recognize_doctype(text, at, mutation) {
                    Recognized::Canonical { name, end } => {
                        let mut token = canonical_token(source, (at, end), name);
                        apply_token_mutation(source, &mut token, mutation);
                        Ok((doctype_or_masquerade(token, mutation), end))
                    }
                    Recognized::Boundary(boundary) => widen(source, at, boundary, mutation),
                }
            } else if rest.starts_with(b"--") {
                widen(source, at, Boundary::OtherMarkup(Markup::Comment), mutation)
            } else if rest.starts_with(b"[CDATA[") {
                widen(source, at, Boundary::OtherMarkup(Markup::Cdata), mutation)
            } else {
                widen(source, at, Boundary::OtherMarkup(Markup::General), mutation)
            }
        }
        b'?' => widen(
            source,
            at,
            Boundary::OtherMarkup(Markup::ProcessingInstruction),
            mutation,
        ),
        b'/' => tag(source, at, at + 2, true),
        byte if byte.is_ascii_alphabetic() => tag(source, at, at + 1, false),
        _ => Err(Boundary::OutsideModelledToken),
    }
}

fn doctype_or_masquerade(token: DoctypeToken, mutation: Option<Mutation>) -> Token {
    match mutation {
        Some(Mutation::DoctypeAsStartTag) => Token::StartTag {
            name: TagName::Html,
            span: token.complete,
        },
        Some(Mutation::DoctypeAsText) => Token::Character {
            span: token.complete,
            interpreted: token.interpreted_name,
        },
        _ => Token::Doctype(token),
    }
}

/// Countermodels that widen the selected profile. The honest path returns the
/// boundary untouched.
fn widen(
    source: &SourceText,
    at: usize,
    boundary: Boundary,
    mutation: Option<Mutation>,
) -> Result<(Token, usize), Boundary> {
    let aliased = match (mutation, boundary) {
        (
            Some(Mutation::AcceptPublicSystem),
            Boundary::BroaderDoctype(Broader::PublicKeyword | Broader::SystemKeyword),
        ) => true,
        (Some(Mutation::AcceptMissingWhitespace), Boundary::RequiresTokenizerParseError) => true,
        (Some(Mutation::AliasMarkup(wanted)), Boundary::OtherMarkup(actual)) => wanted == actual,
        _ => false,
    };
    if !aliased {
        return Err(boundary);
    }
    let end = first_greater_than(source.as_str(), at)
        .map(|index| index + 1)
        .ok_or(boundary)?;
    // Any in-bounds ASCII range will do: the countermodel is judged on the
    // fact that a boundary became a canonical DOCTYPE at all.
    let name = (at + 2, (at + 6).min(end));
    Ok((
        doctype_or_masquerade(canonical_token(source, (at, end), name), None),
        end,
    ))
}

fn tag(
    source: &SourceText,
    at: usize,
    name_start: usize,
    is_end: bool,
) -> Result<(Token, usize), Boundary> {
    let bytes = source.as_str().as_bytes();
    let mut cursor = name_start;
    while cursor < bytes.len() && bytes[cursor].is_ascii_alphanumeric() {
        cursor += 1;
    }
    let name = &bytes[name_start..cursor];
    if cursor >= bytes.len() || bytes[cursor] != b'>' {
        return Err(Boundary::OutsideModelledToken);
    }
    let tag_name = if eq_ignore_ascii_case(name, b"html") {
        TagName::Html
    } else if eq_ignore_ascii_case(name, b"body") {
        TagName::Body
    } else {
        return Err(Boundary::OutsideModelledToken);
    };
    let span = tag_span(source, (at, cursor + 1));
    let token = if is_end {
        Token::EndTag {
            name: tag_name,
            span,
        }
    } else {
        Token::StartTag {
            name: tag_name,
            span,
        }
    };
    Ok((token, cursor + 1))
}

// ---------------------------------------------------------------------------
// 4. Independent tree machine (Initial / BeforeHtml / BeforeHead only)
// ---------------------------------------------------------------------------

struct Machine {
    mode: Mode,
    doc_mode: DocMode,
    diagnostics: Vec<Diag>,
    children: Vec<Child>,
    dispatches: Vec<Dispatch>,
    diagnostics_left: usize,
    mutation: Option<Mutation>,
    stop: Option<Completion>,
    stopped_at: Option<usize>,
}

impl Machine {
    fn new(budget: &Budget, mutation: Option<Mutation>) -> Self {
        Self {
            mode: Mode::Initial,
            // A new Document starts in no-quirks mode.
            doc_mode: DocMode::NoQuirks,
            diagnostics: Vec::new(),
            children: Vec::new(),
            dispatches: Vec::new(),
            diagnostics_left: budget.diagnostics,
            mutation,
            stop: None,
            stopped_at: None,
        }
    }

    fn preflight_diagnostics(&self, count: usize) -> Result<(), Resource> {
        if self.diagnostics_left < count {
            Err(Resource::Diagnostics)
        } else {
            Ok(())
        }
    }

    fn push_diag(&mut self, code: DiagCode, token_index: usize) {
        self.diagnostics_left -= 1;
        self.diagnostics.push(Diag { code, token_index });
    }

    /// Dispatches one token, following reprocessing until it is consumed or a
    /// non-consuming outcome stops the run. Insertion modes are never revisited
    /// by the same token: the loop is bounded by the number of modes.
    fn dispatch(&mut self, index: usize, token: &Token) {
        let mut steps: Vec<Step> = Vec::new();
        loop {
            assert!(
                steps.len() <= ALL_MODES.len(),
                "a single token must not cycle through insertion modes"
            );
            let mode = self.mode;
            let outcome = self.step(mode, index, token);
            steps.push(Step { mode, outcome });
            match outcome {
                Outcome::Consumed { next } => {
                    self.mode = next;
                    break;
                }
                Outcome::Reprocess { next } => {
                    self.mode = next;
                }
                Outcome::OutsideSelectedTreeProfile => {
                    self.stop = Some(Completion::TreeRefused);
                    break;
                }
                Outcome::ModelBoundary => {
                    self.stopped_at = Some(index);
                    break;
                }
                Outcome::ResourceRefused(resource) => {
                    self.stop = Some(Completion::ResourceLimited(resource));
                    if self.mutation == Some(Mutation::AdvanceModeOnRefusal) {
                        self.mode = Mode::BeforeHtml;
                    }
                    break;
                }
            }
        }
        self.dispatches.push(Dispatch {
            token_index: index,
            steps,
        });
    }

    fn step(&mut self, mode: Mode, index: usize, token: &Token) -> Outcome {
        if let Token::Doctype(doctype) = token {
            return match mode {
                Mode::Initial => self.initial_doctype(index, doctype),
                _ if self.mutation == Some(Mutation::AcceptNonInitial) => {
                    self.initial_doctype(index, doctype)
                }
                _ => Outcome::OutsideSelectedTreeProfile,
            };
        }
        match mode {
            Mode::Initial => {
                // Anything else: MissingDoctype, quirks, before html, reprocess.
                if let Err(resource) = self.preflight_diagnostics(1) {
                    return Outcome::ResourceRefused(resource);
                }
                self.push_diag(DiagCode::MissingDoctype, index);
                self.doc_mode = DocMode::Quirks;
                Outcome::Reprocess {
                    next: Mode::BeforeHtml,
                }
            }
            Mode::BeforeHtml => match token {
                Token::StartTag {
                    name: TagName::Html,
                    span,
                } => {
                    self.children
                        .push(Child::Html(ElementOrigin::Authored(*span)));
                    Outcome::Consumed {
                        next: Mode::BeforeHead,
                    }
                }
                _ => {
                    self.children
                        .push(Child::Html(ElementOrigin::SynthesizedByBeforeHtml));
                    Outcome::Reprocess {
                        next: Mode::BeforeHead,
                    }
                }
            },
            Mode::BeforeHead | Mode::InBody | Mode::AfterBody | Mode::AfterAfterBody => {
                Outcome::ModelBoundary
            }
        }
    }

    /// The Initial-mode DOCTYPE cell, restricted to what the selected theorem
    /// needs. Identifier tables are not modelled, so a non-empty authored
    /// identifier stops at the model boundary.
    fn initial_doctype(&mut self, index: usize, doctype: &DoctypeToken) -> Outcome {
        let non_empty = |identifier: &Identifier| matches!(identifier, Identifier::Authored { value, .. } if !value.is_empty());
        if non_empty(&doctype.public) || non_empty(&doctype.system) {
            return Outcome::ModelBoundary;
        }
        let parse_error = doctype.interpreted_name != "html"
            || doctype.public != Identifier::Missing
            || doctype.system != Identifier::Missing;
        if let Err(resource) = self.preflight_diagnostics(usize::from(parse_error)) {
            return Outcome::ResourceRefused(resource);
        }
        let has_doctype_or_element = !self.children.is_empty();
        if !has_doctype_or_element && self.mutation != Some(Mutation::DropDurableEvidence) {
            self.children.push(Child::DocumentType(doctype.clone()));
        }
        if doctype.force_quirks == ForceQuirks::On || doctype.interpreted_name != "html" {
            self.doc_mode = DocMode::Quirks;
        }
        if self.mutation == Some(Mutation::QuirksOnCanonical) {
            self.doc_mode = DocMode::Quirks;
        }
        if parse_error {
            self.push_diag(DiagCode::InitialDoctypeParseError, index);
        }
        if self.mutation == Some(Mutation::MissingDoctypePollution) {
            self.diagnostics_left = self.diagnostics_left.saturating_sub(1);
            self.diagnostics.push(Diag {
                code: DiagCode::MissingDoctype,
                token_index: index,
            });
        }
        if self.mutation == Some(Mutation::ReprocessCanonical) {
            return Outcome::Reprocess {
                next: Mode::BeforeHtml,
            };
        }
        Outcome::Consumed {
            next: Mode::BeforeHtml,
        }
    }
}

fn observe(bytes: &[u8], budget: Budget, mutation: Option<Mutation>) -> Observation {
    let source = SourceText::new(SourceId::new(SOURCE_ID), text_of(bytes));
    let mut lex_budget = budget;
    let lexed = lex(&source, &mut lex_budget, mutation);
    let mut machine = Machine::new(&budget, mutation);
    for (index, token) in lexed.tokens.iter().enumerate() {
        machine.dispatch(index, token);
        if machine.stop.is_some() || machine.stopped_at.is_some() {
            break;
        }
    }
    // Precedence: tree-level refusal > lower-layer incomplete > checkpoint.
    let lower = match lexed.end {
        LexEnd::Complete => None,
        LexEnd::Boundary(boundary) => Some(Completion::Boundary(boundary)),
        LexEnd::Resource(resource) => Some(Completion::ResourceLimited(resource)),
    };
    let completion = match (machine.stop, lower) {
        (Some(stop), _) => stop,
        (None, Some(_)) if mutation == Some(Mutation::UpgradeIncomplete) => {
            Completion::CheckpointReached {
                stopped_at: machine.stopped_at,
            }
        }
        (None, Some(lower)) => lower,
        (None, None) => Completion::CheckpointReached {
            stopped_at: machine.stopped_at,
        },
    };
    Observation {
        tokens: lexed.tokens,
        dispatches: machine.dispatches,
        diagnostics: machine.diagnostics,
        doc_mode: machine.doc_mode,
        mode: machine.mode,
        children: machine.children,
        retained_anchors: lexed.retained_anchors,
        completion,
    }
}

fn observe_plain(bytes: &[u8]) -> Observation {
    observe(bytes, Budget::GENEROUS, None)
}

// ---------------------------------------------------------------------------
// 5. Hand-authored GOLD (BEGIN GOLD / END GOLD is checked by a tripwire test)
// ---------------------------------------------------------------------------

// BEGIN GOLD
fn sp(range: ByteRange) -> Span {
    Span {
        source: SOURCE_ID,
        range,
    }
}

fn gold_doctype(complete: ByteRange, name: ByteRange, spelling: &str) -> DoctypeToken {
    DoctypeToken {
        complete: sp(complete),
        name_source: sp(name),
        name_spelling: spelling.to_owned(),
        interpreted_name: "html".to_owned(),
        public: Identifier::Missing,
        system: Identifier::Missing,
        force_quirks: ForceQuirks::Off,
    }
}

fn consumed(mode: Mode, next: Mode) -> Step {
    Step {
        mode,
        outcome: Outcome::Consumed { next },
    }
}

fn reprocess(mode: Mode, next: Mode) -> Step {
    Step {
        mode,
        outcome: Outcome::Reprocess { next },
    }
}

fn boundary_step(mode: Mode) -> Step {
    Step {
        mode,
        outcome: Outcome::ModelBoundary,
    }
}

fn dispatch(token_index: usize, steps: Vec<Step>) -> Dispatch {
    Dispatch { token_index, steps }
}

fn missing_doctype(token_index: usize) -> Diag {
    Diag {
        code: DiagCode::MissingDoctype,
        token_index,
    }
}

/// The canonical DOCTYPE theorem, composed with the fixture's tail.
fn gold(fixture: &Accepted) -> Observation {
    let doctype = gold_doctype(fixture.complete, fixture.name, fixture.spelling);
    let end = fixture.bytes.len();
    // Initial consumes the DOCTYPE exactly once: one step, no MissingDoctype,
    // NoQuirks, switch to BeforeHtml, no reprocess.
    let initial = dispatch(0, vec![consumed(Mode::Initial, Mode::BeforeHtml)]);
    let mut tokens = vec![Token::Doctype(doctype.clone())];
    let mut dispatches = vec![initial];
    let mut children = vec![Child::DocumentType(doctype)];
    let stopped_at;
    let mut anchors = 2;
    match fixture.tail {
        Tail::None => {
            tokens.push(Token::EndOfFile { at: end });
            dispatches.push(dispatch(
                1,
                vec![
                    reprocess(Mode::BeforeHtml, Mode::BeforeHead),
                    boundary_step(Mode::BeforeHead),
                ],
            ));
            children.push(Child::Html(ElementOrigin::SynthesizedByBeforeHtml));
            stopped_at = 1;
            anchors += 1;
        }
        Tail::HtmlPair { start, end: close } => {
            tokens.push(Token::StartTag {
                name: TagName::Html,
                span: sp(start),
            });
            tokens.push(Token::EndTag {
                name: TagName::Html,
                span: sp(close),
            });
            tokens.push(Token::EndOfFile { at: end });
            dispatches.push(dispatch(
                1,
                vec![consumed(Mode::BeforeHtml, Mode::BeforeHead)],
            ));
            dispatches.push(dispatch(2, vec![boundary_step(Mode::BeforeHead)]));
            children.push(Child::Html(ElementOrigin::Authored(sp(start))));
            stopped_at = 2;
            anchors += 3;
        }
        Tail::BodyShell {
            body,
            body_end,
            html_end,
        } => {
            tokens.push(Token::StartTag {
                name: TagName::Body,
                span: sp(body),
            });
            tokens.push(Token::EndTag {
                name: TagName::Body,
                span: sp(body_end),
            });
            tokens.push(Token::EndTag {
                name: TagName::Html,
                span: sp(html_end),
            });
            tokens.push(Token::EndOfFile { at: end });
            dispatches.push(dispatch(
                1,
                vec![
                    reprocess(Mode::BeforeHtml, Mode::BeforeHead),
                    boundary_step(Mode::BeforeHead),
                ],
            ));
            children.push(Child::Html(ElementOrigin::SynthesizedByBeforeHtml));
            stopped_at = 1;
            anchors += 4;
        }
    }
    Observation {
        tokens,
        dispatches,
        diagnostics: Vec::new(),
        doc_mode: DocMode::NoQuirks,
        mode: Mode::BeforeHead,
        children,
        retained_anchors: anchors,
        completion: Completion::CheckpointReached {
            stopped_at: Some(stopped_at),
        },
    }
}

/// A standalone outside-profile source publishes no token and no tree fact.
fn gold_boundary(boundary: Boundary) -> Observation {
    Observation {
        tokens: Vec::new(),
        dispatches: Vec::new(),
        diagnostics: Vec::new(),
        doc_mode: DocMode::NoQuirks,
        mode: Mode::Initial,
        children: Vec::new(),
        retained_anchors: 0,
        completion: Completion::Boundary(boundary),
    }
}

/// `<body>`: the missing-DOCTYPE predecessor.
fn gold_missing_doctype_body() -> Observation {
    Observation {
        tokens: vec![
            Token::StartTag {
                name: TagName::Body,
                span: sp((0, 6)),
            },
            Token::EndOfFile { at: 6 },
        ],
        dispatches: vec![dispatch(
            0,
            vec![
                reprocess(Mode::Initial, Mode::BeforeHtml),
                reprocess(Mode::BeforeHtml, Mode::BeforeHead),
                boundary_step(Mode::BeforeHead),
            ],
        )],
        diagnostics: vec![missing_doctype(0)],
        doc_mode: DocMode::Quirks,
        mode: Mode::BeforeHead,
        children: vec![Child::Html(ElementOrigin::SynthesizedByBeforeHtml)],
        retained_anchors: 2,
        completion: Completion::CheckpointReached {
            stopped_at: Some(0),
        },
    }
}

/// `<html><!DOCTYPE html>`: DOCTYPE after the document left Initial.
fn gold_html_then_doctype() -> Observation {
    Observation {
        tokens: vec![
            Token::StartTag {
                name: TagName::Html,
                span: sp((0, 6)),
            },
            Token::Doctype(gold_doctype((6, 21), (16, 20), "html")),
            Token::EndOfFile { at: 21 },
        ],
        dispatches: vec![
            dispatch(
                0,
                vec![
                    reprocess(Mode::Initial, Mode::BeforeHtml),
                    consumed(Mode::BeforeHtml, Mode::BeforeHead),
                ],
            ),
            dispatch(
                1,
                vec![Step {
                    mode: Mode::BeforeHead,
                    outcome: Outcome::OutsideSelectedTreeProfile,
                }],
            ),
        ],
        diagnostics: vec![missing_doctype(0)],
        doc_mode: DocMode::Quirks,
        mode: Mode::BeforeHead,
        children: vec![Child::Html(ElementOrigin::Authored(sp((0, 6))))],
        retained_anchors: 4,
        completion: Completion::TreeRefused,
    }
}

/// `<!DOCTYPE html><!DOCTYPE html>`: a second DOCTYPE in BeforeHtml.
fn gold_second_doctype() -> Observation {
    let first = gold_doctype((0, 15), (10, 14), "html");
    Observation {
        tokens: vec![
            Token::Doctype(first.clone()),
            Token::Doctype(gold_doctype((15, 30), (25, 29), "html")),
            Token::EndOfFile { at: 30 },
        ],
        dispatches: vec![
            dispatch(0, vec![consumed(Mode::Initial, Mode::BeforeHtml)]),
            dispatch(
                1,
                vec![Step {
                    mode: Mode::BeforeHtml,
                    outcome: Outcome::OutsideSelectedTreeProfile,
                }],
            ),
        ],
        diagnostics: Vec::new(),
        doc_mode: DocMode::NoQuirks,
        mode: Mode::BeforeHtml,
        children: vec![Child::DocumentType(first)],
        retained_anchors: 5,
        completion: Completion::TreeRefused,
    }
}

/// `<!DOCTYPE html>` followed by a boundary markup: the DOCTYPE is a valid
/// committed prefix, but the run is not complete.
fn gold_doctype_then_boundary(boundary: Boundary) -> Observation {
    let doctype = gold_doctype((0, 15), (10, 14), "html");
    Observation {
        tokens: vec![Token::Doctype(doctype.clone())],
        dispatches: vec![dispatch(0, vec![consumed(Mode::Initial, Mode::BeforeHtml)])],
        diagnostics: Vec::new(),
        doc_mode: DocMode::NoQuirks,
        mode: Mode::BeforeHtml,
        children: vec![Child::DocumentType(doctype)],
        retained_anchors: 2,
        completion: Completion::Boundary(boundary),
    }
}
// END GOLD

// ---------------------------------------------------------------------------
// 6. Dimension comparison
// ---------------------------------------------------------------------------

fn doctype_count(tokens: &[Token]) -> usize {
    tokens
        .iter()
        .filter(|token| matches!(token, Token::Doctype(_)))
        .count()
}

fn constructed_shape(children: &[Child]) -> Vec<Option<&ElementOrigin>> {
    children
        .iter()
        .map(|child| match child {
            Child::DocumentType(_) => None,
            Child::Html(origin) => Some(origin),
        })
        .collect()
}

fn dims(expected: &Observation, got: &Observation) -> BTreeSet<Dim> {
    let mut out = BTreeSet::new();
    if doctype_count(&expected.tokens) != doctype_count(&got.tokens) {
        out.insert(Dim::Acceptance);
    } else if expected.tokens.len() == got.tokens.len() {
        for (want, have) in expected.tokens.iter().zip(&got.tokens) {
            match (want, have) {
                (Token::Doctype(want), Token::Doctype(have)) => {
                    if want.complete.range != have.complete.range {
                        out.insert(Dim::CompleteRange);
                    }
                    if want.name_source.range != have.name_source.range {
                        out.insert(Dim::NameRange);
                    }
                    if want.complete.source != have.complete.source
                        || want.name_source.source != have.name_source.source
                    {
                        out.insert(Dim::SourceIdentity);
                    }
                    if want.name_spelling != have.name_spelling {
                        out.insert(Dim::NameSpelling);
                    }
                    if want.interpreted_name != have.interpreted_name {
                        out.insert(Dim::InterpretedName);
                    }
                    if want.public != have.public {
                        out.insert(Dim::PublicIdentifier);
                    }
                    if want.system != have.system {
                        out.insert(Dim::SystemIdentifier);
                    }
                    if want.force_quirks != have.force_quirks {
                        out.insert(Dim::ForceQuirks);
                    }
                }
                (Token::Doctype(_), _) | (_, Token::Doctype(_)) => {
                    out.insert(Dim::TokenKind);
                }
                (want, have) if want != have => {
                    out.insert(Dim::OtherTokens);
                }
                _ => {}
            }
        }
    } else {
        out.insert(Dim::OtherTokens);
    }
    if expected.dispatches != got.dispatches {
        out.insert(Dim::Dispatch);
    }
    if expected.diagnostics != got.diagnostics {
        out.insert(Dim::Diagnostics);
    }
    if expected.doc_mode != got.doc_mode {
        out.insert(Dim::DocumentMode);
    }
    if expected.mode != got.mode {
        out.insert(Dim::ModeTransition);
    }
    // DocumentType payload evidence is attributed to the token dimensions
    // above; this dimension is the constructed shape and ordering.
    if constructed_shape(&expected.children) != constructed_shape(&got.children) {
        out.insert(Dim::Constructed);
    }
    if expected.retained_anchors != got.retained_anchors {
        out.insert(Dim::RetainedEvidence);
    }
    if expected.completion != got.completion {
        out.insert(Dim::Completion);
    }
    out
}

// ---------------------------------------------------------------------------
// 7. Durable representation alternatives (semantic comparison only)
// ---------------------------------------------------------------------------

/// Premise P1: WHATWG tree construction creates a DocumentType child of the
/// Document for the selected canonical DOCTYPE (Initial insertion mode).
const WHATWG_CONSTRUCTS_DOCUMENT_TYPE_CHILD: bool = true;

/// Test-private, result-scoped, opaque constructed identity.
///
/// Architecture (HTML_TREE_CONSTRUCTION.md): identity is scoped to one parse
/// result; distinct from `SourceId`, ranges, token indexes, browser identity,
/// private storage identity and final placement; and gives no cross-result
/// stability promise. The fields are private to this module, so nothing else in
/// this file can do arithmetic on them or convert them to or from another
/// domain. `==` is representation equality only; whether two ids denote the
/// same constructed observation can only be asked through `same_observation`,
/// which refuses to compare ids from different result scopes.
///
/// This is a validation vocabulary, not a production encoding or placement.
mod identity {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) struct ResultScope(pub(super) u32);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) struct ConstructedId {
        scope: ResultScope,
        mint: u32,
    }

    impl ConstructedId {
        pub(super) fn scope(self) -> ResultScope {
            self.scope
        }

        /// `None` when the ids belong to independent result scopes: they are
        /// not comparable as the same durable observation.
        pub(super) fn same_observation(self, other: Self) -> Option<bool> {
            (self.scope == other.scope).then_some(self.mint == other.mint)
        }

        /// Deliberately wrong: derives an id from another domain's value. Used
        /// only to prove the independence checks can fail.
        pub(super) fn derived_by_countermodel(scope: ResultScope, from: u64) -> Self {
            Self {
                scope,
                mint: u32::try_from(from).expect("small countermodel value"),
            }
        }
    }

    /// Mints ids for one result scope, deterministically and opaquely.
    pub(super) struct Minter {
        scope: ResultScope,
        next: u32,
    }

    impl Minter {
        pub(super) fn new(scope: ResultScope) -> Self {
            Self { scope, next: 0 }
        }

        pub(super) fn mint(&mut self) -> ConstructedId {
            let id = ConstructedId {
                scope: self.scope,
                mint: self.next,
            };
            self.next += 1;
            id
        }
    }
}

use identity::{ConstructedId, Minter, ResultScope};

const SCOPE_1: ResultScope = ResultScope(1);
const SCOPE_2: ResultScope = ResultScope(2);

/// How constructed identities are assigned. Only `Opaque` is the validated
/// model; the others are countermodels, each deriving identity from exactly one
/// domain the architecture forbids as the sole definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentityScheme {
    Opaque,
    FromPlacement,
    FromRange,
    FromSourceId,
    FromDocumentMode,
}

/// Structural perturbation of the Document child list. It is not a recovery
/// claim: the selected slice has no placement-changing recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    Final,
    Reversed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    Document,
    Element,
    DocumentType,
    Text,
}

const ALL_NODE_KINDS: [NodeKind; 4] = [
    NodeKind::Document,
    NodeKind::Element,
    NodeKind::DocumentType,
    NodeKind::Text,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildKind {
    DocumentType,
    Element(TagName),
}

/// Identities of one result's constructed nodes, minted when the nodes are
/// created (one per constructed child of `Observation::children`, in creation
/// order). Placement is a separate fact and never an input of the opaque
/// scheme.
struct Minted {
    scope: ResultScope,
    document: ConstructedId,
    children: Vec<ConstructedId>,
}

fn final_index(placement: Placement, index: usize, total: usize) -> usize {
    match placement {
        Placement::Final => index,
        Placement::Reversed => total - 1 - index,
    }
}

fn mint(
    observation: &Observation,
    scope: ResultScope,
    scheme: IdentityScheme,
    placement: Placement,
) -> Minted {
    let total = observation.children.len();
    if scheme == IdentityScheme::Opaque {
        let mut minter = Minter::new(scope);
        let document = minter.mint();
        let children = observation.children.iter().map(|_| minter.mint()).collect();
        return Minted {
            scope,
            document,
            children,
        };
    }
    let derived = |value: u64| ConstructedId::derived_by_countermodel(scope, value);
    let children = observation
        .children
        .iter()
        .enumerate()
        .map(|(index, child)| {
            // The offset keeps ids distinct inside one result so lookups work.
            let unique = 10_000 * (u64::try_from(index).expect("small") + 1);
            let value = match scheme {
                IdentityScheme::Opaque => unreachable!("handled above"),
                IdentityScheme::FromPlacement => {
                    u64::try_from(final_index(placement, index, total)).expect("small") + 1
                }
                IdentityScheme::FromRange => {
                    let end = match child {
                        Child::DocumentType(token) => token.complete.range.1,
                        Child::Html(ElementOrigin::Authored(span)) => span.range.1,
                        Child::Html(ElementOrigin::SynthesizedByBeforeHtml) => 0,
                    };
                    unique + u64::try_from(end).expect("small")
                }
                IdentityScheme::FromSourceId => {
                    let source = match child {
                        Child::DocumentType(token) => {
                            token.complete.source + token.name_source.source
                        }
                        Child::Html(ElementOrigin::Authored(span)) => span.source,
                        Child::Html(ElementOrigin::SynthesizedByBeforeHtml) => 0,
                    };
                    unique + source
                }
                IdentityScheme::FromDocumentMode => {
                    unique
                        + match observation.doc_mode {
                            DocMode::NoQuirks => 1,
                            DocMode::Quirks => 2,
                        }
                }
            };
            derived(value)
        })
        .collect();
    Minted {
        scope,
        document: derived(0),
        children,
    }
}

/// Document-level fact, distinct from both authored identity and node identity.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DoctypeMeaning {
    complete: Span,
    name_source: Span,
    interpreted_name: String,
    public: Identifier,
    system: Identifier,
}

fn meaning_of(token: &DoctypeToken) -> DoctypeMeaning {
    DoctypeMeaning {
        complete: token.complete,
        name_source: token.name_source,
        interpreted_name: token.interpreted_name.clone(),
        public: token.public.clone(),
        system: token.system.clone(),
    }
}

/// Option A: a distinct constructed DocumentType node domain. Identity,
/// authored evidence (`DoctypeMeaning`), placement (`children`) and document
/// mode are four separately stated facts.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ReprA {
    scope: ResultScope,
    nodes: Vec<NodeA>,
    document_mode: DocMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeA {
    id: ConstructedId,
    kind: NodeKind,
    element: Option<ElementOrigin>,
    doctype: Option<DoctypeMeaning>,
    children: Vec<ConstructedId>,
}

/// Option B: a distinct source-backed evidence relation outside the node
/// domains. It carries no constructed identity and no placement.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ReprB {
    scope: ResultScope,
    document: ConstructedId,
    elements: Vec<ElementB>,
    document_children: Vec<ConstructedId>,
    evidence: Option<DoctypeMeaning>,
    document_mode: DocMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ElementB {
    id: ConstructedId,
    origin: ElementOrigin,
}

/// Option B given the two facts it lacks: a constructed identity (minted like
/// any other constructed node, not derived from the placement) and an explicit
/// Document placement.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ReprBPlus {
    relation: ReprB,
    relation_identity: ConstructedId,
    relation_placement: usize,
}

fn project_a(observation: &Observation, minted: &Minted, placement: Placement) -> ReprA {
    let total = observation.children.len();
    let mut nodes = vec![NodeA {
        id: minted.document,
        kind: NodeKind::Document,
        element: None,
        doctype: None,
        children: Vec::new(),
    }];
    for (child, id) in observation.children.iter().zip(&minted.children) {
        nodes.push(match child {
            Child::DocumentType(token) => NodeA {
                id: *id,
                kind: NodeKind::DocumentType,
                element: None,
                doctype: Some(meaning_of(token)),
                children: Vec::new(),
            },
            Child::Html(origin) => NodeA {
                id: *id,
                kind: NodeKind::Element,
                element: Some(origin.clone()),
                doctype: None,
                children: Vec::new(),
            },
        });
    }
    let mut ordered = vec![None; total];
    for (index, id) in minted.children.iter().enumerate() {
        ordered[final_index(placement, index, total)] = Some(*id);
    }
    nodes[0].children = ordered.into_iter().flatten().collect();
    ReprA {
        scope: minted.scope,
        nodes,
        document_mode: observation.doc_mode,
    }
}

fn project_b(observation: &Observation, minted: &Minted) -> ReprB {
    let mut elements = Vec::new();
    let mut evidence = None;
    for (child, id) in observation.children.iter().zip(&minted.children) {
        match child {
            Child::DocumentType(token) => evidence = Some(meaning_of(token)),
            Child::Html(origin) => elements.push(ElementB {
                id: *id,
                origin: origin.clone(),
            }),
        }
    }
    let document_children = elements.iter().map(|element| element.id).collect();
    ReprB {
        scope: minted.scope,
        document: minted.document,
        elements,
        document_children,
        evidence,
        document_mode: observation.doc_mode,
    }
}

fn doctype_child_index(observation: &Observation) -> usize {
    observation
        .children
        .iter()
        .position(|child| matches!(child, Child::DocumentType(_)))
        .expect("accepted fixtures own a DocumentType child")
}

/// Gives B its missing identity and placement. The identity is the one minted
/// for that constructed observation; the placement is stated beside it.
fn extend_b(observation: &Observation, minted: &Minted, placement: Placement) -> ReprBPlus {
    let index = doctype_child_index(observation);
    ReprBPlus {
        relation: project_b(observation, minted),
        relation_identity: minted.children[index],
        relation_placement: final_index(placement, index, observation.children.len()),
    }
}

/// The extended relation, read back as a constructed node.
fn b_plus_as_a(extended: &ReprBPlus) -> ReprA {
    let relation = &extended.relation;
    let mut nodes = vec![NodeA {
        id: relation.document,
        kind: NodeKind::Document,
        element: None,
        doctype: None,
        children: Vec::new(),
    }];
    let total = relation.elements.len() + 1;
    let mut elements = relation.elements.iter();
    for position in 0..total {
        let node = if position == extended.relation_placement {
            NodeA {
                id: extended.relation_identity,
                kind: NodeKind::DocumentType,
                element: None,
                doctype: relation.evidence.clone(),
                children: Vec::new(),
            }
        } else {
            let element = elements.next().expect("one element per remaining position");
            NodeA {
                id: element.id,
                kind: NodeKind::Element,
                element: Some(element.origin.clone()),
                doctype: None,
                children: Vec::new(),
            }
        };
        nodes[0].children.push(node.id);
        nodes.push(node);
    }
    ReprA {
        scope: relation.scope,
        nodes,
        document_mode: relation.document_mode,
    }
}

/// Option A and B as built for one result scope, with the validated identity
/// model and the final placement.
fn repr_a(observation: &Observation, scope: ResultScope) -> ReprA {
    let minted = mint(observation, scope, IdentityScheme::Opaque, Placement::Final);
    project_a(observation, &minted, Placement::Final)
}

fn repr_b(observation: &Observation, scope: ResultScope) -> ReprB {
    let minted = mint(observation, scope, IdentityScheme::Opaque, Placement::Final);
    project_b(observation, &minted)
}

/// What a consumer can state from one representation, read from its structure.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Answers {
    scope: ResultScope,
    authored_complete: Option<Span>,
    authored_name: Option<Span>,
    identifiers_explicitly_missing: bool,
    fabricated_identifier_evidence: bool,
    document_child_sequence: Vec<ChildKind>,
    constructed_identity: Option<ConstructedId>,
    document_mode: DocMode,
}

fn is_fabricated(identifier: &Identifier) -> bool {
    matches!(identifier, Identifier::Authored { span, .. } if span.range.0 == span.range.1)
}

fn answers_from_meaning(
    meaning: Option<&DoctypeMeaning>,
) -> (Option<Span>, Option<Span>, bool, bool) {
    match meaning {
        Some(meaning) => (
            Some(meaning.complete),
            Some(meaning.name_source),
            meaning.public == Identifier::Missing && meaning.system == Identifier::Missing,
            is_fabricated(&meaning.public) || is_fabricated(&meaning.system),
        ),
        None => (None, None, false, false),
    }
}

fn answers_a(repr: &ReprA) -> Answers {
    let document = &repr.nodes[0];
    let doctype_node = repr
        .nodes
        .iter()
        .find(|node| node.kind == NodeKind::DocumentType);
    let (complete, name, missing, fabricated) =
        answers_from_meaning(doctype_node.and_then(|node| node.doctype.as_ref()));
    let sequence = document
        .children
        .iter()
        .map(|id| {
            let node = repr
                .nodes
                .iter()
                .find(|node| node.id == *id)
                .expect("child exists");
            match node.kind {
                NodeKind::DocumentType => ChildKind::DocumentType,
                _ => ChildKind::Element(TagName::Html),
            }
        })
        .collect();
    Answers {
        scope: repr.scope,
        authored_complete: complete,
        authored_name: name,
        identifiers_explicitly_missing: missing,
        fabricated_identifier_evidence: fabricated,
        document_child_sequence: sequence,
        constructed_identity: doctype_node.map(|node| node.id),
        document_mode: repr.document_mode,
    }
}

fn answers_b(repr: &ReprB) -> Answers {
    let (complete, name, missing, fabricated) = answers_from_meaning(repr.evidence.as_ref());
    Answers {
        scope: repr.scope,
        authored_complete: complete,
        authored_name: name,
        identifiers_explicitly_missing: missing,
        fabricated_identifier_evidence: fabricated,
        // Only constructed nodes appear as Document children.
        document_child_sequence: repr
            .document_children
            .iter()
            .map(|_| ChildKind::Element(TagName::Html))
            .collect(),
        constructed_identity: None,
        document_mode: repr.document_mode,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Question {
    Q1AuthoredIdentity,
    Q2ChildOrdering,
    Q3ThreeWayDistinction,
    Q4NoFabrication,
    Q5NoSilentOmission,
    Q6ResultScopedOpaqueIdentity,
    Q7ArchitectureCompatible,
}

const ALL_QUESTIONS: [Question; 7] = [
    Question::Q1AuthoredIdentity,
    Question::Q2ChildOrdering,
    Question::Q3ThreeWayDistinction,
    Question::Q4NoFabrication,
    Question::Q5NoSilentOmission,
    Question::Q6ResultScopedOpaqueIdentity,
    Question::Q7ArchitectureCompatible,
];

/// The normative observable for an accepted fixture, from the Standard:
/// Document children are `[DocumentType, html]`.
fn normative_child_sequence() -> Vec<ChildKind> {
    vec![ChildKind::DocumentType, ChildKind::Element(TagName::Html)]
}

/// Judges one representation of one result against the selected theorem. No
/// second result takes part: nothing here compares identities across results.
fn verdict(answers: &Answers, fixture: &Accepted) -> BTreeSet<Question> {
    let mut failed = BTreeSet::new();
    // Q1: exact authored complete and name evidence.
    if answers.authored_complete != Some(sp(fixture.complete))
        || answers.authored_name != Some(sp(fixture.name))
    {
        failed.insert(Question::Q1AuthoredIdentity);
    }
    // Q2: final constructed Document child ordering including DocumentType.
    if answers.document_child_sequence != normative_child_sequence() {
        failed.insert(Question::Q2ChildOrdering);
    }
    // Q3: authored identity, document mode and constructed identity are three
    // separately stated facts.
    if answers.authored_complete.is_none()
        || answers.constructed_identity.is_none()
        || answers.document_mode != DocMode::NoQuirks
    {
        failed.insert(Question::Q3ThreeWayDistinction);
    }
    // Q4: no fabricated source range or empty identifier evidence, and
    // Missing stated explicitly.
    if answers.fabricated_identifier_evidence || !answers.identifiers_explicitly_missing {
        failed.insert(Question::Q4NoFabrication);
    }
    // Q5: every normative constructed fact (existence, placement, identity)
    // is representable when the Standard constructs the node.
    if WHATWG_CONSTRUCTS_DOCUMENT_TYPE_CHILD
        && (!answers
            .document_child_sequence
            .contains(&ChildKind::DocumentType)
            || answers.constructed_identity.is_none())
    {
        failed.insert(Question::Q5NoSilentOmission);
    }
    // Q6: the DocumentType owns an opaque constructed identity that belongs to
    // this result's scope. The identity's independence from source, range,
    // placement and mode is proved separately by the scheme tests, and no
    // equality with an identity from another result is asked for.
    if !answers
        .constructed_identity
        .is_some_and(|id| id.scope() == answers.scope)
    {
        failed.insert(Question::Q6ResultScopedOpaqueIdentity);
    }
    // Q7: ADR 0010 requires every supported constructed node to own a
    // constructed identity and constructed placement; the result stays
    // immutable and query-oriented (both options are).
    if WHATWG_CONSTRUCTS_DOCUMENT_TYPE_CHILD
        && (answers.constructed_identity.is_none()
            || answers.document_child_sequence != normative_child_sequence())
    {
        failed.insert(Question::Q7ArchitectureCompatible);
    }
    failed
}

/// Domains an identity must not be defined by, as perturbations of one result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Perturbation {
    FinalPlacement,
    AuthoredRange,
    SourceIdentity,
    DocumentMode,
}

const ALL_PERTURBATIONS: [Perturbation; 4] = [
    Perturbation::FinalPlacement,
    Perturbation::AuthoredRange,
    Perturbation::SourceIdentity,
    Perturbation::DocumentMode,
];

fn doctype_identity(
    observation: &Observation,
    scheme: IdentityScheme,
    placement: Placement,
) -> ConstructedId {
    let minted = mint(observation, SCOPE_1, scheme, placement);
    minted.children[doctype_child_index(observation)]
}

/// The perturbations under which the DocumentType's identity stops denoting
/// the same constructed observation. Ids are compared only inside one scope.
fn identity_moves_under(scheme: IdentityScheme) -> BTreeSet<Perturbation> {
    let baseline = observe_plain(accepted("A01").bytes);
    let before = doctype_identity(&baseline, scheme, Placement::Final);
    let mutated =
        |mutation: Mutation| observe(accepted("A01").bytes, Budget::GENEROUS, Some(mutation));
    let mut moved = BTreeSet::new();
    for perturbation in ALL_PERTURBATIONS {
        let (observation, placement) = match perturbation {
            Perturbation::FinalPlacement => (baseline.clone(), Placement::Reversed),
            Perturbation::AuthoredRange => (mutated(Mutation::CompleteShifted), Placement::Final),
            Perturbation::SourceIdentity => {
                (mutated(Mutation::SourceIdCorrupted), Placement::Final)
            }
            Perturbation::DocumentMode => (mutated(Mutation::QuirksOnCanonical), Placement::Final),
        };
        let after = doctype_identity(&observation, scheme, placement);
        if before.same_observation(after) != Some(true) {
            moved.insert(perturbation);
        }
    }
    moved
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selection {
    SelectA,
    SelectB,
    BothRemainViable,
    Stop,
}

fn select(a_passes: bool, b_passes: bool) -> Selection {
    match (a_passes, b_passes) {
        (true, false) => Selection::SelectA,
        (false, true) => Selection::SelectB,
        (true, true) => Selection::BothRemainViable,
        (false, false) => Selection::Stop,
    }
}

// ---------------------------------------------------------------------------
// 8. Tests
// ---------------------------------------------------------------------------

fn fixture_source(fixture: &Accepted) -> SourceText {
    SourceText::new(SourceId::new(SOURCE_ID), text_of(fixture.bytes))
}

/// CD1 — the handwritten offsets are checked against the literal bytes, with
/// no model involved, before they are trusted as GOLD.
#[test]
fn cd1_fixture_bytes_match_their_handwritten_ranges() {
    for fixture in ACCEPTED {
        let length = fixture.bytes.len();
        assert!(fixture.complete.1 <= length, "{}", fixture.id);
        assert_eq!(fixture.complete.0, 0, "{}", fixture.id);
        for (range, expected) in expected_fragments(fixture) {
            assert_eq!(
                &fixture.bytes[range.0..range.1],
                expected,
                "{}: bytes at {range:?}",
                fixture.id
            );
        }
        let head = &fixture.bytes[..fixture.complete.1];
        assert_eq!(*head.last().unwrap(), b'>', "{}", fixture.id);
        assert!(
            head[..head.len() - 1].iter().all(|byte| *byte != b'>'),
            "{}: the complete range ends at the first `>`",
            fixture.id
        );
    }
}

/// CD1 — every accepted source equals the hand-authored GOLD in every
/// dimension.
#[test]
fn cd1_accepted_sources_match_the_independent_gold() {
    assert_eq!(ACCEPTED.len(), 23);
    for fixture in ACCEPTED {
        assert_eq!(
            observe_plain(fixture.bytes),
            gold(fixture),
            "{}",
            fixture.id
        );
    }
}

/// CD1 — one DOCTYPE token with a semantic identity distinct from every other
/// token kind, and nothing else claims it.
#[test]
fn cd1_doctype_is_one_distinct_semantic_token() {
    for fixture in ACCEPTED {
        let observed = observe_plain(fixture.bytes);
        assert_eq!(doctype_count(&observed.tokens), 1, "{}", fixture.id);
        assert!(
            matches!(observed.tokens[0], Token::Doctype(_)),
            "{}: the DOCTYPE is the first token",
            fixture.id
        );
        assert!(
            !observed.tokens[1..]
                .iter()
                .any(|token| matches!(token, Token::Doctype(_) | Token::Character { .. })),
            "{}",
            fixture.id
        );
        let Token::Doctype(doctype) = &observed.tokens[0] else {
            unreachable!()
        };
        assert_eq!(doctype.complete.range, fixture.complete, "{}", fixture.id);
        assert_eq!(doctype.name_source.range, fixture.name, "{}", fixture.id);
        assert_eq!(doctype.complete.source, SOURCE_ID, "{}", fixture.id);
        assert_eq!(doctype.name_source.source, SOURCE_ID, "{}", fixture.id);
        assert_eq!(doctype.interpreted_name, "html", "{}", fixture.id);
        assert_eq!(doctype.public, Identifier::Missing, "{}", fixture.id);
        assert_eq!(doctype.system, Identifier::Missing, "{}", fixture.id);
        assert_eq!(doctype.force_quirks, ForceQuirks::Off, "{}", fixture.id);
    }
}

/// CD2 — authored casing is exact evidence; normalization is meaning only.
#[test]
fn cd2_casing_is_preserved_in_evidence_and_normalized_in_meaning() {
    for (id, spelling) in [
        ("A03", "HTML"),
        ("A04", "hTmL"),
        ("A05", "HTML"),
        ("A01", "html"),
    ] {
        let fixture = accepted(id);
        let source = fixture_source(fixture);
        let observed = observe_plain(fixture.bytes);
        let Token::Doctype(doctype) = &observed.tokens[0] else {
            panic!("{id}: expected a DOCTYPE token")
        };
        assert_eq!(doctype.name_spelling, spelling, "{id}");
        assert_eq!(doctype.interpreted_name, "html", "{id}");
        let anchor = source
            .anchor(doctype.name_source.range.0, doctype.name_source.range.1)
            .unwrap();
        assert_eq!(
            anchor.fragment(),
            spelling,
            "{id}: evidence is the raw spelling"
        );
    }
    // The keyword casing never reaches meaning or the name anchor.
    assert_eq!(
        observe_plain(accepted("A02").bytes).tokens[0],
        observe_plain(accepted("A01").bytes).tokens[0]
    );
}

/// CD3 — every selected whitespace class is accepted in both positions, and
/// authored CRLF stays distinct from its single interpreted unit.
#[test]
fn cd3_every_selected_whitespace_class_is_accepted() {
    for id in [
        "W01", "W02", "W03", "W04", "W05", "T02", "T03", "T04", "T05", "T06",
    ] {
        let observed = observe_plain(accepted(id).bytes);
        assert_eq!(doctype_count(&observed.tokens), 1, "{id}");
    }
    let plain_space = observe_plain(accepted("A01").bytes);
    assert_eq!(doctype_count(&plain_space.tokens), 1);
    let crlf = accepted("W05");
    assert_eq!(crlf.leading_whitespace_authored, 2);
    assert_eq!(crlf.leading_whitespace_interpreted, 1);
    let authored = &text_of(crlf.bytes)[9..11];
    assert_eq!(authored.len(), 2);
    assert_eq!(interpret_newlines(authored).len(), 1);
    // The name anchor sits after the two authored bytes, not after one.
    assert_eq!(crlf.name, (11, 15));
    assert_eq!(crlf.complete, (0, 16));
    let observed = observe_plain(crlf.bytes);
    let Token::Doctype(doctype) = &observed.tokens[0] else {
        panic!("W05")
    };
    assert_eq!(doctype.complete.range.1 - doctype.complete.range.0, 16);
    assert_ne!(
        interpret_newlines(&text_of(crlf.bytes)).len(),
        crlf.bytes.len(),
        "the interpreted text is shorter than the authored source"
    );
    // Run lengths and mixed classes do not matter.
    for id in ["W06", "W07", "T07", "T08", "T09"] {
        assert_eq!(
            observe_plain(accepted(id).bytes),
            gold(accepted(id)),
            "{id}"
        );
    }
}

/// CD3 — the required whitespace and only HTML whitespace qualify.
#[test]
fn cd3_unicode_and_non_html_whitespace_never_qualify() {
    for id in ["X08", "U01", "U02", "U03", "U04", "U05", "U06"] {
        let (bytes, boundary) = negative(id);
        let observed = observe_plain(bytes);
        assert_eq!(observed.completion, Completion::Boundary(boundary), "{id}");
        assert_eq!(doctype_count(&observed.tokens), 0, "{id}");
    }
    // The host language's notion of whitespace is wider than the theorem's.
    for byte in [0x0b_u8, 0xa0, 0x85] {
        assert!(!is_html_whitespace(byte), "{byte:#x}");
    }
    assert!('\u{a0}'.is_whitespace() && '\u{2003}'.is_whitespace() && '\u{0b}'.is_whitespace());
    // `<!DOCTYPEhtml>` is a Standard DOCTYPE with a tokenizer parse error: the
    // selected canonical profile excludes it for that reason, and the model
    // says so instead of calling it "not a DOCTYPE".
    assert_eq!(negative("X08").1, Boundary::RequiresTokenizerParseError);
}

/// CD4 — Missing is not empty and owns no source range.
#[test]
fn cd4_missing_identifiers_are_not_empty_and_own_no_evidence() {
    let empty = Identifier::Authored {
        span: sp((14, 14)),
        value: String::new(),
    };
    assert_ne!(Identifier::Missing, empty);
    assert!(is_fabricated(&empty));
    assert!(!is_fabricated(&Identifier::Missing));
    for fixture in ACCEPTED {
        let observed = observe_plain(fixture.bytes);
        for token in &observed.tokens {
            if let Token::Doctype(doctype) = token {
                assert_eq!(doctype.public, Identifier::Missing, "{}", fixture.id);
                assert_eq!(doctype.system, Identifier::Missing, "{}", fixture.id);
            }
        }
        // No zero-length authored range exists anywhere in the observation
        // except the EndOfFile position, which is a point, not a range.
        for token in &observed.tokens {
            let spans: Vec<Span> = match token {
                Token::Doctype(d) => vec![d.complete, d.name_source],
                Token::StartTag { span, .. }
                | Token::EndTag { span, .. }
                | Token::Character { span, .. } => vec![*span],
                Token::EndOfFile { .. } => Vec::new(),
            };
            assert!(
                spans.iter().all(|span| span.range.0 < span.range.1),
                "{}: no fabricated zero-byte source range",
                fixture.id
            );
        }
    }
}

/// CD5 — the Initial-mode theorem, each fact individually inspectable.
#[test]
fn cd5_initial_consumes_the_canonical_doctype_exactly_once() {
    for fixture in ACCEPTED {
        let observed = observe_plain(fixture.bytes);
        let first = &observed.dispatches[0];
        assert_eq!(first.token_index, 0, "{}", fixture.id);
        // consumed exactly once, by Initial, with no reprocess
        assert_eq!(
            first.steps,
            vec![Step {
                mode: Mode::Initial,
                outcome: Outcome::Consumed {
                    next: Mode::BeforeHtml
                }
            }],
            "{}",
            fixture.id
        );
        assert!(
            observed
                .dispatches
                .iter()
                .filter(|dispatch| dispatch.token_index == 0)
                .count()
                == 1,
            "{}",
            fixture.id
        );
        // no MissingDoctype, and no diagnostic at all
        assert!(observed.diagnostics.is_empty(), "{}", fixture.id);
        // NoQuirks is preserved
        assert_eq!(observed.doc_mode, DocMode::NoQuirks, "{}", fixture.id);
        // the next token is dispatched in BeforeHtml, never Initial
        assert_eq!(
            observed.dispatches[1].steps[0].mode,
            Mode::BeforeHtml,
            "{}",
            fixture.id
        );
        // durable evidence exists: the DocumentType is the first Document child
        assert!(
            matches!(observed.children.first(), Some(Child::DocumentType(_))),
            "{}",
            fixture.id
        );
    }
}

/// CD5 — token field and tree result are distinct dimensions.
#[test]
fn cd5_force_quirks_and_document_mode_are_distinct_dimensions() {
    let expected = gold(accepted("A01"));
    let mutated = observe(
        accepted("A01").bytes,
        Budget::GENEROUS,
        Some(Mutation::ForceQuirksOn),
    );
    let changed = dims(&expected, &mutated);
    assert!(changed.contains(&Dim::ForceQuirks));
    assert!(changed.contains(&Dim::DocumentMode));
    let quirks_only = observe(
        accepted("A01").bytes,
        Budget::GENEROUS,
        Some(Mutation::QuirksOnCanonical),
    );
    assert_eq!(
        dims(&expected, &quirks_only),
        BTreeSet::from([Dim::DocumentMode])
    );
}

/// CD6 — the missing-DOCTYPE predecessor is separately valid and distinct.
#[test]
fn cd6_missing_doctype_contrast_is_observably_distinct() {
    let missing = observe_plain(b"<body>");
    assert_eq!(missing, gold_missing_doctype_body());
    assert_eq!(missing.diagnostics, vec![missing_doctype(0)]);
    assert_eq!(missing.doc_mode, DocMode::Quirks);
    assert_eq!(
        missing.dispatches[0].steps[0],
        reprocess(Mode::Initial, Mode::BeforeHtml)
    );
    let canonical = observe_plain(accepted("A01").bytes);
    assert_eq!(
        canonical.dispatches[0].steps[0],
        consumed(Mode::Initial, Mode::BeforeHtml)
    );
    assert!(canonical.diagnostics.is_empty());
    assert_eq!(canonical.doc_mode, DocMode::NoQuirks);
    // They differ in diagnostic, document mode, disposition and reprocessing.
    let apart = dims(&canonical, &missing);
    for dim in [
        Dim::Diagnostics,
        Dim::DocumentMode,
        Dim::Dispatch,
        Dim::Constructed,
    ] {
        assert!(apart.contains(&dim), "{dim:?}");
    }
    // Neither path is a special case of the other.
    assert_ne!(canonical.children, missing.children);
    assert_eq!(missing.children.len(), 1);
    assert_eq!(canonical.children.len(), 2);
}

/// CD7 — A/B comparison. Each representation is judged on its own, against the
/// selected theorem, inside its own result scope. No result is compared with
/// another.
#[test]
fn cd7_representation_alternatives_are_compared_and_one_is_selected() {
    let mut a_failures = BTreeSet::new();
    let mut b_failures = BTreeSet::new();
    for fixture in ACCEPTED {
        let observed = observe_plain(fixture.bytes);
        for scope in [SCOPE_1, SCOPE_2] {
            let a = answers_a(&repr_a(&observed, scope));
            let b = answers_b(&repr_b(&observed, scope));
            a_failures.extend(verdict(&a, fixture));
            b_failures.extend(verdict(&b, fixture));
        }
    }
    assert!(a_failures.is_empty(), "{a_failures:?}");
    // B fails because the normative constructed DocumentType identity and
    // Document placement are absent, not because of any cross-result rule.
    assert_eq!(
        b_failures,
        BTreeSet::from([
            Question::Q2ChildOrdering,
            Question::Q3ThreeWayDistinction,
            Question::Q5NoSilentOmission,
            Question::Q6ResultScopedOpaqueIdentity,
            Question::Q7ArchitectureCompatible,
        ])
    );
    // Q1 and Q4 do not separate A from B: both are source-backed and honest.
    assert!(!b_failures.contains(&Question::Q1AuthoredIdentity));
    assert!(!b_failures.contains(&Question::Q4NoFabrication));
    assert_eq!(ALL_QUESTIONS.len(), 7);
    assert_eq!(
        select(a_failures.is_empty(), b_failures.is_empty()),
        Selection::SelectA
    );
}

/// CD7 — B can answer only by being given a constructed identity and a Document
/// placement, at which point it is Option A under another name. The identity it
/// receives is minted, not computed from the placement it is given.
#[test]
fn cd7_option_b_rescued_by_identity_and_placement_is_option_a() {
    for fixture in ACCEPTED {
        let observed = observe_plain(fixture.bytes);
        let minted = mint(&observed, SCOPE_1, IdentityScheme::Opaque, Placement::Final);
        let a = project_a(&observed, &minted, Placement::Final);
        let extended = extend_b(&observed, &minted, Placement::Final);
        assert_eq!(
            a,
            b_plus_as_a(&extended),
            "{}: B plus identity and placement == A",
            fixture.id
        );
        assert!(
            a.nodes
                .iter()
                .any(|node| node.kind == NodeKind::DocumentType),
            "{}",
            fixture.id
        );
        assert!(
            verdict(&answers_a(&b_plus_as_a(&extended)), fixture).is_empty(),
            "{}: the rescued B is judged exactly as A is",
            fixture.id
        );
        // The rescued identity does not follow the placement it sits beside.
        let moved = extend_b(&observed, &minted, Placement::Reversed);
        assert_eq!(
            extended
                .relation_identity
                .same_observation(moved.relation_identity),
            Some(true),
            "{}",
            fixture.id
        );
        assert_ne!(extended.relation_placement, moved.relation_placement);
    }
}

/// CD7 — inside one result, authored evidence, constructed identity and
/// document mode are separate facts of separate types, and none of them is
/// another. Nothing here compares one result's identity with another's.
#[test]
fn cd7_authored_mode_and_constructed_identity_are_three_facts() {
    use std::any::TypeId;
    for fixture in ACCEPTED {
        let answers = answers_a(&repr_a(&observe_plain(fixture.bytes), SCOPE_1));
        assert!(answers.authored_complete.is_some(), "{}", fixture.id);
        assert!(answers.constructed_identity.is_some(), "{}", fixture.id);
        assert_eq!(answers.document_mode, DocMode::NoQuirks, "{}", fixture.id);
        let identity = answers.constructed_identity.unwrap();
        assert_eq!(identity.scope(), answers.scope, "{}", fixture.id);
    }
    // The distinction is carried by type, not by comparing integers.
    let domains = [
        TypeId::of::<ConstructedId>(),
        TypeId::of::<Span>(),
        TypeId::of::<SourceId>(),
        TypeId::of::<DocMode>(),
        TypeId::of::<ResultScope>(),
        TypeId::of::<usize>(),
        TypeId::of::<u64>(),
    ];
    let distinct: BTreeSet<String> = domains.iter().map(|id| format!("{id:?}")).collect();
    assert_eq!(distinct.len(), domains.len());
    // A mode exists without any DocumentType identity: `<body>` is Quirks and
    // owns no DocumentType.
    let missing = answers_a(&repr_a(&observe_plain(b"<body>"), SCOPE_1));
    assert_eq!(missing.document_mode, DocMode::Quirks);
    assert_eq!(missing.constructed_identity, None);
    assert_eq!(missing.authored_complete, None);
    // Authored evidence varies without a second identity domain appearing: the
    // evidence is a `Span`, the identity is not.
    let crlf = answers_a(&repr_a(&observe_plain(accepted("W05").bytes), SCOPE_1));
    assert_eq!(crlf.authored_complete, Some(sp((0, 16))));
    assert!(crlf.constructed_identity.is_some());
}

/// CD7 (I1–I4) — an identity derived from source range, source identity, final
/// placement or document mode moves when that domain moves; the validated
/// opaque identity moves under none of them. This is a structural separation of
/// domains, not a claim about recovery stability: the selected slice has no
/// placement-changing recovery, and the opaque model does not preclude that
/// architecture invariant because it never reads placement.
#[test]
fn cd7_identity_is_opaque_and_independent_of_other_domains() {
    assert_eq!(
        identity_moves_under(IdentityScheme::Opaque),
        BTreeSet::new()
    );
    for (scheme, expected) in [
        (IdentityScheme::FromPlacement, Perturbation::FinalPlacement),
        (IdentityScheme::FromRange, Perturbation::AuthoredRange),
        (IdentityScheme::FromSourceId, Perturbation::SourceIdentity),
        (IdentityScheme::FromDocumentMode, Perturbation::DocumentMode),
    ] {
        assert_eq!(
            identity_moves_under(scheme),
            BTreeSet::from([expected]),
            "{scheme:?}"
        );
    }
}

/// CD7 (I2) — placement is read by the representation separately from identity:
/// re-ordering the Document children changes the placement answer and leaves the
/// identity answer alone.
#[test]
fn cd7_placement_and_identity_are_separate_facts() {
    let observed = observe_plain(accepted("C01").bytes);
    let minted = mint(&observed, SCOPE_1, IdentityScheme::Opaque, Placement::Final);
    let final_order = answers_a(&project_a(&observed, &minted, Placement::Final));
    let reversed = answers_a(&project_a(&observed, &minted, Placement::Reversed));
    assert_eq!(
        final_order.document_child_sequence,
        vec![ChildKind::DocumentType, ChildKind::Element(TagName::Html)]
    );
    assert_eq!(
        reversed.document_child_sequence,
        vec![ChildKind::Element(TagName::Html), ChildKind::DocumentType]
    );
    assert_eq!(
        final_order
            .constructed_identity
            .unwrap()
            .same_observation(reversed.constructed_identity.unwrap()),
        Some(true)
    );
}

/// CD7 (I3, I5) — independent result scopes do not participate in an equality
/// theorem: each is judged alone, the model never asks for equal or unequal
/// values across them, and such ids are explicitly not comparable.
#[test]
fn cd7_cross_result_identity_is_not_comparable_and_not_required() {
    let fixture = accepted("A01");
    let observed = observe_plain(fixture.bytes);
    let first = answers_a(&repr_a(&observed, SCOPE_1));
    let second = answers_a(&repr_a(&observed, SCOPE_2));
    assert!(verdict(&first, fixture).is_empty());
    assert!(verdict(&second, fixture).is_empty());
    let (one, two) = (
        first.constructed_identity.unwrap(),
        second.constructed_identity.unwrap(),
    );
    assert_eq!(one.same_observation(two), None);
    assert_eq!(two.same_observation(one), None);
    // Within one scope a result is deterministic: rebuilding it is equal.
    assert_eq!(repr_a(&observed, SCOPE_1), repr_a(&observed, SCOPE_1));
    // A different parse result in the same scope is a different result; no
    // meaning is attached to its ids relative to this one's.
    let other = observe_plain(accepted("W05").bytes);
    assert!(verdict(&answers_a(&repr_a(&other, SCOPE_1)), accepted("W05")).is_empty());
}

/// CD7 (I3) — the identity module exposes no conversion to or from another
/// domain. Needles are assembled at run time so this test never matches itself.
#[test]
fn cd7_identity_module_has_no_conversion_to_other_domains() {
    let own = include_str!("canonical_doctype_successor_validation.rs");
    let begin = own.find("mod identity {").expect("identity module");
    let end = own[begin..].find("\nuse identity::").expect("module end") + begin;
    let module = &own[begin..end];
    for needle in [
        ["impl From", "<"].concat(),
        ["impl Into", "<"].concat(),
        ["fn val", "ue("].concat(),
        ["fn as_", "u"].concat(),
        ["Source", "Id"].concat(),
        ["Span", ""].concat(),
        ["Browser", ""].concat(),
        ["Node", "Id"].concat(),
    ] {
        assert!(
            !module.contains(&needle),
            "identity module must not mention {needle}"
        );
    }
}

/// CD7 — structural non-equivalences.
#[test]
fn cd7_doctype_is_not_element_text_diagnostic_or_document_mode() {
    let a = repr_a(&observe_plain(accepted("C01").bytes), SCOPE_1);
    let doctype = a
        .nodes
        .iter()
        .find(|node| node.kind == NodeKind::DocumentType)
        .unwrap();
    assert_ne!(doctype.kind, NodeKind::Element);
    assert_ne!(doctype.kind, NodeKind::Text);
    assert_ne!(doctype.kind, NodeKind::Document);
    assert!(doctype.element.is_none());
    assert!(doctype.children.is_empty());
    for kind in ALL_NODE_KINDS {
        assert_eq!(kind == NodeKind::DocumentType, kind == doctype.kind);
    }
    // not a diagnostic and not document mode
    let observed = observe_plain(accepted("C01").bytes);
    assert!(observed.diagnostics.is_empty());
    assert_eq!(observed.doc_mode, DocMode::NoQuirks);
    assert_eq!(a.document_mode, DocMode::NoQuirks);
    // the authored identity is evidence, not a mode: spelling differs, mode
    // does not
    let spellings: BTreeSet<String> = ["A01", "A03", "A04"]
        .iter()
        .map(|id| match &observe_plain(accepted(id).bytes).tokens[0] {
            Token::Doctype(d) => d.name_spelling.clone(),
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(spellings.len(), 3);
}

/// CD7 — the selection rule is not constant.
#[test]
fn cd7_selection_rule_covers_every_outcome() {
    assert_eq!(select(true, false), Selection::SelectA);
    assert_eq!(select(false, true), Selection::SelectB);
    assert_eq!(select(true, true), Selection::BothRemainViable);
    assert_eq!(select(false, false), Selection::Stop);
}

/// CD7 — document mode is not retained by either option's *selected token*
/// facts alone: the DOM view maps Missing to the empty string, the project
/// evidence does not.
#[test]
fn cd7_dom_empty_string_view_does_not_license_authored_empty_evidence() {
    let observed = observe_plain(accepted("A01").bytes);
    let Token::Doctype(doctype) = &observed.tokens[0] else {
        panic!("A01")
    };
    // A DOM-like view may read "" for a missing identifier; that is a derived
    // view and never authored evidence.
    let dom_public = match &doctype.public {
        Identifier::Missing => "",
        Identifier::Authored { value, .. } => value.as_str(),
    };
    assert_eq!(dom_public, "");
    assert_eq!(doctype.public, Identifier::Missing);
    assert_ne!(
        doctype.public,
        Identifier::Authored {
            span: sp((14, 14)),
            value: String::new()
        }
    );
}

/// CD8 — outside-profile sources never become the canonical theorem.
#[test]
fn cd8_outside_profile_sources_publish_no_canonical_result() {
    for (id, bytes, boundary) in NEGATIVES {
        let observed = observe_plain(bytes);
        assert_eq!(observed, gold_boundary(*boundary), "{id}");
        assert_eq!(doctype_count(&observed.tokens), 0, "{id}");
        assert!(observed.children.is_empty(), "{id}");
        assert!(observed.dispatches.is_empty(), "{id}");
        assert_eq!(observed.mode, Mode::Initial, "{id}");
        assert!(
            !matches!(observed.completion, Completion::CheckpointReached { .. }),
            "{id}: lexical boundary is not a completed theorem"
        );
    }
}

/// CD8 — classification is relative to the selected theorem and distinguishes
/// the families the Issue names.
#[test]
fn cd8_classification_distinguishes_the_boundary_families() {
    let classes: BTreeSet<&str> = NEGATIVES
        .iter()
        .map(|(_, _, boundary)| match boundary {
            Boundary::BroaderDoctype(_) => "broader",
            Boundary::MalformedSelectedForm(_) => "malformed",
            Boundary::RequiresTokenizerParseError => "parse-error",
            Boundary::OtherMarkup(Markup::Comment) => "comment",
            Boundary::OtherMarkup(Markup::Cdata) => "cdata",
            Boundary::OtherMarkup(Markup::ProcessingInstruction) => "pi",
            Boundary::OtherMarkup(Markup::General) => "general",
            Boundary::OutsideModelledToken => "outside",
        })
        .collect();
    assert_eq!(
        classes,
        BTreeSet::from([
            "broader",
            "malformed",
            "parse-error",
            "comment",
            "cdata",
            "pi",
            "general"
        ])
    );
}

/// CD8 — comment/CDATA/PI/general markup stay outside, even after a DOCTYPE.
#[test]
fn cd8_other_markup_after_a_canonical_doctype_stays_a_boundary() {
    for (suffix, markup) in [
        (&b"<!-- c -->"[..], Markup::Comment),
        (&b"<![CDATA[x]]>"[..], Markup::Cdata),
        (&b"<!xx>"[..], Markup::General),
        (&b"<?x?>"[..], Markup::ProcessingInstruction),
    ] {
        let mut bytes = b"<!DOCTYPE html>".to_vec();
        bytes.extend_from_slice(suffix);
        let observed = observe_plain(&bytes);
        assert_eq!(
            observed,
            gold_doctype_then_boundary(Boundary::OtherMarkup(markup)),
            "{markup:?}"
        );
    }
}

/// CD9 — a DOCTYPE outside Initial is not selected, and no recovery is invented.
#[test]
fn cd9_doctype_outside_initial_is_outside_the_selected_tree_profile() {
    assert_eq!(
        observe_plain(b"<html><!DOCTYPE html>"),
        gold_html_then_doctype()
    );
    assert_eq!(
        observe_plain(b"<!DOCTYPE html><!DOCTYPE html>"),
        gold_second_doctype()
    );
    for bytes in [
        &b"<html><!DOCTYPE html>"[..],
        &b"<!DOCTYPE html><!DOCTYPE html>"[..],
    ] {
        let observed = observe_plain(bytes);
        assert_eq!(observed.completion, Completion::TreeRefused);
        let doctypes = observed
            .children
            .iter()
            .filter(|child| matches!(child, Child::DocumentType(_)))
            .count();
        assert!(
            doctypes <= 1,
            "a later DOCTYPE never becomes a second DocumentType"
        );
    }
}

/// CD9 — every non-Initial mode of the model refuses a DOCTYPE, so Initial
/// semantics are never silently applied later.
#[test]
fn cd9_every_non_initial_mode_refuses_a_doctype_token() {
    let token = Token::Doctype(gold_doctype((0, 15), (10, 14), "html"));
    for mode in ALL_MODES {
        let mut machine = Machine::new(&Budget::GENEROUS, None);
        machine.mode = mode;
        let outcome = machine.step(mode, 0, &token);
        if mode == Mode::Initial {
            assert_eq!(
                outcome,
                Outcome::Consumed {
                    next: Mode::BeforeHtml
                }
            );
        } else {
            assert_eq!(outcome, Outcome::OutsideSelectedTreeProfile, "{mode:?}");
            assert!(machine.children.is_empty(), "{mode:?}");
            assert!(machine.diagnostics.is_empty(), "{mode:?}");
            assert_eq!(machine.doc_mode, DocMode::NoQuirks, "{mode:?}");
        }
    }
}

/// CD10 — preflight -> prepare -> commit: a refusal leaves no half token, no
/// partial evidence, no consumed transition, and no complete result.
#[test]
fn cd10_resource_refusal_around_the_doctype_is_atomic() {
    let bytes = accepted("A01").bytes;
    let committed = gold(accepted("A01"));
    // 0 token slots: nothing committed.
    let none = observe(
        bytes,
        Budget {
            tokens: 0,
            ..Budget::GENEROUS
        },
        None,
    );
    let nothing = Observation {
        completion: Completion::ResourceLimited(Resource::Tokens),
        ..gold_boundary(Boundary::OutsideModelledToken)
    };
    assert_eq!(none, nothing);
    // one anchor of two: the DOCTYPE is refused whole, with no leaked anchor.
    let one = observe(
        bytes,
        Budget {
            anchors: 1,
            ..Budget::GENEROUS
        },
        None,
    );
    assert_eq!(
        one,
        Observation {
            completion: Completion::ResourceLimited(Resource::Anchors),
            ..gold_boundary(Boundary::OutsideModelledToken)
        }
    );
    assert_eq!(one.retained_anchors, 0);
    assert_eq!(one.mode, Mode::Initial);
    assert!(one.children.is_empty() && one.tokens.is_empty() && one.dispatches.is_empty());
    // exactly two anchors: the DOCTYPE commits whole, EOF is refused.
    let two = observe(
        bytes,
        Budget {
            anchors: 2,
            ..Budget::GENEROUS
        },
        None,
    );
    assert_eq!(doctype_count(&two.tokens), 1);
    assert_eq!(two.retained_anchors, 2);
    assert_eq!(
        two.completion,
        Completion::ResourceLimited(Resource::Anchors)
    );
    assert_eq!(
        two.dispatches,
        vec![dispatch(0, vec![consumed(Mode::Initial, Mode::BeforeHtml)])]
    );
    assert!(matches!(two.children[..], [Child::DocumentType(_)]));
    // one token slot: DOCTYPE commits, EOF is refused.
    let slot = observe(
        bytes,
        Budget {
            tokens: 1,
            ..Budget::GENEROUS
        },
        None,
    );
    assert_eq!(
        slot.completion,
        Completion::ResourceLimited(Resource::Tokens)
    );
    assert_eq!(slot.tokens.len(), 1);
    // none of the refusals is a completed canonical path.
    for observed in [&none, &one, &two, &slot] {
        assert!(!matches!(
            observed.completion,
            Completion::CheckpointReached { .. }
        ));
        assert_ne!(*observed, committed);
    }
    // the canonical path spends no diagnostic budget...
    let no_diag = observe(
        bytes,
        Budget {
            diagnostics: 0,
            ..Budget::GENEROUS
        },
        None,
    );
    assert_eq!(no_diag, committed);
    // ...while the missing-DOCTYPE path does, and is refused before mutation.
    let refused = observe(
        b"<body>",
        Budget {
            diagnostics: 0,
            ..Budget::GENEROUS
        },
        None,
    );
    assert_eq!(
        refused.completion,
        Completion::ResourceLimited(Resource::Diagnostics)
    );
    assert_eq!(refused.mode, Mode::Initial);
    assert_eq!(refused.doc_mode, DocMode::NoQuirks);
    assert!(refused.diagnostics.is_empty() && refused.children.is_empty());
    assert_eq!(
        refused.dispatches,
        vec![dispatch(
            0,
            vec![Step {
                mode: Mode::Initial,
                outcome: Outcome::ResourceRefused(Resource::Diagnostics)
            }]
        )]
    );
}

/// CD10 — lower-layer incomplete, unsupported and resource-limited never become
/// complete.
#[test]
fn cd10_lower_layer_incompleteness_is_monotonic() {
    let scenarios: Vec<(&str, Observation)> = vec![
        ("truncated name", observe_plain(b"<!DOCTYPE html")),
        ("truncated keyword", observe_plain(b"<!DOCTYPE")),
        ("broader", observe_plain(b"<!DOCTYPE html PUBLIC \"x\">")),
        (
            "doctype then comment",
            observe_plain(b"<!DOCTYPE html><!-- c -->"),
        ),
        (
            "anchors",
            observe(
                accepted("A01").bytes,
                Budget {
                    anchors: 2,
                    ..Budget::GENEROUS
                },
                None,
            ),
        ),
        (
            "tokens",
            observe(
                accepted("C01").bytes,
                Budget {
                    tokens: 2,
                    ..Budget::GENEROUS
                },
                None,
            ),
        ),
    ];
    for (label, observed) in scenarios {
        assert!(
            !matches!(observed.completion, Completion::CheckpointReached { .. }),
            "{label}"
        );
    }
    // A committed DOCTYPE prefix is valid evidence but not a complete run.
    let prefix = observe_plain(b"<!DOCTYPE html><!-- c -->");
    assert_eq!(prefix.children.len(), 1);
    assert!(matches!(prefix.completion, Completion::Boundary(_)));
}

/// CDF — each mutation is rejected by the independent GOLD, and changes
/// exactly the dimensions its fact causes.
#[test]
fn cdf_each_load_bearing_dimension_fails_independently() {
    use Dim as D;
    let case = |mutation: Mutation, id: &str, expected: &[Dim]| {
        let fixture = accepted(id);
        let want = gold(fixture);
        let got = observe(fixture.bytes, Budget::GENEROUS, Some(mutation));
        assert_ne!(got, want, "{mutation:?} on {id} must be rejected");
        let changed = dims(&want, &got);
        assert_eq!(
            changed,
            expected.iter().copied().collect::<BTreeSet<_>>(),
            "{mutation:?} on {id}"
        );
    };
    // One semantic fact at a time.
    case(Mutation::NameAnchorOnKeyword, "A01", &[D::NameRange]);
    case(Mutation::NameAnchorOnWhitespace, "A01", &[D::NameRange]);
    case(Mutation::CompleteTruncated, "A01", &[D::CompleteRange]);
    case(Mutation::CompleteShifted, "A01", &[D::CompleteRange]);
    case(Mutation::SourceIdCorrupted, "A01", &[D::SourceIdentity]);
    case(Mutation::NameSpellingLowercased, "A03", &[D::NameSpelling]);
    case(Mutation::NameStartFromFixedOffset, "W06", &[D::NameRange]);
    case(
        Mutation::NameFromCaseSensitiveSearch,
        "A03",
        &[D::NameRange],
    );
    case(
        Mutation::CompleteEndFromInterpretedLength,
        "W05",
        &[D::CompleteRange],
    );
    case(Mutation::RetokenizedSubstring, "A01", &[D::SourceIdentity]);
    // F11: interpreted identity. Causal consequence: not-html quirks mode and
    // the tree parse error.
    case(
        Mutation::InterpretedNameSvg,
        "A01",
        &[D::InterpretedName, D::DocumentMode, D::Diagnostics],
    );
    // F12: token field and its causal tree consequence.
    case(
        Mutation::ForceQuirksOn,
        "A01",
        &[D::ForceQuirks, D::DocumentMode],
    );
    // F7/F8: fabricated empty identifier evidence. Causal consequence: a
    // present identifier is a tree parse error.
    case(
        Mutation::PublicFabricatedEmpty,
        "A01",
        &[D::PublicIdentifier, D::Diagnostics],
    );
    case(
        Mutation::SystemFabricatedEmpty,
        "A01",
        &[D::SystemIdentifier, D::Diagnostics],
    );
    // F3, F4, F6, F18 and atomicity: one fact each.
    case(Mutation::MissingDoctypePollution, "A01", &[D::Diagnostics]);
    case(Mutation::QuirksOnCanonical, "A01", &[D::DocumentMode]);
    case(Mutation::DropDurableEvidence, "A01", &[D::Constructed]);
    // F5: the reprocess disposition. Its causal consequence is the refusal of
    // the re-dispatched DOCTYPE in BeforeHtml: the run stops there, so the
    // mode and the implied html child never follow.
    case(
        Mutation::ReprocessCanonical,
        "A01",
        &[
            D::Dispatch,
            D::ModeTransition,
            D::Constructed,
            D::Completion,
        ],
    );
}

/// CDF — wrong token kind and masquerades, each caught as a token-shape change.
#[test]
fn cdf_wrong_token_kind_is_rejected() {
    for mutation in [Mutation::DoctypeAsStartTag, Mutation::DoctypeAsText] {
        let fixture = accepted("A01");
        let want = gold(fixture);
        let got = observe(fixture.bytes, Budget::GENEROUS, Some(mutation));
        assert_ne!(got, want, "{mutation:?}");
        let changed = dims(&want, &got);
        assert!(
            changed.contains(&Dim::Acceptance),
            "{mutation:?}: {changed:?}"
        );
        assert_eq!(doctype_count(&got.tokens), 0, "{mutation:?}");
        // Its causal consequence is MissingDoctype and quirks: the masquerade
        // would silently change the tree theorem.
        assert!(changed.contains(&Dim::Diagnostics), "{mutation:?}");
        assert!(changed.contains(&Dim::DocumentMode), "{mutation:?}");
    }
}

/// CDF — widening the lexical profile is rejected by the negative GOLD.
#[test]
fn cdf_widened_recognition_is_rejected() {
    let widened = |mutation: Mutation, id: &str| {
        let (bytes, boundary) = negative(id);
        let got = observe(bytes, Budget::GENEROUS, Some(mutation));
        assert_ne!(got, gold_boundary(boundary), "{mutation:?} on {id}");
        assert!(
            dims(&gold_boundary(boundary), &got).contains(&Dim::Acceptance),
            "{mutation:?} on {id}: a boundary became a canonical DOCTYPE"
        );
    };
    // F13: PUBLIC/SYSTEM accepted as canonical.
    for id in ["X03", "X04", "X05", "X06"] {
        widened(Mutation::AcceptPublicSystem, id);
    }
    // F14: missing required whitespace.
    widened(Mutation::AcceptMissingWhitespace, "X08");
    // Unicode whitespace accepted because the host calls it whitespace.
    for id in ["U02", "U03", "U04"] {
        widened(Mutation::AcceptUnicodeWhitespace, id);
    }
    // F15/F16: comment, CDATA, PI and general markup aliased as DOCTYPE.
    widened(Mutation::AliasMarkup(Markup::Comment), "M01");
    widened(Mutation::AliasMarkup(Markup::Cdata), "M02");
    widened(Mutation::AliasMarkup(Markup::General), "M03");
    widened(Mutation::AliasMarkup(Markup::ProcessingInstruction), "M04");
}

/// CDF — F17: Initial semantics applied outside Initial.
#[test]
fn cdf_non_initial_acceptance_is_rejected() {
    for (bytes, want) in [
        (&b"<html><!DOCTYPE html>"[..], gold_html_then_doctype()),
        (
            &b"<!DOCTYPE html><!DOCTYPE html>"[..],
            gold_second_doctype(),
        ),
    ] {
        let got = observe(bytes, Budget::GENEROUS, Some(Mutation::AcceptNonInitial));
        assert_ne!(got, want);
        assert!(dims(&want, &got).contains(&Dim::Completion));
    }
}

/// CDF — F18 and atomicity countermodels.
#[test]
fn cdf_incomplete_upgrade_and_partial_commit_are_rejected() {
    let truncated = b"<!DOCTYPE html".as_slice();
    let want = observe_plain(truncated);
    let got = observe(
        truncated,
        Budget::GENEROUS,
        Some(Mutation::UpgradeIncomplete),
    );
    assert_eq!(dims(&want, &got), BTreeSet::from([Dim::Completion]));
    assert!(matches!(
        got.completion,
        Completion::CheckpointReached { .. }
    ));
    // Also over a resource-limited lower layer.
    let limited = Budget {
        anchors: 2,
        ..Budget::GENEROUS
    };
    let want = observe(accepted("A01").bytes, limited, None);
    let got = observe(
        accepted("A01").bytes,
        limited,
        Some(Mutation::UpgradeIncomplete),
    );
    assert_eq!(dims(&want, &got), BTreeSet::from([Dim::Completion]));
    // A half-committed DOCTYPE leaks retained evidence.
    let one = Budget {
        anchors: 1,
        ..Budget::GENEROUS
    };
    let want = observe(accepted("A01").bytes, one, None);
    let got = observe(
        accepted("A01").bytes,
        one,
        Some(Mutation::PartialAnchorCommit),
    );
    assert_eq!(dims(&want, &got), BTreeSet::from([Dim::RetainedEvidence]));
    // The tree mode advancing on a refusal is a single-fact failure too.
    let refused = Budget {
        diagnostics: 0,
        ..Budget::GENEROUS
    };
    let want = observe(b"<body>", refused, None);
    let got = observe(b"<body>", refused, Some(Mutation::AdvanceModeOnRefusal));
    assert_eq!(dims(&want, &got), BTreeSet::from([Dim::ModeTransition]));
}

/// CDF — the harness itself is not trivially rejecting: no mutation means no
/// difference.
#[test]
fn cdf_unmutated_model_has_no_differences() {
    for fixture in ACCEPTED {
        assert!(dims(&gold(fixture), &observe_plain(fixture.bytes)).is_empty());
    }
}

/// CD11 — expected meaning is hand-authored: the GOLD region of this file
/// never calls the model. Needles are assembled at run time so this test never
/// matches itself.
#[test]
fn cd11_gold_region_never_calls_the_model() {
    let own = include_str!("canonical_doctype_successor_validation.rs");
    let begin = own.find("// BEGIN GOLD").expect("gold begin marker");
    let end = own.find("// END GOLD").expect("gold end marker");
    let region = &own[begin..end];
    for needle in [
        ["observe", "("].concat(),
        ["lex", "("].concat(),
        ["recognize", "_doctype"].concat(),
        ["Machine", "::"].concat(),
        ["canonical", "_token"].concat(),
        ["project", "_a"].concat(),
        ["tokenize", "("].concat(),
    ] {
        assert!(
            !region.contains(&needle),
            "GOLD must not call the candidate model: found {needle}"
        );
    }
}

/// CD11 — no production token, tree-construction or diagnostic type is the
/// oracle. Only generic source primitives and the one tokenizer observation.
#[test]
fn cd11_imports_no_production_token_or_tree_code() {
    let own = include_str!("canonical_doctype_successor_validation.rs");
    let forbidden = [
        ["super", "driver"],
        ["super", "session"],
        ["super", "result"],
        ["super::super::token", ""],
        ["super::super::tokenizer", "diagnostic"],
        ["crate::html::token", ""],
        ["crate::html::tree_construction", "driver"],
        ["crate::html::tree_construction", "session"],
        ["crate::html::tree_construction", "result"],
        ["crate::html::tree", ""],
    ]
    .map(|[head, tail]| format!("{head}::{tail}"));
    for line in own
        .lines()
        .filter(|line| line.trim_start().starts_with("use "))
    {
        for needle in &forbidden {
            assert!(
                !line.contains(needle),
                "{needle} would let production derive its oracle"
            );
        }
    }
    for needle in [
        ["Html", "Token"].concat(),
        ["Html", "TreeNode"].concat(),
        ["Html", "TreeDiagnosticCode"].concat(),
        ["Insertion", "Mode"].concat(),
        ["construct_html_document", "_shell"].concat(),
    ] {
        assert!(
            !own.lines().any(|line| {
                let line = line.trim_start();
                // A whole identifier only: `HtmlTokenizerLimits` is a generic
                // lower-layer primitive and is not the production token enum.
                !line.starts_with("//")
                    && line.match_indices(&needle).any(|(at, _)| {
                        !line[at + needle.len()..]
                            .chars()
                            .next()
                            .is_some_and(|next| next.is_alphanumeric() || next == '_')
                    })
            }),
            "{needle} must not appear in executable test code"
        );
    }
}

/// CD12 — observation only: production still stops canonical DOCTYPE at the
/// existing tokenizer boundary. This defines no expectation for the theorem.
#[test]
fn current_production_stops_at_markup_declaration() {
    let limits = HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024);
    for text in [
        "<!DOCTYPE html>",
        "<!doctype html>",
        "<body></body></html><!DOCTYPE html>",
    ] {
        let source = SourceText::new(SourceId::new(1), text.to_owned());
        let run = tokenize(&source, limits);
        assert!(
            matches!(
                run.completion(),
                HtmlTokenizerCompletion::Incomplete(
                    HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported)
                ) if unsupported.capability() == HtmlTokenizerCapability::MarkupDeclaration
            ),
            "{text}: production boundary changed"
        );
    }
}
