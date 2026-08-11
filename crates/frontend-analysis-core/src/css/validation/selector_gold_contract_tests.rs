use std::collections::BTreeSet;

use super::selector_gold::{
    SelectorGoldGroup, SelectorGoldLifecycle, SelectorGoldMode, SelectorGoldOutcome, fixtures,
};

#[test]
fn selector_gold_ids_are_unique_and_ranges_are_valid_utf8_boundaries() {
    let fixtures = fixtures();
    let mut ids = BTreeSet::new();

    for fixture in &fixtures {
        assert!(
            ids.insert(fixture.id),
            "duplicate selector gold id: {}",
            fixture.id
        );
        assert!(fixture.header.start <= fixture.header.end);
        assert!(fixture.header.end <= fixture.source.len());
        assert!(fixture.source.is_char_boundary(fixture.header.start));
        assert!(fixture.source.is_char_boundary(fixture.header.end));
        if let Some(subject) = fixture.subject {
            assert!(subject.start <= subject.end);
            assert!(subject.start >= fixture.header.start);
            assert!(subject.end <= fixture.header.end);
            assert!(fixture.source.is_char_boundary(subject.start));
            assert!(fixture.source.is_char_boundary(subject.end));
        }
    }
}

#[test]
fn selector_gold_covers_every_approved_architecture_dimension() {
    let fixtures = fixtures();

    for group in [
        SelectorGoldGroup::NormalCore,
        SelectorGoldGroup::InvalidCore,
        SelectorGoldGroup::SourceProvenance,
        SelectorGoldGroup::Nesting,
        SelectorGoldGroup::GroupAncestry,
        SelectorGoldGroup::PseudoProfile,
        SelectorGoldGroup::Namespace,
        SelectorGoldGroup::Lifecycle,
        SelectorGoldGroup::Resource,
    ] {
        assert!(fixtures.iter().any(|fixture| fixture.group == group));
    }

    assert!(
        fixtures
            .iter()
            .any(|fixture| fixture.id == "CSS-SELECTOR-SOURCE-ESCAPED-IDENT-001")
    );

    for mode in [
        SelectorGoldMode::NormalSelectorList,
        SelectorGoldMode::NestedRelativeSelectorList,
        SelectorGoldMode::ScopedRelativeSelectorList,
    ] {
        assert!(fixtures.iter().any(|fixture| fixture.mode == mode));
    }

    assert!(
        fixtures
            .iter()
            .any(|fixture| matches!(fixture.outcome, SelectorGoldOutcome::Qualified))
    );
    assert!(
        fixtures
            .iter()
            .any(|fixture| matches!(fixture.outcome, SelectorGoldOutcome::Invalid(_)))
    );
    assert!(
        fixtures
            .iter()
            .any(|fixture| matches!(fixture.outcome, SelectorGoldOutcome::Unsupported(_)))
    );
    assert!(
        fixtures
            .iter()
            .any(|fixture| matches!(fixture.outcome, SelectorGoldOutcome::Indeterminate(_)))
    );

    for lifecycle in [
        SelectorGoldLifecycle::StructurallyComplete,
        SelectorGoldLifecycle::TrueEofPartialBody,
        SelectorGoldLifecycle::UpstreamTokenizerIncompleteAfterHeader,
        SelectorGoldLifecycle::ParserResourceLimitAfterHeader,
        SelectorGoldLifecycle::SelectorResourceLimitBeforeObservation,
    ] {
        assert!(
            fixtures
                .iter()
                .any(|fixture| fixture.lifecycle == lifecycle)
        );
    }
}

#[test]
fn selector_gold_inventory_is_repeat_deterministic() {
    assert_eq!(fixtures(), fixtures());
}

#[test]
fn non_success_gold_keeps_actionable_subject_evidence_when_authored() {
    for fixture in fixtures() {
        match fixture.outcome {
            SelectorGoldOutcome::Qualified => {}
            SelectorGoldOutcome::Invalid(_)
            | SelectorGoldOutcome::Unsupported(_)
            | SelectorGoldOutcome::Indeterminate(_) => {
                assert!(
                    fixture.subject.is_some(),
                    "missing subject for {}",
                    fixture.id
                );
            }
        }
    }
}
