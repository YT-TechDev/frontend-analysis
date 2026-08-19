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
enum ExpectedOutcome {
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
    code_points: &'static [u32],
    string_value: &'static str,
    rule_id: &'static str,
    expected: ExpectedOutcome,
}

const fn rejecting_fixture(
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    range: Range,
    code_points: &'static [u32],
    string_value: &'static str,
) -> RejectingFixture {
    RejectingFixture {
        id,
        source,
        rhs,
        range,
        code_points,
        string_value,
        rule_id: RULE_ID,
        expected: ExpectedOutcome::StaticSemanticsRejected,
    }
}

const REJECTING: &[RejectingFixture] = &[
    rejecting_fixture(
        "THIS",
        r"const x = \u0074his;",
        r"\u0074his",
        Range::new(10, 19),
        &[0x74, 0x68, 0x69, 0x73],
        "this",
    ),
    rejecting_fixture(
        "NULL",
        r"const x = \u006Eull;",
        r"\u006Eull",
        Range::new(10, 19),
        &[0x6E, 0x75, 0x6C, 0x6C],
        "null",
    ),
    rejecting_fixture(
        "TRUE",
        r"const x = \u0074rue;",
        r"\u0074rue",
        Range::new(10, 19),
        &[0x74, 0x72, 0x75, 0x65],
        "true",
    ),
    rejecting_fixture(
        "FALSE",
        r"const x = \u0066alse;",
        r"\u0066alse",
        Range::new(10, 20),
        &[0x66, 0x61, 0x6C, 0x73, 0x65],
        "false",
    ),
    rejecting_fixture(
        "IF",
        r"const x = \u0069f;",
        r"\u0069f",
        Range::new(10, 17),
        &[0x69, 0x66],
        "if",
    ),
    rejecting_fixture(
        "CLASS",
        r"const x = \u0063lass;",
        r"\u0063lass",
        Range::new(10, 20),
        &[0x63, 0x6C, 0x61, 0x73, 0x73],
        "class",
    ),
    rejecting_fixture(
        "IMPORT",
        r"const x = \u0069mport;",
        r"\u0069mport",
        Range::new(10, 21),
        &[0x69, 0x6D, 0x70, 0x6F, 0x72, 0x74],
        "import",
    ),
    rejecting_fixture(
        "EXPORT",
        r"const x = \u0065xport;",
        r"\u0065xport",
        Range::new(10, 21),
        &[0x65, 0x78, 0x70, 0x6F, 0x72, 0x74],
        "export",
    ),
    rejecting_fixture(
        "NULL-MIXED",
        r"const x = n\u0075ll;",
        r"n\u0075ll",
        Range::new(10, 19),
        &[0x6E, 0x75, 0x6C, 0x6C],
        "null",
    ),
    rejecting_fixture(
        "TRUE-MIXED",
        r"const x = tr\u0075e;",
        r"tr\u0075e",
        Range::new(10, 19),
        &[0x74, 0x72, 0x75, 0x65],
        "true",
    ),
    rejecting_fixture(
        "THIS-MIXED",
        r"const x = thi\u0073;",
        r"thi\u0073",
        Range::new(10, 19),
        &[0x74, 0x68, 0x69, 0x73],
        "this",
    ),
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
    string_value: &'static str,
    kind: NonTriggerKind,
    expected: ExpectedOutcome,
}

const fn non_trigger_fixture(
    rhs: &'static str,
    string_value: &'static str,
    kind: NonTriggerKind,
) -> NonTriggerFixture {
    NonTriggerFixture {
        rhs,
        string_value,
        kind,
        expected: ExpectedOutcome::SelectedAcceptedIncomplete,
    }
}

