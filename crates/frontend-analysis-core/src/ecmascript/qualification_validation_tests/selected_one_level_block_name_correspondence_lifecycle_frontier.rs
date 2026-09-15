//! Candidate-independent one-level Block accepted-witness correspondence
//! lifecycle oracle for Issue #705.
//!
//! Hierarchy: parent program #108; source-name correspondence foundation
//! #314/#315 -> #316/#317; top-level var widening #322/#323 -> #324/#325;
//! direct top-level var `IdentifierReference` validation #330/#331 ->
//! multi-reference composition #332/#333 -> production #334/#335; one-level
//! Block var research #688 -> #689/#690 -> #691/#692 -> #693/#694 ->
//! #695/#696 -> #697/#698 -> #699/#700; all-selected-var contributor domain
//! widening #701/#702 -> #703/#704.
//!
//! Focused research verdict: **PREDECESSOR FRONTIER CONFIRMED**.
//!
//! This oracle freezes exactly the theorem:
//!
//! ```text
//! SelectedOneLevelBlockStaticSemanticsAccepted
//! + existing selected lexical IdentifierReference inputs
//! + existing source-name correspondence meanings
//! + existing all-selected-var contributor domain
//! -> source-name correspondence results
//! ```
//!
//! Production currently recognizes a bare one-level Block source (for
//! example `{ var a; let x=a; }`) as
//! `SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice` ->
//! `SelectedOneLevelBlockStaticSemanticsOutcome::Accepted`, and that branch
//! of `attempt_selected_qualification` never calls
//! `analyze_selected_variable_statement_name_correspondence` -- only the
//! sibling `RecognizedVariableStatementSlice` ->
//! `SelectedVariableStatementStaticSemanticsOutcome::Accepted` branch does.
//! This is the load-bearing lifecycle gap: an otherwise-identical source
//! with an unrelated leading `var q;` reaches a *different* accepted
//! witness (`RecognizedVariableStatementSlice`) purely because a top-level
//! `VariableStatement` now exists, even though `q` is inert for every other
//! queried name. This oracle freezes, independently of production, what the
//! correspondence result for the `OneLevelBlock` witness must be, and shows
//! that it does not change when an unrelated `var q;` moves the witness.
//!
//! It freezes no new correspondence meaning (still exactly
//! `VisibleSelectedLexicalBinding`, `SameSourceSelectedVarNameContributors`,
//! `NoSelectedSameSourceContributor`), no new Early Error identity, no
//! Block-var `IdentifierReference` RHS grammar, and no accepted-witness
//! promotion or collapse. It does not implement, add, or call a production
//! correspondence entrypoint.
//!
//! This module deliberately does not import, call, or otherwise derive
//! expected results from
//! `super::super::selected_variable_statement_name_correspondence`,
//! `selected_lexical_slice`, `selected_static_semantics`,
//! `selected_qualification_integration`, `selected_binding_scope`, or
//! `selected_one_level_block_binding_scope`. Every expected anchor,
//! semantic name, lifecycle classification, and ordered contributor list
//! below is independently authored fixture fact, verified only through the
//! already-accepted Core source-anchor contract (`SourceText::anchor`),
//! never through source search, rescanning, retokenization, reparsing, or a
//! second tokenizer.
//!
//! Wrong-model discrimination (see Issue #705 for the full rationale):
//!
//! ```text
//! W1  Block-only correspondence has no meaning         -> F1, F3, F7, F8
//!     until Block-var RHS exists
//! W2  a Block-only Script must be promoted to           -> all F1-F12
//!     VariableStatement witness
//! W3  Block var contributor matters only when a         -> F1
//!     top-level var exists
//! W4  Block var contributor is same-Block-only           -> F4 (killed by
//!                                                            the #701
//!                                                            frontier;
//!                                                            reused here
//!                                                            uniformly)
//! W5  top-level lexical reference cannot see Block       -> F4
//!     var contributor
//! W6  same semantic name deduplicates                    -> F9
//! W7  escaped contributor equality uses authored          -> F10
//!     spelling only
//! W8  Unicode canonical normalization creates             -> F11
//!     equality
//! W9  decimal RHS becomes correspondence evidence         -> F12
//! W10 var contributors outrank current-region             -> F8
//!     lexical binding
//! W11 var contributors outrank Block-origin top-level     -> F7
//!     lexical fallback
//! W12 statically rejected source may still commit         -> F6
//!     correspondence
//! W13 accepted correspondence requires at least one       -> F2
//!     relation
//! W14 unrelated top-level q changes correspondence        -> F1/F1Q,
//!     semantics for unrelated name a                         F5/F5Q,
//!                                                            F7/F7Q
//! ```

