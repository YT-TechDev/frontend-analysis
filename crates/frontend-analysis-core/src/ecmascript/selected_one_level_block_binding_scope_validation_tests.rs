//! Candidate-independent one-level Block hierarchical Binding / Scope validation for Issue #302.
//!
//! This module materializes fixture-owned nearest-target expectations by composing
//! already-accepted architecture and validation authorities. It deliberately does
//! not call production lexical, static-semantics, Binding / Scope, aggregate
//! qualification, or runtime code to derive expected target meaning.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

const ISSUE_ID: u64 = 302;
const FLAT_RELATION_ORACLE: &str = include_str!("selected_binding_scope_validation_tests.rs");
const FLAT_ORDER_ORACLE: &str =
    include_str!("selected_lexical_initialization_validation_tests.rs");
const BLOCK_ORACLE: &str =
    include_str!("qualification_selected_one_level_block_validation_tests.rs");
const BLOCK_COMPOSITION_ORACLE: &str =
    include_str!("qualification_selected_one_level_block_composition_validation_tests.rs");
const TERMINAL_COMPOSITION_ORACLE: &str =
    include_str!("qualification_selected_one_level_block_terminal_composition_validation_tests.rs");
const CURRENT_COMPLETION_ORACLE: &str =
    include_str!("qualification_validation_tests/selected_one_level_block_slice_completion.rs");
const ESCAPED_BINDING_ORACLE: &str =
    include_str!("qualification_validation_tests/escaped_identifier.rs");
const THIS_SOURCE: &str =
    include_str!("selected_one_level_block_binding_scope_validation_tests.rs");

const PRIMARY_BOUNDARY: &str = "nearest selected lexical target identity != runtime ResolveBinding != Environment Record identity != TDZ state != runtime value";
const ORDER_COMPATIBILITY_BOUNDARY: &str = "hierarchical nearest-target validation may pin Before/Same/After compatibility without requiring future production to expose structural order";

const A_CODE_POINTS: &[u32] = &[0x61];
const B_CODE_POINTS: &[u32] = &[0x62];
const X_CODE_POINTS: &[u32] = &[0x78];
const Y_CODE_POINTS: &[u32] = &[0x79];
const Z_CODE_POINTS: &[u32] = &[0x7a];
const E_COMBINING_ACUTE_CODE_POINTS: &[u32] = &[0x65, 0x0301];

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
enum ExpectedTarget {
    SelectedLexicalBinding {
        binding: ExpectedAnchor,
        region: ExpectedRegion,
    },
    NoSelectedLexicalBindingTargetInCoveredRegions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedRelation {
    containing_binding: ExpectedAnchor,
    current_region: ExpectedRegion,
    reference: ExpectedAnchor,
    semantic_name: &'static str,
    semantic_code_points: &'static [u32],
    target: ExpectedTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpstreamPrerequisite {
    StaticSemanticsRejected {
        rule_id: &'static str,
        subject: ExpectedAnchor,
    },
    UnsupportedCoverage,
    DefinitiveGrammarRejected {
        subject: ExpectedAnchor,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedProcessing {
    Complete(&'static [ExpectedRelation]),
    UpstreamPrerequisiteUnavailable(UpstreamPrerequisite),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureProvenance {
    ExistingBlockAuthority,
    HierarchicalComposition,
    UpstreamPrerequisiteControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RelationFixture {
    id: &'static str,
    source: &'static str,
    provenance: FixtureProvenance,
    blocks: &'static [ExpectedAnchor],
    processing: ExpectedProcessing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedStructuralOrder {
    Before,
    Same,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedOrderCompatibility {
    fixture_id: &'static str,
    containing_binding: ExpectedAnchor,
    reference: ExpectedAnchor,
    target_binding: ExpectedAnchor,
    order: ExpectedStructuralOrder,
}

const MIXED_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(9, 30, "{ let a=2; let x=a; }")];
const MIXED_RELATIONS: &[ExpectedRelation] = &[
    ExpectedRelation {
        containing_binding: ExpectedAnchor::new(24, 25, "x"),
        current_region: ExpectedRegion::Block(MIXED_BLOCKS[0]),
        reference: ExpectedAnchor::new(26, 27, "a"),
        semantic_name: "a",
        semantic_code_points: A_CODE_POINTS,
        target: ExpectedTarget::SelectedLexicalBinding {
            binding: ExpectedAnchor::new(15, 16, "a"),
            region: ExpectedRegion::Block(MIXED_BLOCKS[0]),
        },
    },
    ExpectedRelation {
        containing_binding: ExpectedAnchor::new(35, 36, "y"),
        current_region: ExpectedRegion::TopLevel,
        reference: ExpectedAnchor::new(37, 38, "a"),
        semantic_name: "a",
        semantic_code_points: A_CODE_POINTS,
        target: ExpectedTarget::SelectedLexicalBinding {
            binding: ExpectedAnchor::new(4, 5, "a"),
            region: ExpectedRegion::TopLevel,
        },
    },
];

const FORWARD_INNER_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(9, 30, "{ let x=a; let a=1; }")];
const FORWARD_INNER_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(FORWARD_INNER_BLOCKS[0]),
    reference: ExpectedAnchor::new(17, 18, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(24, 25, "a"),
        region: ExpectedRegion::Block(FORWARD_INNER_BLOCKS[0]),
    },
}];

const SELF_INNER_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(9, 21, "{ let x=x; }")];
const SELF_INNER_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(SELF_INNER_BLOCKS[0]),
    reference: ExpectedAnchor::new(17, 18, "x"),
    semantic_name: "x",
    semantic_code_points: X_CODE_POINTS,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(15, 16, "x"),
        region: ExpectedRegion::Block(SELF_INNER_BLOCKS[0]),
    },
}];

const OUTER_FALLBACK_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(9, 21, "{ let x=a; }")];
const OUTER_FALLBACK_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(OUTER_FALLBACK_BLOCKS[0]),
    reference: ExpectedAnchor::new(17, 18, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(4, 5, "a"),
        region: ExpectedRegion::TopLevel,
    },
}];