const NON_TRIGGERS: &[NonTriggerFixture] = &[
    non_trigger_fixture(r"\u0079ield", "yield", NonTriggerKind::YieldAwait),
    non_trigger_fixture(r"a\u0077ait", "await", NonTriggerKind::YieldAwait),
    non_trigger_fixture(r"\u006Cet", "let", NonTriggerKind::StrictOnly),
    non_trigger_fixture(r"\u0073tatic", "static", NonTriggerKind::StrictOnly),
    non_trigger_fixture(r"\u0069mplements", "implements", NonTriggerKind::StrictOnly),
    non_trigger_fixture(r"\u0069nterface", "interface", NonTriggerKind::StrictOnly),
    non_trigger_fixture(r"\u0070ackage", "package", NonTriggerKind::StrictOnly),
    non_trigger_fixture(r"\u0070rivate", "private", NonTriggerKind::StrictOnly),
    non_trigger_fixture(r"\u0070rotected", "protected", NonTriggerKind::StrictOnly),
    non_trigger_fixture(r"\u0070ublic", "public", NonTriggerKind::StrictOnly),
    non_trigger_fixture(r"\u0065val", "eval", NonTriggerKind::EvalArguments),
    non_trigger_fixture(
        r"\u0061rguments",
        "arguments",
        NonTriggerKind::EvalArguments,
    ),
];

const RESERVED: &[&str] = &[
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
    PrecedenceFixture {
        source: r"let let = \u006Eull;",
        family: PrecedenceFamily::Static,
        primary_rule: Some("EE-15-R01"),
        primary: Range::new(4, 7),
        co_trigger: RULE_ID,
    },
    PrecedenceFixture {
        source: r"let x = \u006Eull, x = foo;",
        family: PrecedenceFamily::Static,
        primary_rule: Some(RULE_ID),
        primary: Range::new(8, 17),
        co_trigger: "EE-15-R02",
    },
    PrecedenceFixture {
        source: r"const x = \u006Eull, y;",
        family: PrecedenceFamily::Static,
        primary_rule: Some(RULE_ID),
        primary: Range::new(10, 19),
        co_trigger: "EE-15-R03",
    },
    PrecedenceFixture {
        source: r"let x = foo; let x = \u006Eull;",
        family: PrecedenceFamily::Static,
        primary_rule: Some(RULE_ID),
        primary: Range::new(21, 30),
        co_trigger: "EE-36-R01",
    },
    PrecedenceFixture {
        source: r"const x = \u006Eull; let \u{};",
        family: PrecedenceFamily::Grammar,
        primary_rule: None,
        primary: Range::new(25, 29),
        co_trigger: RULE_ID,
    },
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
    code_point == u32::from(b'$') || code_point == u32::from(b'_') || is_id_start(code_point)
}

fn valid_part(code_point: u32) -> bool {
    valid_start(code_point) || is_id_continue(code_point) || matches!(code_point, 0x200C | 0x200D)
}

fn decode(text: &str) -> Result<Decoded, DecodeFailure> {
    let mut offset = 0;
    let mut index = 0;
    let mut saw_escape = false;
    let mut code_points = Vec::new();

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

        code_points.push(code_point);
        saw_escape |= escaped;
        offset = end;
        index += 1;
    }

    if !saw_escape {
        return Err(DecodeFailure::MissingEscape);
    }

    let mut string_value = String::new();
    for code_point in &code_points {
        string_value.push(
            char::from_u32(*code_point)
                .expect("position-valid fixture code point must be a Unicode scalar value"),
        );
    }

    Ok(Decoded {
        code_points,
        string_value,
    })
}

fn is_reserved(name: &str) -> bool {
    RESERVED.contains(&name)
}

fn fragment(source: &str, range: Range) -> &str {
    source
        .get(range.start..range.end)
        .expect("fixture range must be valid UTF-8")
}

fn anchor(source: &str, range: Range, id: u64) -> String {
    let source = SourceText::new(SourceId::new(id), source.to_owned());
    source
        .anchor(range.start, range.end)
        .expect("authored range must anchor")
        .fragment()
        .to_owned()
}

