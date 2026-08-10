//! Gold self-validation, category coverage, and deterministic corruption
//! tests for the #166 candidate-independent context/order/resource gold
//! matrix.
//!
//! #166 implements no nested-context producer, so no producer execution or
//! producer-vs-gold comparison is claimed here; that begins in #167. These
//! tests validate the independent gold model and its own fixtures only.

use std::collections::BTreeSet;

use super::context_fixtures::independent_context_fixtures;
use super::context_gold::{
    ContextGoldDeclarationItem, ContextGoldFixture, ContextGoldGroup, ContextGoldKind,
    ContextGoldRecord, ContextGoldResourceExpectation, ContextGoldResourceKind,
    ContextGoldTermination, ContextGoldValidationError, validate_fixture,
};
use super::gold::GoldRange;

fn range(start: usize, end: usize) -> GoldRange {
    GoldRange::new(start, end)
}

fn fixture<'a>(fixtures: &'a [ContextGoldFixture], id: &str) -> &'a ContextGoldFixture {
    fixtures
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing context gold fixture {id}"))
}

#[test]
fn independent_context_fixture_count_is_exact() {
    let fixtures = independent_context_fixtures();
    assert_eq!(fixtures.len(), 16);
}

#[test]
fn independent_context_fixture_inventory_is_self_consistent_and_unique() {
    let fixtures = independent_context_fixtures();
    let mut ids = BTreeSet::new();
    for fixture in &fixtures {
        assert!(
            ids.insert(fixture.id),
            "duplicate context gold fixture id {}",
            fixture.id
        );
        validate_fixture(fixture).unwrap_or_else(|error| {
            panic!(
                "context gold fixture {} failed self-validation: {error:?}",
                fixture.id
            )
        });
    }
}

#[test]
fn every_required_category_has_an_explicit_fixture() {
    let fixtures = independent_context_fixtures();
    let ids: BTreeSet<&str> = fixtures.iter().map(|fixture| fixture.id).collect();

    let required = [
        "CSS-CONTEXT-TOP-LEVEL-SINGLE-DECLARATION-001",
        "CSS-CONTEXT-DECLARATION-ONLY-MULTIPLE-001",
        "CSS-CONTEXT-DECLARATION-CHILD-DECLARATION-001",
        "CSS-CONTEXT-MULTIPLE-CHILD-RULES-BETWEEN-RUNS-001",
        "CSS-CONTEXT-RECURSIVE-NESTING-DEPTH-THREE-001",
        "CSS-CONTEXT-TRUE-EOF-TERMINATION-001",
        "CSS-CONTEXT-UPSTREAM-INCOMPLETE-TERMINATION-001",
        "CSS-CONTEXT-PARSER-RESOURCE-LIMITED-TERMINATION-001",
        "CSS-CONTEXT-NESTED-TRUE-EOF-ANCESTRY-001",
        "CSS-CONTEXT-NESTED-UPSTREAM-INCOMPLETE-ANCESTRY-001",
        "CSS-CONTEXT-NESTED-PARSER-RESOURCE-LIMITED-ANCESTRY-001",
        "CSS-CONTEXT-DUPLICATE-SPELLING-DISTINCT-IDENTITIES-001",
        "CSS-CONTEXT-CUSTOM-PROPERTY-BRACE-VALUE-NOT-CONTEXT-001",
        "CSS-CONTEXT-MALFORMED-RECOVERY-NO-SYNTHETIC-ITEM-001",
        "CSS-CONTEXT-PEAK-CONTEXT-DEPTH-LIMIT-001",
        "CSS-CONTEXT-CONTEXT-RECORDS-LIMIT-001",
    ];
    for id in required {
        assert!(ids.contains(id), "missing required context fixture {id}");
    }
    assert_eq!(required.len(), 16);
}

