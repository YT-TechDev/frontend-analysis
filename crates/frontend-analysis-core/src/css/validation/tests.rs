use std::collections::{BTreeMap, BTreeSet};

use super::fixtures::{resource_limit_fixtures, specification_fixtures};
use super::generated::{
    GENERATED_INPUT_COUNT, GENERATED_MAX_BYTES, GENERATOR_REVISION, GENERATOR_SEED,
    alphabet_contains_required_boundaries, assert_deterministic_replay, generated_inputs,
};
use super::gold::{
    GoldCompletion, GoldDiagnosticCode, GoldDiagnosticSubject, GoldFixture, GoldGroup,
    GoldHashType, GoldLexicalItem, GoldNumber, GoldNumberType, GoldResourceKind, GoldSign,
    GoldTermination, GoldTokenKind, validate_fixture,
};

fn fixture<'a>(fixtures: &'a [GoldFixture], id: &str) -> &'a GoldFixture {
    fixtures
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing gold fixture {id}"))
}

#[test]
fn specification_fixture_inventory_is_self_consistent_and_unique() {
    let fixtures = specification_fixtures();
    assert_eq!(fixtures.len(), 33);

    let mut ids = BTreeSet::new();
    for fixture in &fixtures {
        assert!(
            ids.insert(fixture.id),
            "duplicate fixture id {}",
            fixture.id
        );
        validate_fixture(fixture).unwrap_or_else(|error| {
            panic!("fixture {} failed self-validation: {error:?}", fixture.id)
        });
    }
}

#[test]
fn fixture_groups_cover_the_approved_first_capability_matrix() {
    let fixtures = specification_fixtures();
    let mut counts = BTreeMap::new();
    for fixture in &fixtures {
        *counts.entry(fixture.group).or_insert(0usize) += 1;
    }

    assert_eq!(counts[&GoldGroup::InputPreprocessing], 11);
    assert_eq!(counts[&GoldGroup::Comments], 3);
    assert_eq!(counts[&GoldGroup::IdentEscape], 4);
    assert_eq!(counts[&GoldGroup::Strings], 4);
    assert_eq!(counts[&GoldGroup::Urls], 6);
    assert_eq!(counts[&GoldGroup::Numeric], 1);
    assert_eq!(counts[&GoldGroup::Fixed], 1);
    assert_eq!(counts[&GoldGroup::Historical], 2);
    assert_eq!(counts[&GoldGroup::Lifecycle], 1);
}

#[test]
fn every_first_slice_token_class_has_candidate_independent_gold() {
    let fixtures = specification_fixtures();
    let mut actual = BTreeSet::new();
    let mut semantic_checksum = 0usize;

    for fixture in &fixtures {
        if fixture.group == GoldGroup::Lifecycle {
            continue;
        }
        for item in &fixture.items {
            if let GoldLexicalItem::Token { kind, .. } = item {
                actual.insert(kind.class_name());
                semantic_checksum = semantic_checksum.wrapping_add(token_semantic_signature(kind));
            }
        }
    }

    let expected = BTreeSet::from([
        "ident",
        "function",
        "at-keyword",
        "hash",
        "string",
        "bad-string",
        "url",
        "bad-url",
        "delim",
        "number",
        "percentage",
        "dimension",
        "whitespace",
        "cdo",
        "cdc",
        "colon",
        "semicolon",
        "comma",
        "left-square-bracket",
        "right-square-bracket",
        "left-parenthesis",
        "right-parenthesis",
        "left-curly-bracket",
        "right-curly-bracket",
    ]);

    assert_eq!(actual, expected);
    assert_ne!(semantic_checksum, 0);
}

#[test]
fn every_first_slice_diagnostic_code_has_normative_gold() {
    let fixtures = specification_fixtures();
    let actual: BTreeSet<_> = fixtures
        .iter()
        .filter(|fixture| fixture.group != GoldGroup::Lifecycle)
        .flat_map(|fixture| fixture.diagnostics.iter().map(|diagnostic| diagnostic.code))
        .collect();

    let expected = BTreeSet::from([
        GoldDiagnosticCode::InvalidEscape,
        GoldDiagnosticCode::EofInComment,
        GoldDiagnosticCode::EofInString,
        GoldDiagnosticCode::NewlineInString,
        GoldDiagnosticCode::EofInUrl,
        GoldDiagnosticCode::InvalidUrlCodePoint,
        GoldDiagnosticCode::InvalidUrlEscape,
        GoldDiagnosticCode::EofInEscape,
    ]);

    assert_eq!(actual, expected);
}

