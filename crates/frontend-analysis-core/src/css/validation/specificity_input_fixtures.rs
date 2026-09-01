//! Handwritten specificity-input fixtures for the post-freeze validation gate.

#![allow(dead_code)]

use super::specificity_input_gold::{
    AuthoredRelationshipExpectation, GoldByteRange, GoldCandidate, GoldCandidateDisposition,
    GoldContextId, GoldExpectedOutcome, GoldFixture, GoldInstruction, GoldMaxKind, GoldProgram,
    GoldRelationshipOrigin, GoldRelationshipTarget, GoldSimpleKind, GoldSpecificity,
};

const C1: GoldContextId = GoldContextId(1);
const C2: GoldContextId = GoldContextId(2);
const C3: GoldContextId = GoldContextId(3);

const S000: GoldSpecificity = GoldSpecificity::new(0, 0, 0);
const S001: GoldSpecificity = GoldSpecificity::new(0, 0, 1);
const S010: GoldSpecificity = GoldSpecificity::new(0, 1, 0);
const S020: GoldSpecificity = GoldSpecificity::new(0, 2, 0);
const S100: GoldSpecificity = GoldSpecificity::new(1, 0, 0);
const S110: GoldSpecificity = GoldSpecificity::new(1, 1, 0);
const S210: GoldSpecificity = GoldSpecificity::new(2, 1, 0);

const E_V1A: &[GoldSpecificity] = &[S010];
const E_V1B: &[GoldSpecificity] = &[S100];
const E_V2: &[GoldSpecificity] = &[S100];
const E_V3: &[GoldSpecificity] = &[S000];
const E_V4A: &[GoldSpecificity] = &[S000];
const E_V6: &[GoldSpecificity] = &[S110];
const E_V7: &[GoldSpecificity] = &[S110];
const E_V8: &[GoldSpecificity] = &[S210];
const E_V9: &[GoldSpecificity] = &[S210];
const E_V10: &[GoldSpecificity] = &[S010];
const E_V11: &[GoldSpecificity] = &[S000];
const E_V12: &[GoldSpecificity] = &[S110, S000];
const E_V13A: &[GoldSpecificity] = &[S001];
const E_V13B: &[GoldSpecificity] = &[S000];
const E_V13C: &[GoldSpecificity] = &[S010];
const E_V14: &[GoldSpecificity] = &[S020];
const E_V17: &[GoldSpecificity] = &[S210];
const E_V19: &[GoldSpecificity] = &[S110];
const E_V20: &[GoldSpecificity] = &[S100];

fn program(context: GoldContextId, instructions: Vec<GoldInstruction>) -> GoldCandidate {
    GoldCandidate {
        context,
        disposition: GoldCandidateDisposition::Program(GoldProgram {
            owning_context: context,
            instructions,
        }),
    }
}

fn deferred(context: GoldContextId) -> GoldCandidate {
    GoldCandidate {
        context,
        disposition: GoldCandidateDisposition::DeferredByNormativeAmbiguity,
    }
}

fn simple(kind: GoldSimpleKind) -> Vec<GoldInstruction> {
    vec![
        GoldInstruction::BeginMember,
        GoldInstruction::Simple(kind),
        GoldInstruction::EndMember,
    ]
}

fn parent_id_program() -> GoldCandidate {
    program(C1, simple(GoldSimpleKind::Id))
}

fn derived_parent(parent: GoldContextId) -> GoldInstruction {
    GoldInstruction::Relationship {
        target: GoldRelationshipTarget::ParentSelectorList(parent),
        origin: GoldRelationshipOrigin::Derived,
    }
}

fn authored_parent(parent: GoldContextId, start: usize) -> GoldInstruction {
    GoldInstruction::Relationship {
        target: GoldRelationshipTarget::ParentSelectorList(parent),
        origin: GoldRelationshipOrigin::Authored(GoldByteRange::new(start, start + 1)),
    }
}

fn derived_zero() -> GoldInstruction {
    GoldInstruction::Relationship {
        target: GoldRelationshipTarget::Zero,
        origin: GoldRelationshipOrigin::Derived,
    }
}