#[test]
fn fixture_groups_cover_the_required_category_shapes() {
    let fixtures = independent_context_fixtures();
    let groups: BTreeSet<ContextGoldGroup> = fixtures.iter().map(|fixture| fixture.group).collect();
    assert!(groups.contains(&ContextGoldGroup::TopLevel));
    assert!(groups.contains(&ContextGoldGroup::DeclarationOnly));
    assert!(groups.contains(&ContextGoldGroup::MixedDeclarationsAndRules));
    assert!(groups.contains(&ContextGoldGroup::RecursiveNesting));
    assert!(groups.contains(&ContextGoldGroup::Termination));
    assert!(groups.contains(&ContextGoldGroup::DuplicateSpelling));
    assert!(groups.contains(&ContextGoldGroup::CustomProperty));
    assert!(groups.contains(&ContextGoldGroup::Malformed));
    assert!(groups.contains(&ContextGoldGroup::ResourceLimit));
}

#[test]
fn context_ids_are_contiguous_in_each_fixture() {
    for fixture in &independent_context_fixtures() {
        for (index, record) in fixture.contexts.iter().enumerate() {
            assert_eq!(
                record.id, index,
                "{}: context id must equal its vector index",
                fixture.id
            );
        }
    }
}

#[test]
fn parent_ids_refer_backward_in_every_fixture() {
    for fixture in &independent_context_fixtures() {
        for (index, record) in fixture.contexts.iter().enumerate() {
            if let Some(parent_id) = record.parent {
                assert!(
                    parent_id < index,
                    "{}: context {index} parent must refer to an earlier record",
                    fixture.id
                );
            }
        }
    }
}

/// Every retained context carries exactly one direct-item ordinal, scoped
/// either to its real parent or to the implicit stylesheet root when
/// `parent` is `None`. This checks sibling ordinal determinism/uniqueness
/// uniformly for both scopes.
#[test]
fn sibling_item_ordinals_are_deterministic_and_non_duplicated_in_every_scope() {
    for fixture in &independent_context_fixtures() {
        let mut last_ordinal: std::collections::BTreeMap<Option<usize>, usize> =
            std::collections::BTreeMap::new();
        for record in &fixture.contexts {
            let scope = record.parent;
            if let Some(&previous) = last_ordinal.get(&scope) {
                assert!(
                    record.item_ordinal > previous,
                    "{}: sibling item ordinals in scope {scope:?} must strictly increase",
                    fixture.id
                );
            }
            last_ordinal.insert(scope, record.item_ordinal);
        }
    }
}

#[test]
fn root_scoped_top_level_siblings_carry_distinct_increasing_ordinals() {
    let fixtures = independent_context_fixtures();
    let duplicate = fixture(
        &fixtures,
        "CSS-CONTEXT-DUPLICATE-SPELLING-DISTINCT-IDENTITIES-001",
    );
    let root_siblings: Vec<&ContextGoldRecord> = duplicate
        .contexts
        .iter()
        .filter(|record| record.parent.is_none())
        .collect();
    assert_eq!(
        root_siblings.len(),
        2,
        "fixture must exercise two implicit-root-scoped top-level contexts"
    );
    assert!(root_siblings[0].item_ordinal < root_siblings[1].item_ordinal);
}

#[test]
fn source_ranges_are_valid_utf8_boundaries_for_every_fixture() {
    use crate::{SourceId, SourceText};
    for fixture in &independent_context_fixtures() {
        let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
        for record in &fixture.contexts {
            assert!(
                source
                    .anchor(record.header.start, record.header.end)
                    .is_ok()
            );
            assert!(
                source
                    .anchor(record.block_opener.start, record.block_opener.end)
                    .is_ok()
            );
            assert!(source.anchor(record.body.start, record.body.end).is_ok());
            for declaration in &record.declarations {
                assert!(
                    source
                        .anchor(declaration.span.start, declaration.span.end)
                        .is_ok(),
                    "{}: declaration span must be valid source evidence",
                    fixture.id
                );
            }
        }
    }
}

