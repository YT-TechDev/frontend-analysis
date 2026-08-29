//! Test-only evidence for the owned generated Named Character Reference data.
//!
//! The owned rows and the owner-private provenance facts are read through the
//! canonical owner, never through a raw generated declaration, so these tests
//! cannot become an alternate production authority for the table.

// Accidental production exposure of this module is a compiler error rather
// than something repository tooling has to detect by scanning source.
#[cfg(not(test))]
compile_error!("generated Named Character Reference data tests are test-only");

use super::named_character_reference_data::{
    maximum_name_byte_length, rows,
    test_inspection::{
        entry_count, maximum_names, retained_entities_sha256, semicolonless_entry_count,
        two_scalar_entry_count, upstream_manifest_sha256, whatwg_html_snapshot,
    },
};

fn lookup(name: &str) -> Option<&'static str> {
    let table = rows();
    table
        .binary_search_by(|(candidate, _)| candidate.cmp(&name))
        .ok()
        .map(|index| table[index].1)
}

#[test]
fn authority_metadata_is_frozen() {
    assert_eq!(
        whatwg_html_snapshot(),
        "8ad51e24e9d9e48d92317467f434f7192df9d63d"
    );
    assert_eq!(
        retained_entities_sha256(),
        "d741d877ac77c4194c4ad526b5b4a19aef8dfe411ab840a466891cdbb9f362e6"
    );
    assert_eq!(
        upstream_manifest_sha256(),
        "234ff9717d9189382699bbc26105671d4d62240377a514a15e28e716d747e908"
    );
}

#[test]
fn complete_table_is_sorted_unique_and_exactly_counted() {
    assert_eq!(entry_count(), 2_231);
    assert_eq!(rows().len(), entry_count());
    for adjacent in rows().windows(2) {
        assert!(
            adjacent[0].0 < adjacent[1].0,
            "generated names must be strictly sorted and unique: {:?}",
            adjacent
        );
    }
}

#[test]
fn generated_key_shape_is_exact() {
    for (name, value) in rows() {
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
    for (name, value) in rows().iter().filter(|(name, _)| !name.ends_with(';')) {
        let counterpart = format!("{name};");
        assert_eq!(lookup(&counterpart), Some(*value), "counterpart for {name}");
    }
}

#[test]
fn derived_metadata_matches_complete_rows() {
    let semicolonless = rows()
        .iter()
        .filter(|(name, _)| !name.ends_with(';'))
        .count();
    let two_scalar = rows()
        .iter()
        .filter(|(_, value)| value.chars().count() == 2)
        .count();
    let maximum_length = rows()
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .expect("the generated table is non-empty");
    let observed_maximum_names: Vec<_> = rows()
        .iter()
        .filter_map(|(name, _)| (name.len() == maximum_length).then_some(*name))
        .collect();

    assert_eq!(semicolonless, semicolonless_entry_count());
    assert_eq!(two_scalar, two_scalar_entry_count());
    assert_eq!(maximum_length, maximum_name_byte_length());
    assert_eq!(observed_maximum_names, maximum_names());
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
