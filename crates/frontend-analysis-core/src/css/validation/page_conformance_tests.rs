//! Production-vs-independent-gold conformance and inventory/category-
//! coverage contract tests for the #170 `@page`/page-margin evidence layer.

use super::page_candidate::{
    PAGE_CONTRACT_ONLY_FIXTURE_IDS, assert_matches_page_gold, run_fixture,
};
use super::page_fixtures::independent_page_fixtures;
use super::page_gold::{ALL_CATEGORIES, validate_fixture};

#[test]
fn every_page_fixture_is_internally_self_consistent() {
    let fixtures = independent_page_fixtures();
    for fixture in &fixtures {
        assert_eq!(
            validate_fixture(fixture),
            Ok(()),
            "{}: independent page gold fixture must be internally self-consistent",
            fixture.id
        );
    }
}

#[test]
fn page_fixture_ids_are_unique() {
    let fixtures = independent_page_fixtures();
    let ids: std::collections::BTreeSet<&str> = fixtures.iter().map(|fixture| fixture.id).collect();
    assert_eq!(ids.len(), fixtures.len(), "page fixture ids must be unique");
}

/// Every one of the 40 architecture categories (section 14) must be
/// explicitly mapped to coverage: either a dedicated fixture in this
/// inventory, or one of the documented cross-cutting/dedicated-test
/// categories proved elsewhere.
#[test]
fn all_forty_categories_are_explicitly_mapped_to_coverage() {
    use super::page_gold::PageGoldCategory;

    // Categories 32/33 (synthetic upstream-tokenizer-incomplete while a
    // Page/Margin context is already active) and 34/35 (a generic
    // parser-resource stop reached only after a Page/Margin context has
    // already been entered) require either a genuine tokenizer-produced
    // lexical prefix or a non-deterministic-exact-step resource stop, so
    // they are covered by dedicated tests in
    // `page_lifecycle_validation_tests.rs`, not this inventory (see that
    // module's doc comment).
    //
    // Category 40 (deterministic repeat / cross-`SourceId` equivalence) is a
    // cross-cutting property proved by `page_repeat_execution_is_deterministic`
    // and `page_execution_is_equivalent_across_distinct_source_ids` below,
    // observed across this whole fixture inventory rather than by one
    // dedicated fixture.
    const COVERED_ELSEWHERE: [PageGoldCategory; 5] = [
        PageGoldCategory::UpstreamIncompleteWhilePageActive,
        PageGoldCategory::UpstreamIncompleteWhileMarginActive,
        PageGoldCategory::ParserResourceStopAfterPageEntry,
        PageGoldCategory::ParserResourceStopAfterMarginEntry,
        PageGoldCategory::DeterministicRepeatAndCrossSourceId,
    ];

    let fixtures = independent_page_fixtures();
    let mut covered: std::collections::BTreeSet<PageGoldCategory> =
        COVERED_ELSEWHERE.into_iter().collect();
    for fixture in &fixtures {
        covered.extend(fixture.categories.iter().copied());
    }

    for category in ALL_CATEGORIES {
        assert!(
            covered.contains(&category),
            "category {category:?} is not mapped to any page fixture or documented cross-cutting proof"
        );
    }
}

#[test]
fn production_parser_matches_every_source_driven_independent_page_fixture() {
    let fixtures = independent_page_fixtures();
    let mut executed = 0usize;
    for fixture in &fixtures {
        if PAGE_CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id) {
            continue;
        }
        let result = run_fixture(fixture);
        assert_matches_page_gold(fixture, &result);
        executed += 1;
    }
    assert_eq!(
        executed,
        fixtures.len() - PAGE_CONTRACT_ONLY_FIXTURE_IDS.len(),
        "every non-contract-only page fixture must execute exactly once"
    );
}

/// Category 40 (part 1): repeated execution of the same fixture source under
/// the same `SourceId` is byte-for-byte deterministic.
#[test]
fn page_repeat_execution_is_deterministic() {
    let fixtures = independent_page_fixtures();
    for fixture in &fixtures {
        if PAGE_CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id) {
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

/// Category 40 (part 2): the same source text produces an identical
/// canonical signature under two distinct `SourceId`s.
#[test]
fn page_execution_is_equivalent_across_distinct_source_ids() {
    use crate::css::parser::producer::run;
    use crate::css::tokenizer::producer::run as run_tokenizer;
    use crate::{SourceId, SourceText};

    let fixtures = independent_page_fixtures();
    for fixture in &fixtures {
        if PAGE_CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id) {
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

        if let Some(expectation) = fixture.resource_expectation {
            let alt_a = SourceText::new(
                SourceId::new(fixture.source_id + 500_000),
                fixture.source.to_owned(),
            );
            let alt_b = SourceText::new(
                SourceId::new(fixture.source_id + 600_000),
                fixture.source.to_owned(),
            );
            let limits = super::page_candidate::tight_limits_for(expectation);
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

/// Page fixture ids must not collide with the fixed #166/#168/#169
/// inventories.
#[test]
fn page_fixture_inventory_is_disjoint_from_other_inventories() {
    let page_ids: std::collections::BTreeSet<&str> = independent_page_fixtures()
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
    let descriptor_ids: std::collections::BTreeSet<&str> =
        super::descriptor_fixtures::independent_descriptor_fixtures()
            .iter()
            .map(|fixture| fixture.id)
            .collect();
    assert!(
        page_ids.is_disjoint(&context_ids),
        "page fixture ids must not collide with the fixed #166 inventory"
    );
    assert!(
        page_ids.is_disjoint(&group_ids),
        "page fixture ids must not collide with the fixed #168 inventory"
    );
    assert!(
        page_ids.is_disjoint(&descriptor_ids),
        "page fixture ids must not collide with the #169 inventory"
    );
}
