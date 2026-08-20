//! Candidate-independent one-level Block hierarchical Binding / Scope validation for Issue #302.
//!
//! Expected nearest-target relations are fixture-owned. This oracle composes
//! accepted authority without calling production lexical, static, Binding /
//! Scope, qualification, or runtime behavior to derive expected meaning.

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
const NO_TARGET_BOUNDARY: &str = "no selected lexical target in covered regions != runtime unbound != ReferenceError != initialized != uninitialized";
const ORDER_COMPATIBILITY_BOUNDARY: &str = "Before/Same/After compatibility may be pinned without requiring future hierarchical production to expose structural order";

const A: &[u32] = &[0x61];
const B: &[u32] = &[0x62];
const X: &[u32] = &[0x78];
const Y: &[u32] = &[0x79];
const Z: &[u32] = &[0x7a];
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

const MIXED_BLOCK: ExpectedAnchor = ExpectedAnchor::new(9, 30, "{ let a=2; let x=a; }");
const MIXED_RELATIONS: &[ExpectedRelation] = &[
    ExpectedRelation {
        containing_binding: ExpectedAnchor::new(24, 25, "x"),
        current_region: ExpectedRegion::Block(MIXED_BLOCK),
        reference: ExpectedAnchor::new(26, 27, "a"),
        semantic_name: "a",
        semantic_code_points: A,
        target: ExpectedTarget::SelectedLexicalBinding {
            binding: ExpectedAnchor::new(15, 16, "a"),
            region: ExpectedRegion::Block(MIXED_BLOCK),
        },
    },
    ExpectedRelation {
        containing_binding: ExpectedAnchor::new(35, 36, "y"),
        current_region: ExpectedRegion::TopLevel,
        reference: ExpectedAnchor::new(37, 38, "a"),
        semantic_name: "a",
        semantic_code_points: A,
        target: ExpectedTarget::SelectedLexicalBinding {
            binding: ExpectedAnchor::new(4, 5, "a"),
            region: ExpectedRegion::TopLevel,
        },
    },
];

const FORWARD_BLOCK: ExpectedAnchor = ExpectedAnchor::new(9, 30, "{ let x=a; let a=1; }");
const FORWARD_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(FORWARD_BLOCK),
    reference: ExpectedAnchor::new(17, 18, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(24, 25, "a"),
        region: ExpectedRegion::Block(FORWARD_BLOCK),
    },
}];

const SELF_BLOCK: ExpectedAnchor = ExpectedAnchor::new(9, 21, "{ let x=x; }");
const SELF_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(SELF_BLOCK),
    reference: ExpectedAnchor::new(17, 18, "x"),
    semantic_name: "x",
    semantic_code_points: X,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(15, 16, "x"),
        region: ExpectedRegion::Block(SELF_BLOCK),
    },
}];

const OUTER_BLOCK: ExpectedAnchor = ExpectedAnchor::new(9, 21, "{ let x=a; }");
const OUTER_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(OUTER_BLOCK),
    reference: ExpectedAnchor::new(17, 18, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(4, 5, "a"),
        region: ExpectedRegion::TopLevel,
    },
}];

const SIBLING_FIRST: ExpectedAnchor = ExpectedAnchor::new(0, 12, "{ let a=1; }");
const SIBLING_SECOND: ExpectedAnchor = ExpectedAnchor::new(13, 25, "{ let x=a; }");
const SIBLING_BLOCKS: &[ExpectedAnchor] = &[SIBLING_FIRST, SIBLING_SECOND];
const SIBLING_FALLBACK_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(19, 20, "x"),
    current_region: ExpectedRegion::Block(SIBLING_SECOND),
    reference: ExpectedAnchor::new(21, 22, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(30, 31, "a"),
        region: ExpectedRegion::TopLevel,
    },
}];
const SAME_SOURCE_NO_TARGET_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(19, 20, "x"),
    current_region: ExpectedRegion::Block(SIBLING_SECOND),
    reference: ExpectedAnchor::new(21, 22, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    target: ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions,
}];

