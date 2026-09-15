//! Candidate-independent all-selected-var contributor correspondence oracle
//! for Issue #701.
//!
//! Hierarchy: parent program #108; source-name correspondence foundation
//! #314/#315 -> #316/#317; top-level var `1..N` widening #322/#323 ->
//! #324/#325; direct top-level var `IdentifierReference` validation #330/#331;
//! multi-reference composition #332/#333; direct top-level var
//! `IdentifierReference` production #334/#335; one-level Block var research
//! #688; Block var `1..N` #693/#694 -> #695/#696; Block var decimal-integer
//! initializer #697/#698 -> #699/#700.
//!
//! Predecessor research verdict: **PREDECESSOR FRONTIER REQUIRED**. Before a
//! future Block-var direct-`IdentifierReference` initializer frontier can
//! exist, `SameSourceSelectedVarNameContributors` must widen its eligible
//! authored contributor domain uniformly from
//!
//! ```text
//! selected top-level VariableStatement declarators only
//! ```
//!
//! to
//!
//! ```text
//! every selected authored var declarator in the Script
//! = selected top-level VariableStatement declarators
//! + selected one-level Block-contained var declarators
//! ```
//!
//! This oracle freezes exactly that widened domain for the three existing
//! correspondence meanings:
//!
//! ```text
//! VisibleSelectedLexicalBinding
//! SameSourceSelectedVarNameContributors
//! NoSelectedSameSourceContributor
//! ```
//!
//! No fourth correspondence meaning is introduced. No Block-var
//! `IdentifierReference` initializer grammar is introduced or exercised;
//! every Block `var` declarator below is either bare or decimal-initialized,
//! matching the accepted #688-#700 grammar. Every reference exercised below
//! is an *existing* correspondence query input: a lexical-declaration or
//! top-level-var `IdentifierReference` initializer, including lexical
//! references authored inside the existing one-level Block profile.
//!
//! This module deliberately does not import, call, or otherwise derive
//! expected results from
//! `super::super::selected_variable_statement_name_correspondence`,
//! `selected_lexical_slice`, `selected_static_semantics`,
//! `selected_qualification_integration`, `selected_binding_scope`, or
//! `selected_one_level_block_binding_scope`. Every expected anchor, semantic
//! name, and ordered contributor list below is independently authored
//! fixture fact, verified only through the already-accepted Core
//! source-anchor contract (`SourceText::anchor`), never through source
//! search, rescanning, retokenization, reparsing, or a second tokenizer.
//!
//! Wrong-model discrimination (see the Issue for the full rationale):
//!
//! ```text
//! W1  top-level-only contributor domain          -> F1, F2, F3
//! W2  Block contributors visible only to Block   -> F3, F4
//!     references
//! W3  Block contributors visible only to         -> F1
//!     top-level references
//! W4  Block contributors visible only in the     -> F2
//!     same Block
//! W5  sibling Blocks excluded                    -> F2
//! W6  later contributors excluded                -> F5
//! W7  contributors grouped top-level-first        -> F6, F6I
//! W8  contributors grouped Block-first            -> F6, F6I
//! W9  contributors deduplicated by semantic name  -> F7, F8
//! W10 only first Block-var declarator contributes -> G
//! W11 only final Block-var declarator contributes -> G
//! W12 decimal-initialized Block vars fail to      -> K1, K2
//!     contribute
//! W13 decimal RHS becomes contributor/reference   -> K1, K2
//!     evidence
//! W14 escaped LHS compared by authored spelling   -> F9
//!     rather than semantic name
//! W15 Unicode normalization makes canonically     -> F10
//!     equivalent strings equal
//! W16 widened var contributors outrank lexical    -> F11A, F11B
//!     bindings
//! W17 statically rejected sources still produce   -> SR1
//!     relations
//! W18 Block contributor collection modeled as     -> (this whole file;
//!     runtime binding identity                       no Environment
//!                                                     Record/ResolveBinding
//!                                                     vocabulary appears)
//! ```

use super::inventory::{CONTAINERS, RULE_UNITS, RuleUnitKind};
use crate::{SourceId, SourceText};

