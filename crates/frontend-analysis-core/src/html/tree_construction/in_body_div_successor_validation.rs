//! Candidate-independent TC-S3 successor validation.
//!
//! TC-S3 is the *proposed* successor theorem "Selected In-Body No-Attribute
//! `div` Construction" (Issue #357). This module is **validation only**. It
//! changes no production tree-construction behavior, and production is
//! expected to remain TC-S1 + TC-S2 — which still refuse every non-shell tag
//! at [`super::session::admit`]. Nothing here authorizes production placement,
//! a production ordinary-element representation, or a general element-name or
//! namespace architecture.
//!
//! # Independent oracle boundary
//!
//! The expected meaning in this module is authored from the candidate theorem,
//! not from any production run. The boundary is structural and greppable: this
//! module imports **nothing** from [`super::driver`], [`super::session`], or
//! [`super::result`]. It therefore cannot call production `admit`, cannot call
//! `construct_html_document_shell`, and cannot project a production result
//! into an expectation.
//!
//! The only production code it uses is the already-accepted lower layer — the
//! existing batch tokenizer — and only as *evidence input*: emitted token
//! kinds, interpreted and raw tag-name evidence, attribute and self-closing
//! evidence, retained source anchors, and run completion. The tokenizer is
//! never the tree-semantic oracle.
//!
//! Two independent statements meet here and must agree:
//!
//! 1. [`CandidateSession`] — a test-only machine that implements the candidate
//!    action set over lower-layer token evidence, and
//! 2. [`candidate_gold`] — hand-authored expected observations for DV1–DV14.
//!
//! If the theorem were internally incoherent, the machine could not reproduce
//! the authored observations, and the comparison fails rather than adapting.
//!
//! # Pinned normative authority
//!
//! The candidate semantics below are read from the #348 pinned WHATWG HTML
//! source, commit `508a037333d8a1806504303aeb489d931fabbef6`, source blob
//! `68dbcb98bbe1001c6ae2531be2368c608fbafddd`, as recorded in
//! `docs/provenance/html.md`. The five load-bearing clauses are the `in body`
//! start-tag rule for the `address`…`ul` group (which contains `div`), the
//! matching end-tag rule for that group, "have a particular element in
//! scope", "generate implied end tags", and the `in body` end-of-file rule.
//! Those algorithms are implemented here over the candidate's own private
//! stack — [`CandidateSession::has_element_in_scope`],
//! [`CandidateSession::generate_implied_end_tags`], and
//! [`CandidateSession::has_p_in_button_scope`] — so the theorem's claimed
//! branch outcomes are *proved* against the candidate state rather than
//! assumed.
//!
//! # Deliberately partial model
//!
//! Per Issue #357 the model covers only the cells DV1–DV14 traverse plus the
//! selected `in body` `div` rules under validation. Every other cell is
//! refused with typed [`CandidateUnsupported`] evidence. That is a property of
//! this oracle, not a claim about production TC-S1/TC-S2's wider proved action
//! sets: this module deliberately does not restate those sets, and it is not a
//! second HTML parser. In particular the TC-S2 `after body` character-run
//! frontier is *not* re-modelled here; it stays owned by
//! [`super::after_body_successor_validation`].
//!
//! # Termination without a work budget
//!
//! The selected `div` cells add no same-token redispatch edge: a start tag, a
//! matching end tag, and a stray end tag each consume the token in one
//! dispatch, and end of file stops in one dispatch. The inherited shell walk
//! keeps TC-S2's structural proof — [`CandidateSession::process`] asserts that
//! an insertion mode is never evaluated twice while processing one token — so
//! per-token work stays bounded by the mode cardinality. Stack depth grows by
//! exactly one per accepted `div` start tag and shrinks by exactly one per
//! matching end tag, so tree size is bounded by committed semantic token
//! effects. No retry limit, work budget, parser-step budget, nesting limit, or
//! node-count limit exists here or is proposed for production. The bounded
//! enumeration in [`generated_candidate_sequences`] is test infrastructure
//! only and is never a runtime policy.
//!
//! # Canonical byte authority
//!
//! Rendered GitHub text is not fixture-byte authority. DV1–DV14 are stored as
//! escaped byte literals and are checked in-process against their exact byte
//! length and against every byte range the theorem depends on. The canonical
//! SHA-256 digests are retained as documentary metadata in
//! [`CANDIDATE_FIXTURES`] and verified outside the test process, following the
//! accepted TC-S2 precedent: this repository has no dependency-free SHA-256
//! implementation, and inventing general crypto code to satisfy a fixture
//! check would be an unauthorized expansion. Each digest is reproducible from
//! the escaped literal in this file with:
//!
//! ```text
//! printf '<body><div></div>' | sha256sum
//! ```
//!
//! The in-process range checks are the stronger statement for these fixtures,
//! because they pin the exact bytes the theorem reads rather than a summary of
//! the whole file.

use crate::{SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlTagToken, HtmlToken};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;

// ---------------------------------------------------------------------------
// Canonical DV1–DV14 byte authority
// ---------------------------------------------------------------------------

/// One canonical candidate fixture, materialized from the escaped byte
/// sequence recorded in Issue #357.
struct CandidateFixture {
    id: &'static str,
    /// The exact canonical bytes, written as an escaped byte literal.
    bytes: &'static [u8],
    /// The canonical exact byte length.
    length: usize,
    /// The canonical SHA-256 digest of `bytes`, retained as documentary
    /// metadata and verified outside this test process.
    sha256: &'static str,
    /// The byte ranges the candidate theorem depends on, with their exact
    /// expected content.
    required_ranges: &'static [((usize, usize), &'static [u8])],
}

const CANDIDATE_FIXTURES: &[CandidateFixture] = &[
    // <body><div></div>
    CandidateFixture {
        id: "DV1",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e",
        length: 17,
        sha256: "44f9dbc6331c75636ef2eec39853fd6c931b1fba28272c62786ebac16cb4ba84",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 17), b"</div>"),
        ],
    },
    // <body><DiV>x</dIv>
    CandidateFixture {
        id: "DV2",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x44\x69\x56\x3e\x78\x3c\x2f\x64\x49\x76\x3e",
        length: 18,
        sha256: "e6d7d8bdef2d9ab8d87a5c60a50b84a69b62e25ba467eef3dd7def3b02af2ea4",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<DiV>"),
            ((7, 10), b"DiV"),
            ((11, 12), b"\x78"),
            ((12, 18), b"</dIv>"),
            ((14, 17), b"dIv"),
        ],
    },
    // <body><div><div>x</div></div>
    CandidateFixture {
        id: "DV3",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x3c\x64\x69\x76\x3e\x78\x3c\x2f\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e",
        length: 29,
        sha256: "892f118c39733725de1fb73faace78efb3ce71f784f0a2d7b56be4184cf42e37",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 16), b"<div>"),
            ((12, 15), b"div"),
            ((16, 17), b"\x78"),
            ((17, 23), b"</div>"),
            ((23, 29), b"</div>"),
        ],
    },
    // <body><div></div><div></div>
    CandidateFixture {
        id: "DV4",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e\x3c\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e",
        length: 28,
        sha256: "0b3ed57bb102f1b262a6e8e681ed27e9ab75ae1ed6b4f519a27e7445d3a8fc81",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 17), b"</div>"),
            ((17, 22), b"<div>"),
            ((18, 21), b"div"),
            ((22, 28), b"</div>"),
        ],
    },
    // <body></div>
    CandidateFixture {
        id: "DV5",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x64\x69\x76\x3e",
        length: 12,
        sha256: "365a11b5e706c966789fb89d83350dbd9f26d5cfffadc53fa5fce9cfdbdd4e84",
        required_ranges: &[((0, 6), b"<body>"), ((1, 5), b"body"), ((6, 12), b"</div>")],
    },
    // <body><div>x
    CandidateFixture {
        id: "DV6",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x78",
        length: 12,
        sha256: "f2552ec2bc6659b8315e7c1f2c342f2278bbc3efd613c603146382dbd209f04b",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 12), b"\x78"),
        ],
    },
    // <body><div>a<div>b</div>c</div>
    CandidateFixture {
        id: "DV7",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x61\x3c\x64\x69\x76\x3e\x62\x3c\x2f\x64\x69\x76\x3e\x63\x3c\x2f\x64\x69\x76\x3e",
        length: 31,
        sha256: "ddfbed7c8d6a377da9762833373c0ce1b266727fe6863e7490a9022754995b6f",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 12), b"\x61"),
            ((12, 17), b"<div>"),
            ((13, 16), b"div"),
            ((17, 18), b"\x62"),
            ((18, 24), b"</div>"),
            ((24, 25), b"\x63"),
            ((25, 31), b"</div>"),
        ],
    },
    // <body><div id=x>
    CandidateFixture {
        id: "DV8a",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x20\x69\x64\x3d\x78\x3e",
        length: 16,
        sha256: "2365fac0b8ea4475ec187ee0f4ecf7ef9c546e5f82c991fd57b8fc0276110496",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 16), b"<div id=x>"),
            ((7, 10), b"div"),
            ((11, 15), b"id=x"),
        ],
    },
    // <body><div/>
    CandidateFixture {
        id: "DV8b",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x2f\x3e",
        length: 12,
        sha256: "211e3885d26f9cf03b6a15755da5738826e3b5989e2b3fab864d7f5d0dcf7620",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 12), b"<div/>"),
            ((7, 10), b"div"),
            ((10, 11), b"\x2f"),
        ],
    },
    // <body></body><div>
    CandidateFixture {
        id: "DV9",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e",
        length: 18,
        sha256: "5ac460c96f826b009a05235b42483980e351f4e6bc68470358bcf5afb558b173",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 13), b"</body>"),
            ((13, 18), b"<div>"),
            ((14, 17), b"div"),
        ],
    },
    // <body><div></body>
    CandidateFixture {
        id: "DV10",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x3c\x2f\x62\x6f\x64\x79\x3e",
        length: 18,
        sha256: "7ce1ef731dd9fe36fbb191fe420587c66f2bc58cb35654338209b0ade2f18b97",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 18), b"</body>"),
        ],
    },
    // <body><p>
    CandidateFixture {
        id: "DV11",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x70\x3e",
        length: 9,
        sha256: "648ccd6dff0fb3d71045933acb5ea913a0c5566f1d52abb69056a073cbfc1b8c",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 9), b"<p>"),
            ((7, 8), b"\x70"),
        ],
    },
    // <body><div>&amp;
    CandidateFixture {
        id: "DV12",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x26\x61\x6d\x70\x3b",
        length: 16,
        sha256: "174bb8c3b05e81890cb3a9cd0388c3c4e22aa74d25bcf0a16018af07debcb910",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 16), b"&amp;"),
        ],
    },
    // x<div></div>
    CandidateFixture {
        id: "DV13",
        bytes: b"\x78\x3c\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e",
        length: 12,
        sha256: "1a155a4c794347039cc5791bc4d109e38236b5611de00ad96f5aca34470a859c",
        required_ranges: &[
            ((0, 1), b"\x78"),
            ((1, 6), b"<div>"),
            ((2, 5), b"div"),
            ((6, 12), b"</div>"),
        ],
    },
    // <body><div></div></body>
    CandidateFixture {
        id: "DV14",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e\x3c\x2f\x62\x6f\x64\x79\x3e",
        length: 24,
        sha256: "ec7b7eee750428e84c00289fc830b8657bdef0c7ed00bc0102ed243fb18df2cc",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 17), b"</div>"),
            ((17, 24), b"</body>"),
        ],
    },
];

const CANDIDATE_IDS: [&str; 15] = [
    "DV1", "DV2", "DV3", "DV4", "DV5", "DV6", "DV7", "DV8a", "DV8b", "DV9", "DV10", "DV11", "DV12",
    "DV13", "DV14",
];

fn fixture(id: &str) -> &'static CandidateFixture {
    CANDIDATE_FIXTURES
        .iter()
        .find(|candidate| candidate.id == id)
        .expect("canonical candidate fixture")
}

impl CandidateFixture {
    /// The canonical bytes as source text. Panics rather than lossily
    /// converting: a fixture that is not valid UTF-8 is a stop condition, not
    /// something to normalize.
    fn source_text(&self) -> &'static str {
        std::str::from_utf8(self.bytes).expect("canonical fixture bytes are valid UTF-8")
    }
}

// ---------------------------------------------------------------------------
// Independent candidate domain
// ---------------------------------------------------------------------------

/// Candidate insertion modes. Test-only, and deliberately not the production
/// mode type. Exactly the modes DV1–DV14 traverse are modelled; `after after
/// body` is outside this oracle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    AfterBody,
}

impl CandidateMode {
    /// Every modelled mode. Used to state the structural per-token dispatch
    /// bound as a cardinality fact rather than an invented constant.
    const ALL: [Self; 7] = [
        Self::Initial,
        Self::BeforeHtml,
        Self::BeforeHead,
        Self::InHead,
        Self::AfterHead,
        Self::InBody,
        Self::AfterBody,
    ];
}

/// The candidate's fixed element namespace.
///
/// TC-S3 deliberately selects no namespace architecture. The candidate `div`
/// is HTML-namespaced by construction, and this single-variant type makes that
/// an asserted invariant rather than an unstated assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateNamespace {
    Html,
}

/// The candidate's fixed element-name set.
///
/// This is deliberately a closed enum rather than a general
/// `ElementName(String)`: TC-S3 admits exactly one new ordinary element and
/// chooses no production name representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateElementName {
    Html,
    Head,
    Body,
    Div,
}

impl CandidateElementName {
    /// The candidate-local projection of the pinned "generate implied end
    /// tags" element list (`dd`, `dt`, `li`, `optgroup`, `option`, `p`, `rb`,
    /// `rp`, `rt`, `rtc`). No name in this candidate's fixed set is in it, so
    /// implied-end generation is a structural no-op here — which is the
    /// property [`CandidateSession::generate_implied_end_tags`] proves rather
    /// than assumes.
    fn is_implied_end_element(self) -> bool {
        match self {
            Self::Html | Self::Head | Self::Body | Self::Div => false,
        }
    }

    /// The candidate-local projection of the pinned "have a particular element
    /// in scope" element-type list (`applet`, `caption`, `html`, `table`,
    /// `td`, `th`, `marquee`, `object`, `select`, `template`, and the listed
    /// MathML and SVG elements). Of this candidate's fixed set only `html` is
    /// in that list.
    fn is_scope_boundary(self) -> bool {
        match self {
            Self::Html => true,
            Self::Head | Self::Body | Self::Div => false,
        }
    }

    /// The pinned `in body` end-of-file rule records a parse error when the
    /// stack of open elements holds a node that is not one of `dd`, `dt`,
    /// `li`, `optgroup`, `option`, `p`, `rb`, `rp`, `rt`, `rtc`, `tbody`,
    /// `td`, `tfoot`, `th`, `thead`, `tr`, `body`, or `html`. `div` is not in
    /// that list; `body` and `html` are.
    fn is_permitted_open_element_at_end_of_file(self) -> bool {
        match self {
            Self::Body | Self::Html => true,
            Self::Head | Self::Div => false,
        }
    }

    fn is_shell(self) -> bool {
        match self {
            Self::Html | Self::Head | Self::Body => true,
            Self::Div => false,
        }
    }
}

/// Whether the candidate parser's scripting flag is enabled.
///
/// Fixed to [`CandidateScripting::Disabled`] by the candidate invariant and
/// asserted rather than assumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateScripting {
    Disabled,
}

/// The candidate's fixed parse configuration.
///
/// Every field is part of the Issue #357 candidate state invariant. They are
/// stored on the session and asserted on every dispatch so the theorem cannot
/// quietly depend on an unstated assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateConfiguration {
    scripting: CandidateScripting,
    fragment_parse: bool,
    template_state: bool,
    table_state: bool,
    foreign_content_state: bool,
    reentrant_state: bool,
}

impl CandidateConfiguration {
    const FIXED: Self = Self {
        scripting: CandidateScripting::Disabled,
        fragment_parse: false,
        template_state: false,
        table_state: false,
        foreign_content_state: false,
        reentrant_state: false,
    };
}