const SIBLING_FALLBACK_BLOCKS: &[ExpectedAnchor] = &[
    ExpectedAnchor::new(0, 12, "{ let a=1; }"),
    ExpectedAnchor::new(13, 25, "{ let x=a; }"),
];
const SIBLING_FALLBACK_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(19, 20, "x"),
    current_region: ExpectedRegion::Block(SIBLING_FALLBACK_BLOCKS[1]),
    reference: ExpectedAnchor::new(21, 22, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(30, 31, "a"),
        region: ExpectedRegion::TopLevel,
    },
}];

const CHILD_EXCLUSION_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(9, 21, "{ let a=1; }")];
const CHILD_EXCLUSION_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(4, 5, "x"),
    current_region: ExpectedRegion::TopLevel,
    reference: ExpectedAnchor::new(6, 7, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(26, 27, "a"),
        region: ExpectedRegion::TopLevel,
    },
}];

const SAME_SOURCE_NO_TARGET_BLOCKS: &[ExpectedAnchor] = &[
    ExpectedAnchor::new(0, 12, "{ let a=1; }"),
    ExpectedAnchor::new(13, 25, "{ let x=a; }"),
];
const SAME_SOURCE_NO_TARGET_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(19, 20, "x"),
    current_region: ExpectedRegion::Block(SAME_SOURCE_NO_TARGET_BLOCKS[1]),
    reference: ExpectedAnchor::new(21, 22, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    target: ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions,
}];

const GENUINE_NO_TARGET_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(0, 12, "{ let x=y; }")];
const GENUINE_NO_TARGET_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(6, 7, "x"),
    current_region: ExpectedRegion::Block(GENUINE_NO_TARGET_BLOCKS[0]),
    reference: ExpectedAnchor::new(8, 9, "y"),
    semantic_name: "y",
    semantic_code_points: Y_CODE_POINTS,
    target: ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions,
}];

const ZERO_RELATION_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(9, 21, "{ let a=2; }")];

const ESCAPED_REFERENCE_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(9, 26, r"{ let x=\u0061; }")];
const ESCAPED_REFERENCE_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(ESCAPED_REFERENCE_BLOCKS[0]),
    reference: ExpectedAnchor::new(17, 23, r"\u0061"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(4, 5, "a"),
        region: ExpectedRegion::TopLevel,
    },
}];

const ESCAPED_INNER_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(9, 35, r"{ let x=a; let \u{61}=1; }")];
const ESCAPED_INNER_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(ESCAPED_INNER_BLOCKS[0]),
    reference: ExpectedAnchor::new(17, 18, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(24, 30, r"\u{61}"),
        region: ExpectedRegion::Block(ESCAPED_INNER_BLOCKS[0]),
    },
}];