#[test]
fn child_ranges_are_contained_by_expected_parent_body_ranges() {
    for fixture in &independent_context_fixtures() {
        for record in &fixture.contexts {
            if let Some(parent_id) = record.parent {
                let parent_body = fixture.contexts[parent_id].body;
                let extent_end = match record.termination {
                    ContextGoldTermination::AuthoredRightCurly(evidence)
                    | ContextGoldTermination::EndOfInput(evidence)
                    | ContextGoldTermination::UpstreamTokenizerIncomplete(evidence)
                    | ContextGoldTermination::ParserResourceLimit(evidence) => evidence.end,
                };
                assert!(
                    record.header.start >= parent_body.start && extent_end <= parent_body.end,
                    "{}: child context must be contained in parent body",
                    fixture.id
                );
            }
        }
    }
}

#[test]
fn termination_ranges_have_correct_empty_non_empty_shape() {
    for fixture in &independent_context_fixtures() {
        for record in &fixture.contexts {
            match record.termination {
                ContextGoldTermination::AuthoredRightCurly(evidence) => {
                    assert!(
                        !evidence.is_empty(),
                        "{}: authored `}}` must be non-empty",
                        fixture.id
                    );
                }
                ContextGoldTermination::EndOfInput(evidence)
                | ContextGoldTermination::UpstreamTokenizerIncomplete(evidence)
                | ContextGoldTermination::ParserResourceLimit(evidence) => {
                    assert!(
                        evidence.is_empty(),
                        "{}: partial/EOF terminal must be an empty point",
                        fixture.id
                    );
                }
            }
        }
    }
}

#[test]
fn declaration_run_ordinals_are_deterministic() {
    for fixture in &independent_context_fixtures() {
        for record in &fixture.contexts {
            let mut previous_run = None;
            for declaration in &record.declarations {
                if let Some(previous) = previous_run {
                    assert!(
                        declaration.run_ordinal >= previous,
                        "{}: run ordinals must never decrease",
                        fixture.id
                    );
                }
                previous_run = Some(declaration.run_ordinal);
            }
        }
    }
}

#[test]
fn declaration_item_ordinals_preserve_before_between_after_rule_ordering() {
    let fixtures = independent_context_fixtures();
    let outer = &fixture(&fixtures, "CSS-CONTEXT-DECLARATION-CHILD-DECLARATION-001").contexts[0];
    assert_eq!(outer.declarations.len(), 2);
    assert_eq!(outer.declarations[0].item_ordinal, 0);
    assert_eq!(outer.declarations[0].run_ordinal, 0);
    assert_eq!(outer.declarations[1].item_ordinal, 2);
    assert_eq!(outer.declarations[1].run_ordinal, 1);
    // Item ordinal 1 is the child rule itself: no declaration claims it.
    assert!(outer.declarations.iter().all(|d| d.item_ordinal != 1));
}

#[test]
fn malformed_recovery_fixture_child_ordinal_skips_the_abandoned_span() {
    let fixtures = independent_context_fixtures();
    let malformed = fixture(
        &fixtures,
        "CSS-CONTEXT-MALFORMED-RECOVERY-NO-SYNTHETIC-ITEM-001",
    );
    let child = &malformed.contexts[1];
    assert!(
        child.parent.is_some(),
        "child must have a parent relationship"
    );
    assert_eq!(
        child.item_ordinal, 0,
        "the malformed span before the child rule must not consume item ordinal 0"
    );
}

#[test]
fn custom_property_fixture_contains_no_child_context() {
    let fixtures = independent_context_fixtures();
    let custom_property = fixture(
        &fixtures,
        "CSS-CONTEXT-CUSTOM-PROPERTY-BRACE-VALUE-NOT-CONTEXT-001",
    );
    assert_eq!(custom_property.contexts.len(), 1);
    assert!(custom_property.contexts[0].parent.is_none());
    assert_eq!(custom_property.expected_context_record_count, 1);
}

#[test]
fn duplicate_spelling_fixture_assigns_distinct_identities_to_identical_text() {
    let fixtures = independent_context_fixtures();
    let duplicate = fixture(
        &fixtures,
        "CSS-CONTEXT-DUPLICATE-SPELLING-DISTINCT-IDENTITIES-001",
    );
    let source = duplicate.source;
    let x_in_a = &duplicate.contexts[1];
    let x_in_b = &duplicate.contexts[3];
    assert_ne!(x_in_a.id, x_in_b.id);
    assert_ne!(x_in_a.parent, x_in_b.parent);
    assert_eq!(
        &source[x_in_a.header.start..x_in_a.body.end + 1],
        &source[x_in_b.header.start..x_in_b.body.end + 1],
        "the two child contexts must share identical raw spelling"
    );
}