/// The HTML whitespace set this candidate fixes.
fn is_candidate_html_whitespace(character: char) -> bool {
    matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' ')
}

/// The candidate's normalization of one accepted lower-layer token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateTokenShape<'run> {
    Characters {
        range: (usize, usize),
        interpreted: &'run str,
    },
    StartTag {
        name: CandidateElementName,
        /// The exact complete authored start-tag anchor.
        complete: (usize, usize),
        /// The exact authored raw-name anchor.
        raw_name: (usize, usize),
    },
    EndTag {
        name: CandidateElementName,
        complete: (usize, usize),
    },
    EndOfFile {
        at: usize,
    },
}

impl CandidateTokenShape<'_> {
    /// The exclusive source offset a committed processing of this token covers.
    fn committed_end(&self) -> usize {
        match self {
            Self::Characters { range, .. } => range.1,
            Self::StartTag { complete, .. } | Self::EndTag { complete, .. } => complete.1,
            Self::EndOfFile { at } => *at,
        }
    }

    /// The shell element name this token names, if it names one.
    fn shell_tag_name(&self) -> Option<CandidateElementName> {
        match self {
            Self::StartTag { name, .. } | Self::EndTag { name, .. } if name.is_shell() => {
                Some(*name)
            }
            _ => None,
        }
    }

    fn is_div_tag(&self) -> bool {
        matches!(
            self,
            Self::StartTag {
                name: CandidateElementName::Div,
                ..
            } | Self::EndTag {
                name: CandidateElementName::Div,
                ..
            }
        )
    }
}

/// What the candidate refuses, with exact typed meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateUnsupported {
    /// A tag carrying attributes. TC-S3 admits no attribute semantics.
    TagWithAttributes,
    /// A tag carrying a self-closing solidus.
    SelfClosingTag,
    /// A `div` tag first evaluated outside actual `InBody`.
    DivTagOutsideInBody,
    /// A shell-element interaction reached while a candidate `div` is still
    /// open. TC-S3 proves no such interleaving.
    ShellInteractionWithOpenDiv,
    /// A character run whose handling outside `in body` depends on a
    /// whitespace distinction the candidate does not extend there.
    WhitespaceSensitiveCharacterData,
    /// A cell this deliberately partial oracle does not model. Says nothing
    /// about production TC-S1/TC-S2's proved action sets.
    OutsideModelledCandidateCells,
}

/// Where a candidate element node's existence comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateOrigin {
    /// The trigger token's own authored start tag: the exact complete anchor
    /// and the exact raw-name anchor, both retained, never reconstructed.
    Authored {
        complete: (usize, usize),
        raw_name: (usize, usize),
    },
    /// No authored source. The trigger token made the node necessary but is
    /// not its origin.
    Synthesized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateProvenance {
    AuthoredByTriggerToken,
    Synthesized,
}

/// Which emitted token caused an observation. Never an authored origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateTrigger {
    Authored {
        index: usize,
        range: (usize, usize),
    },
    /// End of file has no authored extent and gets no dummy span.
    EndOfFile {
        index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDiagnosticCode {
    MissingDoctype,
    /// The pinned end-tag rule's "not in scope" branch.
    UnmatchedDivEndTag,
    /// The pinned `in body` end-of-file rule's parse error for a node on the
    /// stack outside its permitted list.
    OpenOrdinaryElementAtEndOfFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateRecovery {
    ContinuedInQuirksDocumentMode,
    /// The token advances committed coverage and commits nothing else.
    IgnoredToken,
    /// Parsing stops normally; no end tag, closure anchor, or close action is
    /// fabricated.
    StoppedParsingWithOpenElements,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateDiagnostic {
    code: CandidateDiagnosticCode,
    trigger: CandidateTrigger,
    recovery: CandidateRecovery,
}

/// What one rule dispatch did with the current token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDispatchOutcome {
    Consumed,
    /// The token advanced committed coverage but committed no tree, stack, or
    /// insertion-mode effect.
    ConsumedAsIgnored,
    Reprocessed,
    Stopped,
    /// Nothing was mutated by this cell.
    Refused(CandidateUnsupported),
}

/// One evaluation of one insertion-mode rule for one emitted token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateDispatch {
    /// The candidate's actual insertion mode when the rule was selected.
    evaluated_in: CandidateMode,
    outcome: CandidateDispatchOutcome,
}

/// Everything one emitted token did.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateTokenRecord {
    index: usize,
    mode_before: CandidateMode,
    mode_after: CandidateMode,
    dispatches: Vec<CandidateDispatch>,
    /// Same-token reprocessing count. Not a budget: an observation.
    reprocesses: usize,
    /// `k` in `S(k) = [html, body] ++ [div]^k` after this token.
    open_div_depth_after: usize,
    committed_prefix_end: usize,
}

impl CandidateTokenRecord {
    fn refusal(&self) -> Option<CandidateUnsupported> {
        match self.dispatches.last()?.outcome {
            CandidateDispatchOutcome::Refused(capability) => Some(capability),
            _ => None,
        }
    }

    fn stopped(&self) -> bool {
        matches!(
            self.dispatches.last().map(|dispatch| dispatch.outcome),
            Some(CandidateDispatchOutcome::Stopped)
        )
    }
}

/// The candidate's final tree shape, projected for comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTree {
    Document(Vec<CandidateTree>),
    Element {
        name: CandidateElementName,
        namespace: CandidateNamespace,
        origin: CandidateOrigin,
        children: Vec<CandidateTree>,
    },
    Text {
        interpreted: String,
        /// Ordered, individually retained source contributions.
        contributions: Vec<(usize, usize)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateCompletion {
    Complete,
    /// The candidate stopped at exactly this capability and trigger.
    IncompleteUnsupported {
        capability: CandidateUnsupported,
        trigger: CandidateTrigger,
    },
    /// Lower-layer evidence was not complete, so the candidate cannot be.
    IncompleteLowerLayer,
}

/// The terminal candidate checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateCheckpoint {
    mode: CandidateMode,
    open_div_depth: usize,
    committed_prefix_end: usize,
    completion: CandidateCompletion,
}

/// The complete independent observation of one candidate run.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateObservation {
    tree: CandidateTree,
    diagnostics: Vec<CandidateDiagnostic>,
    tokens: Vec<CandidateTokenRecord>,
    /// Semantic creation events. Coalescing consumes none. Never a raw ID.
    identity_events: usize,
    checkpoint: CandidateCheckpoint,
}

// ---------------------------------------------------------------------------
// Independent candidate rule table
// ---------------------------------------------------------------------------

/// One effect a candidate rule commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateEffect {
    RecordMissingDoctype,
    InsertShellElement {
        name: CandidateElementName,
        provenance: CandidateProvenance,
    },
    CloseHeadElement,
    /// Disposition evidence only: creates no node, no text, no identity.
    AcknowledgeShellEndTag(CandidateElementName),
    InsertCharacters,
    /// The selected TC-S3 start-tag rule.
    InsertDivElement,
    /// The selected TC-S3 matching end-tag rule.
    PopMatchingDivElement,
    RecordUnmatchedDivEndTag,
    RecordOpenOrdinaryElementAtEndOfFile,
}

/// What a candidate rule does with the current token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateStep {
    Consume {
        effect: Option<CandidateEffect>,
        next: Option<CandidateMode>,
    },
    /// Parse-error recovery: the token is consumed, but the cell commits no
    /// tree, stack, or insertion-mode effect beyond the diagnostic itself.
    ConsumeAsIgnored {
        effect: CandidateEffect,
    },
    Reprocess {
        effect: Option<CandidateEffect>,
        next: CandidateMode,
    },
    Stop {
        effect: Option<CandidateEffect>,
    },
}

/// The immutable candidate stack summary a rule may read.
///
/// Passing a `Copy` summary rather than the session keeps [`select`] pure, so
/// a refusal is structurally guaranteed to precede any mutation by that cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateStackSummary {
    /// `k` in `S(k) = [html, body] ++ [div]^k`.
    open_div_depth: usize,
}

/// Selects the candidate rule for one (mode, stack, token) cell.
///
/// Pure: it takes no mutable session state and mutates nothing.
fn select(
    mode: CandidateMode,
    stack: CandidateStackSummary,
    shape: &CandidateTokenShape<'_>,
) -> Result<CandidateStep, CandidateUnsupported> {
    match mode {
        CandidateMode::Initial => {
            reject_whitespace_sensitive(shape)?;
            expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::RecordMissingDoctype),
                next: CandidateMode::BeforeHtml,
            })
        }
        CandidateMode::BeforeHtml => {
            reject_whitespace_sensitive(shape)?;
            expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::InsertShellElement {
                    name: CandidateElementName::Html,
                    provenance: CandidateProvenance::Synthesized,
                }),
                next: CandidateMode::BeforeHead,
            })
        }
        CandidateMode::BeforeHead => {
            reject_whitespace_sensitive(shape)?;
            expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::InsertShellElement {
                    name: CandidateElementName::Head,
                    provenance: CandidateProvenance::Synthesized,
                }),
                next: CandidateMode::InHead,
            })
        }
        CandidateMode::InHead => {
            reject_whitespace_sensitive(shape)?;
            expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::CloseHeadElement),
                next: CandidateMode::AfterHead,
            })
        }
        // The pinned `after head` rules split here: an authored `body` start
        // tag is consumed and becomes the body element's origin, while the
        // "anything else" branch synthesizes a body and reprocesses. Only a
        // non-whitespace character run reaches the second branch in this
        // deliberately partial oracle.
        CandidateMode::AfterHead => {
            reject_whitespace_sensitive(shape)?;
            expect_shell_walk_trigger(shape)?;
            match shape {
                CandidateTokenShape::StartTag {
                    name: CandidateElementName::Body,
                    ..
                } => Ok(CandidateStep::Consume {
                    effect: Some(CandidateEffect::InsertShellElement {
                        name: CandidateElementName::Body,
                        provenance: CandidateProvenance::AuthoredByTriggerToken,
                    }),
                    next: Some(CandidateMode::InBody),
                }),
                _ => Ok(CandidateStep::Reprocess {
                    effect: Some(CandidateEffect::InsertShellElement {
                        name: CandidateElementName::Body,
                        provenance: CandidateProvenance::Synthesized,
                    }),
                    next: CandidateMode::InBody,
                }),
            }
        }
        // The TC-S3 frontier.
        CandidateMode::InBody => {
            // Any shell-element interaction reached while a candidate `div`
            // is still open is outside the theorem and is refused before the
            // cell mutates anything.
            if shape.shell_tag_name().is_some() && stack.open_div_depth > 0 {
                return Err(CandidateUnsupported::ShellInteractionWithOpenDiv);
            }
            match shape {
                // Inside `in body` whitespace and non-whitespace characters
                // are inserted identically, so an aggregate run needs no
                // splitting and no whitespace refusal.
                CandidateTokenShape::Characters { .. } => Ok(CandidateStep::Consume {
                    effect: Some(CandidateEffect::InsertCharacters),
                    next: None,
                }),
                CandidateTokenShape::StartTag {
                    name: CandidateElementName::Div,
                    ..
                } => Ok(CandidateStep::Consume {
                    effect: Some(CandidateEffect::InsertDivElement),
                    next: None,
                }),
                CandidateTokenShape::EndTag {
                    name: CandidateElementName::Div,
                    ..
                } => {
                    if stack.open_div_depth > 0 {
                        Ok(CandidateStep::Consume {
                            effect: Some(CandidateEffect::PopMatchingDivElement),
                            next: None,
                        })
                    } else {
                        Ok(CandidateStep::ConsumeAsIgnored {
                            effect: CandidateEffect::RecordUnmatchedDivEndTag,
                        })
                    }
                }
                CandidateTokenShape::EndTag {
                    name: CandidateElementName::Body,
                    ..
                } => Ok(CandidateStep::Consume {
                    effect: Some(CandidateEffect::AcknowledgeShellEndTag(
                        CandidateElementName::Body,
                    )),
                    next: Some(CandidateMode::AfterBody),
                }),
                CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop {
                    effect: (stack.open_div_depth > 0)
                        .then_some(CandidateEffect::RecordOpenOrdinaryElementAtEndOfFile),
                }),
                _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
            }
        }
        CandidateMode::AfterBody => match shape {
            CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop { effect: None }),
            _ if shape.is_div_tag() => Err(CandidateUnsupported::DivTagOutsideInBody),
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
    }
}

/// Refuses a character run whose handling in the current mode would depend on a
/// whitespace distinction the candidate does not extend outside `in body`.
fn reject_whitespace_sensitive(
    shape: &CandidateTokenShape<'_>,
) -> Result<(), CandidateUnsupported> {
    match shape {
        CandidateTokenShape::Characters { interpreted, .. }
            if interpreted.chars().any(is_candidate_html_whitespace) =>
        {
            Err(CandidateUnsupported::WhitespaceSensitiveCharacterData)
        }
        _ => Ok(()),
    }
}

/// The DV shell prefix is uniform: either an authored `body` start tag or a
/// non-whitespace character run walks the shell modes. Everything else in
/// those modes is outside this deliberately partial model, and a `div` tag
/// there is refused with its own typed meaning because TC-S3 admits `div`
/// only when the actual insertion mode is already `in body`.
fn expect_shell_walk_trigger(shape: &CandidateTokenShape<'_>) -> Result<(), CandidateUnsupported> {
    if shape.is_div_tag() {
        return Err(CandidateUnsupported::DivTagOutsideInBody);
    }
    match shape {
        CandidateTokenShape::StartTag {
            name: CandidateElementName::Body,
            ..
        }
        | CandidateTokenShape::Characters { .. } => Ok(()),
        _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
    }
}

// ---------------------------------------------------------------------------
// Independent candidate machine
// ---------------------------------------------------------------------------

/// A node in the candidate's construction arena. Storage positions are private
/// working state and never become durable meaning.
#[derive(Debug, Clone)]
enum CandidateArenaKind {
    Document,
    Element {
        name: CandidateElementName,
        namespace: CandidateNamespace,
        origin: CandidateOrigin,
    },
    Text {
        interpreted: String,
        contributions: Vec<(usize, usize)>,
    },
    /// A slot that is never referenced by any parent. It exists only to move
    /// real nodes to different storage indices so that identity-based
    /// relationship meaning can be shown not to be storage-index based.
    UnusedStorageSlot,
}

#[derive(Debug, Clone)]
struct CandidateArenaNode {
    children: Vec<usize>,
    kind: CandidateArenaKind,
}

/// How the private arena lays real nodes out in storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateStorageLayout {
    /// Unreferenced slots inserted before each real allocation.
    padding_before_each_node: usize,
}

impl CandidateStorageLayout {
    const COMPACT: Self = Self {
        padding_before_each_node: 0,
    };
}

/// A snapshot of everything the transaction theorem says a refused cell must
/// leave unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateStateFingerprint {
    referenced_nodes: usize,
    open_elements: usize,
    open_div_depth: usize,
    mode: CandidateMode,
    diagnostics: usize,
    identity_events: usize,
    committed_prefix_end: usize,
}

/// The test-only candidate construction machine.
struct CandidateSession {
    configuration: CandidateConfiguration,
    layout: CandidateStorageLayout,
    nodes: Vec<CandidateArenaNode>,
    document: usize,
    open_elements: Vec<usize>,
    /// The candidate's list of active formatting elements. TC-S3 proves no
    /// cell that pushes onto it, so it must stay empty; it exists so that the
    /// invariant is asserted rather than assumed.
    active_formatting_elements: Vec<usize>,
    head_element: Option<usize>,
    mode: CandidateMode,
    diagnostics: Vec<CandidateDiagnostic>,
    identity_events: usize,
    committed_prefix_end: usize,
    processed_tokens: usize,
}

impl CandidateSession {
    fn new(layout: CandidateStorageLayout) -> Self {
        let mut session = Self {
            configuration: CandidateConfiguration::FIXED,
            layout,
            nodes: Vec::new(),
            document: 0,
            open_elements: Vec::new(),
            active_formatting_elements: Vec::new(),
            head_element: None,
            mode: CandidateMode::Initial,
            diagnostics: Vec::new(),
            identity_events: 0,
            committed_prefix_end: 0,
            processed_tokens: 0,
        };
        session.document = session.allocate(CandidateArenaKind::Document);
        // The Document container is one semantic creation event.
        session.identity_events += 1;
        session
    }

