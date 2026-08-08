//! Internal HTML source-parser contracts.

pub(crate) mod analysis;
pub(crate) mod parser;
pub(crate) mod token;
pub(crate) mod tokenizer;

#[cfg(test)]
mod token_contract_matrix_tests;
#[cfg(test)]
mod token_tests;
