//! Source-backed CSS selector qualification domain contracts (#182).
//!
//! This module materializes the #181-approved semantic boundary without
//! implementing the production selector qualifier. Structural CSS parsing
//! remains owned by `super::parser`; selector qualification is a later stage
//! over the Core-validated parser result.
//!
//! No type in this module is publicly exported.

pub(crate) mod context;
pub(crate) mod lexical;
pub(crate) mod profile;
pub(crate) mod resource;
pub(crate) mod result;
