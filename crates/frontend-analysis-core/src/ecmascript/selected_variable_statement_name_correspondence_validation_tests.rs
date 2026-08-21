//! Candidate-independent selected `var` name-correspondence / lexical-shadowing validation
//! for Issue #314.
//!
//! Expected relations are fixture-owned. This oracle composes already-accepted source,
//! static, lexical-region, and `var` provenance authority without calling production
//! recognition, static semantics, qualification, Binding / Scope, or runtime behavior
//! to derive expected meaning.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

const ISSUE_ID: u64 = 314;
const SELECTED_ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const SELECTED_ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const SELECTED_UNICODE_VERSION: &str = "17.0.0";
const SELECTED_SOURCE_AUTHORITY: &str = "Core UTF-8 SourceText / SourceAnchor";
const SELECTED_SOURCE_CONTEXT: &str = "Independent Source Unit";
const SELECTED_PARSE_GOAL: &str = "Script";
const SELECTED_IS_STRICT: bool = false;
const SELECTED_YIELD: bool = false;
const SELECTED_AWAIT: bool = false;
const SELECTED_VARIABLE_STATEMENT_GRAMMAR: &str =
    "VariableStatement ::= var SelectedBindingIdentifier ;";
const SELECTED_VARIABLE_TOP_LEVEL_ONLY: bool = true;
const SELECTED_VARIABLE_SINGLE_BINDING_ONLY: bool = true;
const SELECTED_VARIABLE_INITIALIZER: bool = false;
const SELECTED_VARIABLE_DESTRUCTURING: bool = false;
const SELECTED_VARIABLE_EOF_ASI: bool = false;
const SELECTED_BLOCK_VAR: bool = false;
const POSITIVE_AGGREGATE_LIFECYCLE: &str = "SelectedAcceptedIncomplete";
const AGGREGATE_QUALIFIED_AVAILABLE: bool = false;

const _: () = {
    assert!(!SELECTED_IS_STRICT);
    assert!(!SELECTED_YIELD);
    assert!(!SELECTED_AWAIT);
    assert!(SELECTED_VARIABLE_TOP_LEVEL_ONLY);
    assert!(SELECTED_VARIABLE_SINGLE_BINDING_ONLY);
    assert!(!SELECTED_VARIABLE_INITIALIZER);
    assert!(!SELECTED_VARIABLE_DESTRUCTURING);
    assert!(!SELECTED_VARIABLE_EOF_ASI);
    assert!(!SELECTED_BLOCK_VAR);
    assert!(!AGGREGATE_QUALIFIED_AVAILABLE);
};

const AUTHORED_VAR_BOUNDARY: &str =
    "authored VariableDeclaration provenance != unique runtime binding identity";
const VAR_CORRESPONDENCE_BOUNDARY: &str =
    "same-source selected var contributor != runtime ResolveBinding target";
const LEXICAL_CORRESPONDENCE_BOUNDARY: &str = "visible selected lexical binding != proof of runtime binding instantiation != proof of ResolveBinding success != proof that execution reached the reference";
const NO_CONTRIBUTOR_BOUNDARY: &str =
    "no selected same-source contributor != runtime unresolvable != ReferenceError";
const VAR_ORDER_BOUNDARY: &str = "selected var contributor authored order is declaration provenance only, not Before/Same/After, TDZ, initialization, hoisting, runtime creation order, or runtime lookup order";

const BLOCK_RELATION_ORACLE: &str =
    include_str!("selected_one_level_block_binding_scope_validation_tests.rs");
const VAR_ORACLE: &str =
    include_str!("qualification_selected_top_level_variable_statement_validation_tests.rs");
const VAR_COMPOSITION_ORACLE: &str = include_str!(
    "qualification_selected_top_level_variable_statement_composition_validation_tests.rs"
);
const CURRENT_COMPLETION_ORACLE: &str =
    include_str!("qualification_validation_tests/selected_variable_statement_slice_completion.rs");
const THIS_SOURCE: &str =
    include_str!("selected_variable_statement_name_correspondence_validation_tests.rs");

