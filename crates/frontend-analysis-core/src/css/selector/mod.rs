//! Source-backed CSS selector qualification boundary (#182/#184).
//!
//! #182 freezes the crate-private domain/resource/gold contracts. #184 adds
//! the bounded production `CoreV1` qualifier and a Core-integrated entry path
//! over the existing structurally validated parser result. Structural parsing
//! remains owned by `super::parser`.
//!
//! No type in this module is publicly exported.

pub(crate) mod analysis;
pub(crate) mod context;
pub(crate) mod lexical;
pub(crate) mod producer;
pub(crate) mod profile;
pub(crate) mod resource;
pub(crate) mod result;
