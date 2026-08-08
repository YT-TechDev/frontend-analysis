//! #136 production tokenizer wiring against the #135 validation foundation.
//!
//! Gold remains independent: fixtures are authored in `fixtures.rs`/`gold.rs`
//! without reference to production output, and this module only reads
//! production results to compare them against that already-fixed gold.

use super::candidate::{
    assert_matches_gold, canonical_signature, collect_preprocessed_units, run_fixture,
};
use super::fixtures::specification_fixtures;
use super::generated::{assert_deterministic_replay, generated_inputs};
use super::gold::GoldGroup;

use crate::css::tokenizer::producer::run;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::{SourceId, SourceText};

fn generous_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
}

#[test]
fn production_tokenizer_matches_every_normative_specification_fixture() {
    let fixtures = specification_fixtures();
    assert_eq!(fixtures.len(), 36);

    for fixture in &fixtures {
        let result = run_fixture(fixture, generous_limits());
        assert_matches_gold(fixture, &result);
    }
}

#[test]
fn production_lazy_cursor_matches_input_preprocessing_gold_units() {
    let fixtures = specification_fixtures();

    for fixture in fixtures
        .iter()
        .filter(|fixture| fixture.group == GoldGroup::InputPreprocessing)
    {
        let start = fixture.leading_bom.map_or(0, |range| range.end);
        let actual = collect_preprocessed_units(fixture.source, start);
        let expected: Vec<(char, usize, usize)> = fixture
            .preprocessed_units
            .iter()
            .map(|unit| (unit.semantic, unit.raw.start, unit.raw.end))
            .collect();
        assert_eq!(actual, expected, "{}: preprocessed units", fixture.id);
    }
}

#[test]
fn production_tokenizer_terminates_deterministically_over_generated_corpus() {
    let inputs = generated_inputs();
    assert_eq!(inputs.len(), 4_096);

    for source in &inputs {
        let text = SourceText::new(SourceId::new(0), source.clone());
        let result = run(&text, generous_limits()).unwrap_or_else(|error| {
            panic!("generated input {source:?} produced a run error: {error:?}")
        });
        // Construction of `CssTokenizerRunResult` already enforces source
        // coverage, ordering, and lifecycle invariants; reaching `Ok` here
        // proves those invariants held for this input under generous limits.
        drop(result);
    }

    assert_deterministic_replay(&inputs, |source| {
        let text = SourceText::new(SourceId::new(0), source.to_owned());
        let result = run(&text, generous_limits()).expect("generated input must tokenize cleanly");
        canonical_signature(&result)
    });
}
