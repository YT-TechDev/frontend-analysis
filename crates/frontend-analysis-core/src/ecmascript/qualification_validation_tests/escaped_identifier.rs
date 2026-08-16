//! Candidate-independent escaped `BindingIdentifier` validation for Issue #222.
//!
//! This module freezes only the accepted validation oracle. It deliberately
//! does not call the selected production lexical/static-semantics candidate.

use super::super::unicode::{is_id_continue, is_id_start};
use super::focused::FOCUSED_RULE_EVIDENCE;
use super::gold::fixtures;
use super::inventory::{RULE_UNITS, RuleUnit, RuleUnitKind};
use super::model::{
    ExpectedProcessing, ExpectedQualification, GoldFixture, GoldRange, ImplementationCoverage,
    RequestApplicability,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UnicodeEscapeFormation {
    end: usize,
    code_point: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ee01Occurrence {
    rule_id: &'static str,
    subject: GoldRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SameBindingEe01Fixture {
    source: &'static str,
    primary: Ee01Occurrence,
    later_invalid_occurrences: &'static [Ee01Occurrence],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuleWitness {
    rule_id: &'static str,
    subject: GoldRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RulePrecedenceFixture {
    source: &'static str,
    primary: RuleWitness,
    lower_priority: &'static [RuleWitness],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeywordBoundaryExpectation {
    ExtendedIdentifierNameCandidate,
    NoValidEscapeExtension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KeywordBoundaryFixture {
    source: &'static str,
    keyword_len: usize,
    expectation: KeywordBoundaryExpectation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedOwnershipExpectation {
    OwnedEe01(Ee01Occurrence),
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelectedOwnershipFixture {
    source: &'static str,
    expectation: SelectedOwnershipExpectation,
    tentative_ee01: Option<Ee01Occurrence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SemanticEqualityFixture {
    gold_id: &'static str,
    source: &'static str,
    first: GoldRange,
    second: GoldRange,
    first_code_points: &'static [u32],
    second_code_points: &'static [u32],
    equal: bool,
}

const SAME_BINDING_EE01: &[SameBindingEe01Fixture] = &[
    SameBindingEe01Fixture {
        source: r"let \u0030\u002D;",
        primary: Ee01Occurrence {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(4, 10),
        },
        later_invalid_occurrences: &[Ee01Occurrence {
            rule_id: "EE-01-R02",
            subject: GoldRange::new(10, 16),
        }],
    },
    SameBindingEe01Fixture {
        source: r"let a\u002D\u002E;",
        primary: Ee01Occurrence {
            rule_id: "EE-01-R02",
            subject: GoldRange::new(5, 11),
        },
        later_invalid_occurrences: &[Ee01Occurrence {
            rule_id: "EE-01-R02",
            subject: GoldRange::new(11, 17),
        }],
    },
    SameBindingEe01Fixture {
        source: r"let \uD800\uDC00;",
        primary: Ee01Occurrence {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(4, 10),
        },
        later_invalid_occurrences: &[Ee01Occurrence {
            rule_id: "EE-01-R02",
            subject: GoldRange::new(10, 16),
        }],
    },
];

const RULE_PRECEDENCE_FIXTURES: &[RulePrecedenceFixture] = &[
    RulePrecedenceFixture {
        source: r"let let, \u0030;",
        primary: RuleWitness {
            rule_id: "EE-15-R01",
            subject: GoldRange::new(4, 7),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(9, 15),
        }],
    },
    RulePrecedenceFixture {
        source: r"let \u0030, let;",
        primary: RuleWitness {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(4, 10),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-15-R01",
            subject: GoldRange::new(12, 15),
        }],
    },
    RulePrecedenceFixture {
        source: r"let \u0069f, let;",
        primary: RuleWitness {
            rule_id: "EE-04-R08",
            subject: GoldRange::new(4, 11),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-15-R01",
            subject: GoldRange::new(13, 16),
        }],
    },
    RulePrecedenceFixture {
        source: r"let let, \u0069f;",
        primary: RuleWitness {
            rule_id: "EE-15-R01",
            subject: GoldRange::new(4, 7),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-04-R08",
            subject: GoldRange::new(9, 16),
        }],
    },
    RulePrecedenceFixture {
        source: r"let x, x, \u0030;",
        primary: RuleWitness {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(10, 16),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-15-R02",
            subject: GoldRange::new(7, 8),
        }],
    },
    RulePrecedenceFixture {
        source: r"let x, x, \u0069f;",
        primary: RuleWitness {
            rule_id: "EE-04-R08",
            subject: GoldRange::new(10, 17),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-15-R02",
            subject: GoldRange::new(7, 8),
        }],
    },
    RulePrecedenceFixture {
        source: r"const x, \u0030;",
        primary: RuleWitness {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(9, 15),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-15-R03",
            subject: GoldRange::new(6, 7),
        }],
    },
    RulePrecedenceFixture {
        source: r"const x, \u0069f;",
        primary: RuleWitness {
            rule_id: "EE-04-R08",
            subject: GoldRange::new(9, 16),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-15-R03",
            subject: GoldRange::new(6, 7),
        }],
    },
    RulePrecedenceFixture {
        source: r"let x, x; let \u0030;",
        primary: RuleWitness {
            rule_id: "EE-15-R02",
            subject: GoldRange::new(7, 8),
        },
        lower_priority: &[
            RuleWitness {
                rule_id: "EE-01-R01",
                subject: GoldRange::new(14, 20),
            },
            RuleWitness {
                rule_id: "EE-36-R01",
                subject: GoldRange::new(7, 8),
            },
        ],
    },
    RulePrecedenceFixture {
        source: r"const x; let \u0030;",
        primary: RuleWitness {
            rule_id: "EE-15-R03",
            subject: GoldRange::new(6, 7),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(13, 19),
        }],
    },
    RulePrecedenceFixture {
        source: r"let x; let x; let \u0030;",
        primary: RuleWitness {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(18, 24),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-36-R01",
            subject: GoldRange::new(11, 12),
        }],
    },
    RulePrecedenceFixture {
        source: r"let x; let x; let \u0069f;",
        primary: RuleWitness {
            rule_id: "EE-04-R08",
            subject: GoldRange::new(18, 25),
        },
        lower_priority: &[RuleWitness {
            rule_id: "EE-36-R01",
            subject: GoldRange::new(11, 12),
        }],
    },
];

