//! Candidate-independent one-level selected Block composition validation for Issue #295.
//!
//! This module materializes only relationships between already-accepted validation
//! authorities. It deliberately does not call production lexical, static-semantics,
//! aggregate qualification, Binding / Scope, or runtime code to derive expectations.

use crate::{SourceId, SourceText};

const ISSUE_ID: u64 = 295;
const STATIC_PRECEDENCE_ORACLE: &str =
    include_str!("qualification_static_semantics_validation_tests.rs");
const GRAMMAR_EVIDENCE_ORACLE: &str =
    include_str!("qualification_grammar_evidence_validation_tests.rs");
const GRAMMAR_POLICY_ORACLE: &str =
    include_str!("qualification_grammar_rejection_policy_validation_tests.rs");
const RHS_ORACLE: &str =
    include_str!("qualification_selected_escaped_identifier_reference_initializer_validation_tests.rs");
const BLOCK_ORACLE: &str =
    include_str!("qualification_selected_one_level_block_validation_tests.rs");
const HISTORICAL_COMPLETION_ORACLE: &str =
    include_str!("qualification_validation_tests/selected_slice_completion.rs");
const THIS_SOURCE: &str =
    include_str!("qualification_selected_one_level_block_composition_validation_tests.rs");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ByteRange {
    start: usize,
    end: usize,
}