#[test]
fn resource_limit_fixtures_are_internally_coherent() {
    let fixtures = independent_context_fixtures();
    let depth_limit = fixture(&fixtures, "CSS-CONTEXT-PEAK-CONTEXT-DEPTH-LIMIT-001");
    let expectation = depth_limit
        .resource_expectation
        .expect("depth-limit fixture must document its resource expectation");
    assert_eq!(expectation.kind, ContextGoldResourceKind::PeakContextDepth);
    assert!(expectation.attempted > expectation.limit);
    // No ID gap: exactly the committed ancestors are retained.
    assert_eq!(
        depth_limit.contexts.len(),
        depth_limit.expected_context_record_count
    );

    let records_limit = fixture(&fixtures, "CSS-CONTEXT-CONTEXT-RECORDS-LIMIT-001");
    let expectation = records_limit
        .resource_expectation
        .expect("records-limit fixture must document its resource expectation");
    assert_eq!(expectation.kind, ContextGoldResourceKind::ContextRecords);
    assert!(expectation.attempted > expectation.limit);
    assert_eq!(
        records_limit.contexts.len(),
        records_limit.expected_context_record_count
    );
}

// -------------------------------------------------------------------
// Blocker 2 remediation: independent partial-ancestry gold. Each nested
// fixture proves parent identity/containment retention, a shared honest
// terminal for both active contexts, no fabricated `}`, and a correct
// achieved peak depth.
// -------------------------------------------------------------------

fn assert_nested_ancestry_shares_one_honest_terminal(
    fixtures: &[ContextGoldFixture],
    fixture_id: &str,
    expected_terminal: GoldRange,
) {
    let nested = fixture(fixtures, fixture_id);
    assert_eq!(nested.contexts.len(), 2);
    let outer = &nested.contexts[0];
    let inner = &nested.contexts[1];

    assert!(outer.parent.is_none());
    assert_eq!(inner.parent, Some(outer.id));

    let outer_terminal = outer.termination.evidence_range();
    let inner_terminal = inner.termination.evidence_range();
    assert_eq!(outer_terminal, expected_terminal);
    assert_eq!(inner_terminal, expected_terminal);
    assert_eq!(
        outer_terminal, inner_terminal,
        "both active ancestors must terminate at exactly the same honest point"
    );
    assert!(
        outer_terminal.is_empty(),
        "a partial terminal must be an empty point, never a fabricated `}}`"
    );

    // Child extent must be contained in the parent's retained body.
    assert!(inner.header.start >= outer.body.start);
    assert!(inner_terminal.end <= outer.body.end);

    assert_eq!(nested.expected_peak_context_depth, 2);
    assert_eq!(nested.expected_context_record_count, 2);
    validate_fixture(nested).expect("nested ancestry fixture must self-validate");
}

#[test]
fn nested_true_eof_ancestry_shares_the_true_source_end_terminal() {
    let fixtures = independent_context_fixtures();
    let nested = fixture(&fixtures, "CSS-CONTEXT-NESTED-TRUE-EOF-ANCESTRY-001");
    let expected = range(nested.byte_len, nested.byte_len);
    assert_nested_ancestry_shares_one_honest_terminal(
        &fixtures,
        "CSS-CONTEXT-NESTED-TRUE-EOF-ANCESTRY-001",
        expected,
    );
    for record in &nested.contexts {
        assert!(matches!(
            record.termination,
            ContextGoldTermination::EndOfInput(_)
        ));
    }
}

