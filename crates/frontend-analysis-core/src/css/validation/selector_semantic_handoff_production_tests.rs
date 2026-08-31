//! Candidate-independent production bridge for the #402 semantic-handoff
//! theorem. Expected semantic facts below are handwritten in the #402 gold
//! vocabulary; production is only adapted into that vocabulary for equality.

use super::selector_semantic_handoff_gold::{
    AuthoredRange, ContextId, FunctionKind, MemberId, NestingPresenceDisposition,
    RelationshipOrigin, RelationshipTarget, SelectorFact, SimpleKind, UnitId,
};
use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::selector::handoff::{
    CssSelectorNestingPresenceDisposition, CssSelectorSemanticFact,
    CssSelectorSemanticFunctionKind, CssSelectorSemanticProgram,
    CssSelectorSemanticRelationshipOrigin, CssSelectorSemanticRelationshipTarget,
    CssSelectorSemanticSimpleKind,
};
use crate::css::selector::producer::run as run_selector;
use crate::css::selector::resource::{CssSelectorLimits, CssSelectorResourceKind};
use crate::css::selector::result::{
    CssSelectorIndeterminateReason, CssSelectorQualificationOutcome, CssSelectorTermination,
    CssSelectorUnsupportedFeature,
};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::{SourceId, SourceText};

fn tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(16 * 1024, 200_000, 16 * 1024, 2048, 16 * 1024, 16 * 1024).unwrap()
}

fn parser_limits() -> CssParserLimits {
    CssParserLimits::new(
        200_000,
        512,
        512,
        16 * 1024,
        2048,
        2048,
        2048,
        2048,
        16 * 1024,
    )
    .unwrap()
}

fn selector_limits() -> CssSelectorLimits {
    CssSelectorLimits::new(500_000, 256, 16 * 1024, 512 * 1024).unwrap()
}

fn run(
    source_id: u64,
    source: &str,
) -> crate::css::selector::result::CssSelectorQualificationRunResult {
    let source = SourceText::new(SourceId::new(source_id), source.to_owned());
    let parser = analyze_css_source(&source, tokenizer_limits(), parser_limits()).unwrap();
    run_selector(&source, parser, selector_limits()).unwrap()
}

fn range(start: usize, end: usize) -> AuthoredRange {
    AuthoredRange::new(start, end)
}

fn authored(start: usize, end: usize) -> RelationshipOrigin {
    RelationshipOrigin::Authored(range(start, end))
}

fn member(value: u32, start: usize, end: usize) -> SelectorFact {
    SelectorFact::OpenMember {
        member: MemberId(value),
        range: range(start, end),
    }
}

fn close(value: u32) -> SelectorFact {
    SelectorFact::CloseMember {
        member: MemberId(value),
    }
}

fn simple(value: u32, kind: SimpleKind, start: usize, end: usize) -> SelectorFact {
    SelectorFact::Simple {
        unit: UnitId(value),
        kind,
        range: range(start, end),
    }
}

fn map_origin(origin: &CssSelectorSemanticRelationshipOrigin) -> RelationshipOrigin {
    match origin {
        CssSelectorSemanticRelationshipOrigin::Authored(anchor) => {
            authored(anchor.range().start(), anchor.range().end())
        }
        CssSelectorSemanticRelationshipOrigin::Derived => RelationshipOrigin::Derived,
    }
}

fn map_target(target: CssSelectorSemanticRelationshipTarget) -> RelationshipTarget {
    match target {
        CssSelectorSemanticRelationshipTarget::ParentSelectorList(context) => {
            RelationshipTarget::ParentSelectorList(ContextId(context.index() as u32))
        }
        CssSelectorSemanticRelationshipTarget::ScopeRoot(context) => {
            RelationshipTarget::ScopeRoot(ContextId(context.index() as u32))
        }
        CssSelectorSemanticRelationshipTarget::Zero => RelationshipTarget::Zero,
    }
}