    /// Allocates one arena slot for `kind`, honouring the storage layout.
    fn allocate(&mut self, kind: CandidateArenaKind) -> usize {
        for _ in 0..self.layout.padding_before_each_node {
            self.nodes.push(CandidateArenaNode {
                children: Vec::new(),
                kind: CandidateArenaKind::UnusedStorageSlot,
            });
        }
        let index = self.nodes.len();
        self.nodes.push(CandidateArenaNode {
            children: Vec::new(),
            kind,
        });
        index
    }

    // -- pinned algorithms over the candidate's own stack --------------------

    fn element_name(&self, node: usize) -> CandidateElementName {
        match self.nodes[node].kind {
            CandidateArenaKind::Element { name, .. } => name,
            _ => panic!("an open-elements entry must be an element"),
        }
    }

    fn current_node(&self) -> Option<usize> {
        self.open_elements.last().copied()
    }

    /// The pinned "have an element in a specific scope" algorithm, run over the
    /// candidate's own stack with the pinned "in scope" element-type list.
    fn has_element_in_scope(&self, target: CandidateElementName) -> bool {
        for node in self.open_elements.iter().rev() {
            let name = self.element_name(*node);
            if name == target {
                return true;
            }
            if name.is_scope_boundary() {
                return false;
            }
        }
        false
    }

    /// The pinned "have a particular element in button scope" algorithm for a
    /// `p` element. The candidate's fixed element-name set contains no `p`, so
    /// this walk cannot match — which is exactly the branch condition the
    /// TC-S3 start-tag theorem claims is false, proved here against the live
    /// stack rather than assumed.
    fn has_p_in_button_scope(&self) -> bool {
        for node in self.open_elements.iter().rev() {
            let name = self.element_name(*node);
            // No candidate name is `p`; the button-scope list additionally
            // contains `button`, which is likewise outside the fixed set.
            if name.is_scope_boundary() {
                return false;
            }
        }
        false
    }

    /// The pinned "generate implied end tags" algorithm. Returns how many
    /// elements it popped, so the theorem's "proved no-op" claim is observed
    /// rather than asserted by construction.
    fn generate_implied_end_tags(&mut self) -> usize {
        let mut popped = 0;
        while let Some(node) = self.current_node() {
            if !self.element_name(node).is_implied_end_element() {
                break;
            }
            self.open_elements.pop();
            popped += 1;
        }
        popped
    }

    fn open_div_depth(&self) -> usize {
        self.open_elements
            .iter()
            .filter(|node| self.element_name(**node) == CandidateElementName::Div)
            .count()
    }

    fn stack_summary(&self) -> CandidateStackSummary {
        CandidateStackSummary {
            open_div_depth: self.open_div_depth(),
        }
    }

    fn open_element_names(&self) -> Vec<CandidateElementName> {
        self.open_elements
            .iter()
            .map(|node| self.element_name(*node))
            .collect()
    }

    // -- candidate state invariant ------------------------------------------

    /// Asserts the complete Issue #357 candidate state invariant.
    ///
    /// Every clause is checked against live session state on every dispatch
    /// boundary, so none of the theorem's assumptions can hold silently.
    fn assert_candidate_invariant(&self) {
        assert_eq!(
            self.configuration,
            CandidateConfiguration::FIXED,
            "candidate parse configuration must stay fixed: scripting disabled, non-fragment, and \
             free of template, table, foreign-content, and reentrant state"
        );
        assert_eq!(
            self.configuration.scripting,
            CandidateScripting::Disabled,
            "candidate scripting mode must remain Disabled"
        );
        assert!(
            self.active_formatting_elements.is_empty(),
            "the candidate list of active formatting elements must remain empty"
        );

        for node in &self.open_elements {
            let CandidateArenaKind::Element {
                name, namespace, ..
            } = self.nodes[*node].kind
            else {
                panic!("an open-elements entry must be an element")
            };
            assert_eq!(
                namespace,
                CandidateNamespace::Html,
                "every candidate open element is HTML-namespaced"
            );
            assert!(
                !name.is_implied_end_element(),
                "no implied-end element may become reachable on the candidate stack: {name:?}"
            );
            assert!(
                name == CandidateElementName::Html || !name.is_scope_boundary(),
                "no scope boundary other than `html` may become reachable: {name:?}"
            );
        }

        // `S(k) = [html, body] ++ [div]^k`, plus the strictly smaller shell
        // prefixes the walk passes through before `body` exists.
        let names = self.open_element_names();
        let valid = match names.as_slice() {
            [] | [CandidateElementName::Html] => true,
            [CandidateElementName::Html, CandidateElementName::Head] => true,
            [
                CandidateElementName::Html,
                CandidateElementName::Body,
                divs @ ..,
            ] => divs.iter().all(|name| *name == CandidateElementName::Div),
            _ => false,
        };
        assert!(
            valid,
            "the candidate open-elements stack must stay S(k) = [html, body] ++ [div]^k: {names:?}"
        );
    }

    fn fingerprint(&self) -> CandidateStateFingerprint {
        CandidateStateFingerprint {
            referenced_nodes: self.referenced_node_count(),
            open_elements: self.open_elements.len(),
            open_div_depth: self.open_div_depth(),
            mode: self.mode,
            diagnostics: self.diagnostics.len(),
            identity_events: self.identity_events,
            committed_prefix_end: self.committed_prefix_end,
        }
    }

    /// Counts only nodes reachable from the document, so unreferenced storage
    /// padding can never be mistaken for constructed meaning.
    fn referenced_node_count(&self) -> usize {
        fn walk(session: &CandidateSession, node: usize) -> usize {
            1 + session.nodes[node]
                .children
                .iter()
                .map(|child| walk(session, *child))
                .sum::<usize>()
        }
        walk(self, self.document)
    }
}

impl CandidateSession {
    /// Processes one emitted token to a terminal disposition.
    ///
    /// Termination is structural: an insertion mode is never evaluated twice
    /// for the same token. The assertion *is* the theorem obligation — if the
    /// candidate action set admitted a same-token cycle, this would fire
    /// instead of looping, and TC-S3 would be falsified.
    ///
    /// The transaction theorem is enforced here rather than described: the
    /// session is fingerprinted before every rule selection, and a refused
    /// cell must leave that fingerprint bit-identical.
    fn process(
        &mut self,
        index: usize,
        shape: CandidateTokenShape<'_>,
        trigger: CandidateTrigger,
    ) -> CandidateTokenRecord {
        let mode_before = self.mode;
        let mut dispatches = Vec::new();
        let mut visited: Vec<CandidateMode> = Vec::new();
        let mut reprocesses = 0;

        loop {
            self.assert_candidate_invariant();
            assert!(
                !visited.contains(&self.mode),
                "candidate theorem falsified: insertion mode {:?} was evaluated twice while \
                 processing token {index}",
                self.mode
            );
            visited.push(self.mode);

            let before = self.fingerprint();
            let evaluated_in = self.mode;
            let step = select(self.mode, self.stack_summary(), &shape);

            match step {
                Err(capability) => {
                    assert_eq!(
                        self.fingerprint(),
                        before,
                        "a refused candidate cell must mutate nothing"
                    );
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        outcome: CandidateDispatchOutcome::Refused(capability),
                    });
                    break;
                }
                Ok(CandidateStep::Stop { effect }) => {
                    if let Some(effect) = effect {
                        self.apply(effect, trigger, &shape);
                    }
                    self.commit(&shape);
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        outcome: CandidateDispatchOutcome::Stopped,
                    });
                    break;
                }
                Ok(CandidateStep::Consume { effect, next }) => {
                    if let Some(effect) = effect {
                        self.apply(effect, trigger, &shape);
                    }
                    if let Some(next) = next {
                        self.mode = next;
                    }
                    self.commit(&shape);
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        outcome: CandidateDispatchOutcome::Consumed,
                    });
                    break;
                }
                Ok(CandidateStep::ConsumeAsIgnored { effect }) => {
                    self.apply(effect, trigger, &shape);
                    self.commit(&shape);
                    let after = self.fingerprint();
                    assert_eq!(
                        (
                            after.referenced_nodes,
                            after.open_elements,
                            after.open_div_depth,
                            after.mode,
                            after.identity_events,
                        ),
                        (
                            before.referenced_nodes,
                            before.open_elements,
                            before.open_div_depth,
                            before.mode,
                            before.identity_events,
                        ),
                        "an ignored candidate token must mutate no tree, stack, insertion mode, \
                         or identity state"
                    );
                    assert_eq!(
                        after.diagnostics,
                        before.diagnostics + 1,
                        "an ignored candidate token records exactly one diagnostic"
                    );
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        outcome: CandidateDispatchOutcome::ConsumedAsIgnored,
                    });
                    break;
                }
                Ok(CandidateStep::Reprocess { effect, next }) => {
                    if let Some(effect) = effect {
                        self.apply(effect, trigger, &shape);
                    }
                    self.mode = next;
                    reprocesses += 1;
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        outcome: CandidateDispatchOutcome::Reprocessed,
                    });
                }
            }
        }

        self.assert_candidate_invariant();
        CandidateTokenRecord {
            index,
            mode_before,
            mode_after: self.mode,
            dispatches,
            reprocesses,
            open_div_depth_after: self.open_div_depth(),
            committed_prefix_end: self.committed_prefix_end,
        }
    }

    fn apply(
        &mut self,
        effect: CandidateEffect,
        trigger: CandidateTrigger,
        shape: &CandidateTokenShape<'_>,
    ) {
        match effect {
            CandidateEffect::RecordMissingDoctype => self.diagnostics.push(CandidateDiagnostic {
                code: CandidateDiagnosticCode::MissingDoctype,
                trigger,
                recovery: CandidateRecovery::ContinuedInQuirksDocumentMode,
            }),
            CandidateEffect::RecordUnmatchedDivEndTag => {
                // The pinned end-tag rule's "not in scope" branch, proved
                // against the live candidate stack.
                assert!(
                    !self.has_element_in_scope(CandidateElementName::Div),
                    "an unmatched `div` end tag requires that no `div` is in scope"
                );
                self.diagnostics.push(CandidateDiagnostic {
                    code: CandidateDiagnosticCode::UnmatchedDivEndTag,
                    trigger,
                    recovery: CandidateRecovery::IgnoredToken,
                });
            }
            CandidateEffect::RecordOpenOrdinaryElementAtEndOfFile => {
                assert!(
                    self.open_element_names()
                        .iter()
                        .any(|name| !name.is_permitted_open_element_at_end_of_file()),
                    "the end-of-file parse error requires an open element outside the pinned \
                     permitted list"
                );
                assert!(
                    matches!(trigger, CandidateTrigger::EndOfFile { .. }),
                    "the end-of-file diagnostic is triggered by the end-of-file token and \
                     receives no authored source anchor"
                );
                self.diagnostics.push(CandidateDiagnostic {
                    code: CandidateDiagnosticCode::OpenOrdinaryElementAtEndOfFile,
                    trigger,
                    recovery: CandidateRecovery::StoppedParsingWithOpenElements,
                });
            }
            CandidateEffect::InsertShellElement { name, provenance } => {
                self.insert_shell_element(name, provenance, shape);
            }
            CandidateEffect::CloseHeadElement => {
                let head = self.head_element.expect("an open head element");
                assert_eq!(
                    self.open_elements.last(),
                    Some(&head),
                    "head must be the open element when it is closed"
                );
                self.open_elements.pop();
            }
            // Acknowledgement is disposition evidence only.
            CandidateEffect::AcknowledgeShellEndTag(_) => {}
            CandidateEffect::InsertCharacters => self.insert_characters(shape),
            CandidateEffect::InsertDivElement => self.insert_div_element(shape),
            CandidateEffect::PopMatchingDivElement => self.pop_matching_div_element(),
        }
    }

    fn insert_shell_element(
        &mut self,
        name: CandidateElementName,
        provenance: CandidateProvenance,
        shape: &CandidateTokenShape<'_>,
    ) {
        let parent = match name {
            CandidateElementName::Html => self.document,
            _ => self
                .current_node()
                .expect("an open insertion parent for a nested shell element"),
        };
        let origin = match provenance {
            CandidateProvenance::AuthoredByTriggerToken => {
                let CandidateTokenShape::StartTag {
                    complete, raw_name, ..
                } = shape
                else {
                    panic!("authored insertion requires the trigger token's own start tag")
                };
                CandidateOrigin::Authored {
                    complete: *complete,
                    raw_name: *raw_name,
                }
            }
            CandidateProvenance::Synthesized => CandidateOrigin::Synthesized,
        };
        let inserted = self.allocate(CandidateArenaKind::Element {
            name,
            namespace: CandidateNamespace::Html,
            origin,
        });
        self.nodes[parent].children.push(inserted);
        self.open_elements.push(inserted);
        self.identity_events += 1;
        if name == CandidateElementName::Head {
            self.head_element = Some(inserted);
        }
    }

    /// The selected TC-S3 `in body` `div` start-tag rule.
    ///
    /// The pinned rule's first step is the `p`-in-button-scope branch; it is
    /// proved false against the live stack here, before any mutation, rather
    /// than assumed away.
    fn insert_div_element(&mut self, shape: &CandidateTokenShape<'_>) {
        assert!(
            !self.has_p_in_button_scope(),
            "the candidate invariant makes the `p`-in-button-scope branch false"
        );
        let CandidateTokenShape::StartTag {
            name: CandidateElementName::Div,
            complete,
            raw_name,
        } = shape
        else {
            panic!("div insertion requires the trigger token's own `div` start tag")
        };
        let parent = self
            .current_node()
            .expect("an open insertion parent for a candidate div");
        let inserted = self.allocate(CandidateArenaKind::Element {
            name: CandidateElementName::Div,
            namespace: CandidateNamespace::Html,
            origin: CandidateOrigin::Authored {
                complete: *complete,
                raw_name: *raw_name,
            },
        });
        self.nodes[parent].children.push(inserted);
        self.open_elements.push(inserted);
        self.identity_events += 1;
    }

    /// The selected TC-S3 `in body` matching `div` end-tag rule.
    ///
    /// Each pinned step is executed and its claimed outcome asserted: the
    /// element is in scope, implied-end generation pops nothing, the current
    /// node is the matching `div`, and exactly one element is popped. The end
    /// tag admits no identity and becomes no node's origin.
    fn pop_matching_div_element(&mut self) {
        assert!(
            self.has_element_in_scope(CandidateElementName::Div),
            "a matching `div` end tag requires a `div` in scope"
        );
        let popped_by_implied_end = self.generate_implied_end_tags();
        assert_eq!(
            popped_by_implied_end, 0,
            "implied-end generation is a proved no-op under the candidate invariant"
        );
        let current = self
            .current_node()
            .expect("a current node for a matching div end tag");
        assert_eq!(
            self.element_name(current),
            CandidateElementName::Div,
            "the current node is the matching `div`, so the pinned current-node parse error \
             branch is false"
        );
        let depth_before = self.open_elements.len();
        self.open_elements.pop();
        assert_eq!(
            self.open_elements.len(),
            depth_before - 1,
            "exactly one element is popped"
        );
    }

    /// Inserts the token's characters, coalescing into the adjacent text node
    /// when one is already the last child of the insertion parent.
    ///
    /// Coalescing appends an ordered contribution and consumes no new identity
    /// event. Contributions are retained individually and are never merged into
    /// a reconstructed span.
    fn insert_characters(&mut self, shape: &CandidateTokenShape<'_>) {
        let CandidateTokenShape::Characters { range, interpreted } = shape else {
            panic!("character insertion requires a character token")
        };
        let parent = self
            .current_node()
            .expect("an open insertion target for character data");
        let adjacent_text = self.nodes[parent]
            .children
            .last()
            .copied()
            .filter(|child| matches!(self.nodes[*child].kind, CandidateArenaKind::Text { .. }));

        if let Some(text) = adjacent_text {
            let CandidateArenaKind::Text {
                interpreted: existing,
                contributions,
            } = &mut self.nodes[text].kind
            else {
                unreachable!("filtered to a text node")
            };
            existing.push_str(interpreted);
            contributions.push(*range);
            return;
        }

        let inserted = self.allocate(CandidateArenaKind::Text {
            interpreted: (*interpreted).to_owned(),
            contributions: vec![*range],
        });
        self.nodes[parent].children.push(inserted);
        self.identity_events += 1;
    }

    fn commit(&mut self, shape: &CandidateTokenShape<'_>) {
        let end = shape.committed_end();
        assert!(
            end >= self.committed_prefix_end,
            "committed candidate coverage must not move backwards"
        );
        self.committed_prefix_end = end;
        self.processed_tokens += 1;
    }

    fn tree(&self) -> CandidateTree {
        self.project(self.document)
    }

    fn project(&self, node: usize) -> CandidateTree {
        let children = || {
            self.nodes[node]
                .children
                .iter()
                .map(|child| self.project(*child))
                .collect()
        };
        match &self.nodes[node].kind {
            CandidateArenaKind::Document => CandidateTree::Document(children()),
            CandidateArenaKind::Element {
                name,
                namespace,
                origin,
            } => CandidateTree::Element {
                name: *name,
                namespace: *namespace,
                origin: *origin,
                children: children(),
            },
            CandidateArenaKind::Text {
                interpreted,
                contributions,
            } => CandidateTree::Text {
                interpreted: interpreted.clone(),
                contributions: contributions.clone(),
            },
            CandidateArenaKind::UnusedStorageSlot => {
                panic!("an unreferenced storage slot is never part of the candidate tree")
            }
        }
    }
}

