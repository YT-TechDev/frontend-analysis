//! Candidate-independent validation for Issue #237.
//!
//! This module fixes the selected, escape-free `IdentifierReference` initializer
//! boundary without calling production lexical, static-semantics, aggregate, or
//! future expression code. It consumes the frozen Unicode authority directly and
//! records selected lifecycle expectations independently of production.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::{gold_source, gold_subject_range};
use super::unicode::{is_id_continue, is_id_start};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 237;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const IDENTIFIER_INITIALIZER_GOLD_ID: &str = "JS-GOLD-LEXDECL-CONST-IDENTIFIER-INIT-001";
const GOLD_SOURCE: &str = include_str!("qualification_validation_tests/gold.rs");
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const THIS_SOURCE: &str =
    include_str!("qualification_selected_identifier_reference_initializer_validation_tests.rs");

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
enum ExpectedStaticSemantics {
    Accepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FutureLifecycle {
    SelectedAcceptedIncomplete,
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
struct RhsFixture {
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    rhs_range: ByteRange,
    boundary_offset: usize,
    boundary: BoundaryKind,
    expected: FutureSelectedExpectation,
}

const fn rhs_fixture(
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    rhs_range: ByteRange,
    boundary_offset: usize,
    boundary: BoundaryKind,
) -> RhsFixture {
    RhsFixture {
        id,
        source,
        rhs,
        rhs_range,
        boundary_offset,
        boundary,
        expected: SELECTED_ACCEPTED_INCOMPLETE,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamePolicyCategory {
    StrictOnlyRestricted,
    YieldAwaitParameterSpecial,
    BindingContrast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NamePolicyFixture {
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    rhs_range: ByteRange,
    category: NamePolicyCategory,
    expected: FutureSelectedExpectation,
}

const fn name_fixture(
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    rhs_range: ByteRange,
    category: NamePolicyCategory,
) -> NamePolicyFixture {
    NamePolicyFixture {
        id,
        source,
        rhs,
        rhs_range,
        category,
        expected: SELECTED_ACCEPTED_INCOMPLETE,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StaticCompositionFixture {
    id: &'static str,
    source: &'static str,
    rule_id: &'static str,
    subject: ByteRange,
    subject_fragment: &'static str,
    control_gold_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GrammarControlFixture {
    id: &'static str,
    source: &'static str,
    subject: ByteRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedReason {
    EscapedIdentifierReference,
    ReservedToken,
    MemberExpression,
    CallExpression,
    BinaryExpression,
    ParenthesizedExpression,
    AssignmentExpression,
    ConditionalExpression,
    MissingRightHandSide,
    Comment,
    RequiresNonEofAsi,
    UnexpectedTail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UnsupportedFixture {
    id: &'static str,
    source: &'static str,
    reason: UnsupportedReason,
}

const RHS_FIXTURES: &[RhsFixture] = &[
    rhs_fixture(
        "IDREF-INIT-POSITIVE-SEMICOLON-001",
        "const x = foo;",
        "foo",
        ByteRange::new(10, 13),
        13,
        BoundaryKind::AuthoredSemicolon,
    ),
    rhs_fixture(
        "IDREF-INIT-POSITIVE-COMMA-001",
        "let x = foo, y = bar;",
        "foo",
        ByteRange::new(8, 11),
        11,
        BoundaryKind::BindingListComma,
    ),
    rhs_fixture(
        "IDREF-INIT-POSITIVE-COMMA-002",
        "let x = foo, y = bar;",
        "bar",
        ByteRange::new(17, 20),
        20,
        BoundaryKind::AuthoredSemicolon,
    ),
    rhs_fixture(
        "IDREF-INIT-POSITIVE-EOF-001",
        "const x = foo",
        "foo",
        ByteRange::new(10, 13),
        13,
        BoundaryKind::AutomaticAtEof,
    ),
    rhs_fixture(
        "IDREF-INIT-POSITIVE-MULTIDECL-001",
        "let x = 1; const y = foo",
        "foo",
        ByteRange::new(21, 24),
        24,
        BoundaryKind::AutomaticAtEof,
    ),
    rhs_fixture(
        "IDREF-INIT-POSITIVE-UNICODE-001",
        "const π = 𝒜;",
        "𝒜",
        ByteRange::new(11, 15),
        15,
        BoundaryKind::AuthoredSemicolon,
    ),
    rhs_fixture(
        "IDREF-INIT-POSITIVE-ESCAPED-BINDING-COMPOSITION-001",
        r"const \u0078 = foo;",
        "foo",
        ByteRange::new(15, 18),
        18,
        BoundaryKind::AuthoredSemicolon,
    ),
    rhs_fixture(
        "IDREF-INIT-POSITIVE-TRIVIA-BEFORE-SEMICOLON-001",
        "const x = foo \t\n;",
        "foo",
        ByteRange::new(10, 13),
        16,
        BoundaryKind::AuthoredSemicolon,
    ),
    rhs_fixture(
        "IDREF-INIT-DIRECT-DOLLAR-001",
        "const x = $;",
        "$",
        ByteRange::new(10, 11),
        11,
        BoundaryKind::AuthoredSemicolon,
    ),
    rhs_fixture(
        "IDREF-INIT-DIRECT-LOW-LINE-001",
        "const x = _;",
        "_",
        ByteRange::new(10, 11),
        11,
        BoundaryKind::AuthoredSemicolon,
    ),
    rhs_fixture(
        "IDREF-INIT-DIRECT-DIGIT-PART-001",
        "const x = a0;",
        "a0",
        ByteRange::new(10, 12),
        12,
        BoundaryKind::AuthoredSemicolon,
    ),
    rhs_fixture(
        "IDREF-INIT-DIRECT-COMBINING-PART-001",
        "const x = a\u{0301};",
        "a\u{0301}",
        ByteRange::new(10, 13),
        13,
        BoundaryKind::AuthoredSemicolon,
    ),
    rhs_fixture(
        "IDREF-INIT-DIRECT-ZWNJ-PART-001",
        "const x = a\u{200C}b;",
        "a\u{200C}b",
        ByteRange::new(10, 15),
        15,
        BoundaryKind::AuthoredSemicolon,
    ),
    rhs_fixture(
        "IDREF-INIT-DIRECT-ZWJ-PART-001",
        "const x = a\u{200D}b;",
        "a\u{200D}b",
        ByteRange::new(10, 15),
        15,
        BoundaryKind::AuthoredSemicolon,
    ),
];

const NAME_POLICY_FIXTURES: &[NamePolicyFixture] = &[
    name_fixture(
        "IDREF-NAME-STRICT-ONLY-LET",
        "const x = let;",
        "let",
        ByteRange::new(10, 13),
        NamePolicyCategory::StrictOnlyRestricted,
    ),
    name_fixture(
        "IDREF-NAME-STRICT-ONLY-STATIC",
        "const x = static;",
        "static",
        ByteRange::new(10, 16),
        NamePolicyCategory::StrictOnlyRestricted,
    ),
    name_fixture(
        "IDREF-NAME-STRICT-ONLY-IMPLEMENTS",
        "const x = implements;",
        "implements",
        ByteRange::new(10, 20),
        NamePolicyCategory::StrictOnlyRestricted,
    ),
    name_fixture(
        "IDREF-NAME-STRICT-ONLY-INTERFACE",
        "const x = interface;",
        "interface",
        ByteRange::new(10, 19),
        NamePolicyCategory::StrictOnlyRestricted,
    ),
    name_fixture(
        "IDREF-NAME-STRICT-ONLY-PACKAGE",
        "const x = package;",
        "package",
        ByteRange::new(10, 17),
        NamePolicyCategory::StrictOnlyRestricted,
    ),
    name_fixture(
        "IDREF-NAME-STRICT-ONLY-PRIVATE",
        "const x = private;",
        "private",
        ByteRange::new(10, 17),
        NamePolicyCategory::StrictOnlyRestricted,
    ),
    name_fixture(
        "IDREF-NAME-STRICT-ONLY-PROTECTED",
        "const x = protected;",
        "protected",
        ByteRange::new(10, 19),
        NamePolicyCategory::StrictOnlyRestricted,
    ),
    name_fixture(
        "IDREF-NAME-STRICT-ONLY-PUBLIC",
        "const x = public;",
        "public",
        ByteRange::new(10, 16),
        NamePolicyCategory::StrictOnlyRestricted,
    ),
    name_fixture(
        "IDREF-NAME-PARAMETER-SPECIAL-YIELD",
        "const x = yield;",
        "yield",
        ByteRange::new(10, 15),
        NamePolicyCategory::YieldAwaitParameterSpecial,
    ),
    name_fixture(
        "IDREF-NAME-PARAMETER-SPECIAL-AWAIT",
        "const x = await;",
        "await",
        ByteRange::new(10, 15),
        NamePolicyCategory::YieldAwaitParameterSpecial,
    ),
    name_fixture(
        "IDREF-NAME-BINDING-CONTRAST-EVAL",
        "const x = eval;",
        "eval",
        ByteRange::new(10, 14),
        NamePolicyCategory::BindingContrast,
    ),
    name_fixture(
        "IDREF-NAME-BINDING-CONTRAST-ARGUMENTS",
        "const x = arguments;",
        "arguments",
        ByteRange::new(10, 19),
        NamePolicyCategory::BindingContrast,
    ),
];

const UNCONDITIONALLY_RESERVED_RHS: &[&str] = &[
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

const STATIC_COMPOSITION_FIXTURES: &[StaticCompositionFixture] = &[
    StaticCompositionFixture {
        id: "IDREF-STATIC-R01-001",
        source: "let let = foo;",
        rule_id: "EE-15-R01",
        subject: ByteRange::new(4, 7),
        subject_fragment: "let",
        control_gold_id: "JS-GOLD-LEXDECL-LET-BINDING-001",
    },
    StaticCompositionFixture {
        id: "IDREF-STATIC-R02-001",
        source: "let x = foo, x = bar;",
        rule_id: "EE-15-R02",
        subject: ByteRange::new(13, 14),
        subject_fragment: "x",
        control_gold_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
    },
    StaticCompositionFixture {
        id: "IDREF-STATIC-EE36-001",
        source: "let x = foo; let x = bar;",
        rule_id: "EE-36-R01",
        subject: ByteRange::new(17, 18),
        subject_fragment: "x",
        control_gold_id: "JS-GOLD-SCRIPT-DUPLEXICAL-001",
    },
    StaticCompositionFixture {
        id: "IDREF-STATIC-R01-RHS-LET-CONTRAST-001",
        source: r"let \u006Cet = let;",
        rule_id: "EE-15-R01",
        subject: ByteRange::new(4, 12),
        subject_fragment: r"\u006Cet",
        control_gold_id: "JS-GOLD-LEXDECL-ESCAPED-LET-BINDING-001",
    },
];

const GRAMMAR_CONTROLS: &[GrammarControlFixture] = &[
    GrammarControlFixture {
        id: "IDREF-GRAMMAR-LATER-EMPTY-BRACED-001",
        source: r"const x = foo; let \u{};",
        subject: ByteRange::new(19, 23),
    },
    GrammarControlFixture {
        id: "IDREF-GRAMMAR-LATER-PART-EMPTY-BRACED-001",
        source: r"const x = foo; let a\u{};",
        subject: ByteRange::new(20, 24),
    },
    GrammarControlFixture {
        id: "IDREF-GRAMMAR-LATER-UNCLOSED-BRACED-001",
        source: r"const x = foo; let \u{61",
        subject: ByteRange::new(19, 24),
    },
];

const UNSUPPORTED_FIXTURES: &[UnsupportedFixture] = &[
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-ESCAPED-RHS-001",
        source: r"const x = \u0066oo;",
        reason: UnsupportedReason::EscapedIdentifierReference,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-RESERVED-TOKEN-001",
        source: "const x = if;",
        reason: UnsupportedReason::ReservedToken,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-MEMBER-001",
        source: "const x = foo.bar;",
        reason: UnsupportedReason::MemberExpression,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-CALL-001",
        source: "const x = foo();",
        reason: UnsupportedReason::CallExpression,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-BINARY-001",
        source: "const x = foo + 1;",
        reason: UnsupportedReason::BinaryExpression,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-GROUPING-001",
        source: "const x = (foo);",
        reason: UnsupportedReason::ParenthesizedExpression,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-ASSIGNMENT-001",
        source: "const x = foo = bar;",
        reason: UnsupportedReason::AssignmentExpression,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-CONDITIONAL-001",
        source: "const x = foo ? bar : baz;",
        reason: UnsupportedReason::ConditionalExpression,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-MISSING-RHS-001",
        source: "const x = ;",
        reason: UnsupportedReason::MissingRightHandSide,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-COMMENT-001",
        source: "const x = foo/*comment*/;",
        reason: UnsupportedReason::Comment,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-NON-EOF-ASI-001",
        source: "let x = foo\nconst y = bar;",
        reason: UnsupportedReason::RequiresNonEofAsi,
    },
    UnsupportedFixture {
        id: "IDREF-UNSUPPORTED-TAIL-001",
        source: "const x = foo unexpected;",
        reason: UnsupportedReason::UnexpectedTail,
    },
];

fn fixture_block<'a>(source: &'a str, fixture_id: &str) -> &'a str {
    let marker = format!("id: \"{fixture_id}\"");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("fixture {fixture_id} must remain in frozen authority"));
    let rest = &source[start..];
    let end = rest
        .find("\n        },")
        .unwrap_or_else(|| panic!("fixture {fixture_id} must retain a complete block"))
        + "\n        },".len();
    &rest[..end]
}

fn slice(text: &str, range: ByteRange) -> &str {
    text.get(range.start..range.end)
        .unwrap_or_else(|| panic!("range {range:?} must be a UTF-8 boundary in {text:?}"))
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

fn is_direct_identifier_start(code_point: char) -> bool {
    matches!(code_point, '$' | '_') || is_id_start(code_point as u32)
}

fn is_direct_identifier_part(code_point: char) -> bool {
    code_point == '$' || is_id_continue(code_point as u32)
}

fn is_escape_free_identifier_name(spelling: &str) -> bool {
    let mut code_points = spelling.chars();
    let Some(first) = code_points.next() else {
        return false;
    };

    is_direct_identifier_start(first) && code_points.all(is_direct_identifier_part)
}

#[test]
fn frozen_authority_and_existing_identifier_initializer_gold_are_exact() {
    assert_eq!(ISSUE_ID, 237);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert_eq!(
        gold_source(IDENTIFIER_INITIALIZER_GOLD_ID),
        Some("const x = foo;")
    );

    let block = fixture_block(GOLD_SOURCE, IDENTIFIER_INITIALIZER_GOLD_ID);
    assert!(block.contains("qualification: Some(ExpectedQualification::Qualified)"));
    assert!(block.contains("processing: ExpectedProcessing::Complete"));
    assert!(block.contains("implementation_coverage: ImplementationCoverage::Pending"));
    assert!(block.contains("synthetic: NO_SYNTHETIC"));
    assert_eq!(gold_subject_range(IDENTIFIER_INITIALIZER_GOLD_ID), None);
}

#[test]
fn direct_code_point_family_consumes_frozen_unicode_17_authority() {
    for code_point in ['A', 'π', '𝒜'] {
        assert!(is_id_start(code_point as u32));
        assert!(is_direct_identifier_start(code_point));
        assert!(is_direct_identifier_part(code_point));
    }

    assert!(!is_id_start('$' as u32));
    assert!(!is_id_continue('$' as u32));
    assert!(is_direct_identifier_start('$'));
    assert!(is_direct_identifier_part('$'));

    assert!(!is_id_start('_' as u32));
    assert!(is_id_continue('_' as u32));
    assert!(is_direct_identifier_start('_'));
    assert!(is_direct_identifier_part('_'));

    assert!(!is_direct_identifier_start('0'));
    assert!(is_id_continue('0' as u32));
    assert!(is_direct_identifier_part('0'));

    assert!(!is_direct_identifier_start('\u{0301}'));
    assert!(is_id_continue('\u{0301}' as u32));
    assert!(is_direct_identifier_part('\u{0301}'));

    for code_point in ['\u{200C}', '\u{200D}'] {
        assert!(!is_id_start(code_point as u32));
        assert!(is_id_continue(code_point as u32));
        assert!(!is_direct_identifier_start(code_point));
        assert!(is_direct_identifier_part(code_point));
    }

    for code_point in ['-', '\u{FEFF}'] {
        assert!(!is_direct_identifier_start(code_point));
        assert!(!is_direct_identifier_part(code_point));
    }
}

#[test]
fn escape_free_identifier_name_shape_is_exact_and_not_a_reserved_word_policy() {
    for spelling in [
        "$",
        "_",
        "$x",
        "a0",
        "π",
        "𝒜",
        "a\u{0301}",
        "a\u{200C}b",
        "a\u{200D}b",
        "if",
    ] {
        assert!(
            is_escape_free_identifier_name(spelling),
            "{spelling:?} must match the direct IdentifierName code-point family"
        );
    }

    for spelling in ["", "0a", "-a", "a-b", "\u{200C}a", "\u{200D}a"] {
        assert!(!is_escape_free_identifier_name(spelling), "{spelling:?}");
    }

    assert!(UNCONDITIONALLY_RESERVED_RHS.contains(&"if"));
    assert!(!UNCONDITIONALLY_RESERVED_RHS.contains(&"yield"));
    assert!(!UNCONDITIONALLY_RESERVED_RHS.contains(&"await"));
    for spelling in UNCONDITIONALLY_RESERVED_RHS {
        assert!(is_escape_free_identifier_name(spelling));
    }
}

#[test]
fn rhs_fixtures_pin_authored_ranges_boundaries_and_future_selected_lifecycle() {
    for (index, fixture) in RHS_FIXTURES.iter().enumerate() {
        assert_eq!(
            slice(fixture.source, fixture.rhs_range),
            fixture.rhs,
            "{}",
            fixture.id
        );
        assert!(is_escape_free_identifier_name(fixture.rhs), "{}", fixture.id);
        assert_eq!(fixture.expected, SELECTED_ACCEPTED_INCOMPLETE, "{}", fixture.id);

        let source = SourceText::new(
            SourceId::new(237_000 + index as u64),
            fixture.source.to_owned(),
        );
        let anchor = source
            .anchor(fixture.rhs_range.start, fixture.rhs_range.end)
            .unwrap_or_else(|error| {
                panic!(
                    "{} must have a valid authored RHS anchor: {error}",
                    fixture.id
                )
            });
        assert_eq!(anchor.fragment(), fixture.rhs, "{}", fixture.id);

        let trivia = fixture
            .source
            .get(fixture.rhs_range.end..fixture.boundary_offset)
            .unwrap_or_else(|| panic!("{} must have valid boundary offsets", fixture.id));
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
                "{} must reach actual EOF",
                fixture.id
            ),
        }
    }
}

#[test]
fn name_policy_matrix_pins_three_categories_and_selected_positive_disposition() {
    let mut strict_only = Vec::new();
    let mut parameter_special = Vec::new();
    let mut binding_contrast = Vec::new();

    for (index, fixture) in NAME_POLICY_FIXTURES.iter().enumerate() {
        assert_eq!(slice(fixture.source, fixture.rhs_range), fixture.rhs);
        assert!(is_escape_free_identifier_name(fixture.rhs), "{}", fixture.id);
        assert_eq!(fixture.expected, SELECTED_ACCEPTED_INCOMPLETE, "{}", fixture.id);

        let source = SourceText::new(
            SourceId::new(237_100 + index as u64),
            fixture.source.to_owned(),
        );
        assert_eq!(
            source
                .anchor(fixture.rhs_range.start, fixture.rhs_range.end)
                .expect("name-policy RHS must have a valid UTF-8 range")
                .fragment(),
            fixture.rhs
        );

        match fixture.category {
            NamePolicyCategory::StrictOnlyRestricted => strict_only.push(fixture.rhs),
            NamePolicyCategory::YieldAwaitParameterSpecial => parameter_special.push(fixture.rhs),
            NamePolicyCategory::BindingContrast => binding_contrast.push(fixture.rhs),
        }
    }

    assert_eq!(
        strict_only,
        [
            "let",
            "static",
            "implements",
            "interface",
            "package",
            "private",
            "protected",
            "public"
        ]
    );
    assert_eq!(parameter_special, ["yield", "await"]);
    assert_eq!(binding_contrast, ["eval", "arguments"]);
}

#[test]
fn selected_positive_expectation_cannot_be_confused_with_whole_standard_qualified() {
    assert_eq!(
        SELECTED_ACCEPTED_INCOMPLETE.static_semantics,
        ExpectedStaticSemantics::Accepted
    );
    assert_eq!(
        SELECTED_ACCEPTED_INCOMPLETE.lifecycle,
        FutureLifecycle::SelectedAcceptedIncomplete
    );

    for fixture in RHS_FIXTURES {
        assert_eq!(fixture.expected, SELECTED_ACCEPTED_INCOMPLETE);
    }
    for fixture in NAME_POLICY_FIXTURES {
        assert_eq!(fixture.expected, SELECTED_ACCEPTED_INCOMPLETE);
    }
}

#[test]
fn direct_unicode_rhs_provenance_is_utf8_exact_and_not_normalized() {
    let fixture = RHS_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "IDREF-INIT-POSITIVE-UNICODE-001")
        .expect("Unicode RHS fixture must remain present");

    assert_eq!(fixture.rhs, "𝒜");
    assert_eq!(fixture.rhs_range, ByteRange::new(11, 15));
    assert_eq!(fixture.rhs_range.end - fixture.rhs_range.start, "𝒜".len());

    let source = SourceText::new(SourceId::new(237_999), fixture.source.to_owned());
    let anchor = source
        .anchor(fixture.rhs_range.start, fixture.rhs_range.end)
        .expect("supplementary-plane RHS must remain a valid UTF-8 anchor");
    assert_eq!(anchor.fragment(), "𝒜");
    assert_ne!(anchor.fragment(), "A");
}

#[test]
fn static_composition_keeps_existing_binding_subjects_primary() {
    for fixture in STATIC_COMPOSITION_FIXTURES {
        assert_eq!(slice(fixture.source, fixture.subject), fixture.subject_fragment);
        assert!(fixture.rule_id.starts_with("EE-"), "{}", fixture.id);

        let control_source = gold_source(fixture.control_gold_id)
            .unwrap_or_else(|| panic!("{} control gold must remain present", fixture.id));
        let control_range = gold_subject_range(fixture.control_gold_id)
            .unwrap_or_else(|| panic!("{} control gold must retain a subject", fixture.id));
        assert_eq!(
            control_source
                .get(control_range.0..control_range.1)
                .expect("control gold subject must remain UTF-8 valid"),
            fixture.subject_fragment,
            "{} must preserve the same authored binding subject meaning",
            fixture.id
        );
    }
}

#[test]
fn valid_rhs_before_later_owned_grammar_evidence_does_not_create_prefix_success() {
    for (index, fixture) in GRAMMAR_CONTROLS.iter().enumerate() {
        let expected_subject = match fixture.id {
            "IDREF-GRAMMAR-LATER-EMPTY-BRACED-001"
            | "IDREF-GRAMMAR-LATER-PART-EMPTY-BRACED-001" => r"\u{}",
            "IDREF-GRAMMAR-LATER-UNCLOSED-BRACED-001" => r"\u{61",
            other => panic!("unrecognized grammar control {other}"),
        };
        assert_eq!(slice(fixture.source, fixture.subject), expected_subject);

        let source = SourceText::new(
            SourceId::new(237_200 + index as u64),
            fixture.source.to_owned(),
        );
        let anchor = source
            .anchor(fixture.subject.start, fixture.subject.end)
            .expect("existing Grammar subject must remain a valid authored anchor");
        assert_eq!(anchor.fragment(), expected_subject);
    }
}

#[test]
fn unsupported_neighbors_are_explicit_and_do_not_gain_a_selected_verdict() {
    assert_eq!(UNSUPPORTED_FIXTURES.len(), 12);

    for fixture in UNSUPPORTED_FIXTURES {
        assert!(!fixture.id.is_empty());
        assert!(!fixture.source.is_empty());

        match fixture.reason {
            UnsupportedReason::EscapedIdentifierReference => {
                assert!(fixture.source.contains(r"\u"))
            }
            UnsupportedReason::ReservedToken => assert!(fixture.source.contains(" = if;")),
            UnsupportedReason::MemberExpression => assert!(fixture.source.contains('.')),
            UnsupportedReason::CallExpression => assert!(fixture.source.contains("()")),
            UnsupportedReason::BinaryExpression => assert!(fixture.source.contains(" + ")),
            UnsupportedReason::ParenthesizedExpression => assert!(fixture.source.contains("(foo)")),
            UnsupportedReason::AssignmentExpression => {
                assert!(fixture.source.contains(" = foo = "))
            }
            UnsupportedReason::ConditionalExpression => assert!(fixture.source.contains(" ? ")),
            UnsupportedReason::MissingRightHandSide => assert!(fixture.source.contains("= ;")),
            UnsupportedReason::Comment => assert!(fixture.source.contains("/*")),
            UnsupportedReason::RequiresNonEofAsi => assert!(fixture.source.contains('\n')),
            UnsupportedReason::UnexpectedTail => assert!(fixture.source.contains(" unexpected")),
        }
    }
}

#[test]
fn deterministic_pathological_utf8_ranges_remain_checked_and_bounded() {
    let identifier = format!("π{}", "α".repeat(4096));
    assert!(is_escape_free_identifier_name(&identifier));
    let source_text = format!("const x = {identifier};");
    let start = "const x = ".len();
    let end = start + identifier.len();

    for source_id in [237_300_u64, 237_301_u64] {
        let source = SourceText::new(SourceId::new(source_id), source_text.clone());
        let anchor = source
            .anchor(start, end)
            .expect("bounded long direct identifier must preserve UTF-8 endpoints");
        assert_eq!(anchor.fragment(), identifier);
        assert_eq!(anchor.range().start(), start);
        assert_eq!(anchor.range().end(), end);
    }
}

#[test]
fn validation_source_has_no_dependency_on_selected_production_or_runtime_evaluation() {
    for forbidden in [
        ["use super::selected_", "lexical_slice"].concat(),
        ["use super::selected_", "static_semantics"].concat(),
        ["use super::selected_", "qualification_integration"].concat(),
        ["recognize_selected_", "lexical_slice"].concat(),
        ["attempt_selected_", "qualification"].concat(),
        ["Resolve", "Binding("].concat(),
        ["Get", "Value("].concat(),
    ] {
        assert!(
            !THIS_SOURCE.contains(&forbidden),
            "validation oracle must not depend on production/runtime path {forbidden}"
        );
    }
}

#[test]
fn validation_contract_handoff_stays_focused_on_one_future_architecture_gate() {
    assert!(RHS_FIXTURES.iter().any(|fixture| {
        fixture.id == "IDREF-INIT-POSITIVE-SEMICOLON-001"
            && fixture.expected == SELECTED_ACCEPTED_INCOMPLETE
    }));
    assert!(RHS_FIXTURES.iter().any(|fixture| {
        fixture.id == "IDREF-INIT-POSITIVE-EOF-001"
            && fixture.expected == SELECTED_ACCEPTED_INCOMPLETE
    }));
    assert!(NAME_POLICY_FIXTURES.iter().all(|fixture| {
        fixture.expected == SELECTED_ACCEPTED_INCOMPLETE
    }));
    assert!(UNSUPPORTED_FIXTURES.iter().any(|fixture| {
        fixture.reason == UnsupportedReason::EscapedIdentifierReference
    }));
    assert!(UNSUPPORTED_FIXTURES.iter().any(|fixture| {
        fixture.reason == UnsupportedReason::RequiresNonEofAsi
    }));
    assert!(UNSUPPORTED_FIXTURES.iter().any(|fixture| {
        fixture.reason == UnsupportedReason::MemberExpression
    }));
}
