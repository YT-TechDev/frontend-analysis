mod adversarial;
mod diagnostics;
mod helpers;
mod preprocessing;
mod supported;
#[rustfmt::skip]
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