fn map_simple(kind: CssSelectorSemanticSimpleKind) -> SimpleKind {
    match kind {
        CssSelectorSemanticSimpleKind::Type => SimpleKind::Type,
        CssSelectorSemanticSimpleKind::Universal => SimpleKind::Universal,
        CssSelectorSemanticSimpleKind::Id => SimpleKind::Id,
        CssSelectorSemanticSimpleKind::Class => SimpleKind::Class,
        CssSelectorSemanticSimpleKind::Attribute => SimpleKind::Attribute,
        CssSelectorSemanticSimpleKind::IdentifierPseudoClass => SimpleKind::IdentifierPseudoClass,
    }
}

fn map_function(kind: CssSelectorSemanticFunctionKind) -> FunctionKind {
    match kind {
        CssSelectorSemanticFunctionKind::Is => FunctionKind::Is,
        CssSelectorSemanticFunctionKind::Where => FunctionKind::Where,
        CssSelectorSemanticFunctionKind::Not => FunctionKind::Not,
        CssSelectorSemanticFunctionKind::Has => FunctionKind::Has,
    }
}

fn map_disposition(
    disposition: CssSelectorNestingPresenceDisposition,
) -> NestingPresenceDisposition {
    match disposition {
        CssSelectorNestingPresenceDisposition::Contributing => {
            NestingPresenceDisposition::Contributing
        }
        CssSelectorNestingPresenceDisposition::NonContributingPresenceOnly => {
            NestingPresenceDisposition::NonContributingPresenceOnly
        }
    }
}

fn map_program(program: &CssSelectorSemanticProgram) -> Vec<SelectorFact> {
    program
        .facts()
        .iter()
        .map(|fact| match fact {
            CssSelectorSemanticFact::OpenMember {
                member,
                range: source,
            } => SelectorFact::OpenMember {
                member: MemberId(member.value() as u32),
                range: range(source.range().start(), source.range().end()),
            },
            CssSelectorSemanticFact::CloseMember { member } => SelectorFact::CloseMember {
                member: MemberId(member.value() as u32),
            },
            CssSelectorSemanticFact::RejectedForgivingMember {
                member,
                range: source,
            } => SelectorFact::RejectedForgivingMember {
                member: MemberId(member.value() as u32),
                range: range(source.range().start(), source.range().end()),
            },
            CssSelectorSemanticFact::Simple {
                unit,
                kind,
                range: source,
            } => SelectorFact::Simple {
                unit: UnitId(unit.value() as u32),
                kind: map_simple(*kind),
                range: range(source.range().start(), source.range().end()),
            },
            CssSelectorSemanticFact::OpenFunction {
                unit,
                kind,
                range: source,
            } => SelectorFact::OpenFunction {
                unit: UnitId(unit.value() as u32),
                kind: map_function(*kind),
                range: range(source.range().start(), source.range().end()),
            },
            CssSelectorSemanticFact::CloseFunction { unit } => SelectorFact::CloseFunction {
                unit: UnitId(unit.value() as u32),
            },
            CssSelectorSemanticFact::NestingPresence {
                member,
                unit,
                origin,
                disposition,
            } => SelectorFact::NestingPresence {
                member: MemberId(member.value() as u32),
                unit: UnitId(unit.value() as u32),
                origin: map_origin(origin),
                disposition: map_disposition(*disposition),
            },
            CssSelectorSemanticFact::Relationship { target, origin } => {
                SelectorFact::Relationship {
                    target: map_target(*target),
                    origin: map_origin(origin),
                }
            }
        })
        .collect()
}

fn facts(
    result: &crate::css::selector::result::CssSelectorQualificationRunResult,
    observation: usize,
) -> Vec<SelectorFact> {
    map_program(
        result.observations()[observation]
            .semantic_program()
            .expect("qualified observation owns a semantic program"),
    )
}

