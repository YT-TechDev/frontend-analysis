//! Candidate-independent EOF-only ASI validation for Issue #233.
//!
//! This module fixes the selected `LexicalDeclaration` composition policy
//! without calling production lexical, static-semantics, or aggregate code.
//! It validates expected meaning and source provenance rather than a future
//! production representation.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::{gold_source, gold_subject_range};

const ISSUE_ID: u64 = 233;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const ASI_GOLD_ID: &str = "JS-GOLD-ASI-NO-FABRICATED-RANGE-001";
const GOLD_SOURCE: &str = include_str!("qualification_validation_tests/gold.rs");
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const THIS_SOURCE: &str = include_str!("qualification_selected_eof_asi_validation_tests.rs");

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
enum FutureLifecycle {
    SelectedAcceptedIncomplete,
    StaticSemanticsRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StaticExpectation {
    rule_id: &'static str,
    subject: ByteRange,
    explicit_control_gold_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SyntheticSemicolonExpectation {
    decision_offset: usize,
    authored_range: Option<ByteRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EofSelectedFixture {
    id: &'static str,
    source: &'static str,
    final_declaration: ByteRange,
    trailing_trivia: ByteRange,
    required_authored_semicolons: &'static [ByteRange],
    lifecycle: FutureLifecycle,
    static_expectation: Option<StaticExpectation>,
    synthetic_semicolon: SyntheticSemicolonExpectation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GrammarControlFixture {
    id: &'static str,
    source: &'static str,
    subject: ByteRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedReason {
    IncompleteInitializer,
    IncompleteBindingList,
    UnexpectedToken,
    RequiresNonEofAsi,
    CommentsOutOfScope,
    DeferredMalformedEscape,
    GeneralExpressionInitializer,
    HashbangOutOfScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UnsupportedFixture {
    id: &'static str,
    source: &'static str,
    reason: UnsupportedReason,
}

const EOF_SELECTED_FIXTURES: &[EofSelectedFixture] = &[
    EofSelectedFixture {
        id: "EOF-ASI-POSITIVE-LET-INIT-001",
        source: "let x = 1",
        final_declaration: ByteRange::new(0, 9),
        trailing_trivia: ByteRange::new(9, 9),
        required_authored_semicolons: &[],
        lifecycle: FutureLifecycle::SelectedAcceptedIncomplete,
        static_expectation: None,
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 9,
            authored_range: None,
        },
    },
    EofSelectedFixture {
        id: "EOF-ASI-POSITIVE-CONST-INIT-001",
        source: "const x = 1",
        final_declaration: ByteRange::new(0, 11),
        trailing_trivia: ByteRange::new(11, 11),
        required_authored_semicolons: &[],
        lifecycle: FutureLifecycle::SelectedAcceptedIncomplete,
        static_expectation: None,
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 11,
            authored_range: None,
        },
    },
    EofSelectedFixture {
        id: "EOF-ASI-POSITIVE-MULTIBIND-001",
        source: "let x, y",
        final_declaration: ByteRange::new(0, 8),
        trailing_trivia: ByteRange::new(8, 8),
        required_authored_semicolons: &[],
        lifecycle: FutureLifecycle::SelectedAcceptedIncomplete,
        static_expectation: None,
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 8,
            authored_range: None,
        },
    },
    EofSelectedFixture {
        id: "EOF-ASI-POSITIVE-CONST-MULTIBIND-001",
        source: "const x = 1, y = 2",
        final_declaration: ByteRange::new(0, 18),
        trailing_trivia: ByteRange::new(18, 18),
        required_authored_semicolons: &[],
        lifecycle: FutureLifecycle::SelectedAcceptedIncomplete,
        static_expectation: None,
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 18,
            authored_range: None,
        },
    },
    EofSelectedFixture {
        id: "EOF-ASI-POSITIVE-TRAILING-TRIVIA-001",
        source: "let x = 1 \t\n\r\u{2028}\u{2029}\u{00A0}\u{1680}\u{2000}\u{202F}\u{205F}\u{3000}\u{FEFF}",
        final_declaration: ByteRange::new(0, 9),
        trailing_trivia: ByteRange::new(9, 39),
        required_authored_semicolons: &[],
        lifecycle: FutureLifecycle::SelectedAcceptedIncomplete,
        static_expectation: None,
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 39,
            authored_range: None,
        },
    },
    EofSelectedFixture {
        id: "EOF-ASI-POSITIVE-FINAL-DECLARATION-ONLY-001",
        source: "let x; const y = 1",
        final_declaration: ByteRange::new(7, 18),
        trailing_trivia: ByteRange::new(18, 18),
        required_authored_semicolons: &[ByteRange::new(5, 6)],
        lifecycle: FutureLifecycle::SelectedAcceptedIncomplete,
        static_expectation: None,
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 18,
            authored_range: None,
        },
    },
    EofSelectedFixture {
        id: "EOF-ASI-STATIC-R03-001",
        source: "const x",
        final_declaration: ByteRange::new(0, 7),
        trailing_trivia: ByteRange::new(7, 7),
        required_authored_semicolons: &[],
        lifecycle: FutureLifecycle::StaticSemanticsRejected,
        static_expectation: Some(StaticExpectation {
            rule_id: "EE-15-R03",
            subject: ByteRange::new(6, 7),
            explicit_control_gold_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        }),
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 7,
            authored_range: None,
        },
    },
    EofSelectedFixture {
        id: "EOF-ASI-STATIC-R01-001",
        source: "let let",
        final_declaration: ByteRange::new(0, 7),
        trailing_trivia: ByteRange::new(7, 7),
        required_authored_semicolons: &[],
        lifecycle: FutureLifecycle::StaticSemanticsRejected,
        static_expectation: Some(StaticExpectation {
            rule_id: "EE-15-R01",
            subject: ByteRange::new(4, 7),
            explicit_control_gold_id: "JS-GOLD-LEXDECL-LET-BINDING-001",
        }),
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 7,
            authored_range: None,
        },
    },
    EofSelectedFixture {
        id: "EOF-ASI-STATIC-R02-001",
        source: "let x, x",
        final_declaration: ByteRange::new(0, 8),
        trailing_trivia: ByteRange::new(8, 8),
        required_authored_semicolons: &[],
        lifecycle: FutureLifecycle::StaticSemanticsRejected,
        static_expectation: Some(StaticExpectation {
            rule_id: "EE-15-R02",
            subject: ByteRange::new(7, 8),
            explicit_control_gold_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        }),
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 8,
            authored_range: None,
        },
    },
    EofSelectedFixture {
        id: "EOF-ASI-STATIC-EE36-001",
        source: "let x; let x",
        final_declaration: ByteRange::new(7, 12),
        trailing_trivia: ByteRange::new(12, 12),
        required_authored_semicolons: &[ByteRange::new(5, 6)],
        lifecycle: FutureLifecycle::StaticSemanticsRejected,
        static_expectation: Some(StaticExpectation {
            rule_id: "EE-36-R01",
            subject: ByteRange::new(11, 12),
            explicit_control_gold_id: "JS-GOLD-SCRIPT-DUPLEXICAL-001",
        }),
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 12,
            authored_range: None,
        },
    },
];

