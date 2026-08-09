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
    ContextGoldParent, ContextGoldRecord, ContextGoldResourceExpectation, ContextGoldResourceKind,
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
    assert_eq!(fixtures.len(), 13);
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
        "CSS-CONTEXT-DUPLICATE-SPELLING-DISTINCT-IDENTITIES-001",
        "CSS-CONTEXT-CUSTOM-PROPERTY-BRACE-VALUE-NOT-CONTEXT-001",
        "CSS-CONTEXT-MALFORMED-RECOVERY-NO-SYNTHETIC-ITEM-001",
        "CSS-CONTEXT-PEAK-CONTEXT-DEPTH-LIMIT-001",
        "CSS-CONTEXT-CONTEXT-RECORDS-LIMIT-001",
    ];
    for id in required {
        assert!(ids.contains(id), "missing required context fixture {id}");
    }
    assert_eq!(required.len(), 13);
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
            if let Some(parent) = record.parent {
                assert!(
                    parent.parent_id < index,
                    "{}: context {index} parent must refer to an earlier record",
                    fixture.id
                );
            }
        }
    }
}

#[test]
fn parent_item_ordinals_are_deterministic_and_non_duplicated() {
    for fixture in &independent_context_fixtures() {
        let mut last_ordinal: std::collections::BTreeMap<usize, usize> =
            std::collections::BTreeMap::new();
        for record in &fixture.contexts {
            if let Some(parent) = record.parent {
                if let Some(&previous) = last_ordinal.get(&parent.parent_id) {
                    assert!(
                        parent.item_ordinal > previous,
                        "{}: sibling item ordinals under parent {} must strictly increase",
                        fixture.id,
                        parent.parent_id
                    );
                }
                last_ordinal.insert(parent.parent_id, parent.item_ordinal);
            }
        }
    }
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
            if let Some(parent) = record.parent {
                let parent_body = fixture.contexts[parent.parent_id].body;
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
    let parent = child.parent.expect("child must have a parent relationship");
    assert_eq!(
        parent.item_ordinal, 0,
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
// Gold corruption matrix: the independent model must itself reject
// invalid id/parent/order/range/lifecycle/depth/count relationships,
// without any production-type reuse.
// -------------------------------------------------------------------

fn minimal_valid_record(id: usize) -> ContextGoldRecord {
    ContextGoldRecord {
        id,
        parent: None,
        kind: ContextGoldKind::QualifiedRuleBlock,
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
    child.parent = Some(ContextGoldParent {
        parent_id: 0,
        item_ordinal: 0,
    });
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
        parent: Some(ContextGoldParent {
            parent_id: 2,
            item_ordinal: 0,
        }),
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
        kind: ContextGoldKind::QualifiedRuleBlock,
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
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
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
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 8),
        termination: ContextGoldTermination::AuthoredRightCurly(range(8, 9)),
        declarations: vec![],
    };
    let child1 = ContextGoldRecord {
        id: 1,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(2, 3),
        block_opener: range(3, 4),
        body: range(4, 4),
        termination: ContextGoldTermination::AuthoredRightCurly(range(4, 5)),
        declarations: vec![],
    };
    let child2 = ContextGoldRecord {
        id: 2,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
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
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::DuplicateChildItemOrdinal)
    );
}

#[test]
fn corruption_decreasing_sibling_item_ordinal_is_rejected() {
    let source = "a{b{}c{}}";
    let a = ContextGoldRecord {
        id: 0,
        parent: None,
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 8),
        termination: ContextGoldTermination::AuthoredRightCurly(range(8, 9)),
        declarations: vec![],
    };
    let child1 = ContextGoldRecord {
        id: 1,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 1,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(2, 3),
        block_opener: range(3, 4),
        body: range(4, 4),
        termination: ContextGoldTermination::AuthoredRightCurly(range(4, 5)),
        declarations: vec![],
    };
    let child2 = ContextGoldRecord {
        id: 2,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
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
    };
    assert_eq!(
        validate_fixture(&fixture),
        Err(ContextGoldValidationError::ChildOrderViolation)
    );
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
