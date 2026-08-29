//! Selected Named Character Reference maximum matching.
//!
//! This module owns *matching behavior* only. The canonical WHATWG generated
//! data stays owned by the compiler-sealed tokenizer-private owner accepted
//! under #398 / PR #399, and is reached here through exactly its two narrow
//! wrappers:
//!
//! ```text
//! producer::named_character_reference        // matching semantics
//!         ↓
//! named_character_reference_data::rows()
//! named_character_reference_data::maximum_name_byte_length()
//!         ↓
//! compiler-sealed canonical generated owner  // data ownership
//! ```
//!
//! There is deliberately no other path. This module never names
//! `named_character_references_generated`, never declares a table, whitelist,
//! copy, or subset of its own, never reads a file at runtime, and never
//! consults a third-party entity source. Removing the owner or its wrappers
//! stops this module compiling, which is the intended seal.
//!
//! Matching is **maximum match**, not whole-string lookup: the selected result
//! is the longest complete generated identifier that is a prefix of the
//! authored candidate. That distinction is load-bearing — `&notit;` resolves
//! the semicolonless `not` and leaves `it;` as ordinary later input, while
//! `&notin;` resolves the longer `notin;`.
//!
//! Maximum match is decided by **bounded exact lookup, never by scanning the
//! table**. The canonical rows are sorted by identifier bytes, so each
//! candidate prefix is settled by one binary search; prefixes are tried
//! longest-first and the first hit is the maximum match. The work per
//! reference is therefore bounded by
//! `maximum_name_byte_length()` binary searches — independent of how many rows
//! the canonical table holds. Both preconditions this relies on, sortedness
//! and the bounded probe budget, are pinned by the tests below.
//!
//! Nothing here consumes source, advances a cursor, raises a diagnostic,
//! emits a token, or retains a matching buffer. The candidate is never
//! materialized: it is addressed in place as "the already-materialized first
//! scalar, then borrowed unconsumed bytes", so the selected mechanism keeps
//! `TemporaryBufferBytes` truthfully zero.

use std::cmp::Ordering;

use super::super::named_character_reference_data;

/// One selected maximum match against the canonical generated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NamedCharacterReferenceMatch {
    /// The exact generated identifier, without the authored `&`. Its byte
    /// length is exactly the authored length the caller must consume.
    pub(super) name: &'static str,
    /// The generated decoded value: one or two Unicode scalars. It is output
    /// only and is never reintroduced as tokenizer input.
    pub(super) value: &'static str,
}

impl NamedCharacterReferenceMatch {
    /// Whether the matched identifier ended in the authored `;`.
    ///
    /// A maximum match that did not is still a resolution; the pinned
    /// standard additionally requires
    /// `missing-semicolon-after-character-reference`.
    pub(super) fn ends_with_semicolon(self) -> bool {
        self.name.ends_with(';')
    }

    /// Whether every byte of the matched identifier is one the authoritative
    /// cursor will hand back unchanged and without a preprocessing
    /// diagnostic.
    ///
    /// Canonical identifiers are ASCII alphanumeric plus `;`, so none of them
    /// CR-normalizes and none is a flagged control character or a
    /// noncharacter. The selected commit relies on this to consume the matched
    /// source through an infallible primitive, so it is checked rather than
    /// assumed, and pinned exhaustively over the whole canonical table below.
    pub(super) fn is_plainly_consumable(self) -> bool {
        self.name
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b';')
    }
}

/// The largest number of still-unconsumed authored bytes a maximum match can
/// need, given that the caller has already materialized the first identifier
/// scalar.
///
/// Bounded by the owner's own longest generated identifier, so the bound
/// cannot drift from the data it bounds.
pub(super) fn maximum_lookahead_bytes() -> usize {
    named_character_reference_data::maximum_name_byte_length().saturating_sub(1)
}