const GRAMMAR_CONTROLS: &[GrammarControlFixture] = &[
    GrammarControlFixture {
        id: "EOF-ASI-GRAMMAR-EMPTY-BRACED-001",
        source: r"let \u{}",
        subject: ByteRange::new(4, 8),
    },
    GrammarControlFixture {
        id: "EOF-ASI-GRAMMAR-PART-EMPTY-BRACED-001",
        source: r"let a\u{}",
        subject: ByteRange::new(5, 9),
    },
    GrammarControlFixture {
        id: "EOF-ASI-GRAMMAR-UNCLOSED-BRACED-001",
        source: r"let \u{61",
        subject: ByteRange::new(4, 9),
    },
];

const UNSUPPORTED_FIXTURES: &[UnsupportedFixture] = &[
    UnsupportedFixture {
        id: "EOF-ASI-UNSUPPORTED-INCOMPLETE-INITIALIZER-001",
        source: "let x =",
        reason: UnsupportedReason::IncompleteInitializer,
    },
    UnsupportedFixture {
        id: "EOF-ASI-UNSUPPORTED-INCOMPLETE-BINDING-LIST-001",
        source: "let x,",
        reason: UnsupportedReason::IncompleteBindingList,
    },
    UnsupportedFixture {
        id: "EOF-ASI-UNSUPPORTED-UNEXPECTED-TOKEN-001",
        source: "let x y",
        reason: UnsupportedReason::UnexpectedToken,
    },
    UnsupportedFixture {
        id: "EOF-ASI-UNSUPPORTED-NON-EOF-ASI-001",
        source: "let x\nconst y = 1",
        reason: UnsupportedReason::RequiresNonEofAsi,
    },
    UnsupportedFixture {
        id: "EOF-ASI-UNSUPPORTED-COMMENT-001",
        source: "let x/*comment*/",
        reason: UnsupportedReason::CommentsOutOfScope,
    },
    UnsupportedFixture {
        id: "EOF-ASI-UNSUPPORTED-DEFERRED-ESCAPE-001",
        source: r"let \u",
        reason: UnsupportedReason::DeferredMalformedEscape,
    },
    UnsupportedFixture {
        id: "EOF-ASI-UNSUPPORTED-DEFERRED-ESCAPE-002",
        source: r"let \u{",
        reason: UnsupportedReason::DeferredMalformedEscape,
    },
    UnsupportedFixture {
        id: "EOF-ASI-UNSUPPORTED-DEFERRED-ESCAPE-003",
        source: r"let \u0",
        reason: UnsupportedReason::DeferredMalformedEscape,
    },
    UnsupportedFixture {
        id: "EOF-ASI-UNSUPPORTED-IDENTIFIER-INITIALIZER-001",
        source: "const x = foo",
        reason: UnsupportedReason::GeneralExpressionInitializer,
    },
    UnsupportedFixture {
        id: "EOF-ASI-UNSUPPORTED-HASHBANG-001",
        source: "#!node\nlet x = 1",
        reason: UnsupportedReason::HashbangOutOfScope,
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
fn frozen_authority_separates_generic_asi_provenance_from_selected_eof_applicability() {
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));

    let generic_asi_source = gold_source(ASI_GOLD_ID).expect("#197 generic ASI gold must remain present");
    assert!(
        !EOF_SELECTED_FIXTURES
            .iter()
            .any(|fixture| fixture.source == generic_asi_source),
        "generic #197 ASI gold must not become the selected EOF-only applicability oracle"
    );

    let asi_block = fixture_block(GOLD_SOURCE, ASI_GOLD_ID);
    assert!(asi_block.contains("synthetic: ASI_EXPECTATION"));

    let start = GOLD_SOURCE
        .find("const ASI_EXPECTATION: &[SyntheticExpectation]")
        .expect("#197 ASI expectation authority must remain present");
    let rest = &GOLD_SOURCE[start..];
    let end = rest
        .find("];\n")
        .expect("#197 ASI expectation must retain a complete declaration")
        + "];\n".len();
    let expectation = &rest[..end];
    assert!(expectation.contains("SyntheticKind::AutomaticSemicolonInsertion"));
    assert!(expectation.contains("authored_range: None"));
}