const ISSUE_ID: u64 = 701;
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
/// fallback (the two branches of the existing region-resolution rule). The
/// widened var-contributor domain must not change either branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexicalOrigin {
    SameRegion,
    TopLevelFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Relation {
    Lexical(A, LexicalOrigin),
    Vars(&'static [A]),
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

#[derive(Debug, Clone, Copy)]
struct F {
    id: &'static str,
    source: &'static str,
    relations: &'static [R],
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

// F1 — same-Block Block-var contributor observed by an existing Block
// lexical reference. The central predecessor-removing fixture (kills W1,
// W3).
const F1_A: &[A] = &[A(6, 7, "a")];
const F1: F = F {
    id: "same-block-var-seen-by-block-lexical-reference",
    source: "{ var a; let x=a; }",
    relations: &[R {
        binding: A(13, 14, "x"),
        reference: A(15, 16, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Vars(F1_A),
    }],
};

// F2 — sibling-Block Block-var contributor: Script-wide provenance, not
// Block-local lookup (kills W4, W5).
const F2_A: &[A] = &[A(6, 7, "a")];
const F2: F = F {
    id: "sibling-block-var-seen-by-block-lexical-reference",
    source: "{ var a; } { let x=a; }",
    relations: &[R {
        binding: A(17, 18, "x"),
        reference: A(19, 20, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Vars(F2_A),
    }],
};

// F3 — Block-var contributor observed by an existing top-level lexical
// reference (kills W2).
const F3_A: &[A] = &[A(6, 7, "a")];
const F3: F = F {
    id: "block-var-seen-by-top-level-lexical-reference",
    source: "{ var a; } let x=a;",
    relations: &[R {
        binding: A(15, 16, "x"),
        reference: A(17, 18, "a"),
        name: "a",
        region: Region::TopLevel,
        relation: Relation::Vars(F3_A),
    }],
};

// F4 — Block-var contributor observed by an existing top-level var
// reference: no new query-input type, uniformity for the other existing
// owner (kills W2).
const F4_A: &[A] = &[A(6, 7, "a")];
const F4: F = F {
    id: "block-var-seen-by-top-level-var-reference",
    source: "{ var a; } var x=a;",
    relations: &[R {
        binding: A(15, 16, "x"),
        reference: A(17, 18, "a"),
        name: "a",
        region: Region::TopLevel,
        relation: Relation::Vars(F4_A),
    }],
};

// F5 — reference authored before its later Block-var contributor: whole-
// source provenance, not textual before/after visibility (kills W6).
const F5_A: &[A] = &[A(15, 16, "a")];
const F5: F = F {
    id: "reference-before-later-block-var-contributor",
    source: "var x=a; { var a; }",
    relations: &[R {
        binding: A(4, 5, "x"),
        reference: A(6, 7, "a"),
        name: "a",
        region: Region::TopLevel,
        relation: Relation::Vars(F5_A),
    }],
};

// F6 / F6I — mixed top-level/Block contributor order in both authored
// arrangements: order follows exact authored occurrence, never a
// region-grouped collection strategy (kills W7, W8).
const F6_A: &[A] = &[A(4, 5, "a"), A(13, 14, "a")];
const F6: F = F {
    id: "mixed-top-level-then-block-contributor-order",
    source: "var a; { var a; } var x=a;",
    relations: &[R {
        binding: A(22, 23, "x"),
        reference: A(24, 25, "a"),
        name: "a",
        region: Region::TopLevel,
        relation: Relation::Vars(F6_A),
    }],
};

const F6I_A: &[A] = &[A(6, 7, "a"), A(15, 16, "a")];
const F6I: F = F {
    id: "mixed-block-then-top-level-contributor-order",
    source: "{ var a; } var a; var x=a;",
    relations: &[R {
        binding: A(22, 23, "x"),
        reference: A(24, 25, "a"),
        name: "a",
        region: Region::TopLevel,
        relation: Relation::Vars(F6I_A),
    }],
};

// F7 — repeated Block contributors of the same authored `VariableStatement`
// remain two distinct authored occurrences (kills W9).
const F7_A: &[A] = &[A(6, 7, "a"), A(8, 9, "a")];
const F7: F = F {
    id: "repeated-block-contributors-same-statement",
    source: "{ var a,a; } var x=a;",
    relations: &[R {
        binding: A(17, 18, "x"),
        reference: A(19, 20, "a"),
        name: "a",
        region: Region::TopLevel,
        relation: Relation::Vars(F7_A),
    }],
};

// F8 — repeated contributors across top-level and Block placements remain
// four distinct authored occurrences in exact authored order (kills W9).
const F8_A: &[A] = &[A(4, 5, "a"), A(13, 14, "a"), A(15, 16, "a"), A(24, 25, "a")];
const F8: F = F {
    id: "repeated-contributors-across-placements",
    source: "var a; { var a,a; } var a; var x=a;",
    relations: &[R {
        binding: A(31, 32, "x"),
        reference: A(33, 34, "a"),
        name: "a",
        region: Region::TopLevel,
        relation: Relation::Vars(F8_A),
    }],
};

// F9 — escaped Block-var LHS semantic equality: authored escaped spelling
// and decoded semantic name remain distinct facts (kills W14). The braced
// CodePointEscape form is used for the authored spelling because it is an
// already-accepted alternate UnicodeEscapeSequence spelling for the same
// selected BindingIdentifier grammar (already used for accepted Block
// lexical escaped-identifier fixtures).
const F9_A: &[A] = &[A(6, 12, r"\u{61}")];
const F9: F = F {
    id: "escaped-block-var-lhs-semantic-equality",
    source: r"{ var \u{61}; } var x=a;",
    relations: &[R {
        binding: A(20, 21, "x"),
        reference: A(22, 23, "a"),
        name: "a",
        region: Region::TopLevel,
        relation: Relation::Vars(F9_A),
    }],
};

// F10 — no Unicode normalization: a composed contributor matches only a
// composed reference; a code-point-distinct decomposed reference of the
// same visual name matches nothing (kills W15). The decomposed reference is
// authored as an unescaped "e" followed by a braced combining-acute
// CodePointEscape, which decodes to the two-code-point sequence e + U+0301
// rather than the single-code-point precomposed U+00E9.
const F10_COMPOSED: &[A] = &[A(6, 8, "é")];
const F10: F = F {
    id: "unicode-composed-matches-decomposed-does-not",
    source: r"{ var é; } var x=é; var y=e\u{301};",
    relations: &[
        R {
            binding: A(16, 17, "x"),
            reference: A(18, 20, "é"),
            name: "é",
            region: Region::TopLevel,
            relation: Relation::Vars(F10_COMPOSED),
        },
        R {
            binding: A(26, 27, "y"),
            reference: A(28, 36, r"e\u{301}"),
            name: "e\u{0301}",
            region: Region::TopLevel,
            relation: Relation::None,
        },
    ],
};

// F11A — current-region (Block) lexical binding outranks a widened var
// contributor of the same name authored elsewhere (kills W16).
const F11A: F = F {
    id: "current-block-lexical-outranks-var-contributor",
    source: "var a; { let a; let y=a; }",
    relations: &[R {
        binding: A(20, 21, "y"),
        reference: A(22, 23, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Lexical(A(13, 14, "a"), LexicalOrigin::SameRegion),
    }],
};

// F11B — the existing Block-origin top-level lexical fallback is unchanged
// by the widened var domain (kills W16). Per the Issue, a var contributor of
// the same name cannot be added here without a distinct static rejection
// (Script `VarDeclaredNames`/`LexicallyDeclaredNames` collision), so this
// fixture proves the unchanged fallback alone, matching the Issue's own
// "smallest static-valid control" guidance.
const F11B: F = F {
    id: "top-level-lexical-fallback-unchanged-for-block-origin-reference",
    source: "let a; { let x=a; }",
    relations: &[R {
        binding: A(13, 14, "x"),
        reference: A(15, 16, "a"),
        name: "a",
        region: Region::Block,
        relation: Relation::Lexical(A(4, 5, "a"), LexicalOrigin::TopLevelFallback),
    }],
};

// F12 — `NoSelectedSameSourceContributor` remains representable: the
// widened domain must not fabricate a false-positive match for an unrelated
// name.
const F12: F = F {
    id: "no-selected-same-source-contributor-control",
    source: "{ var a; } let y=q;",
    relations: &[R {
        binding: A(15, 16, "y"),
        reference: A(17, 18, "q"),
        name: "q",
        region: Region::TopLevel,
        relation: Relation::None,
    }],
};

// G — Block `var` `1..N` cardinality: first, interior, and final declarators
// are independently addressable contributors (kills W10, W11).
const G_A: &[A] = &[A(6, 7, "a")];
const G_B: &[A] = &[A(8, 9, "b")];
const G_C: &[A] = &[A(10, 11, "c")];
const G: F = F {
    id: "block-var-cardinality-first-interior-final-addressable",
    source: "{ var a,b,c; } var p=a; var q=b; var r=c;",
    relations: &[
        R {
            binding: A(19, 20, "p"),
            reference: A(21, 22, "a"),
            name: "a",
            region: Region::TopLevel,
            relation: Relation::Vars(G_A),
        },
        R {
            binding: A(28, 29, "q"),
            reference: A(30, 31, "b"),
            name: "b",
            region: Region::TopLevel,
            relation: Relation::Vars(G_B),
        },
        R {
            binding: A(37, 38, "r"),
            reference: A(39, 40, "c"),
            name: "c",
            region: Region::TopLevel,
            relation: Relation::Vars(G_C),
        },
    ],
};

// K1 — a decimal-initialized Block var contributes through its LHS only;
// the decimal RHS is never contributor evidence (kills W12, W13).
const K1_A: &[A] = &[A(6, 7, "a")];
const K1: F = F {
    id: "decimal-initialized-block-var-contributes-via-lhs-only",
    source: "{ var a=1; } var x=a;",
    relations: &[R {
        binding: A(17, 18, "x"),
        reference: A(19, 20, "a"),
        name: "a",
        region: Region::TopLevel,
        relation: Relation::Vars(K1_A),
    }],
};

// K2 — a bare declarator sharing a Block `var` statement with a
// decimal-initialized sibling contributes identically (kills W12, W13).
const K2_A: &[A] = &[A(10, 11, "b")];
const K2: F = F {
    id: "mixed-decimal-and-bare-block-var-declarators-contribute-uniformly",
    source: "{ var a=1,b; } var x=b;",
    relations: &[R {
        binding: A(19, 20, "x"),
        reference: A(21, 22, "b"),
        name: "b",
        region: Region::TopLevel,
        relation: Relation::Vars(K2_A),
    }],
};

const FIXTURES: &[F] = &[
    F1, F2, F3, F4, F5, F6, F6I, F7, F8, F9, F10, F11A, F11B, F12, G, K1, K2,
];

/// A statically rejected source is never expected to reach the fixture
/// table above: the widened contributor domain must not run ahead of static
/// acceptance (kills W17). `EE-36-R01` (Script duplicate lexical) is
/// existing frozen authority, not a new identity introduced by this Issue.
const SR1_SOURCE: &str = "let a=1; let a=2; var x=a;";
const SR1_FIRST_A: A = A(4, 5, "a");
const SR1_SECOND_A: A = A(13, 14, "a");
const SR1_REJECTING_RULE: &str = "EE-36-R01";

#[test]
fn fixture_anchor_and_semantic_name_audit() {
    let mut audited_fixtures = 0usize;
    let mut audited_anchors = 0usize;

    for fixture in FIXTURES {
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
    }

    // Unescaped, non-normalized references carry a raw fragment identical to
    // their semantic name; F9/F10 intentionally break that identity and are
    // audited separately below.
    for fixture in FIXTURES {
        if fixture.id == F9.id || fixture.id == F10.id {
            continue;
        }
        for relation in fixture.relations {
            assert_eq!(relation.reference.2, relation.name);
        }
    }

    // SR1 is audited for anchor validity even though its whole source is
    // statically rejected: byte-range validity is a Core source-anchor
    // property, not a static-acceptance claim.
    let sr1 = source(SR1_SOURCE);
    assert_anchor(&sr1, SR1_FIRST_A);
    assert_anchor(&sr1, SR1_SECOND_A);
    audited_fixtures += 1;
    audited_anchors += 2;

    assert_eq!(audited_fixtures, FIXTURES.len() + 1);
    assert!(audited_anchors >= 40, "{audited_anchors} anchors audited");
}

#[test]
fn all_var_contributor_domain_includes_top_level_and_block_declarators() {
    // F1: a Block-only contributor is seen by a Block lexical reference.
    // A top-level-only domain (W1) would wrongly predict `None`; a
    // Block-visible-only-to-top-level domain (W3) would also wrongly
    // predict `None` here since the reference itself is Block-origin.
    assert_eq!(F1.relations.len(), 1);
    assert_eq!(F1.relations[0].relation, Relation::Vars(F1_A));

    // F3/F4: the same Block-only contributor is seen by the *other* two
    // existing query-input owners (top-level lexical, top-level var). A
    // Block-visible-only-to-Block domain (W2) would wrongly predict `None`
    // for both.
    assert_eq!(F3.relations[0].relation, Relation::Vars(F3_A));
    assert_eq!(F4.relations[0].relation, Relation::Vars(F4_A));
}

#[test]
fn contributor_membership_is_script_wide_not_block_local() {
    // F2: the contributor and the reference live in different, sibling
    // Blocks. A same-Block-only collector (W4) or a sibling-Block exclusion
    // (W5) would wrongly predict `None`.
    assert_eq!(F2.relations[0].relation, Relation::Vars(F2_A));
    assert_eq!(F2_A, F1_A, "same anchor shape, different Block placement");
}

#[test]
fn contributor_visibility_is_whole_source_not_temporal() {
    // F5: the reference is authored strictly before its only contributor.
    // A before-only visibility filter (W6) would wrongly predict `None`.
    assert_eq!(F5.relations[0].relation, Relation::Vars(F5_A));
    assert!(F5.relations[0].reference.0 < F5_A[0].0);
}

#[test]
fn global_authored_order_is_preserved_across_placement() {
    // F6 / F6I: swapping which placement (top-level vs Block) is authored
    // first swaps which anchor leads the expected contributor list. A
    // region-grouped strategy (W7 top-level-first, W8 Block-first) would
    // produce the same order in both fixtures; the independent oracle does
    // not.
    assert_eq!(F6_A, &[A(4, 5, "a"), A(13, 14, "a")]);
    assert_eq!(F6I_A, &[A(6, 7, "a"), A(15, 16, "a")]);
    assert_ne!(F6_A, F6I_A);
    assert!(F6_A[0].0 < F6_A[1].0);
    assert!(F6I_A[0].0 < F6I_A[1].0);
}

#[test]
fn repeated_contributors_are_never_deduplicated() {
    // F7: two declarators of one Block `var` statement remain two distinct
    // authored occurrences of the same semantic name (kills W9).
    assert_eq!(F7_A.len(), 2);
    assert_ne!(F7_A[0], F7_A[1]);
    assert_eq!(F7_A[0].2, F7_A[1].2);

    // F8: four occurrences across top-level and Block placements remain
    // four occurrences, in exact authored order, with no set semantics.
    assert_eq!(F8_A.len(), 4);
    let starts: Vec<_> = F8_A.iter().map(|a| a.0).collect();
    assert!(starts.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        F8_A,
        &[A(4, 5, "a"), A(13, 14, "a"), A(15, 16, "a"), A(24, 25, "a")]
    );
}

#[test]
fn block_var_cardinality_first_interior_final_are_independently_addressable() {
    // G: three declarators of one Block `var` statement each remain
    // independently addressable as a sole contributor for their own
    // reference (kills W10 first-only, W11 final-only).
    assert_eq!(G.relations.len(), 3);
    assert_eq!(G.relations[0].relation, Relation::Vars(G_A));
    assert_eq!(G.relations[1].relation, Relation::Vars(G_B));
    assert_eq!(G.relations[2].relation, Relation::Vars(G_C));
    assert_ne!(G_A[0], G_B[0]);
    assert_ne!(G_B[0], G_C[0]);
}

#[test]
fn escaped_lhs_authored_spelling_and_semantic_name_remain_distinct() {
    // F9: the Block-var contributor anchor keeps its exact authored escaped
    // spelling; the reference's semantic name is the decoded "a" (kills
    // W14).
    assert_eq!(F9_A[0].2, r"\u{61}");
    assert_ne!(F9_A[0].2, "a");
    assert_eq!(F9.relations[0].name, "a");
    assert_eq!(F9.relations[0].relation, Relation::Vars(F9_A));
}

#[test]
fn unicode_composed_and_decomposed_never_normalize_equal() {
    // F10: a composed contributor matches only a composed reference of the
    // identical code point sequence.
    assert_eq!(F10.relations[0].relation, Relation::Vars(F10_COMPOSED));

    // The decomposed reference (`e` + U+0301) never matches, even though it
    // is visually identical to the composed contributor (kills W15).
    let decomposed_name = F10.relations[1].name;
    let composed_name = F10_COMPOSED[0].2;
    assert_ne!(decomposed_name, composed_name);
    assert_eq!(decomposed_name.chars().count(), 2);
    assert_eq!(composed_name.chars().count(), 1);
    assert_eq!(F10.relations[1].relation, Relation::None);
}

#[test]
fn lexical_precedence_outranks_widened_var_contributors() {
    // F11A: a current-region Block lexical binding wins even though a
    // same-named var contributor exists elsewhere in the Script (kills
    // W16).
    match F11A.relations[0].relation {
        Relation::Lexical(anchor, LexicalOrigin::SameRegion) => {
            assert_eq!(anchor, A(13, 14, "a"));
        }
        other => panic!("expected same-region lexical precedence, got {other:?}"),
    }

    // F11B: the existing Block-origin top-level lexical fallback is
    // unchanged by the widened var domain.
    match F11B.relations[0].relation {
        Relation::Lexical(anchor, LexicalOrigin::TopLevelFallback) => {
            assert_eq!(anchor, A(4, 5, "a"));
        }
        other => panic!("expected top-level lexical fallback, got {other:?}"),
    }
}

#[test]
fn no_selected_same_source_contributor_remains_representable() {
    // F12: the widened domain must not fabricate a match for an unrelated
    // semantic name.
    assert_eq!(F12.relations[0].relation, Relation::None);
}

#[test]
fn decimal_initializer_never_becomes_contributor_evidence() {
    // K1: the decimal RHS anchor (8, 9, "1") never appears as a contributor
    // or reference anchor anywhere in the expected relation (kills W12,
    // W13).
    assert_eq!(K1.relations[0].relation, Relation::Vars(K1_A));
    assert_eq!(K1_A, &[A(6, 7, "a")]);
    assert_ne!(K1_A[0], A(8, 9, "1"));

    // K2: a bare declarator sharing a statement with a decimal-initialized
    // sibling contributes identically through its own LHS.
    assert_eq!(K2.relations[0].relation, Relation::Vars(K2_A));
    assert_eq!(K2_A, &[A(10, 11, "b")]);
}

#[test]
fn static_rejection_gates_correspondence_commitment() {
    // SR1: the whole source is statically rejected (Script duplicate
    // lexical `a`) before any correspondence relation for `var x=a;` could
    // be committed (kills W17). This cites existing frozen `EE-36-R01`
    // authority; it introduces no new Early Error identity.
    assert_eq!(SR1_REJECTING_RULE, "EE-36-R01");
    assert_eq!(SR1_FIRST_A.2, "a");
    assert_eq!(SR1_SECOND_A.2, "a");
    assert_ne!(SR1_FIRST_A, SR1_SECOND_A);

    let rejecting_rule = RULE_UNITS
        .iter()
        .find(|rule| rule.id == SR1_REJECTING_RULE)
        .expect("EE-36-R01 must remain frozen inventory authority");
    assert_eq!(rejecting_rule.kind, RuleUnitKind::NormativeRule);
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

    // `required` (10 exact identities) and `complement` (183, the remaining
    // rule units) are this validation lineage's own frozen bookkeeping
    // constants, unchanged from prior accepted authority: every required
    // identity remains active (`NormativeRule`) inventory, and
    // `RULE_UNITS.len() - REQUIRED_RULE_IDS.len()` is the frozen complement.
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

    // Correspondence contributor widening creates no new Early Error
    // identity: NEWLY_REACHABLE remains empty.
    const NEWLY_REACHABLE: &[&str] = &[];
    assert!(NEWLY_REACHABLE.is_empty());
}

#[test]
fn oracle_source_stays_candidate_independent_of_production_correspondence() {
    // Each forbidden marker is split across two `concat!` pieces so this
    // test's own source (scanned via `include_str!` below) never contains
    // the contiguous marker itself; only the runtime-concatenated value
    // does, for comparison against genuine production references.
    let this_source = include_str!("selected_all_var_contributor_correspondence_frontier.rs");
    for forbidden in [
        concat!("selected_variable_statement_name_correspond", "ence::"),
        concat!("SelectedVariableStatementNameCorrespon", "dence"),
        concat!(
            "analyze_selected_variable_statement_name_correspon",
            "dence"
        ),
        concat!("selected_lexical_sl", "ice::"),
        concat!("selected_static_seman", "tics::"),
        concat!("selected_qualification_integra", "tion::"),
        concat!("selected_binding_sc", "ope::"),
        concat!("selected_one_level_block_binding_sc", "ope::"),
        concat!("crate::ecmascr", "ipt::"),
    ] {
        assert!(
            !this_source.contains(forbidden),
            "candidate-independent oracle must not reference production correspondence \
             authority: found {forbidden}"
        );
    }
}
