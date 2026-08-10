//! Production-vs-independent-gold conformance and inventory/category-
//! coverage contract tests for the #169 descriptor-context evidence layer
//! (validation phase authorization comment `5236795522`).

use super::descriptor_candidate::{
    DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS, assert_matches_descriptor_gold, run_fixture,
};
use super::descriptor_fixtures::independent_descriptor_fixtures;
use super::descriptor_gold::{ALL_CATEGORIES, validate_fixture};

#[test]
fn every_descriptor_fixture_is_internally_self_consistent() {
    let fixtures = independent_descriptor_fixtures();
    for fixture in &fixtures {
        assert_eq!(
            validate_fixture(fixture),
            Ok(()),
            "{}: independent descriptor gold fixture must be internally self-consistent",
            fixture.id
        );
    }
}

#[test]
fn descriptor_fixture_ids_are_unique() {
    let fixtures = independent_descriptor_fixtures();
    let ids: std::collections::BTreeSet<&str> = fixtures.iter().map(|fixture| fixture.id).collect();
    assert_eq!(
        ids.len(),
        fixtures.len(),
        "descriptor fixture ids must be unique"
    );
}

/// Every one of the 24 architecture categories (proposal comment
/// `5236345421` section 15) must be explicitly mapped to coverage: either a
/// dedicated fixture in this inventory, or one of the three explicitly
/// cross-cutting categories documented below and proved elsewhere in this
/// validation phase.
#[test]
fn all_twenty_four_categories_are_explicitly_mapped_to_coverage() {
    use super::descriptor_gold::DescriptorGoldCategory;

    // Category 16 (synthetic upstream-tokenizer-incomplete while a
    // descriptor context is already active) requires a genuine
    // tokenizer-produced lexical prefix, so it is covered by a dedicated
    // test in `descriptor_lifecycle_validation_tests.rs`, not this
    // inventory (see that module's doc comment).
    //
    // Category 22 (deterministic repeat / cross-`SourceId` equivalence) is
    // a cross-cutting property proved by
    // `descriptor_repeat_execution_is_deterministic` and
    // `descriptor_execution_is_equivalent_across_distinct_source_ids` below,
    // observed across this whole fixture inventory rather than by one
    // dedicated fixture.
    //
    // Category 24 (existing #167/#168 qualified/group behavior remains
    // green) is proved by the fixed 16-fixture (`context_conformance_tests`)
    // and 28-fixture (`group_context_contract_tests`) suites continuing to
    // pass unmodified in the same full validation run, not by a descriptor
    // fixture.
    const COVERED_ELSEWHERE: [DescriptorGoldCategory; 3] = [
        DescriptorGoldCategory::UpstreamIncompleteWhileDescriptorActive,
        DescriptorGoldCategory::DeterministicRepeatAndCrossSourceId,
        DescriptorGoldCategory::QualifiedGroupBehaviorRemainsGreen,
    ];

    let fixtures = independent_descriptor_fixtures();
    let mut covered: std::collections::BTreeSet<DescriptorGoldCategory> =
        COVERED_ELSEWHERE.into_iter().collect();
    for fixture in &fixtures {
        covered.extend(fixture.categories.iter().copied());
    }

    for category in ALL_CATEGORIES {
        assert!(
            covered.contains(&category),
            "category {category:?} is not mapped to any descriptor fixture or documented cross-cutting proof"
        );
    }
}

#[test]
fn production_parser_matches_every_source_driven_independent_descriptor_fixture() {
    let fixtures = independent_descriptor_fixtures();
    let mut executed = 0usize;
    for fixture in &fixtures {
        if DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id) {
            continue;
        }
        let result = run_fixture(fixture);
        assert_matches_descriptor_gold(fixture, &result);
        executed += 1;
    }
    assert_eq!(
        executed,
        fixtures.len() - DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS.len(),
        "every non-contract-only descriptor fixture must execute exactly once"
    );
}

#[test]
fn exactly_one_descriptor_fixture_remains_contract_only() {
    assert_eq!(DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS.len(), 1);
    let fixtures = independent_descriptor_fixtures();
    let ids: std::collections::BTreeSet<&str> = fixtures.iter().map(|fixture| fixture.id).collect();
    for id in DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS {
        assert!(ids.contains(id), "unknown contract-only fixture id {id}");
    }
}