use super::inventory::{CONTAINERS, RULE_UNITS, RuleUnitKind};
use crate::{SourceId, SourceText};

const ISSUE_ID: u64 = 705;
const REQUIRED_RULE_IDS: &[&str] = &[
    "EE-01-R01",
    "EE-01-R02",
    "EE-04-R08",
    "EE-14-R01",
    "EE-14-R02",
    "EE-15-R01",
    "EE-15-R02",
    "EE-15-R03",
    "EE-36-R01",
    "EE-36-R02",
];

/// One exact authored occurrence: half-open byte range plus raw authored
/// spelling (never the decoded semantic name for an escaped identifier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct A(usize, usize, &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Region {
    TopLevel,
    Block,
}

/// Whether a resolved lexical target lives in the reference's own current
/// region, or was reached only through the existing Block-origin top-level
/// fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexicalOrigin {
    SameRegion,
    TopLevelFallback,
}

/// The three existing, immutable correspondence meanings. No fourth
/// meaning is introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Relation {
    /// `VisibleSelectedLexicalBinding`.
    Lexical(A, LexicalOrigin),
    /// `SameSourceSelectedVarNameContributors`.
    Vars(&'static [A]),
    /// `NoSelectedSameSourceContributor`.
    None,
}

/// One expected correspondence relation for one existing reference input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct R {
    binding: A,
    reference: A,
    name: &'static str,
    region: Region,
    relation: Relation,
}

/// The two distinct accepted-witness lifecycle proofs. Never encoded as
/// equal, and no fixture below asserts a promotion/collapse between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedAcceptedWitness {
    OneLevelBlockStaticSemanticsAccepted,
    VariableStatementStaticSemanticsAccepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedLifecycle {
    /// Accepted static semantics reached the stated witness. A correspondence
    /// completion of zero relations is representable and distinct from
    /// absent fixture data (see `F2`).
    Accepted(ExpectedAcceptedWitness),
    /// Static semantics rejected the whole source before any correspondence
    /// relation could be committed. `relations` must be empty for such a
    /// fixture.
    StaticRejected { rule_id: &'static str, subject: A },
}

#[derive(Debug, Clone, Copy)]
struct F {
    id: &'static str,
    source: &'static str,
    lifecycle: ExpectedLifecycle,
    relations: &'static [R],
    /// Extra literal anchors this fixture wants audited that are not
    /// already reachable through `relations` (for example, the non-primary
    /// half of a `StaticRejected` collision pair).
    additional_anchors: &'static [A],
}

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(ISSUE_ID), text.to_owned())
}

fn assert_anchor(src: &SourceText, expected: A) {
    let anchor = src.anchor(expected.0, expected.1).expect("fixture range");
    assert_eq!(anchor.range().start(), expected.0, "{expected:?} start");
    assert_eq!(anchor.range().end(), expected.1, "{expected:?} end");
    assert_eq!(anchor.fragment(), expected.2, "{expected:?} fragment");
}

// F1 -- the central lifecycle fixture: a same-Block Block-var contributor
// observed by an existing Block lexical reference, under the
// `OneLevelBlock` accepted witness (kills W1, W2, W3).
const F1_VARS: &[A] = &[A(6, 7, "a")];
const F1: F = F {
    id: "same-block-var-seen-by-block-lexical-reference",
    source: "{ var a; let x=a; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(13, 14, "x"),
        reference: A(15, 16, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Vars(F1_VARS),
    }],
    additional_anchors: &[],
};

// F2 -- an accepted `OneLevelBlock` witness with zero reference inputs
// still freezes a completed, empty correspondence result
// (`Complete([])`), never absence of fixture data (kills W13).
const F2: F = F {
    id: "accepted-empty-correspondence-completion",
    source: "{ var a; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[],
    additional_anchors: &[],
};