const KEYWORD_BOUNDARY_FIXTURES: &[KeywordBoundaryFixture] = &[
    KeywordBoundaryFixture {
        source: r"let\u0030",
        keyword_len: 3,
        expectation: KeywordBoundaryExpectation::ExtendedIdentifierNameCandidate,
    },
    KeywordBoundaryFixture {
        source: r"let\u002D",
        keyword_len: 3,
        expectation: KeywordBoundaryExpectation::ExtendedIdentifierNameCandidate,
    },
    KeywordBoundaryFixture {
        source: r"let\u00001",
        keyword_len: 3,
        expectation: KeywordBoundaryExpectation::ExtendedIdentifierNameCandidate,
    },
    KeywordBoundaryFixture {
        source: r"const\u0030",
        keyword_len: 5,
        expectation: KeywordBoundaryExpectation::ExtendedIdentifierNameCandidate,
    },
    KeywordBoundaryFixture {
        source: r"let\u{00000061}",
        keyword_len: 3,
        expectation: KeywordBoundaryExpectation::ExtendedIdentifierNameCandidate,
    },
    KeywordBoundaryFixture {
        source: r"let\u{}",
        keyword_len: 3,
        expectation: KeywordBoundaryExpectation::NoValidEscapeExtension,
    },
    KeywordBoundaryFixture {
        source: r"let\u{110000}",
        keyword_len: 3,
        expectation: KeywordBoundaryExpectation::NoValidEscapeExtension,
    },
    KeywordBoundaryFixture {
        source: r"let\u002D\u{}",
        keyword_len: 3,
        expectation: KeywordBoundaryExpectation::ExtendedIdentifierNameCandidate,
    },
];

