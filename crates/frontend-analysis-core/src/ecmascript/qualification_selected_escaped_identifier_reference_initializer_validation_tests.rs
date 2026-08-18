//! Candidate-independent escaped `IdentifierReference` initializer validation for Issue #241.
//!
//! This module composes already-accepted Unicode/UES facts with the selected
//! RHS `IdentifierReference` policy without calling production lexical,
//! static-semantics, aggregate, runtime-evaluation, or future expression code.
//! Expected authored ranges, decoded identity, applicability, and lifecycle are
//! fixture authority in this module rather than output from a candidate parser.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::{gold_source, gold_subject_range};
use super::unicode::{is_id_continue, is_id_start};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 241;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const THIS_SOURCE: &str = include_str!(
    "qualification_selected_escaped_identifier_reference_initializer_validation_tests.rs"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ByteRange {
    start: usize,
    end: usize,
}

impl ByteRange {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundaryKind {
    BindingListComma,
    AuthoredSemicolon,
    AutomaticAtEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamePolicyCategory {
    Ordinary,
    StrictOnlyInSelectedNonStrictScript,
    YieldAwaitViaEscapedIdentifier,
    BindingPolicyContrast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedStaticSemantics {
    Accepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FutureLifecycle {
    SelectedAcceptedIncomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FutureUnsupportedDisposition {
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FutureSelectedExpectation {
    static_semantics: ExpectedStaticSemantics,
    lifecycle: FutureLifecycle,
}

const SELECTED_ACCEPTED_INCOMPLETE: FutureSelectedExpectation = FutureSelectedExpectation {
    static_semantics: ExpectedStaticSemantics::Accepted,
    lifecycle: FutureLifecycle::SelectedAcceptedIncomplete,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentifierReferenceRoute {
    DirectYieldAlternative,
    DirectAwaitAlternative,
    EscapedIdentifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PositiveFixture {
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    rhs_range: ByteRange,
    decoded_code_points: &'static [u32],
    string_value: &'static str,
    boundary_offset: usize,
    boundary: BoundaryKind,
    category: NamePolicyCategory,
    route: IdentifierReferenceRoute,
    expected: FutureSelectedExpectation,
}

// The constructor deliberately mirrors independent authored, decoded, boundary,
// and policy fields instead of grouping them into opaque test-only containers.
#[allow(clippy::too_many_arguments)]
const fn positive_fixture(
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    rhs_range: ByteRange,
    decoded_code_points: &'static [u32],
    string_value: &'static str,
    boundary_offset: usize,
    boundary: BoundaryKind,
    category: NamePolicyCategory,
) -> PositiveFixture {
    PositiveFixture {
        id,
        source,
        rhs,
        rhs_range,
        decoded_code_points,
        string_value,
        boundary_offset,
        boundary,
        category,
        route: IdentifierReferenceRoute::EscapedIdentifier,
        expected: SELECTED_ACCEPTED_INCOMPLETE,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RouteContrastFixture {
    direct_spelling: &'static str,
    escaped_rhs: &'static str,
    expected_string_value: &'static str,
    direct_route: IdentifierReferenceRoute,
    escaped_route: IdentifierReferenceRoute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecodeFailure {
    MissingEscape,
    MalformedEscape,
    NonCodePoint,
    InvalidStart,
    InvalidPart,
    DecodedReserved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecodedIdentifier {
    code_points: Vec<u32>,
    string_value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EscapeFormation {
    end: usize,
    code_point: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NegativeFixture {
    id: &'static str,
    rhs: &'static str,
    expected: DecodeFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StaticReachabilityFixture {
    id: &'static str,
    source: &'static str,
    rhs_range: ByteRange,
    rule_id: &'static str,
    subject: ByteRange,
    subject_fragment: &'static str,
    control_gold_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GrammarReachabilityFixture {
    id: &'static str,
    source: &'static str,
    rhs_range: ByteRange,
    subject: ByteRange,
    subject_fragment: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedTailKind {
    Member,
    Call,
    Binary,
    Assignment,
    Conditional,
    Comment,
    UnexpectedTail,
    ExtraSemicolon,
    RequiresNonEofAsi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UnsupportedTailFixture {
    id: &'static str,
    source: &'static str,
    rhs_range: ByteRange,
    tail_kind: UnsupportedTailKind,
}

struct FutureUnsupportedFixtureSet<T: 'static> {
    fixtures: &'static [T],
    disposition: FutureUnsupportedDisposition,
}

const BASE_POSITIVE_FIXTURES: &[PositiveFixture] = &[
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-FIXED-START-001",
        r"const x = \u0066oo;",
        r"\u0066oo",
        ByteRange::new(10, 18),
        &[0x66, 0x6F, 0x6F],
        "foo",
        18,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-MIXED-PART-001",
        r"const x = f\u006Fo;",
        r"f\u006Fo",
        ByteRange::new(10, 18),
        &[0x66, 0x6F, 0x6F],
        "foo",
        18,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-BRACED-COMMA-001",
        r"let x = \u{66}oo, y = bar;",
        r"\u{66}oo",
        ByteRange::new(8, 16),
        &[0x66, 0x6F, 0x6F],
        "foo",
        16,
        BoundaryKind::BindingListComma,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-LEADING-ZERO-EOF-001",
        r"const x = \u{00000066}oo",
        r"\u{00000066}oo",
        ByteRange::new(10, 24),
        &[0x66, 0x6F, 0x6F],
        "foo",
        24,
        BoundaryKind::AutomaticAtEof,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-SUPPLEMENTARY-001",
        r"const x = \u{1D49C};",
        r"\u{1D49C}",
        ByteRange::new(10, 19),
        &[0x1D49C],
        "𝒜",
        19,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-DOLLAR-001",
        r"const x = \u0024;",
        r"\u0024",
        ByteRange::new(10, 16),
        &[0x24],
        "$",
        16,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-LOW-LINE-001",
        r"const x = \u005F;",
        r"\u005F",
        ByteRange::new(10, 16),
        &[0x5F],
        "_",
        16,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-DIGIT-PART-001",
        r"const x = a\u0030;",
        r"a\u0030",
        ByteRange::new(10, 17),
        &[0x61, 0x30],
        "a0",
        17,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-COMBINING-PART-001",
        r"const x = a\u0301;",
        r"a\u0301",
        ByteRange::new(10, 17),
        &[0x61, 0x301],
        "a\u{0301}",
        17,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-ZWNJ-PART-001",
        r"const x = a\u200Cb;",
        r"a\u200Cb",
        ByteRange::new(10, 18),
        &[0x61, 0x200C, 0x62],
        "a\u{200C}b",
        18,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-ZWJ-PART-001",
        r"const x = a\u200Db;",
        r"a\u200Db",
        ByteRange::new(10, 18),
        &[0x61, 0x200D, 0x62],
        "a\u{200D}b",
        18,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-MULTIPLE-ESCAPES-001",
        r"const x = \u0066\u006F\u006F;",
        r"\u0066\u006F\u006F",
        ByteRange::new(10, 28),
        &[0x66, 0x6F, 0x6F],
        "foo",
        28,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-PRECOMPOSED-001",
        r"const x = \u00E9;",
        r"\u00E9",
        ByteRange::new(10, 16),
        &[0xE9],
        "é",
        16,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-DECOMPOSED-001",
        r"const x = e\u0301;",
        r"e\u0301",
        ByteRange::new(10, 17),
        &[0x65, 0x301],
        "e\u{0301}",
        17,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-ESCAPED-BINDING-COMPOSITION-001",
        r"const \u0078 = \u0066oo;",
        r"\u0066oo",
        ByteRange::new(15, 23),
        &[0x66, 0x6F, 0x6F],
        "foo",
        23,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-MULTIDECL-001",
        r"let x = bar; const y = \u0066oo",
        r"\u0066oo",
        ByteRange::new(23, 31),
        &[0x66, 0x6F, 0x6F],
        "foo",
        31,
        BoundaryKind::AutomaticAtEof,
        NamePolicyCategory::Ordinary,
    ),
    positive_fixture(
        "ESCAPED-IDREF-POSITIVE-TRAILING-TRIVIA-EOF-001",
        concat!(r"const x = \u0066oo", " \t\n"),
        r"\u0066oo",
        ByteRange::new(10, 18),
        &[0x66, 0x6F, 0x6F],
        "foo",
        21,
        BoundaryKind::AutomaticAtEof,
        NamePolicyCategory::Ordinary,
    ),
];

const NAME_POLICY_FIXTURES: &[PositiveFixture] = &[
    positive_fixture(
        "ESCAPED-IDREF-NAME-STRICT-ONLY-LET",
        r"const x = \u006Cet;",
        r"\u006Cet",
        ByteRange::new(10, 18),
        &[0x6C, 0x65, 0x74],
        "let",
        18,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::StrictOnlyInSelectedNonStrictScript,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-STRICT-ONLY-STATIC",
        r"const x = \u0073tatic;",
        r"\u0073tatic",
        ByteRange::new(10, 21),
        &[0x73, 0x74, 0x61, 0x74, 0x69, 0x63],
        "static",
        21,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::StrictOnlyInSelectedNonStrictScript,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-STRICT-ONLY-IMPLEMENTS",
        r"const x = \u0069mplements;",
        r"\u0069mplements",
        ByteRange::new(10, 25),
        &[0x69, 0x6D, 0x70, 0x6C, 0x65, 0x6D, 0x65, 0x6E, 0x74, 0x73],
        "implements",
        25,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::StrictOnlyInSelectedNonStrictScript,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-STRICT-ONLY-INTERFACE",
        r"const x = \u0069nterface;",
        r"\u0069nterface",
        ByteRange::new(10, 24),
        &[0x69, 0x6E, 0x74, 0x65, 0x72, 0x66, 0x61, 0x63, 0x65],
        "interface",
        24,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::StrictOnlyInSelectedNonStrictScript,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-STRICT-ONLY-PACKAGE",
        r"const x = \u0070ackage;",
        r"\u0070ackage",
        ByteRange::new(10, 22),
        &[0x70, 0x61, 0x63, 0x6B, 0x61, 0x67, 0x65],
        "package",
        22,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::StrictOnlyInSelectedNonStrictScript,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-STRICT-ONLY-PRIVATE",
        r"const x = \u0070rivate;",
        r"\u0070rivate",
        ByteRange::new(10, 22),
        &[0x70, 0x72, 0x69, 0x76, 0x61, 0x74, 0x65],
        "private",
        22,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::StrictOnlyInSelectedNonStrictScript,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-STRICT-ONLY-PROTECTED",
        r"const x = \u0070rotected;",
        r"\u0070rotected",
        ByteRange::new(10, 24),
        &[0x70, 0x72, 0x6F, 0x74, 0x65, 0x63, 0x74, 0x65, 0x64],
        "protected",
        24,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::StrictOnlyInSelectedNonStrictScript,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-STRICT-ONLY-PUBLIC",
        r"const x = \u0070ublic;",
        r"\u0070ublic",
        ByteRange::new(10, 21),
        &[0x70, 0x75, 0x62, 0x6C, 0x69, 0x63],
        "public",
        21,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::StrictOnlyInSelectedNonStrictScript,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-YIELD-VIA-IDENTIFIER",
        r"const x = \u0079ield;",
        r"\u0079ield",
        ByteRange::new(10, 20),
        &[0x79, 0x69, 0x65, 0x6C, 0x64],
        "yield",
        20,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::YieldAwaitViaEscapedIdentifier,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-AWAIT-VIA-IDENTIFIER",
        r"const x = a\u0077ait;",
        r"a\u0077ait",
        ByteRange::new(10, 20),
        &[0x61, 0x77, 0x61, 0x69, 0x74],
        "await",
        20,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::YieldAwaitViaEscapedIdentifier,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-BINDING-CONTRAST-EVAL",
        r"const x = \u0065val;",
        r"\u0065val",
        ByteRange::new(10, 19),
        &[0x65, 0x76, 0x61, 0x6C],
        "eval",
        19,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::BindingPolicyContrast,
    ),
    positive_fixture(
        "ESCAPED-IDREF-NAME-BINDING-CONTRAST-ARGUMENTS",
        r"const x = \u0061rguments;",
        r"\u0061rguments",
        ByteRange::new(10, 24),
        &[0x61, 0x72, 0x67, 0x75, 0x6D, 0x65, 0x6E, 0x74, 0x73],
        "arguments",
        24,
        BoundaryKind::AuthoredSemicolon,
        NamePolicyCategory::BindingPolicyContrast,
    ),
];

const ROUTE_CONTRASTS: &[RouteContrastFixture] = &[
    RouteContrastFixture {
        direct_spelling: "yield",
        escaped_rhs: r"\u0079ield",
        expected_string_value: "yield",
        direct_route: IdentifierReferenceRoute::DirectYieldAlternative,
        escaped_route: IdentifierReferenceRoute::EscapedIdentifier,
    },
    RouteContrastFixture {
        direct_spelling: "await",
        escaped_rhs: r"a\u0077ait",
        expected_string_value: "await",
        direct_route: IdentifierReferenceRoute::DirectAwaitAlternative,
        escaped_route: IdentifierReferenceRoute::EscapedIdentifier,
    },
];

const NEGATIVE_FIXTURES: &[NegativeFixture] = &[
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-MALFORMED-EMPTY-BRACED",
        rhs: r"\u{}",
        expected: DecodeFailure::MalformedEscape,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-MALFORMED-SHORT-FIXED",
        rhs: r"\u0",
        expected: DecodeFailure::MalformedEscape,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-MALFORMED-UNCLOSED-BRACED",
        rhs: r"\u{61",
        expected: DecodeFailure::MalformedEscape,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-NONCODEPOINT",
        rhs: r"\u{110000}",
        expected: DecodeFailure::NonCodePoint,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-DEFERRED-NONHEX-BRACED",
        rhs: r"\u{G}",
        expected: DecodeFailure::MalformedEscape,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-DEFERRED-NONHEX-FIXED",
        rhs: r"\u00G0",
        expected: DecodeFailure::MalformedEscape,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-DEFERRED-SEMICOLON-BRACED",
        rhs: r"\u{61;",
        expected: DecodeFailure::MalformedEscape,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-DEFERRED-SHORT-NONHEX",
        rhs: r"\u0x",
        expected: DecodeFailure::MalformedEscape,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-DEFERRED-U-ONLY",
        rhs: r"\u",
        expected: DecodeFailure::MalformedEscape,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-DEFERRED-OPEN-BRACE",
        rhs: r"\u{",
        expected: DecodeFailure::MalformedEscape,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-INVALID-START-DIGIT",
        rhs: r"\u0030",
        expected: DecodeFailure::InvalidStart,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-INVALID-PART-HYPHEN",
        rhs: r"a\u002D",
        expected: DecodeFailure::InvalidPart,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-SURROGATE-FIXED",
        rhs: r"\uD800",
        expected: DecodeFailure::InvalidStart,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-SURROGATE-BRACED",
        rhs: r"\u{D800}",
        expected: DecodeFailure::InvalidStart,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-ZWNJ-START",
        rhs: r"\u200C",
        expected: DecodeFailure::InvalidStart,
    },
    NegativeFixture {
        id: "ESCAPED-IDREF-NEGATIVE-ZWJ-START",
        rhs: r"\u200D",
        expected: DecodeFailure::InvalidStart,
    },
];

const UNCONDITIONALLY_RESERVED_WORDS: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
];

const DIRECT_ONLY_CONTROLS: &[&str] = &["foo", "let", "yield", "await", "eval", "arguments"];

const STATIC_REACHABILITY_FIXTURES: &[StaticReachabilityFixture] = &[
    StaticReachabilityFixture {
        id: "ESCAPED-IDREF-STATIC-EE01",
        source: r"let \u0030 = \u0066oo;",
        rhs_range: ByteRange::new(13, 21),
        rule_id: "EE-01-R01",
        subject: ByteRange::new(4, 10),
        subject_fragment: r"\u0030",
        control_gold_id: "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
    },
    StaticReachabilityFixture {
        id: "ESCAPED-IDREF-STATIC-EE04",
        source: r"let \u0069f = \u0066oo;",
        rhs_range: ByteRange::new(14, 22),
        rule_id: "EE-04-R08",
        subject: ByteRange::new(4, 11),
        subject_fragment: r"\u0069f",
        control_gold_id: "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
    },
    StaticReachabilityFixture {
        id: "ESCAPED-IDREF-STATIC-R01",
        source: r"let let = \u0066oo;",
        rhs_range: ByteRange::new(10, 18),
        rule_id: "EE-15-R01",
        subject: ByteRange::new(4, 7),
        subject_fragment: "let",
        control_gold_id: "JS-GOLD-LEXDECL-LET-BINDING-001",
    },
    StaticReachabilityFixture {
        id: "ESCAPED-IDREF-STATIC-R02",
        source: r"let x = \u0066oo, x = bar;",
        rhs_range: ByteRange::new(8, 16),
        rule_id: "EE-15-R02",
        subject: ByteRange::new(18, 19),
        subject_fragment: "x",
        control_gold_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
    },
    StaticReachabilityFixture {
        id: "ESCAPED-IDREF-STATIC-R03",
        source: r"const x = \u0066oo, y;",
        rhs_range: ByteRange::new(10, 18),
        rule_id: "EE-15-R03",
        subject: ByteRange::new(20, 21),
        subject_fragment: "y",
        control_gold_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
    },
    StaticReachabilityFixture {
        id: "ESCAPED-IDREF-STATIC-EE36",
        source: r"let x = \u0066oo; let x = bar;",
        rhs_range: ByteRange::new(8, 16),
        rule_id: "EE-36-R01",
        subject: ByteRange::new(22, 23),
        subject_fragment: "x",
        control_gold_id: "JS-GOLD-SCRIPT-DUPLEXICAL-001",
    },
];

const GRAMMAR_REACHABILITY_FIXTURES: &[GrammarReachabilityFixture] = &[
    GrammarReachabilityFixture {
        id: "ESCAPED-IDREF-GRAMMAR-LATER-EMPTY-BRACED",
        source: r"const x = \u0066oo; let \u{};",
        rhs_range: ByteRange::new(10, 18),
        subject: ByteRange::new(24, 28),
        subject_fragment: r"\u{}",
    },
    GrammarReachabilityFixture {
        id: "ESCAPED-IDREF-GRAMMAR-LATER-PART-EMPTY-BRACED",
        source: r"const x = \u0066oo; let a\u{};",
        rhs_range: ByteRange::new(10, 18),
        subject: ByteRange::new(25, 29),
        subject_fragment: r"\u{}",
    },
    GrammarReachabilityFixture {
        id: "ESCAPED-IDREF-GRAMMAR-LATER-UNCLOSED-BRACED",
        source: r"const x = \u0066oo; let \u{61",
        rhs_range: ByteRange::new(10, 18),
        subject: ByteRange::new(24, 29),
        subject_fragment: r"\u{61",
    },
];

const UNSUPPORTED_TAIL_FIXTURES: &[UnsupportedTailFixture] = &[
    UnsupportedTailFixture {
        id: "ESCAPED-IDREF-TAIL-MEMBER",
        source: r"const x = \u0066oo.bar;",
        rhs_range: ByteRange::new(10, 18),
        tail_kind: UnsupportedTailKind::Member,
    },
    UnsupportedTailFixture {
        id: "ESCAPED-IDREF-TAIL-CALL",
        source: r"const x = \u0066oo();",
        rhs_range: ByteRange::new(10, 18),
        tail_kind: UnsupportedTailKind::Call,
    },
    UnsupportedTailFixture {
        id: "ESCAPED-IDREF-TAIL-BINARY",
        source: r"const x = \u0066oo + 1;",
        rhs_range: ByteRange::new(10, 18),
        tail_kind: UnsupportedTailKind::Binary,
    },
    UnsupportedTailFixture {
        id: "ESCAPED-IDREF-TAIL-ASSIGNMENT",
        source: r"const x = \u0066oo = bar;",
        rhs_range: ByteRange::new(10, 18),
        tail_kind: UnsupportedTailKind::Assignment,
    },
    UnsupportedTailFixture {
        id: "ESCAPED-IDREF-TAIL-CONDITIONAL",
        source: r"const x = \u0066oo ? bar : baz;",
        rhs_range: ByteRange::new(10, 18),
        tail_kind: UnsupportedTailKind::Conditional,
    },
    UnsupportedTailFixture {
        id: "ESCAPED-IDREF-TAIL-COMMENT",
        source: r"const x = \u0066oo/*comment*/;",
        rhs_range: ByteRange::new(10, 18),
        tail_kind: UnsupportedTailKind::Comment,
    },
    UnsupportedTailFixture {
        id: "ESCAPED-IDREF-TAIL-UNEXPECTED",
        source: r"const x = \u0066oo unexpected;",
        rhs_range: ByteRange::new(10, 18),
        tail_kind: UnsupportedTailKind::UnexpectedTail,
    },
    UnsupportedTailFixture {
        id: "ESCAPED-IDREF-TAIL-EXTRA-SEMICOLON",
        source: r"const x = \u0066oo;;",
        rhs_range: ByteRange::new(10, 18),
        tail_kind: UnsupportedTailKind::ExtraSemicolon,
    },
    UnsupportedTailFixture {
        id: "ESCAPED-IDREF-TAIL-NON-EOF-ASI",
        source: concat!(r"const x = \u0066oo", "\nconst y = bar;"),
        rhs_range: ByteRange::new(10, 18),
        tail_kind: UnsupportedTailKind::RequiresNonEofAsi,
    },
];

const NEGATIVE_UNSUPPORTED_EXPECTATION: FutureUnsupportedFixtureSet<NegativeFixture> =
    FutureUnsupportedFixtureSet {
        fixtures: NEGATIVE_FIXTURES,
        disposition: FutureUnsupportedDisposition::UnsupportedCoverage,
    };

const DECODED_RESERVED_UNSUPPORTED_EXPECTATION: FutureUnsupportedFixtureSet<&'static str> =
    FutureUnsupportedFixtureSet {
        fixtures: UNCONDITIONALLY_RESERVED_WORDS,
        disposition: FutureUnsupportedDisposition::UnsupportedCoverage,
    };

const UNSUPPORTED_TAIL_EXPECTATION: FutureUnsupportedFixtureSet<UnsupportedTailFixture> =
    FutureUnsupportedFixtureSet {
        fixtures: UNSUPPORTED_TAIL_FIXTURES,
        disposition: FutureUnsupportedDisposition::UnsupportedCoverage,
    };

fn ascii_hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

fn parse_braced_code_point(digits: &[u8]) -> Result<u32, DecodeFailure> {
    if digits.is_empty() || digits.iter().any(|byte| ascii_hex_value(*byte).is_none()) {
        return Err(DecodeFailure::MalformedEscape);
    }

    let significant = match digits.iter().position(|byte| *byte != b'0') {
        Some(index) => &digits[index..],
        None => return Ok(0),
    };

    if significant.len() > 6 {
        return Err(DecodeFailure::NonCodePoint);
    }

    let mut value = 0_u32;
    for byte in significant {
        value = value * 16 + ascii_hex_value(*byte).ok_or(DecodeFailure::MalformedEscape)?;
    }

    (value <= 0x10_FFFF)
        .then_some(value)
        .ok_or(DecodeFailure::NonCodePoint)
}

fn formed_escape_at(spelling: &str, start: usize) -> Result<EscapeFormation, DecodeFailure> {
    let bytes = spelling.as_bytes();
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return Err(DecodeFailure::MalformedEscape);
    }

    let payload = start + 2;
    if bytes.get(payload) == Some(&b'{') {
        let digits_start = payload + 1;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(|byte| byte.is_ascii_hexdigit()) {
            end += 1;
        }
        if end == digits_start || bytes.get(end) != Some(&b'}') {
            return Err(DecodeFailure::MalformedEscape);
        }
        let code_point = parse_braced_code_point(&bytes[digits_start..end])?;
        return Ok(EscapeFormation {
            end: end + 1,
            code_point,
        });
    }

    let end = payload
        .checked_add(4)
        .ok_or(DecodeFailure::MalformedEscape)?;
    let digits = bytes
        .get(payload..end)
        .ok_or(DecodeFailure::MalformedEscape)?;
    if !digits.iter().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(DecodeFailure::MalformedEscape);
    }

    let mut code_point = 0_u32;
    for byte in digits {
        code_point =
            code_point * 16 + ascii_hex_value(*byte).ok_or(DecodeFailure::MalformedEscape)?;
    }

    Ok(EscapeFormation { end, code_point })
}

fn is_selected_identifier_start_for_oracle(code_point: u32) -> bool {
    code_point == u32::from(b'$') || code_point == u32::from(b'_') || is_id_start(code_point)
}

fn is_selected_identifier_part_for_oracle(code_point: u32) -> bool {
    is_selected_identifier_start_for_oracle(code_point)
        || is_id_continue(code_point)
        || matches!(code_point, 0x200C | 0x200D)
}

fn is_unconditionally_reserved_word_for_oracle(spelling: &str) -> bool {
    UNCONDITIONALLY_RESERVED_WORDS.contains(&spelling)
}

fn decode_selected_escaped_identifier(spelling: &str) -> Result<DecodedIdentifier, DecodeFailure> {
    let mut offset = 0_usize;
    let mut element_index = 0_usize;
    let mut saw_escape = false;
    let mut code_points = Vec::new();

    while offset < spelling.len() {
        let (code_point, end, escaped) = if spelling.as_bytes().get(offset) == Some(&b'\\') {
            let formation = formed_escape_at(spelling, offset)?;
            (formation.code_point, formation.end, true)
        } else {
            let scalar = spelling[offset..]
                .chars()
                .next()
                .ok_or(DecodeFailure::MalformedEscape)?;
            (scalar as u32, offset + scalar.len_utf8(), false)
        };

        let valid_position = if element_index == 0 {
            is_selected_identifier_start_for_oracle(code_point)
        } else {
            is_selected_identifier_part_for_oracle(code_point)
        };
        if !valid_position {
            return Err(if element_index == 0 {
                DecodeFailure::InvalidStart
            } else {
                DecodeFailure::InvalidPart
            });
        }

        code_points.push(code_point);
        saw_escape |= escaped;
        offset = end;
        element_index += 1;
    }

    if !saw_escape {
        return Err(DecodeFailure::MissingEscape);
    }

    let mut string_value = String::new();
    string_value
        .try_reserve(code_points.len())
        .expect("test oracle allocation must succeed for bounded fixtures");
    for code_point in &code_points {
        let scalar = char::from_u32(*code_point)
            .expect("position-valid identifier code point must be a Unicode scalar value");
        string_value.push(scalar);
    }

    if is_unconditionally_reserved_word_for_oracle(&string_value) {
        return Err(DecodeFailure::DecodedReserved);
    }

    Ok(DecodedIdentifier {
        code_points,
        string_value,
    })
}

fn slice(text: &str, range: ByteRange) -> &str {
    text.get(range.start..range.end)
        .unwrap_or_else(|| panic!("range {range:?} must be a valid UTF-8 range in {text:?}"))
}

fn is_selected_trivia_for_oracle(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{0020}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'
            | '\u{2001}'
            | '\u{2002}'
            | '\u{2003}'
            | '\u{2004}'
            | '\u{2005}'
            | '\u{2006}'
            | '\u{2007}'
            | '\u{2008}'
            | '\u{2009}'
            | '\u{200A}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
            | '\u{FEFF}'
            | '\n'
            | '\r'
            | '\u{2028}'
            | '\u{2029}'
    )
}

fn assert_positive_fixture(index: usize, fixture: &PositiveFixture) {
    assert_eq!(
        slice(fixture.source, fixture.rhs_range),
        fixture.rhs,
        "{}",
        fixture.id
    );
    assert!(
        fixture.rhs.contains('\\'),
        "{} must contain an authored UES",
        fixture.id
    );
    assert_eq!(fixture.route, IdentifierReferenceRoute::EscapedIdentifier);
    assert_eq!(fixture.expected, SELECTED_ACCEPTED_INCOMPLETE);

    let source = SourceText::new(
        SourceId::new(241_000 + index as u64),
        fixture.source.to_owned(),
    );
    let anchor = source
        .anchor(fixture.rhs_range.start, fixture.rhs_range.end)
        .unwrap_or_else(|error| panic!("{} must have a valid RHS anchor: {error}", fixture.id));
    assert_eq!(anchor.fragment(), fixture.rhs, "{}", fixture.id);

    let decoded = decode_selected_escaped_identifier(fixture.rhs)
        .unwrap_or_else(|failure| panic!("{} must decode successfully: {failure:?}", fixture.id));
    assert_eq!(
        decoded.code_points.as_slice(),
        fixture.decoded_code_points,
        "{}",
        fixture.id
    );
    assert_eq!(decoded.string_value, fixture.string_value, "{}", fixture.id);

    let trivia = fixture
        .source
        .get(fixture.rhs_range.end..fixture.boundary_offset)
        .unwrap_or_else(|| panic!("{} must retain valid boundary offsets", fixture.id));
    assert!(
        trivia.chars().all(is_selected_trivia_for_oracle),
        "{} has non-selected trivia before its boundary: {trivia:?}",
        fixture.id
    );

    match fixture.boundary {
        BoundaryKind::BindingListComma => assert_eq!(
            fixture.source.as_bytes().get(fixture.boundary_offset),
            Some(&b','),
            "{}",
            fixture.id
        ),
        BoundaryKind::AuthoredSemicolon => assert_eq!(
            fixture.source.as_bytes().get(fixture.boundary_offset),
            Some(&b';'),
            "{}",
            fixture.id
        ),
        BoundaryKind::AutomaticAtEof => assert_eq!(
            fixture.boundary_offset,
            fixture.source.len(),
            "{} must terminate at actual EOF",
            fixture.id
        ),
    }
}

fn escaped_first_ascii_code_point(word: &str) -> String {
    let first = word
        .chars()
        .next()
        .expect("reserved-word fixture must be non-empty");
    let rest = &word[first.len_utf8()..];
    format!(r"\u{:04X}{rest}", first as u32)
}

#[test]
fn frozen_authority_and_validation_only_envelope_are_exact() {
    assert_eq!(ISSUE_ID, 241);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert_eq!(BASE_POSITIVE_FIXTURES.len(), 17);
    assert_eq!(NAME_POLICY_FIXTURES.len(), 12);
    assert_eq!(NEGATIVE_UNSUPPORTED_EXPECTATION.fixtures.len(), 16);
    assert_eq!(DECODED_RESERVED_UNSUPPORTED_EXPECTATION.fixtures.len(), 36);
    assert_eq!(UNSUPPORTED_TAIL_EXPECTATION.fixtures.len(), 9);
}

#[test]
fn escaped_rhs_positive_fixtures_pin_authored_decoded_and_boundary_facts() {
    for (index, fixture) in BASE_POSITIVE_FIXTURES.iter().enumerate() {
        assert_positive_fixture(index, fixture);
        assert_eq!(fixture.category, NamePolicyCategory::Ordinary);
    }
}

#[test]
fn escaped_rhs_name_policy_matrix_is_position_specific_and_selected_positive() {
    let mut strict_only = 0;
    let mut parameter_special = 0;
    let mut binding_contrast = 0;

    for (index, fixture) in NAME_POLICY_FIXTURES.iter().enumerate() {
        assert_positive_fixture(100 + index, fixture);
        match fixture.category {
            NamePolicyCategory::StrictOnlyInSelectedNonStrictScript => strict_only += 1,
            NamePolicyCategory::YieldAwaitViaEscapedIdentifier => parameter_special += 1,
            NamePolicyCategory::BindingPolicyContrast => binding_contrast += 1,
            NamePolicyCategory::Ordinary => {
                panic!("name-policy table must not contain ordinary rows")
            }
        }
    }

    assert_eq!(strict_only, 8);
    assert_eq!(parameter_special, 2);
    assert_eq!(binding_contrast, 2);
}

#[test]
fn direct_yield_await_alternatives_are_not_the_escaped_identifier_route() {
    assert_eq!(ROUTE_CONTRASTS.len(), 2);

    for fixture in ROUTE_CONTRASTS {
        let decoded = decode_selected_escaped_identifier(fixture.escaped_rhs)
            .expect("escaped route fixture must be position-valid");
        assert_eq!(decoded.string_value, fixture.expected_string_value);
        assert_eq!(fixture.direct_spelling, fixture.expected_string_value);
        assert_eq!(
            fixture.escaped_route,
            IdentifierReferenceRoute::EscapedIdentifier
        );
        assert_ne!(fixture.direct_route, fixture.escaped_route);

        match fixture.direct_spelling {
            "yield" => assert_eq!(
                fixture.direct_route,
                IdentifierReferenceRoute::DirectYieldAlternative
            ),
            "await" => assert_eq!(
                fixture.direct_route,
                IdentifierReferenceRoute::DirectAwaitAlternative
            ),
            other => panic!("unexpected direct parameter-special spelling {other}"),
        }
    }
}

#[test]
fn direct_only_identifier_reference_controls_are_not_redefined_by_this_oracle() {
    for spelling in DIRECT_ONLY_CONTROLS {
        assert_eq!(
            decode_selected_escaped_identifier(spelling),
            Err(DecodeFailure::MissingEscape),
            "{spelling}"
        );
    }
}

#[test]
fn ues_formation_precedes_position_validation_and_surrogates_never_become_chars_first() {
    assert_eq!(
        formed_escape_at(r"\uD800", 0),
        Ok(EscapeFormation {
            end: 6,
            code_point: 0xD800,
        })
    );
    assert_eq!(
        formed_escape_at(r"\u{D800}", 0),
        Ok(EscapeFormation {
            end: 8,
            code_point: 0xD800,
        })
    );
    assert!(!is_selected_identifier_start_for_oracle(0xD800));
    assert_eq!(
        decode_selected_escaped_identifier(r"\uD800"),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{D800}"),
        Err(DecodeFailure::InvalidStart)
    );
}

#[test]
fn identifier_position_additions_and_unicode_membership_are_composed_explicitly() {
    assert!(is_selected_identifier_start_for_oracle('$' as u32));
    assert!(is_selected_identifier_start_for_oracle('_' as u32));
    assert!(!is_id_start('$' as u32));
    assert!(!is_id_start('_' as u32));

    assert!(!is_selected_identifier_start_for_oracle('0' as u32));
    assert!(is_selected_identifier_part_for_oracle('0' as u32));
    assert!(!is_selected_identifier_start_for_oracle(0x301));
    assert!(is_selected_identifier_part_for_oracle(0x301));
    assert!(!is_selected_identifier_start_for_oracle(0x200C));
    assert!(is_selected_identifier_part_for_oracle(0x200C));
    assert!(!is_selected_identifier_start_for_oracle(0x200D));
    assert!(is_selected_identifier_part_for_oracle(0x200D));
    assert!(is_selected_identifier_start_for_oracle(0x1D49C));
    assert!(!is_selected_identifier_part_for_oracle(0xFEFF));
}

#[test]
fn malformed_non_code_point_and_invalid_position_controls_remain_outside_positive_family() {
    assert_eq!(
        NEGATIVE_UNSUPPORTED_EXPECTATION.disposition,
        FutureUnsupportedDisposition::UnsupportedCoverage
    );

    for fixture in NEGATIVE_UNSUPPORTED_EXPECTATION.fixtures {
        assert_eq!(
            decode_selected_escaped_identifier(fixture.rhs),
            Err(fixture.expected),
            "{}",
            fixture.id
        );
    }
}

#[test]
fn every_unconditionally_reserved_decoded_name_is_a_negative_control() {
    assert_eq!(
        DECODED_RESERVED_UNSUPPORTED_EXPECTATION.disposition,
        FutureUnsupportedDisposition::UnsupportedCoverage
    );
    assert_eq!(DECODED_RESERVED_UNSUPPORTED_EXPECTATION.fixtures.len(), 36);

    for word in DECODED_RESERVED_UNSUPPORTED_EXPECTATION.fixtures {
        let escaped = escaped_first_ascii_code_point(word);
        assert_eq!(
            decode_selected_escaped_identifier(&escaped),
            Err(DecodeFailure::DecodedReserved),
            "escaped spelling {escaped:?} must decode to reserved word {word:?}"
        );
    }

    for allowed in ["yield", "await", "let", "static", "eval", "arguments"] {
        assert!(!is_unconditionally_reserved_word_for_oracle(allowed));
    }
}

#[test]
fn canonical_equivalents_are_not_normalized_or_conflated() {
    let precomposed = decode_selected_escaped_identifier(r"\u00E9")
        .expect("precomposed escaped identifier must be accepted");
    let decomposed = decode_selected_escaped_identifier(r"e\u0301")
        .expect("decomposed escaped identifier must be accepted");

    assert_eq!(precomposed.code_points, [0xE9]);
    assert_eq!(decomposed.code_points, [0x65, 0x301]);
    assert_eq!(precomposed.string_value, "é");
    assert_eq!(decomposed.string_value, "e\u{0301}");
    assert_ne!(precomposed, decomposed);
}

#[test]
fn existing_static_subjects_remain_binding_owned_when_escaped_rhs_becomes_reachable() {
    for (index, fixture) in STATIC_REACHABILITY_FIXTURES.iter().enumerate() {
        let rhs = slice(fixture.source, fixture.rhs_range);
        let _decoded = decode_selected_escaped_identifier(rhs).unwrap_or_else(|failure| {
            panic!("{} RHS must be selected-positive: {failure:?}", fixture.id)
        });

        assert_eq!(
            slice(fixture.source, fixture.subject),
            fixture.subject_fragment,
            "{}",
            fixture.id
        );
        assert!(fixture.rule_id.starts_with("EE-"), "{}", fixture.id);

        let source = SourceText::new(
            SourceId::new(241_300 + index as u64),
            fixture.source.to_owned(),
        );
        let subject = source
            .anchor(fixture.subject.start, fixture.subject.end)
            .expect("static subject must be a valid authored anchor");
        assert_eq!(
            subject.fragment(),
            fixture.subject_fragment,
            "{}",
            fixture.id
        );

        let control_source = gold_source(fixture.control_gold_id)
            .unwrap_or_else(|| panic!("{} control gold must remain available", fixture.id));
        let control_range = gold_subject_range(fixture.control_gold_id)
            .unwrap_or_else(|| panic!("{} control gold must remain source-backed", fixture.id));
        assert!(
            !control_source
                .get(control_range.0..control_range.1)
                .expect("control gold subject must be UTF-8 valid")
                .is_empty()
        );
    }
}

#[test]
fn later_existing_grammar_subjects_remain_exact_and_terminal_after_valid_escaped_rhs() {
    for (index, fixture) in GRAMMAR_REACHABILITY_FIXTURES.iter().enumerate() {
        let rhs = slice(fixture.source, fixture.rhs_range);
        let _decoded = decode_selected_escaped_identifier(rhs).unwrap_or_else(|failure| {
            panic!("{} RHS must be selected-positive: {failure:?}", fixture.id)
        });
        assert_eq!(
            slice(fixture.source, fixture.subject),
            fixture.subject_fragment,
            "{}",
            fixture.id
        );

        let source = SourceText::new(
            SourceId::new(241_400 + index as u64),
            fixture.source.to_owned(),
        );
        let subject = source
            .anchor(fixture.subject.start, fixture.subject.end)
            .expect("existing Grammar subject must remain source-backed");
        assert_eq!(
            subject.fragment(),
            fixture.subject_fragment,
            "{}",
            fixture.id
        );
    }
}

#[test]
fn valid_escaped_atom_plus_unsupported_tail_never_becomes_whole_source_success() {
    assert_eq!(
        UNSUPPORTED_TAIL_EXPECTATION.disposition,
        FutureUnsupportedDisposition::UnsupportedCoverage
    );
    assert_eq!(UNSUPPORTED_TAIL_EXPECTATION.fixtures.len(), 9);

    for fixture in UNSUPPORTED_TAIL_EXPECTATION.fixtures {
        let rhs = slice(fixture.source, fixture.rhs_range);
        let decoded = decode_selected_escaped_identifier(rhs).unwrap_or_else(|failure| {
            panic!(
                "{} prefix must be a valid escaped atom: {failure:?}",
                fixture.id
            )
        });
        assert_eq!(decoded.string_value, "foo", "{}", fixture.id);

        let tail = fixture
            .source
            .get(fixture.rhs_range.end..)
            .expect("tail range must be valid");
        match fixture.tail_kind {
            UnsupportedTailKind::Member => assert!(tail.starts_with('.')),
            UnsupportedTailKind::Call => assert!(tail.starts_with("()")),
            UnsupportedTailKind::Binary => assert!(tail.starts_with(" + ")),
            UnsupportedTailKind::Assignment => assert!(tail.starts_with(" = ")),
            UnsupportedTailKind::Conditional => assert!(tail.starts_with(" ? ")),
            UnsupportedTailKind::Comment => assert!(tail.starts_with("/*")),
            UnsupportedTailKind::UnexpectedTail => assert!(tail.starts_with(" unexpected")),
            UnsupportedTailKind::ExtraSemicolon => assert!(tail.starts_with(";;")),
            UnsupportedTailKind::RequiresNonEofAsi => assert!(tail.starts_with('\n')),
        }
    }
}

#[test]
fn long_leading_zero_and_long_mixed_utf8_validation_is_deterministic() {
    let digits = format!("{}66", "0".repeat(4096));
    let suffix = "α".repeat(4096);
    let rhs = format!("\\u{{{digits}}}{suffix}");

    let first = decode_selected_escaped_identifier(&rhs)
        .expect("long leading-zero escaped identifier must be accepted");
    let second = decode_selected_escaped_identifier(&rhs)
        .expect("repeated validation must be deterministic");

    assert_eq!(first, second);
    assert_eq!(first.code_points.first(), Some(&0x66));
    assert_eq!(first.code_points.len(), 4097);
    assert!(first.string_value.starts_with('f'));
    assert_eq!(first.string_value.chars().count(), 4097);
}

#[test]
fn validation_source_has_no_dependency_on_production_or_runtime_reference_paths() {
    for forbidden in [
        ["selected_", "lexical_slice"].concat(),
        ["parse_selected_", "binding_identifier"].concat(),
        ["recognize_selected_", "lexical_slice"].concat(),
        ["selected_qualification_", "integration"].concat(),
        ["SelectedBinding", "NameState"].concat(),
        ["Resolve", "Binding"].concat(),
        ["Get", "Value"].concat(),
        ["Reference", "Record"].concat(),
    ] {
        assert!(
            !THIS_SOURCE.contains(&forbidden),
            "validation oracle must not depend on production/runtime path {forbidden}"
        );
    }
}

#[test]
fn validation_handoff_remains_one_focused_future_production_frontier() {
    for fixture in BASE_POSITIVE_FIXTURES.iter().chain(NAME_POLICY_FIXTURES) {
        assert_eq!(
            fixture.expected.static_semantics,
            ExpectedStaticSemantics::Accepted
        );
        assert_eq!(
            fixture.expected.lifecycle,
            FutureLifecycle::SelectedAcceptedIncomplete
        );
        assert_eq!(fixture.route, IdentifierReferenceRoute::EscapedIdentifier);
    }

    assert_eq!(
        NEGATIVE_UNSUPPORTED_EXPECTATION.disposition,
        FutureUnsupportedDisposition::UnsupportedCoverage
    );
    assert_eq!(
        DECODED_RESERVED_UNSUPPORTED_EXPECTATION.disposition,
        FutureUnsupportedDisposition::UnsupportedCoverage
    );
    assert_eq!(
        UNSUPPORTED_TAIL_EXPECTATION.disposition,
        FutureUnsupportedDisposition::UnsupportedCoverage
    );

    assert!(
        NEGATIVE_UNSUPPORTED_EXPECTATION
            .fixtures
            .iter()
            .any(|fixture| { fixture.expected == DecodeFailure::MalformedEscape })
    );
    assert!(
        NEGATIVE_UNSUPPORTED_EXPECTATION
            .fixtures
            .iter()
            .any(|fixture| { fixture.expected == DecodeFailure::NonCodePoint })
    );
    assert!(
        NEGATIVE_UNSUPPORTED_EXPECTATION
            .fixtures
            .iter()
            .any(|fixture| { fixture.expected == DecodeFailure::InvalidStart })
    );
    assert!(
        NEGATIVE_UNSUPPORTED_EXPECTATION
            .fixtures
            .iter()
            .any(|fixture| { fixture.expected == DecodeFailure::InvalidPart })
    );
}