const A: &[u32] = &[0x61];
const Y: &[u32] = &[0x79];
const E_COMBINING_ACUTE: &[u32] = &[0x65, 0x0301];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedAnchor {
    start: usize,
    end: usize,
    fragment: &'static str,
}

impl ExpectedAnchor {
    const fn new(start: usize, end: usize, fragment: &'static str) -> Self {
        Self {
            start,
            end,
            fragment,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedRegion {
    TopLevel,
    Block(ExpectedAnchor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedCorrespondence {
    VisibleSelectedLexicalBinding {
        binding: ExpectedAnchor,
        region: ExpectedRegion,
    },
    SameSourceSelectedVarNameContributors {
        contributors: &'static [ExpectedAnchor],
    },
    NoSelectedSameSourceContributor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedRelation {
    containing_binding: ExpectedAnchor,
    current_region: ExpectedRegion,
    reference: ExpectedAnchor,
    semantic_name: &'static str,
    semantic_code_points: &'static [u32],
    correspondence: ExpectedCorrespondence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpstreamPrerequisite {
    StaticSemanticsRejected {
        rule_id: &'static str,
        subject: ExpectedAnchor,
    },
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedProcessing {
    Complete(&'static [ExpectedRelation]),
    UpstreamPrerequisiteUnavailable(UpstreamPrerequisite),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RelationFixture {
    id: &'static str,
    source: &'static str,
    blocks: &'static [ExpectedAnchor],
    controls: &'static [ExpectedAnchor],
    processing: ExpectedProcessing,
}

const F1_VARS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 5, "a")];
const F1_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(11, 12, "x"),
    current_region: ExpectedRegion::TopLevel,
    reference: ExpectedAnchor::new(13, 14, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
        contributors: F1_VARS,
    },
}];

const F2_VARS: &[ExpectedAnchor] = &[ExpectedAnchor::new(13, 14, "a")];
const F2_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(4, 5, "x"),
    current_region: ExpectedRegion::TopLevel,
    reference: ExpectedAnchor::new(6, 7, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
        contributors: F2_VARS,
    },
}];

const F3_VARS: &[ExpectedAnchor] = &[
    ExpectedAnchor::new(4, 5, "a"),
    ExpectedAnchor::new(11, 12, "a"),
];
const F3_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(18, 19, "x"),
    current_region: ExpectedRegion::TopLevel,
    reference: ExpectedAnchor::new(20, 21, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
        contributors: F3_VARS,
    },
}];

const F4_VARS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 10, r"\u0061")];
const F4_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(16, 17, "x"),
    current_region: ExpectedRegion::TopLevel,
    reference: ExpectedAnchor::new(18, 19, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
        contributors: F4_VARS,
    },
}];

const F5_VARS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 5, "a")];
const F5_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(11, 12, "x"),
    current_region: ExpectedRegion::TopLevel,
    reference: ExpectedAnchor::new(13, 19, r"\u0061"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
        contributors: F5_VARS,
    },
}];

const F6_VARS: &[ExpectedAnchor] = &[
    ExpectedAnchor::new(4, 5, "a"),
    ExpectedAnchor::new(11, 17, r"\u0061"),
];
const F6_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(23, 24, "x"),
    current_region: ExpectedRegion::TopLevel,
    reference: ExpectedAnchor::new(25, 26, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
        contributors: F6_VARS,
    },
}];

const F7_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(12, 13, "x"),
    current_region: ExpectedRegion::TopLevel,
    reference: ExpectedAnchor::new(14, 21, r"e\u0301"),
    semantic_name: "e\u{301}",
    semantic_code_points: E_COMBINING_ACUTE,
    correspondence: ExpectedCorrespondence::NoSelectedSameSourceContributor,
}];

const F8_BLOCK: ExpectedAnchor = ExpectedAnchor::new(7, 19, "{ let x=a; }");
const F8_VARS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 5, "a")];
const F8_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(13, 14, "x"),
    current_region: ExpectedRegion::Block(F8_BLOCK),
    reference: ExpectedAnchor::new(15, 16, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
        contributors: F8_VARS,
    },
}];

