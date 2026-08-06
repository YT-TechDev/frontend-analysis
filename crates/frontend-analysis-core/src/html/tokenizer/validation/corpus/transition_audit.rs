//! Candidate-independent transition-step derivation audit for the initial
//! 72-fixture corpus.
//!
//! # Authority
//!
//! This audit table is authored directly against the pinned WHATWG states
//! approved in #109 and the #111 counting rule: one transition step per
//! attempted specification-state transition, including reconsume. It does
//! not consult, generate from, or compare against a production tokenizer.
//!
//! # Derivation model
//!
//! 1. Each normalized input unit, or conceptual EOF, examined under one
//!    specification state is one transition step. A reconsume instruction
//!    causes one additional examination of the same unit under another state.
//! 2. Leading-BOM skipping and input-preprocessing diagnostics are not state
//!    transitions. The resulting normalized unit still costs one step when a
//!    tokenizer state examines it.
//! 3. Examining conceptual EOF is one transition step. Emitting EOF or another
//!    token inside that same state dispatch is not an additional step.
//! 4. Unsupported-capability discovery counts as the attempted dispatch that
//!    discovered it. Coverage remains independently governed by the approved
//!    processed-prefix rule.
//! 5. A non-`TransitionSteps` resource refusal commits the examining state
//!    transition but refuses its specific sub-effect without partial mutation.
//! 6. A transition that would exceed `TransitionSteps` is attempted but does
//!    not increment committed transition usage.
//!
//! Each entry below records the committed step count. The full reviewable
//! derivation is distributed with the fixture that owns it: every PRE, TOK,
//! ERR, and ADV fixture has an ordered state trace next to construction;
//! UNSUP-001..004 have explicit traces and UNSUP-005..014 share the documented
//! context-changing-name formula; every RES fixture records preprocessing,
//! attempted/committed steps, refusal operation, attempted value, and boundary.
//! This table mechanically proves complete ID coverage and count agreement.

#[rustfmt::skip]
pub(super) const TRANSITION_STEP_AUDIT: &[(&str, usize)] = &[
    ("PRE-001", 1), ("PRE-002", 2), ("PRE-003", 3), ("PRE-004", 2),
    ("PRE-005", 3), ("PRE-006", 2), ("PRE-007", 2), ("PRE-008", 2),
    ("PRE-009", 4), ("PRE-010", 5),
    ("TOK-001", 4), ("TOK-002", 4), ("TOK-003", 5), ("TOK-004", 7),
    ("TOK-005", 6), ("TOK-006", 8), ("TOK-007", 7), ("TOK-008", 9),
    ("TOK-009", 11), ("TOK-010", 18), ("TOK-011", 18), ("TOK-012", 6),
    ("ERR-001", 2), ("ERR-002", 2), ("ERR-003", 2),
    ("ERR-004", 2), // Data('<') + TagOpen(EOF); both tokens emit in TagOpen.
    ("ERR-005", 4), // One reconsume instruction; authored '1' is examined twice.
    ("ERR-006", 2), ("ERR-007", 4), ("ERR-008", 4), ("ERR-009", 9),
    ("ERR-010", 11), ("ERR-011", 9), ("ERR-012", 13), ("ERR-013", 16),
    ("ERR-014", 12), ("ERR-015", 13), ("ERR-016", 10), ("ERR-017", 7),
    ("UNSUP-001", 1), ("UNSUP-002", 9), ("UNSUP-003", 2), ("UNSUP-004", 2),
    ("UNSUP-005", 8), ("UNSUP-006", 11), ("UNSUP-007", 8), ("UNSUP-008", 6),
    ("UNSUP-009", 9), ("UNSUP-010", 10), ("UNSUP-011", 11), ("UNSUP-012", 9),
    ("UNSUP-013", 11), ("UNSUP-014", 12),
    ("RES-001", 0), ("RES-002", 1), ("RES-003", 2), ("RES-004", 1),
    ("RES-005", 9), ("RES-006", 2),
    ("RES-007", 3), // RetainedInterpretedBytes in the active tag-name builder.
    ("RES-008", 0), ("RES-009", 0),
    ("ADV-001", 4), ("ADV-002", 4), ("ADV-003", 25), ("ADV-004", 13),
    ("ADV-005", 6), ("ADV-006", 13), ("ADV-007", 13), ("ADV-008", 8),
    ("ADV-009", 9), ("ADV-010", 65),
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::super::initial_corpus;
    use super::super::super::expected::{Completion, Resource};
    use super::TRANSITION_STEP_AUDIT;

    #[test]
    fn transition_audit_covers_every_fixture_and_matches_corpus() {
        let fixtures = initial_corpus();
        assert_eq!(fixtures.len(), 72);
        assert_eq!(fixtures.len(), TRANSITION_STEP_AUDIT.len());

        let audit_ids: BTreeSet<&str> =
            TRANSITION_STEP_AUDIT.iter().map(|(id, _)| *id).collect();
        assert_eq!(audit_ids.len(), TRANSITION_STEP_AUDIT.len());

        let corpus_ids: BTreeSet<&str> = fixtures.iter().map(|fixture| fixture.id).collect();
        assert_eq!(audit_ids, corpus_ids);

        for fixture in &fixtures {
            let (_, expected_steps) = TRANSITION_STEP_AUDIT
                .iter()
                .find(|(id, _)| *id == fixture.id)
                .unwrap_or_else(|| panic!("missing transition audit entry for {}", fixture.id));
            assert_eq!(
                fixture.expected.0.usage.transition_steps,
                *expected_steps,
                "{}: fixture usage.transition_steps must match the independent audit",
                fixture.id
            );
        }
    }

    #[test]
    fn reviewed_edge_cases_and_resource_ownership_are_fixed() {
        let steps = |id| {
            TRANSITION_STEP_AUDIT
                .iter()
                .find(|(fixture_id, _)| *fixture_id == id)
                .map(|(_, value)| *value)
                .unwrap()
        };
        assert_eq!(steps("ERR-004"), 2);
        assert_eq!(steps("ERR-005"), 4);
        assert_eq!(steps("RES-007"), 3);

        let fixtures = initial_corpus();
        let res7 = fixtures.iter().find(|fixture| fixture.id == "RES-007").unwrap();
        assert!(matches!(
            res7.expected.0.completion,
            Completion::ResourceLimit {
                resource: Resource::RetainedInterpretedBytes,
                attempted: 1,
                ..
            }
        ));
        assert!(!fixtures.iter().any(|fixture| matches!(
            fixture.expected.0.completion,
            Completion::ResourceLimit {
                resource: Resource::TemporaryBufferBytes,
                ..
            }
        )));
    }
}