#[test]
fn nested_upstream_incomplete_ancestry_shares_one_bounded_terminal_short_of_source_end() {
    let fixtures = independent_context_fixtures();
    let nested = fixture(
        &fixtures,
        "CSS-CONTEXT-NESTED-UPSTREAM-INCOMPLETE-ANCESTRY-001",
    );
    let shared_terminal = nested.contexts[0].termination.evidence_range();
    assert!(
        shared_terminal.start < nested.byte_len,
        "an upstream-incomplete terminal, unlike true EOF, need not sit at source end"
    );
    assert_nested_ancestry_shares_one_honest_terminal(
        &fixtures,
        "CSS-CONTEXT-NESTED-UPSTREAM-INCOMPLETE-ANCESTRY-001",
        shared_terminal,
    );
    for record in &nested.contexts {
        assert!(matches!(
            record.termination,
            ContextGoldTermination::UpstreamTokenizerIncomplete(_)
        ));
    }
}

#[test]
fn nested_parser_resource_limited_ancestry_is_independent_of_the_depth_refusal_fixture() {
    let fixtures = independent_context_fixtures();
    let nested = fixture(
        &fixtures,
        "CSS-CONTEXT-NESTED-PARSER-RESOURCE-LIMITED-ANCESTRY-001",
    );
    // Distinct contract concern from the `PeakContextDepth` refusal fixture:
    // this fixture carries no resource-limit expectation of its own.
    assert!(nested.resource_expectation.is_none());
    let depth_limit_fixture = fixture(&fixtures, "CSS-CONTEXT-PEAK-CONTEXT-DEPTH-LIMIT-001");
    assert_ne!(nested.id, depth_limit_fixture.id);

    let shared_terminal = nested.contexts[0].termination.evidence_range();
    assert_nested_ancestry_shares_one_honest_terminal(
        &fixtures,
        "CSS-CONTEXT-NESTED-PARSER-RESOURCE-LIMITED-ANCESTRY-001",
        shared_terminal,
    );
    for record in &nested.contexts {
        assert!(matches!(
            record.termination,
            ContextGoldTermination::ParserResourceLimit(_)
        ));
    }
}

// -------------------------------------------------------------------
// Gold corruption matrix: the independent model must itself reject
// invalid id/parent/order/range/lifecycle/depth/count relationships,
// without any production-type reuse.
// -------------------------------------------------------------------

fn minimal_valid_record(id: usize) -> ContextGoldRecord {
    ContextGoldRecord {
        id,
        parent: None,
        nearest_qualified_ancestor: None,
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 3),
        termination: ContextGoldTermination::AuthoredRightCurly(range(3, 4)),
        declarations: vec![],
    }
}

fn minimal_fixture(contexts: Vec<ContextGoldRecord>) -> ContextGoldFixture {
    let count = contexts.len();
    ContextGoldFixture {
        id: "CSS-CONTEXT-CORRUPTION-TEST",
        group: ContextGoldGroup::TopLevel,
        source_id: 91_000,
        source: "a{x}",
        byte_len: 4,
        contexts,
        expected_context_record_count: count,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![],
    }
}

#[test]
fn corruption_wrong_context_id_for_vector_index_is_rejected() {
    let fixture = minimal_fixture(vec![minimal_valid_record(1)]);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::IdIndexMismatch)
    );
}

#[test]
fn corruption_missing_parent_is_rejected() {
    let mut child = minimal_valid_record(0);
    child.parent = Some(0);
    let fixture = minimal_fixture(vec![child]);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::ParentNotBefore)
    );
}

#[test]
fn corruption_forward_parent_is_rejected() {
    // `a` and `b` are each locally self-consistent `"a{x}"`-shaped records
    // (same source, distinct ids); `b` claims a parent id that has not yet
    // been retained (id 2 does not exist among the two records).
    let a = minimal_valid_record(0);
    let b = ContextGoldRecord {
        id: 1,
        parent: Some(2),
        nearest_qualified_ancestor: Some(2),
        ..minimal_valid_record(1)
    };
    let fixture = minimal_fixture(vec![a, b]);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::ParentNotBefore)
    );
}