const F9_BLOCK: ExpectedAnchor = ExpectedAnchor::new(7, 28, "{ let a=1; let x=a; }");
const F9_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(22, 23, "x"),
    current_region: ExpectedRegion::Block(F9_BLOCK),
    reference: ExpectedAnchor::new(24, 25, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::VisibleSelectedLexicalBinding {
        binding: ExpectedAnchor::new(13, 14, "a"),
        region: ExpectedRegion::Block(F9_BLOCK),
    },
}];

const F10_BLOCK: ExpectedAnchor = ExpectedAnchor::new(7, 28, "{ let x=a; let a=1; }");
const F10_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(13, 14, "x"),
    current_region: ExpectedRegion::Block(F10_BLOCK),
    reference: ExpectedAnchor::new(15, 16, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::VisibleSelectedLexicalBinding {
        binding: ExpectedAnchor::new(22, 23, "a"),
        region: ExpectedRegion::Block(F10_BLOCK),
    },
}];

const F11_BLOCK: ExpectedAnchor = ExpectedAnchor::new(7, 19, "{ let a=a; }");
const F11_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(13, 14, "a"),
    current_region: ExpectedRegion::Block(F11_BLOCK),
    reference: ExpectedAnchor::new(15, 16, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::VisibleSelectedLexicalBinding {
        binding: ExpectedAnchor::new(13, 14, "a"),
        region: ExpectedRegion::Block(F11_BLOCK),
    },
}];

const F12_BLOCK_1: ExpectedAnchor = ExpectedAnchor::new(0, 12, "{ let a=1; }");
const F12_BLOCK_2: ExpectedAnchor = ExpectedAnchor::new(20, 32, "{ let x=a; }");
const F12_VARS: &[ExpectedAnchor] = &[ExpectedAnchor::new(17, 18, "a")];
const F12_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(26, 27, "x"),
    current_region: ExpectedRegion::Block(F12_BLOCK_2),
    reference: ExpectedAnchor::new(28, 29, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
        contributors: F12_VARS,
    },
}];

const F13_BLOCK: ExpectedAnchor = ExpectedAnchor::new(9, 21, "{ let x=a; }");
const F13_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(F13_BLOCK),
    reference: ExpectedAnchor::new(17, 18, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    correspondence: ExpectedCorrespondence::VisibleSelectedLexicalBinding {
        binding: ExpectedAnchor::new(4, 5, "a"),
        region: ExpectedRegion::TopLevel,
    },
}];

const F14_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(4, 5, "x"),
    current_region: ExpectedRegion::TopLevel,
    reference: ExpectedAnchor::new(6, 7, "y"),
    semantic_name: "y",
    semantic_code_points: Y,
    correspondence: ExpectedCorrespondence::NoSelectedSameSourceContributor,
}];