const CHILD_BLOCK: ExpectedAnchor = ExpectedAnchor::new(9, 21, "{ let a=1; }");
const CHILD_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(4, 5, "x"),
    current_region: ExpectedRegion::TopLevel,
    reference: ExpectedAnchor::new(6, 7, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(26, 27, "a"),
        region: ExpectedRegion::TopLevel,
    },
}];

const GENUINE_NO_TARGET_BLOCK: ExpectedAnchor = ExpectedAnchor::new(0, 12, "{ let x=y; }");
const GENUINE_NO_TARGET_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(6, 7, "x"),
    current_region: ExpectedRegion::Block(GENUINE_NO_TARGET_BLOCK),
    reference: ExpectedAnchor::new(8, 9, "y"),
    semantic_name: "y",
    semantic_code_points: Y,
    target: ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions,
}];

const ZERO_RELATION_BLOCK: ExpectedAnchor = ExpectedAnchor::new(9, 21, "{ let a=2; }");

const ESCAPED_REFERENCE_BLOCK: ExpectedAnchor =
    ExpectedAnchor::new(9, 26, r"{ let x=\u0061; }");
const ESCAPED_REFERENCE_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(ESCAPED_REFERENCE_BLOCK),
    reference: ExpectedAnchor::new(17, 23, r"\u0061"),
    semantic_name: "a",
    semantic_code_points: A,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(4, 5, "a"),
        region: ExpectedRegion::TopLevel,
    },
}];

const ESCAPED_INNER_BLOCK: ExpectedAnchor =
    ExpectedAnchor::new(9, 35, r"{ let x=a; let \u{61}=1; }");
const ESCAPED_INNER_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    current_region: ExpectedRegion::Block(ESCAPED_INNER_BLOCK),
    reference: ExpectedAnchor::new(17, 18, "a"),
    semantic_name: "a",
    semantic_code_points: A,
    target: ExpectedTarget::SelectedLexicalBinding {
        binding: ExpectedAnchor::new(24, 30, r"\u{61}"),
        region: ExpectedRegion::Block(ESCAPED_INNER_BLOCK),
    },
}];

const CANONICAL_BLOCK: ExpectedAnchor = ExpectedAnchor::new(10, 28, r"{ let x=e\u0301; }");
const CANONICAL_RELATIONS: &[ExpectedRelation] = &[ExpectedRelation {
    containing_binding: ExpectedAnchor::new(16, 17, "x"),
    current_region: ExpectedRegion::Block(CANONICAL_BLOCK),
    reference: ExpectedAnchor::new(18, 25, r"e\u0301"),
    semantic_name: "e\u{301}",
    semantic_code_points: E_COMBINING_ACUTE,
    target: ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions,
}];

const MULTI_BLOCK: ExpectedAnchor =
    ExpectedAnchor::new(18, 48, "{ let x=a; let y=z; let z=3; }");
const MULTI_RELATIONS: &[ExpectedRelation] = &[
    ExpectedRelation {
        containing_binding: ExpectedAnchor::new(24, 25, "x"),
        current_region: ExpectedRegion::Block(MULTI_BLOCK),
        reference: ExpectedAnchor::new(26, 27, "a"),
        semantic_name: "a",
        semantic_code_points: A,
        target: ExpectedTarget::SelectedLexicalBinding {
            binding: ExpectedAnchor::new(4, 5, "a"),
            region: ExpectedRegion::TopLevel,
        },
    },
    ExpectedRelation {
        containing_binding: ExpectedAnchor::new(33, 34, "y"),
        current_region: ExpectedRegion::Block(MULTI_BLOCK),
        reference: ExpectedAnchor::new(35, 36, "z"),
        semantic_name: "z",
        semantic_code_points: Z,
        target: ExpectedTarget::SelectedLexicalBinding {
            binding: ExpectedAnchor::new(42, 43, "z"),
            region: ExpectedRegion::Block(MULTI_BLOCK),
        },
    },
    ExpectedRelation {
        containing_binding: ExpectedAnchor::new(53, 54, "q"),
        current_region: ExpectedRegion::TopLevel,
        reference: ExpectedAnchor::new(55, 56, "b"),
        semantic_name: "b",
        semantic_code_points: B,
        target: ExpectedTarget::SelectedLexicalBinding {
            binding: ExpectedAnchor::new(13, 14, "b"),
            region: ExpectedRegion::TopLevel,
        },
    },
];