/// Selects the longest generated identifier that is a prefix of `first`
/// followed by `rest`.
///
/// `first` is the already-materialized first identifier scalar and `rest` is
/// bounded borrowed unconsumed source. Returns `None` when no complete
/// generated identifier matches, which is exactly the Ambiguous Ampersand
/// condition.
///
/// Longest-first bounded exact lookup: for each candidate length, one binary
/// search decides whether the canonical table holds exactly that identifier.
/// The first hit is the maximum match, so at most
/// `maximum_name_byte_length()` searches run and the table is never scanned.
pub(super) fn maximum_match(first: char, rest: &[u8]) -> Option<NamedCharacterReferenceMatch> {
    if !first.is_ascii() {
        // Every canonical identifier is ASCII, so a non-ASCII first scalar
        // completes none of them and needs no lookup at all.
        return None;
    }
    let rows = named_character_reference_data::rows();
    let longest = named_character_reference_data::maximum_name_byte_length();
    // The candidate is the first scalar plus whatever was lent, and can never
    // be longer than the longest identifier the owner reports.
    let candidate_length = rest.len().saturating_add(1).min(longest);
    for length in (1..=candidate_length).rev() {
        if let Ok(index) = rows.binary_search_by(|(name, _)| compare_row(name, first, rest, length))
        {
            let (name, value) = rows[index];
            return Some(NamedCharacterReferenceMatch { name, value });
        }
    }
    None
}

