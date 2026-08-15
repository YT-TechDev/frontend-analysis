//! Browser-independent validated source anchoring and raw coordinate primitives.

// The approved HTML token contracts are intentionally internal until the first
// tokenizer and Core integration consume them under Issues #113 and #116.
#[allow(dead_code)]
mod html;
// The first CSS lexical contracts remain crate-private while Issues #133-#140
// validate the project-owned CSS frontend before any public API commitment.
#[allow(dead_code)]
mod css;
#[allow(dead_code)]
mod ecmascript;
mod raw_source_coordinate;
mod source;

#[cfg(test)]
mod contract_tests;

pub use raw_source_coordinate::RawSourceCoordinate;
pub use source::{SourceAnchor, SourceId, SourceRange, SourceRangeError, SourceText};
