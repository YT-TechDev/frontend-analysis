//! Candidate-independent successor oracle for the selected top-level `var`
//! EE-04-R08 initializer source-position frontier (Issue #340).
//!
//! This module fixes only future observable semantics. It does not call
//! production recognition, static semantics, qualification, correspondence,
//! Binding / Scope, or runtime code, and it does not prescribe a production
//! representation.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

use super::super::unicode::{is_id_continue, is_id_start};
use super::inventory::{CONTAINERS, RULE_UNITS, RuleUnitKind};

const ISSUE_ID: u64 = 340;
const RULE_ID: &str = "EE-04-R08";
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const POSITIVE_LIFECYCLE: &str = "SelectedAcceptedIncomplete";
const CURRENT_MATURITY: &str = "V1";
const EXPECTED_POST_VALIDATION_MATURITY: &str = "V3";
const REPRESENTATION_POLICY: &str =
    "candidate-independent authored evidence and ordering only; production representation deferred";

const ESCAPED_RESERVED_CONSTITUENT: &str = include_str!(
    "../qualification_selected_escaped_reserved_identifier_initializer_validation_tests.rs"
);
const HISTORICAL_VAR_ORACLE: &str =
    include_str!("../qualification_selected_top_level_variable_statement_validation_tests.rs");
const HISTORICAL_C1_ORACLE: &str =
    include_str!("selected_variable_statement_escaped_identifier_reference_initializer_frontier.rs");
const EOF_ASI_ORACLE: &str = include_str!("selected_variable_statement_eof_asi_frontier.rs");
const CURRENT_COMPLETION: &str = include_str!("selected_variable_statement_slice_completion.rs");
const THIS_SOURCE: &str = include_str!("selected_variable_statement_ee04_initializer_frontier.rs");

const EXPECTED_CHANGED_FILES: &[&str] = &[
    "crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/mod.rs",
    "crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/selected_variable_statement_ee04_initializer_frontier.rs",
];

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

const C6_RESERVED: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
];

