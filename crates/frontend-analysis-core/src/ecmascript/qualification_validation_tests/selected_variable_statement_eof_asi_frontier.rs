//! Candidate-independent successor oracle for selected top-level
//! `VariableStatement` EOF-only ASI composition (Issue #318).
//!
//! Historical #306/#308/#312 validation remains immutable. This module fixes
//! only the expected successor-frontier meaning and source provenance; it does
//! not call production recognition, static semantics, qualification, relation,
//! or runtime behavior and does not prescribe a future production representation.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

use super::inventory::{RULE_UNITS, RuleUnitKind};

const ISSUE_ID: u64 = 318;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const HISTORICAL_VAR_ORACLE: &str =
    include_str!("../qualification_selected_top_level_variable_statement_validation_tests.rs");
const HISTORICAL_VAR_COMPOSITION_ORACLE: &str = include_str!(
    "../qualification_selected_top_level_variable_statement_composition_validation_tests.rs"
);
const HISTORICAL_COMPLETION: &str = include_str!("selected_variable_statement_slice_completion.rs");
const EOF_ASI_AUTHORITY: &str = include_str!("../qualification_selected_eof_asi_validation_tests.rs");
const THIS_SOURCE: &str = include_str!("selected_variable_statement_eof_asi_frontier.rs");

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
struct SyntheticSemicolonExpectation {
    decision_offset: usize,
    authored_range: Option<ByteRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedContributor {
    semantic_name: &'static str,
    authored: ByteRange,
}

const DIRECT_A: &[ExpectedContributor] = &[ExpectedContributor {
    semantic_name: "a",
    authored: ByteRange::new(4, 5),
}];
const ESCAPED_A: &[ExpectedContributor] = &[ExpectedContributor {
    semantic_name: "a",
    authored: ByteRange::new(4, 10),
}];
const PRECOMPOSED_E_ACUTE: &[ExpectedContributor] = &[ExpectedContributor {
    semantic_name: "é",
    authored: ByteRange::new(4, 6),
}];
const DECOMPOSED_E_ACUTE: &[ExpectedContributor] = &[ExpectedContributor {
    semantic_name: "e\u{301}",
    authored: ByteRange::new(4, 7),
}];
const REPEATED_A: &[ExpectedContributor] = &[
    ExpectedContributor {
        semantic_name: "a",
        authored: ByteRange::new(4, 5),
    },
    ExpectedContributor {
        semantic_name: "a",
        authored: ByteRange::new(11, 12),
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EofVarFixture {
    id: &'static str,
    source: &'static str,
    final_statement: ByteRange,
    binding: ByteRange,
    semantic_name: &'static str,
    contributors: &'static [ExpectedContributor],
    trailing_trivia: ByteRange,
    required_authored_semicolons: &'static [ByteRange],
    synthetic_semicolon: SyntheticSemicolonExpectation,
}

const EOF_VAR_FIXTURES: &[EofVarFixture] = &[
    EofVarFixture {
        id: "VAR-EOF-ASI-DIRECT-001",
        source: "var a",
        final_statement: ByteRange::new(0, 5),
        binding: ByteRange::new(4, 5),
        semantic_name: "a",
        contributors: DIRECT_A,
        trailing_trivia: ByteRange::new(5, 5),
        required_authored_semicolons: &[],
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 5,
            authored_range: None,
        },
    },
    EofVarFixture {
        id: "VAR-EOF-ASI-TRAILING-TRIVIA-001",
        source: "var a   \t\n\r\u{2028}\u{2029}\u{00A0}\u{FEFF}",
        final_statement: ByteRange::new(0, 5),
        binding: ByteRange::new(4, 5),
        semantic_name: "a",
        contributors: DIRECT_A,
        trailing_trivia: ByteRange::new(5, 22),
        required_authored_semicolons: &[],
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 22,
            authored_range: None,
        },
    },
    EofVarFixture {
        id: "VAR-EOF-ASI-ESCAPED-001",
        source: r"var \u0061",
        final_statement: ByteRange::new(0, 10),
        binding: ByteRange::new(4, 10),
        semantic_name: "a",
        contributors: ESCAPED_A,
        trailing_trivia: ByteRange::new(10, 10),
        required_authored_semicolons: &[],
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 10,
            authored_range: None,
        },
    },
    EofVarFixture {
        id: "VAR-EOF-ASI-PRECOMPOSED-001",
        source: "var é",
        final_statement: ByteRange::new(0, 6),
        binding: ByteRange::new(4, 6),
        semantic_name: "é",
        contributors: PRECOMPOSED_E_ACUTE,
        trailing_trivia: ByteRange::new(6, 6),
        required_authored_semicolons: &[],
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 6,
            authored_range: None,
        },
    },
    EofVarFixture {
        id: "VAR-EOF-ASI-DECOMPOSED-001",
        source: "var e\u{301}",
        final_statement: ByteRange::new(0, 7),
        binding: ByteRange::new(4, 7),
        semantic_name: "e\u{301}",
        contributors: DECOMPOSED_E_ACUTE,
        trailing_trivia: ByteRange::new(7, 7),
        required_authored_semicolons: &[],
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 7,
            authored_range: None,
        },
    },
    EofVarFixture {
        id: "VAR-EOF-ASI-REPEATED-001",
        source: "var a; var a",
        final_statement: ByteRange::new(7, 12),
        binding: ByteRange::new(11, 12),
        semantic_name: "a",
        contributors: REPEATED_A,
        trailing_trivia: ByteRange::new(12, 12),
        required_authored_semicolons: &[ByteRange::new(5, 6)],
        synthetic_semicolon: SyntheticSemicolonExpectation {
            decision_offset: 12,
            authored_range: None,
        },
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AuthoredSemicolonControl {
    source: &'static str,
    binding: ByteRange,
    semantic_name: &'static str,
    contributors: &'static [ExpectedContributor],
    semicolon: ByteRange,
}

const DIRECT_AUTHORED_CONTROL: AuthoredSemicolonControl = AuthoredSemicolonControl {
    source: "var a;",
    binding: ByteRange::new(4, 5),
    semantic_name: "a",
    contributors: DIRECT_A,
    semicolon: ByteRange::new(5, 6),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StaticRejectionFixture {
    id: &'static str,
    source: &'static str,
    rule_id: &'static str,
    subject: ByteRange,
}

const STATIC_REJECTION_FIXTURES: &[StaticRejectionFixture] = &[
    StaticRejectionFixture {
        id: "VAR-EOF-ASI-EE36-R02-001",
        source: "let a; var a",
        rule_id: "EE-36-R02",
        subject: ByteRange::new(11, 12),
    },
    StaticRejectionFixture {
        id: "VAR-EOF-ASI-EE04-R08-001",
        source: r"var \u0069f",
        rule_id: "EE-04-R08",
        subject: ByteRange::new(4, 11),
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum UnsupportedReason {
    RequiresNonEofAsi,
    IncompleteBindingIdentifier,
    MultipleVarDeclarators,
    VarInitializer,
    VarDestructuring,
    ForVar,
    BlockVar,
    CommentsOutOfScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UnsupportedFixture {
    id: &'static str,
    source: &'static str,
    reason: UnsupportedReason,
}

const UNSUPPORTED_FIXTURES: &[UnsupportedFixture] = &[
    UnsupportedFixture {
        id: "VAR-EOF-ASI-NON-EOF-001",
        source: "var a\nvar b;",
        reason: UnsupportedReason::RequiresNonEofAsi,
    },
    UnsupportedFixture {
        id: "VAR-EOF-ASI-INCOMPLETE-BINDING-001",
        source: "var",
        reason: UnsupportedReason::IncompleteBindingIdentifier,
    },
    UnsupportedFixture {
        id: "VAR-EOF-ASI-MULTI-DECLARATOR-001",
        source: "var a, b;",
        reason: UnsupportedReason::MultipleVarDeclarators,
    },
    UnsupportedFixture {
        id: "VAR-EOF-ASI-INITIALIZER-001",
        source: "var a = 1;",
        reason: UnsupportedReason::VarInitializer,
    },
    UnsupportedFixture {
        id: "VAR-EOF-ASI-DESTRUCTURING-001",
        source: "var {a} = b;",
        reason: UnsupportedReason::VarDestructuring,
    },
    UnsupportedFixture {
        id: "VAR-EOF-ASI-FOR-VAR-001",
        source: "for (var a;;) {}",
        reason: UnsupportedReason::ForVar,
    },
    UnsupportedFixture {
        id: "VAR-EOF-ASI-BLOCK-VAR-001",
        source: "{ var a; }",
        reason: UnsupportedReason::BlockVar,
    },
    UnsupportedFixture {
        id: "VAR-EOF-ASI-COMMENT-001",
        source: "var a/*comment*/",
        reason: UnsupportedReason::CommentsOutOfScope,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedGrammarPositivePartition {
    HistoricalFlat,
    BlockEnabledWithoutVar,
    VariableEnabled,
}

fn partition_selected_grammar_positive(
    block_count: usize,
    variable_statement_count: usize,
) -> SelectedGrammarPositivePartition {
    if variable_statement_count > 0 {
        SelectedGrammarPositivePartition::VariableEnabled
    } else if block_count > 0 {
        SelectedGrammarPositivePartition::BlockEnabledWithoutVar
    } else {
        SelectedGrammarPositivePartition::HistoricalFlat
    }
}

fn slice(text: &str, range: ByteRange) -> &str {
    text.get(range.start..range.end)
        .unwrap_or_else(|| panic!("range {range:?} must be a UTF-8 boundary in {text:?}"))
}

fn assert_source_anchor(source: &str, range: ByteRange, source_id: u64) {
    let source = SourceText::new(SourceId::new(source_id), source.to_owned());
    let anchor = source
        .anchor(range.start, range.end)
        .unwrap_or_else(|| panic!("{range:?} must be a valid authored source range"));
    assert_eq!(anchor.range().start(), range.start);
    assert_eq!(anchor.range().end(), range.end);
}

#[test]
fn fixed_authority_chain_is_additive_and_historical_oracles_remain_immutable() {
    assert_eq!(ISSUE_ID, 318);
    assert_eq!(ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(UNICODE_VERSION, "17.0.0");

    assert!(EOF_ASI_AUTHORITY.contains("Candidate-independent EOF-only ASI validation for Issue #233"));
    assert!(EOF_ASI_AUTHORITY.contains("authored_range: None"));
    assert!(EOF_ASI_AUTHORITY.contains("RequiresNonEofAsi"));

    assert!(HISTORICAL_VAR_ORACLE.contains("SELECTED_VARIABLE_EOF_ASI: bool = false"));
    assert!(HISTORICAL_VAR_ORACLE.contains("var-eof-asi-deferred"));
    assert!(HISTORICAL_VAR_ORACLE.contains("UnsupportedReason::EofAsiDeferred"));

    assert!(HISTORICAL_VAR_COMPOSITION_ORACLE.contains("var-eof-asi-remains-deferred"));
    assert!(
        HISTORICAL_VAR_COMPOSITION_ORACLE
            .contains("authored_var_semicolon_does_not_widen_var_eof_asi")
    );
    assert!(HISTORICAL_VAR_COMPOSITION_ORACLE.contains(
        "authored-var-semicolon-composes-with-existing-final-lexical-eof-asi"
    ));

    assert!(HISTORICAL_COMPLETION.contains("immutable historical completion authority"));
    assert!(HISTORICAL_COMPLETION.contains("assert_eq!(required.len(), 9);"));
    assert!(HISTORICAL_COMPLETION.contains("assert_eq!(non_triggering.len(), 184);"));
    assert!(HISTORICAL_COMPLETION.contains("SelectedGrammarPositivePartition::VariableEnabled"));
}

#[test]
fn eof_only_var_fixtures_preserve_authored_binding_and_synthetic_provenance() {
    for (index, fixture) in EOF_VAR_FIXTURES.iter().enumerate() {
        assert_source_anchor(fixture.source, fixture.binding, ISSUE_ID * 100 + index as u64);
        assert_eq!(
            slice(fixture.source, fixture.binding),
            slice(
                fixture.source,
                fixture
                    .contributors
                    .last()
                    .expect("positive var fixture must retain a contributor")
                    .authored
            )
        );
        assert_eq!(
            fixture
                .contributors
                .last()
                .expect("positive var fixture must retain a contributor")
                .semantic_name,
            fixture.semantic_name
        );
        assert_eq!(
            slice(fixture.source, fixture.final_statement),
            &fixture.source[fixture.final_statement.start..fixture.final_statement.end]
        );
        assert_eq!(fixture.synthetic_semicolon.authored_range, None);
        assert_eq!(
            fixture.synthetic_semicolon.decision_offset,
            fixture.source.len(),
            "EOF insertion decision belongs at actual authored EOF"
        );
        assert_eq!(fixture.trailing_trivia.end, fixture.source.len());
        assert!(fixture.final_statement.end <= fixture.trailing_trivia.start);
        for semicolon in fixture.required_authored_semicolons {
            assert_eq!(slice(fixture.source, *semicolon), ";");
        }
    }
}

#[test]
fn trailing_trivia_is_outside_binding_and_final_statement_authored_bytes() {
    let fixture = EOF_VAR_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "VAR-EOF-ASI-TRAILING-TRIVIA-001")
        .expect("trailing-trivia fixture");

    assert_eq!(slice(fixture.source, fixture.binding), "a");
    assert_eq!(slice(fixture.source, fixture.final_statement), "var a");
    assert_eq!(fixture.trailing_trivia.start, fixture.final_statement.end);
    assert!(!slice(fixture.source, fixture.trailing_trivia).is_empty());
    assert!(
        slice(fixture.source, fixture.trailing_trivia)
            .chars()
            .all(|character| character.is_whitespace() || character == '\u{FEFF}')
    );
}

#[test]
fn direct_and_escaped_semantic_names_match_without_source_normalization() {
    let direct = EOF_VAR_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "VAR-EOF-ASI-DIRECT-001")
        .expect("direct fixture");
    let escaped = EOF_VAR_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "VAR-EOF-ASI-ESCAPED-001")
        .expect("escaped fixture");
    let precomposed = EOF_VAR_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "VAR-EOF-ASI-PRECOMPOSED-001")
        .expect("precomposed fixture");
    let decomposed = EOF_VAR_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "VAR-EOF-ASI-DECOMPOSED-001")
        .expect("decomposed fixture");

    assert_eq!(direct.semantic_name, escaped.semantic_name);
    assert_ne!(slice(direct.source, direct.binding), slice(escaped.source, escaped.binding));
    assert_ne!(precomposed.semantic_name, decomposed.semantic_name);
    assert_ne!(
        slice(precomposed.source, precomposed.binding),
        slice(decomposed.source, decomposed.binding)
    );
}

#[test]
fn eof_and_authored_semicolon_forms_preserve_binding_and_contributor_meaning() {
    let eof = EOF_VAR_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "VAR-EOF-ASI-DIRECT-001")
        .expect("direct EOF fixture");

    assert_eq!(slice(eof.source, eof.binding), slice(DIRECT_AUTHORED_CONTROL.source, DIRECT_AUTHORED_CONTROL.binding));
    assert_eq!(eof.semantic_name, DIRECT_AUTHORED_CONTROL.semantic_name);
    assert_eq!(eof.contributors, DIRECT_AUTHORED_CONTROL.contributors);
    assert_eq!(slice(DIRECT_AUTHORED_CONTROL.source, DIRECT_AUTHORED_CONTROL.semicolon), ";");
    assert_eq!(eof.synthetic_semicolon.authored_range, None);
}

#[test]
fn repeated_var_retains_every_authored_contributor_in_order() {
    let fixture = EOF_VAR_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "VAR-EOF-ASI-REPEATED-001")
        .expect("repeated-var fixture");

    assert_eq!(fixture.contributors.len(), 2);
    assert_eq!(fixture.contributors, REPEATED_A);
    assert_eq!(slice(fixture.source, fixture.contributors[0].authored), "a");
    assert_eq!(slice(fixture.source, fixture.contributors[1].authored), "a");
    assert!(fixture.contributors[0].authored.start < fixture.contributors[1].authored.start);
}

#[test]
fn non_eof_and_deferred_var_dimensions_remain_outside_successor_frontier() {
    assert!(UNSUPPORTED_FIXTURES.iter().any(|fixture| {
        fixture.source == "var a\nvar b;" && fixture.reason == UnsupportedReason::RequiresNonEofAsi
    }));

    let observed: BTreeSet<_> = UNSUPPORTED_FIXTURES
        .iter()
        .map(|fixture| fixture.reason)
        .collect();
    assert!(observed.contains(&UnsupportedReason::MultipleVarDeclarators));
    assert!(observed.contains(&UnsupportedReason::VarInitializer));
    assert!(observed.contains(&UnsupportedReason::VarDestructuring));
    assert!(observed.contains(&UnsupportedReason::ForVar));
    assert!(observed.contains(&UnsupportedReason::BlockVar));
    assert!(observed.contains(&UnsupportedReason::CommentsOutOfScope));
    assert!(observed.contains(&UnsupportedReason::IncompleteBindingIdentifier));
}

#[test]
fn eof_asi_cannot_launder_existing_static_rejections() {
    for (index, fixture) in STATIC_REJECTION_FIXTURES.iter().enumerate() {
        assert_source_anchor(fixture.source, fixture.subject, ISSUE_ID * 1000 + index as u64);
        match fixture.rule_id {
            "EE-36-R02" => {
                assert!(HISTORICAL_VAR_ORACLE.contains("EE-36-R02"));
                assert_eq!(slice(fixture.source, fixture.subject), "a");
            }
            "EE-04-R08" => {
                assert!(HISTORICAL_VAR_ORACLE.contains("EE-04-R08"));
                assert_eq!(slice(fixture.source, fixture.subject), r"\u0069f");
            }
            other => panic!("unexpected static rule {other}"),
        }
    }
}

#[test]
fn successor_frontier_keeps_frozen_rule_reachability_exactly_nine_of_193() {
    let required: BTreeSet<_> = CURRENT_REQUIRED_RULE_IDS.iter().copied().collect();
    assert_eq!(required.len(), 9);

    let mut seen = BTreeSet::new();
    let mut active = 0usize;
    let mut inactive = 0usize;
    let mut required_seen = BTreeSet::new();
    let mut unchanged_non_triggering = BTreeSet::new();

    for rule in RULE_UNITS {
        assert!(seen.insert(rule.id), "duplicate frozen rule {}", rule.id);
        match rule.kind {
            RuleUnitKind::NormativeRule => active += 1,
            RuleUnitKind::EnvelopeInactiveRule => inactive += 1,
            RuleUnitKind::ExpansionSentinel => panic!("no frozen expansion sentinel may remain"),
        }

        if required.contains(rule.id) {
            assert_eq!(rule.kind, RuleUnitKind::NormativeRule);
            assert!(required_seen.insert(rule.id));
        } else {
            assert!(unchanged_non_triggering.insert(rule.id));
        }
    }

    assert_eq!(active, 183);
    assert_eq!(inactive, 10);
    assert_eq!(seen.len(), 193);
    assert_eq!(required_seen, required);
    assert_eq!(unchanged_non_triggering.len(), 184);
    assert_eq!(required_seen.len() + unchanged_non_triggering.len(), 193);

    for rule_id in CURRENT_REQUIRED_RULE_IDS {
        assert!(
            HISTORICAL_COMPLETION.contains(&format!("\"{rule_id}\"")),
            "successor reachability must be inherited from explicit #312 authority"
        );
    }
}

#[test]
fn successor_sources_stay_inside_existing_three_way_positive_partition() {
    assert_eq!(
        partition_selected_grammar_positive(0, 0),
        SelectedGrammarPositivePartition::HistoricalFlat
    );
    assert_eq!(
        partition_selected_grammar_positive(1, 0),
        SelectedGrammarPositivePartition::BlockEnabledWithoutVar
    );
    for fixture in EOF_VAR_FIXTURES {
        assert!(!fixture.source.is_empty());
        assert_eq!(
            partition_selected_grammar_positive(0, fixture.contributors.len()),
            SelectedGrammarPositivePartition::VariableEnabled
        );
    }

    assert!(
        HISTORICAL_COMPLETION.contains("CurrentSelectedGrammarPositiveSourceSpace")
            || HISTORICAL_COMPLETION
                .contains("current_selected_grammar_positive_source_space_is_exact_disjoint_union")
    );
}

#[test]
fn zero_reference_var_source_has_no_relation_or_runtime_claim_in_successor_oracle() {
    let fixture = EOF_VAR_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "VAR-EOF-ASI-DIRECT-001")
        .expect("direct zero-reference fixture");

    assert_eq!(fixture.contributors, DIRECT_A);
    assert!(!THIS_SOURCE.contains("ResolveBinding target ="));
    assert!(!THIS_SOURCE.contains("runtime binding identity ="));
}

#[test]
fn oracle_has_no_production_expected_result_dependency() {
    for (left, right) in [
        ("recognize_selected_lexical_", "slice("),
        ("evaluate_selected_", "static_semantics("),
        ("evaluate_selected_one_level_block_static_", "semantics("),
        ("evaluate_selected_variable_statement_static_", "semantics("),
        ("attempt_selected_", "qualification("),
        ("analyze_selected_binding_", "scope("),
        ("analyze_selected_one_level_block_binding_", "scope("),
        ("analyze_selected_variable_statement_name_", "correspondence("),
    ] {
        let forbidden = format!("{left}{right}");
        assert!(
            !THIS_SOURCE.contains(&forbidden),
            "#318 successor oracle must not call production owner {forbidden}"
        );
    }
}
