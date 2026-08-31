//! Candidate-independent production bridge for the #405 semantic handoff.
//!
//! Dependency direction is fixed by #404/#405:
//!
//! ```text
//! existing handwritten #402 gold/reference
//!                  ^
//!                  |
//!         production comparison adapter        (this module)
//!                  |
//!            production handoff
//! ```
//!
//! The adapter only *translates* retained production evidence into the
//! independent #402 vocabulary. Every expected program in this module is
//! handwritten from the fixture's authored CSS text; nothing expected is
//! generated from production, parsed by production, or computed by calling
//! production helpers. The #402 gold and reference modules remain byte
//! unchanged and are used here exactly as they were accepted.

use super::selector_semantic_handoff_gold::{
    AuthoredFactExpectation, AuthoredRange, CompletionState, ContextId, FunctionKind,
    GoldObservation, GoldOutcome, GoldProgram, GoldRun, MemberId, NestingPresenceDisposition,
    RejectedNestingEffect, RejectedNestingPresenceExpectation, RelationshipOrigin,
    RelationshipTarget, RunId, SelectorFact, SimpleKind, SourceId as GoldSourceId, UnitId,
    validate_program_authored_provenance, validate_rejected_nesting_presence,
};
use super::selector_semantic_handoff_reference::{
    ConsumerBudget, ConsumerRunCompletion, resolve_retained_run,
};

use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::selector::analysis::analyze_css_selectors;
use crate::css::selector::handoff::{
    CssSelectorSemanticFact, CssSelectorSemanticMemberId, CssSelectorSemanticNestingDisposition,
    CssSelectorSemanticProgram, CssSelectorSemanticRelationshipOrigin,
    CssSelectorSemanticRelationshipTarget, CssSelectorSemanticSimpleKind,
    CssSelectorSemanticUnitId,
};
use crate::css::selector::profile::CssSelectorFunctionalPseudoClass;
use crate::css::selector::resource::{CssSelectorLimits, CssSelectorResourceKind};
use crate::css::selector::result::{
    CssSelectorExecutionCompletion, CssSelectorIndeterminateReason,
    CssSelectorQualificationOutcome, CssSelectorQualificationRunResult, CssSelectorTermination,
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
    CssSelectorLimits::new(200_000, 128, 16 * 1024, 128 * 1024).unwrap()
}

fn qualify(text: &str, source_id: u64) -> (SourceText, CssSelectorQualificationRunResult) {
    let source = SourceText::new(SourceId::new(source_id), text.to_owned());
    let result = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        selector_limits(),
    )
    .unwrap_or_else(|error| panic!("selector execution failed for {text:?}: {error:?}"));
    (source, result)
}

/// Canonicalizes opaque production identifiers by first appearance.
///
/// Production identifiers are deliberately opaque and may contain gaps after
/// rolled-back staging, so numeric equality is not the contract. Order-
/// preserving injective renumbering keeps every placement and cross-reference
/// falsifiable while honoring that the numbers themselves carry no meaning.
#[derive(Default)]
struct IdentityCanonicalizer {
    members: Vec<CssSelectorSemanticMemberId>,
    units: Vec<CssSelectorSemanticUnitId>,
}

impl IdentityCanonicalizer {
    fn member(&mut self, member: CssSelectorSemanticMemberId) -> MemberId {
        let position = self
            .members
            .iter()
            .position(|known| *known == member)
            .unwrap_or_else(|| {
                self.members.push(member);
                self.members.len() - 1
            });
        MemberId(u32::try_from(position + 1).expect("member ordinal fits a gold identifier"))
    }

    fn unit(&mut self, unit: CssSelectorSemanticUnitId) -> UnitId {
        let position = self
            .units
            .iter()
            .position(|known| *known == unit)
            .unwrap_or_else(|| {
                self.units.push(unit);
                self.units.len() - 1
            });
        UnitId(u32::try_from(position + 1).expect("unit ordinal fits a gold identifier"))
    }
}

fn authored_range(source: &SourceText, anchor: &crate::SourceAnchor) -> AuthoredRange {
    assert_eq!(
        anchor.source_id(),
        source.id(),
        "every authored semantic anchor must belong to the analyzed source"
    );
    AuthoredRange::new(anchor.range().start(), anchor.range().end())
}

fn adapt_origin(
    source: &SourceText,
    origin: &CssSelectorSemanticRelationshipOrigin,
) -> RelationshipOrigin {
    match origin {
        CssSelectorSemanticRelationshipOrigin::Authored(anchor) => {
            RelationshipOrigin::Authored(authored_range(source, anchor))
        }
        CssSelectorSemanticRelationshipOrigin::Derived => RelationshipOrigin::Derived,
    }
}

fn adapt_simple_kind(kind: CssSelectorSemanticSimpleKind) -> SimpleKind {
    match kind {
        CssSelectorSemanticSimpleKind::Type => SimpleKind::Type,
        CssSelectorSemanticSimpleKind::Universal => SimpleKind::Universal,
        CssSelectorSemanticSimpleKind::Id => SimpleKind::Id,
        CssSelectorSemanticSimpleKind::Class => SimpleKind::Class,
        CssSelectorSemanticSimpleKind::Attribute => SimpleKind::Attribute,
        CssSelectorSemanticSimpleKind::IdentifierPseudoClass => SimpleKind::IdentifierPseudoClass,
    }
}