const FIXTURES: &[RelationFixture] = &[
    RelationFixture {
        id: "top-level-var-before-reference",
        source: "var a; let x=a;",
        blocks: &[],
        controls: F1_VARS,
        processing: ExpectedProcessing::Complete(F1_RELATIONS),
    },
    RelationFixture {
        id: "top-level-var-after-reference",
        source: "let x=a; var a;",
        blocks: &[],
        controls: F2_VARS,
        processing: ExpectedProcessing::Complete(F2_RELATIONS),
    },
    RelationFixture {
        id: "repeated-var-preserves-all-authored-contributors",
        source: "var a; var a; let x=a;",
        blocks: &[],
        controls: F3_VARS,
        processing: ExpectedProcessing::Complete(F3_RELATIONS),
    },
    RelationFixture {
        id: "escaped-var-direct-reference",
        source: r"var \u0061; let x=a;",
        blocks: &[],
        controls: F4_VARS,
        processing: ExpectedProcessing::Complete(F4_RELATIONS),
    },
    RelationFixture {
        id: "direct-var-escaped-reference",
        source: r"var a; let x=\u0061;",
        blocks: &[],
        controls: F5_VARS,
        processing: ExpectedProcessing::Complete(F5_RELATIONS),
    },
    RelationFixture {
        id: "direct-and-escaped-var-contributors",
        source: r"var a; var \u0061; let x=a;",
        blocks: &[],
        controls: F6_VARS,
        processing: ExpectedProcessing::Complete(F6_RELATIONS),
    },
    RelationFixture {
        id: "canonical-equivalence-does-not-normalize",
        source: "var é; let x=e\\u0301;",
        blocks: &[],
        controls: &[ExpectedAnchor::new(4, 6, "é")],
        processing: ExpectedProcessing::Complete(F7_RELATIONS),
    },
    RelationFixture {
        id: "block-falls-back-to-script-var-contributor",
        source: "var a; { let x=a; }",
        blocks: &[F8_BLOCK],
        controls: F8_VARS,
        processing: ExpectedProcessing::Complete(F8_RELATIONS),
    },
    RelationFixture {
        id: "inner-lexical-wins",
        source: "var a; { let a=1; let x=a; }",
        blocks: &[F9_BLOCK],
        controls: &[ExpectedAnchor::new(4, 5, "a")],
        processing: ExpectedProcessing::Complete(F9_RELATIONS),
    },
    RelationFixture {
        id: "later-authored-inner-lexical-still-wins",
        source: "var a; { let x=a; let a=1; }",
        blocks: &[F10_BLOCK],
        controls: &[ExpectedAnchor::new(4, 5, "a")],
        processing: ExpectedProcessing::Complete(F10_RELATIONS),
    },
    RelationFixture {
        id: "self-lexical-wins-over-outer-var",
        source: "var a; { let a=a; }",
        blocks: &[F11_BLOCK],
        controls: &[ExpectedAnchor::new(4, 5, "a")],
        processing: ExpectedProcessing::Complete(F11_RELATIONS),
    },
    RelationFixture {
        id: "sibling-lexical-excluded-script-var-fallback",
        source: "{ let a=1; } var a; { let x=a; }",
        blocks: &[F12_BLOCK_1, F12_BLOCK_2],
        controls: &[
            ExpectedAnchor::new(6, 7, "a"),
            ExpectedAnchor::new(17, 18, "a"),
        ],
        processing: ExpectedProcessing::Complete(F12_RELATIONS),
    },
    RelationFixture {
        id: "unrelated-var-does-not-perturb-lexical-fallback",
        source: "let a=1; { let x=a; } var b;",
        blocks: &[F13_BLOCK],
        controls: &[
            ExpectedAnchor::new(4, 5, "a"),
            ExpectedAnchor::new(26, 27, "b"),
        ],
        processing: ExpectedProcessing::Complete(F13_RELATIONS),
    },
    RelationFixture {
        id: "no-selected-same-source-contributor",
        source: "let x=y; var a;",
        blocks: &[],
        controls: &[ExpectedAnchor::new(13, 14, "a")],
        processing: ExpectedProcessing::Complete(F14_RELATIONS),
    },
    RelationFixture {
        id: "accepted-repeated-var-with-zero-reference-relations",
        source: "var a; var a;",
        blocks: &[],
        controls: &[
            ExpectedAnchor::new(4, 5, "a"),
            ExpectedAnchor::new(11, 12, "a"),
        ],
        processing: ExpectedProcessing::Complete(&[]),
    },
    RelationFixture {
        id: "script-lexical-var-collision-prerequisite-unavailable",
        source: "var a; let a;",
        blocks: &[],
        controls: &[ExpectedAnchor::new(4, 5, "a")],
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::StaticSemanticsRejected {
                rule_id: "EE-36-R02",
                subject: ExpectedAnchor::new(11, 12, "a"),
            },
        ),
    },
    RelationFixture {
        id: "block-duplicate-prerequisite-unavailable",
        source: "var a; { let a; let a; }",
        blocks: &[ExpectedAnchor::new(7, 24, "{ let a; let a; }")],
        controls: &[ExpectedAnchor::new(4, 5, "a")],
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::StaticSemanticsRejected {
                rule_id: "EE-14-R01",
                subject: ExpectedAnchor::new(20, 21, "a"),
            },
        ),
    },
    RelationFixture {
        id: "escaped-reserved-var-prerequisite-unavailable",
        source: r"var \u0069f; let x=a;",
        blocks: &[],
        controls: &[
            ExpectedAnchor::new(17, 18, "x"),
            ExpectedAnchor::new(19, 20, "a"),
        ],
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::StaticSemanticsRejected {
                rule_id: "EE-04-R08",
                subject: ExpectedAnchor::new(4, 11, r"\u0069f"),
            },
        ),
    },
    RelationFixture {
        id: "var-initializer-remains-unsupported",
        source: "var a=1; let x=a;",
        blocks: &[],
        controls: &[
            ExpectedAnchor::new(4, 5, "a"),
            ExpectedAnchor::new(5, 7, "=1"),
            ExpectedAnchor::new(13, 14, "x"),
            ExpectedAnchor::new(15, 16, "a"),
        ],
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::UnsupportedCoverage,
        ),
    },
    RelationFixture {
        id: "grouped-reference-remains-unsupported",
        source: "var a; let x=(a);",
        blocks: &[],
        controls: &[
            ExpectedAnchor::new(4, 5, "a"),
            ExpectedAnchor::new(11, 12, "x"),
            ExpectedAnchor::new(13, 16, "(a)"),
        ],
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::UnsupportedCoverage,
        ),
    },
    RelationFixture {
        id: "var-eof-asi-remains-unsupported",
        source: "var a",
        blocks: &[],
        controls: &[ExpectedAnchor::new(4, 5, "a")],
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::UnsupportedCoverage,
        ),
    },
    RelationFixture {
        id: "incomplete-block-prevents-tentative-relation",
        source: "var a; { let x=a;",
        blocks: &[],
        controls: &[
            ExpectedAnchor::new(4, 5, "a"),
            ExpectedAnchor::new(7, 8, "{"),
            ExpectedAnchor::new(13, 14, "x"),
            ExpectedAnchor::new(15, 16, "a"),
        ],
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::UnsupportedCoverage,
        ),
    },
];

