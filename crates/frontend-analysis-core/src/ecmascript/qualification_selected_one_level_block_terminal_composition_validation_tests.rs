//! Candidate-independent one-level selected Block terminal/source-composition
//! validation for Issue #297.
//!
//! This module materializes only relationships between already-accepted
//! validation authorities. It deliberately does not call production lexical,
//! static-semantics, aggregate qualification, Binding / Scope, or runtime code
//! to derive expected meaning.

use crate::{SourceId, SourceText};

const ISSUE_ID: u64 = 297;
const GRAMMAR_EVIDENCE_ORACLE: &str =
    include_str!("qualification_grammar_evidence_validation_tests.rs");
const GRAMMAR_POLICY_ORACLE: &str =
    include_str!("qualification_grammar_rejection_policy_validation_tests.rs");
const EOF_ASI_ORACLE: &str = include_str!("qualification_selected_eof_asi_validation_tests.rs");
const BLOCK_ORACLE: &str =
    include_str!("qualification_selected_one_level_block_validation_tests.rs");
const BLOCK_COMPOSITION_ORACLE: &str =
    include_str!("qualification_selected_one_level_block_composition_validation_tests.rs");
const HISTORICAL_COMPLETION_ORACLE: &str =
    include_str!("qualification_validation_tests/selected_slice_completion.rs");
const QUALIFICATION_SOURCE: &str = include_str!("qualification.rs");
const SELECTED_AGGREGATE_SOURCE: &str =
    include_str!(concat!("selected_", "qualification_integration.rs"));
const THIS_SOURCE: &str =
    include_str!("qualification_selected_one_level_block_terminal_composition_validation_tests.rs");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ByteRange {
    start: usize,
    end: usize,
}