const CANONICAL_DISTINCT_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(10, 28, r"{ let x=e\u0301; }")];
const CANONICAL_DISTINCT_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(16, 17, "x"),
    current_region: ExpectedRegion::Block(CANONICAL_DISTINCT_BLOCKS[0]),
    reference: ExpectedAnchor::new(18, 25, r"e\u0301"),
    semantic_name: "e\u{301}",
    semantic_code_points: E_COMBINING_ACUTE_CODE_POINTS,
    target: ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions,
}];

const MULTI_RELATION_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(18, 48, "{ let x=a; let y=z; let z=3; }")];
const MULTI_RELATIONS: &[ExpectedRelation] = &[
    ExpectedRelation {
        containing_binding: ExpectedAnchor::new(24, 25, "x"),
        current_region: ExpectedRegion::Block(MULTI_RELATION_BLOCKS[0]),
        reference: ExpectedAnchor::new(26, 27, "a"),
        semantic_name: "a",
        semantic_code_points: A_CODE_POINTS,
        target: ExpectedTarget::SelectedLexicalBinding {
            binding: ExpectedAnchor::new(4, 5, "a"),
            region: ExpectedRegion::TopLevel,
        },
    },
    ExpectedRelation {
        containing_binding: ExpectedAnchor::new(33, 34, "y"),
        current_region: ExpectedRegion::Block(MULTI_RELATION_BLOCKS[0]),
        reference: ExpectedAnchor::new(35, 36, "z"),
        semantic_name: "z",
        semantic_code_points: Z_CODE_POINTS,
        target: ExpectedTarget::SelectedLexicalBinding {
            binding: ExpectedAnchor::new(42, 43, "z"),
            region: ExpectedRegion::Block(MULTI_RELATION_BLOCKS[0]),
        },
    },
    ExpectedRelation {
        containing_binding: ExpectedAnchor::new(53, 54, "q"),
        current_region: ExpectedRegion::TopLevel,
        reference: ExpectedAnchor::new(55, 56, "b"),
        semantic_name: "b",
        semantic_code_points: B_CODE_POINTS,
        target: ExpectedTarget::SelectedLexicalBinding {
            binding: ExpectedAnchor::new(13, 14, "b"),
            region: ExpectedRegion::TopLevel,
        },
    },
];

const BLOCK_DUPLICATE_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(0, 30, "{ let x=a; let a=1; let a=2; }")];
const SCRIPT_DUPLICATE_BLOCKS: &[ExpectedAnchor] =
    &[ExpectedAnchor::new(9, 21, "{ let x=a; }")];

