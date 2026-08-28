//! The private, non-generated Named Character Reference matching seam.
//!
//! The complete deterministic WHATWG data foundation established by #392/#393
//! stays exactly as generated. This module is the separate tokenizer-owned
//! runtime seam over it: it owns *matching* semantics only, never entity data.
//! It contains no table of its own, no hand-maintained whitelist, no copy of
//! the generated rows, no runtime I/O, and no third-party entity source.
//!
//! Two properties the selected TC-S10 theorem depends on are structural here:
//!
//! - **Maximum match, not whole-string lookup.** Matching walks candidate
//!   identifier bytes one at a time and remembers the *longest* prefix that is
//!   itself a complete generated identifier. `&notit;` therefore matches the
//!   semicolonless `not` and leaves `it;` as ordinary later input, while
//!   `&notin;` resolves the longer `notin;` identifier. An exact whole-string
//!   lookup would get exactly this pair wrong.
//! - **Non-committing.** [`maximum_match`] is a pure function of one already
//!   materialized ASCII byte plus borrowed future raw bytes. It advances
//!   nothing, observes nothing through preprocessing, allocates nothing, and
//!   retains nothing — so the peak `TemporaryBufferBytes` claim of zero stays
//!   truthful, and discovery can be discarded with no partial effect to undo.
//!
//! The generated table is sorted, strictly increasing and unique by
//! identifier, which is what makes the incremental narrowing below a binary
//! search rather than a scan. That ordering is asserted by the generated
//! data's own tests; this module additionally proves the narrowing agrees with
//! a brute-force maximum match over the same generated rows.

use super::super::named_character_references_generated::{
    NAMED_CHARACTER_REFERENCE_MAXIMUM_NAME_BYTE_LENGTH, NAMED_CHARACTER_REFERENCES,
};

/// The exact bounded borrowed lookahead the matcher may observe beyond the
/// already materialized first identifier unit.
///
/// One less than the longest generated identifier: the first byte is the unit
/// the tokenizer has already consumed. A window this size can never truncate
/// a real match and can never be used to reach further into source.
pub(super) const LOOKAHEAD_BYTES: usize = NAMED_CHARACTER_REFERENCE_MAXIMUM_NAME_BYTE_LENGTH - 1;

/// One selected maximum Named Character Reference match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NamedMatch {
    /// Byte length of the matched identifier, counted from the first
    /// identifier byte (the unit immediately after the authored `&`).
    ///
    /// Always at least 1, so the caller's authoritative consumption is always
    /// a bounded, forward, unit-by-unit advance over exactly these bytes.
    pub(super) name_len: usize,
    /// The generated decoded value: one or two Unicode scalars, borrowed
    /// `'static` from the generated table rather than copied.
    pub(super) value: &'static str,
    /// Whether the matched identifier's own last byte is the authored `;`.
    ///
    /// `false` selects the `missing-semicolon-after-character-reference`
    /// obligation; it never changes which identifier was matched.
    pub(super) ends_with_semicolon: bool,
}

/// Selects the maximum generated identifier that is a prefix of
/// `first` followed by `rest`.
///
/// `first` is the already materialized ASCII unit that caused Named
/// Character Reference dispatch; `rest` is the borrowed, not-yet-consumed raw
/// source after it. Returns `None` when no complete identifier matches, which
/// is the Ambiguous Ampersand path — never a shorter "best effort" answer.
pub(super) fn maximum_match(first: u8, rest: &[u8]) -> Option<NamedMatch> {
    if !is_identifier_byte(first) {
        return None;
    }

    // The active window is the contiguous run of generated rows whose
    // identifier starts with the candidate bytes examined so far. It starts
    // as the whole sorted table and only ever narrows.
    let mut low = 0usize;
    let mut high = NAMED_CHARACTER_REFERENCES.len();
    let mut best: Option<NamedMatch> = None;

    let mut length = 0usize;
    while let Some(byte) = candidate_byte(first, rest, length) {
        if !is_identifier_byte(byte) {
            break;
        }
        length += 1;
        if length > NAMED_CHARACTER_REFERENCE_MAXIMUM_NAME_BYTE_LENGTH {
            break;
        }

        let window = &NAMED_CHARACTER_REFERENCES[low..high];
        // Inside `window` every identifier already shares the previous
        // `length - 1` bytes, so rows sort as: the one identifier that ends
        // exactly here (if any) first, then the rest ascending by this byte.
        let narrowed_low =
            low + window.partition_point(|(name, _)| byte_at(name, length - 1) < Some(byte));
        let narrowed_high =
            low + window.partition_point(|(name, _)| byte_at(name, length - 1) <= Some(byte));
        if narrowed_low == narrowed_high {
            break;
        }
        low = narrowed_low;
        high = narrowed_high;

        let (name, value) = NAMED_CHARACTER_REFERENCES[low];
        if name.len() == length {
            best = Some(NamedMatch {
                name_len: length,
                value,
                ends_with_semicolon: byte == b';',
            });
        }
        if byte == b';' {
            // No generated identifier continues past its terminating `;`,
            // so nothing longer than this window can still match.
            break;
        }
    }

    best
}