#[test]
fn preprocessing_matrix_preserves_raw_bytes_and_explicit_semantic_units() {
    let fixtures = specification_fixtures();

    let bom_only = fixture(&fixtures, "CSS-INPUT-BOM-ONLY-001");
    assert_eq!(bom_only.source.as_bytes(), &[0xef, 0xbb, 0xbf]);
    assert_eq!(bom_only.leading_bom.unwrap().start, 0);
    assert_eq!(bom_only.leading_bom.unwrap().end, 3);
    assert!(bom_only.preprocessed_units.is_empty());

    let crlf = fixture(&fixtures, "CSS-INPUT-CRLF-001");
    assert_eq!(crlf.source.as_bytes(), b"\r\n");
    assert_eq!(crlf.preprocessed_units.len(), 1);
    assert_eq!(crlf.preprocessed_units[0].semantic, '\n');
    assert_eq!(crlf.preprocessed_units[0].raw.start, 0);
    assert_eq!(crlf.preprocessed_units[0].raw.end, 2);

    let mixed = fixture(&fixtures, "CSS-INPUT-MIXED-WHITESPACE-001");
    assert_eq!(mixed.source.as_bytes(), b"\n\r\r\n\x0c ");
    assert_eq!(
        mixed
            .preprocessed_units
            .iter()
            .map(|unit| unit.semantic)
            .collect::<Vec<_>>(),
        vec!['\n', '\n', '\n', '\n', ' ']
    );

    let nul = fixture(&fixtures, "CSS-INPUT-NUL-MULTIBYTE-001");
    assert_eq!(nul.source.as_bytes(), &[0x00, 0xc3, 0xa9, b':', b'a']);
    assert_eq!(nul.preprocessed_units[0].semantic, '�');
    assert_eq!(nul.preprocessed_units[0].raw.start, 0);
    assert_eq!(nul.preprocessed_units[0].raw.end, 1);
    assert_eq!(nul.preprocessed_units[1].semantic, 'é');
    assert_eq!(nul.preprocessed_units[1].raw.start, 1);
    assert_eq!(nul.preprocessed_units[1].raw.end, 3);
}

#[test]
fn lone_backslash_eof_and_backslash_newline_are_distinct_gold_paths() {
    let fixtures = specification_fixtures();
    let eof = fixture(&fixtures, "CSS-IDENT-EOF-ESCAPE-001");
    let newline = fixture(&fixtures, "CSS-IDENT-INVALID-ESCAPE-NEWLINE-001");

    assert_eq!(eof.source.as_bytes(), b"\\");
    assert_eq!(eof.diagnostics.len(), 1);
    assert_eq!(eof.diagnostics[0].code, GoldDiagnosticCode::EofInEscape);
    assert!(matches!(
        &eof.items[0],
        GoldLexicalItem::Token {
            kind: GoldTokenKind::Ident("�"),
            ..
        }
    ));

    assert_eq!(newline.source.as_bytes(), b"\\\n");
    assert_eq!(newline.diagnostics.len(), 1);
    assert_eq!(
        newline.diagnostics[0].code,
        GoldDiagnosticCode::InvalidEscape
    );
    assert!(matches!(
        &newline.items[0],
        GoldLexicalItem::Token {
            kind: GoldTokenKind::Delim('\\'),
            ..
        }
    ));
    assert!(matches!(
        &newline.items[1],
        GoldLexicalItem::Token {
            kind: GoldTokenKind::Whitespace,
            ..
        }
    ));
}

#[test]
fn string_backslash_eof_is_eof_in_string_not_eof_in_escape() {
    let fixtures = specification_fixtures();
    let fixture = fixture(&fixtures, "CSS-STRING-BACKSLASH-EOF-001");
    assert_eq!(fixture.source.as_bytes(), b"\"a\\");
    assert_eq!(fixture.diagnostics.len(), 1);
    assert_eq!(fixture.diagnostics[0].code, GoldDiagnosticCode::EofInString);
}

#[test]
fn url_eof_escape_preserves_equal_offset_producer_order() {
    let fixtures = specification_fixtures();
    let fixture = fixture(&fixtures, "CSS-URL-EOF-ESCAPE-001");
    assert_eq!(fixture.diagnostics.len(), 2);
    assert_eq!(fixture.diagnostics[0].location.start, 6);
    assert_eq!(fixture.diagnostics[1].location.start, 6);
    assert_eq!(fixture.diagnostics[0].code, GoldDiagnosticCode::EofInEscape);
    assert_eq!(fixture.diagnostics[1].code, GoldDiagnosticCode::EofInUrl);
}

#[test]
fn mandatory_historical_regressions_preserve_exact_lexical_boundaries() {
    let fixtures = specification_fixtures();
    let provenance = fixture(&fixtures, "CSS-HIST-PROVENANCE-001");
    assert_eq!(provenance.source.as_bytes(), br"a{c\6F lor/**/:red;}");
    assert_eq!(
        provenance
            .items
            .iter()
            .map(GoldLexicalItem::range)
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        vec![
            (0, 1),
            (1, 2),
            (2, 10),
            (10, 14),
            (14, 15),
            (15, 18),
            (18, 19),
            (19, 20),
        ]
    );

    let priority = fixture(&fixtures, "CSS-HIST-PRIORITY-001");
    assert_eq!(
        priority.source.as_bytes(),
        br"a{color:red !/**/Im\70 ortant;}"
    );
    assert_eq!(
        priority
            .items
            .iter()
            .map(GoldLexicalItem::range)
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        vec![
            (0, 1),
            (1, 2),
            (2, 7),
            (7, 8),
            (8, 11),
            (11, 12),
            (12, 13),
            (13, 17),
            (17, 29),
            (29, 30),
            (30, 31),
        ]
    );
}