#[test]
fn production_matches_complete_handwritten_is_program_and_outer_members() {
    let result = run(50_001, ".a:is(#b, c){}");
    assert_eq!(
        facts(&result, 0),
        vec![
            member(1, 0, 12),
            simple(1, SimpleKind::Class, 0, 2),
            SelectorFact::OpenFunction {
                unit: UnitId(2),
                kind: FunctionKind::Is,
                range: range(2, 6),
            },
            member(2, 6, 8),
            simple(3, SimpleKind::Id, 6, 8),
            close(2),
            member(3, 10, 11),
            simple(4, SimpleKind::Type, 10, 11),
            close(3),
            SelectorFact::CloseFunction { unit: UnitId(2) },
            close(1),
        ]
    );

    let list = run(50_002, "a, #b{}");
    assert_eq!(
        facts(&list, 0),
        vec![
            member(1, 0, 1),
            simple(1, SimpleKind::Type, 0, 1),
            close(1),
            member(2, 3, 5),
            simple(2, SimpleKind::Id, 3, 5),
            close(2),
        ]
    );
}

#[test]
fn nested_selected_functions_retain_all_four_function_kinds() {
    let result = run(50_003, ":is(:where(.a), :not(#b), :has(> c)){}");
    let function_kinds: Vec<_> = facts(&result, 0)
        .into_iter()
        .filter_map(|fact| match fact {
            SelectorFact::OpenFunction { kind, .. } => Some(kind),
            _ => None,
        })
        .collect();
    assert_eq!(
        function_kinds,
        vec![
            FunctionKind::Is,
            FunctionKind::Where,
            FunctionKind::Not,
            FunctionKind::Has,
        ]
    );
}

#[test]
fn explicit_nesting_multiple_occurrences_and_where_preserve_exact_placement() {
    let multiple = run(50_004, ".a{& + &{}}");
    assert_eq!(
        facts(&multiple, 1),
        vec![
            member(1, 3, 8),
            SelectorFact::NestingPresence {
                member: MemberId(1),
                unit: UnitId(1),
                origin: authored(3, 4),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(0)),
                origin: authored(3, 4),
            },
            SelectorFact::NestingPresence {
                member: MemberId(1),
                unit: UnitId(2),
                origin: authored(7, 8),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(0)),
                origin: authored(7, 8),
            },
            close(1),
        ]
    );

    let where_result = run(50_005, ".a{:where(&){}}");
    assert_eq!(
        facts(&where_result, 1),
        vec![
            member(1, 3, 12),
            SelectorFact::OpenFunction {
                unit: UnitId(1),
                kind: FunctionKind::Where,
                range: range(3, 10),
            },
            member(2, 10, 11),
            SelectorFact::NestingPresence {
                member: MemberId(2),
                unit: UnitId(2),
                origin: authored(10, 11),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(0)),
                origin: authored(10, 11),
            },
            close(2),
            SelectorFact::CloseFunction { unit: UnitId(1) },
            close(1),
        ]
    );
}