const SELECTED_OWNERSHIP_FIXTURES: &[SelectedOwnershipFixture] = &[
    SelectedOwnershipFixture {
        source: r"let \u0030;",
        expectation: SelectedOwnershipExpectation::OwnedEe01(Ee01Occurrence {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(4, 10),
        }),
        tentative_ee01: None,
    },
    SelectedOwnershipFixture {
        source: r"let \u0030; foo();",
        expectation: SelectedOwnershipExpectation::UnsupportedCoverage,
        tentative_ee01: Some(Ee01Occurrence {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(4, 10),
        }),
    },
    SelectedOwnershipFixture {
        source: r"let \u0030 = foo;",
        expectation: SelectedOwnershipExpectation::UnsupportedCoverage,
        tentative_ee01: Some(Ee01Occurrence {
            rule_id: "EE-01-R01",
            subject: GoldRange::new(4, 10),
        }),
    },
    SelectedOwnershipFixture {
        source: r"let a\u00001\u{};",
        expectation: SelectedOwnershipExpectation::UnsupportedCoverage,
        tentative_ee01: Some(Ee01Occurrence {
            rule_id: "EE-01-R02",
            subject: GoldRange::new(5, 11),
        }),
    },
    SelectedOwnershipFixture {
        source: r"let\u002D;",
        expectation: SelectedOwnershipExpectation::UnsupportedCoverage,
        tentative_ee01: None,
    },
    SelectedOwnershipFixture {
        source: r"let\u{};",
        expectation: SelectedOwnershipExpectation::UnsupportedCoverage,
        tentative_ee01: None,
    },
];