/// Normalizes one accepted lower-layer token into the candidate's shapes.
///
/// Pure and mutation-free, so a refusal here also precedes any mutation. Only
/// the tokenizer's own evidence is read: token kind, interpreted and raw
/// tag-name evidence, attribute and self-closing evidence, and exact anchors.
fn candidate_shape(token: &HtmlToken) -> Result<CandidateTokenShape<'_>, CandidateUnsupported> {
    match token {
        HtmlToken::Character(character) => Ok(CandidateTokenShape::Characters {
            range: span(character.source()),
            interpreted: character.interpreted(),
        }),
        HtmlToken::Tag(tag) => {
            let Some(name) = candidate_element_name(tag.name().interpreted()) else {
                return Err(CandidateUnsupported::OutsideModelledCandidateCells);
            };
            if !tag.attributes().is_empty() {
                return Err(CandidateUnsupported::TagWithAttributes);
            }
            if tag.self_closing_solidus().is_some() {
                return Err(CandidateUnsupported::SelfClosingTag);
            }
            let complete = span(tag.complete());
            match tag.kind() {
                HtmlTagKind::Start => Ok(CandidateTokenShape::StartTag {
                    name,
                    complete,
                    raw_name: span(tag.name().source()),
                }),
                HtmlTagKind::End => Ok(CandidateTokenShape::EndTag { name, complete }),
            }
        }
        HtmlToken::EndOfFile(end_of_file) => Ok(CandidateTokenShape::EndOfFile {
            at: end_of_file.source().range().start(),
        }),
    }
}

/// The candidate's fixed interpreted-name set. Deliberately closed: TC-S3
/// generalizes to no arbitrary ordinary element name.
fn candidate_element_name(interpreted: &str) -> Option<CandidateElementName> {
    match interpreted {
        "html" => Some(CandidateElementName::Html),
        "head" => Some(CandidateElementName::Head),
        "body" => Some(CandidateElementName::Body),
        "div" => Some(CandidateElementName::Div),
        _ => None,
    }
}

fn span(anchor: &crate::SourceAnchor) -> (usize, usize) {
    (anchor.range().start(), anchor.range().end())
}

fn candidate_trigger(token: &HtmlToken, index: usize) -> CandidateTrigger {
    match token {
        HtmlToken::Character(character) => CandidateTrigger::Authored {
            index,
            range: span(character.source()),
        },
        HtmlToken::Tag(tag) => CandidateTrigger::Authored {
            index,
            range: span(tag.complete()),
        },
        HtmlToken::EndOfFile(_) => CandidateTrigger::EndOfFile { index },
    }
}

/// Runs the independent candidate over one accepted lower-layer run.
///
/// Effective completion is authored here from the candidate theorem: lower-layer
/// incompleteness of any cause is never upgraded, no end-of-file semantics are
/// invented when the tokenizer emitted no end-of-file token, and a candidate
/// refusal is reported as the candidate's own typed evidence.
fn observe_with_layout(
    run: &HtmlTokenizerRunResult,
    layout: CandidateStorageLayout,
) -> CandidateObservation {
    let mut session = CandidateSession::new(layout);
    let mut tokens = Vec::new();
    let mut refusal = None;
    let mut stopped = false;

    for (index, token) in run.tokens().iter().enumerate() {
        let trigger = candidate_trigger(token, index);
        let shape = match candidate_shape(token) {
            Ok(shape) => shape,
            Err(capability) => {
                tokens.push(CandidateTokenRecord {
                    index,
                    mode_before: session.mode,
                    mode_after: session.mode,
                    dispatches: vec![CandidateDispatch {
                        evaluated_in: session.mode,
                        outcome: CandidateDispatchOutcome::Refused(capability),
                    }],
                    reprocesses: 0,
                    open_div_depth_after: session.open_div_depth(),
                    committed_prefix_end: session.committed_prefix_end,
                });
                refusal = Some((capability, trigger));
                break;
            }
        };
        let record = session.process(index, shape, trigger);
        let stop = record.stopped();
        if let Some(capability) = record.refusal() {
            refusal = Some((capability, trigger));
        }
        let refused = record.refusal().is_some();
        tokens.push(record);
        if refused {
            break;
        }
        if stop {
            stopped = true;
            break;
        }
    }

    let completion = match refusal {
        Some((capability, trigger)) => CandidateCompletion::IncompleteUnsupported {
            capability,
            trigger,
        },
        None if stopped
            && session.processed_tokens == run.tokens().len()
            && !run.is_incomplete() =>
        {
            CandidateCompletion::Complete
        }
        None => CandidateCompletion::IncompleteLowerLayer,
    };

    CandidateObservation {
        tree: session.tree(),
        diagnostics: session.diagnostics.clone(),
        tokens,
        identity_events: session.identity_events,
        checkpoint: CandidateCheckpoint {
            mode: session.mode,
            open_div_depth: session.open_div_depth(),
            committed_prefix_end: session.committed_prefix_end,
            completion,
        },
    }
}

fn observe(run: &HtmlTokenizerRunResult) -> CandidateObservation {
    observe_with_layout(run, CandidateStorageLayout::COMPACT)
}

fn generous_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn tokenize_text(
    text: &str,
    source_id: u64,
    limits: HtmlTokenizerLimits,
) -> HtmlTokenizerRunResult {
    let source = SourceText::new(SourceId::new(source_id), text.to_owned());
    tokenize(&source, limits)
}

/// Observes one canonical fixture through the accepted lower layer.
fn observe_fixture(id: &str) -> CandidateObservation {
    let candidate = fixture(id);
    observe(&tokenize_text(
        candidate.source_text(),
        1,
        generous_limits(),
    ))
}

// ---------------------------------------------------------------------------
// Authored candidate GOLD
// ---------------------------------------------------------------------------

fn document(children: Vec<CandidateTree>) -> CandidateTree {
    CandidateTree::Document(children)
}

fn synthesized(name: CandidateElementName, children: Vec<CandidateTree>) -> CandidateTree {
    CandidateTree::Element {
        name,
        namespace: CandidateNamespace::Html,
        origin: CandidateOrigin::Synthesized,
        children,
    }
}

fn authored(
    name: CandidateElementName,
    complete: (usize, usize),
    raw_name: (usize, usize),
    children: Vec<CandidateTree>,
) -> CandidateTree {
    CandidateTree::Element {
        name,
        namespace: CandidateNamespace::Html,
        origin: CandidateOrigin::Authored { complete, raw_name },
        children,
    }
}

fn div(
    complete: (usize, usize),
    raw_name: (usize, usize),
    children: Vec<CandidateTree>,
) -> CandidateTree {
    authored(CandidateElementName::Div, complete, raw_name, children)
}

fn text(interpreted: &str, contributions: &[(usize, usize)]) -> CandidateTree {
    CandidateTree::Text {
        interpreted: interpreted.to_owned(),
        contributions: contributions.to_vec(),
    }
}

/// Every DV fixture except DV13 opens with the same authored `body` start tag
/// at `[0,6)` with raw name `[1,5)`, which implies the `html` and `head` shell.
fn shell_authored_body(body_children: Vec<CandidateTree>) -> CandidateTree {
    document(vec![synthesized(
        CandidateElementName::Html,
        vec![
            synthesized(CandidateElementName::Head, vec![]),
            authored(CandidateElementName::Body, (0, 6), (1, 5), body_children),
        ],
    )])
}

/// DV13 reaches `in body` through the pinned "anything else" shell walk, so
/// its `body` element has no authored origin.
fn shell_synthesized_body(body_children: Vec<CandidateTree>) -> CandidateTree {
    document(vec![synthesized(
        CandidateElementName::Html,
        vec![
            synthesized(CandidateElementName::Head, vec![]),
            synthesized(CandidateElementName::Body, body_children),
        ],
    )])
}

fn consumed_in(mode: CandidateMode) -> CandidateDispatch {
    CandidateDispatch {
        evaluated_in: mode,
        outcome: CandidateDispatchOutcome::Consumed,
    }
}

fn ignored_in(mode: CandidateMode) -> CandidateDispatch {
    CandidateDispatch {
        evaluated_in: mode,
        outcome: CandidateDispatchOutcome::ConsumedAsIgnored,
    }
}

fn reprocessed_in(mode: CandidateMode) -> CandidateDispatch {
    CandidateDispatch {
        evaluated_in: mode,
        outcome: CandidateDispatchOutcome::Reprocessed,
    }
}

fn stopped_in(mode: CandidateMode) -> CandidateDispatch {
    CandidateDispatch {
        evaluated_in: mode,
        outcome: CandidateDispatchOutcome::Stopped,
    }
}

fn refused_in(mode: CandidateMode, capability: CandidateUnsupported) -> CandidateDispatch {
    CandidateDispatch {
        evaluated_in: mode,
        outcome: CandidateDispatchOutcome::Refused(capability),
    }
}

/// The shared token 0 record for an authored `<body>` start tag: it walks
/// `Initial` to `InBody` and is consumed by the `after head` rule.
fn body_start_tag_record() -> CandidateTokenRecord {
    CandidateTokenRecord {
        index: 0,
        mode_before: CandidateMode::Initial,
        mode_after: CandidateMode::InBody,
        dispatches: vec![
            reprocessed_in(CandidateMode::Initial),
            reprocessed_in(CandidateMode::BeforeHtml),
            reprocessed_in(CandidateMode::BeforeHead),
            reprocessed_in(CandidateMode::InHead),
            consumed_in(CandidateMode::AfterHead),
        ],
        reprocesses: 4,
        open_div_depth_after: 0,
        committed_prefix_end: 6,
    }
}

/// DV13's token 0 record: a non-whitespace character run walks the same shell
/// modes, but `after head` synthesizes the `body` element and reprocesses, so
/// the `in body` text rule finally consumes the token.
fn character_shell_walk_record(index: usize, committed_prefix_end: usize) -> CandidateTokenRecord {
    CandidateTokenRecord {
        index,
        mode_before: CandidateMode::Initial,
        mode_after: CandidateMode::InBody,
        dispatches: vec![
            reprocessed_in(CandidateMode::Initial),
            reprocessed_in(CandidateMode::BeforeHtml),
            reprocessed_in(CandidateMode::BeforeHead),
            reprocessed_in(CandidateMode::InHead),
            reprocessed_in(CandidateMode::AfterHead),
            consumed_in(CandidateMode::InBody),
        ],
        reprocesses: 5,
        open_div_depth_after: 0,
        committed_prefix_end,
    }
}

/// A token handled by a single rule dispatch.
fn simple_record(
    index: usize,
    mode_before: CandidateMode,
    mode_after: CandidateMode,
    dispatch: CandidateDispatch,
    open_div_depth_after: usize,
    committed_prefix_end: usize,
) -> CandidateTokenRecord {
    CandidateTokenRecord {
        index,
        mode_before,
        mode_after,
        dispatches: vec![dispatch],
        reprocesses: 0,
        open_div_depth_after,
        committed_prefix_end,
    }
}

/// A token consumed by the `in body` rules without changing the insertion mode.
fn in_body_record(
    index: usize,
    open_div_depth_after: usize,
    committed_prefix_end: usize,
) -> CandidateTokenRecord {
    simple_record(
        index,
        CandidateMode::InBody,
        CandidateMode::InBody,
        consumed_in(CandidateMode::InBody),
        open_div_depth_after,
        committed_prefix_end,
    )
}

fn missing_doctype(index: usize, range: (usize, usize)) -> CandidateDiagnostic {
    CandidateDiagnostic {
        code: CandidateDiagnosticCode::MissingDoctype,
        trigger: CandidateTrigger::Authored { index, range },
        recovery: CandidateRecovery::ContinuedInQuirksDocumentMode,
    }
}

/// Every DV fixture whose token 0 is the authored `<body>` start tag records
/// the predecessor missing-DOCTYPE diagnostic against that exact token.
fn missing_doctype_at_body() -> CandidateDiagnostic {
    missing_doctype(0, (0, 6))
}

fn unmatched_div_end_tag(index: usize, range: (usize, usize)) -> CandidateDiagnostic {
    CandidateDiagnostic {
        code: CandidateDiagnosticCode::UnmatchedDivEndTag,
        trigger: CandidateTrigger::Authored { index, range },
        recovery: CandidateRecovery::IgnoredToken,
    }
}

fn open_ordinary_element_at_end_of_file(index: usize) -> CandidateDiagnostic {
    CandidateDiagnostic {
        code: CandidateDiagnosticCode::OpenOrdinaryElementAtEndOfFile,
        trigger: CandidateTrigger::EndOfFile { index },
        recovery: CandidateRecovery::StoppedParsingWithOpenElements,
    }
}

fn complete_at(
    mode: CandidateMode,
    open_div_depth: usize,
    committed_prefix_end: usize,
) -> CandidateCheckpoint {
    CandidateCheckpoint {
        mode,
        open_div_depth,
        committed_prefix_end,
        completion: CandidateCompletion::Complete,
    }
}

fn refused_at(
    mode: CandidateMode,
    open_div_depth: usize,
    committed_prefix_end: usize,
    capability: CandidateUnsupported,
    trigger_index: usize,
    trigger_range: (usize, usize),
) -> CandidateCheckpoint {
    CandidateCheckpoint {
        mode,
        open_div_depth,
        committed_prefix_end,
        completion: CandidateCompletion::IncompleteUnsupported {
            capability,
            trigger: CandidateTrigger::Authored {
                index: trigger_index,
                range: trigger_range,
            },
        },
    }
}

