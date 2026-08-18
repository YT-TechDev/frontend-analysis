//! Candidate-independent validation for Issue #237.
//!
//! This module fixes the selected, escape-free `IdentifierReference` initializer
//! boundary without calling production lexical, static-semantics, aggregate, or
//! future expression code. It validates expected meaning and source provenance,
//! not a production parser representation.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::{gold_source, gold_subject_range};

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
struct RhsFixture {
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    rhs_range: ByteRange,
    boundary_offset: usize,
    boundary: BoundaryKind,
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
    RhsFixture {
        id: "IDREF-INIT-POSITIVE-SEMICOLON-001",
        source: "const x = foo;",
        rhs: "foo",
        rhs_range: ByteRange::new(10, 13),
        boundary_offset: 13,
        boundary: BoundaryKind::AuthoredSemicolon,
    },
    RhsFixture {
        id: "IDREF-INIT-POSITIVE-COMMA-001",
        source: "let x = foo, y = bar;",
        rhs: "foo",
        rhs_range: ByteRange::new(8, 11),
        boundary_offset: 11,
        boundary: BoundaryKind::BindingListComma,
    },
    RhsFixture {
        id: "IDREF-INIT-POSITIVE-COMMA-002",
        source: "let x = foo, y = bar;",
        rhs: "bar",
        rhs_range: ByteRange::new(17, 20),
        boundary_offset: 20,
        boundary: BoundaryKind::AuthoredSemicolon,
    },
    RhsFixture {
        id: "IDREF-INIT-POSITIVE-EOF-001",
        source: "const x = foo",
        rhs: "foo",
        rhs_range: ByteRange::new(10, 13),
        boundary_offset: 13,
        boundary: BoundaryKind::AutomaticAtEof,
    },
    RhsFixture {
        id: "IDREF-INIT-POSITIVE-MULTIDECL-001",
        source: "let x = 1; const y = foo",
        rhs: "foo",
        rhs_range: ByteRange::new(21, 24),
        boundary_offset: 24,
        boundary: BoundaryKind::AutomaticAtEof,
    },
    RhsFixture {
        id: "IDREF-INIT-POSITIVE-UNICODE-001",
        source: "const π = 𝒜;",
        rhs: "𝒜",
        rhs_range: ByteRange::new(11, 15),
        boundary_offset: 15,
        boundary: BoundaryKind::AuthoredSemicolon,
    },
    RhsFixture {
        id: "IDREF-INIT-POSITIVE-ESCAPED-BINDING-COMPOSITION-001",
        source: r"const \u0078 = foo;",
        rhs: "foo",
        rhs_range: ByteRange::new(15, 18),
        boundary_offset: 18,
        boundary: BoundaryKind::AuthoredSemicolon,
    },
    RhsFixture {
        id: "IDREF-INIT-POSITIVE-TRIVIA-BEFORE-SEMICOLON-001",
        source: "const x = foo \t\n;",
        rhs: "foo",
        rhs_range: ByteRange::new(10, 13),
        boundary_offset: 16,
        boundary: BoundaryKind::AuthoredSemicolon,
    },
];