#[test]
fn forgiving_rejection_preserves_only_noncontributing_nesting_presence() {
    let before_fault = run(50_006, ":is(&a, .b){}");
    assert_eq!(
        facts(&before_fault, 0),
        vec![
            member(1, 0, 11),
            SelectorFact::OpenFunction {
                unit: UnitId(1),
                kind: FunctionKind::Is,
                range: range(0, 4),
            },
            SelectorFact::RejectedForgivingMember {
                member: MemberId(2),
                range: range(4, 6),
            },
            SelectorFact::NestingPresence {
                member: MemberId(2),
                unit: UnitId(2),
                origin: authored(4, 5),
                disposition: NestingPresenceDisposition::NonContributingPresenceOnly,
            },
            member(3, 8, 10),
            simple(3, SimpleKind::Class, 8, 10),
            close(3),
            SelectorFact::CloseFunction { unit: UnitId(1) },
            close(1),
        ]
    );
    assert!(!facts(&before_fault, 0).iter().any(|fact| {
        matches!(
            fact,
            SelectorFact::Relationship {
                origin: RelationshipOrigin::Authored(AuthoredRange { start: 4, end: 5 }),
                ..
            }
        )
    }));

    let recovery_only = run(50_007, ":is(>>&, .b){}");
    assert!(facts(&recovery_only, 0).iter().any(|fact| {
        matches!(
            fact,
            SelectorFact::NestingPresence {
                member: MemberId(2),
                origin: RelationshipOrigin::Authored(AuthoredRange { start: 6, end: 7 }),
                disposition: NestingPresenceDisposition::NonContributingPresenceOnly,
                ..
            }
        )
    }));
    assert!(facts(&recovery_only, 0).iter().any(|fact| {
        matches!(
            fact,
            SelectorFact::RejectedForgivingMember {
                member: MemberId(2),
                range: AuthoredRange { start: 4, end: 7 },
            }
        )
    }));

    let without_nesting = run(50_008, ":is(>>x, .b){}");
    assert!(
        facts(&without_nesting, 0)
            .iter()
            .any(|fact| { matches!(fact, SelectorFact::RejectedForgivingMember { .. }) })
    );
    assert!(
        !facts(&without_nesting, 0)
            .iter()
            .any(|fact| matches!(fact, SelectorFact::NestingPresence { .. }))
    );
}

#[test]
fn unsupported_and_indeterminate_faults_are_not_swallowed_or_attached() {
    let unsupported = run(50_009, ":is(.a,:future-pseudo,.b){}");
    assert!(matches!(
        unsupported.observations()[0].outcome(),
        CssSelectorQualificationOutcome::UnsupportedBySelectedGrammarProfile {
            feature: CssSelectorUnsupportedFeature::IdentifierPseudoClass,
            ..
        }
    ));
    assert!(unsupported.observations()[0].semantic_program().is_none());

    let indeterminate = run(50_010, ":is(.a,svg|b,.c){}");
    assert!(matches!(
        indeterminate.observations()[0].outcome(),
        CssSelectorQualificationOutcome::Indeterminate {
            reason: CssSelectorIndeterminateReason::MissingNamespaceEnvironment,
            ..
        }
    ));
    assert!(indeterminate.observations()[0].semantic_program().is_none());
}

#[test]
fn parent_scope_and_implied_relationships_use_direct_structural_parent_order() {
    let scope_outside = run(50_011, "@scope{.a{& .b{}}}");
    let child_facts = facts(&scope_outside, 1);
    assert!(child_facts.iter().any(|fact| matches!(
        fact,
        SelectorFact::Relationship {
            target: RelationshipTarget::ParentSelectorList(ContextId(1)),
            origin: RelationshipOrigin::Authored(AuthoredRange { start: 10, end: 11 }),
        }
    )));

    let scope_inside = run(50_012, ".a{@scope{& .b{}}}");
    assert!(facts(&scope_inside, 1).iter().any(|fact| matches!(
        fact,
        SelectorFact::Relationship {
            target: RelationshipTarget::ScopeRoot(ContextId(1)),
            origin: RelationshipOrigin::Authored(AuthoredRange { start: 10, end: 11 }),
        }
    )));

    let implied = run(50_013, ".a{.b{}}");
    assert_eq!(
        facts(&implied, 1),
        vec![
            member(1, 3, 5),
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(0)),
                origin: RelationshipOrigin::Derived,
            },
            simple(1, SimpleKind::Class, 3, 5),
            close(1),
        ]
    );
}