const SEMANTIC_EQUALITY_FIXTURES: &[SemanticEqualityFixture] = &[
    SemanticEqualityFixture {
        gold_id: "JS-GOLD-LEXDECL-ESCAPED-DUPBOUNDNAMES-001",
        source: r"let \u0061, a;",
        first: GoldRange::new(4, 10),
        second: GoldRange::new(12, 13),
        first_code_points: &[0x61],
        second_code_points: &[0x61],
        equal: true,
    },
    SemanticEqualityFixture {
        gold_id: "JS-GOLD-LEXDECL-DOLLAR-ESCAPED-DUPBOUNDNAMES-001",
        source: r"let $, \u0024;",
        first: GoldRange::new(4, 5),
        second: GoldRange::new(7, 13),
        first_code_points: &[0x24],
        second_code_points: &[0x24],
        equal: true,
    },
    SemanticEqualityFixture {
        gold_id: "JS-GOLD-LEXDECL-UNDERSCORE-ESCAPED-DUPBOUNDNAMES-001",
        source: r"let _, \u005F;",
        first: GoldRange::new(4, 5),
        second: GoldRange::new(7, 13),
        first_code_points: &[0x5F],
        second_code_points: &[0x5F],
        equal: true,
    },
    SemanticEqualityFixture {
        gold_id: "JS-GOLD-LEXDECL-ESCAPED-CANONICAL-DISTINCT-001",
        source: r"let \u00E9, e\u0301;",
        first: GoldRange::new(4, 10),
        second: GoldRange::new(12, 19),
        first_code_points: &[0xE9],
        second_code_points: &[0x65, 0x301],
        equal: false,
    },
    SemanticEqualityFixture {
        gold_id: "JS-GOLD-LEXDECL-SUPPLEMENTARY-ESCAPED-DUPBOUNDNAMES-001",
        source: r"let \u{1D49C}, 𝒜;",
        first: GoldRange::new(4, 13),
        second: GoldRange::new(15, 19),
        first_code_points: &[0x1D49C],
        second_code_points: &[0x1D49C],
        equal: true,
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

fn parse_braced_code_point(digits: &[u8]) -> Option<u32> {
    if digits.is_empty() || digits.iter().any(|byte| ascii_hex_value(*byte).is_none()) {
        return None;
    }

    let first_significant = digits.iter().position(|byte| *byte != b'0');
    let significant = match first_significant {
        Some(index) => &digits[index..],
        None => return Some(0),
    };

    // The six-digit bound applies only after removing leading zeroes because it
    // follows from the mathematical Code Point maximum. It is not an authored
    // UnicodeEscapeSequence digit-count limit.
    if significant.len() > 6 {
        return None;
    }

    let mut value = 0_u32;
    for byte in significant {
        value = value * 16 + ascii_hex_value(*byte)?;
    }
    (value <= 0x10_FFFF).then_some(value)
}

fn formed_unicode_escape_at(source: &str, start: usize) -> Option<UnicodeEscapeFormation> {
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
        let code_point = parse_braced_code_point(&bytes[digits_start..end])?;
        return Some(UnicodeEscapeFormation {
            end: end + 1,
            code_point,
        });
    }

    let end = payload.checked_add(4)?;
    let digits = bytes.get(payload..end)?;
    if !digits.iter().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut code_point = 0_u32;
    for byte in digits {
        code_point = code_point * 16 + ascii_hex_value(*byte)?;
    }
    Some(UnicodeEscapeFormation { end, code_point })
}

fn ecmascript_identifier_start(code_point: u32) -> bool {
    matches!(code_point, 0x24 | 0x5F) || is_id_start(code_point)
}

fn ecmascript_identifier_part(code_point: u32) -> bool {
    ecmascript_identifier_start(code_point)
        || is_id_continue(code_point)
        || matches!(code_point, 0x200C | 0x200D)
}

fn decode_fixture_identifier_code_points(authored: &str) -> Option<Vec<u32>> {
    let mut decoded = Vec::new();
    let mut offset = 0;

    while offset < authored.len() {
        if authored.as_bytes().get(offset) == Some(&b'\\') {
            let formation = formed_unicode_escape_at(authored, offset)?;
            decoded.push(formation.code_point);
            offset = formation.end;
            continue;
        }

        let literal = authored.get(offset..)?.chars().next()?;
        decoded.push(literal as u32);
        offset += literal.len_utf8();
    }

    Some(decoded)
}

fn assert_ee01_occurrence(source: &str, occurrence: Ee01Occurrence) -> UnicodeEscapeFormation {
    let _ = active_rule(occurrence.rule_id);
    let authored = assert_authored_range(source, occurrence.subject);
    assert!(authored.starts_with(r"\u"));

    let formation = formed_unicode_escape_at(source, occurrence.subject.start)
        .expect("EE-01 occurrence must contain a syntactically formed UnicodeEscapeSequence");
    assert_eq!(formation.end, occurrence.subject.end);

    match occurrence.rule_id {
        "EE-01-R01" => assert!(!ecmascript_identifier_start(formation.code_point)),
        "EE-01-R02" => assert!(!ecmascript_identifier_part(formation.code_point)),
        _ => panic!("unexpected EE-01 occurrence rule {}", occurrence.rule_id),
    }

    formation
}

fn active_rule(rule_id: &str) -> &'static RuleUnit {
    let rule = RULE_UNITS
        .iter()
        .find(|rule| rule.id == rule_id)
        .unwrap_or_else(|| panic!("escaped-Identifier validation maps to missing rule {rule_id}"));
    assert_eq!(rule.kind, RuleUnitKind::NormativeRule);
    rule
}

fn gold(gold_id: &str) -> GoldFixture {
    fixtures()
        .into_iter()
        .find(|fixture| fixture.id == gold_id)
        .unwrap_or_else(|| panic!("missing escaped-Identifier gold fixture {gold_id}"))
}

fn assert_complete_supported_gold(
    gold_id: &str,
    source: &str,
    qualification: ExpectedQualification,
    subject: Option<GoldRange>,
) {
    let fixture = gold(gold_id);
    assert_eq!(fixture.source, source);
    assert_eq!(fixture.applicability, RequestApplicability::Supported);
    assert_eq!(fixture.processing, ExpectedProcessing::Complete);
    assert_eq!(fixture.qualification, Some(qualification));
    assert_eq!(
        fixture.implementation_coverage,
        ImplementationCoverage::Pending
    );
    assert_eq!(fixture.subject, subject);
}