/// The authored TC-S3 candidate GOLD for one fixture.
///
/// Hand-authored from the Issue #357 theorem and the pinned normative clauses.
/// It reads no production tree semantics and projects no production output.
fn candidate_gold(id: &str) -> CandidateObservation {
    match id {
        // Basic authored insertion, matching closure, complete result.
        "DV1" => CandidateObservation {
            tree: shell_authored_body(vec![div((6, 11), (7, 10), vec![])]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                in_body_record(1, 1, 11),
                in_body_record(2, 0, 17),
                simple_record(
                    3,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    0,
                    17,
                ),
            ],
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 0, 17),
        },
        // Interpreted name is `div` for any ASCII case; the raw authored
        // spelling `DiV` is retained exactly, and the mixed-case end tag is
        // closure evidence only.
        "DV2" => CandidateObservation {
            tree: shell_authored_body(vec![div((6, 11), (7, 10), vec![text("x", &[(11, 12)])])]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                in_body_record(1, 1, 11),
                in_body_record(2, 1, 12),
                in_body_record(3, 0, 18),
                simple_record(
                    4,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    0,
                    18,
                ),
            ],
            identity_events: 6,
            checkpoint: complete_at(CandidateMode::InBody, 0, 18),
        },
        // Nested stack S(0) -> S(1) -> S(2) -> S(1) -> S(0) with deterministic
        // creation order.
        "DV3" => CandidateObservation {
            tree: shell_authored_body(vec![div(
                (6, 11),
                (7, 10),
                vec![div((11, 16), (12, 15), vec![text("x", &[(16, 17)])])],
            )]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                in_body_record(1, 1, 11),
                in_body_record(2, 2, 16),
                in_body_record(3, 2, 17),
                in_body_record(4, 1, 23),
                in_body_record(5, 0, 29),
                simple_record(
                    6,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    0,
                    29,
                ),
            ],
            identity_events: 7,
            checkpoint: complete_at(CandidateMode::InBody, 0, 29),
        },
        // Sibling placement under `body` and two distinct semantic creation
        // events with distinct authored origins.
        "DV4" => CandidateObservation {
            tree: shell_authored_body(vec![
                div((6, 11), (7, 10), vec![]),
                div((17, 22), (18, 21), vec![]),
            ]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                in_body_record(1, 1, 11),
                in_body_record(2, 0, 17),
                in_body_record(3, 1, 22),
                in_body_record(4, 0, 28),
                simple_record(
                    5,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    0,
                    28,
                ),
            ],
            identity_events: 6,
            checkpoint: complete_at(CandidateMode::InBody, 0, 28),
        },
        // Stray `</div>` at k = 0: one diagnostic, ignored disposition, no
        // node, no identity, no stack or mode mutation.
        "DV5" => CandidateObservation {
            tree: shell_authored_body(vec![]),
            diagnostics: vec![missing_doctype_at_body(), unmatched_div_end_tag(1, (6, 12))],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    ignored_in(CandidateMode::InBody),
                    0,
                    12,
                ),
                simple_record(
                    2,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    0,
                    12,
                ),
            ],
            identity_events: 4,
            checkpoint: complete_at(CandidateMode::InBody, 0, 12),
        },
        // Open `div` at end of file: exactly one diagnostic, a normal stop,
        // an unchanged stack, and no fabricated closure origin or action.
        "DV6" => CandidateObservation {
            tree: shell_authored_body(vec![div((6, 11), (7, 10), vec![text("x", &[(11, 12)])])]),
            diagnostics: vec![
                missing_doctype_at_body(),
                open_ordinary_element_at_end_of_file(3),
            ],
            tokens: vec![
                body_start_tag_record(),
                in_body_record(1, 1, 11),
                in_body_record(2, 1, 12),
                simple_record(
                    3,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    1,
                    12,
                ),
            ],
            identity_events: 6,
            checkpoint: complete_at(CandidateMode::InBody, 1, 12),
        },
        // Parent-sensitive text placement: `c` does not coalesce with `a`
        // because the closed inner `div` separates them.
        "DV7" => CandidateObservation {
            tree: shell_authored_body(vec![div(
                (6, 11),
                (7, 10),
                vec![
                    text("a", &[(11, 12)]),
                    div((12, 17), (13, 16), vec![text("b", &[(17, 18)])]),
                    text("c", &[(24, 25)]),
                ],
            )]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                in_body_record(1, 1, 11),
                in_body_record(2, 1, 12),
                in_body_record(3, 2, 17),
                in_body_record(4, 2, 18),
                in_body_record(5, 1, 24),
                in_body_record(6, 1, 25),
                in_body_record(7, 0, 31),
                simple_record(
                    8,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    0,
                    31,
                ),
            ],
            identity_events: 9,
            checkpoint: complete_at(CandidateMode::InBody, 0, 31),
        },
        // An attributed `div` is refused before any candidate mutation.
        "DV8a" => CandidateObservation {
            tree: shell_authored_body(vec![]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    refused_in(
                        CandidateMode::InBody,
                        CandidateUnsupported::TagWithAttributes,
                    ),
                    0,
                    6,
                ),
            ],
            identity_events: 4,
            checkpoint: refused_at(
                CandidateMode::InBody,
                0,
                6,
                CandidateUnsupported::TagWithAttributes,
                1,
                (6, 16),
            ),
        },
        // A self-closing `div` is refused before any candidate mutation.
        "DV8b" => CandidateObservation {
            tree: shell_authored_body(vec![]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    refused_in(CandidateMode::InBody, CandidateUnsupported::SelfClosingTag),
                    0,
                    6,
                ),
            ],
            identity_events: 4,
            checkpoint: refused_at(
                CandidateMode::InBody,
                0,
                6,
                CandidateUnsupported::SelfClosingTag,
                1,
                (6, 12),
            ),
        },
        // `div` first evaluated in `after body` is outside the theorem; the
        // committed prefix stays at the end of the supported `</body>`.
        "DV9" => CandidateObservation {
            tree: shell_authored_body(vec![]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    0,
                    13,
                ),
                simple_record(
                    2,
                    CandidateMode::AfterBody,
                    CandidateMode::AfterBody,
                    refused_in(
                        CandidateMode::AfterBody,
                        CandidateUnsupported::DivTagOutsideInBody,
                    ),
                    0,
                    13,
                ),
            ],
            identity_events: 4,
            checkpoint: refused_at(
                CandidateMode::AfterBody,
                0,
                13,
                CandidateUnsupported::DivTagOutsideInBody,
                2,
                (13, 18),
            ),
        },
        // A shell interaction while a candidate `div` is open is refused
        // transactionally: stack, tree, identity, diagnostics, and committed
        // coverage are all unchanged for that token.
        "DV10" => CandidateObservation {
            tree: shell_authored_body(vec![div((6, 11), (7, 10), vec![])]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                in_body_record(1, 1, 11),
                simple_record(
                    2,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    refused_in(
                        CandidateMode::InBody,
                        CandidateUnsupported::ShellInteractionWithOpenDiv,
                    ),
                    1,
                    11,
                ),
            ],
            identity_events: 5,
            checkpoint: refused_at(
                CandidateMode::InBody,
                1,
                11,
                CandidateUnsupported::ShellInteractionWithOpenDiv,
                2,
                (11, 18),
            ),
        },
        // The historical `<body><p>` boundary stays unsupported at [6,9).
        "DV11" => CandidateObservation {
            tree: shell_authored_body(vec![]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    refused_in(
                        CandidateMode::InBody,
                        CandidateUnsupported::OutsideModelledCandidateCells,
                    ),
                    0,
                    6,
                ),
            ],
            identity_events: 4,
            checkpoint: refused_at(
                CandidateMode::InBody,
                0,
                6,
                CandidateUnsupported::OutsideModelledCandidateCells,
                1,
                (6, 9),
            ),
        },
        // The lower layer stops at the character reference. No end-of-file
        // token exists, so no end-of-file semantics and no completion upgrade
        // may be invented.
        "DV12" => CandidateObservation {
            tree: shell_authored_body(vec![div((6, 11), (7, 10), vec![])]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![body_start_tag_record(), in_body_record(1, 1, 11)],
            identity_events: 5,
            checkpoint: CandidateCheckpoint {
                mode: CandidateMode::InBody,
                open_div_depth: 1,
                committed_prefix_end: 11,
                completion: CandidateCompletion::IncompleteLowerLayer,
            },
        },
        // The candidate operates normally after the pinned "anything else"
        // shell walk synthesizes `html`, `head`, and `body`.
        "DV13" => CandidateObservation {
            tree: shell_synthesized_body(vec![text("x", &[(0, 1)]), div((1, 6), (2, 5), vec![])]),
            diagnostics: vec![missing_doctype(0, (0, 1))],
            tokens: vec![
                character_shell_walk_record(0, 1),
                in_body_record(1, 1, 6),
                in_body_record(2, 0, 12),
                simple_record(
                    3,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    0,
                    12,
                ),
            ],
            identity_events: 6,
            checkpoint: complete_at(CandidateMode::InBody, 0, 12),
        },
        // Once the candidate div stack returns to k = 0, predecessor
        // body-close behavior resumes and the run ends in `after body`.
        "DV14" => CandidateObservation {
            tree: shell_authored_body(vec![div((6, 11), (7, 10), vec![])]),
            diagnostics: vec![missing_doctype_at_body()],
            tokens: vec![
                body_start_tag_record(),
                in_body_record(1, 1, 11),
                in_body_record(2, 0, 17),
                simple_record(
                    3,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    0,
                    24,
                ),
                simple_record(
                    4,
                    CandidateMode::AfterBody,
                    CandidateMode::AfterBody,
                    stopped_in(CandidateMode::AfterBody),
                    0,
                    24,
                ),
            ],
            // Document + html + head + body + div. The matching `</div>` and
            // the supported `</body>` admit no identity.
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::AfterBody, 0, 24),
        },
        other => panic!("no authored candidate GOLD for {other}"),
    }
}

// ---------------------------------------------------------------------------
// Observation helpers
// ---------------------------------------------------------------------------

fn collect_text_nodes(tree: &CandidateTree, into: &mut Vec<(String, Vec<(usize, usize)>)>) {
    match tree {
        CandidateTree::Document(children) | CandidateTree::Element { children, .. } => {
            for child in children {
                collect_text_nodes(child, into);
            }
        }
        CandidateTree::Text {
            interpreted,
            contributions,
        } => into.push((interpreted.clone(), contributions.clone())),
    }
}

fn text_nodes(tree: &CandidateTree) -> Vec<(String, Vec<(usize, usize)>)> {
    let mut collected = Vec::new();
    collect_text_nodes(tree, &mut collected);
    collected
}

fn collect_elements(
    tree: &CandidateTree,
    into: &mut Vec<(CandidateElementName, CandidateNamespace, CandidateOrigin)>,
) {
    match tree {
        CandidateTree::Document(children) => {
            for child in children {
                collect_elements(child, into);
            }
        }
        CandidateTree::Element {
            name,
            namespace,
            origin,
            children,
        } => {
            into.push((*name, *namespace, *origin));
            for child in children {
                collect_elements(child, into);
            }
        }
        CandidateTree::Text { .. } => {}
    }
}

fn elements(
    tree: &CandidateTree,
) -> Vec<(CandidateElementName, CandidateNamespace, CandidateOrigin)> {
    let mut collected = Vec::new();
    collect_elements(tree, &mut collected);
    collected
}

fn element_origins(tree: &CandidateTree) -> Vec<CandidateOrigin> {
    elements(tree)
        .into_iter()
        .map(|(_, _, origin)| origin)
        .collect()
}

fn div_count(tree: &CandidateTree) -> usize {
    elements(tree)
        .into_iter()
        .filter(|(name, _, _)| *name == CandidateElementName::Div)
        .count()
}

fn node_count(tree: &CandidateTree) -> usize {
    match tree {
        CandidateTree::Document(children) | CandidateTree::Element { children, .. } => {
            1 + children.iter().map(node_count).sum::<usize>()
        }
        CandidateTree::Text { .. } => 1,
    }
}

fn all_contributions(tree: &CandidateTree) -> Vec<(usize, usize)> {
    text_nodes(tree)
        .into_iter()
        .flat_map(|(_, contributions)| contributions)
        .collect()
}

/// The candidate open-`div` depth immediately before the token at `position`.
fn depth_before(observed: &CandidateObservation, position: usize) -> usize {
    if position == 0 {
        0
    } else {
        observed.tokens[position - 1].open_div_depth_after
    }
}

/// The committed prefix immediately before the token at `position`.
fn committed_before(observed: &CandidateObservation, position: usize) -> usize {
    if position == 0 {
        0
    } else {
        observed.tokens[position - 1].committed_prefix_end
    }
}

/// The lower-layer token at a record's index, for cross-checking the candidate
/// record against the evidence it was derived from.
fn token_at(run: &HtmlTokenizerRunResult, index: usize) -> &HtmlToken {
    run.tokens().get(index).expect("an emitted token")
}

fn run_for(id: &str) -> HtmlTokenizerRunResult {
    tokenize_text(fixture(id).source_text(), 1, generous_limits())
}

// ---------------------------------------------------------------------------
// 1. Canonical byte authority
// ---------------------------------------------------------------------------