const BLOCK_DUPLICATE_BLOCK: ExpectedAnchor =
    ExpectedAnchor::new(0, 30, "{ let x=a; let a=1; let a=2; }");
const SCRIPT_DUPLICATE_BLOCK: ExpectedAnchor = ExpectedAnchor::new(9, 21, "{ let x=a; }");

const FIXTURES: &[RelationFixture] = &[
    RelationFixture {
        id: "mixed-inner-shadowing-and-outer-preservation",
        source: "let a=1; { let a=2; let x=a; } let y=a;",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: &[MIXED_BLOCK],
        processing: ExpectedProcessing::Complete(MIXED_RELATIONS),
    },
    RelationFixture {
        id: "forward-inner-shadowing",
        source: "let a=0; { let x=a; let a=1; }",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: &[FORWARD_BLOCK],
        processing: ExpectedProcessing::Complete(FORWARD_RELATIONS),
    },
    RelationFixture {
        id: "inner-self-shadowing",
        source: "let x=0; { let x=x; }",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: &[SELF_BLOCK],
        processing: ExpectedProcessing::Complete(SELF_RELATIONS),
    },
    RelationFixture {
        id: "outer-fallback",
        source: "let a=1; { let x=a; }",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: &[OUTER_BLOCK],
        processing: ExpectedProcessing::Complete(OUTER_RELATIONS),
    },
    RelationFixture {
        id: "sibling-exclusion-later-top-fallback",
        source: "{ let a=1; } { let x=a; } let a=2;",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: SIBLING_BLOCKS,
        processing: ExpectedProcessing::Complete(SIBLING_FALLBACK_RELATIONS),
    },
    RelationFixture {
        id: "child-exclusion-later-top-target",
        source: "let x=a; { let a=1; } let a=2;",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: &[CHILD_BLOCK],
        processing: ExpectedProcessing::Complete(CHILD_RELATIONS),
    },
    RelationFixture {
        id: "same-source-match-outside-search-path",
        source: "{ let a=1; } { let x=a; }",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: SIBLING_BLOCKS,
        processing: ExpectedProcessing::Complete(SAME_SOURCE_NO_TARGET_RELATIONS),
    },
    RelationFixture {
        id: "genuine-no-target",
        source: "{ let x=y; }",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: &[GENUINE_NO_TARGET_BLOCK],
        processing: ExpectedProcessing::Complete(GENUINE_NO_TARGET_RELATIONS),
    },
    RelationFixture {
        id: "block-enabled-zero-relations",
        source: "let a=1; { let a=2; }",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: &[ZERO_RELATION_BLOCK],
        processing: ExpectedProcessing::Complete(&[]),
    },
    RelationFixture {
        id: "escaped-reference-direct-outer-target",
        source: r"let a=1; { let x=\u0061; }",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: &[ESCAPED_REFERENCE_BLOCK],
        processing: ExpectedProcessing::Complete(ESCAPED_REFERENCE_RELATIONS),
    },
    RelationFixture {
        id: "direct-reference-escaped-inner-target",
        source: r"let a=0; { let x=a; let \u{61}=1; }",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: &[ESCAPED_INNER_BLOCK],
        processing: ExpectedProcessing::Complete(ESCAPED_INNER_RELATIONS),
    },
    RelationFixture {
        id: "canonical-distinct-no-normalization",
        source: "let é=1; { let x=e\\u0301; }",
        provenance: FixtureProvenance::ExistingBlockAuthority,
        blocks: &[CANONICAL_BLOCK],
        processing: ExpectedProcessing::Complete(CANONICAL_RELATIONS),
    },
    RelationFixture {
        id: "multiple-relations-reference-order",
        source: "let a=1; let b=2; { let x=a; let y=z; let z=3; } let q=b;",
        provenance: FixtureProvenance::HierarchicalComposition,
        blocks: &[MULTI_BLOCK],
        processing: ExpectedProcessing::Complete(MULTI_RELATIONS),
    },
    RelationFixture {
        id: "block-static-rejection",
        source: "{ let x=a; let a=1; let a=2; }",
        provenance: FixtureProvenance::UpstreamPrerequisiteControl,
        blocks: &[BLOCK_DUPLICATE_BLOCK],
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
        blocks: &[SCRIPT_DUPLICATE_BLOCK],
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
            assert!(blocks.contains(&block));
            validate_anchor(source, block);
        }
    }
}