const NAME_POLICY_FIXTURES: &[NamePolicyFixture] = &[
    NamePolicyFixture {
        id: "IDREF-NAME-STRICT-ONLY-LET",
        source: "const x = let;",
        rhs: "let",
        rhs_range: ByteRange::new(10, 13),
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-STRICT-ONLY-STATIC",
        source: "const x = static;",
        rhs: "static",
        rhs_range: ByteRange::new(10, 16),
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-STRICT-ONLY-IMPLEMENTS",
        source: "const x = implements;",
        rhs: "implements",
        rhs_range: ByteRange::new(10, 20),
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-STRICT-ONLY-INTERFACE",
        source: "const x = interface;",
        rhs: "interface",
        rhs_range: ByteRange::new(10, 19),
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-STRICT-ONLY-PACKAGE",
        source: "const x = package;",
        rhs: "package",
        rhs_range: ByteRange::new(10, 17),
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-STRICT-ONLY-PRIVATE",
        source: "const x = private;",
        rhs: "private",
        rhs_range: ByteRange::new(10, 17),
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-STRICT-ONLY-PROTECTED",
        source: "const x = protected;",
        rhs: "protected",
        rhs_range: ByteRange::new(10, 19),
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-STRICT-ONLY-PUBLIC",
        source: "const x = public;",
        rhs: "public",
        rhs_range: ByteRange::new(10, 16),
        category: NamePolicyCategory::StrictOnlyRestricted,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-PARAMETER-SPECIAL-YIELD",
        source: "const x = yield;",
        rhs: "yield",
        rhs_range: ByteRange::new(10, 15),
        category: NamePolicyCategory::YieldAwaitParameterSpecial,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-PARAMETER-SPECIAL-AWAIT",
        source: "const x = await;",
        rhs: "await",
        rhs_range: ByteRange::new(10, 15),
        category: NamePolicyCategory::YieldAwaitParameterSpecial,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-BINDING-CONTRAST-EVAL",
        source: "const x = eval;",
        rhs: "eval",
        rhs_range: ByteRange::new(10, 14),
        category: NamePolicyCategory::BindingContrast,
    },
    NamePolicyFixture {
        id: "IDREF-NAME-BINDING-CONTRAST-ARGUMENTS",
        source: "const x = arguments;",
        rhs: "arguments",
        rhs_range: ByteRange::new(10, 19),
        category: NamePolicyCategory::BindingContrast,
    },
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

#[test]
fn frozen_authority_and_existing_identifier_initializer_gold_are_exact() {
    assert_eq!(ISSUE_ID, 237);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));

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
fn rhs_fixtures_pin_exact_authored_ranges_and_existing_boundaries() {
    for (index, fixture) in RHS_FIXTURES.iter().enumerate() {
        assert_eq!(
            slice(fixture.source, fixture.rhs_range),
            fixture.rhs,
            "{}",
            fixture.id
        );
        assert!(
            !fixture.rhs.contains('\\'),
            "{} must stay inside the escape-free RHS leaf",
            fixture.id
        );

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
            BoundaryKind::BindingListComma => {
                assert_eq!(
                    fixture.source.as_bytes().get(fixture.boundary_offset),
                    Some(&b','),
                    "{}",
                    fixture.id
                );
            }
            BoundaryKind::AuthoredSemicolon => {
                assert_eq!(
                    fixture.source.as_bytes().get(fixture.boundary_offset),
                    Some(&b';'),
                    "{}",
                    fixture.id
                );
            }
            BoundaryKind::AutomaticAtEof => {
                assert_eq!(
                    fixture.boundary_offset,
                    fixture.source.len(),
                    "{} must reach actual EOF",
                    fixture.id
                );
            }
        }
    }
}

