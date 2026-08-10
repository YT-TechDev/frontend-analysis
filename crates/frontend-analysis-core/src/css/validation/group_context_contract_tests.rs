//! Contract self-validation and production-vs-independent-gold conformance
//! for the #168 nested group-rule fixtures (`group_context_fixtures.rs`),
//! kept structurally separate from the fixed #166 16-fixture inventory and
//! its own conformance suite (`context_conformance_tests.rs`).

use super::context_candidate::{assert_matches_context_gold, run_fixture};
use super::context_gold::validate_fixture;
use super::group_context_fixtures::independent_group_context_fixtures;

#[test]
fn every_group_fixture_is_internally_self_consistent() {
    let fixtures = independent_group_context_fixtures();
    for fixture in &fixtures {
        assert_eq!(
            validate_fixture(fixture),
            Ok(()),
            "{}: independent group gold fixture must be internally self-consistent",
            fixture.id
        );
    }
}

#[test]
fn production_parser_matches_every_group_context_fixture() {
    let fixtures = independent_group_context_fixtures();
    for fixture in &fixtures {
        let result = run_fixture(fixture);
        assert_matches_context_gold(fixture, &result);
    }
}

#[test]
fn production_group_context_execution_is_deterministic_across_distinct_source_ids() {
    let fixtures = independent_group_context_fixtures();
    for fixture in &fixtures {
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

#[test]
fn group_fixture_inventory_is_separate_from_the_fixed_sixteen_fixture_inventory() {
    let group_ids: std::collections::BTreeSet<&str> = independent_group_context_fixtures()
        .iter()
        .map(|fixture| fixture.id)
        .collect();
    let context_ids: std::collections::BTreeSet<&str> =
        super::context_fixtures::independent_context_fixtures()
            .iter()
            .map(|fixture| fixture.id)
            .collect();
    assert!(
        group_ids.is_disjoint(&context_ids),
        "group-rule fixture ids must not collide with the fixed #166 inventory"
    );
    assert_eq!(context_ids.len(), 16);
    assert_eq!(group_ids.len(), 28);
}

/// All five supported group kinds have at least one positive production
/// fixture.
#[test]
fn every_supported_group_kind_has_a_positive_fixture() {
    use super::context_gold::{ContextGoldGroupRuleKind, ContextGoldKind};

    let fixtures = independent_group_context_fixtures();
    let mut seen = std::collections::BTreeSet::new();
    for fixture in &fixtures {
        for context in &fixture.contexts {
            if let ContextGoldKind::GroupRuleBlock(kind) = context.kind {
                seen.insert(kind);
            }
        }
    }
    for kind in [
        ContextGoldGroupRuleKind::Media,
        ContextGoldGroupRuleKind::Supports,
        ContextGoldGroupRuleKind::Container,
        ContextGoldGroupRuleKind::Layer,
        ContextGoldGroupRuleKind::Scope,
    ] {
        assert!(
            seen.contains(&kind),
            "missing a positive group fixture for {kind:?}"
        );
    }
}
