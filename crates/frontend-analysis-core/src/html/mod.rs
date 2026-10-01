//! Internal HTML source-parser contracts.
//!
//! All modules are crate-private except [`crate::html::tree`], the narrow
//! public selected document-construction consumer facade (Issue #864, ADR
//! 0011).

pub(crate) mod analysis;
pub(crate) mod parser;
pub(crate) mod token;
pub(crate) mod tokenizer;
pub mod tree;
pub(crate) mod tree_construction;

#[cfg(test)]
mod compiler_sealed_ownership_validation;
#[cfg(test)]
mod token_contract_matrix_tests;
#[cfg(test)]
mod token_tests;