#[test]
fn eof_selected_fixtures_separate_authored_bytes_trivia_eof_and_synthetic_semicolon() {
    for fixture in EOF_SELECTED_FIXTURES {
        let text = fixture.source;
        let source = SourceText::new(SourceId::new(ISSUE_ID), text.to_owned());

        assert_eq!(
            fixture.synthetic_semicolon.decision_offset,
            text.len(),
            "{}",
            fixture.id
        );
        assert_eq!(
            fixture.synthetic_semicolon.authored_range, None,
            "{}",
            fixture.id
        );
        assert_eq!(
            fixture.final_declaration.end, fixture.trailing_trivia.start,
            "{}",
            fixture.id
        );
        assert_eq!(fixture.trailing_trivia.end, text.len(), "{}", fixture.id);

        let declaration = source
            .anchor(fixture.final_declaration.start, fixture.final_declaration.end)
            .unwrap_or_else(|_| panic!("{} declaration must remain authored", fixture.id));
        assert!(!declaration.fragment().ends_with(';'), "{}", fixture.id);

        let trailing = slice(text, fixture.trailing_trivia);
        assert!(
            trailing.chars().all(is_selected_trivia_for_oracle),
            "{} trailing bytes must stay in the independently frozen selected trivia set",
            fixture.id
        );

        for semicolon_range in fixture.required_authored_semicolons {
            let semicolon = source
                .anchor(semicolon_range.start, semicolon_range.end)
                .unwrap_or_else(|_| panic!("{} prior terminator must be authored", fixture.id));
            assert_eq!(semicolon.fragment(), ";", "{}", fixture.id);
            assert!(
                semicolon_range.end <= fixture.final_declaration.start,
                "{}",
                fixture.id
            );
        }

        match fixture.lifecycle {
            FutureLifecycle::SelectedAcceptedIncomplete => {
                assert!(fixture.static_expectation.is_none(), "{}", fixture.id);
            }
            FutureLifecycle::StaticSemanticsRejected => {
                assert!(fixture.static_expectation.is_some(), "{}", fixture.id);
            }
        }
    }
}