fn adapt_function_kind(kind: CssSelectorFunctionalPseudoClass) -> FunctionKind {
    match kind {
        CssSelectorFunctionalPseudoClass::Is => FunctionKind::Is,
        CssSelectorFunctionalPseudoClass::Where => FunctionKind::Where,
        CssSelectorFunctionalPseudoClass::Not => FunctionKind::Not,
        CssSelectorFunctionalPseudoClass::Has => FunctionKind::Has,
    }
}

fn adapt_target(target: CssSelectorSemanticRelationshipTarget) -> RelationshipTarget {
    match target {
        CssSelectorSemanticRelationshipTarget::ParentSelectorList(context) => {
            RelationshipTarget::ParentSelectorList(context_id(context.index()))
        }
        CssSelectorSemanticRelationshipTarget::ScopeRoot(context) => {
            RelationshipTarget::ScopeRoot(context_id(context.index()))
        }
        CssSelectorSemanticRelationshipTarget::Zero => RelationshipTarget::Zero,
    }
}

fn adapt_disposition(
    disposition: CssSelectorSemanticNestingDisposition,
) -> NestingPresenceDisposition {
    match disposition {
        CssSelectorSemanticNestingDisposition::Contributing => {
            NestingPresenceDisposition::Contributing
        }
        CssSelectorSemanticNestingDisposition::NonContributingPresenceOnly => {
            NestingPresenceDisposition::NonContributingPresenceOnly
        }
    }
}

fn context_id(index: usize) -> ContextId {
    ContextId(u32::try_from(index).expect("retained context index fits a gold identifier"))
}

/// Translates one retained production program into the #402 vocabulary.
///
/// Production has no program-level RunId, SourceId, or profile (#404
/// FA404-02); the adapter supplies the single-run constants the independent
/// gold model expects and asserts nothing about them.
fn adapt_program(source: &SourceText, program: &CssSelectorSemanticProgram) -> GoldProgram {
    let mut identities = IdentityCanonicalizer::default();
    let facts = program
        .facts()
        .iter()
        .map(|fact| match fact {
            CssSelectorSemanticFact::OpenMember { member, range } => SelectorFact::OpenMember {
                member: identities.member(*member),
                range: authored_range(source, range),
            },
            CssSelectorSemanticFact::CloseMember { member } => SelectorFact::CloseMember {
                member: identities.member(*member),
            },
            CssSelectorSemanticFact::RejectedForgivingMember { member, range } => {
                SelectorFact::RejectedForgivingMember {
                    member: identities.member(*member),
                    range: authored_range(source, range),
                }
            }
            CssSelectorSemanticFact::Simple { unit, kind, range } => SelectorFact::Simple {
                unit: identities.unit(*unit),
                kind: adapt_simple_kind(*kind),
                range: authored_range(source, range),
            },
            CssSelectorSemanticFact::OpenFunction { unit, kind, range } => {
                SelectorFact::OpenFunction {
                    unit: identities.unit(*unit),
                    kind: adapt_function_kind(*kind),
                    range: authored_range(source, range),
                }
            }
            CssSelectorSemanticFact::CloseFunction { unit } => SelectorFact::CloseFunction {
                unit: identities.unit(*unit),
            },
            CssSelectorSemanticFact::NestingPresence {
                member,
                unit,
                origin,
                disposition,
            } => SelectorFact::NestingPresence {
                member: identities.member(*member),
                unit: identities.unit(*unit),
                origin: adapt_origin(source, origin),
                disposition: adapt_disposition(*disposition),
            },
            CssSelectorSemanticFact::Relationship { target, origin } => {
                SelectorFact::Relationship {
                    target: adapt_target(*target),
                    origin: adapt_origin(source, origin),
                }
            }
        })
        .collect();

    GoldProgram {
        source: GoldSourceId(1),
        run: RunId(1),
        profile: "CoreV1",
        context: context_id(program.owning_context().index()),
        facts,
    }
}

/// Adapts the retained program of one qualified observation.
fn adapt_observation_program(
    source: &SourceText,
    result: &CssSelectorQualificationRunResult,
    observation_index: usize,
) -> GoldProgram {
    let observation = &result.observations()[observation_index];
    assert!(
        matches!(
            observation.outcome(),
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
        ),
        "observation {observation_index} must qualify"
    );
    let program = observation
        .semantic_program()
        .expect("a qualified observation owns a retained semantic program");
    assert_eq!(
        program.owning_context(),
        observation.context_id(),
        "retained program attachment must match its observation"
    );
    adapt_program(source, program)
}

fn program(context: u32, facts: Vec<SelectorFact>) -> GoldProgram {
    GoldProgram {
        source: GoldSourceId(1),
        run: RunId(1),
        profile: "CoreV1",
        context: ContextId(context),
        facts,
    }
}

fn range(start: usize, end: usize) -> AuthoredRange {
    AuthoredRange::new(start, end)
}

fn open(member: u32, start: usize, end: usize) -> SelectorFact {
    SelectorFact::OpenMember {
        member: MemberId(member),
        range: range(start, end),
    }
}