fn authored_zero(start: usize) -> GoldInstruction {
    GoldInstruction::Relationship {
        target: GoldRelationshipTarget::Zero,
        origin: GoldRelationshipOrigin::Authored(GoldByteRange::new(start, start + 1)),
    }
}

pub(super) fn fixtures() -> Vec<GoldFixture> {
    vec![
        GoldFixture {
            id: "V1A-class",
            source: ".a",
            target: C1,
            candidates: vec![program(C1, simple(GoldSimpleKind::Class))],
            expected: GoldExpectedOutcome::Known(E_V1A),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V1B-id",
            source: "#a",
            target: C1,
            candidates: vec![program(C1, simple(GoldSimpleKind::Id))],
            expected: GoldExpectedOutcome::Known(E_V1B),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V1C-attribute",
            source: "[a]",
            target: C1,
            candidates: vec![program(C1, simple(GoldSimpleKind::Attribute))],
            expected: GoldExpectedOutcome::Known(E_V1A),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V1D-type",
            source: "a",
            target: C1,
            candidates: vec![program(C1, simple(GoldSimpleKind::Type))],
            expected: GoldExpectedOutcome::Known(E_V13A),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V1E-universal",
            source: "*",
            target: C1,
            candidates: vec![program(C1, simple(GoldSimpleKind::Universal))],
            expected: GoldExpectedOutcome::Known(E_V3),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V1F-identifier-pseudo-class",
            source: ":hover",
            target: C1,
            candidates: vec![program(C1, simple(GoldSimpleKind::IdentifierPseudoClass))],
            expected: GoldExpectedOutcome::Known(E_V1A),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V2-is-max",
            source: ":is(.a,#a)",
            target: C1,
            candidates: vec![program(
                C1,
                vec![
                    GoldInstruction::BeginMember,
                    GoldInstruction::BeginMax(GoldMaxKind::Is),
                    GoldInstruction::BeginMember,
                    GoldInstruction::Simple(GoldSimpleKind::Class),
                    GoldInstruction::EndMember,
                    GoldInstruction::BeginMember,
                    GoldInstruction::Simple(GoldSimpleKind::Id),
                    GoldInstruction::EndMember,
                    GoldInstruction::EndMax(GoldMaxKind::Is),
                    GoldInstruction::EndMember,
                ],
            )],
            expected: GoldExpectedOutcome::Known(E_V2),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V2-not-max",
            source: ":not(.a,#a)",
            target: C1,
            candidates: vec![program(
                C1,
                vec![
                    GoldInstruction::BeginMember,
                    GoldInstruction::BeginMax(GoldMaxKind::Not),
                    GoldInstruction::BeginMember,
                    GoldInstruction::Simple(GoldSimpleKind::Class),
                    GoldInstruction::EndMember,
                    GoldInstruction::BeginMember,
                    GoldInstruction::Simple(GoldSimpleKind::Id),
                    GoldInstruction::EndMember,
                    GoldInstruction::EndMax(GoldMaxKind::Not),
                    GoldInstruction::EndMember,
                ],
            )],
            expected: GoldExpectedOutcome::Known(E_V2),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V2-has-max",
            source: ":has(.a,#a)",
            target: C1,
            candidates: vec![program(
                C1,
                vec![
                    GoldInstruction::BeginMember,
                    GoldInstruction::BeginMax(GoldMaxKind::Has),
                    GoldInstruction::BeginMember,
                    GoldInstruction::Simple(GoldSimpleKind::Class),
                    GoldInstruction::EndMember,
                    GoldInstruction::BeginMember,
                    GoldInstruction::Simple(GoldSimpleKind::Id),
                    GoldInstruction::EndMember,
                    GoldInstruction::EndMax(GoldMaxKind::Has),
                    GoldInstruction::EndMember,
                ],
            )],
            expected: GoldExpectedOutcome::Known(E_V2),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V3-where-zero",
            source: ":where(#a,.a)",
            target: C1,
            candidates: vec![program(
                C1,
                vec![
                    GoldInstruction::BeginMember,
                    GoldInstruction::WhereZero,
                    GoldInstruction::EndMember,
                ],
            )],
            expected: GoldExpectedOutcome::Known(E_V3),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V4A-where-authored-nesting-zero",
            source: "#a { :where(&) {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        GoldInstruction::WhereZero,
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V4A),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V4B-where-no-nesting-blocked",
            source: "#a { :where(.x) {} }",
            target: C2,
            candidates: vec![program(
                C2,
                vec![
                    GoldInstruction::BeginMember,
                    derived_parent(C1),
                    GoldInstruction::WhereZero,
                    GoldInstruction::EndMember,
                ],
            )],
            expected: GoldExpectedOutcome::BlockedOnParent(C1),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V5-leading-combinator-where-authored-nesting-blocked",
            source: "#a { > :where(&) {} }",
            target: C2,
            candidates: vec![program(
                C2,
                vec![
                    GoldInstruction::BeginMember,
                    derived_parent(C1),
                    GoldInstruction::WhereZero,
                    GoldInstruction::EndMember,
                ],
            )],
            expected: GoldExpectedOutcome::BlockedOnParent(C1),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V6-implied-parent",
            source: "#a { .x {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        derived_parent(C1),
                        GoldInstruction::Simple(GoldSimpleKind::Class),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V6),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V7-authored-parent",
            source: "#a { &.x {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        authored_parent(C1, 5),
                        GoldInstruction::Simple(GoldSimpleKind::Class),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V7),
            authored_relationships: vec![AuthoredRelationshipExpectation {
                context: C2,
                instruction_index: 1,
                range: GoldByteRange::new(5, 6),
            }],
        },
        GoldFixture {
            id: "V8-derived-plus-authored",
            source: "#a { > &.x {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        derived_parent(C1),
                        authored_parent(C1, 7),
                        GoldInstruction::Simple(GoldSimpleKind::Class),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V8),
            authored_relationships: vec![AuthoredRelationshipExpectation {
                context: C2,
                instruction_index: 2,
                range: GoldByteRange::new(7, 8),
            }],
        },
        GoldFixture {
            id: "V9-two-authored",
            source: "#a { &&.x {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        authored_parent(C1, 5),
                        authored_parent(C1, 6),
                        GoldInstruction::Simple(GoldSimpleKind::Class),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V9),
            authored_relationships: vec![
                AuthoredRelationshipExpectation {
                    context: C2,
                    instruction_index: 1,
                    range: GoldByteRange::new(5, 6),
                },
                AuthoredRelationshipExpectation {
                    context: C2,
                    instruction_index: 2,
                    range: GoldByteRange::new(6, 7),
                },
            ],
        },
        GoldFixture {
            id: "V9B-max-does-not-sum-authored-relationships-across-members",
            source: "#a { :is(&, &) {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        GoldInstruction::BeginMax(GoldMaxKind::Is),
                        GoldInstruction::BeginMember,
                        authored_parent(C1, 9),
                        GoldInstruction::EndMember,
                        GoldInstruction::BeginMember,
                        authored_parent(C1, 12),
                        GoldInstruction::EndMember,
                        GoldInstruction::EndMax(GoldMaxKind::Is),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V1B),
            authored_relationships: vec![
                AuthoredRelationshipExpectation {
                    context: C2,
                    instruction_index: 3,
                    range: GoldByteRange::new(9, 10),
                },
                AuthoredRelationshipExpectation {
                    context: C2,
                    instruction_index: 6,
                    range: GoldByteRange::new(12, 13),
                },
            ],
        },
        GoldFixture {
            id: "V10-forgiving-invalid-nesting-presence",
            source: "#a { :is(&Bar, .x) {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        GoldInstruction::BeginMax(GoldMaxKind::Is),
                        GoldInstruction::BeginMember,
                        GoldInstruction::Simple(GoldSimpleKind::Class),
                        GoldInstruction::EndMember,
                        GoldInstruction::EndMax(GoldMaxKind::Is),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V10),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V11-forgiving-empty-max",
            source: "#a { :is(&Bar) {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        GoldInstruction::BeginMax(GoldMaxKind::Is),
                        GoldInstruction::EndMax(GoldMaxKind::Is),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V11),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V12-member-local-relative-classification",
            source: "#a { .x, :where(&) {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        derived_parent(C1),
                        GoldInstruction::Simple(GoldSimpleKind::Class),
                        GoldInstruction::EndMember,
                        GoldInstruction::BeginMember,
                        GoldInstruction::WhereZero,
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V12),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V13A-scope-prelude-zero",
            source: "@scope (#s) { img {} }",
            target: C1,
            candidates: vec![program(
                C1,
                vec![
                    GoldInstruction::BeginMember,
                    derived_zero(),
                    GoldInstruction::Simple(GoldSimpleKind::Type),
                    GoldInstruction::EndMember,
                ],
            )],
            expected: GoldExpectedOutcome::Known(E_V13A),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V13B-scoped-authored-nesting-zero",
            source: "@scope (#s) { & {} }",
            target: C1,
            candidates: vec![program(
                C1,
                vec![
                    GoldInstruction::BeginMember,
                    authored_zero(14),
                    GoldInstruction::EndMember,
                ],
            )],
            expected: GoldExpectedOutcome::Known(E_V13B),
            authored_relationships: vec![AuthoredRelationshipExpectation {
                context: C1,
                instruction_index: 1,
                range: GoldByteRange::new(14, 15),
            }],
        },
        GoldFixture {
            id: "V13C-explicit-scope-pseudo",
            source: "@scope (#s) { :scope {} }",
            target: C1,
            candidates: vec![program(
                C1,
                vec![
                    GoldInstruction::BeginMember,
                    derived_zero(),
                    GoldInstruction::Simple(GoldSimpleKind::IdentifierPseudoClass),
                    GoldInstruction::EndMember,
                ],
            )],
            expected: GoldExpectedOutcome::Known(E_V13C),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V14-scope-qualified-qualified",
            source: "@scope (.s) { .p { .q {} } }",
            target: C2,
            candidates: vec![
                program(
                    C1,
                    vec![
                        GoldInstruction::BeginMember,
                        derived_zero(),
                        GoldInstruction::Simple(GoldSimpleKind::Class),
                        GoldInstruction::EndMember,
                    ],
                ),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        derived_parent(C1),
                        GoldInstruction::Simple(GoldSimpleKind::Class),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V14),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V15-qualified-scope-qualified-deferred",
            source: ".outer { @scope (.s) { .inner {} } }",
            target: C3,
            candidates: vec![deferred(C3)],
            expected: GoldExpectedOutcome::DeferredByNormativeAmbiguity,
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V17-utf8-authored-anchor",
            source: "#é { > &.x {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        derived_parent(C1),
                        authored_parent(C1, 8),
                        GoldInstruction::Simple(GoldSimpleKind::Class),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V17),
            authored_relationships: vec![AuthoredRelationshipExpectation {
                context: C2,
                instruction_index: 2,
                range: GoldByteRange::new(8, 9),
            }],
        },
        GoldFixture {
            id: "V19-has-relative-combinator-does-not-add-root-relationship",
            source: "#a { :has(> .x) {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        derived_parent(C1),
                        GoldInstruction::BeginMax(GoldMaxKind::Has),
                        GoldInstruction::BeginMember,
                        GoldInstruction::Simple(GoldSimpleKind::Class),
                        GoldInstruction::EndMember,
                        GoldInstruction::EndMax(GoldMaxKind::Has),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V19),
            authored_relationships: vec![],
        },
        GoldFixture {
            id: "V20-has-authored-nesting-suppresses-root-implied",
            source: "#a { :has(> &) {} }",
            target: C2,
            candidates: vec![
                parent_id_program(),
                program(
                    C2,
                    vec![
                        GoldInstruction::BeginMember,
                        GoldInstruction::BeginMax(GoldMaxKind::Has),
                        GoldInstruction::BeginMember,
                        authored_parent(C1, 12),
                        GoldInstruction::EndMember,
                        GoldInstruction::EndMax(GoldMaxKind::Has),
                        GoldInstruction::EndMember,
                    ],
                ),
            ],
            expected: GoldExpectedOutcome::Known(E_V20),
            authored_relationships: vec![AuthoredRelationshipExpectation {
                context: C2,
                instruction_index: 3,
                range: GoldByteRange::new(12, 13),
            }],
        },
    ]
}
