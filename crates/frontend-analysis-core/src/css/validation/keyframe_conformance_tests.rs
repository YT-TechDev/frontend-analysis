//! Production-vs-independent-gold conformance for #171 keyframes evidence.

use super::keyframe_candidate::{assert_matches_keyframe_gold, run_fixture, run_source};
use super::keyframe_fixtures::independent_keyframe_fixtures;
use super::keyframe_gold::{ALL_CATEGORIES, KeyframeGoldCategory, validate_fixture};

#[test]
fn every_keyframe_fixture_is_internally_self_consistent() {
    for fixture in independent_keyframe_fixtures() {
        assert_eq!(
            validate_fixture(&fixture),
            Ok(()),
            "{}: independent keyframe gold must be internally coherent",
            fixture.id
        );
    }
}

#[test]
fn keyframe_fixture_ids_are_unique_and_disjoint() {
    let fixtures = independent_keyframe_fixtures();
    let ids: std::collections::BTreeSet<&str> = fixtures.iter().map(|fixture| fixture.id).collect();
    assert_eq!(ids.len(), fixtures.len(), "keyframe fixture ids must be unique");

    let context_ids: std::collections::BTreeSet<&str> =
        super::context_fixtures::independent_context_fixtures()
            .iter()
            .map(|fixture| fixture.id)
            .collect();
    let group_ids: std::collections::BTreeSet<&str> =
        super::group_context_fixtures::independent_group_context_fixtures()
            .iter()
            .map(|fixture| fixture.id)
            .collect();
    let descriptor_ids: std::collections::BTreeSet<&str> =
        super::descriptor_fixtures::independent_descriptor_fixtures()
            .iter()
            .map(|fixture| fixture.id)
            .collect();
    let page_ids: std::collections::BTreeSet<&str> =
        super::page_fixtures::independent_page_fixtures()
            .iter()
            .map(|fixture| fixture.id)
            .collect();

    assert!(ids.is_disjoint(&context_ids));
    assert!(ids.is_disjoint(&group_ids));
    assert!(ids.is_disjoint(&descriptor_ids));
    assert!(ids.is_disjoint(&page_ids));
}

#[test]
fn every_keyframe_architecture_category_has_an_explicit_proof() {
    const COVERED_ELSEWHERE: [KeyframeGoldCategory; 3] = [
        KeyframeGoldCategory::UpstreamIncompleteLifecycle,
        KeyframeGoldCategory::ParserResourceLifecycle,
        KeyframeGoldCategory::DeterministicRepeatAndCrossSourceId,
    ];

    let mut covered: std::collections::BTreeSet<KeyframeGoldCategory> =
        COVERED_ELSEWHERE.into_iter().collect();
    for fixture in independent_keyframe_fixtures() {
        covered.extend(fixture.categories);
    }
    for category in ALL_CATEGORIES {
        assert!(
            covered.contains(&category),
            "keyframe category {category:?} lacks an explicit proof"
        );
    }
}

#[test]
fn production_parser_matches_every_independent_keyframe_fixture() {
    for fixture in independent_keyframe_fixtures() {
        let result = run_fixture(&fixture);
        assert_matches_keyframe_gold(&fixture, &result);
    }
}

#[test]
fn keyframe_execution_is_repeat_deterministic() {
    for fixture in independent_keyframe_fixtures() {
        let first = run_fixture(&fixture);
        let second = run_fixture(&fixture);
        assert_matches_keyframe_gold(&fixture, &first);
        assert_matches_keyframe_gold(&fixture, &second);
        assert_eq!(
            super::parser_candidate::canonical_signature(&first),
            super::parser_candidate::canonical_signature(&second),
            "{}: repeated keyframe execution must be deterministic",
            fixture.id
        );
    }
}

#[test]
fn keyframe_execution_is_equivalent_across_distinct_source_ids() {
    for fixture in independent_keyframe_fixtures() {
        let first = run_source(fixture.source_id + 700_000, fixture.source);
        let second = run_source(fixture.source_id + 800_000, fixture.source);
        assert_eq!(
            super::parser_candidate::canonical_signature(&first),
            super::parser_candidate::canonical_signature(&second),
            "{}: keyframe evidence must be SourceId-independent apart from provenance identity",
            fixture.id
        );
    }
}
