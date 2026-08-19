//! Candidate-independent RHS escaped-ReservedWord validation for Issue #263.
//!
//! This test-only oracle pins future RHS reachability of the already-inventoried
//! `EE-04-R08` Early Error. It does not call production lexical, static,
//! aggregate, runtime, or generic expression code.

use crate::{SourceId, SourceText};

use super::unicode::{is_id_continue, is_id_start};

const ISSUE_ID: u64 = 263;
const RULE_ID: &str = "EE-04-R08";
const INVENTORY_SOURCE: &str = include_str!("qualification_validation_tests/inventory.rs");
const PRIOR_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_escaped_identifier_reference_initializer_validation_tests.rs"
);
const THIS_SOURCE: &str = include_str!(
    "qualification_selected_escaped_reserved_identifier_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = "Richer syntax remains outside this escaped-reserved RHS frontier and is not assigned a permanent future disposition.";

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
enum FutureOutcome {
    StaticSemanticsRejected,
    SelectedAcceptedIncomplete,
}

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
    code_points: Vec<u32>,
    string_value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RejectingFixture {
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    range: Range,
    decoded: &'static str,
}

const REJECTING: &[RejectingFixture] = &[
    RejectingFixture { id: "THIS", source: r"const x = \u0074his;", rhs: r"\u0074his", range: Range::new(10, 19), decoded: "this" },
    RejectingFixture { id: "NULL", source: r"const x = \u006Eull;", rhs: r"\u006Eull", range: Range::new(10, 19), decoded: "null" },
    RejectingFixture { id: "TRUE", source: r"const x = \u0074rue;", rhs: r"\u0074rue", range: Range::new(10, 19), decoded: "true" },
    RejectingFixture { id: "FALSE", source: r"const x = \u0066alse;", rhs: r"\u0066alse", range: Range::new(10, 20), decoded: "false" },
    RejectingFixture { id: "IF", source: r"const x = \u0069f;", rhs: r"\u0069f", range: Range::new(10, 17), decoded: "if" },
    RejectingFixture { id: "CLASS", source: r"const x = \u0063lass;", rhs: r"\u0063lass", range: Range::new(10, 20), decoded: "class" },
    RejectingFixture { id: "IMPORT", source: r"const x = \u0069mport;", rhs: r"\u0069mport", range: Range::new(10, 21), decoded: "import" },
    RejectingFixture { id: "EXPORT", source: r"const x = \u0065xport;", rhs: r"\u0065xport", range: Range::new(10, 21), decoded: "export" },
    RejectingFixture { id: "NULL-MIXED", source: r"const x = n\u0075ll;", rhs: r"n\u0075ll", range: Range::new(10, 19), decoded: "null" },
    RejectingFixture { id: "TRUE-MIXED", source: r"const x = tr\u0075e;", rhs: r"tr\u0075e", range: Range::new(10, 19), decoded: "true" },
    RejectingFixture { id: "THIS-MIXED", source: r"const x = thi\u0073;", rhs: r"thi\u0073", range: Range::new(10, 19), decoded: "this" },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NonTriggerKind {
    YieldAwait,
    StrictOnly,
    EvalArguments,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NonTriggerFixture {
    rhs: &'static str,
    decoded: &'static str,
    kind: NonTriggerKind,
}

const NON_TRIGGERS: &[NonTriggerFixture] = &[
    NonTriggerFixture { rhs: r"\u0079ield", decoded: "yield", kind: NonTriggerKind::YieldAwait },
    NonTriggerFixture { rhs: r"a\u0077ait", decoded: "await", kind: NonTriggerKind::YieldAwait },
    NonTriggerFixture { rhs: r"\u006Cet", decoded: "let", kind: NonTriggerKind::StrictOnly },
    NonTriggerFixture { rhs: r"\u0073tatic", decoded: "static", kind: NonTriggerKind::StrictOnly },
    NonTriggerFixture { rhs: r"\u0069mplements", decoded: "implements", kind: NonTriggerKind::StrictOnly },
    NonTriggerFixture { rhs: r"\u0069nterface", decoded: "interface", kind: NonTriggerKind::StrictOnly },
    NonTriggerFixture { rhs: r"\u0070ackage", decoded: "package", kind: NonTriggerKind::StrictOnly },
    NonTriggerFixture { rhs: r"\u0070rivate", decoded: "private", kind: NonTriggerKind::StrictOnly },
    NonTriggerFixture { rhs: r"\u0070rotected", decoded: "protected", kind: NonTriggerKind::StrictOnly },
    NonTriggerFixture { rhs: r"\u0070ublic", decoded: "public", kind: NonTriggerKind::StrictOnly },
    NonTriggerFixture { rhs: r"\u0065val", decoded: "eval", kind: NonTriggerKind::EvalArguments },
    NonTriggerFixture { rhs: r"\u0061rguments", decoded: "arguments", kind: NonTriggerKind::EvalArguments },
];

const RESERVED: &[&str] = &[
    "break", "case", "catch", "class", "const", "continue", "debugger", "default",
    "delete", "do", "else", "enum", "export", "extends", "false", "finally", "for",
    "function", "if", "import", "in", "instanceof", "new", "null", "return", "super",
    "switch", "this", "throw", "true", "try", "typeof", "var", "void", "while", "with",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrecedenceFamily {
    Static,
    Grammar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PrecedenceFixture {
    source: &'static str,
    family: PrecedenceFamily,
    primary_rule: Option<&'static str>,
    primary: Range,
    co_trigger: &'static str,
}

const PRECEDENCE: &[PrecedenceFixture] = &[
    PrecedenceFixture { source: r"let let = \u006Eull;", family: PrecedenceFamily::Static, primary_rule: Some("EE-15-R01"), primary: Range::new(4, 7), co_trigger: RULE_ID },
    PrecedenceFixture { source: r"let x = \u006Eull, x = foo;", family: PrecedenceFamily::Static, primary_rule: Some(RULE_ID), primary: Range::new(8, 17), co_trigger: "EE-15-R02" },
    PrecedenceFixture { source: r"const x = \u006Eull, y;", family: PrecedenceFamily::Static, primary_rule: Some(RULE_ID), primary: Range::new(10, 19), co_trigger: "EE-15-R03" },
    PrecedenceFixture { source: r"let x = foo; let x = \u006Eull;", family: PrecedenceFamily::Static, primary_rule: Some(RULE_ID), primary: Range::new(21, 30), co_trigger: "EE-36-R01" },
    PrecedenceFixture { source: r"const x = \u006Eull; let \u{};", family: PrecedenceFamily::Grammar, primary_rule: None, primary: Range::new(25, 29), co_trigger: RULE_ID },
];

fn hex(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

fn braced_code_point(digits: &[u8]) -> Result<u32, DecodeFailure> {
    if digits.is_empty() || digits.iter().any(|b| hex(*b).is_none()) {
        return Err(DecodeFailure::Malformed);
    }
    let significant = match digits.iter().position(|b| *b != b'0') {
        Some(i) => &digits[i..],
        None => return Ok(0),
    };
    if significant.len() > 6 {
        return Err(DecodeFailure::NonCodePoint);
    }
    let mut value = 0_u32;
    for b in significant {
        value = value * 16 + hex(*b).ok_or(DecodeFailure::Malformed)?;
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
        while bytes.get(end).is_some_and(|b| b.is_ascii_hexdigit()) {
            end += 1;
        }
        if bytes.get(end) != Some(&b'}') {
            return Err(DecodeFailure::Malformed);
        }
        return Ok((end + 1, braced_code_point(&bytes[digits_start..end])?));
    }
    let end = payload.checked_add(4).ok_or(DecodeFailure::Malformed)?;
    let digits = bytes.get(payload..end).ok_or(DecodeFailure::Malformed)?;
    if !digits.iter().all(|b| b.is_ascii_hexdigit()) {
        return Err(DecodeFailure::Malformed);
    }
    let mut value = 0_u32;
    for b in digits {
        value = value * 16 + hex(*b).ok_or(DecodeFailure::Malformed)?;
    }
    Ok((end, value))
}

fn valid_start(cp: u32) -> bool {
    cp == u32::from(b'$') || cp == u32::from(b'_') || is_id_start(cp)
}

fn valid_part(cp: u32) -> bool {
    valid_start(cp) || is_id_continue(cp) || matches!(cp, 0x200C | 0x200D)
}

fn decode(text: &str) -> Result<Decoded, DecodeFailure> {
    let mut offset = 0;
    let mut index = 0;
    let mut saw_escape = false;
    let mut code_points = Vec::new();
    while offset < text.len() {
        let (cp, end, escaped) = if text.as_bytes().get(offset) == Some(&b'\\') {
            let (end, cp) = escape_at(text, offset)?;
            (cp, end, true)
        } else {
            let ch = text[offset..].chars().next().ok_or(DecodeFailure::Malformed)?;
            (ch as u32, offset + ch.len_utf8(), false)
        };
        let valid = if index == 0 { valid_start(cp) } else { valid_part(cp) };
        if !valid {
            return Err(if index == 0 { DecodeFailure::InvalidStart } else { DecodeFailure::InvalidPart });
        }
        code_points.push(cp);
        saw_escape |= escaped;
        offset = end;
        index += 1;
    }
    if !saw_escape {
        return Err(DecodeFailure::MissingEscape);
    }
    let mut string_value = String::new();
    for cp in &code_points {
        string_value.push(char::from_u32(*cp).expect("position-valid fixture code point must be scalar"));
    }
    Ok(Decoded { code_points, string_value })
}

fn is_reserved(name: &str) -> bool {
    RESERVED.contains(&name)
}

fn fragment(source: &str, range: Range) -> &str {
    source.get(range.start..range.end).expect("fixture range must be valid UTF-8")
}

fn anchor(source: &str, range: Range, id: u64) -> String {
    let source = SourceText::new(SourceId::new(id), source.to_owned());
    source.anchor(range.start, range.end).expect("authored range must anchor").fragment().to_owned()
}

fn active_rule(rule_id: &str) -> &'static str {
    let marker = format!("active_rule(\n        \"{rule_id}\",");
    let start = INVENTORY_SOURCE.find(&marker).unwrap_or_else(|| panic!("missing {rule_id}"));
    let rest = &INVENTORY_SOURCE[start..];
    let end = rest.find("\n    ),").expect("rule block must close") + "\n    ),".len();
    &rest[..end]
}

#[test]
fn authority_and_existing_ee_04_r08_identity_are_exact() {
    assert_eq!(ISSUE_ID, 263);
    let rule = active_rule(RULE_ID);
    assert!(rule.contains("Identifier : IdentifierName but not ReservedWord"));
    assert!(rule.contains("escaped ReservedWord StringValue rejection"));
    assert!(rule.contains("JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001"));
    assert_eq!(RESERVED.len(), 36);
    assert_eq!(REJECTING.len(), 11);
    assert_eq!(NON_TRIGGERS.len(), 12);
    assert_eq!(PRECEDENCE.len(), 5);
}

#[test]
fn escaped_reserved_rhs_pins_exact_authored_subject_decoding_and_static_outcome() {
    for (i, fixture) in REJECTING.iter().enumerate() {
        assert_eq!(fragment(fixture.source, fixture.range), fixture.rhs, "{}", fixture.id);
        let decoded = decode(fixture.rhs).unwrap_or_else(|e| panic!("{}: {e:?}", fixture.id));
        assert_eq!(decoded.string_value, fixture.decoded, "{}", fixture.id);
        assert_eq!(
            decoded.code_points,
            fixture.decoded.chars().map(|ch| ch as u32).collect::<Vec<_>>(),
            "{}",
            fixture.id
        );
        assert!(is_reserved(&decoded.string_value), "{}", fixture.id);
        assert_eq!(anchor(fixture.source, fixture.range, 263_000 + i as u64), fixture.rhs);
        assert_eq!(FutureOutcome::StaticSemanticsRejected, FutureOutcome::StaticSemanticsRejected);
    }
}

#[test]
fn yield_await_strict_only_and_eval_arguments_are_non_triggers_in_this_envelope() {
    let mut counts = [0_usize; 3];
    for fixture in NON_TRIGGERS {
        let decoded = decode(fixture.rhs).expect("non-trigger must decode");
        assert_eq!(decoded.string_value, fixture.decoded);
        assert!(!is_reserved(fixture.decoded));
        match fixture.kind {
            NonTriggerKind::YieldAwait => counts[0] += 1,
            NonTriggerKind::StrictOnly => counts[1] += 1,
            NonTriggerKind::EvalArguments => counts[2] += 1,
        }
    }
    assert_eq!(counts, [2, 8, 2]);
    assert_eq!(FutureOutcome::SelectedAcceptedIncomplete, FutureOutcome::SelectedAcceptedIncomplete);
}

#[test]
fn direct_this_null_boolean_spellings_remain_separate_existing_owners() {
    for (i, (source, rhs, range)) in [
        ("const x = this;", "this", Range::new(10, 14)),
        ("const x = null;", "null", Range::new(10, 14)),
        ("const x = true;", "true", Range::new(10, 14)),
        ("const x = false;", "false", Range::new(10, 15)),
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(fragment(source, *range), *rhs);
        assert!(is_reserved(rhs));
        assert_eq!(decode(rhs), Err(DecodeFailure::MissingEscape));
        assert_eq!(anchor(source, *range, 263_100 + i as u64), *rhs);
    }
}

#[test]
fn malformed_non_code_point_and_position_invalid_families_are_not_reclassified() {
    for (rhs, expected) in [
        (r"\u{}", DecodeFailure::Malformed),
        (r"\u0", DecodeFailure::Malformed),
        (r"\u{110000}", DecodeFailure::NonCodePoint),
        (r"\u0030", DecodeFailure::InvalidStart),
        (r"a\u002D", DecodeFailure::InvalidPart),
        (r"\uD800", DecodeFailure::InvalidStart),
    ] {
        assert_eq!(decode(rhs), Err(expected), "{rhs}");
    }
}

#[test]
fn richer_tail_controls_remain_frontier_scoped_without_prefix_verdict() {
    assert!(FRONTIER_SCOPE_NOTE.contains("Richer syntax remains outside"));
    assert!(FRONTIER_SCOPE_NOTE.contains("not assigned a permanent future disposition"));
    for source in [
        r"const x = \u006Eull.x;",
        r"const x = \u006Eull();",
        r"const x = \u006Eull + 1;",
        r"const x = \u006Eull ? a : b;",
        r"const x = \u006Eull/*c*/;",
    ] {
        assert_eq!(fragment(source, Range::new(10, 19)), r"\u006Eull");
        assert_eq!(decode(r"\u006Eull").expect("prefix must decode").string_value, "null");
        assert!(source.len() > 19);
    }
}

#[test]
fn static_and_grammar_precedence_witnesses_are_explicit() {
    assert_eq!(PRECEDENCE[0].primary_rule, Some("EE-15-R01"));
    assert_eq!(PRECEDENCE[1].primary_rule, Some(RULE_ID));
    assert_eq!(PRECEDENCE[2].primary_rule, Some(RULE_ID));
    assert_eq!(PRECEDENCE[3].primary_rule, Some(RULE_ID));
    assert_eq!(PRECEDENCE[4].family, PrecedenceFamily::Grammar);
    assert_eq!(PRECEDENCE[4].primary_rule, None);
    for (i, fixture) in PRECEDENCE.iter().enumerate() {
        let subject = fragment(fixture.source, fixture.primary);
        assert!(!subject.is_empty());
        let _ = active_rule(fixture.co_trigger);
        if let Some(rule) = fixture.primary_rule {
            let _ = active_rule(rule);
        }
        assert_eq!(anchor(fixture.source, fixture.primary, 263_200 + i as u64), subject);
    }
    assert_eq!(fragment(PRECEDENCE[4].source, PRECEDENCE[4].primary), r"\u{}");
}

#[test]
fn corrected_prior_oracle_remains_historical_and_frontier_scoped() {
    assert!(PRIOR_ORACLE_SOURCE.contains("escaped-IdentifierReference frontier"));
    assert!(PRIOR_ORACLE_SOURCE.contains("later independently qualified owners"));
    assert!(!PRIOR_ORACLE_SOURCE.contains(concat!("FutureUnsupported", "Disposition")));
}

#[test]
fn source_is_candidate_independent_and_representation_neutral() {
    for forbidden in [
        ["selected_", "lexical_slice"].concat(),
        ["consume_selected_", "identifier_reference"].concat(),
        ["selected_", "static_semantics"].concat(),
        ["selected_qualification_", "integration"].concat(),
        ["Resolve", "Binding"].concat(),
        ["QualificationOutcome", "::"].concat(),
        ["Initializer", "Kind"].concat(),
        ["Expression", "Node"].concat(),
    ] {
        assert!(!THIS_SOURCE.contains(&forbidden), "forbidden dependency {forbidden}");
    }
}