#[test]
fn eof_asi_static_composition_preserves_existing_rule_and_authored_subject_identity() {
    let static_fixtures: Vec<_> = EOF_SELECTED_FIXTURES
        .iter()
        .filter(|fixture| fixture.static_expectation.is_some())
        .collect();
    assert_eq!(static_fixtures.len(), 4);

    for fixture in static_fixtures {
        let expected = fixture.static_expectation.expect("filtered static fixture");
        let control_source = gold_source(expected.explicit_control_gold_id)
            .unwrap_or_else(|| panic!("{} explicit-semicolon gold must exist", fixture.id));
        let control_subject = gold_subject_range(expected.explicit_control_gold_id)
            .unwrap_or_else(|| panic!("{} explicit-semicolon subject must exist", fixture.id));

        assert_eq!(
            control_source.strip_suffix(';'),
            Some(fixture.source),
            "{}",
            fixture.id
        );
        assert_eq!(
            control_subject,
            (expected.subject.start, expected.subject.end),
            "{} must preserve the existing authored static subject",
            fixture.id
        );

        let source = SourceText::new(SourceId::new(ISSUE_ID), fixture.source.to_owned());
        let subject = source
            .anchor(expected.subject.start, expected.subject.end)
            .unwrap_or_else(|_| panic!("{} static subject must remain authored", fixture.id));
        assert!(!subject.fragment().is_empty(), "{}", fixture.id);
        assert!(expected.rule_id.starts_with("EE-"), "{}", fixture.id);
    }
}

#[test]
fn existing_terminal_grammar_controls_never_acquire_an_eof_asi_event() {
    for fixture in GRAMMAR_CONTROLS {
        let source = SourceText::new(SourceId::new(ISSUE_ID), fixture.source.to_owned());
        let subject = source
            .anchor(fixture.subject.start, fixture.subject.end)
            .unwrap_or_else(|_| panic!("{} grammar subject must remain authored", fixture.id));

        assert!(!subject.fragment().is_empty(), "{}", fixture.id);
        assert_eq!(fixture.subject.end, fixture.source.len(), "{}", fixture.id);
        assert!(
            !EOF_SELECTED_FIXTURES
                .iter()
                .any(|selected| selected.source == fixture.source),
            "{} must stay on existing terminal Grammar authority",
            fixture.id
        );
    }

    assert_eq!(
        slice(GRAMMAR_CONTROLS[0].source, GRAMMAR_CONTROLS[0].subject),
        r"\u{}"
    );
    assert_eq!(
        slice(GRAMMAR_CONTROLS[1].source, GRAMMAR_CONTROLS[1].subject),
        r"\u{}"
    );
    assert_eq!(
        slice(GRAMMAR_CONTROLS[2].source, GRAMMAR_CONTROLS[2].subject),
        r"\u{61"
    );
}

