//! Browser-independent validated source anchoring and raw coordinate primitives.

mod raw_source_coordinate;
mod source;

#[cfg(test)]
mod contract_tests;

pub use raw_source_coordinate::RawSourceCoordinate;
pub use source::{SourceAnchor, SourceId, SourceRange, SourceRangeError, SourceText};