impl ByteRange {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn fragment(self, source: &str) -> &str {
        assert!(self.start <= self.end);
        assert!(self.end <= source.len());
        assert!(source.is_char_boundary(self.start));
        assert!(source.is_char_boundary(self.end));
        source
            .get(self.start..self.end)
            .expect("validated range must slice authoritative UTF-8 source")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Grammar {
        subject: ByteRange,
        fragment: &'static str,
    },
    StaticSemanticsRejected {
        rule_id: &'static str,
        subject: ByteRange,
        fragment: &'static str,
    },
    UnsupportedCoverage,
    SelectedCandidateAccepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OutcomeFixture {
    source: &'static str,
    outcome: ExpectedOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EvidencePoint {
    rule_id: &'static str,
    subject: ByteRange,
    fragment: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KeywordAdjacentFixture {
    source: &'static str,
    keyword: &'static str,
    outcome: ExpectedOutcome,
}

const KEYWORD_ADJACENT_FIXTURES: &[KeywordAdjacentFixture] = &[
    KeywordAdjacentFixture {
        source: r"{ let\u{}; }",
        keyword: "let",
        outcome: ExpectedOutcome::Grammar {
            subject: ByteRange::new(5, 9),
            fragment: r"\u{}",
        },
    },
    KeywordAdjacentFixture {
        source: r"{ let\u{110000}; }",
        keyword: "let",
        outcome: ExpectedOutcome::Grammar {
            subject: ByteRange::new(5, 15),
            fragment: r"\u{110000}",
        },
    },
    KeywordAdjacentFixture {
        source: r"{ let\u0; }",
        keyword: "let",
        outcome: ExpectedOutcome::UnsupportedCoverage,
    },
    KeywordAdjacentFixture {
        source: r"{ let\u{61",
        keyword: "let",
        outcome: ExpectedOutcome::UnsupportedCoverage,
    },
    KeywordAdjacentFixture {
        source: r"{ const\u{}; }",
        keyword: "const",
        outcome: ExpectedOutcome::UnsupportedCoverage,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EofCompositionFixture {
    source: &'static str,
    block_close: ByteRange,
    final_declaration: ByteRange,
    trailing_trivia: ByteRange,
    automatic_semicolon_authored_range: Option<ByteRange>,
    outcome: ExpectedOutcome,
}

const EOF_COMPOSITION_FIXTURES: &[EofCompositionFixture] = &[
    EofCompositionFixture {
        source: "{ let a=1; } let b=2",
        block_close: ByteRange::new(11, 12),
        final_declaration: ByteRange::new(13, 20),
        trailing_trivia: ByteRange::new(20, 20),
        automatic_semicolon_authored_range: None,
        outcome: ExpectedOutcome::SelectedCandidateAccepted,
    },
    EofCompositionFixture {
        source: "{ let a=1; } const b",
        block_close: ByteRange::new(11, 12),
        final_declaration: ByteRange::new(13, 20),
        trailing_trivia: ByteRange::new(20, 20),
        automatic_semicolon_authored_range: None,
        outcome: ExpectedOutcome::StaticSemanticsRejected {
            rule_id: "EE-15-R03",
            subject: ByteRange::new(19, 20),
            fragment: "b",
        },
    },
    EofCompositionFixture {
        source: "{ let a=1; } let b=2 \n",
        block_close: ByteRange::new(11, 12),
        final_declaration: ByteRange::new(13, 20),
        trailing_trivia: ByteRange::new(20, 22),
        automatic_semicolon_authored_range: None,
        outcome: ExpectedOutcome::SelectedCandidateAccepted,
    },
];

const NON_EOF_ASI_FIXTURES: &[OutcomeFixture] = &[
    OutcomeFixture {
        source: "let a=1 { let b=2; }",
        outcome: ExpectedOutcome::UnsupportedCoverage,
    },
    OutcomeFixture {
        source: "{ let a=1; } let b=2\nconst c=3",
        outcome: ExpectedOutcome::UnsupportedCoverage,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IncompleteBlockFixture {
    source: &'static str,
    block_close: Option<ByteRange>,
    outcome: ExpectedOutcome,
    prohibited_committed_rule: Option<&'static str>,
}

const INCOMPLETE_BLOCK_FIXTURES: &[IncompleteBlockFixture] = &[
    IncompleteBlockFixture {
        source: "{ let x=1;",
        block_close: None,
        outcome: ExpectedOutcome::UnsupportedCoverage,
        prohibited_committed_rule: None,
    },
    IncompleteBlockFixture {
        source: "{ const x",
        block_close: None,
        outcome: ExpectedOutcome::UnsupportedCoverage,
        prohibited_committed_rule: Some("EE-15-R03"),
    },
    IncompleteBlockFixture {
        source: "{ let a=1; let a=2;",
        block_close: None,
        outcome: ExpectedOutcome::UnsupportedCoverage,
        prohibited_committed_rule: Some("EE-14-R01"),
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalDecision {
    StableAtSubjectEnd,
    RequiresSemicolonLookahead,
    RequiresEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IncompleteBlockGrammarFixture {
    source: &'static str,
    subject: ByteRange,
    fragment: &'static str,
    block_close: Option<ByteRange>,
    decision: TerminalDecision,
    outcome: ExpectedOutcome,
}

const INCOMPLETE_BLOCK_GRAMMAR_FIXTURES: &[IncompleteBlockGrammarFixture] = &[
    IncompleteBlockGrammarFixture {
        source: r"{ let \u{}",
        subject: ByteRange::new(6, 10),
        fragment: r"\u{}",
        block_close: None,
        decision: TerminalDecision::StableAtSubjectEnd,
        outcome: ExpectedOutcome::Grammar {
            subject: ByteRange::new(6, 10),
            fragment: r"\u{}",
        },
    },
    IncompleteBlockGrammarFixture {
        source: r"{ let \u0;",
        subject: ByteRange::new(6, 9),
        fragment: r"\u0",
        block_close: None,
        decision: TerminalDecision::RequiresSemicolonLookahead,
        outcome: ExpectedOutcome::Grammar {
            subject: ByteRange::new(6, 9),
            fragment: r"\u0",
        },
    },
    IncompleteBlockGrammarFixture {
        source: r"{ let \u0",
        subject: ByteRange::new(6, 9),
        fragment: r"\u0",
        block_close: None,
        decision: TerminalDecision::RequiresSemicolonLookahead,
        outcome: ExpectedOutcome::UnsupportedCoverage,
    },
    IncompleteBlockGrammarFixture {
        source: r"{ let \u{61",
        subject: ByteRange::new(6, 11),
        fragment: r"\u{61",
        block_close: None,
        decision: TerminalDecision::RequiresEof,
        outcome: ExpectedOutcome::Grammar {
            subject: ByteRange::new(6, 11),
            fragment: r"\u{61",
        },
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IncompleteBlockCompetitionFixture {
    source: &'static str,
    block_close: Option<ByteRange>,
    uncommitted_static_candidate: EvidencePoint,
    outcome: ExpectedOutcome,
}

const INCOMPLETE_BLOCK_COMPETITION_FIXTURES: &[IncompleteBlockCompetitionFixture] = &[
    IncompleteBlockCompetitionFixture {
        source: r"{ let a=1; let a=2; let \u{}",
        block_close: None,
        uncommitted_static_candidate: EvidencePoint {
            rule_id: "EE-14-R01",
            subject: ByteRange::new(15, 16),
            fragment: "a",
        },
        outcome: ExpectedOutcome::Grammar {
            subject: ByteRange::new(24, 28),
            fragment: r"\u{}",
        },
    },
    IncompleteBlockCompetitionFixture {
        source: r"{ const x; let \u{}",
        block_close: None,
        uncommitted_static_candidate: EvidencePoint {
            rule_id: "EE-15-R03",
            subject: ByteRange::new(8, 9),
            fragment: "x",
        },
        outcome: ExpectedOutcome::Grammar {
            subject: ByteRange::new(15, 19),
            fragment: r"\u{}",
        },
    },
];

const DEFERRED_EOF_AFTER_BLOCK: &[OutcomeFixture] = &[
    OutcomeFixture {
        source: r"{ let a=1; } let \u0",
        outcome: ExpectedOutcome::UnsupportedCoverage,
    },
    OutcomeFixture {
        source: r"{ let a=1; } let \u",
        outcome: ExpectedOutcome::UnsupportedCoverage,
    },
    OutcomeFixture {
        source: r"{ let a=1; } let \u{",
        outcome: ExpectedOutcome::UnsupportedCoverage,
    },
];

fn assert_anchor(source: &str, range: ByteRange, expected: &str) {
    let source_text = SourceText::new(SourceId::new(ISSUE_ID), source.to_owned());
    let anchor = source_text
        .anchor(range.start, range.end)
        .expect("fixture range must reconcile through Core SourceText");
    assert_eq!(anchor.fragment(), expected);
    assert_eq!(range.fragment(source), expected);
}

fn assert_expected_outcome_source(source: &str, outcome: ExpectedOutcome) {
    match outcome {
        ExpectedOutcome::Grammar { subject, fragment } => {
            assert_anchor(source, subject, fragment);
        }
        ExpectedOutcome::StaticSemanticsRejected {
            rule_id,
            subject,
            fragment,
        } => {
            assert!(!rule_id.is_empty());
            assert_anchor(source, subject, fragment);
        }
        ExpectedOutcome::UnsupportedCoverage | ExpectedOutcome::SelectedCandidateAccepted => {}
    }
}

fn source_section<'source>(
    source: &'source str,
    start_marker: &str,
    end_marker: &str,
) -> &'source str {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("missing authority section start {start_marker}"));
    let rest = &source[start..];
    let end = rest
        .find(end_marker)
        .unwrap_or_else(|| panic!("missing authority section end {end_marker}"));
    &rest[..end]
}

#[test]
fn authority_chain_is_present_and_candidate_independent() {
    let keyword_section = source_section(
        GRAMMAR_EVIDENCE_ORACLE,
        "const KEYWORD_BOUNDARY_EVIDENCE",
        "const EARLY_DEFINITIVE_EVIDENCE",
    );
    assert_eq!(
        keyword_section.matches("GrammarEvidenceFixture {").count(),
        2
    );
    assert!(keyword_section.contains(r#"source: r"let\u{};""#));
    assert!(keyword_section.contains(r#"source: r"let\u{110000};""#));
    assert!(keyword_section.contains("EmptyBraced"));
    assert!(keyword_section.contains("NonCodePointBraced"));
    assert!(!keyword_section.contains("ShortFixed"));
    assert!(!keyword_section.contains("UnclosedBraced"));
    assert!(!keyword_section.contains(r#"source: r"const"#));

    assert!(GRAMMAR_EVIDENCE_ORACLE.contains("CandidateDefinitiveGrammarEvidence"));
    assert!(GRAMMAR_POLICY_ORACLE.contains("RequiresLookahead(\";\")"));
    assert!(GRAMMAR_POLICY_ORACLE.contains("RequiresEof"));
    assert!(EOF_ASI_ORACLE.contains("EOF_SELECTED_FIXTURES"));
    assert!(EOF_ASI_ORACLE.contains("only_the_final_declaration_may_use_the_eof_only_policy"));
    assert!(EOF_ASI_ORACLE.contains("RequiresNonEofAsi"));
    assert!(EOF_ASI_ORACLE.contains("DeferredMalformedEscape"));
    assert!(BLOCK_ORACLE.contains("SELECTED_BLOCK_BODY"));
    assert!(BLOCK_ORACLE.contains("LexicalDeclaration+"));
    assert!(BLOCK_COMPOSITION_ORACLE.contains("SHORT_FIXED_DELIMITER_FIXTURES"));
    assert!(BLOCK_COMPOSITION_ORACLE.contains("FormedEscapeButUnclosedBlock"));
    assert!(HISTORICAL_COMPLETION_ORACLE.contains("SELECTED_TOP_LEVEL_GRAMMAR"));

    for forbidden in [
        concat!("recognize_selected_", "lexical_slice("),
        concat!("evaluate_selected_one_level_block_", "static_semantics("),
        concat!("attempt_selected_", "qualification("),
        concat!("analyze_selected_", "binding_scope("),
        concat!("QualificationOutcome::", "qualified("),
        concat!("CompleteQualification", "Witness {"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "#297 oracle must not derive expected meaning from production: {forbidden}"
        );
    }
}

#[test]
fn keyword_adjacent_block_embedding_preserves_only_existing_literal_let_ownership() {
    assert_eq!(KEYWORD_ADJACENT_FIXTURES.len(), 5);

    for fixture in KEYWORD_ADJACENT_FIXTURES {
        let prefix = format!("{{ {}", fixture.keyword);
        assert!(fixture.source.starts_with(&prefix));
        assert_expected_outcome_source(fixture.source, fixture.outcome);
    }

    assert!(matches!(
        KEYWORD_ADJACENT_FIXTURES[0].outcome,
        ExpectedOutcome::Grammar {
            subject: ByteRange { start: 5, end: 9 },
            fragment: r"\u{}"
        }
    ));
    assert!(matches!(
        KEYWORD_ADJACENT_FIXTURES[1].outcome,
        ExpectedOutcome::Grammar {
            subject: ByteRange { start: 5, end: 15 },
            fragment: r"\u{110000}"
        }
    ));

    for fixture in &KEYWORD_ADJACENT_FIXTURES[2..] {
        assert_eq!(fixture.outcome, ExpectedOutcome::UnsupportedCoverage);
    }
}

#[test]
fn completed_block_can_precede_only_a_final_top_level_eof_asi_declaration() {
    assert_eq!(EOF_COMPOSITION_FIXTURES.len(), 3);

    for fixture in EOF_COMPOSITION_FIXTURES {
        assert!(fixture.source.starts_with("{ let a=1; } "));
        assert_anchor(fixture.source, fixture.block_close, "}");
        assert_eq!(fixture.automatic_semicolon_authored_range, None);
        assert_eq!(
            fixture.final_declaration.fragment(fixture.source),
            match fixture.outcome {
                ExpectedOutcome::StaticSemanticsRejected { .. } => "const b",
                _ => "let b=2",
            }
        );
        assert_eq!(
            fixture.trailing_trivia.end,
            fixture.source.len(),
            "EOF decision occurs only after selected trailing trivia"
        );
        assert_expected_outcome_source(fixture.source, fixture.outcome);
    }

    assert_eq!(
        EOF_COMPOSITION_FIXTURES[2]
            .trailing_trivia
            .fragment(EOF_COMPOSITION_FIXTURES[2].source),
        " \n"
    );
}

#[test]
fn block_item_boundaries_do_not_authorize_non_eof_asi() {
    assert_eq!(NON_EOF_ASI_FIXTURES.len(), 2);
    assert!(EOF_ASI_ORACLE.contains("RequiresNonEofAsi"));

    for fixture in NON_EOF_ASI_FIXTURES {
        assert_eq!(fixture.outcome, ExpectedOutcome::UnsupportedCoverage);
        assert_expected_outcome_source(fixture.source, fixture.outcome);
    }

    assert!(NON_EOF_ASI_FIXTURES[0].source.contains(" { "));
    assert!(NON_EOF_ASI_FIXTURES[1].source.contains("\nconst "));
}

#[test]
fn incomplete_block_never_commits_positive_or_static_region_facts() {
    assert_eq!(INCOMPLETE_BLOCK_FIXTURES.len(), 3);

    for fixture in INCOMPLETE_BLOCK_FIXTURES {
        assert!(fixture.source.starts_with('{'));
        assert_eq!(fixture.block_close, None);
        assert_eq!(fixture.outcome, ExpectedOutcome::UnsupportedCoverage);
        assert_expected_outcome_source(fixture.source, fixture.outcome);

        if let Some(rule_id) = fixture.prohibited_committed_rule {
            assert!(matches!(rule_id, "EE-15-R03" | "EE-14-R01"));
        }
    }

    assert!(BLOCK_ORACLE.contains("EE-14-R01"));
    assert!(BLOCK_ORACLE.contains("block-local-duplicate"));
}

#[test]
fn incomplete_block_retains_only_already_owned_terminal_grammar_when_decision_is_satisfied() {
    assert_eq!(INCOMPLETE_BLOCK_GRAMMAR_FIXTURES.len(), 4);

    for fixture in INCOMPLETE_BLOCK_GRAMMAR_FIXTURES {
        assert!(fixture.source.starts_with("{ let "));
        assert_eq!(fixture.block_close, None);
        assert_anchor(fixture.source, fixture.subject, fixture.fragment);
        assert_expected_outcome_source(fixture.source, fixture.outcome);

        match fixture.decision {
            TerminalDecision::StableAtSubjectEnd => {
                assert_eq!(fixture.subject.end, fixture.source.len());
            }
            TerminalDecision::RequiresSemicolonLookahead => {
                let actual = fixture.source.as_bytes().get(fixture.subject.end).copied();
                match fixture.outcome {
                    ExpectedOutcome::Grammar { .. } => assert_eq!(actual, Some(b';')),
                    ExpectedOutcome::UnsupportedCoverage => assert_eq!(actual, None),
                    _ => panic!("short-fixed challenge has unexpected outcome"),
                }
            }
            TerminalDecision::RequiresEof => {
                assert_eq!(fixture.subject.end, fixture.source.len());
            }
        }
    }

    assert_eq!(
        INCOMPLETE_BLOCK_GRAMMAR_FIXTURES[0]
            .subject
            .fragment(INCOMPLETE_BLOCK_GRAMMAR_FIXTURES[0].source),
        r"\u{}"
    );
    assert_eq!(INCOMPLETE_BLOCK_GRAMMAR_FIXTURES[0].block_close, None);
}

#[test]
fn definitive_grammar_remains_primary_over_uncommitted_static_candidates() {
    assert_eq!(INCOMPLETE_BLOCK_COMPETITION_FIXTURES.len(), 2);

    for fixture in INCOMPLETE_BLOCK_COMPETITION_FIXTURES {
        assert!(fixture.source.starts_with('{'));
        assert_eq!(fixture.block_close, None);
        assert!(matches!(
            fixture.uncommitted_static_candidate.rule_id,
            "EE-14-R01" | "EE-15-R03"
        ));
        assert_anchor(
            fixture.source,
            fixture.uncommitted_static_candidate.subject,
            fixture.uncommitted_static_candidate.fragment,
        );
        assert_expected_outcome_source(fixture.source, fixture.outcome);

        let grammar_subject = match fixture.outcome {
            ExpectedOutcome::Grammar { subject, .. } => subject,
            _ => panic!("competition fixture must retain definitive Grammar as primary"),
        };
        assert_ne!(
            fixture.uncommitted_static_candidate.subject,
            grammar_subject
        );
        assert!(fixture.uncommitted_static_candidate.subject.end <= grammar_subject.start);
    }

    assert!(
        GRAMMAR_POLICY_ORACLE.contains(
            "grammar_primary_discards_tentative_selected_static_evidence_across_families"
        )
    );
    assert!(BLOCK_COMPOSITION_ORACLE.contains("GrammarVsBlockFixture"));
}

#[test]
fn eof_asi_after_block_does_not_repair_deferred_malformed_binding_sources() {
    assert_eq!(DEFERRED_EOF_AFTER_BLOCK.len(), 3);
    assert!(EOF_ASI_ORACLE.contains("DeferredMalformedEscape"));

    for fixture in DEFERRED_EOF_AFTER_BLOCK {
        assert!(fixture.source.starts_with("{ let a=1; } let "));
        assert!(!fixture.source.ends_with(';'));
        assert_eq!(fixture.outcome, ExpectedOutcome::UnsupportedCoverage);
        assert_expected_outcome_source(fixture.source, fixture.outcome);
    }
}

#[test]
fn terminal_composition_stays_below_completion_and_qualified_boundaries() {
    assert!(HISTORICAL_COMPLETION_ORACLE.contains("LexicalDeclaration+"));
    assert!(BLOCK_ORACLE.contains(
        "historical flat selected-slice completion != Block-enabled selected-slice completion"
    ));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(!SELECTED_AGGREGATE_SOURCE.contains("QualificationOutcome::qualified"));
    assert!(QUALIFICATION_SOURCE.contains("struct CompleteQualificationWitness"));
    assert!(!QUALIFICATION_SOURCE.contains("fn complete_qualification_witness("));
}
