//! Canonical owner of the generated WHATWG Named Character Reference data.
//!
//! Ownership is proved by the compiler rather than by repository source
//! scanning. This module declares a private registration authority, lexically
//! includes the canonical generated source so that source can name that
//! authority, and exposes only a narrow tokenizer-private wrapper:
//!
//! - privacy stops any module outside this one naming the registration
//!   authority or the raw generated declarations;
//! - trait coherence rejects a second active registration of
//!   [`OwnershipRegistration`] for [`OwnerToken`];
//! - the wrapper below is the only production path to the owned rows, and it
//!   is reachable only while the canonical registration exists.
//!
//! `rustc` owns syntax, `cfg`/module selection, privacy, and coherence. Nothing
//! here re-derives those semantics, and nothing here interprets the generated
//! file as anything other than lexical Rust.
//!
//! The semantic and provenance authority for the included rows remains the
//! deterministic offline generator and its independent verifier under
//! `tools/html/named_character_references`. This module establishes ownership
//! and wiring only; it selects no matcher, lookahead, or tokenizer behavior.

/// Private registration authority.
///
/// Only the canonical generated source included below implements this trait,
/// and only for [`OwnerToken`]. It is a marker: the registration item carries
/// no data, so it stays fixed while the generated rows vary with the retained
/// upstream evidence.
trait OwnershipRegistration {}

/// Private token named by the canonical generated registration.
struct OwnerToken;

include!("named_character_references_generated.rs");

/// The owned rows, as exact generated name to decoded Unicode string.
///
/// The registration bound is load-bearing rather than decorative: without the
/// canonical generated registration, `OwnerToken` does not implement
/// [`OwnershipRegistration`] and this function does not resolve.
fn registered_rows<Registered: OwnershipRegistration>() -> &'static [(&'static str, &'static str)] {
    NAMED_CHARACTER_REFERENCES
}

/// Tokenizer-private access to the complete owned table.
pub(super) fn rows() -> &'static [(&'static str, &'static str)] {
    registered_rows::<OwnerToken>()
}

/// The longest owned generated name in bytes, for bounded consumer lookahead.
pub(super) fn maximum_name_byte_length() -> usize {
    NAMED_CHARACTER_REFERENCE_MAXIMUM_NAME_BYTE_LENGTH
}

/// Test-only inspection of owner-private generated facts.
///
/// This seam exists so the generated data tests can assert the retained
/// provenance and derived metadata without any of it becoming production
/// surface. It is absent from every non-test configuration, so it cannot
/// widen the production boundary established above.
#[cfg(test)]
pub(super) mod test_inspection {
    pub(in crate::html::tokenizer) fn whatwg_html_snapshot() -> &'static str {
        super::WHATWG_HTML_SNAPSHOT
    }

    pub(in crate::html::tokenizer) fn retained_entities_sha256() -> &'static str {
        super::RETAINED_ENTITIES_SHA256
    }

    pub(in crate::html::tokenizer) fn upstream_manifest_sha256() -> &'static str {
        super::UPSTREAM_MANIFEST_SHA256
    }

    pub(in crate::html::tokenizer) fn entry_count() -> usize {
        super::NAMED_CHARACTER_REFERENCE_ENTRY_COUNT
    }

    pub(in crate::html::tokenizer) fn semicolonless_entry_count() -> usize {
        super::NAMED_CHARACTER_REFERENCE_SEMICOLONLESS_ENTRY_COUNT
    }

    pub(in crate::html::tokenizer) fn two_scalar_entry_count() -> usize {
        super::NAMED_CHARACTER_REFERENCE_TWO_SCALAR_ENTRY_COUNT
    }

    pub(in crate::html::tokenizer) fn maximum_names() -> &'static [&'static str] {
        super::NAMED_CHARACTER_REFERENCE_MAXIMUM_NAMES
    }
}