fn fixture(id: &str) -> &'static RelationFixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing #314 fixture {id}"))
}

fn validate_anchor(source: &SourceText, expected: ExpectedAnchor) {
    let anchor = source
        .anchor(expected.start, expected.end)
        .unwrap_or_else(|error| panic!("invalid fixture anchor {expected:?}: {error}"));
    assert_eq!(anchor.fragment(), expected.fragment);
    assert_eq!(anchor.range().start(), expected.start);
    assert_eq!(anchor.range().end(), expected.end);
}

fn validate_region(source: &SourceText, region: ExpectedRegion, blocks: &[ExpectedAnchor]) {
    match region {
        ExpectedRegion::TopLevel => {}
        ExpectedRegion::Block(block) => {
            assert!(blocks.contains(&block));
            validate_anchor(source, block);
        }
    }
}

fn validate_relation(source: &SourceText, blocks: &[ExpectedAnchor], relation: ExpectedRelation) {
    validate_anchor(source, relation.containing_binding);
    validate_region(source, relation.current_region, blocks);
    validate_anchor(source, relation.reference);
    assert!(
        relation
            .semantic_name
            .chars()
            .map(u32::from)
            .eq(relation.semantic_code_points.iter().copied())
    );

    match relation.correspondence {
        ExpectedCorrespondence::VisibleSelectedLexicalBinding { binding, region } => {
            validate_anchor(source, binding);
            validate_region(source, region, blocks);
        }
        ExpectedCorrespondence::SameSourceSelectedVarNameContributors { contributors } => {
            assert!(!contributors.is_empty());
            for contributor in contributors {
                validate_anchor(source, *contributor);
            }
        }
        ExpectedCorrespondence::NoSelectedSameSourceContributor => {}
    }
}

