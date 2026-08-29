pub(crate) mod diagnostic;
pub(crate) mod producer;
pub(crate) mod resource;
pub(crate) mod result;

// The canonical Named Character Reference data owner is private to the
// tokenizer: production consumers reach the owned rows only through its narrow
// wrapper, and the canonical generated source is included lexically inside it
// rather than declared as a module of its own.
mod named_character_reference_data;

#[cfg(test)]
mod named_character_references_data_tests;
#[cfg(test)]
mod review_regression_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod validation;