#[test]
fn corruption_child_outside_parent_body_is_rejected() {
    let source = "a{b}c{}";
    let a = ContextGoldRecord {
        id: 0,
        parent: None,
        nearest_qualified_ancestor: None,
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 3),
        termination: ContextGoldTermination::AuthoredRightCurly(range(3, 4)),
        declarations: vec![],
    };
    // This child is locally valid (`c{}` at [4, 7)) but sits entirely
    // outside `a`'s retained body [2, 3).
    let escaping_child = ContextGoldRecord {
        id: 1,
        parent: Some(0),
        nearest_qualified_ancestor: Some(0),
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(4, 5),
        block_opener: range(5, 6),
        body: range(6, 6),
        termination: ContextGoldTermination::AuthoredRightCurly(range(6, 7)),
        declarations: vec![],
    };
    let fixture = ContextGoldFixture {
        id: "CSS-CONTEXT-CORRUPTION-TEST",
        group: ContextGoldGroup::TopLevel,
        source_id: 91_001,
        source,
        byte_len: source.len(),
        contexts: vec![a, escaping_child],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::ChildOutsideParentBody)
    );
}

#[test]
fn corruption_duplicate_sibling_item_ordinal_is_rejected() {
    let source = "a{b{}c{}}";
    let a = ContextGoldRecord {
        id: 0,
        parent: None,
        nearest_qualified_ancestor: None,
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 8),
        termination: ContextGoldTermination::AuthoredRightCurly(range(8, 9)),
        declarations: vec![],
    };
    let child1 = ContextGoldRecord {
        id: 1,
        parent: Some(0),
        nearest_qualified_ancestor: Some(0),
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(2, 3),
        block_opener: range(3, 4),
        body: range(4, 4),
        termination: ContextGoldTermination::AuthoredRightCurly(range(4, 5)),
        declarations: vec![],
    };
    let child2 = ContextGoldRecord {
        id: 2,
        parent: Some(0),
        nearest_qualified_ancestor: Some(0),
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(5, 6),
        block_opener: range(6, 7),
        body: range(7, 7),
        termination: ContextGoldTermination::AuthoredRightCurly(range(7, 8)),
        declarations: vec![],
    };
    let fixture = ContextGoldFixture {
        id: "CSS-CONTEXT-CORRUPTION-TEST",
        group: ContextGoldGroup::TopLevel,
        source_id: 91_002,
        source,
        byte_len: source.len(),
        contexts: vec![a, child1, child2],
        expected_context_record_count: 3,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::DuplicateSiblingItemOrdinal)
    );
}

#[test]
fn corruption_decreasing_sibling_item_ordinal_is_rejected() {
    let source = "a{b{}c{}}";
    let a = ContextGoldRecord {
        id: 0,
        parent: None,
        nearest_qualified_ancestor: None,
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 8),
        termination: ContextGoldTermination::AuthoredRightCurly(range(8, 9)),
        declarations: vec![],
    };
    let child1 = ContextGoldRecord {
        id: 1,
        parent: Some(0),
        nearest_qualified_ancestor: Some(0),
        item_ordinal: 1,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(2, 3),
        block_opener: range(3, 4),
        body: range(4, 4),
        termination: ContextGoldTermination::AuthoredRightCurly(range(4, 5)),
        declarations: vec![],
    };
    let child2 = ContextGoldRecord {
        id: 2,
        parent: Some(0),
        nearest_qualified_ancestor: Some(0),
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(5, 6),
        block_opener: range(6, 7),
        body: range(7, 7),
        termination: ContextGoldTermination::AuthoredRightCurly(range(7, 8)),
        declarations: vec![],
    };
    let fixture = ContextGoldFixture {
        id: "CSS-CONTEXT-CORRUPTION-TEST",
        group: ContextGoldGroup::TopLevel,
        source_id: 91_003,
        source,
        byte_len: source.len(),
        contexts: vec![a, child1, child2],
        expected_context_record_count: 3,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::SiblingOrdinalOrderViolation)
    );
}