fn close(member: u32) -> SelectorFact {
    SelectorFact::CloseMember {
        member: MemberId(member),
    }
}

fn atom(unit: u32, kind: SimpleKind, start: usize, end: usize) -> SelectorFact {
    SelectorFact::Simple {
        unit: UnitId(unit),
        kind,
        range: range(start, end),
    }
}

fn open_function(unit: u32, kind: FunctionKind, start: usize, end: usize) -> SelectorFact {
    SelectorFact::OpenFunction {
        unit: UnitId(unit),
        kind,
        range: range(start, end),
    }
}

fn close_function(unit: u32) -> SelectorFact {
    SelectorFact::CloseFunction { unit: UnitId(unit) }
}

fn nesting(
    member: u32,
    unit: u32,
    start: usize,
    end: usize,
    disposition: NestingPresenceDisposition,
) -> SelectorFact {
    SelectorFact::NestingPresence {
        member: MemberId(member),
        unit: UnitId(unit),
        origin: RelationshipOrigin::Authored(range(start, end)),
        disposition,
    }
}

fn authored_relationship(target: RelationshipTarget, start: usize, end: usize) -> SelectorFact {
    SelectorFact::Relationship {
        target,
        origin: RelationshipOrigin::Authored(range(start, end)),
    }
}

fn derived_relationship(target: RelationshipTarget) -> SelectorFact {
    SelectorFact::Relationship {
        target,
        origin: RelationshipOrigin::Derived,
    }
}

fn authored_fact(
    fact_index: usize,
    start: usize,
    end: usize,
    spelling: &'static str,
) -> AuthoredFactExpectation {
    AuthoredFactExpectation {
        fact_index,
        range: range(start, end),
        spelling,
    }
}

/// One handwritten production-comparison fixture.
struct BridgeFixture {
    id: &'static str,
    source: &'static str,
    /// Handwritten expected program per qualified observation, in retained
    /// observation order.
    expected: Vec<GoldProgram>,
}

fn assert_fixture(fixture: &BridgeFixture, source_id: u64) {
    let (source, result) = qualify(fixture.source, source_id);
    assert_eq!(
        result.execution_completion(),
        CssSelectorExecutionCompletion::Complete,
        "{} must complete",
        fixture.id
    );
    assert_eq!(
        result.observations().len(),
        fixture.expected.len(),
        "{} observation cardinality",
        fixture.id
    );

    for (index, expected) in fixture.expected.iter().enumerate() {
        let actual = adapt_observation_program(&source, &result, index);
        assert_eq!(
            &actual, expected,
            "{} retained program mismatch at observation {index}",
            fixture.id
        );
    }

    // Independent retained accounting, recomputed from the handwritten gold.
    let expected_units: usize = fixture
        .expected
        .iter()
        .map(|program| 1 + program.facts.len())
        .sum();
    assert_eq!(
        result
            .resources()
            .value(CssSelectorResourceKind::RetainedSemanticUnits),
        expected_units,
        "{} retained-unit accounting",
        fixture.id
    );
}

