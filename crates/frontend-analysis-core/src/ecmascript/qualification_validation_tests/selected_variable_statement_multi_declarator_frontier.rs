//! Candidate-independent successor oracle for selected top-level
//! `VariableStatement` multi-declarator semantics (Issue #322).
//!
//! Historical #306/#308/#312/#314/#318 authority remains immutable. This module
//! fixes only the inductive `VariableDeclarationList` successor theorem, selected
//! static-evidence composition, and #314 contributor-domain compatibility. It
//! does not call production recognition, static semantics, qualification,
//! correspondence, Binding / Scope, or runtime behavior, and it does not choose
//! a future production representation.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

use super::inventory::{RULE_UNITS, RuleUnitKind};

const ISSUE_ID: u64 = 322;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";

const HISTORICAL_VAR_ORACLE: &str =
    include_str!("../qualification_selected_top_level_variable_statement_validation_tests.rs");
const HISTORICAL_VAR_COMPOSITION: &str = include_str!(
    "../qualification_selected_top_level_variable_statement_composition_validation_tests.rs"
);
const HISTORICAL_COMPLETION: &str = include_str!("selected_variable_statement_slice_completion.rs");
const HISTORICAL_CORRESPONDENCE: &str =
    include_str!("../selected_variable_statement_name_correspondence_validation_tests.rs");
const HISTORICAL_EOF_ASI: &str = include_str!("selected_variable_statement_eof_asi_frontier.rs");
const TIER_PRECEDENCE_AUTHORITY: &str = include_str!("escaped_identifier.rs");
const GRAMMAR_EVIDENCE_AUTHORITY: &str =
    include_str!("../qualification_grammar_evidence_validation_tests.rs");
const THIS_SOURCE: &str = include_str!("selected_variable_statement_multi_declarator_frontier.rs");

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

const VARIABLE_DECLARATION_LIST_BASE: &str = "VariableDeclarationList : VariableDeclaration";
const VARIABLE_DECLARATION_LIST_SUCCESSOR: &str =
    "VariableDeclarationList : VariableDeclarationList , VariableDeclaration";
