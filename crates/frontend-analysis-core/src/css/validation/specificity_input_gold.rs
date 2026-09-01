//! Candidate-independent specificity-input gold model.
//!
//! This module deliberately defines a specificity-consumer-specific linear
//! vocabulary. It does not import selector production contracts, parser state,
//! authored source handles, historical handoff types, or production helpers.

#![allow(dead_code)]

use std::collections::BTreeSet;

pub(super) const CSSWG_REVISION: &str = "b1ebca428ca1ab224f5fc1d2da5df1d493c9d282";
pub(super) const SELECTORS_4_BLOB: &str = "3b81851cdaf8ea6eec5f63e6867822de0bad9410";
pub(super) const CSS_NESTING_1_BLOB: &str = "41db452e107401cab5b8394b85213f007287a14e";
pub(super) const CSS_CASCADE_6_BLOB: &str = "8cd75053a1babf221f724781334180a842bf1d7b";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct GoldContextId(pub(super) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GoldByteRange {
    pub(super) start: usize,
    pub(super) end: usize,
}

impl GoldByteRange {
    pub(super) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct GoldSpecificity {
    pub(super) a: u32,
    pub(super) b: u32,
    pub(super) c: u32,
}

impl GoldSpecificity {
    pub(super) const ZERO: Self = Self { a: 0, b: 0, c: 0 };

    pub(super) const fn new(a: u32, b: u32, c: u32) -> Self {
        Self { a, b, c }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldSimpleKind {
    Type,
    Universal,
    Id,
    Class,
    Attribute,
    IdentifierPseudoClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldMaxKind {
    Is,
    Not,
    Has,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldRelationshipTarget {
    ParentSelectorList(GoldContextId),
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldRelationshipOrigin {
    Authored(GoldByteRange),
    Derived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldInstruction {
    BeginMember,
    EndMember,
    Simple(GoldSimpleKind),
    BeginMax(GoldMaxKind),
    EndMax(GoldMaxKind),
    WhereZero,
    Relationship {
        target: GoldRelationshipTarget,
        origin: GoldRelationshipOrigin,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldProgram {
    pub(super) owning_context: GoldContextId,
    pub(super) instructions: Vec<GoldInstruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GoldCandidateDisposition {
    Program(GoldProgram),
    DeferredByNormativeAmbiguity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldCandidate {
    pub(super) context: GoldContextId,
    pub(super) disposition: GoldCandidateDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldExpectedOutcome {
    Known(&'static [GoldSpecificity]),
    BlockedOnParent(GoldContextId),
    DeferredByNormativeAmbiguity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AuthoredRelationshipExpectation {
    pub(super) context: GoldContextId,
    pub(super) instruction_index: usize,
    pub(super) range: GoldByteRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldFixture {
    pub(super) id: &'static str,
    pub(super) source: &'static str,
    pub(super) target: GoldContextId,
    pub(super) candidates: Vec<GoldCandidate>,
    pub(super) expected: GoldExpectedOutcome,
    pub(super) authored_relationships: Vec<AuthoredRelationshipExpectation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProvenanceFailure {
    DuplicateExpectation {
        context: GoldContextId,
        instruction_index: usize,
    },
    DuplicateAuthoredRange {
        context: GoldContextId,
        instruction_index: usize,
        range: GoldByteRange,
    },
    MissingProgram(GoldContextId),
    MissingExpectation {
        context: GoldContextId,
        instruction_index: usize,
    },
    ExpectationForNonAuthoredRelationship {
        context: GoldContextId,
        instruction_index: usize,
    },
    RangeMismatch {
        context: GoldContextId,
        instruction_index: usize,
        expected: GoldByteRange,
        actual: GoldByteRange,
    },
    ReversedRange(GoldByteRange),
    OutOfBounds(GoldByteRange),
    InvalidStartBoundary(GoldByteRange),
    InvalidEndBoundary(GoldByteRange),
    AmpSpellingMismatch(GoldByteRange),
}

fn program_for_context(fixture: &GoldFixture, context: GoldContextId) -> Option<&GoldProgram> {
    fixture
        .candidates
        .iter()
        .find_map(|candidate| (candidate.context == context).then_some(&candidate.disposition))
        .and_then(|disposition| match disposition {
            GoldCandidateDisposition::Program(program) => Some(program),
            GoldCandidateDisposition::DeferredByNormativeAmbiguity => None,
        })
}

fn authored_relationship_range(instruction: &GoldInstruction) -> Option<GoldByteRange> {
    match instruction {
        GoldInstruction::Relationship {
            origin: GoldRelationshipOrigin::Authored(range),
            ..
        } => Some(*range),
        _ => None,
    }
}

pub(super) fn validate_authored_relationship_provenance(
    fixture: &GoldFixture,
) -> Result<(), ProvenanceFailure> {
    let mut seen = BTreeSet::new();
    for expectation in &fixture.authored_relationships {
        if !seen.insert((expectation.context, expectation.instruction_index)) {
            return Err(ProvenanceFailure::DuplicateExpectation {
                context: expectation.context,
                instruction_index: expectation.instruction_index,
            });
        }
    }

    let mut seen_authored_ranges = BTreeSet::new();
    for candidate in &fixture.candidates {
        let GoldCandidateDisposition::Program(program) = &candidate.disposition else {
            continue;
        };
        for (instruction_index, instruction) in program.instructions.iter().enumerate() {
            let Some(actual_range) = authored_relationship_range(instruction) else {
                continue;
            };
            if !seen_authored_ranges.insert((actual_range.start, actual_range.end)) {
                return Err(ProvenanceFailure::DuplicateAuthoredRange {
                    context: candidate.context,
                    instruction_index,
                    range: actual_range,
                });
            }
            let Some(expectation) = fixture.authored_relationships.iter().find(|expectation| {
                expectation.context == candidate.context
                    && expectation.instruction_index == instruction_index
            }) else {
                return Err(ProvenanceFailure::MissingExpectation {
                    context: candidate.context,
                    instruction_index,
                });
            };
            if expectation.range != actual_range {
                return Err(ProvenanceFailure::RangeMismatch {
                    context: candidate.context,
                    instruction_index,
                    expected: expectation.range,
                    actual: actual_range,
                });
            }
            if actual_range.start > actual_range.end {
                return Err(ProvenanceFailure::ReversedRange(actual_range));
            }
            if actual_range.end > fixture.source.len() {
                return Err(ProvenanceFailure::OutOfBounds(actual_range));
            }
            if !fixture.source.is_char_boundary(actual_range.start) {
                return Err(ProvenanceFailure::InvalidStartBoundary(actual_range));
            }
            if !fixture.source.is_char_boundary(actual_range.end) {
                return Err(ProvenanceFailure::InvalidEndBoundary(actual_range));
            }
            if &fixture.source[actual_range.start..actual_range.end] != "&" {
                return Err(ProvenanceFailure::AmpSpellingMismatch(actual_range));
            }
        }
    }

    for expectation in &fixture.authored_relationships {
        let Some(program) = program_for_context(fixture, expectation.context) else {
            return Err(ProvenanceFailure::MissingProgram(expectation.context));
        };
        if program
            .instructions
            .get(expectation.instruction_index)
            .and_then(authored_relationship_range)
            .is_none()
        {
            return Err(ProvenanceFailure::ExpectationForNonAuthoredRelationship {
                context: expectation.context,
                instruction_index: expectation.instruction_index,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod provenance_tests {
    use super::*;

    fn fixture_with_authored_ranges(source: &'static str, ranges: &[GoldByteRange]) -> GoldFixture {
        let context = GoldContextId(2);
        let mut instructions = vec![GoldInstruction::BeginMember];
        let mut authored_relationships = Vec::new();
        for range in ranges {
            let instruction_index = instructions.len();
            instructions.push(GoldInstruction::Relationship {
                target: GoldRelationshipTarget::ParentSelectorList(GoldContextId(1)),
                origin: GoldRelationshipOrigin::Authored(*range),
            });
            authored_relationships.push(AuthoredRelationshipExpectation {
                context,
                instruction_index,
                range: *range,
            });
        }
        instructions.push(GoldInstruction::EndMember);

        GoldFixture {
            id: "provenance-test",
            source,
            target: context,
            candidates: vec![GoldCandidate {
                context,
                disposition: GoldCandidateDisposition::Program(GoldProgram {
                    owning_context: context,
                    instructions,
                }),
            }],
            expected: GoldExpectedOutcome::BlockedOnParent(GoldContextId(1)),
            authored_relationships,
        }
    }

    #[test]
    fn duplicate_authored_byte_occurrence_fails_closed() {
        let range = GoldByteRange::new(5, 6);
        let fixture = fixture_with_authored_ranges("#a { &.x {} }", &[range, range]);

        assert_eq!(
            validate_authored_relationship_provenance(&fixture),
            Err(ProvenanceFailure::DuplicateAuthoredRange {
                context: GoldContextId(2),
                instruction_index: 2,
                range,
            })
        );
    }

    #[test]
    fn distinct_adjacent_authored_occurrences_remain_valid() {
        let fixture = fixture_with_authored_ranges(
            "#a { &&.x {} }",
            &[GoldByteRange::new(5, 6), GoldByteRange::new(6, 7)],
        );

        assert_eq!(validate_authored_relationship_provenance(&fixture), Ok(()));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldQualifierCompletion {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldQualifierOutcome {
    Qualified,
    Invalid,
    Unsupported,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldQualifierSnapshot {
    pub(super) completion: GoldQualifierCompletion,
    pub(super) outcomes: Vec<GoldQualifierOutcome>,
    pub(super) algorithm_steps: usize,
    pub(super) peak_selector_depth: usize,
    pub(super) observations: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SidecarLimits {
    pub(super) preparation_steps: usize,
    pub(super) retained_input_units: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SidecarCandidatePlan {
    pub(super) candidate: GoldCandidate,
    pub(super) additional_preparation_mutations: usize,
    pub(super) ancestry_records_to_inspect: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SidecarResource {
    PreparationSteps,
    RetainedInputUnits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SidecarFailure {
    Resource(SidecarResource),
    ArithmeticOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SidecarCompletion {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SidecarEvent {
    PreparationPreflight { granted: bool },
    CandidateIdentityEstablished { context: GoldContextId },
    PreparationMutation,
    AncestryPreflight { granted: bool },
    AncestryInspect,
    RetainedPreflight { required: usize, granted: bool },
    Commit { context: GoldContextId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SidecarCollection {
    pub(super) qualifier: GoldQualifierSnapshot,
    pub(super) completion: SidecarCompletion,
    pub(super) committed: Vec<GoldCandidate>,
    pub(super) preparation_steps: usize,
    pub(super) retained_input_units: usize,
    pub(super) failure: Option<SidecarFailure>,
    pub(super) events: Vec<SidecarEvent>,
}