fn assert_authored_range(source: &str, range: GoldRange) -> &str {
    assert!(range.is_valid_for(source));
    source
        .get(range.start..range.end)
        .expect("validated UTF-8 byte range must slice the authoritative source")
}

#[test]
fn escaped_identifier_start_and_part_gold_separate_ues_formation_from_positional_validity() {
    let cases = [
        (
            "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
            r"let \u0030;",
            "EE-01-R01",
            GoldRange::new(4, 10),
            0x30,
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPED-PART-HYPHEN-001",
            r"let a\u002D;",
            "EE-01-R02",
            GoldRange::new(5, 11),
            0x2D,
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPED-START-SURROGATE-FIXED-001",
            r"let \uD800;",
            "EE-01-R01",
            GoldRange::new(4, 10),
            0xD800,
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPED-START-SURROGATE-BRACED-001",
            r"let \u{D800};",
            "EE-01-R01",
            GoldRange::new(4, 12),
            0xD800,
        ),
    ];

    for (gold_id, source, rule_id, subject, expected_code_point) in cases {
        assert_complete_supported_gold(
            gold_id,
            source,
            ExpectedQualification::StaticSemanticsRejected,
            Some(subject),
        );
        let rule = active_rule(rule_id);
        assert!(rule.gold_fixture_ids.contains(&gold_id));

        let formation = formed_unicode_escape_at(source, subject.start)
            .unwrap_or_else(|| panic!("{gold_id} must contain a syntactically formed UES"));
        assert_eq!(formation.end, subject.end);
        assert_eq!(formation.code_point, expected_code_point);
        assert_eq!(
            assert_authored_range(source, subject),
            &source[subject.start..subject.end]
        );

        match rule_id {
            "EE-01-R01" => assert!(!ecmascript_identifier_start(formation.code_point)),
            "EE-01-R02" => assert!(!ecmascript_identifier_part(formation.code_point)),
            _ => unreachable!(),
        }
    }
}

#[test]
fn malformed_or_non_code_point_escapes_are_syntax_gold_not_ee01_coverage() {
    let malformed = [
        (
            "JS-GOLD-IDENTIFIER-ESCAPE-MALFORMED-EMPTY-BRACED-001",
            r"let \u{};",
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPE-MALFORMED-SHORT-FIXED-001",
            r"let \u0;",
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPE-MALFORMED-UNCLOSED-BRACED-001",
            r"let \u{61",
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPE-NONCODEPOINT-001",
            r"let \u{110000};",
        ),
    ];

    for (gold_id, source) in malformed {
        assert_complete_supported_gold(
            gold_id,
            source,
            ExpectedQualification::SyntaxRejected,
            None,
        );
        assert_eq!(formed_unicode_escape_at(source, 4), None);
        for rule_id in ["EE-01-R01", "EE-01-R02"] {
            assert!(
                !active_rule(rule_id).gold_fixture_ids.contains(&gold_id),
                "malformed/non-CodePoint syntax must not count as {rule_id} coverage"
            );
        }
    }
}

