pub(crate) mod diagnostic;
pub(crate) mod producer;
pub(crate) mod resource;
pub(crate) mod result;

// The canonical Named Character Reference data owner is private to the
// tokenizer: production consumers reach the owned rows only through its narrow
// wrapper, and the canonical generated source is included lexically inside it
// rather than declared as a module of its own.
mod named_character_reference_data;

// Unconditional production ownership anchor.
//
// Declaring the owner is not by itself enough to seal it. Without a consumer
// outside the owner, gating the module — for example `#[cfg(test)] mod
// named_character_reference_data;` — would silently remove the owner, and with
// it the canonical generated registration, from every production build while
// that build still succeeded.
//
// These witnesses close that outer edge. They are ordinary const items, so
// `rustc` resolves and type-checks them in every configuration that compiles
// this module: the tokenizer cannot build unless the canonical owner is
// present and its narrow wrapper resolves. They coerce function items to
// function pointers at compile time, hold no state, run nothing, and are
// reachable from no other code.
const _: fn() -> &'static [(&'static str, &'static str)] = named_character_reference_data::rows;
const _: fn() -> usize = named_character_reference_data::maximum_name_byte_length;

#[cfg(test)]
mod named_character_references_data_tests;
#[cfg(test)]
mod review_regression_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod validation;
