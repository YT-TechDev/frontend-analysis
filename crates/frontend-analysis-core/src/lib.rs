//! Browser-independent validated source anchoring and raw coordinate
//! primitives, plus the narrow CSS `CoreV1` selector-analysis consumer facade.

// The approved HTML token, tokenizer, and tree-construction contracts remain
// crate-private; only the narrow `html::tree` consumer facade is public
// (Issue #864, ADR 0011).
#[allow(dead_code)]
pub mod html;
// The CSS tokenizer, parser, selector, and resource contracts remain
// crate-private; only the narrow `css::selectors` CoreV1 consumer facade is
// public (Issue #857, ADR 0011).
#[allow(dead_code)]
pub mod css;
// The ECMAScript selected-slice implementation modules remain crate-private;
// only the narrow `ecmascript::binding_refs` consumer facade is public
// (Issue #862, ADR 0011).
#[allow(dead_code)]
pub mod ecmascript;
mod raw_source_coordinate;
mod source;

#[cfg(test)]
mod contract_tests;

pub use raw_source_coordinate::RawSourceCoordinate;
pub use source::{SourceAnchor, SourceId, SourceRange, SourceRangeError, SourceText};