#[test]
fn unsupported_neighbors_do_not_receive_synthetic_repair_or_source_verdicts() {
    assert_eq!(UNSUPPORTED_FIXTURES.len(), 10);

    for fixture in UNSUPPORTED_FIXTURES {
        assert!(
            !EOF_SELECTED_FIXTURES
                .iter()
                .any(|selected| selected.source == fixture.source),
            "{} unsupported neighbor must not be promoted by the EOF-only oracle",
            fixture.id
        );
        assert!(
            !GRAMMAR_CONTROLS
                .iter()
                .any(|grammar| grammar.source == fixture.source),
            "{} unsupported neighbor must not be turned into Grammar rejection",
            fixture.id
        );
    }

    let non_eof = UNSUPPORTED_FIXTURES
        .iter()
        .find(|fixture| fixture.reason == UnsupportedReason::RequiresNonEofAsi)
        .expect("non-EOF ASI control must remain explicit");
    assert_eq!(non_eof.source, "let x\nconst y = 1");

    let deferred: Vec<_> = UNSUPPORTED_FIXTURES
        .iter()
        .filter(|fixture| fixture.reason == UnsupportedReason::DeferredMalformedEscape)
        .map(|fixture| fixture.source)
        .collect();
    assert_eq!(deferred.as_slice(), &[r"let \u", r"let \u{", r"let \u0"]);
}

#[test]
fn only_the_final_declaration_may_use_the_eof_only_policy() {
    let selected = EOF_SELECTED_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "EOF-ASI-POSITIVE-FINAL-DECLARATION-ONLY-001")
        .expect("final-only positive fixture must exist");
    assert_eq!(selected.source, "let x; const y = 1");
    assert_eq!(
        selected.required_authored_semicolons,
        &[ByteRange::new(5, 6)]
    );
    assert_eq!(slice(selected.source, selected.final_declaration), "const y = 1");

    let unsupported = UNSUPPORTED_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "EOF-ASI-UNSUPPORTED-NON-EOF-ASI-001")
        .expect("non-EOF ASI nearest neighbor must exist");
    assert_eq!(unsupported.source, "let x\nconst y = 1");
    assert_eq!(unsupported.reason, UnsupportedReason::RequiresNonEofAsi);
}

#[test]
fn explicit_semicolon_gold_remains_the_no_synthetic_control() {
    assert_eq!(gold_source("JS-GOLD-SCRIPT-VALID-001"), Some("let x = 1;"));
    assert_eq!(
        gold_source("JS-GOLD-LEXDECL-CONST-MISSING-INIT-001"),
        Some("const x;")
    );
    assert_eq!(
        gold_subject_range("JS-GOLD-LEXDECL-CONST-MISSING-INIT-001"),
        Some((6, 7))
    );

    for gold_id in [
        "JS-GOLD-SCRIPT-VALID-001",
        "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
    ] {
        let block = fixture_block(GOLD_SOURCE, gold_id);
        assert!(block.contains("synthetic: NO_SYNTHETIC"), "{gold_id}");
    }
}

#[test]
fn long_multibyte_trailing_trivia_preserves_source_partition_without_fabrication() {
    let declaration = "let x = 1";
    let trailing_unit = "\u{3000}\u{2028}\n";
    let trailing = trailing_unit.repeat(4096);
    let text = format!("{declaration}{trailing}");
    let declaration_end = declaration.len();

    assert_eq!(slice(&text, ByteRange::new(0, declaration_end)), declaration);
    assert!(text[declaration_end..].chars().all(is_selected_trivia_for_oracle));
    assert_eq!(text.len(), declaration_end + trailing.len());
    assert!(!text.as_bytes().contains(&b';'));

    let source = SourceText::new(SourceId::new(ISSUE_ID), text.clone());
    let authored = source
        .anchor(0, declaration_end)
        .expect("long-trivia control must retain only authored declaration bytes");
    assert_eq!(authored.fragment(), declaration);

    for _ in 0..4 {
        assert_eq!(text.len(), declaration_end + trailing.len());
        assert!(text[declaration_end..].chars().all(is_selected_trivia_for_oracle));
    }
}

#[test]
fn validation_source_has_no_dependency_on_selected_production_or_future_parser_control_flow() {
    for forbidden in [
        ["use super::selected_", "lexical_slice"].concat(),
        ["use super::selected_", "static_semantics"].concat(),
        ["use super::selected_", "qualification_integration"].concat(),
        ["recognize_selected_", "lexical_slice"].concat(),
        ["attempt_selected_", "qualification"].concat(),
    ] {
        assert!(
            !THIS_SOURCE.contains(&forbidden),
            "validation oracle must not depend on production candidate path {forbidden}"
        );
    }
}