#[test]
fn every_interposed_parent_is_charged_before_relationship_resolution() {
    let source_text = ".a{@layer{@supports (x:){@media{.b{}}}}}";
    let source = SourceText::new(SourceId::new(50_014), source_text.to_owned());
    let parser = analyze_css_source(&source, tokenizer_limits(), parser_limits()).unwrap();
    let complete = run_selector(&source, parser.clone(), selector_limits()).unwrap();
    assert_eq!(complete.observations().len(), 2);
    let complete_steps = complete
        .resources()
        .value(CssSelectorResourceKind::AlgorithmSteps);
    let before_relationship = complete_steps
        .checked_sub(4)
        .expect("three group parents plus one qualified parent are charged");

    let limited = run_selector(
        &source,
        parser,
        CssSelectorLimits::new(before_relationship, 256, 16 * 1024, 512 * 1024).unwrap(),
    )
    .unwrap();
    assert_eq!(limited.observations().len(), 1);
    let CssSelectorTermination::ResourceLimit(evidence) = limited.termination() else {
        panic!("relationship traversal must stop on selector AlgorithmSteps");
    };
    assert_eq!(evidence.kind(), CssSelectorResourceKind::AlgorithmSteps);
    assert_eq!(evidence.location().range().start(), 32);
    assert_eq!(evidence.location().range().end(), 32);
}

#[test]
fn retained_units_are_exact_atomic_and_observation_refusal_has_precedence() {
    let complete = run(50_015, "a{}");
    assert_eq!(
        complete
            .resources()
            .value(CssSelectorResourceKind::RetainedSemanticUnits),
        4
    );

    let invalid = run(50_016, "a,{}");
    assert!(invalid.observations()[0].semantic_program().is_none());
    assert_eq!(
        invalid
            .resources()
            .value(CssSelectorResourceKind::RetainedSemanticUnits),
        1
    );

    let source = SourceText::new(SourceId::new(50_017), "a{}b{}".to_owned());
    let parser = analyze_css_source(&source, tokenizer_limits(), parser_limits()).unwrap();
    let limited = run_selector(
        &source,
        parser.clone(),
        CssSelectorLimits::new(500_000, 256, 16, 4).unwrap(),
    )
    .unwrap();
    assert_eq!(limited.observations().len(), 1);
    assert_eq!(
        limited
            .resources()
            .value(CssSelectorResourceKind::RetainedSemanticUnits),
        4
    );
    assert!(matches!(
        limited.termination(),
        CssSelectorTermination::ResourceLimit(evidence)
            if evidence.kind() == CssSelectorResourceKind::RetainedSemanticUnits
    ));

    let precedence = run_selector(
        &source,
        parser,
        CssSelectorLimits::new(500_000, 256, 0, 0).unwrap(),
    )
    .unwrap();
    assert!(precedence.observations().is_empty());
    assert!(matches!(
        precedence.termination(),
        CssSelectorTermination::ResourceLimit(evidence)
            if evidence.kind() == CssSelectorResourceKind::Observations
    ));
}

#[test]
fn utf8_provenance_repeat_determinism_and_source_identity_are_stable() {
    let first = run(50_018, "éあ.x{}");
    let second = run(50_018, "éあ.x{}");
    let distinct_source = run(50_019, "éあ.x{}");
    let expected = vec![
        member(1, 0, 7),
        simple(1, SimpleKind::Type, 0, 5),
        simple(2, SimpleKind::Class, 5, 7),
        close(1),
    ];
    assert_eq!(facts(&first, 0), expected);
    assert_eq!(facts(&first, 0), facts(&second, 0));
    assert_eq!(facts(&first, 0), facts(&distinct_source, 0));
}

#[test]
fn selector_execution_does_not_mutate_parser_or_tokenizer_resource_counters() {
    let source = SourceText::new(SourceId::new(50_020), ".a{& .b{}}".to_owned());
    let parser = analyze_css_source(&source, tokenizer_limits(), parser_limits()).unwrap();
    let parser_usage = parser.resources();
    let tokenizer_usage = parser.upstream_tokenizer_result().resources();
    let result = run_selector(&source, parser, selector_limits()).unwrap();

    assert_eq!(result.upstream_parser_result().resources(), parser_usage);
    assert_eq!(
        result
            .upstream_parser_result()
            .upstream_tokenizer_result()
            .resources(),
        tokenizer_usage
    );
}