const FIXTURES: &[RelationFixture] = &[
    RelationFixture {
        id: "mixed-inner-shadowing-and-outer-preservation",
        source: "let a=1; { let a=2; let x=a; } let y=a;",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: MIXED_BLOCKS,
        processing: ExpectedProcessing::Complete(MIXED_RELATIONS),
    },
    RelationFixture {
        id: "forward-inner-shadowing",
        source: "let a=0; { let x=a; let a=1; }",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: FORWARD_INNER_BLOCKS,
        processing: ExpectedProcessing::Complete(FORWARD_INNER_RELATIONS),
    },
    RelationFixture {
        id: "inner-self-shadowing",
        source: "let x=0; { let x=x; }",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: SELF_INNER_BLOCKS,
        processing: ExpectedProcessing::Complete(SELF_INNER_RELATIONS),
    },
    RelationFixture {
        id: "outer-fallback",
        source: "let a=1; { let x=a; }",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: OUTER_FALLBACK_BLOCKS,
        processing: ExpectedProcessing::Complete(OUTER_FALLBACK_RELATIONS),
    },
    RelationFixture {
        id: "sibling-exclusion-later-top-fallback",
        source: "{ let a=1; } { let x=a; } let a=2;",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: SIBLING_FALLBACK_BLOCKS,
        processing: ExpectedProcessing::Complete(SIBLING_FALLBACK_RELATIONS),
    },
    RelationFixture {
        id: "child-exclusion-later-top-target",
        source: "let x=a; { let a=1; } let a=2;",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: CHILD_EXCLUSION_BLOCKS,
        processing: ExpectedProcessing::Complete(CHILD_EXCLUSION_RELATIONS),
    },
    RelationFixture {
        id: "same-source-match-outside-search-path",
        source: "{ let a=1; } { let x=a; }",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: SAME_SOURCE_NO_TARGET_BLOCKS,
        processing: ExpectedProcessing::Complete(SAME_SOURCE_NO_TARGET_RELATIONS),
    },
    RelationFixture {
        id: "genuine-no-target",
        source: "{ let x=y; }",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: GENUINE_NO_TARGET_BLOCKS,
        processing: ExpectedProcessing::Complete(GENUINE_NO_TARGET_RELATIONS),
    },
    RelationFixture {
        id: "block-enabled-zero-relations",
        source: "let a=1; { let a=2; }",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: ZERO_RELATION_BLOCKS,
        processing: ExpectedProcessing::Complete(&[]),
    },
    RelationFixture {
        id: "escaped-reference-direct-outer-target",
        source: r"let a=1; { let x=\u0061; }",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: ESCAPED_REFERENCE_BLOCKS,
        processing: ExpectedProcessing::Complete(ESCAPED_REFERENCE_RELATIONS),
    },
    RelationFixture {
        id: "direct-reference-escaped-inner-target",
        source: r"let a=0; { let x=a; let \u{61}=1; }",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: ESCAPED_INNER_BLOCKS,
        processing: ExpectedProcessing::Complete(ESCAPED_INNER_RELATIONS),
    },
    RelationFixture {
        id: "canonical-distinct-no-normalization",
        source: "let é=1; { let x=e\\u0301; }",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: CANONICAL_DISTINCT_BLOCKS,
        processing: ExpectedProcessing::Complete(CANONICAL_DISTINCT_RELATIONS),
    },
    RelationFixture {
        id: "multiple-relations-reference-order",
        source: "let a=1; let b=2; { let x=a; let y=z; let z=3; } let q=b;",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: MULTI_RELATION_BLOCKS,
        processing: ExpectedProcessing::Complete(MULTI_RELATIONS),
    },
    RelationFixture {
        id: "block-static-rejection",
        source: "{ let x=a; let a=1; let a=2; }",
        provenance: FixtureProvenance::UpstreamPrerequisiteControl,
        blocks: BLOCK_DUPLICATE_BLOCKS,
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::StaticSemanticsRejected {
                rule_id: "EE-14-R01",
                subject: ExpectedAnchor::new(24, 25, "a"),
            },
        ),
    },
    RelationFixture {
        id: "script-static-rejection",
        source: "let a=1; { let x=a; } let a=2;",
        provenance: FixtureProvenance::UpstreamPrerequisiteControl,
        blocks: SCRIPT_DUPLICATE_BLOCKS,
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::StaticSemanticsRejected {
                rule_id: "EE-36-R01",
                subject: ExpectedAnchor::new(26, 27, "a"),
            },
        ),
    },
    RelationFixture {
        id: "incomplete-block",
        source: "{ let x=y;",
        provenance: FixtureProvenance::UpstreamPrerequisiteControl,
        blocks: &[],
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::UnsupportedCoverage,
        ),
    },
    RelationFixture {
        id: "definitive-grammar-after-tentative-relation",
        source: r"{ let x=y; let \u{}",
        provenance: FixtureProvenance::UpstreamPrerequisiteControl,
        blocks: &[],
        processing: ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::DefinitiveGrammarRejected {
                subject: ExpectedAnchor::new(15, 19, r"\u{}"),
            },
        ),
    },
];

const ORDER_COMPATIBILITY: &[ExpectedOrderCompatibility] = &[
    ExpectedOrderCompatibility {
        fixture_id: "forward-inner-shadowing",
        containing_binding: ExpectedAnchor::new(15, 16, "x"),
        reference: ExpectedAnchor::new(17, 18, "a"),
        target_binding: ExpectedAnchor::new(24, 25, "a"),
        order: ExpectedStructuralOrder::After,
    },
    ExpectedOrderCompatibility {
        fixture_id: "inner-self-shadowing",
        containing_binding: ExpectedAnchor::new(15, 16, "x"),
        reference: ExpectedAnchor::new(17, 18, "x"),
        target_binding: ExpectedAnchor::new(15, 16, "x"),
        order: ExpectedStructuralOrder::Same,
    },
    ExpectedOrderCompatibility {
        fixture_id: "outer-fallback",
        containing_binding: ExpectedAnchor::new(15, 16, "x"),
        reference: ExpectedAnchor::new(17, 18, "a"),
        target_binding: ExpectedAnchor::new(4, 5, "a"),
        order: ExpectedStructuralOrder::Before,
    },
    ExpectedOrderCompatibility {
        fixture_id: "sibling-exclusion-later-top-fallback",
        containing_binding: ExpectedAnchor::new(19, 20, "x"),
        reference: ExpectedAnchor::new(21, 22, "a"),
        target_binding: ExpectedAnchor::new(30, 31, "a"),
        order: ExpectedStructuralOrder::After,
    },
    ExpectedOrderCompatibility {
        fixture_id: "child-exclusion-later-top-target",
        containing_binding: ExpectedAnchor::new(4, 5, "x"),
        reference: ExpectedAnchor::new(6, 7, "a"),
        target_binding: ExpectedAnchor::new(26, 27, "a"),
        order: ExpectedStructuralOrder::After,
    },
    ExpectedOrderCompatibility {
        fixture_id: "direct-reference-escaped-inner-target",
        containing_binding: ExpectedAnchor::new(15, 16, "x"),
        reference: ExpectedAnchor::new(17, 18, "a"),
        target_binding: ExpectedAnchor::new(24, 30, r"\u{61}"),
        order: ExpectedStructuralOrder::After,
    },
];