/// Orders one canonical row against the candidate prefix of `length` bytes.
///
/// The candidate is addressed in place through [`candidate_byte`] rather than
/// copied, and the comparison is plain lexicographic byte order — the same
/// order the canonical table is generated in, which
/// [`tests::the_canonical_table_is_sorted_by_identifier_bytes`] pins as the
/// precondition this binary search depends on.
fn compare_row(name: &str, first: char, rest: &[u8], length: usize) -> Ordering {
    #[cfg(test)]
    tests::record_row_comparison();
    let name = name.as_bytes();
    let common = name.len().min(length);
    for (index, byte) in name.iter().enumerate().take(common) {
        let ordering = byte.cmp(&candidate_byte(index, first, rest));
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    name.len().cmp(&length)
}

/// The candidate's byte at `index`: the already-materialized first scalar,
/// then the borrowed unconsumed bytes.
///
/// Generated identifiers are ASCII, and every byte of a multi-byte UTF-8
/// sequence is `>= 0x80`, so a byte comparison against `rest` can never match
/// across a scalar boundary and this never becomes a second decoding
/// authority.
fn candidate_byte(index: usize, first: char, rest: &[u8]) -> u8 {
    match index.checked_sub(1) {
        None => first as u8,
        Some(borrowed) => rest[borrowed],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    thread_local! {
        /// Test-only budget instrumentation. Present only under `cfg(test)`,
        /// so production carries no counter and no interior mutability.
        static ROW_COMPARISONS: Cell<usize> = const { Cell::new(0) };
    }

    pub(super) fn record_row_comparison() {
        ROW_COMPARISONS.with(|count| count.set(count.get().saturating_add(1)));
    }

    fn counted<T>(body: impl FnOnce() -> T) -> (T, usize) {
        ROW_COMPARISONS.with(|count| count.set(0));
        let value = body();
        (value, ROW_COMPARISONS.with(Cell::get))
    }

    fn matched(authored: &str) -> Option<NamedCharacterReferenceMatch> {
        let mut characters = authored.chars();
        let first = characters.next().expect("a first identifier scalar");
        let rest = characters.as_str().as_bytes();
        let bounded = &rest[..rest.len().min(maximum_lookahead_bytes())];
        maximum_match(first, bounded)
    }

    #[test]
    fn the_lookahead_bound_follows_the_owner_and_leaves_room_for_the_first_scalar() {
        let owned = named_character_reference_data::maximum_name_byte_length();
        assert!(owned > 1);
        assert_eq!(maximum_lookahead_bytes(), owned - 1);
    }

    /// The precondition the bounded lookup depends on. Binary search over an
    /// unsorted table would silently miss identifiers, so this is checked
    /// against the owner rather than assumed from the generator.
    #[test]
    fn the_canonical_table_is_sorted_by_identifier_bytes() {
        let rows = named_character_reference_data::rows();
        assert!(rows.len() > 1);
        for pair in rows.windows(2) {
            assert!(
                pair[0].0.as_bytes() < pair[1].0.as_bytes(),
                "canonical rows are not strictly ascending at {:?} / {:?}",
                pair[0].0,
                pair[1].0
            );
        }
    }

    /// Falsifies a return to full-table scanning.
    ///
    /// A linear scan touches every one of the canonical rows for every
    /// reference. Bounded longest-first exact lookup touches at most one
    /// binary search per candidate length, so the comparison budget stays
    /// far below the table size no matter which identifier is matched.
    #[test]
    fn matching_never_scans_the_canonical_table() {
        let rows = named_character_reference_data::rows().len();
        let longest = named_character_reference_data::maximum_name_byte_length();
        // Generous: ceil(log2(rows)) + 1 comparisons per binary search, at
        // most `longest` searches. Still an order of magnitude under `rows`.
        let budget = longest * (usize::BITS as usize - rows.leading_zeros() as usize + 1);
        assert!(budget < rows, "the budget must falsify a full scan");

        for authored in [
            "notin;</title>",
            "notit;</title>",
            "CounterClockwiseContourIntegral;</title>",
            "acE;</title>",
            "nOtAnEntityAtAllHereOkay;</title>",
            "amp b</title>",
        ] {
            let (_, comparisons) = counted(|| matched(authored));
            assert!(
                comparisons <= budget,
                "{authored:?} used {comparisons} row comparisons, budget {budget}, table {rows}"
            );
            // Lower bound, so the budget cannot be satisfied vacuously: an
            // implementation that reached the rows some other way — a
            // `starts_with` scan, say — would never reach the comparator and
            // would report zero here.
            assert!(
                comparisons > 0,
                "{authored:?} settled without consulting the canonical table"
            );
        }
    }

    #[test]
    fn matching_selects_the_maximum_generated_identifier() {
        // The adversarial pair: no `notit` identifier exists, so the maximum
        // match of `notit;` is the shorter semicolonless `not`, while
        // `notin;` resolves the longer identifier. A shortest-match or
        // whole-string implementation fails one of these.
        let shorter = matched("notit;").expect("a maximum match");
        assert_eq!(shorter.name, "not");
        assert!(!shorter.ends_with_semicolon());

        let longer = matched("notin;").expect("a maximum match");
        assert_eq!(longer.name, "notin;");
        assert!(longer.ends_with_semicolon());
    }

    #[test]
    fn matching_resolves_a_two_scalar_generated_value() {
        let two_scalar = matched("acE;").expect("a maximum match");
        assert_eq!(two_scalar.name, "acE;");
        assert_eq!(two_scalar.value.chars().count(), 2);
    }

    #[test]
    fn matching_refuses_a_candidate_that_completes_no_generated_identifier() {
        assert!(matched("nOtAnEntity;").is_none());
    }

    /// The exhaustive semantic oracle: bounded lookup must agree with the
    /// complete owned table, identifier by identifier.
    #[test]
    fn matching_agrees_with_the_owner_over_every_generated_identifier() {
        for (name, value) in named_character_reference_data::rows() {
            let selected = matched(name).expect("every generated identifier resolves");
            assert!(
                name.starts_with(selected.name),
                "selected {} is not a prefix of {name}",
                selected.name
            );
            assert!(
                selected.name.len() >= name.len(),
                "selected {} is shorter than the exact generated {name}",
                selected.name
            );
            // Exactly the generated identifier, so the decoded value matches.
            assert_eq!(selected.name, *name);
            assert_eq!(selected.value, *value);
        }
    }

    /// The precondition the infallible selected consumption relies on.
    #[test]
    fn every_generated_identifier_is_plainly_consumable() {
        for (name, _) in named_character_reference_data::rows() {
            let selected = matched(name).expect("every generated identifier resolves");
            assert!(
                selected.is_plainly_consumable(),
                "{name:?} is not plainly consumable"
            );
            assert!(
                !name.contains('\r') && name.is_ascii(),
                "{name:?} would not survive the authoritative cursor unchanged"
            );
        }
    }

    #[test]
    fn matching_never_reads_past_the_borrowed_bound() {
        // Truncating the borrow one byte short of the identifier must lose
        // the match rather than read beyond what the caller lent.
        let name = "notin;";
        let mut characters = name.chars();
        let first = characters.next().expect("a first identifier scalar");
        let rest = characters.as_str().as_bytes();
        assert_eq!(
            maximum_match(first, &rest[..rest.len() - 1])
                .expect("the shorter maximum match remains")
                .name,
            "not"
        );
    }

    #[test]
    fn a_non_ascii_first_scalar_matches_no_generated_identifier() {
        let (selected, comparisons) = counted(|| maximum_match('\u{00e9}', b"acute;"));
        assert!(selected.is_none());
        // Settled without consulting the table at all.
        assert_eq!(comparisons, 0);
    }
}