#[test]
fn canonical_fixture_bytes_match_the_issue_357_authority() {
    assert_eq!(
        CANDIDATE_FIXTURES.len(),
        CANDIDATE_IDS.len(),
        "the candidate fixture set is exactly DV1-DV14"
    );
    for (candidate, id) in CANDIDATE_FIXTURES.iter().zip(CANDIDATE_IDS) {
        assert_eq!(candidate.id, id, "fixture order matches the DV identifiers");
        assert_eq!(
            candidate.bytes.len(),
            candidate.length,
            "{}: exact canonical byte length",
            candidate.id
        );
        assert_eq!(
            candidate.sha256.len(),
            64,
            "{}: canonical digest metadata is a full SHA-256 hex digest",
            candidate.id
        );
        assert!(
            candidate
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
            "{}: canonical digest metadata is lowercase hex",
            candidate.id
        );
        // The fixture must be exactly the bytes, not a lossy normalization.
        let text = candidate.source_text();
        assert_eq!(text.as_bytes(), candidate.bytes, "{}", candidate.id);
        assert_eq!(text.len(), candidate.length, "{}", candidate.id);

        // Every byte range the theorem depends on is verified in place, so no
        // expected span is ever reconstructed from prose or rendered markup.
        for ((start, end), expected) in candidate.required_ranges {
            assert!(
                *start <= *end && *end <= candidate.length,
                "{}: required range [{start},{end}) is inside the fixture",
                candidate.id
            );
            assert_eq!(
                &candidate.bytes[*start..*end],
                *expected,
                "{}: required byte range [{start},{end})",
                candidate.id
            );
        }
    }

    // Every digest is distinct, so no two DV fixtures are the same bytes.
    for (position, candidate) in CANDIDATE_FIXTURES.iter().enumerate() {
        for other in &CANDIDATE_FIXTURES[position + 1..] {
            assert_ne!(
                candidate.bytes, other.bytes,
                "{} and {} must be different canonical sources",
                candidate.id, other.id
            );
            assert_ne!(
                candidate.sha256, other.sha256,
                "{} and {} must have different canonical digests",
                candidate.id, other.id
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Lower-layer evidence shape
// ---------------------------------------------------------------------------

/// One expected emitted token, in the exact evidence the candidate reads.
#[derive(Debug, PartialEq, Eq)]
enum ExpectedToken {
    Start {
        complete: (usize, usize),
        interpreted: String,
        raw_name: (usize, usize),
        attributes: usize,
        self_closing: Option<(usize, usize)>,
    },
    End {
        complete: (usize, usize),
        interpreted: String,
        raw_name: (usize, usize),
    },
    Characters {
        range: (usize, usize),
        interpreted: String,
    },
    EndOfFile {
        at: usize,
    },
}

fn observed_token(token: &HtmlToken) -> ExpectedToken {
    match token {
        HtmlToken::Character(character) => ExpectedToken::Characters {
            range: span(character.source()),
            interpreted: character.interpreted().to_owned(),
        },
        HtmlToken::Tag(tag) => {
            let complete = span(tag.complete());
            let interpreted = tag.name().interpreted().to_owned();
            let raw_name = span(tag.name().source());
            match tag.kind() {
                HtmlTagKind::Start => ExpectedToken::Start {
                    complete,
                    interpreted,
                    raw_name,
                    attributes: tag.attributes().len(),
                    self_closing: tag.self_closing_solidus().map(span),
                },
                HtmlTagKind::End => ExpectedToken::End {
                    complete,
                    interpreted,
                    raw_name,
                },
            }
        }
        HtmlToken::EndOfFile(end_of_file) => ExpectedToken::EndOfFile {
            at: end_of_file.source().range().start(),
        },
    }
}

fn start(complete: (usize, usize), interpreted: &str, raw_name: (usize, usize)) -> ExpectedToken {
    ExpectedToken::Start {
        complete,
        interpreted: interpreted.to_owned(),
        raw_name,
        attributes: 0,
        self_closing: None,
    }
}

fn end(complete: (usize, usize), interpreted: &str, raw_name: (usize, usize)) -> ExpectedToken {
    ExpectedToken::End {
        complete,
        interpreted: interpreted.to_owned(),
        raw_name,
    }
}

fn characters(range: (usize, usize), interpreted: &str) -> ExpectedToken {
    ExpectedToken::Characters {
        range,
        interpreted: interpreted.to_owned(),
    }
}

fn end_of_file(at: usize) -> ExpectedToken {
    ExpectedToken::EndOfFile { at }
}

/// The candidate theorem assumes a specific emitted evidence shape — including
/// the interpreted lowercase names, the exact raw-name anchors, and the
/// attribute and self-closing evidence DV8a/DV8b turn on. This pins that shape
/// against the accepted lower layer instead of assuming it.
#[test]
fn tokenizer_emits_the_evidence_shape_the_candidate_assumes() {
    let expected: [(&str, Vec<ExpectedToken>); 15] = [
        (
            "DV1",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                end((11, 17), "div", (13, 16)),
                end_of_file(17),
            ],
        ),
        (
            "DV2",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                characters((11, 12), "x"),
                end((12, 18), "div", (14, 17)),
                end_of_file(18),
            ],
        ),
        (
            "DV3",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                start((11, 16), "div", (12, 15)),
                characters((16, 17), "x"),
                end((17, 23), "div", (19, 22)),
                end((23, 29), "div", (25, 28)),
                end_of_file(29),
            ],
        ),
        (
            "DV4",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                end((11, 17), "div", (13, 16)),
                start((17, 22), "div", (18, 21)),
                end((22, 28), "div", (24, 27)),
                end_of_file(28),
            ],
        ),
        (
            "DV5",
            vec![
                start((0, 6), "body", (1, 5)),
                end((6, 12), "div", (8, 11)),
                end_of_file(12),
            ],
        ),
        (
            "DV6",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                characters((11, 12), "x"),
                end_of_file(12),
            ],
        ),
        (
            "DV7",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                characters((11, 12), "a"),
                start((12, 17), "div", (13, 16)),
                characters((17, 18), "b"),
                end((18, 24), "div", (20, 23)),
                characters((24, 25), "c"),
                end((25, 31), "div", (27, 30)),
                end_of_file(31),
            ],
        ),
        (
            "DV8a",
            vec![
                start((0, 6), "body", (1, 5)),
                ExpectedToken::Start {
                    complete: (6, 16),
                    interpreted: "div".to_owned(),
                    raw_name: (7, 10),
                    attributes: 1,
                    self_closing: None,
                },
                end_of_file(16),
            ],
        ),
        (
            "DV8b",
            vec![
                start((0, 6), "body", (1, 5)),
                ExpectedToken::Start {
                    complete: (6, 12),
                    interpreted: "div".to_owned(),
                    raw_name: (7, 10),
                    attributes: 0,
                    self_closing: Some((10, 11)),
                },
                end_of_file(12),
            ],
        ),
        (
            "DV9",
            vec![
                start((0, 6), "body", (1, 5)),
                end((6, 13), "body", (8, 12)),
                start((13, 18), "div", (14, 17)),
                end_of_file(18),
            ],
        ),
        (
            "DV10",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                end((11, 18), "body", (13, 17)),
                end_of_file(18),
            ],
        ),
        (
            "DV11",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 9), "p", (7, 8)),
                end_of_file(9),
            ],
        ),
        (
            // The lower layer stops at the character reference and emits no
            // end-of-file token at all.
            "DV12",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
            ],
        ),
        (
            "DV13",
            vec![
                characters((0, 1), "x"),
                start((1, 6), "div", (2, 5)),
                end((6, 12), "div", (8, 11)),
                end_of_file(12),
            ],
        ),
        (
            "DV14",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                end((11, 17), "div", (13, 16)),
                end((17, 24), "body", (19, 23)),
                end_of_file(24),
            ],
        ),
    ];

    for (id, tokens) in expected {
        let run = run_for(id);
        let observed: Vec<ExpectedToken> = run.tokens().iter().map(observed_token).collect();
        assert_eq!(observed, tokens, "{id}: emitted lower-layer evidence");
        assert_eq!(
            run.is_incomplete(),
            id == "DV12",
            "{id}: only DV12 relies on lower-layer incompleteness"
        );
    }
}

// ---------------------------------------------------------------------------
// 3. DV1-DV14 against the authored candidate GOLD
// ---------------------------------------------------------------------------

#[test]
fn dv_cases_match_the_independent_candidate_gold() {
    for id in CANDIDATE_IDS {
        assert_eq!(
            observe_fixture(id),
            candidate_gold(id),
            "{id}: independent candidate theorem"
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Candidate invariant and pinned projections
// ---------------------------------------------------------------------------

/// The candidate-local projections of the pinned element lists are exact for
/// the fixed candidate name set, and the fixed parse configuration is the one
/// Issue #357 states.
#[test]
fn pinned_element_list_projections_are_exact() {
    let names = [
        CandidateElementName::Html,
        CandidateElementName::Head,
        CandidateElementName::Body,
        CandidateElementName::Div,
    ];

    for name in names {
        // No candidate name is `dd`, `dt`, `li`, `optgroup`, `option`, `p`,
        // `rb`, `rp`, `rt`, or `rtc`.
        assert!(
            !name.is_implied_end_element(),
            "{name:?} must not be an implied-end element"
        );
        // Of the pinned "in scope" element-type list only `html` is a
        // candidate name.
        assert_eq!(
            name.is_scope_boundary(),
            name == CandidateElementName::Html,
            "{name:?} scope-boundary projection"
        );
        // The pinned `in body` end-of-file permitted list contains `body` and
        // `html`; `div` and `head` are outside it.
        assert_eq!(
            name.is_permitted_open_element_at_end_of_file(),
            matches!(
                name,
                CandidateElementName::Body | CandidateElementName::Html
            ),
            "{name:?} end-of-file permitted projection"
        );
        assert_eq!(
            name.is_shell(),
            name != CandidateElementName::Div,
            "{name:?} shell projection"
        );
    }

    assert_eq!(
        CandidateConfiguration::FIXED,
        CandidateConfiguration {
            scripting: CandidateScripting::Disabled,
            fragment_parse: false,
            template_state: false,
            table_state: false,
            foreign_content_state: false,
            reentrant_state: false,
        },
        "the candidate parse configuration is the Issue #357 invariant"
    );

    // The candidate whitespace set is exactly the five HTML whitespace
    // characters. Nothing else in the Basic Latin and Latin-1 range joins it.
    for code in 0u32..=0xff {
        let character = char::from_u32(code).expect("scalar value");
        assert_eq!(
            is_candidate_html_whitespace(character),
            matches!(code, 0x09 | 0x0a | 0x0c | 0x0d | 0x20),
            "U+{code:04X}"
        );
    }
}

/// The candidate's fixed interpreted-name set admits exactly four names, so
/// the model cannot silently generalize to arbitrary ordinary elements.
#[test]
fn the_candidate_name_set_stays_closed() {
    for (interpreted, expected) in [
        ("html", Some(CandidateElementName::Html)),
        ("head", Some(CandidateElementName::Head)),
        ("body", Some(CandidateElementName::Body)),
        ("div", Some(CandidateElementName::Div)),
        ("p", None),
        ("span", None),
        ("section", None),
        ("divx", None),
        ("di", None),
        ("dl", None),
        ("DIV", None),
    ] {
        assert_eq!(
            candidate_element_name(interpreted),
            expected,
            "{interpreted:?} candidate name admission"
        );
    }
}

// ---------------------------------------------------------------------------
// 5. The selected start-tag theorem: S(k) -> S(k+1)
// ---------------------------------------------------------------------------

#[test]
fn a_selected_div_start_tag_moves_s_k_to_s_k_plus_one() {
    for id in CANDIDATE_IDS {
        let observed = observe_fixture(id);
        let run = run_for(id);
        for (position, record) in observed.tokens.iter().enumerate() {
            let HtmlToken::Tag(tag) = token_at(&run, record.index) else {
                continue;
            };
            if tag.kind() != HtmlTagKind::Start
                || tag.name().interpreted() != "div"
                || record.refusal().is_some()
            {
                continue;
            }

            let before = depth_before(&observed, position);
            assert_eq!(
                record.open_div_depth_after,
                before + 1,
                "{id}: token {} must move S({before}) to S({})",
                record.index,
                before + 1
            );
            assert_eq!(
                record.dispatches,
                vec![consumed_in(CandidateMode::InBody)],
                "{id}: an accepted div start tag is one consuming in-body dispatch"
            );
            assert_eq!(
                record.reprocesses, 0,
                "{id}: an accepted div start tag adds no same-token redispatch"
            );
            assert_eq!(
                record.mode_before,
                CandidateMode::InBody,
                "{id}: div is admitted only in actual in body"
            );
            assert_eq!(
                record.mode_after, record.mode_before,
                "{id}: an accepted div start tag changes no insertion mode"
            );
            assert_eq!(
                record.committed_prefix_end,
                span(tag.complete()).1,
                "{id}: the token is consumed through its own complete anchor"
            );
        }

        // Exactly one constructed identity per accepted div start tag, and the
        // exact authored anchors are retained.
        let accepted_div_starts: Vec<&HtmlTagToken> = observed
            .tokens
            .iter()
            .filter(|record| record.refusal().is_none())
            .filter_map(|record| match token_at(&run, record.index) {
                HtmlToken::Tag(tag)
                    if tag.kind() == HtmlTagKind::Start && tag.name().interpreted() == "div" =>
                {
                    Some(tag)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            div_count(&observed.tree),
            accepted_div_starts.len(),
            "{id}: one candidate div node per accepted div start tag"
        );
        let authored_div_origins: Vec<CandidateOrigin> = elements(&observed.tree)
            .into_iter()
            .filter(|(name, _, _)| *name == CandidateElementName::Div)
            .map(|(_, _, origin)| origin)
            .collect();
        let expected_origins: Vec<CandidateOrigin> = accepted_div_starts
            .iter()
            .map(|tag| CandidateOrigin::Authored {
                complete: span(tag.complete()),
                raw_name: span(tag.name().source()),
            })
            .collect();
        assert_eq!(
            authored_div_origins, expected_origins,
            "{id}: each candidate div retains its own exact complete and raw-name anchors"
        );
    }
}

// ---------------------------------------------------------------------------
// 6. The selected end-tag theorem: S(k) -> S(k-1)
// ---------------------------------------------------------------------------

#[test]
fn a_matching_div_end_tag_moves_s_k_to_s_k_minus_one_and_admits_no_identity() {
    for id in CANDIDATE_IDS {
        let observed = observe_fixture(id);
        let run = run_for(id);
        for (position, record) in observed.tokens.iter().enumerate() {
            let HtmlToken::Tag(tag) = token_at(&run, record.index) else {
                continue;
            };
            if tag.kind() != HtmlTagKind::End || tag.name().interpreted() != "div" {
                continue;
            }
            let before = depth_before(&observed, position);
            if before == 0 {
                continue;
            }

            assert_eq!(
                record.open_div_depth_after,
                before - 1,
                "{id}: token {} must move S({before}) to S({})",
                record.index,
                before - 1
            );
            assert_eq!(
                record.dispatches,
                vec![consumed_in(CandidateMode::InBody)],
                "{id}: a matching div end tag is one consuming in-body dispatch"
            );
            assert_eq!(record.reprocesses, 0, "{id}: no same-token redispatch");
            assert_eq!(
                record.mode_after, record.mode_before,
                "{id}: no mode change"
            );

            // The end tag is trigger/closure evidence only.
            let closure = span(tag.complete());
            assert!(
                !element_origins(&observed.tree).contains(&CandidateOrigin::Authored {
                    complete: closure,
                    raw_name: span(tag.name().source()),
                }),
                "{id}: an end tag is never an element's authored origin"
            );
            assert!(
                !all_contributions(&observed.tree).contains(&closure),
                "{id}: an end tag contributes no text"
            );
            assert!(
                !observed
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.trigger
                        == CandidateTrigger::Authored {
                            index: record.index,
                            range: closure
                        }),
                "{id}: a matching div end tag records no diagnostic"
            );
        }

        // Identity is admitted for constructed nodes only, so a matching end
        // tag can never introduce an ordinal gap.
        assert_eq!(
            observed.identity_events,
            node_count(&observed.tree),
            "{id}: one creation event per constructed node, none for closures"
        );
    }
}

// ---------------------------------------------------------------------------
// 7. The stray end-tag theorem at k = 0
// ---------------------------------------------------------------------------

#[test]
fn a_stray_div_end_tag_is_diagnosed_ignored_and_mutates_nothing() {
    let observed = observe_fixture("DV5");
    let record = &observed.tokens[1];

    assert_eq!(
        record.dispatches,
        vec![ignored_in(CandidateMode::InBody)],
        "one dispatch with an ignored-token disposition"
    );
    assert_eq!(record.reprocesses, 0, "no same-token redispatch");
    assert_eq!(record.mode_before, record.mode_after, "no mode mutation");
    assert_eq!(
        record.open_div_depth_after,
        depth_before(&observed, 1),
        "no stack mutation"
    );
    assert_eq!(
        record.committed_prefix_end, 12,
        "the token is still consumed through its own complete anchor"
    );

    let strays: Vec<&CandidateDiagnostic> = observed
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::UnmatchedDivEndTag)
        .collect();
    assert_eq!(strays.len(), 1, "exactly one candidate parse diagnostic");
    assert_eq!(
        strays[0].trigger,
        CandidateTrigger::Authored {
            index: 1,
            range: (6, 12)
        },
        "the diagnostic names the emitted end tag"
    );
    assert_eq!(strays[0].recovery, CandidateRecovery::IgnoredToken);

    assert_eq!(div_count(&observed.tree), 0, "no candidate div node");
    assert_eq!(text_nodes(&observed.tree), vec![], "no text node");
    assert_eq!(
        observed.identity_events, 4,
        "no identity is admitted for the ignored token"
    );
    assert_eq!(
        observed.identity_events,
        node_count(&observed.tree),
        "no ordinal gap"
    );
    assert!(
        !element_origins(&observed.tree).contains(&CandidateOrigin::Authored {
            complete: (6, 12),
            raw_name: (8, 11)
        }),
        "the stray end tag is no element's source origin"
    );
    assert_eq!(
        observed.checkpoint,
        complete_at(CandidateMode::InBody, 0, 12),
        "the run still completes"
    );
}

// ---------------------------------------------------------------------------
// 8. Character runs with an open candidate div
// ---------------------------------------------------------------------------

/// The candidate `div` becomes the insertion parent, contributions stay exact
/// and ordered, coalescing admits no identity, and no merged source range is
/// synthesized.
#[test]
fn character_runs_under_an_open_div_keep_exact_ordered_contributions() {
    // DV2: one run inside one div.
    let dv2 = observe_fixture("DV2");
    assert_eq!(
        text_nodes(&dv2.tree),
        vec![("x".to_owned(), vec![(11, 12)])],
        "DV2: the run is placed under the open candidate div"
    );

    // DV7: parent-sensitive placement across a nested div, where `c` must not
    // coalesce with `a` because the closed inner div separates them.
    let dv7 = observe_fixture("DV7");
    assert_eq!(
        text_nodes(&dv7.tree),
        vec![
            ("a".to_owned(), vec![(11, 12)]),
            ("b".to_owned(), vec![(17, 18)]),
            ("c".to_owned(), vec![(24, 25)]),
        ],
        "DV7: three separate text nodes with their own exact contributions"
    );
    assert_eq!(
        all_contributions(&dv7.tree),
        vec![(11, 12), (17, 18), (24, 25)],
        "DV7: ordered contributions, each the emitted token's own anchor"
    );
    assert_eq!(
        dv7.identity_events,
        node_count(&dv7.tree),
        "DV7: one creation event per node"
    );

    // Adjacent runs in the same parent coalesce into one node, appending an
    // ordered contribution and admitting no new identity.
    let run = tokenize_text("<body><div>a</div>bc", 1, generous_limits());
    assert!(!run.is_incomplete(), "the lower layer completes");
    let coalescing = observe(&run);
    assert_eq!(
        text_nodes(&coalescing.tree),
        vec![
            ("a".to_owned(), vec![(11, 12)]),
            ("bc".to_owned(), vec![(18, 20)]),
        ],
        "one aggregate emitted run is one contribution, never a split"
    );
    assert_eq!(
        coalescing.identity_events,
        node_count(&coalescing.tree),
        "appending admits no new identity"
    );

    // Two emitted runs in the same parent, separated only by an ignored stray
    // end tag, coalesce into one node with two retained contributions.
    let stray = tokenize_text("<body>a</div>b", 1, generous_limits());
    assert!(!stray.is_incomplete(), "the lower layer completes");
    let coalesced = observe(&stray);
    assert_eq!(
        text_nodes(&coalesced.tree),
        vec![("ab".to_owned(), vec![(6, 7), (13, 14)])],
        "individually retained contributions, never a reconstructed span"
    );
    assert_eq!(
        coalesced.identity_events,
        node_count(&coalesced.tree),
        "coalescing consumes no identity event"
    );
}

// ---------------------------------------------------------------------------
// 9. The end-of-file theorem with an open candidate div
// ---------------------------------------------------------------------------

#[test]
fn end_of_file_with_an_open_div_diagnoses_once_and_fabricates_no_closure() {
    let observed = observe_fixture("DV6");
    let record = &observed.tokens[3];

    assert_eq!(
        record.dispatches,
        vec![stopped_in(CandidateMode::InBody)],
        "one dispatch that stops parsing normally"
    );
    assert_eq!(
        record.open_div_depth_after, 1,
        "the candidate stack is left unchanged"
    );
    assert_eq!(
        record.committed_prefix_end,
        committed_before(&observed, 3),
        "end of file advances committed coverage past nothing"
    );

    let eof_diagnostics: Vec<&CandidateDiagnostic> = observed
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == CandidateDiagnosticCode::OpenOrdinaryElementAtEndOfFile
        })
        .collect();
    assert_eq!(
        eof_diagnostics.len(),
        1,
        "exactly one open-element end-of-file diagnostic"
    );
    assert_eq!(
        eof_diagnostics[0].trigger,
        CandidateTrigger::EndOfFile { index: 3 },
        "end of file receives no authored source anchor"
    );
    assert_eq!(
        eof_diagnostics[0].recovery,
        CandidateRecovery::StoppedParsingWithOpenElements
    );

    // No fabricated end tag, closure anchor, or close action anywhere.
    assert_eq!(
        observed.tree,
        shell_authored_body(vec![div((6, 11), (7, 10), vec![text("x", &[(11, 12)])])]),
        "the tree is unchanged by end of file"
    );
    assert_eq!(
        observed.identity_events,
        node_count(&observed.tree),
        "end of file admits no identity"
    );
    assert!(
        !all_contributions(&observed.tree)
            .iter()
            .any(|(start, end)| *start >= 12 || *end > 12),
        "no source evidence is invented past the last emitted token"
    );

    // Diagnostics and completion stay orthogonal: this run is Complete.
    assert_eq!(
        observed.checkpoint.completion,
        CandidateCompletion::Complete,
        "a parse diagnostic does not imply incomplete parsing"
    );

    // At k = 0 the same end-of-file cell records no diagnostic at all, because
    // `body` and `html` are inside the pinned permitted list.
    for id in ["DV1", "DV5", "DV14"] {
        let closed = observe_fixture(id);
        assert_eq!(
            closed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code
                    == CandidateDiagnosticCode::OpenOrdinaryElementAtEndOfFile)
                .count(),
            0,
            "{id}: no end-of-file diagnostic when the div stack is empty"
        );
    }
}

