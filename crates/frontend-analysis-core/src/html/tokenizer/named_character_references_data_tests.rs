use super::named_character_references_generated::{
    NAMED_CHARACTER_REFERENCE_ENTRY_COUNT, NAMED_CHARACTER_REFERENCE_MAXIMUM_NAME_BYTE_LENGTH,
    NAMED_CHARACTER_REFERENCE_MAXIMUM_NAMES, NAMED_CHARACTER_REFERENCE_SEMICOLONLESS_ENTRY_COUNT,
    NAMED_CHARACTER_REFERENCE_TWO_SCALAR_ENTRY_COUNT, NAMED_CHARACTER_REFERENCES,
    RETAINED_ENTITIES_SHA256, UPSTREAM_MANIFEST_SHA256, WHATWG_HTML_SNAPSHOT,
};

fn lookup(name: &str) -> Option<&'static str> {
    NAMED_CHARACTER_REFERENCES
        .binary_search_by(|(candidate, _)| candidate.cmp(&name))
        .ok()
        .map(|index| NAMED_CHARACTER_REFERENCES[index].1)
}

#[test]
fn authority_metadata_is_frozen() {
    assert_eq!(
        WHATWG_HTML_SNAPSHOT,
        "8ad51e24e9d9e48d92317467f434f7192df9d63d"
    );
    assert_eq!(
        RETAINED_ENTITIES_SHA256,
        "d741d877ac77c4194c4ad526b5b4a19aef8dfe411ab840a466891cdbb9f362e6"
    );
    assert_eq!(
        UPSTREAM_MANIFEST_SHA256,
        "234ff9717d9189382699bbc26105671d4d62240377a514a15e28e716d747e908"
    );
}

#[test]
fn complete_table_is_sorted_unique_and_exactly_counted() {
    assert_eq!(NAMED_CHARACTER_REFERENCE_ENTRY_COUNT, 2_231);
    assert_eq!(
        NAMED_CHARACTER_REFERENCES.len(),
        NAMED_CHARACTER_REFERENCE_ENTRY_COUNT
    );
    for adjacent in NAMED_CHARACTER_REFERENCES.windows(2) {
        assert!(
            adjacent[0].0 < adjacent[1].0,
            "generated names must be strictly sorted and unique: {:?}",
            adjacent
        );
    }
}

#[test]
fn generated_key_shape_is_exact() {
    for (name, value) in NAMED_CHARACTER_REFERENCES {
        assert!(!name.is_empty());
        assert!(!name.starts_with('&'));
        assert!(name.is_ascii());
        assert!(
            name.bytes().enumerate().all(|(index, byte)| {
                byte.is_ascii_alphanumeric() || (byte == b';' && index + 1 == name.len())
            }),
            "invalid generated key shape: {name}"
        );
        assert!(matches!(value.chars().count(), 1 | 2));
    }
}

#[test]
fn semicolonless_entries_have_equal_terminated_counterparts() {
    for (name, value) in NAMED_CHARACTER_REFERENCES
        .iter()
        .filter(|(name, _)| !name.ends_with(';'))
    {
        let counterpart = format!("{name};");
        assert_eq!(lookup(&counterpart), Some(*value), "counterpart for {name}");
    }
}

#[test]
fn derived_metadata_matches_complete_rows() {
    let semicolonless = NAMED_CHARACTER_REFERENCES
        .iter()
        .filter(|(name, _)| !name.ends_with(';'))
        .count();
    let two_scalar = NAMED_CHARACTER_REFERENCES
        .iter()
        .filter(|(_, value)| value.chars().count() == 2)
        .count();
    let maximum_length = NAMED_CHARACTER_REFERENCES
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .expect("the generated table is non-empty");
    let maximum_names: Vec<_> = NAMED_CHARACTER_REFERENCES
        .iter()
        .filter_map(|(name, _)| (name.len() == maximum_length).then_some(*name))
        .collect();

    assert_eq!(
        semicolonless,
        NAMED_CHARACTER_REFERENCE_SEMICOLONLESS_ENTRY_COUNT
    );
    assert_eq!(two_scalar, NAMED_CHARACTER_REFERENCE_TWO_SCALAR_ENTRY_COUNT);
    assert_eq!(
        maximum_length,
        NAMED_CHARACTER_REFERENCE_MAXIMUM_NAME_BYTE_LENGTH
    );
    assert_eq!(maximum_names, NAMED_CHARACTER_REFERENCE_MAXIMUM_NAMES);
}

#[test]
fn challenge_cells_are_exact_and_case_sensitive() {
    for (name, expected) in [
        ("amp;", "\u{26}"),
        ("amp", "\u{26}"),
        ("lt;", "\u{3C}"),
        ("AMP;", "\u{26}"),
        ("not", "\u{AC}"),
        ("not;", "\u{AC}"),
        ("notin;", "\u{2209}"),
        ("acE;", "\u{223E}\u{333}"),
        ("CounterClockwiseContourIntegral;", "\u{2233}"),
        ("Afr;", "\u{1D504}"),
    ] {
        assert_eq!(lookup(name), Some(expected), "challenge cell {name}");
    }
    assert_ne!("AMP;", "amp;");
    assert_ne!("amp", "amp;");
}