// F3 -- `NoSelectedSameSourceContributor` inside a Block whose only item is
// a lexical declaration referencing an unrelated, undeclared name.
const F3: F = F {
    id: "no-selected-same-source-contributor-inside-block",
    source: "{ let x=a; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(6, 7, "x"),
        reference: A(8, 9, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::None,
    }],
    additional_anchors: &[],
};

// F4 -- a Block-var contributor observed by an existing *top-level* lexical
// reference input, under the same `OneLevelBlock` witness (a top-level
// lexical declaration alone does not introduce a `VariableStatement`).
// Defeats a model where Block contributors are visible only to Block-origin
// references (kills W4, W5).
const F4_VARS: &[A] = &[A(6, 7, "a")];
const F4: F = F {
    id: "block-var-seen-by-top-level-lexical-reference",
    source: "{ var a; }\nlet x=a;",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(15, 16, "x"),
        reference: A(17, 18, "a"),
        name: "a",
        region: Region::TopLevel,
        relation: Relation::Vars(F4_VARS),
    }],
    additional_anchors: &[],
};

// F5 -- a same-Block Block-var contributor of an unrelated semantic name
// must not fabricate a false-positive match for a different queried name.
const F5: F = F {
    id: "same-block-unrelated-reference-no-false-positive",
    source: "{ var a; let x=b; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(13, 14, "x"),
        reference: A(15, 16, "b"),
        name: "b",
        region: Region::Block,
        relation: Relation::None,
    }],
    additional_anchors: &[],
};

// F6 -- the existing Block lexical/var collision (`EE-14-R02`: Block
// `LexicallyDeclaredNames` intersects Block `VarDeclaredNames`) statically
// rejects the whole source before any correspondence relation for `let
// a=1;` could ever be committed (kills W12). This cites existing frozen
// `EE-14-R02` authority; it introduces no new Early Error identity.
const F6: F = F {
    id: "static-rejection-gates-correspondence-commitment",
    source: "{ var a; let a=1; }",
    lifecycle: ExpectedLifecycle::StaticRejected {
        rule_id: "EE-14-R02",
        subject: A(13, 14, "a"),
    },
    relations: &[],
    additional_anchors: &[A(6, 7, "a")],
};

// F7 -- the existing Block-origin top-level lexical fallback is reachable
// from a `OneLevelBlock` witness exactly as before: no var contributor may
// override it (kills W11).
const F7_LEXICAL: A = A(4, 5, "a");
const F7: F = F {
    id: "block-origin-top-level-lexical-fallback",
    source: "let a=1;\n{ let x=a; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(15, 16, "x"),
        reference: A(17, 18, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Lexical(F7_LEXICAL, LexicalOrigin::TopLevelFallback),
    }],
    additional_anchors: &[],
};

// F8 -- current-region (Block) lexical precedence outranks a same-named
// widened var contributor authored elsewhere (kills W10). No Block var
// exists in this fixture at all; it isolates lexical-vs-lexical precedence
// under the `OneLevelBlock` witness.
const F8_LEXICAL: A = A(6, 7, "a");
const F8: F = F {
    id: "current-region-lexical-precedence",
    source: "{ let a=1; let x=a; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(15, 16, "x"),
        reference: A(17, 18, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Lexical(F8_LEXICAL, LexicalOrigin::SameRegion),
    }],
    additional_anchors: &[],
};

// F9 -- repeated Block-var contributors of one authored `var` statement
// remain two distinct authored occurrences, in exact authored order,
// never deduplicated (kills W6).
const F9_VARS: &[A] = &[A(6, 7, "a"), A(8, 9, "a")];
const F9: F = F {
    id: "repeated-block-var-contributors-same-statement",
    source: "{ var a,a; let x=a; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(15, 16, "x"),
        reference: A(17, 18, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Vars(F9_VARS),
    }],
    additional_anchors: &[],
};

// F10 -- an escaped Block-var LHS: the authored escaped spelling and the
// decoded semantic name remain distinct facts (kills W7). The braced
// CodePointEscape spelling is the already-accepted alternate
// `UnicodeEscapeSequence` spelling for the selected `BindingIdentifier`
// grammar (already used for the accepted Block-var escaped-identifier
// fixture in the #701 predecessor).
const F10_VARS: &[A] = &[A(6, 12, r"\u{61}")];
const F10: F = F {
    id: "escaped-block-var-lhs-semantic-equality",
    source: r"{ var \u{61}; let x=a; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(18, 19, "x"),
        reference: A(20, 21, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Vars(F10_VARS),
    }],
    additional_anchors: &[],
};