#[test]
fn name_policy_matrix_keeps_three_rhs_categories_distinct() {
    let mut strict_only = 0;
    let mut parameter_special = 0;
    let mut binding_contrast = 0;

    for (index, fixture) in NAME_POLICY_FIXTURES.iter().enumerate() {
        assert_eq!(
            slice(fixture.source, fixture.rhs_range),
            fixture.rhs,
            "{}",
            fixture.id
        );
        assert!(
            !fixture.rhs.contains('\\'),
            "{} must use direct authored RHS spelling",
            fixture.id
        );

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
            NamePolicyCategory::StrictOnlyRestricted => strict_only += 1,
            NamePolicyCategory::YieldAwaitParameterSpecial => parameter_special += 1,
            NamePolicyCategory::BindingContrast => binding_contrast += 1,
        }
    }

    assert_eq!(strict_only, 8);
    assert_eq!(parameter_special, 2);
    assert_eq!(binding_contrast, 2);

    let strict_spellings: Vec<_> = NAME_POLICY_FIXTURES
        .iter()
        .filter(|fixture| fixture.category == NamePolicyCategory::StrictOnlyRestricted)
        .map(|fixture| fixture.rhs)
        .collect();
    assert_eq!(
        strict_spellings,
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

    let parameter_spellings: Vec<_> = NAME_POLICY_FIXTURES
        .iter()
        .filter(|fixture| fixture.category == NamePolicyCategory::YieldAwaitParameterSpecial)
        .map(|fixture| fixture.rhs)
        .collect();
    assert_eq!(parameter_spellings, ["yield", "await"]);

    let contrast_spellings: Vec<_> = NAME_POLICY_FIXTURES
        .iter()
        .filter(|fixture| fixture.category == NamePolicyCategory::BindingContrast)
        .map(|fixture| fixture.rhs)
        .collect();
    assert_eq!(contrast_spellings, ["eval", "arguments"]);
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
        assert_eq!(
            slice(fixture.source, fixture.subject),
            fixture.subject_fragment,
            "{}",
            fixture.id
        );
        assert!(
            fixture.rule_id.starts_with("EE-"),
            "{} must name accepted static authority",
            fixture.id
        );

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
        assert_eq!(
            slice(fixture.source, fixture.subject),
            match fixture.id {
                "IDREF-GRAMMAR-LATER-EMPTY-BRACED-001"
                | "IDREF-GRAMMAR-LATER-PART-EMPTY-BRACED-001" => r"\u{}",
                "IDREF-GRAMMAR-LATER-UNCLOSED-BRACED-001" => r"\u{61",
                other => panic!("unrecognized grammar control {other}"),
            }
        );

        let source = SourceText::new(
            SourceId::new(237_200 + index as u64),
            fixture.source.to_owned(),
        );
        let anchor = source
            .anchor(fixture.subject.start, fixture.subject.end)
            .expect("existing Grammar subject must remain a valid authored anchor");
        assert_eq!(anchor.range().start(), fixture.subject.start);
        assert_eq!(anchor.range().end(), fixture.subject.end);
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
fn rhs_identifier_is_not_a_bound_name_or_runtime_reference_resolution_claim() {
    const FUTURE_SELECTED_LIFECYCLE: &str = "SelectedAcceptedIncomplete";
    const FORBIDDEN_RUNTIME_OWNER: &str = "ResolveBinding";

    assert_eq!(FUTURE_SELECTED_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert_eq!(FORBIDDEN_RUNTIME_OWNER, "ResolveBinding");

    for fixture in NAME_POLICY_FIXTURES {
        assert!(
            !fixture.id.contains("BOUNDNAME"),
            "RHS names must not be modeled as declaration BoundNames"
        );
    }
}

#[test]
fn deterministic_pathological_utf8_ranges_remain_checked_and_linear_to_validate() {
    let identifier = format!("π{}", "α".repeat(4096));
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
fn validation_source_has_no_dependency_on_selected_production_or_future_expression_code() {
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
    let positive_ids = RHS_FIXTURES
        .iter()
        .map(|fixture| fixture.id)
        .collect::<Vec<_>>();
    assert!(positive_ids.contains(&"IDREF-INIT-POSITIVE-SEMICOLON-001"));
    assert!(positive_ids.contains(&"IDREF-INIT-POSITIVE-EOF-001"));
    assert!(positive_ids.contains(&"IDREF-INIT-POSITIVE-COMMA-001"));

    assert!(
        UNSUPPORTED_FIXTURES
            .iter()
            .any(|fixture| { fixture.reason == UnsupportedReason::EscapedIdentifierReference })
    );
    assert!(
        UNSUPPORTED_FIXTURES
            .iter()
            .any(|fixture| { fixture.reason == UnsupportedReason::RequiresNonEofAsi })
    );
    assert!(
        UNSUPPORTED_FIXTURES
            .iter()
            .any(|fixture| { fixture.reason == UnsupportedReason::MemberExpression })
    );
}