/// Category 22 (part 1): repeated execution of the same fixture source
/// under the same `SourceId` is byte-for-byte deterministic.
#[test]
fn descriptor_repeat_execution_is_deterministic() {
    let fixtures = independent_descriptor_fixtures();
    for fixture in &fixtures {
        if DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id) {
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

/// Category 22 (part 2): the same source text produces an identical
/// canonical signature under two distinct `SourceId`s -- the signature is
/// entirely offset/shape-based and carries no `SourceId`-dependent content,
/// so descriptor context/occurrence evidence (at_keyword, descriptor
/// property name, placement, ranges) must observe cross-`SourceId`
/// equivalence exactly like every other #166/#167/#168 evidence kind.
#[test]
fn descriptor_execution_is_equivalent_across_distinct_source_ids() {
    use crate::css::parser::producer::run;
    use crate::css::tokenizer::producer::run as run_tokenizer;
    use crate::{SourceId, SourceText};

    let fixtures = independent_descriptor_fixtures();
    for fixture in &fixtures {
        if DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id) {
            continue;
        }
        let tokenizer_limits = crate::css::tokenizer::resource::CssTokenizerLimits::new(
            1 << 20,
            1 << 20,
            1 << 16,
            1 << 16,
            1 << 20,
            1 << 20,
        )
        .unwrap();
        let parser_limits = crate::css::parser::resource::CssParserLimits::new(
            1 << 20,
            1 << 12,
            1 << 12,
            1 << 16,
            1 << 16,
            1 << 16,
            1 << 16,
            1 << 16,
            1 << 16,
        )
        .unwrap();

        if let Some(expectation) = fixture.resource_expectation {
            // Resource-refusal fixtures use a tight limit tailored to
            // `fixture.source_id`'s own run via `run_fixture`; cross-SourceId
            // equivalence for those is already exercised identically by
            // `descriptor_repeat_execution_is_deterministic` re-running
            // `run_fixture` (which is `SourceId`-parametric internally).
            // Here we additionally confirm the two alternate `SourceId`s
            // agree with each other under the same tight limit.
            let alt_a = SourceText::new(
                SourceId::new(fixture.source_id + 500_000),
                fixture.source.to_owned(),
            );
            let alt_b = SourceText::new(
                SourceId::new(fixture.source_id + 600_000),
                fixture.source.to_owned(),
            );
            let limits = tight_limits_for(expectation);
            let tokenizer_a = run_tokenizer(&alt_a, tokenizer_limits).unwrap();
            let tokenizer_b = run_tokenizer(&alt_b, tokenizer_limits).unwrap();
            let result_a = run(&alt_a, tokenizer_a, limits).unwrap();
            let result_b = run(&alt_b, tokenizer_b, limits).unwrap();
            assert_eq!(
                super::parser_candidate::canonical_signature(&result_a),
                super::parser_candidate::canonical_signature(&result_b),
                "{}: cross-SourceId execution must be equivalent",
                fixture.id
            );
            continue;
        }

        let alt_a = SourceText::new(
            SourceId::new(fixture.source_id + 500_000),
            fixture.source.to_owned(),
        );
        let alt_b = SourceText::new(
            SourceId::new(fixture.source_id + 600_000),
            fixture.source.to_owned(),
        );
        let tokenizer_a = run_tokenizer(&alt_a, tokenizer_limits).unwrap();
        let tokenizer_b = run_tokenizer(&alt_b, tokenizer_limits).unwrap();
        let result_a = run(&alt_a, tokenizer_a, parser_limits).unwrap();
        let result_b = run(&alt_b, tokenizer_b, parser_limits).unwrap();
        assert_eq!(
            super::parser_candidate::canonical_signature(&result_a),
            super::parser_candidate::canonical_signature(&result_b),
            "{}: cross-SourceId execution must be equivalent",
            fixture.id
        );
    }
}

fn tight_limits_for(
    expectation: super::descriptor_gold::DescriptorGoldResourceExpectation,
) -> crate::css::parser::resource::CssParserLimits {
    use super::descriptor_gold::DescriptorGoldResourceKind;
    use crate::css::parser::resource::CssParserLimits;

    let mut max_algorithm_steps = 1 << 20;
    let mut max_peak_context_depth = 1 << 12;
    let mut max_declaration_occurrences = 1 << 16;
    let mut max_unsupported_regions = 1 << 16;
    let mut max_context_records = 1 << 16;
    match expectation.kind {
        DescriptorGoldResourceKind::AlgorithmSteps => {
            max_algorithm_steps = expectation.limit.max(1)
        }
        DescriptorGoldResourceKind::PeakContextDepth => max_peak_context_depth = expectation.limit,
        DescriptorGoldResourceKind::ContextRecords => max_context_records = expectation.limit,
        DescriptorGoldResourceKind::DeclarationOccurrences => {
            max_declaration_occurrences = expectation.limit
        }
        DescriptorGoldResourceKind::UnsupportedRegions => {
            max_unsupported_regions = expectation.limit
        }
    }
    CssParserLimits::new(
        max_algorithm_steps,
        1 << 12,
        max_peak_context_depth,
        max_declaration_occurrences,
        1 << 16,
        1 << 16,
        max_unsupported_regions,
        1 << 16,
        max_context_records,
    )
    .unwrap()
}

/// Descriptor fixture ids must not collide with the fixed #166/#168
/// inventories.
#[test]
fn descriptor_fixture_inventory_is_disjoint_from_fixed_inventories() {
    let descriptor_ids: std::collections::BTreeSet<&str> = independent_descriptor_fixtures()
        .iter()
        .map(|fixture| fixture.id)
        .collect();
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
    assert!(
        descriptor_ids.is_disjoint(&context_ids),
        "descriptor fixture ids must not collide with the fixed #166 inventory"
    );
    assert!(
        descriptor_ids.is_disjoint(&group_ids),
        "descriptor fixture ids must not collide with the fixed #168 inventory"
    );
    assert_eq!(
        context_ids.len(),
        16,
        "fixed #166 inventory must remain exactly 16"
    );
    assert_eq!(
        group_ids.len(),
        28,
        "fixed #168 inventory must remain exactly 28"
    );
}