// F11 -- no Unicode normalization: a composed Block-var contributor matches
// only a composed reference; a code-point-distinct decomposed reference of
// the same visual name matches nothing (kills W8). The decomposed
// reference is authored as an unescaped "e" followed by a braced
// combining-acute `CodePointEscape`, which decodes to the two-code-point
// sequence e + U+0301 rather than the single-code-point precomposed
// U+00E9.
const F11_VARS: &[A] = &[A(6, 8, "é")];
const F11: F = F {
    id: "unicode-composed-matches-decomposed-does-not",
    source: r"{ var é; let x=é; let y=e\u{301}; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[
        R {
            binding: A(14, 15, "x"),
            reference: A(16, 18, "é"),
            name: "é",
            region: Region::Block,
            relation: Relation::Vars(F11_VARS),
        },
        R {
            binding: A(24, 25, "y"),
            reference: A(26, 34, r"e\u{301}"),
            name: "e\u{0301}",
            region: Region::Block,
            relation: Relation::None,
        },
    ],
    additional_anchors: &[],
};

// F12 -- a decimal-initialized Block var contributes through its LHS only;
// the decimal RHS is never contributor or reference evidence (kills W9).
const F12_VARS: &[A] = &[A(6, 7, "a")];
const F12: F = F {
    id: "decimal-initialized-block-var-contributes-via-lhs-only",
    source: "{ var a=1; let x=a; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(15, 16, "x"),
        reference: A(17, 18, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Vars(F12_VARS),
    }],
    additional_anchors: &[],
};

// --- Q-prefix witness-route equivalence (load-bearing) ---------------------
//
// `S` and `var q;\nS` currently reach *different* accepted-witness
// lifecycle proofs even when `q` is irrelevant to the queried name: adding
// an unrelated top-level `var q;` moves recognition from
// `RecognizedOneLevelBlockSlice` to `RecognizedVariableStatementSlice`, and
// therefore the accepted witness from `OneLevelBlockStaticSemanticsAccepted`
// to `VariableStatementStaticSemanticsAccepted`. The witness identity is
// NOT claimed invariant across the prefix; only the correspondence
// *projection* for the unrelated queried name is (kills W14). Absolute
// byte anchors are never compared raw across `S` and the prefixed source;
// every prefixed anchor below is independently authored fixture fact, and
// its exact offset shift from the corresponding `S`-local anchor is
// verified against the literal prefix length rather than assumed.

const Q_PREFIX: &str = "var q;\n";
const Q_PREFIX_LEN: usize = 7;

const _: () = assert!(Q_PREFIX.len() == Q_PREFIX_LEN);

// F1Q -- the central required pair: the same-Block Block-var contributor
// relation for `a`, unaffected by the unrelated leading `var q;`.
const F1Q_VARS: &[A] = &[A(13, 14, "a")];
const F1Q: F = F {
    id: "q-prefixed-same-block-var-seen-by-block-lexical-reference",
    source: "var q;\n{ var a; let x=a; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::VariableStatementStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(20, 21, "x"),
        reference: A(22, 23, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Vars(F1Q_VARS),
    }],
    additional_anchors: &[],
};

// F5Q -- the `NoSelectedSameSourceContributor` projection is likewise
// unaffected by the unrelated leading `var q;`.
const F5Q: F = F {
    id: "q-prefixed-same-block-unrelated-reference-no-false-positive",
    source: "var q;\n{ var a; let x=b; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::VariableStatementStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(20, 21, "x"),
        reference: A(22, 23, "b"),
        name: "b",
        region: Region::Block,
        relation: Relation::None,
    }],
    additional_anchors: &[],
};

