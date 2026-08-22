//! Candidate-independent successor oracle for selected top-level `VariableStatement`
//! decimal-integer initializer composition (Issue #326).
//!
//! This module fixes only the validation successor obtained by composing the
//! already-selected decimal-integer leaf with the already-selected top-level
//! var 1..N declaration-list frontier. It does not call production recognition,
//! static semantics, qualification, correspondence, Binding / Scope, or runtime
//! behavior, and it does not prescribe a future production representation.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

use super::inventory::{CONTAINERS, RULE_UNITS, RuleUnitKind};

const ISSUE_ID: u64 = 326;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const SELECTED_DECIMAL_INTEGER_GRAMMAR: &str = "0 | [1-9][0-9]*";
const POSITIVE_LIFECYCLE: &str = "SelectedAcceptedIncomplete";
const AGGREGATE_QUALIFIED_AVAILABLE: bool = false;

const HISTORICAL_VAR_ORACLE: &str =
    include_str!("../qualification_selected_top_level_variable_statement_validation_tests.rs");
const HISTORICAL_COMPLETION: &str = include_str!("selected_variable_statement_slice_completion.rs");
const HISTORICAL_CORRESPONDENCE: &str =
    include_str!("../selected_variable_statement_name_correspondence_validation_tests.rs");
const HISTORICAL_MULTI_DECLARATOR: &str =
    include_str!("selected_variable_statement_multi_declarator_frontier.rs");
const THIS_SOURCE: &str =
    include_str!("selected_variable_statement_decimal_initializer_frontier.rs");

