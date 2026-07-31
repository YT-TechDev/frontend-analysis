//! Browser-independent validated source anchoring primitives.

mod source;

#[cfg(test)]
mod contract_tests;

pub use source::{SourceAnchor, SourceId, SourceRange, SourceRangeError, SourceText};