// ---------------------------------------------------------------------------
// 10. Refusal before mutation
// ---------------------------------------------------------------------------

/// Every DV refusal is terminal for its token, precedes any mutation by that
/// cell, and leaves the committed prefix at the last supported token's end.
#[test]
fn dv_refusals_are_transactional_and_precede_mutation() {
    for (id, capability, trigger_index, trigger_range, committed, depth) in [
        (
            "DV8a",
            CandidateUnsupported::TagWithAttributes,
            1,
            (6, 16),
            6,
            0,
        ),
        (
            "DV8b",
            CandidateUnsupported::SelfClosingTag,
            1,
            (6, 12),
            6,
            0,
        ),
        (
            "DV9",
            CandidateUnsupported::DivTagOutsideInBody,
            2,
            (13, 18),
            13,
            0,
        ),
        (
            "DV10",
            CandidateUnsupported::ShellInteractionWithOpenDiv,
            2,
            (11, 18),
            11,
            1,
        ),
        (
            "DV11",
            CandidateUnsupported::OutsideModelledCandidateCells,
            1,
            (6, 9),
            6,
            0,
        ),
    ] {
        let observed = observe_fixture(id);
        let position = observed.tokens.len() - 1;
        let record = &observed.tokens[position];

        assert_eq!(record.index, trigger_index, "{id}: the refused token index");
        assert_eq!(
            record.refusal(),
            Some(capability),
            "{id}: the exact typed refusal"
        );
        assert_eq!(
            record.dispatches.len(),
            1,
            "{id}: the refusal is the token's first and only dispatch"
        );
        assert_eq!(
            record.mode_before, record.mode_after,
            "{id}: no mode change"
        );
        assert_eq!(record.reprocesses, 0, "{id}: no same-token redispatch");
        assert_eq!(
            record.open_div_depth_after,
            depth_before(&observed, position),
            "{id}: no stack mutation"
        );
        assert_eq!(
            record.committed_prefix_end,
            committed_before(&observed, position),
            "{id}: a refused token commits no coverage"
        );
        assert_eq!(
            observed.checkpoint,
            refused_at(
                record.mode_after,
                depth,
                committed,
                capability,
                trigger_index,
                trigger_range,
            ),
            "{id}: the terminal checkpoint"
        );

        // Nothing anywhere records the refused token as origin, contribution,
        // or diagnostic trigger.
        let refused_trigger = CandidateTrigger::Authored {
            index: trigger_index,
            range: trigger_range,
        };
        assert!(
            !observed
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.trigger == refused_trigger),
            "{id}: the refused token records no diagnostic"
        );
        assert!(
            !all_contributions(&observed.tree).contains(&trigger_range),
            "{id}: the refused token contributes no text"
        );
        assert!(
            !element_origins(&observed.tree)
                .iter()
                .any(|origin| matches!(
                    origin,
                    CandidateOrigin::Authored { complete, .. } if *complete == trigger_range
                )),
            "{id}: the refused token is no element's origin"
        );
        assert_eq!(
            observed.identity_events,
            node_count(&observed.tree),
            "{id}: no identity gap after a refusal"
        );
    }
}

// ---------------------------------------------------------------------------
// 11. Provenance domains stay distinct
// ---------------------------------------------------------------------------