fn fixture(id: &str) -> &'static RelationFixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing #302 fixture {id}"))
}

fn validate_anchor(source: &SourceText, expected: ExpectedAnchor) {
    let anchor = source
        .anchor(expected.start, expected.end)
        .unwrap_or_else(|error| panic!("invalid fixture anchor {expected:?}: {error}"));
    assert_eq!(anchor.fragment(), expected.fragment);
}

fn validate_region(source: &SourceText, region: ExpectedRegion, blocks: &[ExpectedAnchor]) {
    match region {
        ExpectedRegion::TopLevel => {}
        ExpectedRegion::Block(block) => {
            assert!(
                blocks.contains(&block),
                "relation region must be one of the fixture-owned Block anchors"
            );
            validate_anchor(source, block);
        }
    }
}

fn decoded_expected_name(code_points: &[u32]) -> String {
    let mut decoded = String::new();
    for code_point in code_points {
        let scalar = char::from_u32(*code_point)
            .unwrap_or_else(|| panic!("fixture contains non-scalar U+{code_point:04X}"));
        decoded.push(scalar);
    }
    decoded
}

fn source_section<'source>(
    source: &'source str,
    start_marker: &str,
    end_marker: &str,
) -> &'source str {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("missing source section start {start_marker}"));
    let rest = &source[start..];
    let end = rest
        .find(end_marker)
        .unwrap_or_else(|| panic!("missing source section end {end_marker}"));
    &rest[..end]
}