#[test]
fn fixed_envelope_and_negative_runtime_boundaries_are_explicit() {
    assert_eq!(SELECTED_ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        SELECTED_ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(SELECTED_UNICODE_VERSION, "17.0.0");
    assert_eq!(
        SELECTED_SOURCE_AUTHORITY,
        "Core UTF-8 SourceText / SourceAnchor"
    );
    assert_eq!(SELECTED_SOURCE_CONTEXT, "Independent Source Unit");
    assert_eq!(SELECTED_PARSE_GOAL, "Script");
    assert_eq!(
        SELECTED_VARIABLE_STATEMENT_GRAMMAR,
        "VariableStatement ::= var SelectedBindingIdentifier ;"
    );
    assert_eq!(POSITIVE_AGGREGATE_LIFECYCLE, "SelectedAcceptedIncomplete");

    assert_eq!(
        AUTHORED_VAR_BOUNDARY,
        "authored VariableDeclaration provenance != unique runtime binding identity"
    );
    assert_eq!(
        VAR_CORRESPONDENCE_BOUNDARY,
        "same-source selected var contributor != runtime ResolveBinding target"
    );
    assert!(LEXICAL_CORRESPONDENCE_BOUNDARY.contains("!= proof of ResolveBinding success"));
    assert!(NO_CONTRIBUTOR_BOUNDARY.contains("!= runtime unresolvable"));
    assert!(VAR_ORDER_BOUNDARY.contains("authored order is declaration provenance only"));
}

#[test]
fn accepted_authority_chain_is_present_and_candidate_independent() {
    assert!(BLOCK_RELATION_ORACLE.contains("forward-inner-shadowing"));
    assert!(BLOCK_RELATION_ORACLE.contains("inner-self-shadowing"));
    assert!(BLOCK_RELATION_ORACLE.contains("sibling-exclusion-later-top-fallback"));
    assert!(BLOCK_RELATION_ORACLE.contains("NoSelectedLexicalBindingTargetInCoveredRegions"));

    assert!(VAR_ORACLE.contains("VariableStatement ::= var SelectedBindingIdentifier ;"));
    assert!(VAR_ORACLE.contains("repeated-var-alone-is-not-lexical-var-collision"));
    assert!(VAR_ORACLE.contains("canonical-equivalence-does-not-normalize"));
    assert!(VAR_ORACLE.contains("block-local-lexical-does-not-enter-script-domain"));

    assert!(VAR_COMPOSITION_ORACLE.contains("REPEATED_VAR_EVENTS"));
    assert!(
        VAR_COMPOSITION_ORACLE.contains(
            "repeated_var_preserves_first_var_provenance_until_lexical_collision_completes"
        )
    );

    assert!(
        CURRENT_COMPLETION_ORACLE
            .contains("LexicalDeclaration | SelectedBlock | SelectedVariableStatement")
    );
    assert!(CURRENT_COMPLETION_ORACLE.contains("EE-36-R02"));

    for (left, right) in [
        ("recognize_selected_lexical_", "slice("),
        ("evaluate_selected_static_", "semantics("),
        ("evaluate_selected_one_level_block_static_", "semantics("),
        ("evaluate_selected_variable_statement_static_", "semantics("),
        ("attempt_selected_", "qualification("),
        ("analyze_selected_binding_", "scope("),
        ("analyze_selected_one_level_block_binding_", "scope("),
        (
            "analyze_selected_variable_statement_name_",
            "correspondence(",
        ),
    ] {
        let forbidden = format!("{left}{right}");
        assert!(
            !THIS_SOURCE.contains(&forbidden),
            "#314 expected meaning must not call production owner {forbidden}"
        );
    }
}

#[test]
fn exact_twenty_two_fixture_matrix_and_all_owned_ranges_are_valid() {
    assert_eq!(FIXTURES.len(), 22);
    let mut ids = BTreeSet::new();

    for fixture in FIXTURES {
        assert!(
            ids.insert(fixture.id),
            "duplicate #314 fixture id {}",
            fixture.id
        );
        let source = SourceText::new(SourceId::new(ISSUE_ID), fixture.source.to_owned());

        for block in fixture.blocks {
            validate_anchor(&source, *block);
        }
        for control in fixture.controls {
            validate_anchor(&source, *control);
        }

        match fixture.processing {
            ExpectedProcessing::Complete(relations) => {
                for relation in relations {
                    validate_relation(&source, fixture.blocks, *relation);
                }
            }
            ExpectedProcessing::UpstreamPrerequisiteUnavailable(
                UpstreamPrerequisite::StaticSemanticsRejected { subject, .. },
            ) => validate_anchor(&source, subject),
            ExpectedProcessing::UpstreamPrerequisiteUnavailable(
                UpstreamPrerequisite::UnsupportedCoverage,
            ) => {}
        }
    }

    assert_eq!(ids.len(), 22);
}

#[test]
fn repeated_var_contributors_are_many_source_anchors_not_one_runtime_target() {
    let repeated = fixture("repeated-var-preserves-all-authored-contributors");
    let ExpectedProcessing::Complete([relation]) = repeated.processing else {
        panic!("repeated-var fixture must contain exactly one relation");
    };
    let ExpectedCorrespondence::SameSourceSelectedVarNameContributors { contributors } =
        relation.correspondence
    else {
        panic!("repeated-var fixture must retain authored var contributors");
    };

    assert_eq!(
        contributors,
        &[
            ExpectedAnchor::new(4, 5, "a"),
            ExpectedAnchor::new(11, 12, "a"),
        ]
    );
    assert_eq!(contributors.len(), 2);
    assert!(AUTHORED_VAR_BOUNDARY.contains("!= unique runtime binding identity"));
}

#[test]
fn authored_source_order_does_not_gate_var_name_correspondence() {
    let later_var = fixture("top-level-var-after-reference");
    let ExpectedProcessing::Complete([relation]) = later_var.processing else {
        panic!("later-var fixture must contain exactly one relation");
    };
    let ExpectedCorrespondence::SameSourceSelectedVarNameContributors { contributors } =
        relation.correspondence
    else {
        panic!("later-authored var must remain a same-source name contributor");
    };

    assert!(contributors[0].start > relation.reference.start);
    assert_eq!(contributors[0], ExpectedAnchor::new(13, 14, "a"));
    assert!(VAR_ORDER_BOUNDARY.contains("not Before/Same/After"));
}

#[test]
fn escaped_and_direct_spellings_use_exact_semantic_name_without_normalization() {
    for id in [
        "escaped-var-direct-reference",
        "direct-var-escaped-reference",
        "direct-and-escaped-var-contributors",
    ] {
        let selected = fixture(id);
        let ExpectedProcessing::Complete([relation]) = selected.processing else {
            panic!("{id} must contain one relation");
        };
        assert_eq!(relation.semantic_name, "a");
        assert_eq!(relation.semantic_code_points, A);
        assert!(matches!(
            relation.correspondence,
            ExpectedCorrespondence::SameSourceSelectedVarNameContributors { .. }
        ));
    }

    let distinct = fixture("canonical-equivalence-does-not-normalize");
    let ExpectedProcessing::Complete([relation]) = distinct.processing else {
        panic!("canonical distinction fixture must contain one relation");
    };
    assert_eq!(relation.semantic_code_points, E_COMBINING_ACUTE);
    assert_eq!(
        relation.correspondence,
        ExpectedCorrespondence::NoSelectedSameSourceContributor
    );
}

#[test]
fn lexical_shadowing_precedes_var_correspondence_on_the_covered_region_path() {
    for id in [
        "inner-lexical-wins",
        "later-authored-inner-lexical-still-wins",
        "self-lexical-wins-over-outer-var",
    ] {
        let selected = fixture(id);
        let ExpectedProcessing::Complete([relation]) = selected.processing else {
            panic!("{id} must contain one relation");
        };
        assert!(matches!(
            relation.correspondence,
            ExpectedCorrespondence::VisibleSelectedLexicalBinding { .. }
        ));
    }

    let forward = fixture("later-authored-inner-lexical-still-wins");
    let ExpectedProcessing::Complete([relation]) = forward.processing else {
        panic!("forward shadowing fixture must contain one relation");
    };
    let ExpectedCorrespondence::VisibleSelectedLexicalBinding { binding, region } =
        relation.correspondence
    else {
        panic!("later-authored inner lexical must be selected");
    };
    assert_eq!(region, ExpectedRegion::Block(F10_BLOCK));
    assert!(binding.start > relation.reference.start);
    assert_eq!(binding, ExpectedAnchor::new(22, 23, "a"));

    let self_shadow = fixture("self-lexical-wins-over-outer-var");
    let ExpectedProcessing::Complete([relation]) = self_shadow.processing else {
        panic!("self-shadowing fixture must contain one relation");
    };
    let ExpectedCorrespondence::VisibleSelectedLexicalBinding { binding, .. } =
        relation.correspondence
    else {
        panic!("self lexical must be visible");
    };
    assert_eq!(binding, relation.containing_binding);
    assert!(LEXICAL_CORRESPONDENCE_BOUNDARY.contains("!= proof of runtime binding instantiation"));
}

#[test]
fn block_fallback_and_sibling_exclusion_remain_distinct() {
    let fallback = fixture("block-falls-back-to-script-var-contributor");
    let ExpectedProcessing::Complete([relation]) = fallback.processing else {
        panic!("Block fallback fixture must contain one relation");
    };
    assert_eq!(relation.current_region, ExpectedRegion::Block(F8_BLOCK));
    assert_eq!(
        relation.correspondence,
        ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: F8_VARS,
        }
    );

    let sibling = fixture("sibling-lexical-excluded-script-var-fallback");
    let ExpectedProcessing::Complete([relation]) = sibling.processing else {
        panic!("sibling exclusion fixture must contain one relation");
    };
    assert_eq!(relation.current_region, ExpectedRegion::Block(F12_BLOCK_2));
    assert_eq!(
        relation.correspondence,
        ExpectedCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: F12_VARS,
        }
    );
    assert!(!F12_VARS.contains(&ExpectedAnchor::new(6, 7, "a")));

    let lexical = fixture("unrelated-var-does-not-perturb-lexical-fallback");
    let ExpectedProcessing::Complete([relation]) = lexical.processing else {
        panic!("lexical fallback fixture must contain one relation");
    };
    assert_eq!(
        relation.correspondence,
        ExpectedCorrespondence::VisibleSelectedLexicalBinding {
            binding: ExpectedAnchor::new(4, 5, "a"),
            region: ExpectedRegion::TopLevel,
        }
    );
}