fn decoded_expected_name(code_points: &[u32]) -> String {
    let mut decoded = String::new();
    for code_point in code_points {
        decoded.push(
            char::from_u32(*code_point)
                .unwrap_or_else(|| panic!("fixture contains non-scalar U+{code_point:04X}")),
        );
    }
    decoded
}

fn source_section<'source>(source: &'source str, start: &str, end: &str) -> &'source str {
    let start_offset = source
        .find(start)
        .unwrap_or_else(|| panic!("missing source section start {start}"));
    let rest = &source[start_offset..];
    let end_offset = rest
        .find(end)
        .unwrap_or_else(|| panic!("missing source section end {end}"));
    &rest[..end_offset]
}

#[test]
fn accepted_authority_chain_is_present_and_candidate_independent() {
    assert!(FLAT_RELATION_ORACLE.contains("Candidate-independent validation for the first selected Binding / Scope"));
    assert!(FLAT_RELATION_ORACLE.contains("SameSourceSelectedLexicalBinding"));
    assert!(FLAT_RELATION_ORACLE.contains("NoSameSourceSelectedLexicalBinding"));
    assert!(FLAT_RELATION_ORACLE.contains("multiple_relations_are_ordered_by_reference_source_order"));

    assert!(FLAT_ORDER_ORACLE.contains("Candidate-independent validation for the selected lexical initialization-order"));
    assert!(FLAT_ORDER_ORACLE.contains("TargetBindingBeforeContainingBinding"));
    assert!(FLAT_ORDER_ORACLE.contains("TargetBindingAfterContainingBinding"));
    assert!(FLAT_ORDER_ORACLE.contains("P1"));
    assert!(FLAT_ORDER_ORACLE.contains("P7"));

    assert!(BLOCK_ORACLE.contains("lexical-region source structure != hierarchical Binding / Scope target selection"));
    for marker in [
        "id: \"mixed-shadowing\"",
        "id: \"outer-fallback-shape\"",
        "id: \"sibling-regions\"",
        "id: \"no-match-shape\"",
        "id: \"escaped-spelling\"",
        "id: \"canonical-distinct\"",
        "id: \"legal-outer-inner-shadowing\"",
    ] {
        assert!(BLOCK_ORACLE.contains(marker), "missing #285 authority {marker}");
    }

    assert!(BLOCK_COMPOSITION_ORACLE.contains("REGION_DOMAIN_FIXTURES"));
    assert!(BLOCK_COMPOSITION_ORACLE.contains("DELIMITER_FIXTURES"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("INCOMPLETE_BLOCK_FIXTURES"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("INCOMPLETE_BLOCK_COMPETITION_FIXTURES"));

    assert!(CURRENT_COMPLETION_ORACLE.contains("SelectedPositivePartition"));
    assert!(CURRENT_COMPLETION_ORACLE.contains("BlockEnabled"));
    assert!(CURRENT_COMPLETION_ORACLE.contains("CURRENT_SELECTED_TOP_LEVEL_ITEM_GRAMMAR"));

    assert!(ESCAPED_BINDING_ORACLE.contains("permanent_long_braced_gold_and_large_leading_zero_challenge"));
    assert!(ESCAPED_BINDING_ORACLE.contains("validation must not apply Unicode normalization"));

    assert!(PRIMARY_BOUNDARY.contains("runtime ResolveBinding"));
    assert!(NO_TARGET_BOUNDARY.contains("runtime unbound"));
    assert!(NO_TARGET_BOUNDARY.contains("ReferenceError"));
    assert!(ORDER_COMPATIBILITY_BOUNDARY.contains("without requiring future hierarchical production"));

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
        assert!(sources.insert(fixture.source), "duplicate fixture source {}", fixture.source);
        let source = SourceText::new(SourceId::new(ISSUE_ID), fixture.source.to_owned());

        for block in fixture.blocks {
            validate_anchor(&source, *block);
            assert_eq!(block.fragment.as_bytes().first(), Some(&b'{'));
            assert_eq!(block.fragment.as_bytes().last(), Some(&b'}'));
        }

        match fixture.processing {
            ExpectedProcessing::Complete(relations) => {
                complete += 1;
                assert!(!fixture.blocks.is_empty(), "positive fixture must be Block-enabled");
                let mut previous_reference = None;
                for relation in relations {
                    validate_anchor(&source, relation.containing_binding);
                    validate_anchor(&source, relation.reference);
                    validate_region(&source, relation.current_region, fixture.blocks);
                    assert_eq!(decoded_expected_name(relation.semantic_code_points), relation.semantic_name);
                    if let ExpectedTarget::SelectedLexicalBinding { binding, region } = relation.target {
                        validate_anchor(&source, binding);
                        validate_region(&source, region, fixture.blocks);
                    }
                    if let Some(previous) = previous_reference {
                        assert!(previous < relation.reference.start);
                    }
                    previous_reference = Some(relation.reference.start);
                }
            }
            ExpectedProcessing::UpstreamPrerequisiteUnavailable(reason) => {
                upstream += 1;
                assert_eq!(fixture.provenance, FixtureProvenance::UpstreamPrerequisiteControl);
                match reason {
                    UpstreamPrerequisite::StaticSemanticsRejected { rule_id, subject } => {
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

    assert_eq!((complete, upstream), (13, 4));
    assert_eq!(ids.len(), 17);
    assert_eq!(sources.len(), 17);
}

#[test]
fn target_selection_falsifiers_are_explicit() {
    let mixed = fixture("mixed-inner-shadowing-and-outer-preservation");
    let ExpectedProcessing::Complete(relations) = mixed.processing else {
        panic!("mixed fixture must be complete");
    };
    assert_eq!(relations.len(), 2);
    assert!(matches!(relations[0].target, ExpectedTarget::SelectedLexicalBinding { binding: ExpectedAnchor { start: 15, end: 16, .. }, region: ExpectedRegion::Block(_) }));
    assert!(matches!(relations[1].target, ExpectedTarget::SelectedLexicalBinding { binding: ExpectedAnchor { start: 4, end: 5, .. }, region: ExpectedRegion::TopLevel }));

    let forward = fixture("forward-inner-shadowing");
    let ExpectedProcessing::Complete(relations) = forward.processing else {
        panic!("forward fixture must be complete");
    };
    let ExpectedTarget::SelectedLexicalBinding { binding, region } = relations[0].target else {
        panic!("forward fixture must have target");
    };
    assert_eq!(binding, ExpectedAnchor::new(24, 25, "a"));
    assert_eq!(region, ExpectedRegion::Block(FORWARD_BLOCK));
    assert!(binding.start > relations[0].reference.start);

    let self_shadow = fixture("inner-self-shadowing");
    let ExpectedProcessing::Complete(relations) = self_shadow.processing else {
        panic!("self fixture must be complete");
    };
    assert!(matches!(relations[0].target, ExpectedTarget::SelectedLexicalBinding { binding: ExpectedAnchor { start: 15, end: 16, .. }, .. }));

    let sibling = fixture("sibling-exclusion-later-top-fallback");
    let ExpectedProcessing::Complete(relations) = sibling.processing else {
        panic!("sibling fixture must be complete");
    };
    let ExpectedTarget::SelectedLexicalBinding { binding, region } = relations[0].target else {
        panic!("sibling fixture must have target");
    };
    assert_eq!(binding, ExpectedAnchor::new(30, 31, "a"));
    assert_eq!(region, ExpectedRegion::TopLevel);
    assert_ne!(binding, ExpectedAnchor::new(6, 7, "a"));

    let child = fixture("child-exclusion-later-top-target");
    let ExpectedProcessing::Complete(relations) = child.processing else {
        panic!("child fixture must be complete");
    };
    let ExpectedTarget::SelectedLexicalBinding { binding, region } = relations[0].target else {
        panic!("child fixture must have target");
    };
    assert_eq!(binding, ExpectedAnchor::new(26, 27, "a"));
    assert_eq!(region, ExpectedRegion::TopLevel);
    assert_ne!(binding, ExpectedAnchor::new(15, 16, "a"));
}

#[test]
fn no_target_distinguishes_true_absence_from_search_path_exclusion() {
    let excluded = fixture("same-source-match-outside-search-path");
    let ExpectedProcessing::Complete(relations) = excluded.processing else {
        panic!("excluded fixture must be complete");
    };
    assert!(matches!(relations[0].target, ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions));
    let source = SourceText::new(SourceId::new(ISSUE_ID), excluded.source.to_owned());
    validate_anchor(&source, ExpectedAnchor::new(6, 7, "a"));

    let genuine = fixture("genuine-no-target");
    let ExpectedProcessing::Complete(relations) = genuine.processing else {
        panic!("genuine fixture must be complete");
    };
    assert!(matches!(relations[0].target, ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions));
    assert_ne!(excluded.source, genuine.source);
}

#[test]
fn zero_reference_positive_source_remains_complete() {
    let zero = fixture("block-enabled-zero-relations");
    let ExpectedProcessing::Complete(relations) = zero.processing else {
        panic!("zero-reference fixture must be complete");
    };
    assert!(relations.is_empty());
    assert_eq!(zero.blocks, &[ZERO_RELATION_BLOCK]);
}

#[test]
fn escaped_and_unicode_identity_are_exact_and_non_normalized() {
    let escaped_reference = fixture("escaped-reference-direct-outer-target");
    let ExpectedProcessing::Complete(relations) = escaped_reference.processing else {
        panic!("escaped-reference fixture must be complete");
    };
    assert_eq!(relations[0].reference.fragment, r"\u0061");
    assert_eq!(relations[0].semantic_name, "a");

    let escaped_binding = fixture("direct-reference-escaped-inner-target");
    let ExpectedProcessing::Complete(relations) = escaped_binding.processing else {
        panic!("escaped-binding fixture must be complete");
    };
    let ExpectedTarget::SelectedLexicalBinding { binding, .. } = relations[0].target else {
        panic!("escaped-binding fixture must have target");
    };
    assert_eq!(binding, ExpectedAnchor::new(24, 30, r"\u{61}"));

    let canonical = fixture("canonical-distinct-no-normalization");
    let ExpectedProcessing::Complete(relations) = canonical.processing else {
        panic!("canonical fixture must be complete");
    };
    let source = SourceText::new(SourceId::new(ISSUE_ID), canonical.source.to_owned());
    validate_anchor(&source, ExpectedAnchor::new(4, 6, "é"));
    assert_eq!(relations[0].semantic_code_points, E_COMBINING_ACUTE);
    assert_eq!(relations[0].semantic_name, "e\u{301}");
    assert!(matches!(relations[0].target, ExpectedTarget::NoSelectedLexicalBindingTargetInCoveredRegions));
}

#[test]
fn multiple_relations_preserve_reference_order_not_target_region_order() {
    let multiple = fixture("multiple-relations-reference-order");
    let ExpectedProcessing::Complete(relations) = multiple.processing else {
        panic!("multiple fixture must be complete");
    };
    assert_eq!(relations.iter().map(|relation| relation.reference.start).collect::<Vec<_>>(), vec![26, 35, 55]);
    assert!(matches!(relations[0].target, ExpectedTarget::SelectedLexicalBinding { region: ExpectedRegion::TopLevel, .. }));
    assert!(matches!(relations[1].target, ExpectedTarget::SelectedLexicalBinding { region: ExpectedRegion::Block(_), .. }));
    assert!(matches!(relations[2].target, ExpectedTarget::SelectedLexicalBinding { region: ExpectedRegion::TopLevel, .. }));
}

#[test]
fn upstream_failures_never_become_relation_results() {
    assert!(matches!(fixture("block-static-rejection").processing, ExpectedProcessing::UpstreamPrerequisiteUnavailable(UpstreamPrerequisite::StaticSemanticsRejected { rule_id: "EE-14-R01", .. })));
    assert!(matches!(fixture("script-static-rejection").processing, ExpectedProcessing::UpstreamPrerequisiteUnavailable(UpstreamPrerequisite::StaticSemanticsRejected { rule_id: "EE-36-R01", .. })));
    assert!(matches!(fixture("incomplete-block").processing, ExpectedProcessing::UpstreamPrerequisiteUnavailable(UpstreamPrerequisite::UnsupportedCoverage)));

    let grammar = fixture("definitive-grammar-after-tentative-relation");
    let ExpectedProcessing::UpstreamPrerequisiteUnavailable(UpstreamPrerequisite::DefinitiveGrammarRejected { subject }) = grammar.processing else {
        panic!("grammar fixture must remain an upstream prerequisite failure");
    };
    assert_eq!(subject, ExpectedAnchor::new(15, 19, r"\u{}"));
}

#[test]
fn structural_order_is_secondary_compatibility_not_primary_requirement() {
    assert_eq!(ORDER_COMPATIBILITY.len(), 6);
    let primary_section = source_section(THIS_SOURCE, "struct ExpectedRelation", "enum UpstreamPrerequisite");
    assert!(!primary_section.contains("order:"));

    for expectation in ORDER_COMPATIBILITY {
        let fixture = fixture(expectation.fixture_id);
        let source = SourceText::new(SourceId::new(ISSUE_ID), fixture.source.to_owned());
        validate_anchor(&source, expectation.containing_binding);
        validate_anchor(&source, expectation.reference);
        validate_anchor(&source, expectation.target_binding);

        let ExpectedProcessing::Complete(relations) = fixture.processing else {
            panic!("order compatibility requires a complete fixture");
        };
        let relation = relations
            .iter()
            .find(|relation| relation.reference == expectation.reference)
            .unwrap_or_else(|| panic!("missing primary relation for {}", expectation.fixture_id));
        let ExpectedTarget::SelectedLexicalBinding { binding, .. } = relation.target else {
            panic!("order compatibility requires a selected target");
        };
        assert_eq!(binding, expectation.target_binding);
    }

    let self_order = ORDER_COMPATIBILITY
        .iter()
        .find(|entry| entry.fixture_id == "inner-self-shadowing")
        .expect("missing self compatibility");
    assert_eq!(self_order.order, ExpectedStructuralOrder::Same);
    assert!(self_order.target_binding.start < self_order.reference.start);
}

#[test]
fn provenance_layers_remain_distinct_and_historical_oracles_are_not_reowned() {
    let existing = FIXTURES.iter().filter(|fixture| fixture.provenance == FixtureProvenance::ExistingBlockAuthority).count();
    let composition = FIXTURES.iter().filter(|fixture| fixture.provenance == FixtureProvenance::HierarchicalComposition).count();
    let upstream = FIXTURES.iter().filter(|fixture| fixture.provenance == FixtureProvenance::UpstreamPrerequisiteControl).count();
    assert_eq!((existing, composition, upstream), (6, 7, 4));

    assert!(FLAT_RELATION_ORACLE.contains("Candidate-independent validation for the first selected Binding / Scope"));
    assert!(FLAT_ORDER_ORACLE.contains("Candidate-independent validation for the selected lexical initialization-order"));
    assert!(CURRENT_COMPLETION_ORACLE.contains("HISTORICAL_FLAT_COMPLETION"));
}