const C1_NON_TRIGGER_NAMES: &[&str] = &[
    "yield",
    "await",
    "let",
    "static",
    "implements",
    "interface",
    "package",
    "private",
    "protected",
    "public",
    "eval",
    "arguments",
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
struct C6Candidate {
    range: ByteRange,
    decoded: &'static str,
}

const fn candidate(start: usize, end: usize, decoded: &'static str) -> C6Candidate {
    C6Candidate {
        range: ByteRange::new(start, end),
        decoded,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedPrimary {
    Static {
        rule_id: &'static str,
        subject: ByteRange,
    },
    Grammar {
        subject: ByteRange,
    },
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminator {
    AuthoredSemicolon,
    AutomaticAtEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fixture {
    id: &'static str,
    source: &'static str,
    c6_candidates: [Option<C6Candidate>; 2],
    selected_script_committed: bool,
    static_rejection_committed: bool,
    primary: ExpectedPrimary,
    pre_rejection_relation_candidates: usize,
    committed_relations: usize,
    terminator: Option<Terminator>,
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        id: "F1-basic-c6-successor",
        source: r"var x=\u0069f;",
        c6_candidates: [Some(candidate(6, 13, "if")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(6, 13),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F2-mixed-authored-escape-position",
        source: r"var x=n\u0075ll;",
        c6_candidates: [Some(candidate(6, 15, "null")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(6, 15),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F3-lhs-ee04-precedes-rhs-c6",
        source: r"var \u0069f=\u0074his;",
        c6_candidates: [Some(candidate(12, 21, "this")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(4, 11),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F4-earlier-tier1-local-precedes-later-c6",
        source: r"let let; var x=\u0069f;",
        c6_candidates: [Some(candidate(15, 22, "if")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: "EE-15-R01",
            subject: ByteRange::new(4, 7),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F5-earlier-c6-precedes-later-tier1-local",
        source: r"var x=\u0069f; let let;",
        c6_candidates: [Some(candidate(6, 13, "if")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(6, 13),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F6-multiple-c6-authored-order",
        source: r"var a=\u0069f,b=\u0074his;",
        c6_candidates: [
            Some(candidate(6, 13, "if")),
            Some(candidate(16, 25, "this")),
        ],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(6, 13),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F7-c6-precedes-tier2-block-duplicate",
        source: r"{ let a; let a; } var x=\u0069f;",
        c6_candidates: [Some(candidate(24, 31, "if")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(24, 31),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F8-c6-precedes-tier3-script-duplicate",
        source: r"let a; let a; var x=\u0069f;",
        c6_candidates: [Some(candidate(20, 27, "if")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(20, 27),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F9-c6-precedes-tier4-lexical-var-collision",
        source: r"let a; var a,x=\u0069f;",
        c6_candidates: [Some(candidate(15, 22, "if")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(15, 22),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F10-static-rejection-suppresses-earlier-relation",
        source: r"var a=\u0066oo,x=\u0069f;",
        c6_candidates: [Some(candidate(17, 24, "if")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(17, 24),
        },
        pre_rejection_relation_candidates: 1,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F11-decimal-c6-absent-three-declarator-composition",
        source: r"var a=1,b=\u0069f,c;",
        c6_candidates: [Some(candidate(10, 17, "if")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(10, 17),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AuthoredSemicolon),
    },
    Fixture {
        id: "F12-eof-only-asi",
        source: r"var x=\u0069f",
        c6_candidates: [Some(candidate(6, 13, "if")), None],
        selected_script_committed: true,
        static_rejection_committed: true,
        primary: ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(6, 13),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: Some(Terminator::AutomaticAtEof),
    },
    Fixture {
        id: "F13-later-incomplete-initializer-rolls-back-c6",
        source: r"var x=\u0069f,y=",
        c6_candidates: [Some(candidate(6, 13, "if")), None],
        selected_script_committed: false,
        static_rejection_committed: false,
        primary: ExpectedPrimary::None,
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: None,
    },
    Fixture {
        id: "F14-later-definitive-grammar-rejection-outranks-tentative-c6",
        source: r"var x=\u0069f,\u{};",
        c6_candidates: [Some(candidate(6, 13, "if")), None],
        selected_script_committed: false,
        static_rejection_committed: false,
        primary: ExpectedPrimary::Grammar {
            subject: ByteRange::new(14, 18),
        },
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: None,
    },
    Fixture {
        id: "F15-richer-expression-does-not-commit-c6-prefix",
        source: r"var x=\u0069f.foo;",
        c6_candidates: [Some(candidate(6, 13, "if")), None],
        selected_script_committed: false,
        static_rejection_committed: false,
        primary: ExpectedPrimary::None,
        pre_rejection_relation_candidates: 0,
        committed_relations: 0,
        terminator: None,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecodeFailure {
    MissingEscape,
    Malformed,
    NonCodePoint,
    InvalidStart,
    InvalidPart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Decoded {
    string_value: String,
}

fn hex(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

fn braced_code_point(digits: &[u8]) -> Result<u32, DecodeFailure> {
    if digits.is_empty() || digits.iter().any(|byte| hex(*byte).is_none()) {
        return Err(DecodeFailure::Malformed);
    }

    let significant = match digits.iter().position(|byte| *byte != b'0') {
        Some(index) => &digits[index..],
        None => return Ok(0),
    };
    if significant.len() > 6 {
        return Err(DecodeFailure::NonCodePoint);
    }

    let mut value = 0_u32;
    for byte in significant {
        value = value * 16 + hex(*byte).ok_or(DecodeFailure::Malformed)?;
    }
    (value <= 0x10_FFFF)
        .then_some(value)
        .ok_or(DecodeFailure::NonCodePoint)
}

fn escape_at(text: &str, start: usize) -> Result<(usize, u32), DecodeFailure> {
    let bytes = text.as_bytes();
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return Err(DecodeFailure::Malformed);
    }

    let payload = start + 2;
    if bytes.get(payload) == Some(&b'{') {
        let digits_start = payload + 1;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(|byte| byte.is_ascii_hexdigit()) {
            end += 1;
        }
        if bytes.get(end) != Some(&b'}') {
            return Err(DecodeFailure::Malformed);
        }
        return Ok((end + 1, braced_code_point(&bytes[digits_start..end])?));
    }

    let end = payload.checked_add(4).ok_or(DecodeFailure::Malformed)?;
    let digits = bytes.get(payload..end).ok_or(DecodeFailure::Malformed)?;
    if !digits.iter().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(DecodeFailure::Malformed);
    }

    let mut value = 0_u32;
    for byte in digits {
        value = value * 16 + hex(*byte).ok_or(DecodeFailure::Malformed)?;
    }
    Ok((end, value))
}

fn valid_start(code_point: u32) -> bool {
    matches!(code_point, 0x24 | 0x5F) || is_id_start(code_point)
}

fn valid_part(code_point: u32) -> bool {
    valid_start(code_point) || is_id_continue(code_point) || matches!(code_point, 0x200C | 0x200D)
}

fn decode_identifier(text: &str) -> Result<Decoded, DecodeFailure> {
    let mut offset = 0;
    let mut index = 0;
    let mut saw_escape = false;
    let mut string_value = String::new();

    while offset < text.len() {
        let (code_point, end, escaped) = if text.as_bytes().get(offset) == Some(&b'\\') {
            let (end, code_point) = escape_at(text, offset)?;
            (code_point, end, true)
        } else {
            let scalar = text[offset..]
                .chars()
                .next()
                .ok_or(DecodeFailure::Malformed)?;
            (scalar as u32, offset + scalar.len_utf8(), false)
        };

        let valid = if index == 0 {
            valid_start(code_point)
        } else {
            valid_part(code_point)
        };
        if !valid {
            return Err(if index == 0 {
                DecodeFailure::InvalidStart
            } else {
                DecodeFailure::InvalidPart
            });
        }

        let scalar = char::from_u32(code_point).ok_or(DecodeFailure::InvalidPart)?;
        string_value.push(scalar);
        saw_escape |= escaped;
        offset = end;
        index += 1;
    }

    if !saw_escape {
        return Err(DecodeFailure::MissingEscape);
    }

    Ok(Decoded { string_value })
}

fn fragment(source: &str, range: ByteRange) -> &str {
    source
        .get(range.start..range.end)
        .unwrap_or_else(|| panic!("{range:?} must be a UTF-8 source boundary in {source:?}"))
}

fn assert_authored_range(source: &str, range: ByteRange, source_id: u64) {
    let source = SourceText::new(SourceId::new(source_id), source.to_owned());
    let anchor = source
        .anchor(range.start, range.end)
        .unwrap_or_else(|_| panic!("{range:?} must be a valid authored range"));
    assert_eq!(anchor.range().start(), range.start);
    assert_eq!(anchor.range().end(), range.end);
}

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing fixture {id}"))
}

#[test]
fn corrected_authority_chain_and_two_file_boundary_are_explicit() {
    assert_eq!(ISSUE_ID, 340);
    assert_eq!(ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(UNICODE_VERSION, "17.0.0");
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert_eq!(CURRENT_MATURITY, "V1");
    assert_eq!(EXPECTED_POST_VALIDATION_MATURITY, "V3");
    assert_eq!(EXPECTED_CHANGED_FILES.len(), 2);
    assert!(EXPECTED_CHANGED_FILES[0].ends_with("qualification_validation_tests/mod.rs"));
    assert!(EXPECTED_CHANGED_FILES[1].ends_with("selected_variable_statement_ee04_initializer_frontier.rs"));
    assert!(REPRESENTATION_POLICY.contains("production representation deferred"));

    assert!(ESCAPED_RESERVED_CONSTITUENT.contains("Issue #263"));
    assert!(ESCAPED_RESERVED_CONSTITUENT.contains("assert_eq!(RESERVED.len(), 36);"));
    assert!(HISTORICAL_VAR_ORACLE.contains("escaped-reserved-var-remains-ee04-owned"));
    assert!(HISTORICAL_VAR_ORACLE.contains(r#"source: r\"var \\u0069f;\""#));
    assert!(HISTORICAL_C1_ORACLE.contains("UNCONDITIONALLY_RESERVED_NAME_COUNT: usize = 35"));
    assert!(HISTORICAL_C1_ORACLE.contains(r#"source: r\"var x=\\u0069f;\""#));
    assert!(EOF_ASI_ORACLE.contains("VAR-EOF-ASI-EE04-R08-001"));
    assert!(CURRENT_COMPLETION.contains("assert_eq!(required.len(), 9);"));
    assert!(CURRENT_COMPLETION.contains("assert_eq!(non_triggering.len(), 184);"));
}

#[test]
fn explicit_c6_membership_is_closed_at_36_and_disjoint_from_c1_controls() {
    assert_eq!(C6_RESERVED.len(), 36);
    let reserved: BTreeSet<_> = C6_RESERVED.iter().copied().collect();
    assert_eq!(reserved.len(), 36, "C6 membership must not contain duplicates");

    for name in C6_RESERVED {
        let mut chars = name.chars();
        let first = chars.next().expect("reserved word must not be empty");
        let authored = format!("\\u{:04X}{}", first as u32, chars.as_str());
        let decoded = decode_identifier(&authored).expect("C6 member must form an escaped IdentifierName");
        assert_eq!(decoded.string_value, *name, "{name}");
        assert!(reserved.contains(name));
    }

    let controls: BTreeSet<_> = C1_NON_TRIGGER_NAMES.iter().copied().collect();
    assert_eq!(controls.len(), C1_NON_TRIGGER_NAMES.len());
    assert!(reserved.is_disjoint(&controls));

    for name in C1_NON_TRIGGER_NAMES {
        let mut chars = name.chars();
        let first = chars.next().expect("control name must not be empty");
        let authored = format!("\\u{:04X}{}", first as u32, chars.as_str());
        let decoded = decode_identifier(&authored).expect("C1 control must form an escaped IdentifierName");
        assert_eq!(decoded.string_value, *name, "{name}");
        assert!(!reserved.contains(name), "{name} must remain outside C6");
    }

    assert_eq!(
        decode_identifier(r"\u{69}f")
            .expect("braced UES must retain constituent formation authority")
            .string_value,
        "if"
    );
}

#[test]
fn every_c6_candidate_preserves_exact_authored_range_and_decoded_identity() {
    let reserved: BTreeSet<_> = C6_RESERVED.iter().copied().collect();

    for (fixture_index, fixture) in FIXTURES.iter().enumerate() {
        for (candidate_index, candidate) in fixture.c6_candidates.iter().flatten().enumerate() {
            assert_authored_range(
                fixture.source,
                candidate.range,
                ISSUE_ID * 100 + (fixture_index * 2 + candidate_index) as u64,
            );
            let authored = fragment(fixture.source, candidate.range);
            assert!(authored.contains("\\u"), "{} must retain an authored escape", fixture.id);
            let decoded = decode_identifier(authored)
                .unwrap_or_else(|failure| panic!("{} candidate decode failed: {failure:?}", fixture.id));
            assert_eq!(decoded.string_value, candidate.decoded, "{}", fixture.id);
            assert!(
                reserved.contains(candidate.decoded),
                "{} candidate must decode into the explicit C6 set",
                fixture.id
            );
        }
    }
}

#[test]
fn mandatory_falsifier_matrix_is_complete_and_primary_ranges_are_authored() {
    assert_eq!(FIXTURES.len(), 15);
    let ids: BTreeSet<_> = FIXTURES.iter().map(|fixture| fixture.id).collect();
    assert_eq!(ids.len(), 15);

    for (index, fixture) in FIXTURES.iter().enumerate() {
        match fixture.primary {
            ExpectedPrimary::Static { rule_id, subject } => {
                assert!(!rule_id.is_empty());
                assert_authored_range(fixture.source, subject, ISSUE_ID * 1000 + index as u64);
            }
            ExpectedPrimary::Grammar { subject } => {
                assert_authored_range(fixture.source, subject, ISSUE_ID * 1000 + index as u64);
            }
            ExpectedPrimary::None => {}
        }
        assert_eq!(fixture.committed_relations, 0, "{}", fixture.id);
    }
}

#[test]
fn tier1_authored_order_and_lower_tier_precedence_are_closed() {
    let f3 = fixture("F3-lhs-ee04-precedes-rhs-c6");
    assert_eq!(
        f3.primary,
        ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: ByteRange::new(4, 11),
        }
    );
    assert_ne!(f3.primary, ExpectedPrimary::Static {
        rule_id: RULE_ID,
        subject: f3.c6_candidates[0].expect("F3 C6 candidate").range,
    });

    let f4 = fixture("F4-earlier-tier1-local-precedes-later-c6");
    assert_eq!(
        f4.primary,
        ExpectedPrimary::Static {
            rule_id: "EE-15-R01",
            subject: ByteRange::new(4, 7),
        }
    );

    for id in [
        "F5-earlier-c6-precedes-later-tier1-local",
        "F7-c6-precedes-tier2-block-duplicate",
        "F8-c6-precedes-tier3-script-duplicate",
        "F9-c6-precedes-tier4-lexical-var-collision",
    ] {
        let fixture = fixture(id);
        let first = fixture.c6_candidates[0].expect("precedence fixture must contain C6");
        assert_eq!(
            fixture.primary,
            ExpectedPrimary::Static {
                rule_id: RULE_ID,
                subject: first.range,
            },
            "{id}"
        );
    }

    let f6 = fixture("F6-multiple-c6-authored-order");
    let first = f6.c6_candidates[0].expect("first C6");
    let second = f6.c6_candidates[1].expect("second C6");
    assert!(first.range.start < second.range.start);
    assert_eq!(
        f6.primary,
        ExpectedPrimary::Static {
            rule_id: RULE_ID,
            subject: first.range,
        }
    );
}

#[test]
fn parser_transaction_failure_and_static_rejection_are_distinct_transactions() {
    for id in [
        "F13-later-incomplete-initializer-rolls-back-c6",
        "F14-later-definitive-grammar-rejection-outranks-tentative-c6",
        "F15-richer-expression-does-not-commit-c6-prefix",
    ] {
        let fixture = fixture(id);
        assert!(!fixture.selected_script_committed, "{id}");
        assert!(!fixture.static_rejection_committed, "{id}");
        assert_eq!(fixture.committed_relations, 0, "{id}");
        assert_eq!(fixture.terminator, None, "{id}");
    }

    let grammar = fixture("F14-later-definitive-grammar-rejection-outranks-tentative-c6");
    assert_eq!(
        grammar.primary,
        ExpectedPrimary::Grammar {
            subject: ByteRange::new(14, 18),
        }
    );
    assert_eq!(fragment(grammar.source, ByteRange::new(14, 18)), r"\u{}");

    let unsupported = fixture("F15-richer-expression-does-not-commit-c6-prefix");
    assert_eq!(unsupported.primary, ExpectedPrimary::None);
    assert_eq!(fragment(unsupported.source, ByteRange::new(6, 13)), r"\u0069f");
    assert_eq!(&unsupported.source[13..], ".foo;");
}

#[test]
fn complete_static_rejection_suppresses_all_correspondence_relations() {
    for fixture in FIXTURES
        .iter()
        .filter(|fixture| fixture.static_rejection_committed)
    {
        assert!(fixture.selected_script_committed, "{}", fixture.id);
        assert!(matches!(fixture.primary, ExpectedPrimary::Static { .. }));
        assert_eq!(fixture.committed_relations, 0, "{}", fixture.id);
    }

    let streaming_falsifier = fixture("F10-static-rejection-suppresses-earlier-relation");
    assert_eq!(streaming_falsifier.pre_rejection_relation_candidates, 1);
    assert_eq!(streaming_falsifier.committed_relations, 0);
    assert_eq!(fragment(streaming_falsifier.source, ByteRange::new(6, 14)), r"\u0066oo");
}

#[test]
fn authored_semicolon_and_eof_only_asi_both_compose_without_non_eof_widening() {
    let authored = fixture("F1-basic-c6-successor");
    let eof = fixture("F12-eof-only-asi");

    assert_eq!(authored.terminator, Some(Terminator::AuthoredSemicolon));
    assert_eq!(eof.terminator, Some(Terminator::AutomaticAtEof));
    assert!(authored.source.ends_with(';'));
    assert!(!eof.source.ends_with(';'));
    assert_eq!(eof.c6_candidates[0], Some(candidate(6, 13, "if")));
}

#[test]
fn rule_identity_closure_and_positive_lifecycle_remain_unchanged() {
    assert_eq!(CONTAINERS.len(), 37);
    assert_eq!(RULE_UNITS.len(), 193);
    assert_eq!(CURRENT_REQUIRED_RULE_IDS.len(), 9);
    assert!(CURRENT_REQUIRED_RULE_IDS.contains(&RULE_ID));

    let active = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::NormativeRule)
        .count();
    let inactive = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::EnvelopeInactiveRule)
        .count();
    assert_eq!(active, 183);
    assert_eq!(inactive, 10);
    assert_eq!(193 - CURRENT_REQUIRED_RULE_IDS.len(), 184);

    let rule = RULE_UNITS
        .iter()
        .find(|rule| rule.id == RULE_ID)
        .expect("EE-04-R08 must remain inventoried");
    assert_eq!(rule.kind, RuleUnitKind::NormativeRule);
    assert!(rule.normative_locator.contains("escaped ReservedWord StringValue rejection"));
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
}

#[test]
fn oracle_remains_candidate_independent_and_representation_neutral() {
    for forbidden_call in [
        "recognize_selected_lexical_slice(",
        "evaluate_selected_variable_statement_static_semantics(",
        "attempt_selected_qualification(",
        "analyze_selected_variable_statement_name_correspondence(",
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden_call),
            "candidate-independent oracle must not call production: {forbidden_call}"
        );
    }

    assert!(REPRESENTATION_POLICY.contains("authored evidence"));
    assert!(REPRESENTATION_POLICY.contains("production representation deferred"));
}
