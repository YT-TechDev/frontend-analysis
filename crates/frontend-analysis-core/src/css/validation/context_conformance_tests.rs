//! Production-vs-independent-gold conformance for the #167 context-aware
//! producer: executes the real tokenizer/parser against every context gold
//! fixture that has a defined execution recipe and compares the result
//! against the #166 candidate-independent context/order/resource model.
//!
//! Two of the sixteen fixtures intentionally represent a generic
//! parser-resource partial-ancestry state without prescribing a specific
//! resource trigger; they remain contract/gold self-validation only (see
//! `context_candidate::CONTRACT_ONLY_FIXTURE_IDS`) and are not claimed here
//! as ordinary production executions.

use super::context_candidate::{
    CONTRACT_ONLY_FIXTURE_IDS, assert_matches_context_gold, run_fixture,
};
use super::context_fixtures::independent_context_fixtures;

#[test]
fn production_parser_matches_every_source_driven_independent_context_fixture() {
    let fixtures = independent_context_fixtures();
    let mut executed = 0usize;
    for fixture in &fixtures {
        if CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id) {
            continue;
        }
        let result = run_fixture(fixture);
        assert_matches_context_gold(fixture, &result);
        executed += 1;
    }
    assert_eq!(
        executed,
        fixtures.len() - CONTRACT_ONLY_FIXTURE_IDS.len(),
        "every non-contract-only fixture must execute exactly once"
    );
}

#[test]
fn exactly_two_fixtures_remain_contract_only() {
    assert_eq!(CONTRACT_ONLY_FIXTURE_IDS.len(), 2);
    let fixtures = independent_context_fixtures();
    let ids: std::collections::BTreeSet<&str> = fixtures.iter().map(|fixture| fixture.id).collect();
    for id in CONTRACT_ONLY_FIXTURE_IDS {
        assert!(ids.contains(id), "unknown contract-only fixture id {id}");
    }
}

#[test]
fn production_context_execution_is_deterministic_across_distinct_source_ids() {
    let fixtures = independent_context_fixtures();
    for fixture in &fixtures {
        if CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id) {
            continue;
        }
        let first = run_fixture(fixture);
        let second = run_fixture(fixture);
        assert_eq!(
            super::parser_candidate::canonical_signature(&first),
            super::parser_candidate::canonical_signature(&second),
            "{}: repeated execution must be deterministic",
            fixture.id
        );
    }
}