#[test]
fn accepted_authority_chain_is_present_without_candidate_dependence() {
    assert!(FLAT_RELATION_ORACLE.contains("SameSourceSelectedLexicalBinding"));
    assert!(FLAT_RELATION_ORACLE.contains("NoSameSourceSelectedLexicalBinding"));
    assert!(FLAT_RELATION_ORACLE.contains("relation ordering is source-order deterministic"));

    assert!(FLAT_ORDER_ORACLE.contains("TargetBindingBeforeContainingBinding"));
    assert!(FLAT_ORDER_ORACLE.contains("TargetIsContainingBinding"));
    assert!(FLAT_ORDER_ORACLE.contains("TargetBindingAfterContainingBinding"));
    assert!(FLAT_ORDER_ORACLE.contains("P1"));
    assert!(FLAT_ORDER_ORACLE.contains("P7"));

    assert!(BLOCK_ORACLE.contains("lexical-region source structure != hierarchical Binding / Scope target selection"));
    assert!(BLOCK_ORACLE.contains("id: \"mixed-shadowing\""));
    assert!(BLOCK_ORACLE.contains("id: \"outer-fallback-shape\""));
    assert!(BLOCK_ORACLE.contains("id: \"sibling-regions\""));
    assert!(BLOCK_ORACLE.contains("id: \"no-match-shape\""));
    assert!(BLOCK_ORACLE.contains("id: \"escaped-spelling\""));
    assert!(BLOCK_ORACLE.contains("id: \"canonical-distinct\""));
    assert!(BLOCK_ORACLE.contains("id: \"legal-outer-inner-shadowing\""));

    assert!(BLOCK_COMPOSITION_ORACLE.contains("REGION_DOMAIN_FIXTURES"));
    assert!(BLOCK_COMPOSITION_ORACLE.contains("GRAMMAR_VS_BLOCK"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("INCOMPLETE_BLOCK_FIXTURES"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("INCOMPLETE_BLOCK_COMPETITION_FIXTURES"));

    assert!(CURRENT_COMPLETION_ORACLE.contains("SelectedPositivePartition"));
    assert!(CURRENT_COMPLETION_ORACLE.contains("BlockEnabled"));
    assert!(CURRENT_COMPLETION_ORACLE.contains("CURRENT_SELECTED_TOP_LEVEL_ITEM_GRAMMAR"));

    assert!(ESCAPED_BINDING_ORACLE.contains("permanent_long_braced_gold_and_large_leading_zero_challenge"));
    assert!(ESCAPED_BINDING_ORACLE.contains("no Unicode normalization"));

    assert!(PRIMARY_BOUNDARY.contains("nearest selected lexical target identity"));
    assert!(PRIMARY_BOUNDARY.contains("runtime ResolveBinding"));
    assert!(ORDER_COMPATIBILITY_BOUNDARY.contains("without requiring future production"));

    for forbidden in [
        concat!("recognize_selected_", "lexical_slice("),
        concat!("evaluate_selected_one_level_block_", "static_semantics("),
        concat!("analyze_selected_", "binding_scope("),
        concat!("attempt_selected_", "qualification("),
        concat!("QualificationOutcome::", "qualified("),
        concat!("CompleteQualification", "Witness {"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "#302 expected meaning must remain candidate-independent: {forbidden}"
        );
    }
}

#[test]
fn seventeen_fixture_matrix_is_unique_source_backed_and_block_bounded() {
    assert_eq!(FIXTURES.len(), 17);

    let mut ids = BTreeSet::new();
    let mut sources = BTreeSet::new();
    let mut complete = 0usize;
    let mut upstream = 0usize;

    for fixture in FIXTURES {
        assert!(ids.insert(fixture.id), "duplicate fixture id {}", fixture.id);
        assert!(
            sources.insert(fixture.source),
            "duplicate fixture source {}",
            fixture.source
        );

        let source = SourceText::new(SourceId::new(ISSUE_ID), fixture.source.to_owned());
        for block in fixture.blocks {
            validate_anchor(&source, *block);
            assert_eq!(block.fragment.as_bytes().first(), Some(&b'{'));
            assert_eq!(block.fragment.as_bytes().last(), Some(&b'}'));
        }

        match fixture.processing {
            ExpectedProcessing::Complete(relations) => {
                complete += 1;
                assert!(
                    !fixture.blocks.is_empty(),
                    "positive relation semantics must be Block-enabled: {}",
                    fixture.id
                );

                let mut previous_reference_start = None;
                for relation in relations {
                    validate_anchor(&source, relation.containing_binding);
                    validate_anchor(&source, relation.reference);
                    validate_region(&source, relation.current_region, fixture.blocks);
                    assert_eq!(
                        decoded_expected_name(relation.semantic_code_points),
                        relation.semantic_name
                    );

                    match relation.target {
                        ExpectedTarget::SelectedLexicalBinding { binding, region } => {
                            validate_anchor(&source, binding);
                            validate_region(&source, region, fixture.blocks);
                        }
                        ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions => {}
                    }

                    if let Some(previous) = previous_reference_start {
                        assert!(
                            previous < relation.reference.start,
                            "observable relations must remain in authored reference order"
                        );
                    }
                    previous_reference_start = Some(relation.reference.start);
                }
            }
            ExpectedProcessing::UpstreamPrerequisiteUnavailable(reason) => {
                upstream += 1;
                assert_eq!(
                    fixture.provenance,
                    FixtureProvenance::UpstreamPrerequisiteControl
                );

                match reason {
                    UpstreamPrerequisite::StaticSemanticsRejected {
                        rule_id,
                        subject,
                    } => {
                        assert!(matches!(rule_id, "EE-14-R01" | "EE-36-R01"));
                        validate_anchor(&source, subject);
                    }
                    UpstreamPrerequisite::UnsupportedCoverage => {}
                    UpstreamPrerequisite::DefinitiveGrammarRejected { subject } => {
                        validate_anchor(&source, subject);
                    }
                }
            }
        }
    }

    assert_eq!(complete, 13);
    assert_eq!(upstream, 4);
    assert_eq!(ids.len(), 17);
    assert_eq!(sources.len(), 17);
}

#[test]
fn no_target_meaning_distinguishes_absence_from_search_path_exclusion() {
    let same_source = fixture("same-source-match-outside-search-path");
    let ExpectedProcessing::Complete(same_source_relations) = same_source.processing else {
        panic!("same-source search-path fixture must be complete");
    };
    assert_eq!(same_source_relations.len(), 1);
    assert!(matches!(
        same_source_relations[0].target,
        ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions
    ));
    assert_eq!(
        same_source.source.as_bytes()[6],
        b'a',
        "same SourceText must contain a matching sibling binding"
    );

    let genuine = fixture("genuine-no-target");
    let ExpectedProcessing::Complete(genuine_relations) = genuine.processing else {
        panic!("genuine no-target fixture must be complete");
    };
    assert_eq!(genuine_relations.len(), 1);
    assert!(matches!(
        genuine_relations[0].target,
        ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions
    ));

    assert_ne!(same_source.source, genuine.source);
}

#[test]
fn zero_reference_positive_source_is_complete_with_empty_relations() {
    let fixture = fixture("block-enabled-zero-relations");
    let ExpectedProcessing::Complete(relations) = fixture.processing else {
        panic!("zero-reference Block-enabled fixture must be complete");
    };
    assert!(relations.is_empty());
    assert_eq!(fixture.blocks.len(), 1);
}

#[test]
fn primary_nearest_target_falsifiers_pin_region_search_not_source_proximity() {
    let forward = fixture("forward-inner-shadowing");
    let ExpectedProcessing::Complete(forward_relations) = forward.processing else {
        panic!("forward inner fixture must be complete");
    };
    let relation = forward_relations[0];
    let ExpectedTarget::SelectedLexicalBinding { binding, region } = relation.target else {
        panic!("forward inner fixture must have a selected target");
    };
    assert_eq!(binding, ExpectedAnchor::new(24, 25, "a"));
    assert_eq!(region, ExpectedRegion::Block(FORWARD_INNER_BLOCKS[0]));
    assert!(binding.start > relation.reference.start);

    let sibling = fixture("sibling-exclusion-later-top-fallback");
    let ExpectedProcessing::Complete(sibling_relations) = sibling.processing else {
        panic!("sibling fallback fixture must be complete");
    };
    let ExpectedTarget::SelectedLexicalBinding { binding, region } = sibling_relations[0].target
    else {
        panic!("sibling fallback fixture must have a selected target");
    };
    assert_eq!(binding, ExpectedAnchor::new(30, 31, "a"));
    assert_eq!(region, ExpectedRegion::TopLevel);
    assert_ne!(binding, ExpectedAnchor::new(6, 7, "a"));

    let child = fixture("child-exclusion-later-top-target");
    let ExpectedProcessing::Complete(child_relations) = child.processing else {
        panic!("child exclusion fixture must be complete");
    };
    let ExpectedTarget::SelectedLexicalBinding { binding, region } = child_relations[0].target else {
        panic!("child exclusion fixture must have a selected target");
    };
    assert_eq!(binding, ExpectedAnchor::new(26, 27, "a"));
    assert_eq!(region, ExpectedRegion::TopLevel);
    assert_ne!(binding, ExpectedAnchor::new(15, 16, "a"));
}

#[test]
fn escaped_and_unicode_fixtures_preserve_semantic_identity_without_normalization() {
    let escaped_ref = fixture("escaped-reference-direct-outer-target");
    let ExpectedProcessing::Complete(escaped_ref_relations) = escaped_ref.processing else {
        panic!("escaped reference fixture must be complete");
    };
    assert_eq!(escaped_ref_relations[0].reference.fragment, r"\u0061");
    assert_eq!(escaped_ref_relations[0].semantic_name, "a");

    let escaped_binding = fixture("direct-reference-escaped-inner-target");
    let ExpectedProcessing::Complete(escaped_binding_relations) = escaped_binding.processing else {
        panic!("escaped binding fixture must be complete");
    };
    let ExpectedTarget::SelectedLexicalBinding { binding, .. } =
        escaped_binding_relations[0].target
    else {
        panic!("escaped binding fixture must have a selected target");
    };
    assert_eq!(binding.fragment, r"\u{61}");
    assert_eq!(escaped_binding_relations[0].semantic_name, "a");

    let canonical = fixture("canonical-distinct-no-normalization");
    let ExpectedProcessing::Complete(canonical_relations) = canonical.processing else {
        panic!("canonical-distinct fixture must be complete");
    };
    assert_eq!(canonical_relations[0].semantic_name, "e\u{301}");
    assert_eq!(
        canonical_relations[0].semantic_code_points,
        E_COMBINING_ACUTE_CODE_POINTS
    );
    assert!(matches!(
        canonical_relations[0].target,
        ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions
    ));
}

#[test]
fn multiple_relation_fixture_pins_reference_order_not_target_region_order() {
    let fixture = fixture("multiple-relations-reference-order");
    let ExpectedProcessing::Complete(relations) = fixture.processing else {
        panic!("multiple-relation fixture must be complete");
    };
    assert_eq!(relations.len(), 3);
    assert_eq!(
        relations
            .iter()
            .map(|relation| relation.reference.start)
            .collect::<Vec<_>>(),
        vec![26, 35, 55]
    );

    assert!(matches!(
        relations[0].target,
        ExpectedTarget::SelectedLexicalBinding {
            region: ExpectedRegion::TopLevel,
            ..
        }
    ));
    assert!(matches!(
        relations[1].target,
        ExpectedTarget::SelectedLexicalBinding {
            region: ExpectedRegion::Block(_),
            ..
        }
    ));
    assert!(matches!(
        relations[2].target,
        ExpectedTarget::SelectedLexicalBinding {
            region: ExpectedRegion::TopLevel,
            ..
        }
    ));
}

#[test]
fn upstream_prerequisite_controls_do_not_become_relation_results() {
    let block_static = fixture("block-static-rejection");
    assert!(matches!(
        block_static.processing,
        ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::StaticSemanticsRejected {
                rule_id: "EE-14-R01",
                ..
            }
        )
    ));

    let script_static = fixture("script-static-rejection");
    assert!(matches!(
        script_static.processing,
        ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::StaticSemanticsRejected {
                rule_id: "EE-36-R01",
                ..
            }
        )
    ));

    let incomplete = fixture("incomplete-block");
    assert!(matches!(
        incomplete.processing,
        ExpectedProcessing::UpstreamPrerequisiteUnavailable(
            UpstreamPrerequisite::UnsupportedCoverage
        )
    ));

    let grammar = fixture("definitive-grammar-after-tentative-relation");
    let ExpectedProcessing::UpstreamPrerequisiteUnavailable(
        UpstreamPrerequisite::DefinitiveGrammarRejected { subject },
    ) = grammar.processing
    else {
        panic!("grammar fixture must remain an upstream prerequisite failure");
    };
    assert_eq!(subject, ExpectedAnchor::new(15, 19, r"\u{}"));
}