impl ByteRange {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn fragment<'source>(self, source: &'source str) -> &'source str {
        assert!(self.start <= self.end);
        assert!(self.end <= source.len());
        assert!(source.is_char_boundary(self.start));
        assert!(source.is_char_boundary(self.end));
        source
            .get(self.start..self.end)
            .expect("validated UTF-8 range must slice authoritative source")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BindingPosition {
    Start,
    Part,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompositionOutcome {
    Grammar,
    StaticSemanticsRejected,
    UnsupportedCoverage,
    SelectedCandidateAccepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EmbeddedGrammarFixture {
    source: &'static str,
    subject: ByteRange,
    subject_fragment: &'static str,
    position: BindingPosition,
}

const EMBEDDED_GRAMMAR_FIXTURES: &[EmbeddedGrammarFixture] = &[
    EmbeddedGrammarFixture {
        source: r"{ let \u{}; }",
        subject: ByteRange::new(6, 10),
        subject_fragment: r"\u{}",
        position: BindingPosition::Start,
    },
    EmbeddedGrammarFixture {
        source: r"{ let a\u{}; }",
        subject: ByteRange::new(7, 11),
        subject_fragment: r"\u{}",
        position: BindingPosition::Part,
    },
    EmbeddedGrammarFixture {
        source: r"{ let \u{110000}; }",
        subject: ByteRange::new(6, 16),
        subject_fragment: r"\u{110000}",
        position: BindingPosition::Start,
    },
    EmbeddedGrammarFixture {
        source: r"{ let a\u{110000}; }",
        subject: ByteRange::new(7, 17),
        subject_fragment: r"\u{110000}",
        position: BindingPosition::Part,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContextContrastFixture {
    source: &'static str,
    subject: ByteRange,
    subject_fragment: &'static str,
    outcome: CompositionOutcome,
}

const RHS_CONTEXT_CONTRASTS: &[ContextContrastFixture] = &[
    ContextContrastFixture {
        source: r"{ const x = \u{}; }",
        subject: ByteRange::new(12, 16),
        subject_fragment: r"\u{}",
        outcome: CompositionOutcome::UnsupportedCoverage,
    },
    ContextContrastFixture {
        source: r"{ const x = \u{110000}; }",
        subject: ByteRange::new(12, 22),
        subject_fragment: r"\u{110000}",
        outcome: CompositionOutcome::UnsupportedCoverage,
    },
    ContextContrastFixture {
        source: r"{ const x = \u{61}; }",
        subject: ByteRange::new(12, 18),
        subject_fragment: r"\u{61}",
        outcome: CompositionOutcome::SelectedCandidateAccepted,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EvidencePoint {
    rule_id: &'static str,
    subject: ByteRange,
    fragment: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GrammarVsBlockFixture {
    source: &'static str,
    grammar: ByteRange,
    block_first: EvidencePoint,
    block_duplicate: EvidencePoint,
}

const GRAMMAR_VS_BLOCK: GrammarVsBlockFixture = GrammarVsBlockFixture {
    source: r"{ let a=1; let a=2; let \u{}; }",
    grammar: ByteRange::new(24, 28),
    block_first: EvidencePoint {
        rule_id: "EE-14-R01",
        subject: ByteRange::new(6, 7),
        fragment: "a",
    },
    block_duplicate: EvidencePoint {
        rule_id: "EE-14-R01",
        subject: ByteRange::new(15, 16),
        fragment: "a",
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticTier {
    DeclarationLocal,
    BlockRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StaticTierFixture {
    source: &'static str,
    primary_tier: StaticTier,
    primary: EvidencePoint,
    supporting: &'static [EvidencePoint],
}

const TIER_1_SUPPORTING: &[EvidencePoint] = &[
    EvidencePoint {
        rule_id: "EE-14-R01",
        subject: ByteRange::new(6, 7),
        fragment: "a",
    },
    EvidencePoint {
        rule_id: "EE-14-R01",
        subject: ByteRange::new(15, 16),
        fragment: "a",
    },
];

const TIER_2_SUPPORTING: &[EvidencePoint] = &[
    EvidencePoint {
        rule_id: "EE-36-R01",
        subject: ByteRange::new(26, 27),
        fragment: "z",
    },
    EvidencePoint {
        rule_id: "EE-36-R01",
        subject: ByteRange::new(35, 36),
        fragment: "z",
    },
];

const SIBLING_SUPPORTING: &[EvidencePoint] = &[
    EvidencePoint {
        rule_id: "EE-14-R01",
        subject: ByteRange::new(28, 29),
        fragment: "b",
    },
    EvidencePoint {
        rule_id: "EE-14-R01",
        subject: ByteRange::new(37, 38),
        fragment: "b",
    },
];

const STATIC_TIER_FIXTURES: &[StaticTierFixture] = &[
    StaticTierFixture {
        source: "{ let a=1; let a=2; } const x;",
        primary_tier: StaticTier::DeclarationLocal,
        primary: EvidencePoint {
            rule_id: "EE-15-R03",
            subject: ByteRange::new(28, 29),
            fragment: "x",
        },
        supporting: TIER_1_SUPPORTING,
    },
    StaticTierFixture {
        source: "{ let a=1; let a=2; } let z=1; let z=2;",
        primary_tier: StaticTier::BlockRegion,
        primary: EvidencePoint {
            rule_id: "EE-14-R01",
            subject: ByteRange::new(15, 16),
            fragment: "a",
        },
        supporting: TIER_2_SUPPORTING,
    },
    StaticTierFixture {
        source: "{ let a=1; let a=2; } { let b=1; let b=2; }",
        primary_tier: StaticTier::BlockRegion,
        primary: EvidencePoint {
            rule_id: "EE-14-R01",
            subject: ByteRange::new(15, 16),
            fragment: "a",
        },
        supporting: SIBLING_SUPPORTING,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RegionDomainFixture {
    source: &'static str,
    expected: CompositionOutcome,
    script_duplicate: Option<ByteRange>,
}

const REGION_DOMAIN_FIXTURES: &[RegionDomainFixture] = &[
    RegionDomainFixture {
        source: "{ let a=1; } { let a=2; }",
        expected: CompositionOutcome::SelectedCandidateAccepted,
        script_duplicate: None,
    },
    RegionDomainFixture {
        source: "let a=1; { let a=2; } let a=3;",
        expected: CompositionOutcome::StaticSemanticsRejected,
        script_duplicate: Some(ByteRange::new(26, 27)),
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DelimiterDisposition {
    UnclosedBracedGrammar { subject: ByteRange },
    FormedEscapeButUnclosedBlock,
    FormedEscapeAndBlockCloseButMissingSemicolon,
    SelectedCandidateAccepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DelimiterFixture {
    source: &'static str,
    escape: ByteRange,
    semicolon: Option<ByteRange>,
    block_close: Option<ByteRange>,
    disposition: DelimiterDisposition,
}

const DELIMITER_FIXTURES: &[DelimiterFixture] = &[
    DelimiterFixture {
        source: r"{ let \u{61",
        escape: ByteRange::new(6, 11),
        semicolon: None,
        block_close: None,
        disposition: DelimiterDisposition::UnclosedBracedGrammar {
            subject: ByteRange::new(6, 11),
        },
    },
    DelimiterFixture {
        source: r"{ let \u{61}",
        escape: ByteRange::new(6, 12),
        semicolon: None,
        block_close: None,
        disposition: DelimiterDisposition::FormedEscapeButUnclosedBlock,
    },
    DelimiterFixture {
        source: r"{ let \u{61}}",
        escape: ByteRange::new(6, 12),
        semicolon: None,
        block_close: Some(ByteRange::new(12, 13)),
        disposition: DelimiterDisposition::FormedEscapeAndBlockCloseButMissingSemicolon,
    },
    DelimiterFixture {
        source: r"{ let \u{61}; }",
        escape: ByteRange::new(6, 12),
        semicolon: Some(ByteRange::new(12, 13)),
        block_close: Some(ByteRange::new(14, 15)),
        disposition: DelimiterDisposition::SelectedCandidateAccepted,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShortFixedDelimiterFixture {
    source: &'static str,
    subject: ByteRange,
    next: ByteRange,
    expected: CompositionOutcome,
}

const SHORT_FIXED_DELIMITER_FIXTURES: &[ShortFixedDelimiterFixture] = &[
    ShortFixedDelimiterFixture {
        source: r"{ let \u0; }",
        subject: ByteRange::new(6, 9),
        next: ByteRange::new(9, 10),
        expected: CompositionOutcome::Grammar,
    },
    ShortFixedDelimiterFixture {
        source: r"{ let \u0}",
        subject: ByteRange::new(6, 9),
        next: ByteRange::new(9, 10),
        expected: CompositionOutcome::UnsupportedCoverage,
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

fn assert_point(source: &str, point: EvidencePoint) {
    assert_anchor(source, point.subject, point.fragment);
    assert!(!point.rule_id.is_empty());
}

#[test]
fn authority_chain_is_present_and_candidate_independent() {
    assert!(STATIC_PRECEDENCE_ORACLE.contains("RULE_PRECEDENCE_FIXTURES"));
    assert!(STATIC_PRECEDENCE_ORACLE.contains("EE-15-R03"));
    assert!(STATIC_PRECEDENCE_ORACLE.contains("EE-36-R01"));

    assert!(GRAMMAR_EVIDENCE_ORACLE.contains("BASE_GRAMMAR_EVIDENCE"));
    assert!(GRAMMAR_EVIDENCE_ORACLE.contains("CandidateDefinitiveGrammarEvidence"));

    assert!(
        GRAMMAR_POLICY_ORACLE
            .contains("grammar_primary_discards_tentative_selected_static_evidence_across_families")
    );
    assert!(GRAMMAR_POLICY_ORACLE.contains(r#"RequiresLookahead(";")"#));
    assert!(GRAMMAR_POLICY_ORACLE.contains("RequiresEof"));

    assert!(RHS_ORACLE.contains("NEGATIVE_UNSUPPORTED_EXPECTATION"));
    assert!(RHS_ORACLE.contains("EscapedIdentifierReferenceFrontierOutcome::UnsupportedCoverage"));

    assert!(BLOCK_ORACLE.contains("EE-14-R01"));
    assert!(BLOCK_ORACLE.contains("EE-14-R02"));
    assert!(BLOCK_ORACLE.contains("SIBLING_BLOCKS"));
    assert!(BLOCK_ORACLE.contains("LEGAL_SHADOW"));

    assert!(HISTORICAL_COMPLETION_ORACLE.contains("LexicalDeclaration+"));

    for forbidden in [
        concat!("recognize_selected_", "lexical_slice("),
        concat!("evaluate_selected_", "static_semantics("),
        concat!(
            "evaluate_selected_one_level_block_",
            "static_semantics("
        ),
        concat!("attempt_selected_", "qualification("),
        concat!("analyze_selected_", "binding_scope("),
        concat!("QualificationOutcome::", "qualified"),
        concat!("CompleteQualification", "Witness"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "#295 oracle must not derive expected meaning from production: {forbidden}"
        );
    }
}

#[test]
fn bounded_binding_grammar_subjects_relocate_truthfully_inside_block_context() {
    assert_eq!(EMBEDDED_GRAMMAR_FIXTURES.len(), 4);

    for fixture in EMBEDDED_GRAMMAR_FIXTURES {
        assert_anchor(fixture.source, fixture.subject, fixture.subject_fragment);
        assert!(fixture.source.starts_with("{ let "));
        match fixture.position {
            BindingPosition::Start => {
                assert_eq!(fixture.source.get(2..6), Some("let "));
                assert_eq!(fixture.subject.start, 6);
            }
            BindingPosition::Part => {
                assert_eq!(fixture.source.get(2..7), Some("let a"));
                assert_eq!(fixture.subject.start, 7);
            }
        }
        assert!(matches!(
            fixture.subject_fragment,
            r"\u{}" | r"\u{110000}"
        ));
    }
}

#[test]
fn rhs_context_does_not_inherit_binding_identifier_grammar_ownership() {
    assert_eq!(RHS_CONTEXT_CONTRASTS.len(), 3);

    for fixture in RHS_CONTEXT_CONTRASTS {
        assert_anchor(fixture.source, fixture.subject, fixture.subject_fragment);
        assert!(fixture.source.starts_with("{ const x = "));
    }

    assert_eq!(
        RHS_CONTEXT_CONTRASTS[0].outcome,
        CompositionOutcome::UnsupportedCoverage
    );
    assert_eq!(
        RHS_CONTEXT_CONTRASTS[1].outcome,
        CompositionOutcome::UnsupportedCoverage
    );
    assert_eq!(
        RHS_CONTEXT_CONTRASTS[2].outcome,
        CompositionOutcome::SelectedCandidateAccepted
    );

    assert_eq!(EMBEDDED_GRAMMAR_FIXTURES[0].subject_fragment, r"\u{}");
    assert_eq!(RHS_CONTEXT_CONTRASTS[0].subject_fragment, r"\u{}");
    assert_ne!(
        CompositionOutcome::Grammar,
        RHS_CONTEXT_CONTRASTS[0].outcome,
        "same authored bytes in IdentifierReference position must not inherit BindingIdentifier Grammar ownership"
    );
}

#[test]
fn grammar_primary_policy_composes_with_new_block_duplicate_evidence() {
    let fixture = GRAMMAR_VS_BLOCK;

    assert_anchor(fixture.source, fixture.grammar, r"\u{}");
    assert_point(fixture.source, fixture.block_first);
    assert_point(fixture.source, fixture.block_duplicate);
    assert_eq!(fixture.block_first.rule_id, "EE-14-R01");
    assert_eq!(fixture.block_duplicate.rule_id, "EE-14-R01");
    assert!(fixture.block_duplicate.subject.end <= fixture.grammar.start);

    assert!(
        GRAMMAR_POLICY_ORACLE
            .contains("grammar_primary_discards_tentative_selected_static_evidence_across_families"),
        "#229 must remain the authority for Grammar over tentative static evidence"
    );
    assert!(
        BLOCK_ORACLE.contains("ReachableRejecting"),
        "#285 must remain the authority making EE-14-R01 reachable in the selected Block frontier"
    );
}

#[test]
fn static_primary_evidence_tiers_are_explicit_across_regions() {
    assert_eq!(STATIC_TIER_FIXTURES.len(), 3);

    let tier_1 = STATIC_TIER_FIXTURES[0];
    assert_eq!(tier_1.primary_tier, StaticTier::DeclarationLocal);
    assert_eq!(tier_1.primary.rule_id, "EE-15-R03");
    assert_point(tier_1.source, tier_1.primary);
    for point in tier_1.supporting {
        assert_eq!(point.rule_id, "EE-14-R01");
        assert_point(tier_1.source, *point);
    }
    assert!(
        tier_1.supporting[1].subject.end < tier_1.primary.subject.start,
        "later declaration-local evidence intentionally outranks earlier Block duplicate evidence"
    );

    let tier_2 = STATIC_TIER_FIXTURES[1];
    assert_eq!(tier_2.primary_tier, StaticTier::BlockRegion);
    assert_eq!(tier_2.primary.rule_id, "EE-14-R01");
    assert_point(tier_2.source, tier_2.primary);
    for point in tier_2.supporting {
        assert_eq!(point.rule_id, "EE-36-R01");
        assert_point(tier_2.source, *point);
    }
    assert!(
        tier_2.primary.subject.start < tier_2.supporting[1].subject.start,
        "Block duplicate evidence intentionally outranks Script duplicate evidence"
    );

    let sibling = STATIC_TIER_FIXTURES[2];
    assert_eq!(sibling.primary_tier, StaticTier::BlockRegion);
    assert_eq!(sibling.primary.rule_id, "EE-14-R01");
    assert_point(sibling.source, sibling.primary);
    for point in sibling.supporting {
        assert_eq!(point.rule_id, "EE-14-R01");
        assert_point(sibling.source, *point);
    }
    assert!(
        sibling.primary.subject.start < sibling.supporting[0].subject.start,
        "first Block in source order owns the primary Tier-2 duplicate evidence"
    );
}

#[test]
fn region_duplicate_domains_remain_separate_after_block_embedding() {
    assert_eq!(REGION_DOMAIN_FIXTURES.len(), 2);

    let siblings = REGION_DOMAIN_FIXTURES[0];
    assert_eq!(
        siblings.expected,
        CompositionOutcome::SelectedCandidateAccepted
    );
    assert_eq!(siblings.script_duplicate, None);
    assert_eq!(siblings.source, "{ let a=1; } { let a=2; }");

    let script_duplicate = REGION_DOMAIN_FIXTURES[1];
    assert_eq!(
        script_duplicate.expected,
        CompositionOutcome::StaticSemanticsRejected
    );
    let subject = script_duplicate
        .script_duplicate
        .expect("Script duplicate control must retain the top-level duplicate subject");
    assert_anchor(script_duplicate.source, subject, "a");
    assert_eq!(subject, ByteRange::new(26, 27));
    assert_eq!(script_duplicate.source.get(15..16), Some("a"));
    assert_ne!(
        subject,
        ByteRange::new(15, 16),
        "inner Block binding must not become the Script duplicate subject"
    );
}

#[test]
fn unicode_escape_close_and_block_close_are_distinct_authored_delimiters() {
    assert_eq!(DELIMITER_FIXTURES.len(), 4);

    let unclosed = DELIMITER_FIXTURES[0];
    assert_anchor(unclosed.source, unclosed.escape, r"\u{61");
    assert_eq!(unclosed.semicolon, None);
    assert_eq!(unclosed.block_close, None);
    assert_eq!(
        unclosed.disposition,
        DelimiterDisposition::UnclosedBracedGrammar {
            subject: ByteRange::new(6, 11)
        }
    );
    assert_eq!(unclosed.escape.end, unclosed.source.len());

    let formed_unclosed_block = DELIMITER_FIXTURES[1];
    assert_anchor(
        formed_unclosed_block.source,
        formed_unclosed_block.escape,
        r"\u{61}",
    );
    assert_eq!(formed_unclosed_block.block_close, None);
    assert_eq!(
        formed_unclosed_block.disposition,
        DelimiterDisposition::FormedEscapeButUnclosedBlock
    );
    assert_eq!(
        formed_unclosed_block.escape.end,
        formed_unclosed_block.source.len(),
        "the only close brace belongs to the UnicodeEscapeSequence"
    );

    let formed_with_block_close = DELIMITER_FIXTURES[2];
    assert_anchor(
        formed_with_block_close.source,
        formed_with_block_close.escape,
        r"\u{61}",
    );
    let close = formed_with_block_close
        .block_close
        .expect("second close brace is the authored Block close");
    assert_anchor(formed_with_block_close.source, close, "}");
    assert_eq!(formed_with_block_close.escape.end, close.start);
    assert_eq!(
        formed_with_block_close.disposition,
        DelimiterDisposition::FormedEscapeAndBlockCloseButMissingSemicolon
    );

    let valid = DELIMITER_FIXTURES[3];
    assert_anchor(valid.source, valid.escape, r"\u{61}");
    let semicolon = valid
        .semicolon
        .expect("selected Block declaration requires authored semicolon");
    let block_close = valid
        .block_close
        .expect("selected Block control requires authored close brace");
    assert_anchor(valid.source, semicolon, ";");
    assert_anchor(valid.source, block_close, "}");
    assert_eq!(valid.escape.end, semicolon.start);
    assert!(semicolon.end < block_close.start);
    assert_eq!(
        valid.disposition,
        DelimiterDisposition::SelectedCandidateAccepted
    );
}

#[test]
fn short_fixed_grammar_evidence_does_not_silently_acquire_block_close_lookahead() {
    assert_eq!(SHORT_FIXED_DELIMITER_FIXTURES.len(), 2);

    let semicolon = SHORT_FIXED_DELIMITER_FIXTURES[0];
    assert_anchor(semicolon.source, semicolon.subject, r"\u0");
    assert_anchor(semicolon.source, semicolon.next, ";");
    assert_eq!(semicolon.expected, CompositionOutcome::Grammar);

    let block_close = SHORT_FIXED_DELIMITER_FIXTURES[1];
    assert_anchor(block_close.source, block_close.subject, r"\u0");
    assert_anchor(block_close.source, block_close.next, "}");
    assert_eq!(
        block_close.expected,
        CompositionOutcome::UnsupportedCoverage
    );

    assert_eq!(semicolon.subject.fragment(semicolon.source), r"\u0");
    assert_eq!(block_close.subject.fragment(block_close.source), r"\u0");
    assert_ne!(
        semicolon.expected, block_close.expected,
        "composition must preserve the #227/#229 semicolon-specific short-fixed decision boundary"
    );
    assert!(GRAMMAR_POLICY_ORACLE.contains(r#"RequiresLookahead(";")"#));
    assert!(
        !GRAMMAR_POLICY_ORACLE.contains(r#"RequiresLookahead("}")"#),
        "this validation must not fabricate new `}`-delimited short-fixed Grammar authority"
    );
}

#[test]
fn historical_completion_and_production_qualification_remain_outside_this_oracle() {
    assert!(HISTORICAL_COMPLETION_ORACLE.contains("LexicalDeclaration+"));
    assert!(
        !HISTORICAL_COMPLETION_ORACLE.contains("Block-enabled selected-slice completion"),
        "historical #268 must remain flat-only"
    );

    assert!(!THIS_SOURCE.contains(concat!("SelectedAccepted", "Incomplete")));
    assert!(!THIS_SOURCE.contains(concat!("QualificationVerdict", "Kind")));
    assert!(!THIS_SOURCE.contains(concat!("Rejection", "Family")));
}
