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
//! Nothing here consumes source, advances a cursor, raises a diagnostic,
//! emits a token, or retains a matching buffer. The candidate is compared in
//! place against each generated identifier, so the selected mechanism keeps
//! `TemporaryBufferBytes` truthfully zero.

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
pub(super) fn maximum_match(first: char, rest: &[u8]) -> Option<NamedCharacterReferenceMatch> {
    named_character_reference_data::rows()
        .iter()
        .filter(|(name, _)| candidate_starts_with(name, first, rest))
        .max_by_key(|(name, _)| name.len())
        .map(|(name, value)| NamedCharacterReferenceMatch { name, value })
}

/// Whether `first` followed by `rest` begins with exactly `name`.
///
/// Compared in place, byte by byte, with no candidate buffer. Generated
/// identifiers are ASCII, and every byte of a multi-byte UTF-8 sequence is
/// `>= 0x80`, so a byte comparison against `rest` can never match across a
/// scalar boundary and this never becomes a second decoding authority.
fn candidate_starts_with(name: &str, first: char, rest: &[u8]) -> bool {
    let Some((head, tail)) = name.as_bytes().split_first() else {
        return false;
    };
    if !first.is_ascii() || u32::from(*head) != u32::from(first) {
        return false;
    }
    rest.len() >= tail.len() && &rest[..tail.len()] == tail
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn matching_agrees_with_the_owner_over_every_generated_identifier() {
        // Brute-force cross-check against the complete owned table: for every
        // generated identifier, matching that exact identifier resolves a
        // name at least as long, and never a longer name that is not itself a
        // prefix of the candidate.
        for (name, _) in named_character_reference_data::rows() {
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
        assert!(maximum_match('\u{00e9}', b"acute;").is_none());
    }
}
