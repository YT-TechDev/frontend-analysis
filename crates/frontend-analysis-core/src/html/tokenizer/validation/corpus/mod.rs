mod adversarial;
mod diagnostics;
mod helpers;
mod preprocessing;
mod regressions;
mod supported;
mod transition_audit;
mod unsupported_resources;

use super::fixture::HtmlTokenizerFixture;

pub(super) fn initial_corpus() -> Vec<HtmlTokenizerFixture> {
    let mut fixtures = Vec::with_capacity(72);
    preprocessing::add_preprocessing(&mut fixtures);
    supported::add_supported_tokens(&mut fixtures);
    diagnostics::add_diagnostics(&mut fixtures);
    unsupported_resources::add_unsupported(&mut fixtures);
    unsupported_resources::add_resources(&mut fixtures);
    adversarial::add_adversarial(&mut fixtures);
    fixtures
}

/// Supplemental candidate-independent `REG-113-*` regression fixtures. This
/// is a separate corpus from [`initial_corpus`]: it does not count toward,
/// renumber, or alter the immutable 72-fixture initial inventory.
pub(super) fn supplemental_regression_corpus() -> Vec<HtmlTokenizerFixture> {
    let mut fixtures = Vec::with_capacity(3);
    regressions::add_regressions(&mut fixtures);
    fixtures
}

/// Deterministic concatenation of the initial 72-fixture corpus followed by
/// the supplemental `REG-` regression corpus, for future candidate
/// validation that wants all authoritative candidate-independent gold
/// without redefining [`initial_corpus`].
pub(super) fn all_candidate_independent_corpus() -> Vec<HtmlTokenizerFixture> {
    let mut fixtures = initial_corpus();
    fixtures.extend(supplemental_regression_corpus());
    fixtures
}