fn active_rule(rule_id: &str) -> &'static str {
    let marker = format!("active_rule(\n        \"{rule_id}\",");
    let start = INVENTORY_SOURCE
        .find(&marker)
        .unwrap_or_else(|| panic!("missing active rule {rule_id}"));
    let rest = &INVENTORY_SOURCE[start..];
    let end = rest.find("\n    ),").expect("active rule block must close") + "\n    ),".len();
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
fn escaped_reserved_rhs_pins_authored_subject_decoding_and_static_outcome() {
    for (index, fixture) in REJECTING.iter().enumerate() {
        assert_eq!(
            fragment(fixture.source, fixture.range),
            fixture.rhs,
            "{}",
            fixture.id
        );
        let decoded = decode(fixture.rhs)
            .unwrap_or_else(|failure| panic!("{} must decode: {failure:?}", fixture.id));
        assert_eq!(
            decoded.code_points.as_slice(),
            fixture.code_points,
            "{}",
            fixture.id
        );
        assert_eq!(decoded.string_value, fixture.string_value, "{}", fixture.id);
        assert!(is_reserved(&decoded.string_value), "{}", fixture.id);
        assert_eq!(fixture.rule_id, RULE_ID, "{}", fixture.id);
        assert_eq!(
            fixture.expected,
            ExpectedOutcome::StaticSemanticsRejected,
            "{}",
            fixture.id
        );
        assert_eq!(
            anchor(fixture.source, fixture.range, 263_000 + index as u64),
            fixture.rhs
        );
    }
}

#[test]
fn yield_await_strict_only_and_eval_arguments_are_non_triggers_in_this_envelope() {
    let mut counts = [0_usize; 3];
    for fixture in NON_TRIGGERS {
        let decoded = decode(fixture.rhs).expect("non-trigger fixture must decode");
        assert_eq!(decoded.string_value, fixture.string_value);
        assert!(!is_reserved(fixture.string_value));
        assert_eq!(
            fixture.expected,
            ExpectedOutcome::SelectedAcceptedIncomplete
        );
        match fixture.kind {
            NonTriggerKind::YieldAwait => counts[0] += 1,
            NonTriggerKind::StrictOnly => counts[1] += 1,
            NonTriggerKind::EvalArguments => counts[2] += 1,
        }
    }
    assert_eq!(counts, [2, 8, 2]);
}

#[test]
fn direct_this_null_boolean_spellings_remain_separate_existing_owners() {
    let fixtures = [
        ("const x = this;", "this", Range::new(10, 14)),
        ("const x = null;", "null", Range::new(10, 14)),
        ("const x = true;", "true", Range::new(10, 14)),
        ("const x = false;", "false", Range::new(10, 15)),
    ];

    for (index, (source, rhs, range)) in fixtures.iter().enumerate() {
        assert_eq!(fragment(source, *range), *rhs);
        assert!(is_reserved(rhs));
        assert_eq!(decode(rhs), Err(DecodeFailure::MissingEscape));
        assert_eq!(anchor(source, *range, 263_100 + index as u64), *rhs);
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
        assert_eq!(
            decode(r"\u006Eull")
                .expect("reserved prefix must decode")
                .string_value,
            "null"
        );
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

    for (index, fixture) in PRECEDENCE.iter().enumerate() {
        let subject = fragment(fixture.source, fixture.primary);
        assert!(!subject.is_empty());
        let _ = active_rule(fixture.co_trigger);
        if let Some(rule_id) = fixture.primary_rule {
            let _ = active_rule(rule_id);
        }
        assert_eq!(
            anchor(fixture.source, fixture.primary, 263_200 + index as u64),
            subject
        );
    }

    assert_eq!(
        fragment(PRECEDENCE[4].source, PRECEDENCE[4].primary),
        r"\u{}"
    );
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
        assert!(
            !THIS_SOURCE.contains(&forbidden),
            "forbidden dependency {forbidden}"
        );
    }
}