#[test]
fn escaped_semantic_equality_uses_code_point_sequences_without_normalization() {
    for fixture in SEMANTIC_EQUALITY_FIXTURES {
        let first_authored = assert_authored_range(fixture.source, fixture.first);
        let second_authored = assert_authored_range(fixture.source, fixture.second);
        assert_ne!(first_authored, second_authored);
        assert_eq!(
            decode_fixture_identifier_code_points(first_authored).as_deref(),
            Some(fixture.first_code_points)
        );
        assert_eq!(
            decode_fixture_identifier_code_points(second_authored).as_deref(),
            Some(fixture.second_code_points)
        );
        assert_eq!(
            fixture.first_code_points == fixture.second_code_points,
            fixture.equal
        );

        let gold = gold(fixture.gold_id);
        assert_eq!(gold.source, fixture.source);
        if fixture.equal {
            assert_eq!(
                gold.qualification,
                Some(ExpectedQualification::StaticSemanticsRejected)
            );
            assert_eq!(gold.subject, Some(fixture.second));
            assert!(
                active_rule("EE-15-R02")
                    .gold_fixture_ids
                    .contains(&fixture.gold_id)
            );
        } else {
            assert_eq!(gold.qualification, Some(ExpectedQualification::Qualified));
            assert_eq!(gold.subject, None);
        }
    }

    let canonical = &SEMANTIC_EQUALITY_FIXTURES[3];
    assert_eq!(canonical.first_code_points, &[0xE9]);
    assert_eq!(canonical.second_code_points, &[0x65, 0x301]);
    assert!(
        !canonical.equal,
        "validation must not apply Unicode normalization"
    );

    assert!(!is_id_start(0x24));
    assert!(!is_id_start(0x5F));
    assert!(ecmascript_identifier_start(0x24));
    assert!(ecmascript_identifier_start(0x5F));
    assert!(ecmascript_identifier_part(0x24));
    assert!(ecmascript_identifier_part(0x5F));

    assert!(is_id_start(0x1D49C));
    assert!(ecmascript_identifier_start(0x1D49C));
}

#[test]
fn escaped_reserved_word_and_semantic_let_have_distinct_primary_rules() {
    let cases = [
        (
            "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
            r"let \u0069f;",
            "EE-04-R08",
            GoldRange::new(4, 11),
        ),
        (
            "JS-GOLD-LEXDECL-ESCAPED-LET-BINDING-001",
            r"let \u006Cet;",
            "EE-15-R01",
            GoldRange::new(4, 12),
        ),
    ];

    for (gold_id, source, rule_id, subject) in cases {
        assert_complete_supported_gold(
            gold_id,
            source,
            ExpectedQualification::StaticSemanticsRejected,
            Some(subject),
        );
        assert!(active_rule(rule_id).gold_fixture_ids.contains(&gold_id));
        let focused = FOCUSED_RULE_EVIDENCE
            .iter()
            .find(|evidence| evidence.fixture_id == gold_id)
            .unwrap_or_else(|| panic!("{gold_id} must retain focused ownership evidence"));
        assert_eq!(focused.primary_rule_id, rule_id);
        assert_eq!(focused.primary_subject, subject);

        let semantic =
            decode_fixture_identifier_code_points(assert_authored_range(source, subject))
                .expect("escaped binding must have a formed semantic code-point sequence");
        let expected = match rule_id {
            "EE-04-R08" => "if",
            "EE-15-R01" => "let",
            _ => unreachable!(),
        };
        assert_eq!(
            semantic,
            expected.chars().map(|ch| ch as u32).collect::<Vec<_>>()
        );
    }
}

#[test]
fn escaped_contextual_names_remain_non_triggers_in_non_strict_script_gold() {
    const GOLD_ID: &str = "JS-GOLD-LEXDECL-ESCAPED-CONTEXTUAL-NAMES-001";
    const SOURCE: &str =
        r"let \u0061wait, \u0079ield, \u0073tatic, \u0069mplements, \u0061rguments, \u0065val;";

    assert_complete_supported_gold(GOLD_ID, SOURCE, ExpectedQualification::Qualified, None);

    let bindings = [
        (GoldRange::new(4, 14), "await", &["EE-04-R04"][..]),
        (
            GoldRange::new(16, 26),
            "yield",
            &["EE-04-R02", "EE-04-R03"][..],
        ),
        (GoldRange::new(28, 39), "static", &["EE-04-R07"][..]),
        (GoldRange::new(41, 56), "implements", &["EE-04-R07"][..]),
        (GoldRange::new(58, 72), "arguments", &["EE-04-R01"][..]),
        (GoldRange::new(74, 83), "eval", &["EE-04-R01"][..]),
    ];

    for (authored, semantic_name, rule_ids) in bindings {
        let authored_text = assert_authored_range(SOURCE, authored);
        assert!(authored_text.starts_with(r"\u"));
        assert_ne!(authored_text, semantic_name);
        assert_eq!(
            decode_fixture_identifier_code_points(authored_text),
            Some(semantic_name.chars().map(|ch| ch as u32).collect())
        );
        for rule_id in rule_ids {
            let _ = active_rule(rule_id);
        }
    }
}

