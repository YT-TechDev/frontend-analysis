pub(crate) mod diagnostic;
pub(crate) mod producer;
pub(crate) mod resource;
pub(crate) mod result;

/// The #392/#393 deterministic WHATWG named character-reference foundation.
///
/// TC-S10 promotes this mechanically generated module from test-only to
/// production so the runtime matcher consumes the one complete table of 2231
/// entries rather than any second, hand-maintained, or partial copy. The
/// generated source itself is unchanged.
///
/// Its provenance and derived-count constants exist to be asserted by the
/// generated data's own tests; production consumes only the table and the
/// maximum identifier length. Allowing dead code for non-test builds keeps
/// the remaining generated constants intact instead of editing generated
/// output to satisfy a lint.
#[cfg_attr(not(test), allow(dead_code))]
mod named_character_references_generated;

#[cfg(test)]
mod named_character_references_data_tests;
#[cfg(test)]
mod review_regression_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod validation;