/// The candidate identifier byte at `index`, counting the already
/// materialized `first` unit as index 0.
fn candidate_byte(first: u8, rest: &[u8], index: usize) -> Option<u8> {
    match index {
        0 => Some(first),
        _ => rest.get(index - 1).copied(),
    }
}

/// `None` past the identifier's end, so a shorter identifier orders before
/// every longer one sharing its bytes — exactly the generated table's order.
fn byte_at(name: &str, index: usize) -> Option<u8> {
    name.as_bytes().get(index).copied()
}

/// The exact generated identifier alphabet: ASCII alphanumerics, plus the
/// single terminating `;`. Anything else can never extend or start a match.
fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b';'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn match_after_ampersand(rest: &str) -> Option<NamedMatch> {
        let bytes = rest.as_bytes();
        let (first, tail) = bytes.split_first()?;
        maximum_match(*first, tail)
    }

    /// Independent maximum match over the same generated rows, written the
    /// slow obvious way. Any disagreement means the narrowing is wrong.
    fn brute_force(rest: &str) -> Option<(usize, &'static str)> {
        let bytes = rest.as_bytes();
        let mut best = None;
        for (name, value) in NAMED_CHARACTER_REFERENCES {
            let length = name.len();
            if length <= bytes.len() && name.as_bytes() == &bytes[..length] {
                match best {
                    Some((best_length, _)) if best_length >= length => {}
                    _ => best = Some((length, *value)),
                }
            }
        }
        best
    }

    #[test]
    fn maximum_match_prefers_the_longest_complete_identifier() {
        let notit = match_after_ampersand("notit;").expect("semicolonless not");
        assert_eq!(notit.name_len, 3);
        assert_eq!(notit.value, "\u{ac}");
        assert!(!notit.ends_with_semicolon);

        let notin = match_after_ampersand("notin;").expect("notin;");
        assert_eq!(notin.name_len, 6);
        assert_eq!(notin.value, "\u{2209}");
        assert!(notin.ends_with_semicolon);
    }

    #[test]
    fn whole_string_lookup_would_be_wrong_here() {
        // `notit` and `notit;` are not generated identifiers at all, so an
        // exact whole-string lookup answers `None` where maximum match must
        // answer `not`.
        assert!(
            !NAMED_CHARACTER_REFERENCES
                .iter()
                .any(|(name, _)| *name == "notit;")
        );
        assert!(
            !NAMED_CHARACTER_REFERENCES
                .iter()
                .any(|(name, _)| *name == "notit")
        );
        assert!(match_after_ampersand("notit;").is_some());
    }

    #[test]
    fn two_scalar_values_are_borrowed_unchanged() {
        let ace = match_after_ampersand("acE;").expect("acE;");
        assert_eq!(ace.name_len, 4);
        assert_eq!(ace.value, "\u{223e}\u{333}");
        assert_eq!(ace.value.chars().count(), 2);
    }

    #[test]
    fn unmatched_identifiers_select_nothing() {
        assert!(match_after_ampersand("bogus;").is_none());
        assert!(match_after_ampersand("Bogus;").is_none());
        // `no` is a strict prefix of generated identifiers but is not one.
        assert!(match_after_ampersand("no").is_none());
    }

    #[test]
    fn non_identifier_bytes_terminate_matching() {
        assert!(maximum_match(b'#', b"60;").is_none());
        assert!(maximum_match(b' ', b"amp;").is_none());
        let amp = match_after_ampersand("amp x").expect("semicolonless amp");
        assert_eq!(amp.name_len, 3);
        assert!(!amp.ends_with_semicolon);
    }

    #[test]
    fn a_truncated_lookahead_window_cannot_extend_a_match() {
        // The caller bounds the borrowed window; a cut window may only
        // shorten the answer, never invent a longer one.
        assert_eq!(
            maximum_match(b'n', b"otin;").map(|found| found.name_len),
            Some(6)
        );
        assert_eq!(
            maximum_match(b'n', b"oti").map(|found| found.name_len),
            Some(3)
        );
        assert_eq!(maximum_match(b'n', b"").map(|found| found.name_len), None);
    }

    #[test]
    fn narrowing_agrees_with_brute_force_on_every_generated_identifier() {
        for (name, _) in NAMED_CHARACTER_REFERENCES {
            for suffix in ["", ";", "x", "it;", "0"] {
                let probe = format!("{name}{suffix}");
                let expected = brute_force(&probe);
                let actual =
                    match_after_ampersand(&probe).map(|found| (found.name_len, found.value));
                assert_eq!(actual, expected, "maximum match disagreed for {probe:?}");
            }
        }
    }

    #[test]
    fn every_selected_match_reports_its_own_terminating_byte() {
        for (name, _) in NAMED_CHARACTER_REFERENCES {
            let found = match_after_ampersand(name).expect("generated identifier matches itself");
            assert_eq!(found.name_len, name.len());
            assert_eq!(found.ends_with_semicolon, name.ends_with(';'));
        }
    }
}