fn fixtures() -> Vec<BridgeFixture> {
    vec![
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-OUTER-LIST-001",
            source: ".a, #b{}",
            expected: vec![program(
                0,
                vec![
                    open(1, 0, 2),
                    atom(1, SimpleKind::Class, 0, 2),
                    close(1),
                    open(2, 4, 6),
                    atom(2, SimpleKind::Id, 4, 6),
                    close(2),
                ],
            )],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-IS-001",
            source: ".a:is(#b, c){}",
            expected: vec![program(
                0,
                vec![
                    open(1, 0, 12),
                    atom(1, SimpleKind::Class, 0, 2),
                    open_function(2, FunctionKind::Is, 2, 6),
                    open(2, 6, 8),
                    atom(3, SimpleKind::Id, 6, 8),
                    close(2),
                    open(3, 10, 11),
                    atom(4, SimpleKind::Type, 10, 11),
                    close(3),
                    close_function(2),
                    close(1),
                ],
            )],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-WHERE-001",
            source: ".a:where(#b){}",
            expected: vec![program(
                0,
                vec![
                    open(1, 0, 12),
                    atom(1, SimpleKind::Class, 0, 2),
                    open_function(2, FunctionKind::Where, 2, 9),
                    open(2, 9, 11),
                    atom(3, SimpleKind::Id, 9, 11),
                    close(2),
                    close_function(2),
                    close(1),
                ],
            )],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-NOT-001",
            source: ".a:not(.b){}",
            expected: vec![program(
                0,
                vec![
                    open(1, 0, 10),
                    atom(1, SimpleKind::Class, 0, 2),
                    open_function(2, FunctionKind::Not, 2, 7),
                    open(2, 7, 9),
                    atom(3, SimpleKind::Class, 7, 9),
                    close(2),
                    close_function(2),
                    close(1),
                ],
            )],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-HAS-001",
            source: ".a:has(> .b){}",
            expected: vec![program(
                0,
                vec![
                    open(1, 0, 12),
                    atom(1, SimpleKind::Class, 0, 2),
                    open_function(2, FunctionKind::Has, 2, 7),
                    // The member envelope covers its leading combinator; the
                    // accepted #402 vocabulary retains no combinator fact.
                    open(2, 7, 11),
                    atom(3, SimpleKind::Class, 9, 11),
                    close(2),
                    close_function(2),
                    close(1),
                ],
            )],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-SIMPLE-KINDS-001",
            source: "*[x=\"y\" i]{}",
            expected: vec![program(
                0,
                vec![
                    open(1, 0, 10),
                    atom(1, SimpleKind::Universal, 0, 1),
                    atom(2, SimpleKind::Attribute, 1, 10),
                    close(1),
                ],
            )],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-SIMPLE-KINDS-002",
            source: "a:hover{}",
            expected: vec![program(
                0,
                vec![
                    open(1, 0, 7),
                    atom(1, SimpleKind::Type, 0, 1),
                    atom(2, SimpleKind::IdentifierPseudoClass, 1, 7),
                    close(1),
                ],
            )],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-EXPLICIT-NESTING-001",
            source: ".a{& .b{}}",
            expected: vec![
                program(
                    0,
                    vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
                ),
                program(
                    1,
                    vec![
                        open(1, 3, 7),
                        nesting(1, 1, 3, 4, NestingPresenceDisposition::Contributing),
                        authored_relationship(
                            RelationshipTarget::ParentSelectorList(ContextId(0)),
                            3,
                            4,
                        ),
                        atom(2, SimpleKind::Class, 5, 7),
                        close(1),
                    ],
                ),
            ],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-IMPLIED-NESTING-001",
            source: ".a{.b{}}",
            expected: vec![
                program(
                    0,
                    vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
                ),
                program(
                    1,
                    vec![
                        open(1, 3, 5),
                        atom(1, SimpleKind::Class, 3, 5),
                        derived_relationship(RelationshipTarget::ParentSelectorList(ContextId(0))),
                        close(1),
                    ],
                ),
            ],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-MIXED-NESTING-001",
            source: ".a{&, .c{}}",
            expected: vec![
                program(
                    0,
                    vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
                ),
                program(
                    1,
                    vec![
                        open(1, 3, 4),
                        nesting(1, 1, 3, 4, NestingPresenceDisposition::Contributing),
                        authored_relationship(
                            RelationshipTarget::ParentSelectorList(ContextId(0)),
                            3,
                            4,
                        ),
                        close(1),
                        open(2, 6, 8),
                        atom(2, SimpleKind::Class, 6, 8),
                        derived_relationship(RelationshipTarget::ParentSelectorList(ContextId(0))),
                        close(2),
                    ],
                ),
            ],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-MULTIPLE-AMPERSAND-001",
            source: ".a{&:where(&){}}",
            expected: vec![
                program(
                    0,
                    vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
                ),
                program(
                    1,
                    vec![
                        open(1, 3, 13),
                        nesting(1, 1, 3, 4, NestingPresenceDisposition::Contributing),
                        authored_relationship(
                            RelationshipTarget::ParentSelectorList(ContextId(0)),
                            3,
                            4,
                        ),
                        open_function(2, FunctionKind::Where, 4, 11),
                        open(2, 11, 12),
                        nesting(2, 3, 11, 12, NestingPresenceDisposition::Contributing),
                        authored_relationship(
                            RelationshipTarget::ParentSelectorList(ContextId(0)),
                            11,
                            12,
                        ),
                        close(2),
                        close_function(2),
                        close(1),
                    ],
                ),
            ],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-SCOPE-ROOT-001",
            source: ".z{@scope{.b{}}}",
            expected: vec![
                program(
                    0,
                    vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
                ),
                program(
                    2,
                    vec![
                        open(1, 10, 12),
                        atom(1, SimpleKind::Class, 10, 12),
                        derived_relationship(RelationshipTarget::ScopeRoot(ContextId(1))),
                        close(1),
                    ],
                ),
            ],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-UTF8-001",
            source: "é#x{}",
            expected: vec![program(
                0,
                vec![
                    open(1, 0, 4),
                    atom(1, SimpleKind::Type, 0, 2),
                    atom(2, SimpleKind::Id, 2, 4),
                    close(1),
                ],
            )],
        },
        BridgeFixture {
            id: "CSS-HANDOFF-PROD-UTF8-002",
            source: "éあ.x{}",
            expected: vec![program(
                0,
                vec![
                    open(1, 0, 7),
                    atom(1, SimpleKind::Type, 0, 5),
                    atom(2, SimpleKind::Class, 5, 7),
                    close(1),
                ],
            )],
        },
    ]
}

#[test]
fn production_programs_match_every_handwritten_bridge_fixture() {
    for (index, fixture) in fixtures().iter().enumerate() {
        assert_fixture(fixture, 50_000 + index as u64);
    }
}

#[test]
fn production_programs_are_repeat_deterministic_and_source_id_independent() {
    for (index, fixture) in fixtures().iter().enumerate() {
        assert_fixture(fixture, 51_000 + index as u64);
        assert_fixture(fixture, 52_000 + index as u64);
    }
}