const CURRENT_REQUIRED_RULE_IDS: &[&str] = &[
    "EE-01-R01",
    "EE-01-R02",
    "EE-04-R08",
    "EE-14-R01",
    "EE-15-R01",
    "EE-15-R02",
    "EE-15-R03",
    "EE-36-R01",
    "EE-36-R02",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range {
    start: usize,
    end: usize,
}

impl Range {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeclaratorExpectation {
    binding: Range,
    semantic_name: &'static str,
    decimal_rhs: Option<Range>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminator {
    AuthoredSemicolon,
    AutomaticAtEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PositiveFixture {
    id: &'static str,
    source: &'static str,
    declarators: &'static [DeclaratorExpectation],
    terminator: Terminator,
}

const POSITIVE_FIXTURES: &[PositiveFixture] = &[
    PositiveFixture {
        id: "single-zero-authored",
        source: "var a=0;",
        declarators: &[DeclaratorExpectation {
            binding: Range::new(4, 5),
            semantic_name: "a",
            decimal_rhs: Some(Range::new(6, 7)),
        }],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "single-one-eof",
        source: "var a = 1",
        declarators: &[DeclaratorExpectation {
            binding: Range::new(4, 5),
            semantic_name: "a",
            decimal_rhs: Some(Range::new(8, 9)),
        }],
        terminator: Terminator::AutomaticAtEof,
    },
    PositiveFixture {
        id: "mixed-first",
        source: "var a=1,b;",
        declarators: &[
            DeclaratorExpectation {
                binding: Range::new(4, 5),
                semantic_name: "a",
                decimal_rhs: Some(Range::new(6, 7)),
            },
            DeclaratorExpectation {
                binding: Range::new(8, 9),
                semantic_name: "b",
                decimal_rhs: None,
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "mixed-final",
        source: "var a,b=2;",
        declarators: &[
            DeclaratorExpectation {
                binding: Range::new(4, 5),
                semantic_name: "a",
                decimal_rhs: None,
            },
            DeclaratorExpectation {
                binding: Range::new(6, 7),
                semantic_name: "b",
                decimal_rhs: Some(Range::new(8, 9)),
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "three-mixed",
        source: "var a=1,b,c=2;",
        declarators: &[
            DeclaratorExpectation {
                binding: Range::new(4, 5),
                semantic_name: "a",
                decimal_rhs: Some(Range::new(6, 7)),
            },
            DeclaratorExpectation {
                binding: Range::new(8, 9),
                semantic_name: "b",
                decimal_rhs: None,
            },
            DeclaratorExpectation {
                binding: Range::new(10, 11),
                semantic_name: "c",
                decimal_rhs: Some(Range::new(12, 13)),
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "three-initialized-eof",
        source: "var a=1,b=2,c=3",
        declarators: &[
            DeclaratorExpectation {
                binding: Range::new(4, 5),
                semantic_name: "a",
                decimal_rhs: Some(Range::new(6, 7)),
            },
            DeclaratorExpectation {
                binding: Range::new(8, 9),
                semantic_name: "b",
                decimal_rhs: Some(Range::new(10, 11)),
            },
            DeclaratorExpectation {
                binding: Range::new(12, 13),
                semantic_name: "c",
                decimal_rhs: Some(Range::new(14, 15)),
            },
        ],
        terminator: Terminator::AutomaticAtEof,
    },
    PositiveFixture {
        id: "multi-digit",
        source: "var a=12345,b;",
        declarators: &[
            DeclaratorExpectation {
                binding: Range::new(4, 5),
                semantic_name: "a",
                decimal_rhs: Some(Range::new(6, 11)),
            },
            DeclaratorExpectation {
                binding: Range::new(12, 13),
                semantic_name: "b",
                decimal_rhs: None,
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "line-before-comma",
        source: "var a=1\n, b=2;",
        declarators: &[
            DeclaratorExpectation {
                binding: Range::new(4, 5),
                semantic_name: "a",
                decimal_rhs: Some(Range::new(6, 7)),
            },
            DeclaratorExpectation {
                binding: Range::new(10, 11),
                semantic_name: "b",
                decimal_rhs: Some(Range::new(12, 13)),
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "line-after-comma",
        source: "var a=1,\n b=2",
        declarators: &[
            DeclaratorExpectation {
                binding: Range::new(4, 5),
                semantic_name: "a",
                decimal_rhs: Some(Range::new(6, 7)),
            },
            DeclaratorExpectation {
                binding: Range::new(10, 11),
                semantic_name: "b",
                decimal_rhs: Some(Range::new(12, 13)),
            },
        ],
        terminator: Terminator::AutomaticAtEof,
    },
    PositiveFixture {
        id: "direct-escaped-equality",
        source: r"var a=1,\u0061=2;",
        declarators: &[
            DeclaratorExpectation {
                binding: Range::new(4, 5),
                semantic_name: "a",
                decimal_rhs: Some(Range::new(6, 7)),
            },
            DeclaratorExpectation {
                binding: Range::new(8, 14),
                semantic_name: "a",
                decimal_rhs: Some(Range::new(15, 16)),
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Disposition {
    SelectedAcceptedIncomplete,
    StaticRejected { rule_id: &'static str, subject: Range },
    DefinitiveGrammarRejection { subject: Range },
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoundaryFixture {
    source: &'static str,
    disposition: Disposition,
}

const NUMERIC_BOUNDARIES: &[BoundaryFixture] = &[
    BoundaryFixture { source: "var a=01;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=1_0;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=1.0;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=1e2;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=1n;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=+1;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=-1;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=true;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=null;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=this;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=\"x\";", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=foo;", disposition: Disposition::UnsupportedCoverage },
];

const BINDING_BOUNDARIES: &[BoundaryFixture] = &[
    BoundaryFixture {
        source: r"var \u0069f=1;",
        disposition: Disposition::StaticRejected {
            rule_id: "EE-04-R08",
            subject: Range::new(4, 11),
        },
    },
    BoundaryFixture {
        source: r"var a,\u{}=1;",
        disposition: Disposition::DefinitiveGrammarRejection {
            subject: Range::new(6, 10),
        },
    },
    BoundaryFixture {
        source: r"var a,\u0061=1;",
        disposition: Disposition::SelectedAcceptedIncomplete,
    },
    BoundaryFixture { source: "var if=1;", disposition: Disposition::UnsupportedCoverage },
];

const OUT_OF_SCOPE_BOUNDARIES: &[BoundaryFixture] = &[
    BoundaryFixture { source: "var a=1,", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=1,b=", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=1,b=true;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=1,b=1.0;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=1\nvar b;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=1, /*comment*/ b;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "var a=1,{b}=2;", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "{ var a=1,b; }", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture { source: "for (var a=1,b;;) {}", disposition: Disposition::UnsupportedCoverage },
    BoundaryFixture {
        source: r"var a=1,b,\u{}=2;",
        disposition: Disposition::DefinitiveGrammarRejection {
            subject: Range::new(10, 14),
        },
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuleWitness {
    rule_id: &'static str,
    subject: Range,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PrecedenceFixture {
    source: &'static str,
    primary: RuleWitness,
    lower_priority: RuleWitness,
}

const PRECEDENCE_FIXTURE: PrecedenceFixture = PrecedenceFixture {
    source: r"let a; var a=1, \u0069f=2;",
    primary: RuleWitness {
        rule_id: "EE-04-R08",
        subject: Range::new(16, 23),
    },
    lower_priority: RuleWitness {
        rule_id: "EE-36-R02",
        subject: Range::new(11, 12),
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CollisionFixture {
    source: &'static str,
    lexical: Range,
    var: Range,
}

const COLLISION_FIXTURES: &[CollisionFixture] = &[
    CollisionFixture {
        source: "let b; var a=1,b=2;",
        lexical: Range::new(4, 5),
        var: Range::new(15, 16),
    },
    CollisionFixture {
        source: "var a=1,a=2; let a;",
        lexical: Range::new(17, 18),
        var: Range::new(4, 5),
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContributorFixture {
    source: &'static str,
    contributors: &'static [Range],
}

const CONTRIBUTOR_FIXTURES: &[ContributorFixture] = &[
    ContributorFixture {
        source: "var a=1,a; let x=a;",
        contributors: &[Range::new(4, 5), Range::new(8, 9)],
    },
    ContributorFixture {
        source: r"var a=1,\u0061=2; let x=a;",
        contributors: &[Range::new(4, 5), Range::new(8, 14)],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositivePartition {
    HistoricalFlat,
    BlockEnabledWithoutVar,
    VariableEnabled,
}

fn partition(blocks: usize, var_statements: usize) -> PositivePartition {
    if var_statements > 0 {
        PositivePartition::VariableEnabled
    } else if blocks > 0 {
        PositivePartition::BlockEnabledWithoutVar
    } else {
        PositivePartition::HistoricalFlat
    }
}

fn slice(source: &str, range: Range) -> &str {
    source.get(range.start..range.end).expect("fixture range must be exact UTF-8")
}

fn assert_anchor(source_text: &str, range: Range, source_id: u64) {
    let source = SourceText::new(SourceId::new(source_id), source_text.to_owned());
    let anchor = source.anchor(range.start, range.end).expect("fixture range must form SourceAnchor");
    assert_eq!(anchor.fragment(), slice(source_text, range));
}

fn is_selected_decimal_integer(candidate: &str) -> bool {
    if candidate == "0" {
        return true;
    }
    let mut bytes = candidate.bytes();
    match bytes.next() {
        Some(b'1'..=b'9') => bytes.all(|byte| byte.is_ascii_digit()),
        _ => false,
    }
}

#[test]
fn authority_chain_is_additive_and_historical_frontiers_stay_immutable() {
    assert_eq!(ISSUE_ID, 326);
    assert_eq!(ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(ECMA_262_SNAPSHOT, "d89c03f2db8a597bc915b363a6518d0cc8acdbc0");
    assert_eq!(UNICODE_VERSION, "17.0.0");
    assert_eq!(SELECTED_DECIMAL_INTEGER_GRAMMAR, "0 | [1-9][0-9]*");
    assert!(HISTORICAL_VAR_ORACLE.contains("SELECTED_VARIABLE_INITIALIZER: bool = false"));
    assert!(HISTORICAL_COMPLETION.contains("assert_eq!(required.len(), 9);"));
    assert!(HISTORICAL_COMPLETION.contains("assert_eq!(non_triggering.len(), 184);"));
    assert!(HISTORICAL_CORRESPONDENCE.contains("SameSourceSelectedVarNameContributors"));
    assert!(HISTORICAL_MULTI_DECLARATOR.contains("SELECTED_LIST_CARDINALITY: &str = \"1..N\""));
    assert!(HISTORICAL_MULTI_DECLARATOR.contains("var a,b = 1;"));
}

#[test]
fn decimal_leaf_is_exactly_zero_or_nonzero_ascii_decimal_digits() {
    for accepted in ["0", "1", "9", "10", "42", "12345"] {
        assert!(is_selected_decimal_integer(accepted));
    }
    for rejected in ["", "00", "01", "1_0", "1.0", "1e2", "1n", "+1", "-1", "０", "١"] {
        assert!(!is_selected_decimal_integer(rejected));
    }
}

#[test]
fn positives_pin_per_declarator_presence_exact_rhs_ranges_and_one_to_n_order() {
    for (fixture_index, fixture) in POSITIVE_FIXTURES.iter().enumerate() {
        assert!(!fixture.declarators.is_empty());
        assert!(fixture.declarators.iter().any(|declarator| declarator.decimal_rhs.is_some()));
        let mut previous_binding_end = 0;
        for (declarator_index, declarator) in fixture.declarators.iter().enumerate() {
            assert_anchor(
                fixture.source,
                declarator.binding,
                ISSUE_ID * 10_000 + fixture_index as u64 * 100 + declarator_index as u64,
            );
            assert!(declarator.binding.start >= previous_binding_end);
            assert!(!declarator.semantic_name.is_empty());
            previous_binding_end = declarator.binding.end;
            if let Some(rhs) = declarator.decimal_rhs {
                assert_anchor(
                    fixture.source,
                    rhs,
                    ISSUE_ID * 100_000 + fixture_index as u64 * 100 + declarator_index as u64,
                );
                assert!(rhs.start >= declarator.binding.end);
                assert!(is_selected_decimal_integer(slice(fixture.source, rhs)));
            }
        }
    }
    for (id, expected_count) in [("single-zero-authored", 1), ("mixed-first", 2), ("three-mixed", 3)] {
        let fixture = POSITIVE_FIXTURES.iter().find(|fixture| fixture.id == id).unwrap();
        assert_eq!(fixture.declarators.len(), expected_count);
    }
}

#[test]
fn mixed_initialized_and_uninitialized_declarators_remain_distinct() {
    for (id, expected) in [
        ("mixed-first", &[true, false][..]),
        ("mixed-final", &[false, true][..]),
        ("three-mixed", &[true, false, true][..]),
        ("three-initialized-eof", &[true, true, true][..]),
    ] {
        let fixture = POSITIVE_FIXTURES.iter().find(|fixture| fixture.id == id).unwrap();
        let actual: Vec<_> = fixture.declarators.iter().map(|d| d.decimal_rhs.is_some()).collect();
        assert_eq!(actual, expected);
    }
}

#[test]
fn numeric_neighbors_and_richer_initializer_atoms_remain_unsupported() {
    assert!(NUMERIC_BOUNDARIES.iter().all(|fixture| fixture.disposition == Disposition::UnsupportedCoverage));
}

#[test]
fn binding_grammar_and_static_routes_remain_distinct() {
    let escaped = BINDING_BOUNDARIES.iter().find(|fixture| fixture.source == r"var \u0069f=1;").unwrap();
    assert_eq!(
        escaped.disposition,
        Disposition::StaticRejected { rule_id: "EE-04-R08", subject: Range::new(4, 11) }
    );
    assert_eq!(slice(escaped.source, Range::new(4, 11)), r"\u0069f");

    let malformed = BINDING_BOUNDARIES.iter().find(|fixture| fixture.source == r"var a,\u{}=1;").unwrap();
    assert_eq!(
        malformed.disposition,
        Disposition::DefinitiveGrammarRejection { subject: Range::new(6, 10) }
    );
    assert_eq!(slice(malformed.source, Range::new(6, 10)), r"\u{}");

    let equal = BINDING_BOUNDARIES.iter().find(|fixture| fixture.source == r"var a,\u0061=1;").unwrap();
    assert_eq!(equal.disposition, Disposition::SelectedAcceptedIncomplete);
    assert!(BINDING_BOUNDARIES.iter().any(|fixture| fixture.source == "var if=1;" && fixture.disposition == Disposition::UnsupportedCoverage));
}

#[test]
fn tier_priority_remains_above_authored_order_across_initializer_bearing_var() {
    assert_eq!(PRECEDENCE_FIXTURE.primary.rule_id, "EE-04-R08");
    assert_eq!(PRECEDENCE_FIXTURE.lower_priority.rule_id, "EE-36-R02");
    assert_anchor(PRECEDENCE_FIXTURE.source, PRECEDENCE_FIXTURE.primary.subject, ISSUE_ID);
    assert_anchor(PRECEDENCE_FIXTURE.source, PRECEDENCE_FIXTURE.lower_priority.subject, ISSUE_ID + 1);
    assert!(PRECEDENCE_FIXTURE.primary.subject.start > PRECEDENCE_FIXTURE.lower_priority.subject.start);
}

#[test]
fn collision_and_correspondence_evidence_ignore_decimal_initializer_content() {
    for (index, fixture) in COLLISION_FIXTURES.iter().enumerate() {
        assert_anchor(fixture.source, fixture.lexical, ISSUE_ID * 10 + index as u64);
        assert_anchor(fixture.source, fixture.var, ISSUE_ID * 100 + index as u64);
    }
    assert_eq!(slice(COLLISION_FIXTURES[0].source, COLLISION_FIXTURES[0].var), "b");

    assert!(HISTORICAL_CORRESPONDENCE.contains("VisibleSelectedLexicalBinding"));
    assert!(HISTORICAL_CORRESPONDENCE.contains("SameSourceSelectedVarNameContributors"));
    assert!(HISTORICAL_CORRESPONDENCE.contains("NoSelectedSameSourceContributor"));
    for (index, fixture) in CONTRIBUTOR_FIXTURES.iter().enumerate() {
        let mut previous_end = 0;
        for contributor in fixture.contributors {
            assert_anchor(fixture.source, *contributor, ISSUE_ID * 1000 + index as u64);
            assert!(contributor.start >= previous_end);
            previous_end = contributor.end;
        }
    }
}

#[test]
fn incomplete_unselected_and_non_eof_boundaries_commit_no_selected_success() {
    for fixture in OUT_OF_SCOPE_BOUNDARIES {
        assert_ne!(fixture.disposition, Disposition::SelectedAcceptedIncomplete);
    }
    let malformed = OUT_OF_SCOPE_BOUNDARIES.iter().find(|fixture| fixture.source == r"var a=1,b,\u{}=2;").unwrap();
    assert_eq!(malformed.disposition, Disposition::DefinitiveGrammarRejection { subject: Range::new(10, 14) });
    assert_eq!(slice(malformed.source, Range::new(10, 14)), r"\u{}");
}

#[test]
fn terminator_semantics_remain_statement_owned() {
    assert!(POSITIVE_FIXTURES.iter().any(|fixture| fixture.source == "var a=1\n, b=2;" && fixture.terminator == Terminator::AuthoredSemicolon));
    assert!(POSITIVE_FIXTURES.iter().any(|fixture| fixture.source == "var a=1,\n b=2" && fixture.terminator == Terminator::AutomaticAtEof));
    assert!(OUT_OF_SCOPE_BOUNDARIES.iter().any(|fixture| fixture.source == "var a=1\nvar b;" && fixture.disposition == Disposition::UnsupportedCoverage));
    assert!(OUT_OF_SCOPE_BOUNDARIES.iter().any(|fixture| fixture.source == "var a=1, /*comment*/ b;" && fixture.disposition == Disposition::UnsupportedCoverage));
}

#[test]
fn rule_reachability_lifecycle_and_positive_partition_remain_unchanged() {
    let active_ids: BTreeSet<_> = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::NormativeRule)
        .map(|rule| rule.id)
        .collect();
    for rule_id in CURRENT_REQUIRED_RULE_IDS {
        assert!(active_ids.contains(rule_id));
    }
    assert!(active_ids.contains("EE-02-R01"));
    assert!(!CURRENT_REQUIRED_RULE_IDS.contains(&"EE-02-R01"));

    let active_count = RULE_UNITS.iter().filter(|rule| rule.kind == RuleUnitKind::NormativeRule).count();
    let inactive_count = RULE_UNITS.iter().filter(|rule| rule.kind == RuleUnitKind::EnvelopeInactiveRule).count();
    let sentinel_count = RULE_UNITS.iter().filter(|rule| rule.kind == RuleUnitKind::ExpansionSentinel).count();

    assert_eq!(CONTAINERS.len(), 37);
    assert_eq!(active_count, 183);
    assert_eq!(inactive_count, 10);
    assert_eq!(sentinel_count, 0);
    assert_eq!(RULE_UNITS.len(), 193);
    assert_eq!(CURRENT_REQUIRED_RULE_IDS.len(), 9);
    assert_eq!(RULE_UNITS.len() - CURRENT_REQUIRED_RULE_IDS.len(), 184);
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert_ne!(POSITIVE_LIFECYCLE, "Qualified");
    assert!(!AGGREGATE_QUALIFIED_AVAILABLE);

    assert_eq!(partition(0, 0), PositivePartition::HistoricalFlat);
    assert_eq!(partition(1, 0), PositivePartition::BlockEnabledWithoutVar);
    assert_eq!(partition(0, 1), PositivePartition::VariableEnabled);
    assert_eq!(partition(2, 17), PositivePartition::VariableEnabled);
}

#[test]
fn validation_remains_candidate_independent_runtime_negative_and_representation_neutral() {
    for forbidden in [
        concat!("selected_lexical_", "slice::"),
        concat!("selected_static_", "semantics::"),
        concat!("selected_qualification_", "integration::"),
        concat!("selected_variable_statement_name_", "correspondence::"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("evaluate_selected_", "static_semantics"),
        concat!("attempt_selected_", "qualification"),
        concat!("SelectedVariable", "Initializer"),
        concat!("SelectedVariable", "Declaration"),
        concat!("Vec<SelectedVariable", "Binding>"),
        concat!("Environment", "Record"),
        concat!("Resolve", "Binding"),
        concat!("Get", "Value"),
        concat!("Put", "Value"),
        concat!("Declaration", "Instantiation"),
        concat!("QualificationOutcome::", "Qualified"),
    ] {
        assert!(!THIS_SOURCE.contains(forbidden), "forbidden validation dependency or commitment: {forbidden}");
    }
    assert!(HISTORICAL_CORRESPONDENCE.contains("authored VariableDeclaration provenance != unique runtime binding identity"));
}