#[test]
fn corruption_root_scoped_duplicate_sibling_item_ordinal_is_rejected() {
    let source = "a{}b{}";
    let first = ContextGoldRecord {
        id: 0,
        parent: None,
        nearest_qualified_ancestor: None,
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 2),
        termination: ContextGoldTermination::AuthoredRightCurly(range(2, 3)),
        declarations: vec![],
    };
    // Same implicit-root scope as `first` (both `parent = None`), claiming
    // the same ordinal.
    let second = ContextGoldRecord {
        id: 1,
        parent: None,
        nearest_qualified_ancestor: None,
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(3, 4),
        block_opener: range(4, 5),
        body: range(5, 5),
        termination: ContextGoldTermination::AuthoredRightCurly(range(5, 6)),
        declarations: vec![],
    };
    let fixture = ContextGoldFixture {
        id: "CSS-CONTEXT-CORRUPTION-TEST",
        group: ContextGoldGroup::TopLevel,
        source_id: 91_004,
        source,
        byte_len: source.len(),
        contexts: vec![first, second],
        expected_context_record_count: 2,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![],
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::DuplicateSiblingItemOrdinal)
    );
}

#[test]
fn corruption_root_scoped_sibling_source_order_violation_is_rejected() {
    // `second`'s ordinal (1) increases relative to `first`'s (0), but its
    // source extent fully overlaps `first`'s instead of following it:
    // increasing ordinals must not paper over reversed/overlapping source.
    let source = "a{}b{}";
    let first = ContextGoldRecord {
        id: 0,
        parent: None,
        nearest_qualified_ancestor: None,
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 2),
        termination: ContextGoldTermination::AuthoredRightCurly(range(2, 3)),
        declarations: vec![],
    };
    let second = ContextGoldRecord {
        id: 1,
        parent: None,
        nearest_qualified_ancestor: None,
        item_ordinal: 1,
        ..first.clone()
    };
    let fixture = ContextGoldFixture {
        id: "CSS-CONTEXT-CORRUPTION-TEST",
        group: ContextGoldGroup::TopLevel,
        source_id: 91_005,
        source,
        byte_len: source.len(),
        contexts: vec![first, second],
        expected_context_record_count: 2,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![],
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::SiblingSourceOrderViolation)
    );
}

#[test]
fn root_scoped_siblings_with_ordinal_gaps_are_accepted() {
    let source = "a{}b{}";
    let first = ContextGoldRecord {
        id: 0,
        parent: None,
        nearest_qualified_ancestor: None,
        item_ordinal: 0,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 2),
        termination: ContextGoldTermination::AuthoredRightCurly(range(2, 3)),
        declarations: vec![],
    };
    let second = ContextGoldRecord {
        id: 1,
        parent: None,
        nearest_qualified_ancestor: None,
        // Ordinal 1 presumed reserved for a future declaration item; the
        // gap is valid.
        item_ordinal: 2,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(3, 4),
        block_opener: range(4, 5),
        body: range(5, 5),
        termination: ContextGoldTermination::AuthoredRightCurly(range(5, 6)),
        declarations: vec![],
    };
    let fixture = ContextGoldFixture {
        id: "CSS-CONTEXT-CORRUPTION-TEST",
        group: ContextGoldGroup::TopLevel,
        source_id: 91_006,
        source,
        byte_len: source.len(),
        contexts: vec![first, second],
        expected_context_record_count: 2,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![],
    };
    assert_eq!(validate_fixture(&fixture), Ok(()));
}

#[test]
fn corruption_opener_not_exact_is_rejected() {
    let mut record = minimal_valid_record(0);
    record.block_opener = range(1, 2); // fragment is "[" in "a[x}"
    let fixture = ContextGoldFixture {
        source: "a[x}",
        ..minimal_fixture(vec![record])
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::OpenerNotExact)
    );
}

#[test]
fn corruption_authored_closer_not_exact_is_rejected() {
    let record = minimal_valid_record(0);
    let fixture = ContextGoldFixture {
        source: "a{x]",
        ..minimal_fixture(vec![record])
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::AuthoredCloserNotExact)
    );
}