#[test]
fn every_bridge_fixture_run_resolves_under_the_independent_reference() {
    // The accepted #402 reference owns dependency ordering, attachment, and
    // program well-formedness. Every handwritten bridge expectation must be a
    // fully resolvable retained run under that independent authority.
    for fixture in fixtures() {
        let run = GoldRun {
            source: GoldSourceId(1),
            run: RunId(1),
            profile: "CoreV1",
            upstream: CompletionState::Complete,
            qualifier: CompletionState::Complete,
            observations: fixture
                .expected
                .iter()
                .map(|program| GoldObservation {
                    source: program.source,
                    run: program.run,
                    profile: program.profile,
                    context: program.context,
                    completion: CompletionState::Complete,
                    outcome: GoldOutcome::Qualified,
                    program: Some(program.clone()),
                })
                .collect(),
        };
        let resolved = resolve_retained_run(&run, ConsumerBudget { limit: usize::MAX })
            .unwrap_or_else(|error| {
                panic!(
                    "{} must resolve under the #402 reference: {error:?}",
                    fixture.id
                )
            });
        assert_eq!(
            resolved.completion(),
            ConsumerRunCompletion::Complete,
            "{} must resolve completely",
            fixture.id
        );
    }
}

// -- rejected forgiving members ------------------------------------------

/// `&` recognized before the authored-invalid fault.
const REJECTED_AMPERSAND_BEFORE_FAULT: &str = ".a:is(&], .b){}";
/// `&` reached only while authoritative recovery consumes the same member.
const REJECTED_AMPERSAND_AFTER_FAULT: &str = ".a:is(]&, .b){}";
/// Authored-invalid forgiving member with no nesting selector at all.
const REJECTED_WITHOUT_AMPERSAND: &str = ".a:is(], .b){}";

fn rejected_ampersand_expected(rejected_end: usize, presence: (usize, usize)) -> GoldProgram {
    program(
        0,
        vec![
            open(1, 0, 13),
            atom(1, SimpleKind::Class, 0, 2),
            open_function(2, FunctionKind::Is, 2, 6),
            SelectorFact::RejectedForgivingMember {
                member: MemberId(2),
                range: range(6, rejected_end),
            },
            nesting(
                2,
                3,
                presence.0,
                presence.1,
                NestingPresenceDisposition::NonContributingPresenceOnly,
            ),
            open(3, 10, 12),
            atom(4, SimpleKind::Class, 10, 12),
            close(3),
            close_function(2),
            close(1),
        ],
    )
}

#[test]
fn rejected_forgiving_member_preserves_ampersand_recognized_before_the_fault() {
    let (source, result) = qualify(REJECTED_AMPERSAND_BEFORE_FAULT, 53_000);
    let actual = adapt_observation_program(&source, &result, 0);
    let expected = rejected_ampersand_expected(8, (6, 7));
    assert_eq!(actual, expected);

    let evidence = validate_rejected_nesting_presence(
        &actual,
        RejectedNestingPresenceExpectation {
            member: MemberId(2),
            rejected_range: range(6, 8),
            unit: UnitId(3),
            presence_range: range(6, 7),
        },
    )
    .expect("the rejected member owns exactly one non-contributing presence");
    assert_eq!(
        evidence.disposition,
        NestingPresenceDisposition::NonContributingPresenceOnly
    );
    assert_eq!(
        evidence.effect,
        RejectedNestingEffect::SuppressesImpliedNesting
    );

    assert_eq!(
        validate_program_authored_provenance(
            &actual,
            REJECTED_AMPERSAND_BEFORE_FAULT,
            &[
                authored_fact(0, 0, 13, ".a:is(&], .b)"),
                authored_fact(1, 0, 2, ".a"),
                authored_fact(2, 2, 6, ":is("),
                authored_fact(3, 6, 8, "&]"),
                authored_fact(4, 6, 7, "&"),
                authored_fact(5, 10, 12, ".b"),
                authored_fact(6, 10, 12, ".b"),
            ],
        ),
        Ok(())
    );
}

#[test]
fn rejected_forgiving_member_preserves_ampersand_found_during_recovery() {
    let (source, result) = qualify(REJECTED_AMPERSAND_AFTER_FAULT, 53_001);
    let actual = adapt_observation_program(&source, &result, 0);
    assert_eq!(actual, rejected_ampersand_expected(8, (7, 8)));

    assert!(
        validate_rejected_nesting_presence(
            &actual,
            RejectedNestingPresenceExpectation {
                member: MemberId(2),
                rejected_range: range(6, 8),
                unit: UnitId(3),
                presence_range: range(7, 8),
            },
        )
        .is_ok()
    );

    assert_eq!(
        validate_program_authored_provenance(
            &actual,
            REJECTED_AMPERSAND_AFTER_FAULT,
            &[
                authored_fact(0, 0, 13, ".a:is(]&, .b)"),
                authored_fact(1, 0, 2, ".a"),
                authored_fact(2, 2, 6, ":is("),
                authored_fact(3, 6, 8, "]&"),
                authored_fact(4, 7, 8, "&"),
                authored_fact(5, 10, 12, ".b"),
                authored_fact(6, 10, 12, ".b"),
            ],
        ),
        Ok(())
    );
}