#[test]
fn no_contributor_and_zero_relation_are_not_runtime_claims() {
    let no_contributor = fixture("no-selected-same-source-contributor");
    let ExpectedProcessing::Complete([relation]) = no_contributor.processing else {
        panic!("no-contributor fixture must contain one relation");
    };
    assert_eq!(
        relation.correspondence,
        ExpectedCorrespondence::NoSelectedSameSourceContributor
    );
    assert!(NO_CONTRIBUTOR_BOUNDARY.contains("!= ReferenceError"));

    let zero = fixture("accepted-repeated-var-with-zero-reference-relations");
    let ExpectedProcessing::Complete(relations) = zero.processing else {
        panic!("zero-relation fixture must be complete");
    };
    assert!(relations.is_empty());
}

#[test]
fn invalid_and_unsupported_prerequisites_never_acquire_correspondence_results() {
    for (id, expected_rule) in [
        (
            "script-lexical-var-collision-prerequisite-unavailable",
            "EE-36-R02",
        ),
        ("block-duplicate-prerequisite-unavailable", "EE-14-R01"),
        ("escaped-reserved-var-prerequisite-unavailable", "EE-04-R08"),
    ] {
        let selected = fixture(id);
        let ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::StaticSemanticsRejected { rule_id, .. },
        ) = selected.processing
        else {
            panic!("{id} must remain unavailable after upstream static rejection");
        };
        assert_eq!(rule_id, expected_rule);
    }

    for id in [
        "var-initializer-remains-unsupported",
        "grouped-reference-remains-unsupported",
        "var-eof-asi-remains-unsupported",
        "incomplete-block-prevents-tentative-relation",
    ] {
        let selected = fixture(id);
        assert_eq!(
            selected.processing,
            ExpectedProcessing::UpstreamPrerequisiteUnavailable(
                UpstreamPrerequisite::UnsupportedCoverage
            ),
            "{id} must not acquire a correspondence result"
        );
    }
}
