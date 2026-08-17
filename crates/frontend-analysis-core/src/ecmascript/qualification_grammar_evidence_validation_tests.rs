//! Candidate-independent source-backed grammar evidence for Issue #227.
//!
//! #222 already owns whole-standard malformed/non-CodePoint escape meaning.
//! This module adds only the bounded authored evidence and selected-context
//! ownership needed before a future production grammar-rejection architecture
//! may be considered.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::{gold_source, gold_subject_range};

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
enum EscapeGrammarCategory {
    EmptyBraced,
    ShortFixed,
    UnclosedBraced,
    NonCodePointBraced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedGrammarEvidenceExpectation {
    CandidateDefinitiveGrammarEvidence,
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GrammarEvidenceFixture {
    gold_id: Option<&'static str>,
    source: &'static str,
    context_prefix: &'static str,
    subject: EvidenceRange,
    category: EscapeGrammarCategory,
    expectation: SelectedGrammarEvidenceExpectation,
}

const BASE_GRAMMAR_EVIDENCE: &[GrammarEvidenceFixture] = &[
    GrammarEvidenceFixture {
        gold_id: Some("JS-GOLD-IDENTIFIER-ESCAPE-MALFORMED-EMPTY-BRACED-001"),
        source: r"let \u{};",
        context_prefix: "let ",
        subject: EvidenceRange::new(4, 8),
        category: EscapeGrammarCategory::EmptyBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
    GrammarEvidenceFixture {
        gold_id: Some("JS-GOLD-IDENTIFIER-ESCAPE-MALFORMED-SHORT-FIXED-001"),
        source: r"let \u0;",
        context_prefix: "let ",
        subject: EvidenceRange::new(4, 7),
        category: EscapeGrammarCategory::ShortFixed,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
    GrammarEvidenceFixture {
        gold_id: Some("JS-GOLD-IDENTIFIER-ESCAPE-MALFORMED-UNCLOSED-BRACED-001"),
        source: r"let \u{61",
        context_prefix: "let ",
        subject: EvidenceRange::new(4, 9),
        category: EscapeGrammarCategory::UnclosedBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
    GrammarEvidenceFixture {
        gold_id: Some("JS-GOLD-IDENTIFIER-ESCAPE-NONCODEPOINT-001"),
        source: r"let \u{110000};",
        context_prefix: "let ",
        subject: EvidenceRange::new(4, 14),
        category: EscapeGrammarCategory::NonCodePointBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
];

const PART_POSITION_EVIDENCE: &[GrammarEvidenceFixture] = &[
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"let a\u{};",
        context_prefix: "let a",
        subject: EvidenceRange::new(5, 9),
        category: EscapeGrammarCategory::EmptyBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"let a\u0;",
        context_prefix: "let a",
        subject: EvidenceRange::new(5, 8),
        category: EscapeGrammarCategory::ShortFixed,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"let a\u{61",
        context_prefix: "let a",
        subject: EvidenceRange::new(5, 10),
        category: EscapeGrammarCategory::UnclosedBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"let a\u{110000};",
        context_prefix: "let a",
        subject: EvidenceRange::new(5, 15),
        category: EscapeGrammarCategory::NonCodePointBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
];

const KEYWORD_BOUNDARY_EVIDENCE: &[GrammarEvidenceFixture] = &[
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"let\u{};",
        context_prefix: "let",
        subject: EvidenceRange::new(3, 7),
        category: EscapeGrammarCategory::EmptyBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"let\u{110000};",
        context_prefix: "let",
        subject: EvidenceRange::new(3, 13),
        category: EscapeGrammarCategory::NonCodePointBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
];

const EARLY_DEFINITIVE_EVIDENCE: &[GrammarEvidenceFixture] = &[
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"let \u{}; foo();",
        context_prefix: "let ",
        subject: EvidenceRange::new(4, 8),
        category: EscapeGrammarCategory::EmptyBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"let a\u{}; foo();",
        context_prefix: "let a",
        subject: EvidenceRange::new(5, 9),
        category: EscapeGrammarCategory::EmptyBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"let\u{}; foo();",
        context_prefix: "let",
        subject: EvidenceRange::new(3, 7),
        category: EscapeGrammarCategory::EmptyBraced,
        expectation: SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
    },
];

const UNOWNED_CONTEXT_EVIDENCE: &[GrammarEvidenceFixture] = &[
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"foo \u{};",
        context_prefix: "foo ",
        subject: EvidenceRange::new(4, 8),
        category: EscapeGrammarCategory::EmptyBraced,
        expectation: SelectedGrammarEvidenceExpectation::UnsupportedCoverage,
    },
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"x = \u{};",
        context_prefix: "x = ",
        subject: EvidenceRange::new(4, 8),
        category: EscapeGrammarCategory::EmptyBraced,
        expectation: SelectedGrammarEvidenceExpectation::UnsupportedCoverage,
    },
    GrammarEvidenceFixture {
        gold_id: None,
        source: r"let\u002D\u{};",
        context_prefix: r"let\u002D",
        subject: EvidenceRange::new(9, 13),
        category: EscapeGrammarCategory::EmptyBraced,
        expectation: SelectedGrammarEvidenceExpectation::UnsupportedCoverage,
    },
];

fn ascii_hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

fn braced_code_point(digits: &[u8]) -> Option<u32> {
    if digits.is_empty() || digits.iter().any(|byte| ascii_hex_value(*byte).is_none()) {
        return None;
    }

    let significant = match digits.iter().position(|byte| *byte != b'0') {
        Some(index) => &digits[index..],
        None => return Some(0),
    };

    // The length check follows from the mathematical Code Point maximum after
    // leading zeroes are removed; it is not an authored spelling limit.
    if significant.len() > 6 {
        return None;
    }

    let mut value = 0_u32;
    for byte in significant {
        value = value * 16 + ascii_hex_value(*byte)?;
    }
    (value <= 0x10_FFFF).then_some(value)
}

fn formed_unicode_escape_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return None;
    }

    let payload = start + 2;
    if bytes.get(payload) == Some(&b'{') {
        let digits_start = payload + 1;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(|byte| byte.is_ascii_hexdigit()) {
            end += 1;
        }
        if bytes.get(end) != Some(&b'}') {
            return None;
        }
        let _ = braced_code_point(&bytes[digits_start..end])?;
        return Some(end + 1);
    }

    let end = payload.checked_add(4)?;
    let digits = bytes.get(payload..end)?;
    digits
        .iter()
        .all(|byte| byte.is_ascii_hexdigit())
        .then_some(end)
}

fn classify_grammar_subject(spelling: &str) -> Option<EscapeGrammarCategory> {
    let payload = spelling.strip_prefix(r"\u")?;

    if let Some(braced) = payload.strip_prefix('{') {
        let Some(digits) = braced.strip_suffix('}') else {
            return braced
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
                .then_some(EscapeGrammarCategory::UnclosedBraced);
        };

        if digits.is_empty() {
            return Some(EscapeGrammarCategory::EmptyBraced);
        }
        if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        if braced_code_point(digits.as_bytes()).is_none() {
            return Some(EscapeGrammarCategory::NonCodePointBraced);
        }
        return None;
    }

    (payload.len() < 4 && payload.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then_some(EscapeGrammarCategory::ShortFixed)
}

fn authored_fragment(source: &str, range: EvidenceRange) -> &str {
    assert!(range.is_valid_for(source));
    source
        .get(range.start..range.end)
        .expect("validated UTF-8 byte range must slice the authoritative source")
}

fn assert_core_source_anchor(source: &str, range: EvidenceRange) {
    let source_text = SourceText::new(SourceId::new(227), source.to_owned());
    let anchor = source_text
        .anchor(range.start, range.end)
        .expect("grammar evidence must reconcile through Core SourceText");
    assert_eq!(anchor.source_id(), SourceId::new(227));
    assert_eq!(anchor.range().start(), range.start);
    assert_eq!(anchor.range().end(), range.end);
    assert_eq!(anchor.fragment(), authored_fragment(source, range));
}

fn assert_established_selected_binding_context(fixture: &GrammarEvidenceFixture) {
    assert_eq!(
        fixture.source.get(..fixture.subject.start),
        Some(fixture.context_prefix)
    );

    match fixture.context_prefix {
        "let " => assert_eq!(fixture.subject.start, 4),
        "let a" => {
            assert_eq!(fixture.subject.start, 5);
            assert_eq!(fixture.source.get(4..5), Some("a"));
        }
        "let" => {
            assert_eq!(fixture.subject.start, 3);
            assert_eq!(
                formed_unicode_escape_end(fixture.source, 3),
                None,
                "malformed/non-CodePoint escape must not extend literal let as IdentifierName"
            );
        }
        _ => panic!("fixture does not establish the bounded selected binding context"),
    }
}

#[test]
fn existing_syntax_gold_remains_subjectless_while_grammar_evidence_is_source_backed() {
    assert_eq!(BASE_GRAMMAR_EVIDENCE.len(), 4);

    for fixture in BASE_GRAMMAR_EVIDENCE {
        let gold_id = fixture.gold_id.expect("base evidence must reuse #222 gold");
        assert_eq!(gold_source(gold_id), Some(fixture.source));
        assert_eq!(
            gold_subject_range(gold_id),
            None,
            "whole-standard gold must not acquire a fabricated universal grammar subject"
        );

        assert_established_selected_binding_context(fixture);
        let authored = authored_fragment(fixture.source, fixture.subject);
        assert_eq!(classify_grammar_subject(authored), Some(fixture.category));
        assert_core_source_anchor(fixture.source, fixture.subject);
        assert_eq!(
            fixture.expectation,
            SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence
        );
    }
}

#[test]
fn bounded_part_position_evidence_uses_truthful_authored_subjects_after_valid_prefix() {
    assert_eq!(PART_POSITION_EVIDENCE.len(), 4);

    for fixture in PART_POSITION_EVIDENCE {
        assert_eq!(fixture.gold_id, None);
        assert_established_selected_binding_context(fixture);
        assert_eq!(fixture.source.get(4..5), Some("a"));
        assert_eq!(
            classify_grammar_subject(authored_fragment(fixture.source, fixture.subject)),
            Some(fixture.category)
        );
        assert_core_source_anchor(fixture.source, fixture.subject);
        assert_eq!(
            fixture.expectation,
            SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence
        );
    }
}

#[test]
fn malformed_or_non_code_point_escape_after_literal_keyword_does_not_extend_keyword() {
    assert_eq!(KEYWORD_BOUNDARY_EVIDENCE.len(), 2);

    for fixture in KEYWORD_BOUNDARY_EVIDENCE {
        assert_established_selected_binding_context(fixture);
        assert_eq!(formed_unicode_escape_end(fixture.source, 3), None);
        assert_eq!(
            classify_grammar_subject(authored_fragment(fixture.source, fixture.subject)),
            Some(fixture.category)
        );
        assert_core_source_anchor(fixture.source, fixture.subject);
        assert_eq!(
            fixture.expectation,
            SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence
        );
    }
}

#[test]
fn locally_definitive_grammar_evidence_survives_later_unowned_tail_after_context_is_fixed() {
    assert_eq!(EARLY_DEFINITIVE_EVIDENCE.len(), 3);

    for fixture in EARLY_DEFINITIVE_EVIDENCE {
        assert_established_selected_binding_context(fixture);
        assert_eq!(
            classify_grammar_subject(authored_fragment(fixture.source, fixture.subject)),
            Some(fixture.category)
        );
        assert!(
            fixture
                .source
                .get(fixture.subject.end..)
                .is_some_and(|tail| tail.contains("foo();")),
            "challenge must retain a later source tail outside the selected declaration slice"
        );
        assert_eq!(
            fixture.expectation,
            SelectedGrammarEvidenceExpectation::CandidateDefinitiveGrammarEvidence,
            "once the selected binding context is independently fixed, malformed lexical spelling is already sufficient to prove whole-source grammar invalidity"
        );
        assert_core_source_anchor(fixture.source, fixture.subject);
    }
}

#[test]
fn unowned_context_cannot_gain_selected_grammar_evidence_by_searching_for_escape_text() {
    assert_eq!(UNOWNED_CONTEXT_EVIDENCE.len(), 3);

    for fixture in UNOWNED_CONTEXT_EVIDENCE {
        assert_eq!(
            classify_grammar_subject(authored_fragment(fixture.source, fixture.subject)),
            Some(fixture.category)
        );
        assert_core_source_anchor(fixture.source, fixture.subject);
        assert_eq!(
            fixture.expectation,
            SelectedGrammarEvidenceExpectation::UnsupportedCoverage
        );
    }

    let formed_prefix = &UNOWNED_CONTEXT_EVIDENCE[2];
    assert_eq!(formed_prefix.source, r"let\u002D\u{};");
    assert_eq!(
        formed_unicode_escape_end(formed_prefix.source, 3),
        Some(9),
        "the first formed UES already extends the IdentifierName candidate beyond literal let"
    );
    assert_eq!(formed_prefix.subject, EvidenceRange::new(9, 13));
}

#[test]
fn formed_ues_positional_failures_remain_static_semantics_evidence_not_grammar_evidence() {
    let contrasts = [
        (
            "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
            EvidenceRange::new(4, 10),
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPED-PART-HYPHEN-001",
            EvidenceRange::new(5, 11),
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPED-START-SURROGATE-FIXED-001",
            EvidenceRange::new(4, 10),
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPED-START-SURROGATE-BRACED-001",
            EvidenceRange::new(4, 12),
        ),
    ];

    for (gold_id, subject) in contrasts {
        let source = gold_source(gold_id).expect("#222 EE-01 gold must remain present");
        assert_eq!(
            gold_subject_range(gold_id),
            Some((subject.start, subject.end))
        );
        let authored = authored_fragment(source, subject);
        assert_eq!(
            classify_grammar_subject(authored),
            None,
            "formed UES positional failures must not be reclassified as malformed grammar evidence"
        );
        assert_eq!(formed_unicode_escape_end(source, subject.start), Some(subject.end));
    }
}

#[test]
fn grammar_evidence_validation_dependency_direction_stays_candidate_independent() {
    let validation_source = include_str!("qualification_grammar_evidence_validation_tests.rs");
    for forbidden in [
        concat!("selected_binding_", "identifier::"),
        concat!("selected_lexical_", "slice::"),
        concat!("selected_static_", "semantics::"),
        concat!("selected_qualification_", "integration::"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("evaluate_selected_", "static_semantics"),
        concat!("attempt_selected_", "qualification"),
        concat!("DefinitiveGrammarRejection", "Evidence"),
        concat!("Qualification", "Outcome"),
    ] {
        assert!(
            !validation_source.contains(forbidden),
            "grammar-evidence oracle must not import/call production behavior: {forbidden}"
        );
    }

    for production_source in [
        include_str!("selected_binding_identifier.rs"),
        include_str!("selected_lexical_slice.rs"),
        include_str!("selected_static_semantics.rs"),
        include_str!("selected_qualification_integration.rs"),
        include_str!("qualification.rs"),
    ] {
        for forbidden in [
            concat!("qualification_grammar_", "evidence_validation_tests"),
            concat!("qualification_validation_", "tests::gold_source"),
        ] {
            assert!(
                !production_source.contains(forbidden),
                "production must not depend on validation-only grammar evidence: {forbidden}"
            );
        }
    }
}