#[test]
fn rejected_forgiving_member_retains_every_authored_nesting_presence() {
    // Two authored `&` occurrences inside one rejected member. The accepted
    // #402 single-presence validator cannot express this shape, so the whole
    // retained program is compared against the handwritten expectation.
    let (source, result) = qualify(".a:is(&&], .b){}", 53_005);
    let actual = adapt_observation_program(&source, &result, 0);

    assert_eq!(
        actual,
        program(
            0,
            vec![
                open(1, 0, 14),
                atom(1, SimpleKind::Class, 0, 2),
                open_function(2, FunctionKind::Is, 2, 6),
                SelectorFact::RejectedForgivingMember {
                    member: MemberId(2),
                    range: range(6, 9),
                },
                nesting(
                    2,
                    3,
                    6,
                    7,
                    NestingPresenceDisposition::NonContributingPresenceOnly
                ),
                nesting(
                    2,
                    4,
                    7,
                    8,
                    NestingPresenceDisposition::NonContributingPresenceOnly
                ),
                open(3, 11, 13),
                atom(5, SimpleKind::Class, 11, 13),
                close(3),
                close_function(2),
                close(1),
            ],
        )
    );
    assert!(
        !actual
            .facts
            .iter()
            .any(|fact| matches!(fact, SelectorFact::Relationship { .. })),
        "a rejected member contributes no relationship pressure"
    );
}

#[test]
fn rejected_forgiving_member_without_a_nesting_selector_retains_no_presence() {
    let (source, result) = qualify(REJECTED_WITHOUT_AMPERSAND, 53_002);
    let actual = adapt_observation_program(&source, &result, 0);

    assert_eq!(
        actual,
        program(
            0,
            vec![
                open(1, 0, 12),
                atom(1, SimpleKind::Class, 0, 2),
                open_function(2, FunctionKind::Is, 2, 6),
                SelectorFact::RejectedForgivingMember {
                    member: MemberId(2),
                    range: range(6, 7),
                },
                open(3, 9, 11),
                atom(3, SimpleKind::Class, 9, 11),
                close(3),
                close_function(2),
                close(1),
            ],
        )
    );
    assert!(
        !actual.facts.iter().any(|fact| matches!(
            fact,
            SelectorFact::NestingPresence { .. } | SelectorFact::Relationship { .. }
        )),
        "a rejected member without `&` retains no presence or relationship"
    );
}

#[test]
fn a_fault_inside_an_unforgiving_function_never_retains_a_rejected_member() {
    // FA407-01B regression. `:not()` and `:has()` are not forgiving, so an
    // authored-invalid member inside one is never rejected-and-recovered.
    for text in [".a:not(], .b){}", ".a:has(], .b){}"] {
        let (_, result) = qualify(text, 53_006);
        assert!(
            matches!(
                result.observations()[0].outcome(),
                CssSelectorQualificationOutcome::InvalidForSelectedGrammar { .. }
            ),
            "{text} must stay invalid rather than recover"
        );
        assert!(result.observations()[0].semantic_program().is_none());
    }

    // When an unforgiving function faults inside a forgiving one, the rejected
    // member is owned by the forgiving ancestor, and the rolled-back inner
    // function leaves a deliberate identifier gap.
    let (source, result) = qualify(".a:is(:not(]), .b){}", 53_007);
    assert_eq!(
        adapt_observation_program(&source, &result, 0),
        program(
            0,
            vec![
                open(1, 0, 18),
                atom(1, SimpleKind::Class, 0, 2),
                open_function(2, FunctionKind::Is, 2, 6),
                SelectorFact::RejectedForgivingMember {
                    member: MemberId(2),
                    range: range(6, 13),
                },
                open(3, 15, 17),
                atom(3, SimpleKind::Class, 15, 17),
                close(3),
                close_function(2),
                close(1),
            ],
        )
    );
}

#[test]
fn rejected_member_nesting_presence_suppresses_implied_nesting() {
    // The only authored `&` lives inside a rejected forgiving member. It is
    // retained as non-contributing evidence and still suppresses the implied
    // relationship the enclosing nested member would otherwise carry.
    let (source, result) = qualify(".a{:is(&], .b){}}", 53_003);
    let nested = adapt_observation_program(&source, &result, 1);

    assert!(
        nested.facts.iter().any(|fact| matches!(
            fact,
            SelectorFact::NestingPresence {
                disposition: NestingPresenceDisposition::NonContributingPresenceOnly,
                ..
            }
        )),
        "the rejected member retains its non-contributing presence"
    );
    assert!(
        !nested
            .facts
            .iter()
            .any(|fact| matches!(fact, SelectorFact::Relationship { .. })),
        "no contributing or implied relationship survives the rejected alternative"
    );

    // The same selector without any `&` keeps its implied relationship.
    let (plain_source, plain_result) = qualify(".a{:is(], .b){}}", 53_004);
    let plain = adapt_observation_program(&plain_source, &plain_result, 1);
    assert!(
        plain.facts.iter().any(|fact| matches!(
            fact,
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(_),
                origin: RelationshipOrigin::Derived,
            }
        )),
        "without any authored `&` the nested member keeps implied nesting"
    );
}

// -- outcomes that must not be guessed away -------------------------------