// F7Q -- the Block-origin top-level lexical fallback projection is
// likewise unaffected by the unrelated leading `var q;`.
const F7Q_LEXICAL: A = A(11, 12, "a");
const F7Q: F = F {
    id: "q-prefixed-block-origin-top-level-lexical-fallback",
    source: "var q;\nlet a=1;\n{ let x=a; }",
    lifecycle: ExpectedLifecycle::Accepted(
        ExpectedAcceptedWitness::VariableStatementStaticSemanticsAccepted,
    ),
    relations: &[R {
        binding: A(22, 23, "x"),
        reference: A(24, 25, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Lexical(F7Q_LEXICAL, LexicalOrigin::TopLevelFallback),
    }],
    additional_anchors: &[],
};

const Q_PREFIX_PAIRS: &[F] = &[F1Q, F5Q, F7Q];

const ALL_FIXTURES: &[F] = &[
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F1Q, F5Q, F7Q,
];

fn assert_offset_shift(base: A, prefixed: A) {
    assert_eq!(
        prefixed.0,
        base.0 + Q_PREFIX_LEN,
        "start shift for {base:?} -> {prefixed:?}"
    );
    assert_eq!(
        prefixed.1,
        base.1 + Q_PREFIX_LEN,
        "end shift for {base:?} -> {prefixed:?}"
    );
    assert_eq!(
        prefixed.2, base.2,
        "authored fragment must be unchanged by the prefix"
    );
}

#[test]
fn fixture_anchor_and_semantic_name_audit() {
    let mut audited_fixtures = 0usize;
    let mut audited_anchors = 0usize;

    for fixture in ALL_FIXTURES {
        audited_fixtures += 1;
        let src = source(fixture.source);

        for relation in fixture.relations {
            assert_anchor(&src, relation.binding);
            audited_anchors += 1;
            assert_anchor(&src, relation.reference);
            audited_anchors += 1;

            match relation.relation {
                Relation::Lexical(anchor, _) => {
                    assert_anchor(&src, anchor);
                    audited_anchors += 1;
                }
                Relation::Vars(contributors) => {
                    for contributor in contributors {
                        assert_anchor(&src, *contributor);
                        audited_anchors += 1;
                    }
                }
                Relation::None => {}
            }
        }

        if let ExpectedLifecycle::StaticRejected { subject, .. } = fixture.lifecycle {
            assert_anchor(&src, subject);
            audited_anchors += 1;
        }

        for extra in fixture.additional_anchors {
            assert_anchor(&src, *extra);
            audited_anchors += 1;
        }
    }

    // Unescaped, non-normalized references carry a raw fragment identical
    // to their semantic name; F10/F11 intentionally break that identity
    // and are audited separately below.
    for fixture in ALL_FIXTURES {
        if fixture.id == F10.id || fixture.id == F11.id {
            continue;
        }
        for relation in fixture.relations {
            assert_eq!(relation.reference.2, relation.name);
        }
    }

    assert_eq!(audited_fixtures, ALL_FIXTURES.len());
    assert_eq!(audited_fixtures, 15);
    assert_eq!(audited_anchors, 41);
}

#[test]
fn one_level_block_lifecycle_is_reachable_from_existing_lexical_reference_inputs() {
    // F1: a Block-only source reaches source-name correspondence for a
    // Block lexical reference without requiring any top-level
    // `VariableStatement` to exist (kills W1, W2, W3).
    assert_eq!(
        F1.lifecycle,
        ExpectedLifecycle::Accepted(ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted)
    );
    assert_eq!(F1.relations.len(), 1);
    assert_eq!(F1.relations[0].relation, Relation::Vars(F1_VARS));
}

#[test]
fn accepted_empty_correspondence_is_a_completed_result_not_absent_data() {
    // F2: zero reference inputs still yields a completed correspondence
    // result of zero relations under the `OneLevelBlock` witness (kills
    // W13).
    assert_eq!(
        F2.lifecycle,
        ExpectedLifecycle::Accepted(ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted)
    );
    assert!(F2.relations.is_empty());
}

#[test]
fn no_selected_same_source_contributor_remains_representable_inside_a_block() {
    // F3: an unrelated, undeclared reference is `NoSelectedSameSourceContributor`,
    // not a static failure.
    assert_eq!(F3.relations[0].relation, Relation::None);

    // F5: the presence of a same-Block Block-var contributor of a
    // different name must not fabricate a match.
    assert_eq!(F5.relations[0].relation, Relation::None);
    assert_ne!(F5.relations[0].name, "a");
}

#[test]
fn block_var_contributor_is_visible_to_top_level_lexical_input_under_the_same_witness() {
    // F4: a Block-var contributor is seen by an existing top-level lexical
    // reference input, and the whole source still reaches the
    // `OneLevelBlock` witness because no top-level `VariableStatement` is
    // present (kills W4, W5).
    assert_eq!(
        F4.lifecycle,
        ExpectedLifecycle::Accepted(ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted)
    );
    assert_eq!(F4.relations[0].region, Region::TopLevel);
    assert_eq!(F4.relations[0].relation, Relation::Vars(F4_VARS));
}

#[test]
fn static_rejection_gates_correspondence_commitment() {
    // F6: the whole source is statically rejected before any
    // correspondence relation for `let a=1;` could be committed (kills
    // W12). This is existing frozen `EE-14-R02` authority, not a new
    // identity.
    let ExpectedLifecycle::StaticRejected { rule_id, subject } = F6.lifecycle else {
        panic!("F6 must be statically rejected, never Accepted + Complete(...)");
    };
    assert_eq!(rule_id, "EE-14-R02");
    assert_eq!(subject, A(13, 14, "a"));
    assert!(
        F6.relations.is_empty(),
        "a statically rejected fixture must carry no committed correspondence relations"
    );

    let rejecting_rule = RULE_UNITS
        .iter()
        .find(|rule| rule.id == "EE-14-R02")
        .expect("EE-14-R02 must remain frozen inventory authority");
    assert_eq!(rejecting_rule.kind, RuleUnitKind::NormativeRule);
}

#[test]
fn block_origin_top_level_lexical_fallback_is_unchanged() {
    // F7: the existing Block-origin top-level lexical fallback remains
    // reachable from the `OneLevelBlock` witness; no var contributor may
    // override it (kills W11).
    match F7.relations[0].relation {
        Relation::Lexical(anchor, LexicalOrigin::TopLevelFallback) => {
            assert_eq!(anchor, F7_LEXICAL);
        }
        other => panic!("expected top-level lexical fallback, got {other:?}"),
    }
}

#[test]
fn current_region_lexical_precedence_is_unchanged() {
    // F8: current-region (Block) lexical precedence outranks anything else
    // of the same name (kills W10).
    match F8.relations[0].relation {
        Relation::Lexical(anchor, LexicalOrigin::SameRegion) => {
            assert_eq!(anchor, F8_LEXICAL);
        }
        other => panic!("expected same-region lexical precedence, got {other:?}"),
    }
}

#[test]
fn repeated_block_var_contributors_are_never_deduplicated() {
    // F9: two declarators of one Block `var` statement remain two distinct
    // authored occurrences of the same semantic name, in exact authored
    // order (kills W6).
    assert_eq!(F9_VARS.len(), 2);
    assert_ne!(F9_VARS[0], F9_VARS[1]);
    assert_eq!(F9_VARS[0].2, F9_VARS[1].2);
    assert!(F9_VARS[0].0 < F9_VARS[1].0);
}

#[test]
fn escaped_lhs_authored_spelling_and_semantic_name_remain_distinct() {
    // F10: the Block-var contributor anchor keeps its exact authored
    // escaped spelling; the reference's semantic name is the decoded "a"
    // (kills W7).
    assert_eq!(F10_VARS[0].2, r"\u{61}");
    assert_ne!(F10_VARS[0].2, "a");
    assert_eq!(F10.relations[0].name, "a");
    assert_eq!(F10.relations[0].relation, Relation::Vars(F10_VARS));
}

#[test]
fn unicode_composed_and_decomposed_never_normalize_equal() {
    // F11: a composed contributor matches only a composed reference of the
    // identical code point sequence; a code-point-distinct decomposed
    // reference of the same visual name never matches (kills W8).
    assert_eq!(F11.relations[0].relation, Relation::Vars(F11_VARS));

    let decomposed_name = F11.relations[1].name;
    let composed_name = F11_VARS[0].2;
    assert_ne!(decomposed_name, composed_name);
    assert_eq!(decomposed_name.chars().count(), 2);
    assert_eq!(composed_name.chars().count(), 1);
    assert_eq!(F11.relations[1].relation, Relation::None);
}

#[test]
fn decimal_initializer_never_becomes_contributor_evidence() {
    // F12: the decimal RHS anchor (8, 9, "1") never appears as a
    // contributor or reference anchor anywhere in the expected relation
    // (kills W9).
    assert_eq!(F12.relations[0].relation, Relation::Vars(F12_VARS));
    assert_eq!(F12_VARS, &[A(6, 7, "a")]);
    assert_ne!(F12_VARS[0], A(8, 9, "1"));
}

#[test]
fn block_var_rhs_identifier_reference_remains_outside_the_frontier() {
    // No fixture introduces a Block-var `IdentifierReference` initializer
    // (`{ var x=a; }`); every Block `var` declarator above is bare,
    // multi-declarator bare, decimal-initialized, or escaped-bare, matching
    // the accepted #688-#704 Block-var grammar exactly.
    for fixture in ALL_FIXTURES {
        assert!(
            !fixture.source.contains("var a=a")
                && !fixture.source.contains("var x=a")
                && !fixture.source.contains(r"var \u{61}=a"),
            "{} must not introduce a Block-var IdentifierReference initializer",
            fixture.id
        );
    }
}

#[test]
fn accepted_witness_identities_remain_distinct_and_unpromoted() {
    assert_ne!(
        ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted,
        ExpectedAcceptedWitness::VariableStatementStaticSemanticsAccepted
    );

    for fixture in [F1, F2, F3, F4, F5, F7, F8, F9, F10, F11, F12] {
        assert_eq!(
            fixture.lifecycle,
            ExpectedLifecycle::Accepted(
                ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted
            ),
            "{} must reach the OneLevelBlock witness, never VariableStatement",
            fixture.id
        );
    }
    for fixture in Q_PREFIX_PAIRS {
        assert_eq!(
            fixture.lifecycle,
            ExpectedLifecycle::Accepted(
                ExpectedAcceptedWitness::VariableStatementStaticSemanticsAccepted
            ),
            "{} must reach the VariableStatement witness once an unrelated top-level var exists",
            fixture.id
        );
    }
}

#[test]
fn q_prefix_offset_shift_is_exactly_the_literal_prefix_length() {
    assert_eq!(Q_PREFIX, "var q;\n");
    assert_eq!(Q_PREFIX.len(), Q_PREFIX_LEN);
    assert_eq!(Q_PREFIX_LEN, 7);

    for (base, prefixed) in [
        (F1.source, F1Q.source),
        (F5.source, F5Q.source),
        (F7.source, F7Q.source),
    ] {
        assert_eq!(format!("{Q_PREFIX}{base}"), prefixed);
    }

    assert_offset_shift(F1.relations[0].binding, F1Q.relations[0].binding);
    assert_offset_shift(F1.relations[0].reference, F1Q.relations[0].reference);
    match (F1.relations[0].relation, F1Q.relations[0].relation) {
        (Relation::Vars(base), Relation::Vars(prefixed)) => {
            assert_eq!(base.len(), prefixed.len());
            for (base_anchor, prefixed_anchor) in base.iter().zip(prefixed.iter()) {
                assert_offset_shift(*base_anchor, *prefixed_anchor);
            }
        }
        other => panic!("F1/F1Q must both carry SameSourceSelectedVarNameContributors: {other:?}"),
    }

    assert_offset_shift(F5.relations[0].binding, F5Q.relations[0].binding);
    assert_offset_shift(F5.relations[0].reference, F5Q.relations[0].reference);

    assert_offset_shift(F7.relations[0].binding, F7Q.relations[0].binding);
    assert_offset_shift(F7.relations[0].reference, F7Q.relations[0].reference);
    match (F7.relations[0].relation, F7Q.relations[0].relation) {
        (Relation::Lexical(base, base_origin), Relation::Lexical(prefixed, prefixed_origin)) => {
            assert_offset_shift(base, prefixed);
            assert_eq!(base_origin, prefixed_origin);
        }
        other => panic!("F7/F7Q must both carry VisibleSelectedLexicalBinding: {other:?}"),
    }
}

#[test]
fn q_prefix_correspondence_projection_is_invariant_for_the_unrelated_queried_name() {
    for (base, prefixed, label) in [
        (
            F1.relations[0],
            F1Q.relations[0],
            "same-block-var-contributor",
        ),
        (
            F5.relations[0],
            F5Q.relations[0],
            "no-selected-same-source-contributor",
        ),
        (
            F7.relations[0],
            F7Q.relations[0],
            "block-origin-top-level-lexical-fallback",
        ),
    ] {
        assert_eq!(base.name, prefixed.name, "{label} semantic name");
        assert_eq!(base.region, prefixed.region, "{label} current region");
        match (base.relation, prefixed.relation) {
            (Relation::Vars(base_vars), Relation::Vars(prefixed_vars)) => {
                assert_eq!(
                    base_vars.len(),
                    prefixed_vars.len(),
                    "{label} contributor multiplicity"
                );
                for (base_anchor, prefixed_anchor) in base_vars.iter().zip(prefixed_vars.iter()) {
                    assert_eq!(
                        base_anchor.2, prefixed_anchor.2,
                        "{label} contributor authored fragment"
                    );
                }
            }
            (Relation::Lexical(_, base_origin), Relation::Lexical(_, prefixed_origin)) => {
                assert_eq!(
                    base_origin, prefixed_origin,
                    "{label} lexical-target meaning"
                );
            }
            (Relation::None, Relation::None) => {}
            (base_relation, prefixed_relation) => panic!(
                "{label} correspondence meaning diverged: {base_relation:?} vs {prefixed_relation:?}"
            ),
        }
    }

    // The accepted-witness identity itself is explicitly NOT invariant
    // across the prefix -- this is the load-bearing gap #705 exists to
    // bridge, not a bug in this oracle.
    assert_eq!(
        F1.lifecycle,
        ExpectedLifecycle::Accepted(ExpectedAcceptedWitness::OneLevelBlockStaticSemanticsAccepted)
    );
    assert_eq!(
        F1Q.lifecycle,
        ExpectedLifecycle::Accepted(
            ExpectedAcceptedWitness::VariableStatementStaticSemanticsAccepted
        )
    );
    assert_ne!(F1.lifecycle, F1Q.lifecycle);
}

#[test]
fn completion_theorem_remains_frozen_at_193_183_10() {
    let active = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::NormativeRule)
        .count();
    let inactive = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::EnvelopeInactiveRule)
        .count();

    assert_eq!(CONTAINERS.len(), 37);
    assert_eq!(RULE_UNITS.len(), 193);
    assert_eq!(active, 183);
    assert_eq!(inactive, 10);
    assert_eq!(REQUIRED_RULE_IDS.len(), 10);

    let required: std::collections::BTreeSet<_> = REQUIRED_RULE_IDS.iter().copied().collect();
    assert_eq!(
        required.len(),
        REQUIRED_RULE_IDS.len(),
        "no duplicate required id"
    );
    assert_eq!(
        RULE_UNITS.len() - REQUIRED_RULE_IDS.len(),
        183,
        "frozen complement"
    );

    for &rule_id in REQUIRED_RULE_IDS {
        let rule = RULE_UNITS
            .iter()
            .find(|rule| rule.id == rule_id)
            .unwrap_or_else(|| panic!("{rule_id} must remain frozen inventory authority"));
        assert_eq!(
            rule.kind,
            RuleUnitKind::NormativeRule,
            "{rule_id} required identity must remain an active rule"
        );
    }

    // This lifecycle theorem creates no new Early Error identity:
    // NEWLY_REACHABLE remains empty.
    const NEWLY_REACHABLE: &[&str] = &[];
    assert!(NEWLY_REACHABLE.is_empty());
}

#[test]
fn oracle_source_stays_candidate_independent_of_production_correspondence() {
    // Each forbidden marker is split across two `concat!` pieces so this
    // test's own source (scanned via `include_str!` below) never contains
    // the contiguous marker itself; only the runtime-concatenated value
    // does, for comparison against genuine production references.
    let this_source =
        include_str!("selected_one_level_block_name_correspondence_lifecycle_frontier.rs");
    for forbidden in [
        concat!("selected_variable_statement_name_correspond", "ence::"),
        concat!("SelectedVariableStatementNameCorrespon", "dence"),
        concat!("selected_lexical_sl", "ice::"),
        concat!("selected_static_seman", "tics::"),
        concat!("selected_qualification_integra", "tion::"),
        concat!("selected_binding_sc", "ope::"),
        concat!("selected_one_level_block_binding_sc", "ope::"),
        concat!("evaluate_selected_one_level_block_static_seman", "tics"),
        concat!("recognize_selected_lexical_sl", "ice"),
        concat!("crate::ecmascr", "ipt::"),
    ] {
        assert!(
            !this_source.contains(forbidden),
            "candidate-independent oracle must not reference production correspondence \
             authority: found {forbidden}"
        );
    }
}