#[test]
fn literal_keyword_boundary_depends_on_formed_ues_without_lexical_backtracking() {
    for fixture in KEYWORD_BOUNDARY_FIXTURES {
        let formation = formed_unicode_escape_at(fixture.source, fixture.keyword_len);
        match fixture.expectation {
            KeywordBoundaryExpectation::ExtendedIdentifierNameCandidate => {
                assert!(
                    formation.is_some(),
                    "{} must have a valid UES immediately after the keyword spelling",
                    fixture.source
                );
            }
            KeywordBoundaryExpectation::NoValidEscapeExtension => {
                assert_eq!(
                    formation, None,
                    "{} must not extend the keyword",
                    fixture.source
                );
            }
        }
    }

    let late_failure = KEYWORD_BOUNDARY_FIXTURES
        .iter()
        .find(|fixture| fixture.source == r"let\u002D\u{}")
        .expect("late malformed-tail keyword witness must exist");
    let first = formed_unicode_escape_at(late_failure.source, late_failure.keyword_len)
        .expect("the first UES must already extend the IdentifierName candidate");
    assert_eq!(first.end, 9);
    assert_eq!(first.code_point, 0x2D);
    assert_eq!(
        formed_unicode_escape_at(late_failure.source, first.end),
        None
    );
}

#[test]
fn selected_ownership_fixtures_keep_local_candidates_separate_from_whole_source_commit() {
    assert_eq!(SELECTED_OWNERSHIP_FIXTURES.len(), 6);

    for fixture in SELECTED_OWNERSHIP_FIXTURES {
        match fixture.expectation {
            SelectedOwnershipExpectation::OwnedEe01(occurrence) => {
                assert_eq!(fixture.tentative_ee01, None);
                let _ = assert_ee01_occurrence(fixture.source, occurrence);
            }
            SelectedOwnershipExpectation::UnsupportedCoverage => {
                if let Some(tentative) = fixture.tentative_ee01 {
                    let _ = assert_ee01_occurrence(fixture.source, tentative);
                }
            }
        }
    }

    let rollback = SELECTED_OWNERSHIP_FIXTURES
        .iter()
        .find(|fixture| fixture.source == r"let a\u00001\u{};")
        .expect("rollback witness must exist");
    assert_eq!(
        rollback.expectation,
        SelectedOwnershipExpectation::UnsupportedCoverage
    );
    assert_eq!(
        rollback.tentative_ee01,
        Some(Ee01Occurrence {
            rule_id: "EE-01-R02",
            subject: GoldRange::new(5, 11),
        })
    );
    assert_eq!(formed_unicode_escape_at(rollback.source, 12), None);
}

#[test]
fn declaration_local_precedence_matrix_defeats_global_rule_and_byte_ordering() {
    assert_eq!(RULE_PRECEDENCE_FIXTURES.len(), 12);

    for fixture in RULE_PRECEDENCE_FIXTURES {
        let _ = active_rule(fixture.primary.rule_id);
        assert!(fixture.primary.subject.is_valid_for(fixture.source));
        assert!(!assert_authored_range(fixture.source, fixture.primary.subject).is_empty());
        for lower in fixture.lower_priority {
            let _ = active_rule(lower.rule_id);
            assert_ne!(lower.rule_id, fixture.primary.rule_id);
            assert!(lower.subject.is_valid_for(fixture.source));
        }
    }

    assert_eq!(RULE_PRECEDENCE_FIXTURES[0].primary.rule_id, "EE-15-R01");
    assert_eq!(RULE_PRECEDENCE_FIXTURES[1].primary.rule_id, "EE-01-R01");
    assert_eq!(RULE_PRECEDENCE_FIXTURES[2].primary.rule_id, "EE-04-R08");
    assert_eq!(RULE_PRECEDENCE_FIXTURES[3].primary.rule_id, "EE-15-R01");

    let later_tier_a = &RULE_PRECEDENCE_FIXTURES[4];
    assert!(
        later_tier_a.primary.subject.start > later_tier_a.lower_priority[0].subject.start,
        "Tier-A ownership must outrank an earlier-byte R02 witness in the same declaration"
    );

    let earlier_declaration = &RULE_PRECEDENCE_FIXTURES[9];
    assert_eq!(earlier_declaration.primary.rule_id, "EE-15-R03");
    assert_eq!(earlier_declaration.lower_priority[0].rule_id, "EE-01-R01");

    for fixture in &RULE_PRECEDENCE_FIXTURES[10..] {
        assert!(
            fixture
                .lower_priority
                .iter()
                .any(|witness| witness.rule_id == "EE-36-R01"),
            "later declaration-local Tier-A evidence must outrank eager EE-36"
        );
    }
}