#[test]
fn unsupported_and_indeterminate_are_not_swallowed_as_forgiving_members() {
    // `:future-pseudo` is outside the CoreV1 identifier registry and a named
    // namespace has no environment: neither becomes a rejected forgiving
    // member, and neither retains a program.
    let (_, unsupported) = qualify(".a:is(:future-pseudo, .b){}", 54_000);
    assert!(matches!(
        unsupported.observations()[0].outcome(),
        CssSelectorQualificationOutcome::UnsupportedBySelectedGrammarProfile {
            feature: CssSelectorUnsupportedFeature::IdentifierPseudoClass,
            ..
        }
    ));
    assert!(unsupported.observations()[0].semantic_program().is_none());

    let (_, indeterminate) = qualify(".a:is(svg|a, .b){}", 54_001);
    assert!(matches!(
        indeterminate.observations()[0].outcome(),
        CssSelectorQualificationOutcome::Indeterminate {
            reason: CssSelectorIndeterminateReason::MissingNamespaceEnvironment,
            ..
        }
    ));
    assert!(indeterminate.observations()[0].semantic_program().is_none());

    let (_, invalid) = qualify("a,{}", 54_002);
    assert!(matches!(
        invalid.observations()[0].outcome(),
        CssSelectorQualificationOutcome::InvalidForSelectedGrammar { .. }
    ));
    assert!(invalid.observations()[0].semantic_program().is_none());

    // Non-qualified observations still cost exactly one retained unit each.
    for result in [unsupported, indeterminate, invalid] {
        assert_eq!(
            result
                .resources()
                .value(CssSelectorResourceKind::RetainedSemanticUnits),
            1
        );
    }
}

// -- parent-vs-scope ordering ---------------------------------------------

#[test]
fn parent_and_scope_ordering_counterexamples_stay_distinct_end_to_end() {
    // Grammar-entry selection is scoped-relative in both sources; the semantic
    // relationship target is the nearest retained structural boundary. The
    // outer qualified rule exists because this baseline's parser retains group
    // contexts only for nested at-rules.
    let (outer_source, outer) = qualify(".z{@scope{.a{& .b{}}}}", 55_000);
    let nested = adapt_observation_program(&outer_source, &outer, 2);
    assert!(
        nested.facts.iter().any(|fact| matches!(
            fact,
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(2)),
                origin: RelationshipOrigin::Authored(_),
            }
        )),
        "the nearest boundary is the enclosing qualified rule"
    );

    let (inner_source, inner) = qualify(".z{.a{@scope{& .b{}}}}", 55_001);
    let scoped = adapt_observation_program(&inner_source, &inner, 2);
    assert!(
        scoped.facts.iter().any(|fact| matches!(
            fact,
            SelectorFact::Relationship {
                target: RelationshipTarget::ScopeRoot(ContextId(2)),
                origin: RelationshipOrigin::Authored(_),
            }
        )),
        "the intervening scope boundary wins"
    );
}

#[test]
fn derived_relationships_never_carry_a_fabricated_anchor() {
    let (source, result) = qualify(
        ".z{@layer l{@supports (a:b){@media all{.a{.b{}}}}}}",
        55_002,
    );
    let mut derived = 0usize;
    for (index, observation) in result.observations().iter().enumerate() {
        assert!(observation.semantic_program().is_some());
        for fact in adapt_observation_program(&source, &result, index).facts {
            if let SelectorFact::Relationship { origin, .. } = fact
                && origin == RelationshipOrigin::Derived
            {
                derived += 1;
            }
            if let SelectorFact::NestingPresence { origin, .. } = fact {
                assert!(
                    matches!(origin, RelationshipOrigin::Authored(_)),
                    "a nesting presence is always authored evidence"
                );
            }
        }
    }
    assert_eq!(
        derived, 2,
        "both nested members carry implied relationships"
    );
}

// -- charged relationship traversal ---------------------------------------

fn algorithm_steps(text: &str, source_id: u64) -> usize {
    let (_, result) = qualify(text, source_id);
    result
        .resources()
        .value(CssSelectorResourceKind::AlgorithmSteps)
}

#[test]
fn every_inspected_retained_parent_costs_exactly_one_algorithm_step() {
    // The three sources retain identical selector-header token populations, so
    // every difference in selector AlgorithmSteps is relationship traversal.
    let flat = algorithm_steps(".z{.a{.b{}}}", 56_000);
    let one_group = algorithm_steps(".z{@media all{.a{.b{}}}}", 56_001);
    let two_groups = algorithm_steps(".z{@layer l{@media all{.a{.b{}}}}}", 56_002);

    assert_eq!(one_group - flat, 1);
    assert_eq!(two_groups - flat, 2);
}