#[test]
fn structural_order_is_secondary_compatibility_not_primary_target_requirement() {
    assert_eq!(ORDER_COMPATIBILITY.len(), 6);

    let relation_section = source_section(
        THIS_SOURCE,
        "struct ExpectedRelation",
        "enum UpstreamPrerequisite",
    );
    assert!(
        !relation_section.contains("order:"),
        "primary nearest-target relation must not require structural order"
    );

    for expectation in ORDER_COMPATIBILITY {
        let fixture = fixture(expectation.fixture_id);
        let source = SourceText::new(SourceId::new(ISSUE_ID), fixture.source.to_owned());
        validate_anchor(&source, expectation.containing_binding);
        validate_anchor(&source, expectation.reference);
        validate_anchor(&source, expectation.target_binding);

        let ExpectedProcessing::Complete(relations) = fixture.processing else {
            panic!("order compatibility may only refer to complete fixtures");
        };
        let relation = relations
            .iter()
            .find(|relation| relation.reference == expectation.reference)
            .unwrap_or_else(|| {
                panic!(
                    "missing primary relation for order compatibility {}",
                    expectation.fixture_id
                )
            });
        let ExpectedTarget::SelectedLexicalBinding { binding, .. } = relation.target else {
            panic!("order compatibility requires a selected lexical target");
        };
        assert_eq!(binding, expectation.target_binding);
    }

    let self_compat = ORDER_COMPATIBILITY
        .iter()
        .find(|entry| entry.fixture_id == "inner-self-shadowing")
        .expect("missing inner self compatibility");
    assert_eq!(self_compat.order, ExpectedStructuralOrder::Same);
    assert!(
        self_compat.target_binding.start < self_compat.reference.start,
        "self fixture must continue to falsify raw byte-order semantics"
    );
}

#[test]
fn provenance_layers_are_explicit_without_rewriting_historical_oracles() {
    let existing = FIXTURES
        .iter()
        .filter(|fixture| fixture.provenance == FixtureProvenance::ExistingBlockAuthority)
        .count();
    let composition = FIXTURES
        .iter()
        .filter(|fixture| fixture.provenance == FixtureProvenance::HierarchicalComposition)
        .count();
    let upstream = FIXTURES
        .iter()
        .filter(|fixture| fixture.provenance == FixtureProvenance::UpstreamPrerequisiteControl)
        .count();

    assert_eq!(existing, 6);
    assert_eq!(composition, 7);
    assert_eq!(upstream, 4);

    assert!(FLAT_RELATION_ORACLE.contains("Candidate-Independent Selected Binding / Scope"));
    assert!(FLAT_ORDER_ORACLE.contains("Candidate-Independent Validation Leaf"));
    assert!(CURRENT_COMPLETION_ORACLE.contains("HISTORICAL_FLAT_COMPLETION"));
}