#[test]
fn same_binding_ee01_retains_per_occurrence_rule_identity_and_source_order() {
    assert_eq!(SAME_BINDING_EE01.len(), 3);

    for fixture in SAME_BINDING_EE01 {
        let _ = assert_ee01_occurrence(fixture.source, fixture.primary);

        let mut previous_end = fixture.primary.subject.end;
        for later in fixture.later_invalid_occurrences {
            assert!(later.subject.start >= previous_end);
            let _ = assert_ee01_occurrence(fixture.source, *later);
            previous_end = later.subject.end;
        }
    }

    assert_eq!(SAME_BINDING_EE01[0].primary.rule_id, "EE-01-R01");
    assert_eq!(
        SAME_BINDING_EE01[0].later_invalid_occurrences[0].rule_id,
        "EE-01-R02"
    );
    assert_eq!(SAME_BINDING_EE01[1].primary.rule_id, "EE-01-R02");
    assert_eq!(SAME_BINDING_EE01[2].primary.rule_id, "EE-01-R01");
    assert_eq!(
        SAME_BINDING_EE01[2].later_invalid_occurrences[0].rule_id,
        "EE-01-R02"
    );
}

#[test]
fn permanent_long_braced_gold_and_large_leading_zero_challenge_have_no_authored_digit_cap() {
    assert_complete_supported_gold(
        "JS-GOLD-LEXDECL-LONG-BRACED-ESCAPE-001",
        r"let \u{00000061};",
        ExpectedQualification::Qualified,
        None,
    );

    let permanent = formed_unicode_escape_at(r"\u{00000061}", 0)
        .expect("permanent long-braced gold must form a UES");
    assert_eq!(permanent.code_point, 0x61);
    assert_eq!(permanent.end, 12);

    let zeros = "0".repeat(4096);
    let source = format!(r"let \u{{{zeros}61}};");
    let formation = formed_unicode_escape_at(&source, 4)
        .expect("leading-zero challenge must not fail due to digit count or numeric overflow");
    assert_eq!(formation.code_point, 0x61);
    assert_eq!(formation.end, source.len() - 1);
    assert!(formation.end > 4096);
    assert!(ecmascript_identifier_start(formation.code_point));
}

#[test]
fn escaped_identifier_validation_dependency_direction_stays_candidate_independent() {
    let validation_source = include_str!("escaped_identifier.rs");
    for forbidden in [
        concat!("selected_lexical_", "slice::"),
        concat!("selected_static_", "semantics::"),
        concat!("selected_qualification_", "integration::"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("evaluate_selected_", "static_semantics"),
        concat!("attempt_selected_", "qualification"),
        concat!("Qualification", "Outcome"),
    ] {
        assert!(
            !validation_source.contains(forbidden),
            "escaped-Identifier oracle must not import/call candidate behavior: {forbidden}"
        );
    }

    for production_source in [
        include_str!("../selected_lexical_slice.rs"),
        include_str!("../selected_static_semantics.rs"),
        include_str!("../selected_qualification_integration.rs"),
    ] {
        for forbidden in [
            concat!("qualification_validation_", "tests"),
            concat!("escaped_", "identifier"),
        ] {
            assert!(
                !production_source.contains(forbidden),
                "production must not depend on validation-only oracle: {forbidden}"
            );
        }
    }
}
