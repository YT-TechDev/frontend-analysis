//! Candidate-independent cross-family / termination validation for Issue #229.
//!
//! #219/#222 own selected static-family evidence. #227 owns the bounded
//! source-backed malformed/non-CodePoint grammar subjects. This module fixes
//! only the project-local relationship between those existing authorities.
//! It deliberately does not call production lexical/static/aggregate code.

use crate::{SourceId, SourceText};

const INVENTORY_SOURCE: &str = include_str!("qualification_validation_tests/inventory.rs");
const STATIC_PRECEDENCE_ORACLE: &str =
    include_str!("qualification_static_semantics_validation_tests.rs");
const ESCAPED_IDENTIFIER_ORACLE: &str =
    include_str!("qualification_validation_tests/escaped_identifier.rs");
const GRAMMAR_EVIDENCE_ORACLE: &str =
    include_str!("qualification_grammar_evidence_validation_tests.rs");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EvidenceRange {
    start: usize,
    end: usize,
}

impl EvidenceRange {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn is_valid_for(self, source: &str) -> bool {
        self.start <= self.end
            && self.end <= source.len()
            && source.is_char_boundary(self.start)
            && source.is_char_boundary(self.end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvidenceFamily {
    Grammar,
    StaticSemantics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EvidencePoint {
    family: EvidenceFamily,
    rule_id: Option<&'static str>,
    subject: EvidenceRange,
}

impl EvidencePoint {
    const fn grammar(subject: EvidenceRange) -> Self {
        Self {
            family: EvidenceFamily::Grammar,
            rule_id: None,
            subject,
        }
    }

    const fn static_rule(rule_id: &'static str, subject: EvidenceRange) -> Self {
        Self {
            family: EvidenceFamily::StaticSemantics,
            rule_id: Some(rule_id),
            subject,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CrossFamilyFixture {
    source: &'static str,
    primary: EvidencePoint,
    supporting: &'static [EvidencePoint],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MultipleGrammarFixture {
    source: &'static str,
    primary: EvidenceRange,
    later: &'static [EvidenceRange],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalDecisionKind {
    StableAtSubjectEnd,
    RequiresLookahead(&'static str),
    RequiresEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerminalDecisionFixture {
    source: &'static str,
    subject: EvidenceRange,
    decision_offset: usize,
    kind: TerminalDecisionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerminalTailFixture {
    source: &'static str,
    subject: EvidenceRange,
    tail: &'static str,
}

const CROSS_FAMILY_FIXTURES: &[CrossFamilyFixture] = &[
    // A formed UES may already establish a tentative EE-01 fact, but a later
    // owned malformed grammar proof means the whole source never reaches the
    // static-semantics verdict boundary.
    CrossFamilyFixture {
        source: r"let \u0030\u{};",
        primary: EvidencePoint::grammar(EvidenceRange::new(10, 14)),
        supporting: &[EvidencePoint::static_rule(
            "EE-01-R01",
            EvidenceRange::new(4, 10),
        )],
    },
    CrossFamilyFixture {
        source: r"let a\u002D\u{};",
        primary: EvidencePoint::grammar(EvidenceRange::new(11, 15)),
        supporting: &[EvidencePoint::static_rule(
            "EE-01-R02",
            EvidenceRange::new(5, 11),
        )],
    },
    CrossFamilyFixture {
        source: r"let \u{}\u0030;",
        primary: EvidencePoint::grammar(EvidenceRange::new(4, 8)),
        supporting: &[],
    },
    // Existing selected static families remain useful supporting facts, but
    // they are not committed whole-source verdicts once owned Grammar failure
    // is independently established later in the source.
    CrossFamilyFixture {
        source: r"let \u0069f; let \u{};",
        primary: EvidencePoint::grammar(EvidenceRange::new(17, 21)),
        supporting: &[EvidencePoint::static_rule(
            "EE-04-R08",
            EvidenceRange::new(4, 11),
        )],
    },
    CrossFamilyFixture {
        source: r"let let; let \u{};",
        primary: EvidencePoint::grammar(EvidenceRange::new(13, 17)),
        supporting: &[EvidencePoint::static_rule(
            "EE-15-R01",
            EvidenceRange::new(4, 7),
        )],
    },
    CrossFamilyFixture {
        source: r"const x; let \u{};",
        primary: EvidencePoint::grammar(EvidenceRange::new(13, 17)),
        supporting: &[EvidencePoint::static_rule(
            "EE-15-R03",
            EvidenceRange::new(6, 7),
        )],
    },
    CrossFamilyFixture {
        source: r"let x, x; let \u{};",
        primary: EvidencePoint::grammar(EvidenceRange::new(14, 18)),
        supporting: &[EvidencePoint::static_rule(
            "EE-15-R02",
            EvidenceRange::new(7, 8),
        )],
    },
    CrossFamilyFixture {
        source: r"let x; let x; let \u{};",
        primary: EvidencePoint::grammar(EvidenceRange::new(18, 22)),
        supporting: &[EvidencePoint::static_rule(
            "EE-36-R01",
            EvidenceRange::new(11, 12),
        )],
    },
];

const MULTIPLE_GRAMMAR_FIXTURES: &[MultipleGrammarFixture] = &[
    MultipleGrammarFixture {
        source: r"let \u{}, a\u{};",
        primary: EvidenceRange::new(4, 8),
        later: &[EvidenceRange::new(11, 15)],
    },
    MultipleGrammarFixture {
        source: r"let a\u{}, b\u{};",
        primary: EvidenceRange::new(5, 9),
        later: &[EvidenceRange::new(12, 16)],
    },
    MultipleGrammarFixture {
        source: r"let \u{}; let a\u{};",
        primary: EvidenceRange::new(4, 8),
        later: &[EvidenceRange::new(15, 19)],
    },
];

const TERMINAL_DECISION_FIXTURES: &[TerminalDecisionFixture] = &[
    TerminalDecisionFixture {
        source: r"let \u{};",
        subject: EvidenceRange::new(4, 8),
        decision_offset: 8,
        kind: TerminalDecisionKind::StableAtSubjectEnd,
    },
    TerminalDecisionFixture {
        source: r"let \u{110000};",
        subject: EvidenceRange::new(4, 14),
        decision_offset: 14,
        kind: TerminalDecisionKind::StableAtSubjectEnd,
    },
    TerminalDecisionFixture {
        source: r"let \u0;",
        subject: EvidenceRange::new(4, 7),
        decision_offset: 7,
        kind: TerminalDecisionKind::RequiresLookahead(";"),
    },
    TerminalDecisionFixture {
        source: r"let \u{61",
        subject: EvidenceRange::new(4, 9),
        decision_offset: 9,
        kind: TerminalDecisionKind::RequiresEof,
    },
];

const TERMINAL_TAIL_FIXTURES: &[TerminalTailFixture] = &[
    TerminalTailFixture {
        source: r"let \u{} = foo;",
        subject: EvidenceRange::new(4, 8),
        tail: " = foo;",
    },
    TerminalTailFixture {
        source: r"let a\u{} = foo;",
        subject: EvidenceRange::new(5, 9),
        tail: " = foo;",
    },
    TerminalTailFixture {
        source: r"let \u{}, x;",
        subject: EvidenceRange::new(4, 8),
        tail: ", x;",
    },
    TerminalTailFixture {
        source: r"let \u{}; var x;",
        subject: EvidenceRange::new(4, 8),
        tail: "; var x;",
    },
];

const DEFERRED_ADJACENT_MALFORMED: &[&str] =
    &[r"\u{G}", r"\u00G0", r"\u{61;", r"\u0x", r"\u", r"\u{"];

fn authored_fragment(source: &str, range: EvidenceRange) -> &str {
    assert!(range.is_valid_for(source));
    source
        .get(range.start..range.end)
        .expect("validated UTF-8 byte range must slice the authoritative source")
}

fn assert_source_anchor(source: &str, range: EvidenceRange) {
    let source_text = SourceText::new(SourceId::new(229), source.to_owned());
    let anchor = source_text
        .anchor(range.start, range.end)
        .expect("#229 expected evidence must reconcile through Core SourceText");
    assert_eq!(anchor.source_id(), SourceId::new(229));
    assert_eq!(anchor.range().start(), range.start);
    assert_eq!(anchor.range().end(), range.end);
    assert_eq!(anchor.fragment(), authored_fragment(source, range));
}

fn active_rule_block(rule_id: &str) -> &'static str {
    let marker = format!("active_rule(\n        \"{rule_id}\",");
    let start = INVENTORY_SOURCE.find(&marker).unwrap_or_else(|| {
        panic!("#229 supporting evidence maps to missing active rule {rule_id}")
    });
    let rest = &INVENTORY_SOURCE[start..];
    let end = rest
        .find("\n    ),")
        .unwrap_or_else(|| panic!("active rule {rule_id} must retain a complete inventory block"))
        + "\n    ),".len();
    &rest[..end]
}

fn assert_grammar_subject_is_227_owned(source: &str, range: EvidenceRange) {
    let fragment = authored_fragment(source, range);
    assert!(
        matches!(fragment, r"\u{}" | r"\u0" | r"\u{61" | r"\u{110000}"),
        "#229 must not widen beyond the malformed/non-CodePoint spellings independently owned by #227: {fragment:?}"
    );
    assert_source_anchor(source, range);
}

#[test]
fn grammar_primary_discards_tentative_selected_static_evidence_across_families() {
    assert_eq!(CROSS_FAMILY_FIXTURES.len(), 8);

    for fixture in CROSS_FAMILY_FIXTURES {
        assert_eq!(fixture.primary.family, EvidenceFamily::Grammar);
        assert_eq!(fixture.primary.rule_id, None);
        assert_grammar_subject_is_227_owned(fixture.source, fixture.primary.subject);

        for supporting in fixture.supporting {
            assert_eq!(supporting.family, EvidenceFamily::StaticSemantics);
            let rule_id = supporting
                .rule_id
                .expect("static supporting evidence must retain its existing rule identity");
            let _ = active_rule_block(rule_id);
            assert_source_anchor(fixture.source, supporting.subject);
            assert_ne!(supporting.subject, fixture.primary.subject);
        }
    }

    let ee01_start = &CROSS_FAMILY_FIXTURES[0];
    assert_eq!(ee01_start.source, r"let \u0030\u{};");
    assert_eq!(ee01_start.primary.subject, EvidenceRange::new(10, 14));
    assert_eq!(
        ee01_start.supporting,
        &[EvidencePoint::static_rule(
            "EE-01-R01",
            EvidenceRange::new(4, 10)
        )]
    );

    let ee01_part = &CROSS_FAMILY_FIXTURES[1];
    assert_eq!(ee01_part.source, r"let a\u002D\u{};");
    assert_eq!(ee01_part.primary.subject, EvidenceRange::new(11, 15));
    assert_eq!(
        ee01_part.supporting,
        &[EvidencePoint::static_rule(
            "EE-01-R02",
            EvidenceRange::new(5, 11)
        )]
    );

    let grammar_first = &CROSS_FAMILY_FIXTURES[2];
    assert_eq!(grammar_first.source, r"let \u{}\u0030;");
    assert!(grammar_first.supporting.is_empty());
}

#[test]
fn cross_family_matrix_covers_existing_selected_static_families_without_reordering_them() {
    let expected = [
        (3, "EE-04-R08", EvidenceRange::new(4, 11)),
        (4, "EE-15-R01", EvidenceRange::new(4, 7)),
        (5, "EE-15-R03", EvidenceRange::new(6, 7)),
        (6, "EE-15-R02", EvidenceRange::new(7, 8)),
        (7, "EE-36-R01", EvidenceRange::new(11, 12)),
    ];

    for (index, rule_id, subject) in expected {
        let fixture = &CROSS_FAMILY_FIXTURES[index];
        assert_eq!(fixture.primary.family, EvidenceFamily::Grammar);
        assert_eq!(fixture.supporting.len(), 1);
        assert_eq!(
            fixture.supporting[0],
            EvidencePoint::static_rule(rule_id, subject)
        );
        let _ = active_rule_block(rule_id);
    }

    assert!(STATIC_PRECEDENCE_ORACLE.contains("EE-15-R01"));
    assert!(STATIC_PRECEDENCE_ORACLE.contains("EE-15-R02"));
    assert!(STATIC_PRECEDENCE_ORACLE.contains("EE-15-R03"));
    assert!(STATIC_PRECEDENCE_ORACLE.contains("EE-36-R01"));
    assert!(ESCAPED_IDENTIFIER_ORACLE.contains("EE-04-R08"));
    assert!(ESCAPED_IDENTIFIER_ORACLE.contains("EE-01-R01"));
    assert!(ESCAPED_IDENTIFIER_ORACLE.contains("EE-01-R02"));
}

#[test]
fn multiple_owned_grammar_proofs_select_first_stable_source_order_subject() {
    assert_eq!(MULTIPLE_GRAMMAR_FIXTURES.len(), 3);

    for fixture in MULTIPLE_GRAMMAR_FIXTURES {
        assert_grammar_subject_is_227_owned(fixture.source, fixture.primary);
        assert!(!fixture.later.is_empty());

        let mut previous_end = fixture.primary.end;
        for later in fixture.later {
            assert_grammar_subject_is_227_owned(fixture.source, *later);
            assert!(
                later.start >= previous_end,
                "later grammar proof must remain after the already-selected primary proof"
            );
            previous_end = later.end;
        }
    }
}

#[test]
fn authored_subject_and_terminal_decision_point_remain_distinct() {
    assert_eq!(TERMINAL_DECISION_FIXTURES.len(), 4);

    for fixture in TERMINAL_DECISION_FIXTURES {
        assert_grammar_subject_is_227_owned(fixture.source, fixture.subject);
        assert!(fixture.decision_offset >= fixture.subject.end);
        assert!(fixture.decision_offset <= fixture.source.len());
        assert!(fixture.source.is_char_boundary(fixture.decision_offset));

        match fixture.kind {
            TerminalDecisionKind::StableAtSubjectEnd => {
                assert_eq!(fixture.decision_offset, fixture.subject.end);
            }
            TerminalDecisionKind::RequiresLookahead(expected) => {
                assert_eq!(fixture.decision_offset, fixture.subject.end);
                assert_eq!(
                    fixture.source.get(fixture.decision_offset..),
                    Some(expected)
                );
                assert_eq!(
                    authored_fragment(fixture.source, fixture.subject),
                    r"\u0",
                    "the delimiter proves incompleteness but is not part of the authored grammar subject"
                );
            }
            TerminalDecisionKind::RequiresEof => {
                assert_eq!(fixture.decision_offset, fixture.source.len());
                assert_eq!(fixture.subject.end, fixture.source.len());
            }
        }
    }
}

#[test]
fn stable_owned_grammar_proof_is_terminal_across_bounded_unowned_tails() {
    assert_eq!(TERMINAL_TAIL_FIXTURES.len(), 4);

    for fixture in TERMINAL_TAIL_FIXTURES {
        assert_grammar_subject_is_227_owned(fixture.source, fixture.subject);
        assert_eq!(
            fixture.source.get(fixture.subject.end..),
            Some(fixture.tail)
        );
        assert!(!fixture.tail.is_empty());
    }

    // These tails are evidence that later unsupported source cannot repair an
    // already-proven malformed selected BindingIdentifier. They are not new
    // positive grammar coverage for expressions, BindingLists, or `var`.
    assert_eq!(TERMINAL_TAIL_FIXTURES[0].tail, " = foo;");
    assert_eq!(TERMINAL_TAIL_FIXTURES[2].tail, ", x;");
    assert_eq!(TERMINAL_TAIL_FIXTURES[3].tail, "; var x;");
}

#[test]
fn prior_candidate_independent_oracles_remain_the_authority_for_bounded_subjects() {
    for expected in [
        r#"source: r"let \u{};","#,
        r#"source: r"let \u0;","#,
        r#"source: r"let \u{61","#,
        r#"source: r"let \u{110000};","#,
        r#"source: r"let a\u{};","#,
    ] {
        assert!(
            GRAMMAR_EVIDENCE_ORACLE.contains(expected),
            "#227 authority must retain {expected}"
        );
    }

    for expected_range in [
        "EvidenceRange::new(4, 8)",
        "EvidenceRange::new(4, 7)",
        "EvidenceRange::new(4, 9)",
        "EvidenceRange::new(4, 14)",
        "EvidenceRange::new(5, 9)",
    ] {
        assert!(GRAMMAR_EVIDENCE_ORACLE.contains(expected_range));
    }
}

#[test]
fn adjacent_malformed_classes_remain_explicitly_deferred() {
    assert_eq!(DEFERRED_ADJACENT_MALFORMED.len(), 6);

    for deferred in DEFERRED_ADJACENT_MALFORMED {
        assert!(!deferred.is_empty());
        for fixture in CROSS_FAMILY_FIXTURES {
            assert_ne!(
                authored_fragment(fixture.source, fixture.primary.subject),
                *deferred
            );
        }
        for fixture in MULTIPLE_GRAMMAR_FIXTURES {
            assert_ne!(
                authored_fragment(fixture.source, fixture.primary),
                *deferred
            );
            for later in fixture.later {
                assert_ne!(authored_fragment(fixture.source, *later), *deferred);
            }
        }
    }
}

#[test]
fn cross_family_validation_does_not_import_production_candidate_behavior() {
    let source = include_str!("qualification_grammar_rejection_policy_validation_tests.rs");
    for forbidden in [
        concat!("selected_binding", "_identifier"),
        concat!("selected_lexical", "_slice"),
        concat!("selected_static", "_semantics"),
        concat!("selected_qualification", "_integration"),
        concat!("QualificationOutcome", "::syntax_rejected"),
        concat!("DefinitiveGrammar", "RejectionEvidence {"),
    ] {
        assert!(
            !source.contains(forbidden),
            "#229 expected policy must remain candidate-independent: found {forbidden}"
        );
    }
}
