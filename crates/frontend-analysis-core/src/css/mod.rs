//! CSS source-parser contracts.
//!
//! All modules are crate-private except [`crate::css::selectors`], the narrow
//! public `CoreV1` selector-analysis consumer facade.

pub(crate) mod analysis;
pub(crate) mod declaration;
pub(crate) mod parser;
pub(crate) mod selector;
pub mod selectors;
pub(crate) mod token;
pub(crate) mod tokenizer;
pub(crate) mod value_qualification;

#[cfg(test)]
mod token_tests;
#[cfg(test)]
mod tokenizer_contract_tests;
#[cfg(test)]
mod validation;