#[test]
fn corruption_header_opener_boundary_mismatch_is_rejected() {
    let mut record = minimal_valid_record(0);
    record.header = range(0, 0);
    let fixture = minimal_fixture(vec![record]);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::HeaderOpenerBoundaryMismatch)
    );
}

#[test]
fn corruption_body_opener_boundary_mismatch_is_rejected() {
    let mut record = minimal_valid_record(0);
    record.body = range(3, 4);
    let fixture = ContextGoldFixture {
        source: "a{xy}",
        byte_len: 5,
        ..minimal_fixture(vec![record])
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::BodyOpenerBoundaryMismatch)
    );
}

#[test]
fn corruption_termination_boundary_mismatch_is_rejected() {
    let mut record = minimal_valid_record(0);
    record.termination = ContextGoldTermination::AuthoredRightCurly(range(4, 5));
    let fixture = ContextGoldFixture {
        source: "a{x?}",
        byte_len: 5,
        ..minimal_fixture(vec![record])
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::TerminationBoundaryMismatch)
    );
}

#[test]
fn corruption_non_empty_partial_terminal_is_rejected() {
    let mut record = minimal_valid_record(0);
    record.termination = ContextGoldTermination::EndOfInput(range(3, 4));
    let fixture = ContextGoldFixture {
        source: "a{xy",
        byte_len: 4,
        ..minimal_fixture(vec![record])
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::TerminationMustBeEmpty)
    );
}

#[test]
fn corruption_end_of_input_terminal_not_at_source_end_is_rejected() {
    let mut record = minimal_valid_record(0);
    record.body = range(2, 3);
    record.termination = ContextGoldTermination::EndOfInput(range(3, 3));
    let fixture = ContextGoldFixture {
        source: "a{x};",
        byte_len: 5,
        ..minimal_fixture(vec![record])
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::EndOfInputNotAtSourceEnd)
    );
}

#[test]
fn corruption_context_record_count_mismatch_is_rejected() {
    let record = minimal_valid_record(0);
    let fixture = ContextGoldFixture {
        expected_context_record_count: 2,
        ..minimal_fixture(vec![record])
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::ContextRecordCountMismatch)
    );
}

#[test]
fn corruption_peak_context_depth_mismatch_is_rejected() {
    let record = minimal_valid_record(0);
    let fixture = ContextGoldFixture {
        expected_peak_context_depth: 2,
        ..minimal_fixture(vec![record])
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::PeakContextDepthMismatch)
    );
}

#[test]
fn corruption_declaration_outside_body_is_rejected() {
    let mut record = minimal_valid_record(0);
    record.declarations = vec![ContextGoldDeclarationItem {
        item_ordinal: 0,
        run_ordinal: 0,
        span: range(0, 1),
    }];
    let fixture = minimal_fixture(vec![record]);
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::DeclarationOutsideBody)
    );
}

#[test]
fn corruption_duplicate_declaration_item_ordinal_is_rejected() {
    let source = "a{xy}";
    let mut record = minimal_valid_record(0);
    record.body = range(2, 4);
    record.termination = ContextGoldTermination::AuthoredRightCurly(range(4, 5));
    record.declarations = vec![
        ContextGoldDeclarationItem {
            item_ordinal: 0,
            run_ordinal: 0,
            span: range(2, 3),
        },
        ContextGoldDeclarationItem {
            item_ordinal: 0,
            run_ordinal: 0,
            span: range(3, 4),
        },
    ];
    let fixture = ContextGoldFixture {
        source,
        byte_len: source.len(),
        ..minimal_fixture(vec![record])
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::DuplicateDeclarationItemOrdinal)
    );
}

#[test]
fn corruption_resource_expectation_attempt_not_exceeding_limit_is_rejected() {
    let record = minimal_valid_record(0);
    let fixture = ContextGoldFixture {
        resource_expectation: Some(ContextGoldResourceExpectation {
            kind: ContextGoldResourceKind::ContextRecords,
            limit: 2,
            attempted: 2,
        }),
        ..minimal_fixture(vec![record])
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::ResourceExpectationAttemptDidNotExceedLimit)
    );
}