const SELECTED_LIST_CARDINALITY: &str = "1..N";
const SUCCESSOR_FRONTIER_CARDINALITY: &str = "2..N";
const INDUCTIVE_INVARIANTS: &[&str] = &[
    "each selected VariableDeclaration contributes one exact authored binding SourceAnchor",
    "binding and contributor sequences preserve authored VariableDeclarationList order",
    "semantic-name equality uses decoded code-point sequences without Unicode normalization",
    "one VariableStatement owns one terminator independent of list cardinality",
    "static evidence obeys tier priority then authored order within a tier",
    "an incomplete VariableDeclarationList commits no partial selected-success state",
    "rule reachability remains 9 of 193 and the positive source partition remains three-way",
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
struct Binding {
    authored: Range,
    semantic_name: &'static str,
    semantic_code_points: &'static [u32],
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
    bindings: &'static [Binding],
    terminator: Terminator,
}

const A: &[u32] = &[0x61];
const B: &[u32] = &[0x62];
const C: &[u32] = &[0x63];
const LET: &[u32] = &[0x6c, 0x65, 0x74];
const E_ACUTE: &[u32] = &[0x00e9];
const E_COMBINING: &[u32] = &[0x65, 0x0301];

const POSITIVE_FIXTURES: &[PositiveFixture] = &[
    PositiveFixture {
        id: "historical-base",
        source: "var a;",
        bindings: &[Binding {
            authored: Range::new(4, 5),
            semantic_name: "a",
            semantic_code_points: A,
        }],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "two-declarators",
        source: "var a,b;",
        bindings: &[
            Binding {
                authored: Range::new(4, 5),
                semantic_name: "a",
                semantic_code_points: A,
            },
            Binding {
                authored: Range::new(6, 7),
                semantic_name: "b",
                semantic_code_points: B,
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "two-declarators-eof",
        source: "var a,b",
        bindings: &[
            Binding {
                authored: Range::new(4, 5),
                semantic_name: "a",
                semantic_code_points: A,
            },
            Binding {
                authored: Range::new(6, 7),
                semantic_name: "b",
                semantic_code_points: B,
            },
        ],
        terminator: Terminator::AutomaticAtEof,
    },
    PositiveFixture {
        id: "three-declarators",
        source: "var a,b,c;",
        bindings: &[
            Binding {
                authored: Range::new(4, 5),
                semantic_name: "a",
                semantic_code_points: A,
            },
            Binding {
                authored: Range::new(6, 7),
                semantic_name: "b",
                semantic_code_points: B,
            },
            Binding {
                authored: Range::new(8, 9),
                semantic_name: "c",
                semantic_code_points: C,
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "line-terminator-after-comma",
        source: "var a,\n b",
        bindings: &[
            Binding {
                authored: Range::new(4, 5),
                semantic_name: "a",
                semantic_code_points: A,
            },
            Binding {
                authored: Range::new(8, 9),
                semantic_name: "b",
                semantic_code_points: B,
            },
        ],
        terminator: Terminator::AutomaticAtEof,
    },
    PositiveFixture {
        id: "line-terminator-before-comma",
        source: "var a\n, b;",
        bindings: &[
            Binding {
                authored: Range::new(4, 5),
                semantic_name: "a",
                semantic_code_points: A,
            },
            Binding {
                authored: Range::new(8, 9),
                semantic_name: "b",
                semantic_code_points: B,
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "same-statement-repeat",
        source: "var a,a;",
        bindings: &[
            Binding {
                authored: Range::new(4, 5),
                semantic_name: "a",
                semantic_code_points: A,
            },
            Binding {
                authored: Range::new(6, 7),
                semantic_name: "a",
                semantic_code_points: A,
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "direct-escaped-equality",
        source: r"var a,\u0061;",
        bindings: &[
            Binding {
                authored: Range::new(4, 5),
                semantic_name: "a",
                semantic_code_points: A,
            },
            Binding {
                authored: Range::new(6, 12),
                semantic_name: "a",
                semantic_code_points: A,
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "no-unicode-normalization",
        source: "var é,e\\u0301;",
        bindings: &[
            Binding {
                authored: Range::new(4, 6),
                semantic_name: "é",
                semantic_code_points: E_ACUTE,
            },
            Binding {
                authored: Range::new(7, 14),
                semantic_name: "e\u{301}",
                semantic_code_points: E_COMBINING,
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
    PositiveFixture {
        id: "non-strict-let-after-comma",
        source: "var a, let;",
        bindings: &[
            Binding {
                authored: Range::new(4, 5),
                semantic_name: "a",
                semantic_code_points: A,
            },
            Binding {
                authored: Range::new(7, 10),
                semantic_name: "let",
                semantic_code_points: LET,
            },
        ],
        terminator: Terminator::AuthoredSemicolon,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContributorFixture {
    source: &'static str,
    contributors: &'static [Range],
}

const CONTRIBUTOR_FIXTURES: &[ContributorFixture] = &[
    ContributorFixture {
        source: "var a,a; let x=a;",
        contributors: &[Range::new(4, 5), Range::new(6, 7)],
    },
    ContributorFixture {
        source: r"var a,\u0061; let x=a;",
        contributors: &[Range::new(4, 5), Range::new(6, 12)],
    },
    ContributorFixture {
        source: "var a,b; var a; let x=a;",
        contributors: &[Range::new(4, 5), Range::new(13, 14)],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Disposition {
    SelectedAcceptedIncomplete,
    StaticRejected {
        rule_id: &'static str,
        subject: Range,
    },
    DefinitiveGrammarRejection {
        subject: Range,
    },
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoundaryFixture {
    source: &'static str,
    disposition: Disposition,
}

const BOUNDARY_FIXTURES: &[BoundaryFixture] = &[
    BoundaryFixture {
        source: "var a, if;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        source: r"var a, \u0069f;",
        disposition: Disposition::StaticRejected {
            rule_id: "EE-04-R08",
            subject: Range::new(7, 14),
        },
    },
    BoundaryFixture {
        source: r"var a,\u{};",
        disposition: Disposition::DefinitiveGrammarRejection {
            subject: Range::new(6, 10),
        },
    },
    BoundaryFixture {
        source: "var a,",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        source: "var ,a;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        source: "var a,b = 1;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        source: "var a,{b}=c;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        source: "var a\nvar b;",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        source: "{ var a,b; }",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        source: "for (var a,b;;) {}",
        disposition: Disposition::UnsupportedCoverage,
    },
    BoundaryFixture {
        source: "var a, /*comment*/ b;",
        disposition: Disposition::UnsupportedCoverage,
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

const PRECEDENCE_FIXTURES: &[PrecedenceFixture] = &[
    PrecedenceFixture {
        source: r"let a; var a; var \u0069f;",
        primary: RuleWitness {
            rule_id: "EE-04-R08",
            subject: Range::new(18, 25),
        },
        lower_priority: RuleWitness {
            rule_id: "EE-36-R02",
            subject: Range::new(11, 12),
        },
    },
    PrecedenceFixture {
        source: r"let b; var a,b,\u0069f;",
        primary: RuleWitness {
            rule_id: "EE-04-R08",
            subject: Range::new(15, 22),
        },
        lower_priority: RuleWitness {
            rule_id: "EE-36-R02",
            subject: Range::new(13, 14),
        },
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CollisionFixture {
    source: &'static str,
    lexical: Range,
    var: Range,
    completing: Range,
}

const COLLISION_FIXTURES: &[CollisionFixture] = &[
    CollisionFixture {
        source: "let b; var a,b;",
        lexical: Range::new(4, 5),
        var: Range::new(13, 14),
        completing: Range::new(13, 14),
    },
    CollisionFixture {
        source: "let a,b; var b,a;",
        lexical: Range::new(6, 7),
        var: Range::new(13, 14),
        completing: Range::new(13, 14),
    },
    CollisionFixture {
        source: "var b,a; let a,b;",
        lexical: Range::new(13, 14),
        var: Range::new(6, 7),
        completing: Range::new(13, 14),
    },
    CollisionFixture {
        source: "var a,a; let a;",
        lexical: Range::new(13, 14),
        var: Range::new(4, 5),
        completing: Range::new(13, 14),
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FailedListFixture {
    source: &'static str,
    disposition: Disposition,
    committed_bindings: &'static [Range],
    committed_contributors: &'static [Range],
}

const FAILED_LIST_FIXTURES: &[FailedListFixture] = &[
    FailedListFixture {
        source: r"var a,\u{};",
        disposition: Disposition::DefinitiveGrammarRejection {
            subject: Range::new(6, 10),
        },
        committed_bindings: &[],
        committed_contributors: &[],
    },
    FailedListFixture {
        source: r"var a,b,\u{};",
        disposition: Disposition::DefinitiveGrammarRejection {
            subject: Range::new(8, 12),
        },
        committed_bindings: &[],
        committed_contributors: &[],
    },
    FailedListFixture {
        source: "var a,b,",
        disposition: Disposition::UnsupportedCoverage,
        committed_bindings: &[],
        committed_contributors: &[],
    },
    FailedListFixture {
        source: "var a,b = 1;",
        disposition: Disposition::UnsupportedCoverage,
        committed_bindings: &[],
        committed_contributors: &[],
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
    source
        .get(range.start..range.end)
        .expect("fixture range must be exact UTF-8")
}

fn assert_anchor(source: &str, range: Range, source_id: u64) {
    let source = SourceText::new(SourceId::new(source_id), source.to_owned());
    let anchor = source
        .anchor(range.start, range.end)
        .expect("fixture range must form SourceAnchor");
    assert_eq!(anchor.range().start(), range.start);
    assert_eq!(anchor.range().end(), range.end);
}

#[test]
fn authority_chain_is_additive_and_historical_frontiers_stay_immutable() {
    assert_eq!(ISSUE_ID, 322);
    assert_eq!(ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(UNICODE_VERSION, "17.0.0");
    assert!(HISTORICAL_VAR_ORACLE.contains("SELECTED_VARIABLE_SINGLE_BINDING_ONLY: bool = true"));
    assert!(HISTORICAL_VAR_ORACLE.contains("UnsupportedReason::MultipleVarDeclarators"));
    assert!(HISTORICAL_VAR_COMPOSITION.contains("VariableStatement"));
    assert!(HISTORICAL_COMPLETION.contains("assert_eq!(required.len(), 9);"));
    assert!(HISTORICAL_COMPLETION.contains("assert_eq!(non_triggering.len(), 184);"));
    assert!(HISTORICAL_CORRESPONDENCE.contains("SameSourceSelectedVarNameContributors"));
    assert!(HISTORICAL_EOF_ASI.contains("UnsupportedReason::MultipleVarDeclarators"));
    assert!(
        TIER_PRECEDENCE_AUTHORITY
            .contains("declaration_local_precedence_matrix_defeats_global_rule_and_byte_ordering")
    );
    assert!(GRAMMAR_EVIDENCE_AUTHORITY.contains("CandidateDefinitiveGrammarEvidence"));
}

#[test]
fn variable_declaration_list_is_an_inductive_theorem_not_a_fixture_count() {
    assert_eq!(
        VARIABLE_DECLARATION_LIST_BASE,
        "VariableDeclarationList : VariableDeclaration"
    );
    assert_eq!(
        VARIABLE_DECLARATION_LIST_SUCCESSOR,
        "VariableDeclarationList : VariableDeclarationList , VariableDeclaration"
    );
    assert_eq!(SELECTED_LIST_CARDINALITY, "1..N");
    assert_eq!(SUCCESSOR_FRONTIER_CARDINALITY, "2..N");
    assert_eq!(INDUCTIVE_INVARIANTS.len(), 7);
    assert!(
        INDUCTIVE_INVARIANTS
            .iter()
            .all(|invariant| !invariant.is_empty())
    );

    let boundary_counts: Vec<_> = ["historical-base", "two-declarators", "three-declarators"]
        .iter()
        .map(|id| {
            POSITIVE_FIXTURES
                .iter()
                .find(|fixture| fixture.id == *id)
                .expect("induction boundary fixture must exist")
                .bindings
                .len()
        })
        .collect();
    assert_eq!(boundary_counts, [1, 2, 3]);
}

#[test]
fn positive_fixtures_pin_authored_order_semantic_identity_and_one_terminator() {
    for (index, fixture) in POSITIVE_FIXTURES.iter().enumerate() {
        let mut previous_end = 0;
        for binding in fixture.bindings {
            assert_anchor(
                fixture.source,
                binding.authored,
                ISSUE_ID * 100 + index as u64,
            );
            assert!(binding.authored.start >= previous_end);
            assert!(!slice(fixture.source, binding.authored).is_empty());
            assert!(!binding.semantic_name.is_empty());
            assert!(!binding.semantic_code_points.is_empty());
            previous_end = binding.authored.end;
        }
    }
    let equal = POSITIVE_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "direct-escaped-equality")
        .unwrap();
    assert_eq!(
        equal.bindings[0].semantic_name,
        equal.bindings[1].semantic_name
    );
    assert_ne!(
        slice(equal.source, equal.bindings[0].authored),
        slice(equal.source, equal.bindings[1].authored)
    );
    let distinct = POSITIVE_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "no-unicode-normalization")
        .unwrap();
    assert_ne!(
        distinct.bindings[0].semantic_code_points,
        distinct.bindings[1].semantic_code_points
    );
}

#[test]
fn correspondence_is_only_a_314_contributor_domain_successor() {
    for (index, fixture) in CONTRIBUTOR_FIXTURES.iter().enumerate() {
        assert!(fixture.contributors.len() >= 2);
        let mut previous_end = 0;
        for contributor in fixture.contributors {
            assert_anchor(fixture.source, *contributor, ISSUE_ID * 1000 + index as u64);
            assert!(contributor.start >= previous_end);
            previous_end = contributor.end;
        }
    }
    assert!(HISTORICAL_CORRESPONDENCE.contains(concat!(
        "same-source selected var contributor != runtime Resolve",
        "Binding target"
    )));
    assert!(
        HISTORICAL_CORRESPONDENCE
            .contains("authored VariableDeclaration provenance != unique runtime binding identity")
    );
}

#[test]
fn later_declarator_grammar_and_static_routes_stay_distinct() {
    let escaped = BOUNDARY_FIXTURES
        .iter()
        .find(|fixture| fixture.source == r"var a, \u0069f;")
        .unwrap();
    assert_eq!(
        escaped.disposition,
        Disposition::StaticRejected {
            rule_id: "EE-04-R08",
            subject: Range::new(7, 14)
        }
    );
    assert_eq!(slice(escaped.source, Range::new(7, 14)), r"\u0069f");
    let malformed = BOUNDARY_FIXTURES
        .iter()
        .find(|fixture| fixture.source == r"var a,\u{};")
        .unwrap();
    assert_eq!(
        malformed.disposition,
        Disposition::DefinitiveGrammarRejection {
            subject: Range::new(6, 10)
        }
    );
    assert!(
        BOUNDARY_FIXTURES
            .iter()
            .any(|fixture| fixture.source == "var a, if;"
                && fixture.disposition == Disposition::UnsupportedCoverage)
    );
    assert!(
        POSITIVE_FIXTURES
            .iter()
            .any(|fixture| fixture.source == "var a, let;")
    );
}

#[test]
fn tier_priority_beats_global_authored_byte_order() {
    for fixture in PRECEDENCE_FIXTURES {
        assert_eq!(fixture.primary.rule_id, "EE-04-R08");
        assert_eq!(fixture.lower_priority.rule_id, "EE-36-R02");
        assert_anchor(fixture.source, fixture.primary.subject, ISSUE_ID);
        assert_anchor(fixture.source, fixture.lower_priority.subject, ISSUE_ID + 1);
    }
    let race = &PRECEDENCE_FIXTURES[1];
    assert!(race.primary.subject.start > race.lower_priority.subject.start);
}

#[test]
fn ee36_r02_collision_selection_extends_authored_order_inside_the_list() {
    for fixture in COLLISION_FIXTURES {
        assert_anchor(fixture.source, fixture.lexical, ISSUE_ID);
        assert_anchor(fixture.source, fixture.var, ISSUE_ID + 1);
        assert_anchor(fixture.source, fixture.completing, ISSUE_ID + 2);
    }
    assert_eq!(
        slice(COLLISION_FIXTURES[1].source, COLLISION_FIXTURES[1].var),
        "b"
    );
    assert_eq!(COLLISION_FIXTURES[3].var, Range::new(4, 5));
}

#[test]
fn incomplete_list_never_commits_partial_success() {
    for fixture in FAILED_LIST_FIXTURES {
        assert!(fixture.committed_bindings.is_empty());
        assert!(fixture.committed_contributors.is_empty());
        assert_ne!(fixture.disposition, Disposition::SelectedAcceptedIncomplete);
    }
    assert!(FAILED_LIST_FIXTURES.iter().any(|fixture| matches!(
        fixture.disposition,
        Disposition::DefinitiveGrammarRejection { .. }
    )));
    assert!(
        FAILED_LIST_FIXTURES
            .iter()
            .any(|fixture| fixture.disposition == Disposition::UnsupportedCoverage)
    );
}

#[test]
fn terminator_and_asi_meaning_do_not_depend_on_list_cardinality() {
    let authored = POSITIVE_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "two-declarators")
        .unwrap();
    let automatic = POSITIVE_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "two-declarators-eof")
        .unwrap();
    assert_eq!(authored.bindings, automatic.bindings);
    assert_eq!(authored.terminator, Terminator::AuthoredSemicolon);
    assert_eq!(automatic.terminator, Terminator::AutomaticAtEof);

    assert!(
        POSITIVE_FIXTURES
            .iter()
            .any(|fixture| fixture.source == "var a\n, b;"
                && fixture.terminator == Terminator::AuthoredSemicolon)
    );
    assert!(
        POSITIVE_FIXTURES
            .iter()
            .any(|fixture| fixture.source == "var a,\n b"
                && fixture.terminator == Terminator::AutomaticAtEof)
    );
    assert!(
        BOUNDARY_FIXTURES
            .iter()
            .any(|fixture| fixture.source == "var a\nvar b;"
                && fixture.disposition == Disposition::UnsupportedCoverage)
    );
    assert!(
        BOUNDARY_FIXTURES
            .iter()
            .any(|fixture| fixture.source == "var a,"
                && fixture.disposition == Disposition::UnsupportedCoverage)
    );
}

#[test]
fn frozen_rule_reachability_and_positive_partition_remain_unchanged() {
    let active_ids: BTreeSet<_> = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::NormativeRule)
        .map(|rule| rule.id)
        .collect();
    for rule_id in CURRENT_REQUIRED_RULE_IDS {
        assert!(active_ids.contains(rule_id));
    }
    assert_eq!(CURRENT_REQUIRED_RULE_IDS.len(), 9);
    assert_eq!(RULE_UNITS.len(), 193);
    assert_eq!(RULE_UNITS.len() - CURRENT_REQUIRED_RULE_IDS.len(), 184);
    assert_eq!(partition(0, 0), PositivePartition::HistoricalFlat);
    assert_eq!(partition(1, 0), PositivePartition::BlockEnabledWithoutVar);
    for count in [1, 2, 3, 17] {
        assert_eq!(partition(0, count), PositivePartition::VariableEnabled);
        assert_eq!(partition(2, count), PositivePartition::VariableEnabled);
    }
}

#[test]
fn validation_remains_candidate_independent_and_representation_neutral() {
    for forbidden in [
        concat!("selected_lexical_", "slice::"),
        concat!("selected_static_", "semantics::"),
        concat!("selected_qualification_", "integration::"),
        concat!("selected_variable_statement_name_", "correspondence::"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("evaluate_selected_", "static_semantics"),
        concat!("attempt_selected_", "qualification"),
        concat!("SelectedVariable", "Statement {"),
        concat!("SelectedVariable", "Declaration"),
        concat!("Vec<SelectedVariable", "Binding>"),
        concat!("parse_variable_", "declaration_list"),
        concat!("collect_var_", "contributors"),
        concat!("first_", "collision"),
        concat!("Environment", "Record"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "forbidden validation dependency or representation commitment: {forbidden}"
        );
    }
}