#[test]
fn relationship_charge_refusal_precedes_inspection_and_commits_nothing() {
    // `.z` and `.a` each cost two window charges plus two consumption charges.
    // The ninth unit is `.a`'s single parent inspection.
    let source = SourceText::new(SourceId::new(56_100), ".z{.a{}}".to_owned());
    let refused = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        CssSelectorLimits::new(8, 128, 16, 128 * 1024).unwrap(),
    )
    .unwrap();

    assert_eq!(refused.observations().len(), 1);
    assert_eq!(
        refused.execution_completion(),
        CssSelectorExecutionCompletion::Incomplete
    );
    let CssSelectorTermination::ResourceLimit(evidence) = refused.termination() else {
        panic!("relationship traversal must refuse on AlgorithmSteps");
    };
    assert_eq!(evidence.kind(), CssSelectorResourceKind::AlgorithmSteps);

    // The refusal location is the zero-length point at the current
    // qualified-rule header start, never an ancestor-authored location.
    let header = refused.upstream_parser_result().context_records()[1]
        .header()
        .range();
    assert_eq!(evidence.location().range().start(), header.start());
    assert_eq!(evidence.location().range().end(), header.start());

    // The committed prefix and its retained usage are intact: `.z` retains one
    // observation unit plus three fact units.
    assert_eq!(
        refused
            .resources()
            .value(CssSelectorResourceKind::RetainedSemanticUnits),
        4
    );

    let granted = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        CssSelectorLimits::new(9, 128, 16, 128 * 1024).unwrap(),
    )
    .unwrap();
    assert_eq!(granted.observations().len(), 2);
    assert_eq!(
        granted.execution_completion(),
        CssSelectorExecutionCompletion::Complete
    );
}

#[test]
fn relationship_traversal_consumes_no_parser_or_tokenizer_resource() {
    let text = ".z{@layer l{@supports (a:b){@media all{.a{.b{}}}}}}";
    let structural = SourceText::new(SourceId::new(56_200), text.to_owned());
    let expected = analyze_css_source(&structural, tokenizer_limits(), parser_limits()).unwrap();

    let (_, selector) = qualify(text, 56_201);
    let actual = selector.upstream_parser_result();

    assert_eq!(actual.resources(), expected.resources());
    assert_eq!(
        actual.upstream_tokenizer_result().resources(),
        expected.upstream_tokenizer_result().resources()
    );
}

// -- retained-semantic resource accounting --------------------------------

#[test]
fn retained_unit_accounting_is_exact_for_every_outcome_class() {
    // `.a{.b{}}` retains 3 facts for `.a` and 4 for `.b`.
    let (_, qualified) = qualify(".a{.b{}}", 57_000);
    assert_eq!(
        qualified
            .resources()
            .value(CssSelectorResourceKind::RetainedSemanticUnits),
        (1 + 3) + (1 + 4)
    );

    // A non-qualified observation retains its identity unit only.
    let (_, invalid) = qualify("a,{}b{}", 57_001);
    assert_eq!(
        invalid
            .resources()
            .value(CssSelectorResourceKind::RetainedSemanticUnits),
        1 + (1 + 3)
    );
}

#[test]
fn retained_unit_refusal_preserves_the_committed_prefix_and_usage() {
    // `a{}b{}` retains 1 + 3 units per observation.
    let source = SourceText::new(SourceId::new(57_100), "a{}b{}".to_owned());
    let refused = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        CssSelectorLimits::new(200_000, 128, 16, 7).unwrap(),
    )
    .unwrap();

    assert_eq!(refused.observations().len(), 1);
    let CssSelectorTermination::ResourceLimit(evidence) = refused.termination() else {
        panic!("retained-unit exhaustion must refuse");
    };
    assert_eq!(
        evidence.kind(),
        CssSelectorResourceKind::RetainedSemanticUnits
    );
    assert_eq!(evidence.limit(), 7);
    assert_eq!(evidence.attempted(), 8);
    assert_eq!(
        refused
            .resources()
            .value(CssSelectorResourceKind::RetainedSemanticUnits),
        4
    );

    let granted = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        CssSelectorLimits::new(200_000, 128, 16, 8).unwrap(),
    )
    .unwrap();
    assert_eq!(granted.observations().len(), 2);
    assert_eq!(
        granted
            .resources()
            .value(CssSelectorResourceKind::RetainedSemanticUnits),
        8
    );
}

#[test]
fn observations_refusal_still_precedes_retained_unit_refusal() {
    // Both persistent limits would refuse the second observation; the existing
    // Observations refusal keeps precedence.
    let source = SourceText::new(SourceId::new(57_200), "a{}b{}".to_owned());
    let result = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        CssSelectorLimits::new(200_000, 128, 1, 4).unwrap(),
    )
    .unwrap();

    assert_eq!(result.observations().len(), 1);
    let CssSelectorTermination::ResourceLimit(evidence) = result.termination() else {
        panic!("the second observation must refuse");
    };
    assert_eq!(evidence.kind(), CssSelectorResourceKind::Observations);
    assert_eq!(
        result
            .resources()
            .value(CssSelectorResourceKind::RetainedSemanticUnits),
        4,
        "a refused observation charges no retained semantic unit"
    );
}

// -- candidate independence ------------------------------------------------

#[test]
fn accepted_402_authority_remains_independent_of_production() {
    for (name, module) in [
        ("gold", include_str!("selector_semantic_handoff_gold.rs")),
        (
            "reference",
            include_str!("selector_semantic_handoff_reference.rs"),
        ),
        (
            "validation",
            include_str!("selector_semantic_handoff_validation_tests.rs"),
        ),
    ] {
        for forbidden in [
            "css::selector",
            "crate::css",
            "CssSelectorSemantic",
            "analyze_css_selectors",
        ] {
            assert!(
                !module.contains(forbidden),
                "#402 {name} authority must not depend on production: found {forbidden}"
            );
        }
    }
}