/// Authored node origin, end-tag closure evidence, diagnostic trigger, token
/// disposition, constructed identity, and final placement are six different
/// things, and the candidate must not substitute one for another.
#[test]
fn provenance_domains_are_never_substituted_for_one_another() {
    for id in CANDIDATE_IDS {
        let observed = observe_fixture(id);
        let run = run_for(id);

        let end_tag_ranges: Vec<(usize, usize)> = run
            .tokens()
            .iter()
            .filter_map(|token| match token {
                HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::End => Some(span(tag.complete())),
                _ => None,
            })
            .collect();
        let character_ranges: Vec<(usize, usize)> = run
            .tokens()
            .iter()
            .filter_map(|token| match token {
                HtmlToken::Character(character) => Some(span(character.source())),
                _ => None,
            })
            .collect();

        for (name, namespace, origin) in elements(&observed.tree) {
            assert_eq!(
                namespace,
                CandidateNamespace::Html,
                "{id}: every candidate element is HTML-namespaced"
            );
            let CandidateOrigin::Authored { complete, raw_name } = origin else {
                assert!(
                    name != CandidateElementName::Div,
                    "{id}: a candidate div always has an authored origin"
                );
                continue;
            };
            assert!(
                !end_tag_ranges.contains(&complete),
                "{id}: an end tag never becomes a node origin"
            );
            assert!(
                !character_ranges.contains(&complete),
                "{id}: a character run never becomes an element origin"
            );
            assert!(
                complete.0 < raw_name.0 && raw_name.1 < complete.1,
                "{id}: the raw-name anchor is strictly inside the complete start-tag anchor"
            );
            assert!(
                raw_name.0 < raw_name.1,
                "{id}: the raw-name anchor is never empty"
            );
        }

        // No element claims a synthesized origin with a fabricated span, and
        // end of file never receives an authored anchor.
        for diagnostic in &observed.diagnostics {
            match diagnostic.trigger {
                CandidateTrigger::Authored { index, range } => {
                    let token_range = match token_at(&run, index) {
                        HtmlToken::Character(character) => span(character.source()),
                        HtmlToken::Tag(tag) => span(tag.complete()),
                        HtmlToken::EndOfFile(_) => {
                            panic!("{id}: an end-of-file token cannot carry an authored trigger")
                        }
                    };
                    assert_eq!(
                        range, token_range,
                        "{id}: a diagnostic trigger is the emitted token's own exact anchor"
                    );
                }
                CandidateTrigger::EndOfFile { index } => {
                    assert!(
                        matches!(token_at(&run, index), HtmlToken::EndOfFile(_)),
                        "{id}: an end-of-file trigger names the end-of-file token"
                    );
                }
            }
        }

        // Every retained text contribution is exactly one emitted character
        // token's anchor: no rescanning, no merged range, no invented span.
        for contribution in all_contributions(&observed.tree) {
            assert!(
                character_ranges.contains(&contribution),
                "{id}: every contribution is an emitted character token's own anchor"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 12. Identity admission
// ---------------------------------------------------------------------------

#[test]
fn identity_admission_is_exact_and_gap_free() {
    for id in CANDIDATE_IDS {
        let observed = observe_fixture(id);
        assert_eq!(
            observed.identity_events,
            node_count(&observed.tree),
            "{id}: exactly one identity event per constructed node, and no gap"
        );
    }

    // Identity is admitted for constructed nodes only: not for matching end
    // tags, ignored stray end tags, diagnostics, end of file, or refused input.
    for (source, expected_nodes) in [
        // Document + html + head + body.
        ("<body>", 4),
        // ... + one div. The matching end tag admits nothing.
        ("<body><div></div>", 5),
        // ... the ignored stray end tag and its diagnostic admit nothing.
        ("<body><div></div></div>", 5),
        // ... the end-of-file diagnostic admits nothing.
        ("<body><div>", 5),
        // ... the refused attributed tag admits nothing.
        ("<body><div></div><div a>", 5),
        // ... one text node; the second contribution admits nothing.
        ("<body>a</div>b", 5),
    ] {
        let run = tokenize_text(source, 1, generous_limits());
        assert!(
            !run.is_incomplete(),
            "{source:?}: the lower layer completes"
        );
        let observed = observe(&run);
        assert_eq!(
            node_count(&observed.tree),
            expected_nodes,
            "{source:?}: constructed node count"
        );
        assert_eq!(
            observed.identity_events, expected_nodes,
            "{source:?}: identity events equal constructed nodes"
        );
    }
}

/// Identity-based relationship meaning must not be storage-index based. The
/// private arena is re-laid-out with unreferenced padding slots, which moves
/// every real node to a different storage index, and the complete observation
/// must be unchanged.
#[test]
fn storage_order_perturbation_preserves_relationship_meaning() {
    for id in CANDIDATE_IDS {
        let run = run_for(id);
        let baseline = observe_with_layout(&run, CandidateStorageLayout::COMPACT);
        for padding in [1usize, 2, 3, 7] {
            let perturbed = observe_with_layout(
                &run,
                CandidateStorageLayout {
                    padding_before_each_node: padding,
                },
            );
            assert_eq!(
                perturbed, baseline,
                "{id}: storage padding {padding} must not change candidate meaning"
            );
        }
    }

    // The perturbation really does move storage indices, so the check above is
    // not vacuous.
    let run = run_for("DV3");
    let mut compact = CandidateSession::new(CandidateStorageLayout::COMPACT);
    let mut padded = CandidateSession::new(CandidateStorageLayout {
        padding_before_each_node: 3,
    });
    for (index, token) in run.tokens().iter().enumerate() {
        let trigger = candidate_trigger(token, index);
        let Ok(shape) = candidate_shape(token) else {
            break;
        };
        compact.process(index, shape, trigger);
        padded.process(index, shape, trigger);
    }
    assert_ne!(
        compact.open_elements, padded.open_elements,
        "the padded layout must place real nodes at different storage indices"
    );
    assert_eq!(
        compact.tree(),
        padded.tree(),
        "the projected tree is identical despite different storage indices"
    );
}

// ---------------------------------------------------------------------------
// 13. Completion and checkpoint
// ---------------------------------------------------------------------------

/// A candidate run can only be Complete when the accepted lower layer is, no
/// end-of-file semantics are invented when the tokenizer emitted none, and a
/// refused token leaves committed coverage at its own start.
#[test]
fn lower_layer_incompleteness_is_never_upgraded() {
    // DV12 is the canonical case: the tokenizer stops inside a character
    // reference, so there is no end-of-file token to act on.
    let dv12 = observe_fixture("DV12");
    let dv12_run = run_for("DV12");
    assert!(
        dv12_run.is_incomplete(),
        "character references are the tokenizer's own capability"
    );
    assert_eq!(
        dv12_run.coverage().processed_end(),
        11,
        "the lower layer's own committed prefix"
    );
    assert_eq!(
        dv12.checkpoint,
        CandidateCheckpoint {
            mode: CandidateMode::InBody,
            open_div_depth: 1,
            committed_prefix_end: 11,
            completion: CandidateCompletion::IncompleteLowerLayer,
        },
        "no completion upgrade and no invented end of file"
    );
    assert_eq!(
        dv12.diagnostics,
        vec![missing_doctype_at_body()],
        "no end-of-file diagnostic is invented when no end-of-file token exists"
    );
    assert!(
        !dv12.tokens.iter().any(|record| record
            .dispatches
            .iter()
            .any(|dispatch| dispatch.outcome == CandidateDispatchOutcome::Stopped)),
        "no stop disposition without an emitted end-of-file token"
    );

    // Every tokenizer incomplete cause is covered by the same gate.
    let source = fixture("DV1").source_text();
    for (label, limits) in [
        (
            "source bytes",
            HtmlTokenizerLimits::new(4, 8_192, 1_024, 1_024, 256, 4_096, 1_024),
        ),
        (
            "emitted tokens",
            HtmlTokenizerLimits::new(1_024, 8_192, 1, 1_024, 256, 4_096, 1_024),
        ),
        (
            "transition steps",
            HtmlTokenizerLimits::new(1_024, 3, 1_024, 1_024, 256, 4_096, 1_024),
        ),
        (
            "invalid configuration: zero transition steps",
            HtmlTokenizerLimits::new(1_024, 0, 1_024, 1_024, 256, 4_096, 1_024),
        ),
        (
            "invalid configuration: zero emitted tokens",
            HtmlTokenizerLimits::new(1_024, 8_192, 0, 1_024, 256, 4_096, 1_024),
        ),
    ] {
        let run = tokenize_text(source, 1, limits);
        assert!(run.is_incomplete(), "{label}: lower layer is incomplete");
        assert_ne!(
            observe(&run).checkpoint.completion,
            CandidateCompletion::Complete,
            "{label}: candidate completion is never upgraded"
        );
    }

    // An invalid tokenizer configuration stays invalid and emits nothing the
    // candidate could act on.
    let invalid = tokenize_text(
        source,
        1,
        HtmlTokenizerLimits::new(1_024, 0, 1_024, 1_024, 256, 4_096, 1_024),
    );
    let observed = observe(&invalid);
    assert_eq!(
        observed.checkpoint.completion,
        CandidateCompletion::IncompleteLowerLayer
    );
    assert_eq!(observed.checkpoint.committed_prefix_end, 0);
    assert_eq!(
        observed.identity_events, 1,
        "only the Document container exists"
    );
}

/// A candidate parse diagnostic never implies incomplete parsing, and a
/// complete tokenizer run with an open ordinary element still maps to
/// candidate `Complete`.
#[test]
fn diagnostics_and_completion_stay_orthogonal() {
    for id in ["DV5", "DV6"] {
        let observed = observe_fixture(id);
        assert!(
            observed.diagnostics.len() >= 2,
            "{id}: the run records candidate parse diagnostics"
        );
        assert_eq!(
            observed.checkpoint.completion,
            CandidateCompletion::Complete,
            "{id}: diagnostics do not imply incomplete parsing"
        );
    }
    assert_eq!(
        observe_fixture("DV6").checkpoint.open_div_depth,
        1,
        "DV6 completes with an open candidate div"
    );
}

/// Committed coverage is monotonic, ends at the last supported token, and
/// never exceeds the source.
#[test]
fn committed_coverage_is_monotonic_and_honest() {
    for id in CANDIDATE_IDS {
        let observed = observe_fixture(id);
        let candidate = fixture(id);
        let mut previous = 0;
        for record in &observed.tokens {
            assert!(
                record.committed_prefix_end >= previous,
                "{id}: committed coverage never moves backwards"
            );
            previous = record.committed_prefix_end;
        }
        assert!(
            previous <= candidate.length,
            "{id}: committed coverage stays inside the source"
        );
        assert_eq!(
            observed.checkpoint.committed_prefix_end, previous,
            "{id}: the checkpoint reports the last committed prefix"
        );
    }
}

// ---------------------------------------------------------------------------
// 14. Termination and structural bounds
// ---------------------------------------------------------------------------

/// No retry limit, work budget, parser-step budget, nesting limit, or
/// node-count limit is required. Every bound below is a structural consequence
/// of the selected action set and the emitted token stream.
#[test]
fn candidate_work_is_bounded_without_any_runtime_limit() {
    for id in CANDIDATE_IDS {
        let observed = observe_fixture(id);
        let run = run_for(id);

        for record in &observed.tokens {
            let modes: Vec<CandidateMode> = record
                .dispatches
                .iter()
                .map(|dispatch| dispatch.evaluated_in)
                .collect();
            for (position, mode) in modes.iter().enumerate() {
                assert!(
                    !modes[..position].contains(mode),
                    "{id}: token {} evaluated {mode:?} twice",
                    record.index
                );
            }
            assert!(
                record.dispatches.len() <= CandidateMode::ALL.len(),
                "{id}: per-token dispatches are bounded by the mode cardinality"
            );

            // The selected div cells add no same-token redispatch edge.
            if record.mode_before == CandidateMode::InBody {
                assert_eq!(
                    record.reprocesses, 0,
                    "{id}: token {} performed a same-token redispatch inside in body",
                    record.index
                );
                assert_eq!(
                    record.dispatches.len(),
                    1,
                    "{id}: token {} is one in-body dispatch",
                    record.index
                );
            }
        }

        // Stack depth moves by exactly one per accepted start or matching end.
        for (position, record) in observed.tokens.iter().enumerate() {
            let before = depth_before(&observed, position);
            let after = record.open_div_depth_after;
            assert!(
                after == before || after == before + 1 || after + 1 == before,
                "{id}: token {} moved the div stack by more than one",
                record.index
            );
        }

        // Node growth follows committed semantic token effects only.
        let admitted_tokens = observed
            .tokens
            .iter()
            .filter(|record| record.refusal().is_none())
            .count();
        assert!(
            node_count(&observed.tree) <= 4 + admitted_tokens,
            "{id}: node count is bounded by the shell plus admitted tokens"
        );
        assert!(
            observed.diagnostics.len() <= 1 + run.tokens().len(),
            "{id}: diagnostics are linearly bounded by the emitted token stream"
        );
        let dispatches: usize = observed
            .tokens
            .iter()
            .map(|record| record.dispatches.len())
            .sum();
        assert!(
            dispatches <= observed.tokens.len() * CandidateMode::ALL.len(),
            "{id}: total work is bounded by tokens times mode cardinality"
        );
    }
}

// ---------------------------------------------------------------------------
// 15. Bounded generated falsification
// ---------------------------------------------------------------------------

/// The bounded candidate sequences.
///
/// The enumeration depth and count below are **test enumeration
/// infrastructure**. They are not a candidate or production runtime policy,
/// resource dimension, nesting limit, or work budget.
fn generated_candidate_sources() -> Vec<String> {
    const PIECES: [&str; 3] = ["<div>", "</div>", "x"];
    const MAX_SEQUENCE_LENGTH: u32 = 4;
    const MAX_BALANCED_DEPTH: usize = 8;

    let mut sources = Vec::new();
    for length in 0..=MAX_SEQUENCE_LENGTH {
        for code in 0..PIECES.len().pow(length) {
            let mut remaining = code;
            let mut digits = Vec::new();
            for _ in 0..length {
                digits.push(remaining % PIECES.len());
                remaining /= PIECES.len();
            }
            digits.reverse();
            let mut source = String::from("<body>");
            for digit in digits {
                source.push_str(PIECES[digit]);
            }
            sources.push(source);
        }
    }
    for depth in 0..=MAX_BALANCED_DEPTH {
        let mut source = String::from("<body>");
        for _ in 0..depth {
            source.push_str("<div>");
        }
        source.push('x');
        for _ in 0..depth {
            source.push_str("</div>");
        }
        sources.push(source);
    }
    sources
}

#[test]
fn generated_candidate_sequences_uphold_the_theorem() {
    let sources = generated_candidate_sources();
    assert_eq!(
        sources.len(),
        130,
        "the bounded enumeration is exactly the sequences described above"
    );

    for source in &sources {
        let run = tokenize_text(source, 1, generous_limits());
        assert!(
            !run.is_incomplete(),
            "{source:?}: the lower layer completes"
        );
        let observed = observe(&run);

        assert_eq!(
            observed.checkpoint.completion,
            CandidateCompletion::Complete,
            "{source:?}: every generated cell is modelled"
        );
        assert_eq!(
            observed.tokens.len(),
            run.tokens().len(),
            "{source:?}: every emitted token is processed"
        );

        let mut expected_depth = 0usize;
        let mut expected_divs = 0usize;
        let mut expected_strays = 0usize;
        for (position, record) in observed.tokens.iter().enumerate() {
            assert_eq!(
                depth_before(&observed, position),
                expected_depth,
                "{source:?}: token {} entry depth",
                record.index
            );
            match token_at(&run, record.index) {
                HtmlToken::Tag(tag) if tag.name().interpreted() == "div" => match tag.kind() {
                    HtmlTagKind::Start => {
                        expected_depth += 1;
                        expected_divs += 1;
                        assert_eq!(
                            record.dispatches,
                            vec![consumed_in(CandidateMode::InBody)],
                            "{source:?}: an accepted start tag is one consuming dispatch"
                        );
                    }
                    HtmlTagKind::End => {
                        if expected_depth > 0 {
                            // A matching end pops exactly one.
                            expected_depth -= 1;
                            assert_eq!(
                                record.dispatches,
                                vec![consumed_in(CandidateMode::InBody)],
                                "{source:?}: a matching end tag is one consuming dispatch"
                            );
                        } else {
                            // The stack never underflows: a stray end tag is
                            // diagnosed and ignored instead.
                            expected_strays += 1;
                            assert_eq!(
                                record.dispatches,
                                vec![ignored_in(CandidateMode::InBody)],
                                "{source:?}: a stray end tag is ignored, never a pop"
                            );
                        }
                    }
                },
                _ => {}
            }
            assert_eq!(
                record.open_div_depth_after, expected_depth,
                "{source:?}: token {} exit depth",
                record.index
            );
        }

        assert_eq!(
            observed.checkpoint.open_div_depth, expected_depth,
            "{source:?}: terminal depth"
        );
        assert_eq!(
            div_count(&observed.tree),
            expected_divs,
            "{source:?}: one candidate div node per accepted start tag, none for stray ends"
        );
        assert_eq!(
            observed.identity_events,
            node_count(&observed.tree),
            "{source:?}: identity events equal constructed nodes, with no gap"
        );
        assert_eq!(
            observed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::UnmatchedDivEndTag)
                .count(),
            expected_strays,
            "{source:?}: exactly one diagnostic per stray end tag"
        );
        assert_eq!(
            observed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code
                    == CandidateDiagnosticCode::OpenOrdinaryElementAtEndOfFile)
                .count(),
            usize::from(expected_depth > 0),
            "{source:?}: one end-of-file diagnostic exactly when a div is still open"
        );

        // Creation order is deterministic across repeated runs.
        assert_eq!(
            observe(&tokenize_text(source, 1, generous_limits())),
            observed,
            "{source:?}: repeated runs are identical"
        );

        // Refused input leaks no mutation: appending an unsupported token
        // changes the tree, identity, and diagnostics by nothing.
        let extended = tokenize_text(&format!("{source}<p>"), 1, generous_limits());
        assert!(!extended.is_incomplete());
        let after_refusal = observe(&extended);
        assert_eq!(
            after_refusal.tree, observed.tree,
            "{source:?}: a refused token mutates no tree"
        );
        assert_eq!(
            after_refusal.identity_events, observed.identity_events,
            "{source:?}: a refused token admits no identity"
        );
        assert_eq!(
            after_refusal.checkpoint.open_div_depth, expected_depth,
            "{source:?}: a refused token mutates no stack"
        );
        assert_eq!(
            after_refusal.checkpoint.committed_prefix_end,
            source.len(),
            "{source:?}: committed coverage ends at the start of the refused token"
        );
        assert!(
            matches!(
                after_refusal.checkpoint.completion,
                CandidateCompletion::IncompleteUnsupported {
                    capability: CandidateUnsupported::OutsideModelledCandidateCells,
                    ..
                }
            ),
            "{source:?}: the refusal is reported as the candidate's own evidence"
        );

        // Complete/incomplete monotonicity: a truncated lower layer can never
        // be upgraded to candidate Complete.
        let truncated = tokenize_text(
            source,
            1,
            HtmlTokenizerLimits::new(3, 8_192, 1_024, 1_024, 256, 4_096, 1_024),
        );
        assert!(
            truncated.is_incomplete(),
            "{source:?}: lower layer truncated"
        );
        assert_ne!(
            observe(&truncated).checkpoint.completion,
            CandidateCompletion::Complete,
            "{source:?}: candidate completion is never upgraded"
        );
    }
}

// ---------------------------------------------------------------------------
// 16. Determinism and source identity
// ---------------------------------------------------------------------------

/// Semantic correspondence is stable across repeated runs and across differing
/// caller `SourceId` values, while the exact anchors the candidate reads carry
/// the requested source identity. No raw identity encoding is asserted.
#[test]
fn candidate_semantics_are_deterministic_across_source_ids() {
    for id in CANDIDATE_IDS {
        let candidate = fixture(id);
        let baseline = observe(&tokenize_text(
            candidate.source_text(),
            1,
            generous_limits(),
        ));
        for raw_source_id in [1_u64, 7, 4_242, u64::from(u32::MAX)] {
            let source = SourceText::new(
                SourceId::new(raw_source_id),
                candidate.source_text().to_owned(),
            );
            let run = tokenize(&source, generous_limits());

            assert_eq!(
                observe(&run),
                baseline,
                "{id}: semantic correspondence for SourceId {raw_source_id}"
            );

            // Every anchor the candidate reads carries the requested identity.
            for token in run.tokens() {
                let anchors = match token {
                    HtmlToken::Character(character) => vec![character.source()],
                    HtmlToken::Tag(tag) => vec![tag.complete(), tag.name().source()],
                    HtmlToken::EndOfFile(end_of_file) => vec![end_of_file.source()],
                };
                for anchor in anchors {
                    assert_eq!(
                        anchor.source_id(),
                        SourceId::new(raw_source_id),
                        "{id}: exact anchors carry the requested SourceId"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 17. Inherited refusal boundary
// ---------------------------------------------------------------------------

/// TC-S3 widens exactly one thing: no-attribute `div` construction inside
/// actual `in body`. Everything the candidate does not model stays refused
/// before mutation, so the successor cannot silently absorb neighbouring
/// cells or generalize to arbitrary ordinary elements.
#[test]
fn the_candidate_widens_nothing_beyond_selected_in_body_div_cells() {
    for (source, expected) in [
        // Whitespace stays refused outside `in body`.
        (" ", CandidateUnsupported::WhitespaceSensitiveCharacterData),
        (
            "\t<body>",
            CandidateUnsupported::WhitespaceSensitiveCharacterData,
        ),
        // No other ordinary element name is admitted.
        (
            "<body><p>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
        (
            "<body><span>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
        (
            "<body><section>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
        (
            "<body><div></div><p>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
        // Attributes and self-closing solidi stay refused, on shell tags and
        // on the newly selected `div` alike.
        ("<body a>", CandidateUnsupported::TagWithAttributes),
        ("<body/>", CandidateUnsupported::SelfClosingTag),
        ("<body><div id=x>", CandidateUnsupported::TagWithAttributes),
        ("<body><div/>", CandidateUnsupported::SelfClosingTag),
        (
            "<body><div></div a>",
            CandidateUnsupported::TagWithAttributes,
        ),
        // `div` first evaluated outside actual `in body`.
        ("<div>", CandidateUnsupported::DivTagOutsideInBody),
        ("</div>", CandidateUnsupported::DivTagOutsideInBody),
        (
            "<body></body><div>",
            CandidateUnsupported::DivTagOutsideInBody,
        ),
        (
            "<body></body></div>",
            CandidateUnsupported::DivTagOutsideInBody,
        ),
        // Shell interactions while a candidate `div` is open.
        (
            "<body><div></body>",
            CandidateUnsupported::ShellInteractionWithOpenDiv,
        ),
        (
            "<body><div><body>",
            CandidateUnsupported::ShellInteractionWithOpenDiv,
        ),
        (
            "<body><div></html>",
            CandidateUnsupported::ShellInteractionWithOpenDiv,
        ),
        // Predecessor cells this deliberately partial oracle does not model.
        (
            "<html>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
        (
            "<body><body>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
        (
            "<body></body>x",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
    ] {
        let run = tokenize_text(source, 1, generous_limits());
        assert!(!run.is_incomplete(), "{source:?}: lower layer completes");
        let observed = observe(&run);
        let CandidateCompletion::IncompleteUnsupported { capability, .. } =
            observed.checkpoint.completion
        else {
            panic!("{source:?}: expected an explicit candidate refusal")
        };
        assert_eq!(capability, expected, "{source:?}");

        // Refusal precedes mutation *by the refusing cell*: the refused
        // dispatch is terminal for its token, and the refused token advances
        // committed coverage by nothing, so the terminal checkpoint stays the
        // last valid committed prefix.
        //
        // An earlier dispatch of the same token may already have committed a
        // supported effect — `<html>` is reprocessed out of `Initial` before
        // the model refuses it in `before html`. That is checkpoint honesty,
        // not a partial mutation of the refused cell. The DV refusal cases
        // assert the stronger first-dispatch property separately.
        let (last, earlier) = observed
            .tokens
            .split_last()
            .expect("at least one token record");
        assert!(last.refusal().is_some(), "{source:?}");
        let committed_before_refusal = earlier
            .last()
            .map_or(0, |record| record.committed_prefix_end);
        assert_eq!(
            last.committed_prefix_end, committed_before_refusal,
            "{source:?}: a refused token commits no coverage"
        );
        assert_eq!(
            observed.identity_events,
            node_count(&observed.tree),
            "{source:?}: a refused token admits no identity and leaves no gap"
        );
    }
}

/// The predecessor `<body><p>` boundary keeps its exact historical meaning:
/// unsupported, with the committed prefix ending at byte 6 and the refusal
/// naming `[6,9)`.
#[test]
fn the_historical_body_p_boundary_is_unchanged() {
    let observed = observe_fixture("DV11");
    assert_eq!(
        observed.checkpoint,
        refused_at(
            CandidateMode::InBody,
            0,
            6,
            CandidateUnsupported::OutsideModelledCandidateCells,
            1,
            (6, 9),
        )
    );
    assert_eq!(observed.tree, shell_authored_body(vec![]));
    assert_eq!(observed.identity_events, 4);
}