#[test]
fn resource_lifecycle_foundation_covers_every_approved_resource_kind() {
    let fixtures = resource_limit_fixtures();
    assert_eq!(fixtures.len(), 6);

    let mut ids = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    for fixture in &fixtures {
        assert!(
            ids.insert(fixture.id),
            "duplicate resource fixture id {}",
            fixture.id
        );
        validate_fixture(fixture).unwrap();
        assert_eq!(fixture.completion, GoldCompletion::Incomplete);
        match fixture.termination {
            GoldTermination::ResourceLimit {
                kind,
                limit,
                attempted,
                terminal,
            } => {
                kinds.insert(kind);
                assert_eq!(limit, 0);
                assert_eq!(attempted, 1);
                assert_eq!(terminal.start, 0);
                assert_eq!(terminal.end, 0);
            }
            GoldTermination::EndOfInput => panic!("expected resource limit"),
        }
    }

    assert_eq!(
        kinds,
        BTreeSet::from([
            GoldResourceKind::SourceBytes,
            GoldResourceKind::AlgorithmSteps,
            GoldResourceKind::LexicalItems,
            GoldResourceKind::Diagnostics,
            GoldResourceKind::RetainedInterpretedBytes,
            GoldResourceKind::TemporaryBufferBytes,
        ])
    );
}

#[test]
fn diagnostic_subject_model_includes_input_location_without_making_it_an_oracle() {
    assert_eq!(
        GoldDiagnosticSubject::InputLocation,
        GoldDiagnosticSubject::InputLocation
    );
}

#[test]
fn deterministic_generated_corpus_has_fixed_revision_count_and_bound() {
    assert_eq!(GENERATOR_REVISION, "css-tokenizer-valid-utf8-lcg-v1");
    assert_eq!(GENERATOR_SEED, 0x4353_532d_3133_3521);
    assert!(alphabet_contains_required_boundaries());

    let first = generated_inputs();
    let second = generated_inputs();
    assert_eq!(first, second);
    assert_eq!(first.len(), GENERATED_INPUT_COUNT);
    assert!(
        first
            .iter()
            .all(|source| source.len() <= GENERATED_MAX_BYTES)
    );
    assert!(first.iter().any(|source| source.contains('\0')));
    assert!(first.iter().any(|source| source.contains('\\')));
    assert!(first.iter().any(|source| source.contains('é')));
    assert!(first.iter().any(|source| source.contains('😀')));
}

#[test]
fn generated_replay_harness_compares_canonical_observations_twice() {
    let inputs = generated_inputs();
    assert_deterministic_replay(&inputs, |source| {
        (
            source.len(),
            source.chars().count(),
            source.as_bytes().iter().fold(0u64, |state, byte| {
                state.wrapping_mul(131).wrapping_add(*byte as u64)
            }),
        )
    });
}

fn token_semantic_signature(kind: &GoldTokenKind) -> usize {
    match kind {
        GoldTokenKind::Ident(value)
        | GoldTokenKind::Function(value)
        | GoldTokenKind::AtKeyword(value)
        | GoldTokenKind::String(value)
        | GoldTokenKind::Url(value) => value.len(),
        GoldTokenKind::Hash { value, hash_type } => {
            value.len()
                + match hash_type {
                    GoldHashType::Id => 1,
                    GoldHashType::Unrestricted => 2,
                }
        }
        GoldTokenKind::BadString | GoldTokenKind::BadUrl => 3,
        GoldTokenKind::Delim(value) => *value as usize,
        GoldTokenKind::Number(number) | GoldTokenKind::Percentage(number) => {
            number_semantic_signature(number)
        }
        GoldTokenKind::Dimension { number, unit } => {
            number_semantic_signature(number).wrapping_add(unit.len())
        }
        GoldTokenKind::Whitespace => 5,
        GoldTokenKind::Cdo => 6,
        GoldTokenKind::Cdc => 7,
        GoldTokenKind::Colon => 8,
        GoldTokenKind::Semicolon => 9,
        GoldTokenKind::Comma => 10,
        GoldTokenKind::LeftSquareBracket => 11,
        GoldTokenKind::RightSquareBracket => 12,
        GoldTokenKind::LeftParenthesis => 13,
        GoldTokenKind::RightParenthesis => 14,
        GoldTokenKind::LeftCurlyBracket => 15,
        GoldTokenKind::RightCurlyBracket => 16,
    }
}

fn number_semantic_signature(number: &GoldNumber) -> usize {
    let sign = match number.sign {
        Some(GoldSign::Plus) => 1,
        Some(GoldSign::Minus) => 2,
        None => 0,
    };
    let exponent_sign = match number.exponent_sign {
        Some(GoldSign::Plus) => 3,
        Some(GoldSign::Minus) => 5,
        None => 0,
    };
    let number_type = match number.number_type {
        GoldNumberType::Integer => 7,
        GoldNumberType::Number => 11,
    };

    sign + number.integer_digits.len()
        + number.fraction_digits.len()
        + exponent_sign
        + number.exponent_digits.map_or(0, str::len)
        + number_type
}
