//! Candidate-independent validation for the first one-level selected Block /
//! lexical-region frontier accepted by Issue #283.
//!
//! Expected grammar applicability, authored region/binding/reference ranges,
//! Block Early Error reachability, and future nearest-selected-binding meaning
//! are owned by this test module. Future production Block or hierarchical
//! Binding / Scope behavior must not define these expectations.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

use super::inventory::{RULE_UNITS, RuleUnitKind};

const THIS_SOURCE: &str = include_str!("selected_block_region_validation.rs");
const HISTORICAL_FLAT_COMPLETION_SOURCE: &str = include_str!("selected_slice_completion.rs");
const FROZEN_INVENTORY_SOURCE: &str = include_str!("inventory.rs");

const SELECTED_GRAMMAR_FRONTIER: &str =
    "Script items = LexicalDeclaration | one-level SelectedBlock; SelectedBlock body = LexicalDeclaration+";
const HISTORICAL_FLAT_GRAMMAR: &str = "LexicalDeclaration+";
const EE14_R01: &str = "EE-14-R01";
const EE14_R02: &str = "EE-14-R02";
const EE14_R02_NON_TRIGGER: &str =
    "selected Block body has no VarDeclaredNames contributor";
const ANNEX_B_BLOCK_FUNCTION_NON_TRIGGER: &str =
    "selected Block grammar has no FunctionDeclaration, so the Block-Level Function Declaration duplicate exception cannot trigger";
const NO_MATCH_BOUNDARY: &str =
    "NoSameSourceSelectedLexicalBinding != runtime unbound != ReferenceError != runtime ResolveBinding failure";
const HISTORICAL_COMPLETION_BOUNDARY: &str =
    "historical flat #268 completion remains authoritative only for LexicalDeclaration+";

const SELECTED_BLOCK_ALLOWS_NESTED_BLOCK: bool = false;
const SELECTED_BLOCK_HAS_VAR_DECLARATION: bool = false;
const SELECTED_BLOCK_HAS_FUNCTION_DECLARATION: bool = false;
const SELECTED_BLOCK_HAS_EXPRESSION_STATEMENT: bool = false;
const SELECTED_BLOCK_REQUIRES_AUTHORED_SEMICOLON: bool = true;
const SELECTED_CAN_FORM_DIRECTIVE_PROLOGUE: bool = false;
const SELECTED_IS_STRICT: bool = false;
const SELECTED_INCLUDES_NORMATIVE_OPTIONAL_SOURCE_STATIC: bool = false;

const _: () = {
    assert!(!SELECTED_BLOCK_ALLOWS_NESTED_BLOCK);
    assert!(!SELECTED_BLOCK_HAS_VAR_DECLARATION);
    assert!(!SELECTED_BLOCK_HAS_FUNCTION_DECLARATION);
    assert!(!SELECTED_BLOCK_HAS_EXPRESSION_STATEMENT);
    assert!(SELECTED_BLOCK_REQUIRES_AUTHORED_SEMICOLON);
    assert!(!SELECTED_CAN_FORM_DIRECTIVE_PROLOGUE);
    assert!(!SELECTED_IS_STRICT);
    assert!(!SELECTED_INCLUDES_NORMATIVE_OPTIONAL_SOURCE_STATIC);
};

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
struct ExpectedRegion {
    id: &'static str,
    parent: Option<&'static str>,
    authored_block: Option<ExpectedAnchor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedBinding {
    anchor: ExpectedAnchor,
    semantic_name: &'static str,
    region: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedTarget {
    SameSourceSelectedLexicalBinding {
        binding: ExpectedAnchor,
        region: &'static str,
    },
    NoSameSourceSelectedLexicalBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedReference {
    containing_binding: ExpectedAnchor,
    reference: ExpectedAnchor,
    semantic_name: &'static str,
    semantic_code_points: &'static [u32],
    region: &'static str,
    target: ExpectedTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedProcessing {
    SelectedFrontierAccepted,
    StaticRejected { authority: &'static str },
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockFixture {
    id: &'static str,
    source: &'static str,
    processing: ExpectedProcessing,
    regions: &'static [ExpectedRegion],
    bindings: &'static [ExpectedBinding],
    references: &'static [ExpectedReference],
}

const A_CODE_POINTS: &[u32] = &[0x61];
const Y_CODE_POINTS: &[u32] = &[0x79];
const E_COMBINING_ACUTE_CODE_POINTS: &[u32] = &[0x65, 0x0301];

const INNER_SHADOW_REGIONS: &[ExpectedRegion] = &[
    ExpectedRegion {
        id: "script",
        parent: None,
        authored_block: None,
    },
    ExpectedRegion {
        id: "block-1",
        parent: Some("script"),
        authored_block: Some(ExpectedAnchor::new(
            9,
            30,
            "{ let a=2; let x=a; }",
        )),
    },
];
const INNER_SHADOW_BINDINGS: &[ExpectedBinding] = &[
    ExpectedBinding {
        anchor: ExpectedAnchor::new(4, 5, "a"),
        semantic_name: "a",
        region: "script",
    },
    ExpectedBinding {
        anchor: ExpectedAnchor::new(15, 16, "a"),
        semantic_name: "a",
        region: "block-1",
    },
    ExpectedBinding {
        anchor: ExpectedAnchor::new(24, 25, "x"),
        semantic_name: "x",
        region: "block-1",
    },
];
const INNER_SHADOW_REFERENCES: &[ExpectedReference] = &[ExpectedReference {
    containing_binding: ExpectedAnchor::new(24, 25, "x"),
    reference: ExpectedAnchor::new(26, 27, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    region: "block-1",
    target: ExpectedTarget::SameSourceSelectedLexicalBinding {
        binding: ExpectedAnchor::new(15, 16, "a"),
        region: "block-1",
    },
}];

const OUTER_FALLBACK_REGIONS: &[ExpectedRegion] = &[
    ExpectedRegion {
        id: "script",
        parent: None,
        authored_block: None,
    },
    ExpectedRegion {
        id: "block-1",
        parent: Some("script"),
        authored_block: Some(ExpectedAnchor::new(9, 21, "{ let x=a; }")),
    },
];
const OUTER_FALLBACK_BINDINGS: &[ExpectedBinding] = &[
    ExpectedBinding {
        anchor: ExpectedAnchor::new(4, 5, "a"),
        semantic_name: "a",
        region: "script",
    },
    ExpectedBinding {
        anchor: ExpectedAnchor::new(15, 16, "x"),
        semantic_name: "x",
        region: "block-1",
    },
];
const OUTER_FALLBACK_REFERENCES: &[ExpectedReference] = &[ExpectedReference {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    reference: ExpectedAnchor::new(17, 18, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    region: "block-1",
    target: ExpectedTarget::SameSourceSelectedLexicalBinding {
        binding: ExpectedAnchor::new(4, 5, "a"),
        region: "script",
    },
}];

const SIBLING_REGIONS: &[ExpectedRegion] = &[
    ExpectedRegion {
        id: "script",
        parent: None,
        authored_block: None,
    },
    ExpectedRegion {
        id: "block-1",
        parent: Some("script"),
        authored_block: Some(ExpectedAnchor::new(0, 12, "{ let a=1; }")),
    },
    ExpectedRegion {
        id: "block-2",
        parent: Some("script"),
        authored_block: Some(ExpectedAnchor::new(13, 25, "{ let x=a; }")),
    },
];
const SIBLING_BINDINGS: &[ExpectedBinding] = &[
    ExpectedBinding {
        anchor: ExpectedAnchor::new(6, 7, "a"),
        semantic_name: "a",
        region: "block-1",
    },
    ExpectedBinding {
        anchor: ExpectedAnchor::new(19, 20, "x"),
        semantic_name: "x",
        region: "block-2",
    },
];
const SIBLING_REFERENCES: &[ExpectedReference] = &[ExpectedReference {
    containing_binding: ExpectedAnchor::new(19, 20, "x"),
    reference: ExpectedAnchor::new(21, 22, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    region: "block-2",
    target: ExpectedTarget::NoSameSourceSelectedLexicalBinding,
}];

const INNER_DUP_REGIONS: &[ExpectedRegion] = &[
    ExpectedRegion {
        id: "script",
        parent: None,
        authored_block: None,
    },
    ExpectedRegion {
        id: "block-1",
        parent: Some("script"),
        authored_block: Some(ExpectedAnchor::new(0, 21, "{ let a=1; let a=2; }")),
    },
];
const INNER_DUP_BINDINGS: &[ExpectedBinding] = &[
    ExpectedBinding {
        anchor: ExpectedAnchor::new(6, 7, "a"),
        semantic_name: "a",
        region: "block-1",
    },
    ExpectedBinding {
        anchor: ExpectedAnchor::new(15, 16, "a"),
        semantic_name: "a",
        region: "block-1",
    },
];

const LEGAL_SHADOW_REGIONS: &[ExpectedRegion] = &[
    ExpectedRegion {
        id: "script",
        parent: None,
        authored_block: None,
    },
    ExpectedRegion {
        id: "block-1",
        parent: Some("script"),
        authored_block: Some(ExpectedAnchor::new(9, 21, "{ let a=2; }")),
    },
];
const LEGAL_SHADOW_BINDINGS: &[ExpectedBinding] = &[
    ExpectedBinding {
        anchor: ExpectedAnchor::new(4, 5, "a"),
        semantic_name: "a",
        region: "script",
    },
    ExpectedBinding {
        anchor: ExpectedAnchor::new(15, 16, "a"),
        semantic_name: "a",
        region: "block-1",
    },
];

const NO_MATCH_REGIONS: &[ExpectedRegion] = &[
    ExpectedRegion {
        id: "script",
        parent: None,
        authored_block: None,
    },
    ExpectedRegion {
        id: "block-1",
        parent: Some("script"),
        authored_block: Some(ExpectedAnchor::new(0, 12, "{ let x=y; }")),
    },
];
const NO_MATCH_BINDINGS: &[ExpectedBinding] = &[ExpectedBinding {
    anchor: ExpectedAnchor::new(6, 7, "x"),
    semantic_name: "x",
    region: "block-1",
}];
const NO_MATCH_REFERENCES: &[ExpectedReference] = &[ExpectedReference {
    containing_binding: ExpectedAnchor::new(6, 7, "x"),
    reference: ExpectedAnchor::new(8, 9, "y"),
    semantic_name: "y",
    semantic_code_points: Y_CODE_POINTS,
    region: "block-1",
    target: ExpectedTarget::NoSameSourceSelectedLexicalBinding,
}];

const ESCAPED_OUTER_REGIONS: &[ExpectedRegion] = &[
    ExpectedRegion {
        id: "script",
        parent: None,
        authored_block: None,
    },
    ExpectedRegion {
        id: "block-1",
        parent: Some("script"),
        authored_block: Some(ExpectedAnchor::new(9, 26, "{ let x=\\u0061; }")),
    },
];
const ESCAPED_OUTER_BINDINGS: &[ExpectedBinding] = &[
    ExpectedBinding {
        anchor: ExpectedAnchor::new(4, 5, "a"),
        semantic_name: "a",
        region: "script",
    },
    ExpectedBinding {
        anchor: ExpectedAnchor::new(15, 16, "x"),
        semantic_name: "x",
        region: "block-1",
    },
];
const ESCAPED_OUTER_REFERENCES: &[ExpectedReference] = &[ExpectedReference {
    containing_binding: ExpectedAnchor::new(15, 16, "x"),
    reference: ExpectedAnchor::new(17, 23, "\\u0061"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    region: "block-1",
    target: ExpectedTarget::SameSourceSelectedLexicalBinding {
        binding: ExpectedAnchor::new(4, 5, "a"),
        region: "script",
    },
}];

const CANONICAL_DISTINCT_REGIONS: &[ExpectedRegion] = &[
    ExpectedRegion {
        id: "script",
        parent: None,
        authored_block: None,
    },
    ExpectedRegion {
        id: "block-1",
        parent: Some("script"),
        authored_block: Some(ExpectedAnchor::new(10, 28, "{ let x=e\\u0301; }")),
    },
];
const CANONICAL_DISTINCT_BINDINGS: &[ExpectedBinding] = &[
    ExpectedBinding {
        anchor: ExpectedAnchor::new(4, 6, "é"),
        semantic_name: "é",
        region: "script",
    },
    ExpectedBinding {
        anchor: ExpectedAnchor::new(16, 17, "x"),
        semantic_name: "x",
        region: "block-1",
    },
];
const CANONICAL_DISTINCT_REFERENCES: &[ExpectedReference] = &[ExpectedReference {
    containing_binding: ExpectedAnchor::new(16, 17, "x"),
    reference: ExpectedAnchor::new(18, 25, "e\\u0301"),
    semantic_name: "e\u{301}",
    semantic_code_points: E_COMBINING_ACUTE_CODE_POINTS,
    region: "block-1",
    target: ExpectedTarget::NoSameSourceSelectedLexicalBinding,
}];

const FLAT_REGIONS: &[ExpectedRegion] = &[ExpectedRegion {
    id: "script",
    parent: None,
    authored_block: None,
}];
const FLAT_BINDINGS: &[ExpectedBinding] = &[
    ExpectedBinding {
        anchor: ExpectedAnchor::new(4, 5, "a"),
        semantic_name: "a",
        region: "script",
    },
    ExpectedBinding {
        anchor: ExpectedAnchor::new(13, 14, "x"),
        semantic_name: "x",
        region: "script",
    },
];
const FLAT_REFERENCES: &[ExpectedReference] = &[ExpectedReference {
    containing_binding: ExpectedAnchor::new(13, 14, "x"),
    reference: ExpectedAnchor::new(15, 16, "a"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
    region: "script",
    target: ExpectedTarget::SameSourceSelectedLexicalBinding {
        binding: ExpectedAnchor::new(4, 5, "a"),
        region: "script",
    },
}];

const FIXTURES: &[BlockFixture] = &[
    BlockFixture {
        id: "inner-shadowing",
        source: "let a=1; { let a=2; let x=a; }",
        processing: ExpectedProcessing::SelectedFrontierAccepted,
        regions: INNER_SHADOW_REGIONS,
        bindings: INNER_SHADOW_BINDINGS,
        references: INNER_SHADOW_REFERENCES,
    },
    BlockFixture {
        id: "outer-fallback",
        source: "let a=1; { let x=a; }",
        processing: ExpectedProcessing::SelectedFrontierAccepted,
        regions: OUTER_FALLBACK_REGIONS,
        bindings: OUTER_FALLBACK_BINDINGS,
        references: OUTER_FALLBACK_REFERENCES,
    },
    BlockFixture {
        id: "sibling-regions",
        source: "{ let a=1; } { let x=a; }",
        processing: ExpectedProcessing::SelectedFrontierAccepted,
        regions: SIBLING_REGIONS,
        bindings: SIBLING_BINDINGS,
        references: SIBLING_REFERENCES,
    },
    BlockFixture {
        id: "inner-duplicate-ee14-r01",
        source: "{ let a=1; let a=2; }",
        processing: ExpectedProcessing::StaticRejected {
            authority: EE14_R01,
        },
        regions: INNER_DUP_REGIONS,
        bindings: INNER_DUP_BINDINGS,
        references: &[],
    },
    BlockFixture {
        id: "legal-outer-inner-shadow",
        source: "let a=1; { let a=2; }",
        processing: ExpectedProcessing::SelectedFrontierAccepted,
        regions: LEGAL_SHADOW_REGIONS,
        bindings: LEGAL_SHADOW_BINDINGS,
        references: &[],
    },
    BlockFixture {
        id: "no-same-source-target",
        source: "{ let x=y; }",
        processing: ExpectedProcessing::SelectedFrontierAccepted,
        regions: NO_MATCH_REGIONS,
        bindings: NO_MATCH_BINDINGS,
        references: NO_MATCH_REFERENCES,
    },
    BlockFixture {
        id: "escaped-outer-fallback",
        source: "let a=1; { let x=\\u0061; }",
        processing: ExpectedProcessing::SelectedFrontierAccepted,
        regions: ESCAPED_OUTER_REGIONS,
        bindings: ESCAPED_OUTER_BINDINGS,
        references: ESCAPED_OUTER_REFERENCES,
    },
    BlockFixture {
        id: "canonical-distinct",
        source: "let é=1; { let x=e\\u0301; }",
        processing: ExpectedProcessing::SelectedFrontierAccepted,
        regions: CANONICAL_DISTINCT_REGIONS,
        bindings: CANONICAL_DISTINCT_BINDINGS,
        references: CANONICAL_DISTINCT_REFERENCES,
    },
    BlockFixture {
        id: "recursive-block-unsupported",
        source: "{ let a=1; { let x=a; } }",
        processing: ExpectedProcessing::UnsupportedCoverage,
        regions: &[],
        bindings: &[],
        references: &[],
    },
    BlockFixture {
        id: "asi-before-brace-unsupported",
        source: "{ let a=1 }",
        processing: ExpectedProcessing::UnsupportedCoverage,
        regions: &[],
        bindings: &[],
        references: &[],
    },
    BlockFixture {
        id: "historical-flat",
        source: "let a=1; let x=a;",
        processing: ExpectedProcessing::SelectedFrontierAccepted,
        regions: FLAT_REGIONS,
        bindings: FLAT_BINDINGS,
        references: FLAT_REFERENCES,
    },
];

fn fixture(id: &str) -> &'static BlockFixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing selected Block validation fixture {id}"))
}

fn validate_anchor(source: &SourceText, expected: ExpectedAnchor) {
    let anchor = source
        .anchor(expected.start, expected.end)
        .unwrap_or_else(|error| panic!("invalid fixture anchor {expected:?}: {error}"));
    assert_eq!(anchor.fragment(), expected.fragment);
}

fn decoded_expected_name(code_points: &[u32]) -> String {
    let mut decoded = String::new();
    for code_point in code_points {
        let scalar = char::from_u32(*code_point)
            .unwrap_or_else(|| panic!("fixture contains non-scalar code point U+{code_point:04X}"));
        decoded.push(scalar);
    }
    decoded
}

#[test]
fn one_level_selected_block_frontier_is_pinned_independently() {
    assert!(SELECTED_GRAMMAR_FRONTIER.contains("one-level SelectedBlock"));
    assert!(SELECTED_GRAMMAR_FRONTIER.contains("LexicalDeclaration+"));
    assert!(EE14_R02_NON_TRIGGER.contains("no VarDeclaredNames contributor"));
    assert!(ANNEX_B_BLOCK_FUNCTION_NON_TRIGGER.contains("no FunctionDeclaration"));
    assert!(HISTORICAL_COMPLETION_BOUNDARY.contains("historical flat #268"));
}

#[test]
fn frozen_inventory_has_exactly_the_two_block_rule_units() {
    let block_rules: Vec<_> = RULE_UNITS
        .iter()
        .filter(|rule| rule.container_id == "EE-14")
        .collect();

    assert_eq!(block_rules.len(), 2);
    assert_eq!(block_rules[0].id, EE14_R01);
    assert_eq!(block_rules[0].kind, RuleUnitKind::NormativeRule);
    assert!(block_rules[0].normative_locator.contains("duplicate LexicallyDeclaredNames"));
    assert_eq!(block_rules[1].id, EE14_R02);
    assert_eq!(block_rules[1].kind, RuleUnitKind::NormativeRule);
    assert!(block_rules[1].normative_locator.contains("VarDeclaredNames"));
}

#[test]
fn historical_flat_completion_and_frozen_inventory_remain_separate_authority() {
    assert!(HISTORICAL_FLAT_COMPLETION_SOURCE.contains(concat!(
        "const SELECTED_TOP_LEVEL_",
        "GRAMMAR: &str = \"LexicalDeclaration+\";"
    )));
    assert!(HISTORICAL_FLAT_COMPLETION_SOURCE.contains("(\"EE-14\", \"Block\")"));
    assert!(FROZEN_INVENTORY_SOURCE.contains("\"EE-14-R01\""));
    assert!(FROZEN_INVENTORY_SOURCE.contains("\"EE-14-R02\""));
    assert_eq!(HISTORICAL_FLAT_GRAMMAR, "LexicalDeclaration+");
}

#[test]
fn fixture_ids_sources_regions_and_authored_ranges_are_stable() {
    let mut ids = BTreeSet::new();
    let mut sources = BTreeSet::new();

    for (index, fixture) in FIXTURES.iter().enumerate() {
        assert!(ids.insert(fixture.id), "duplicate fixture id {}", fixture.id);
        assert!(
            sources.insert(fixture.source),
            "duplicate fixture source {}",
            fixture.source
        );

        let source = SourceText::new(SourceId::new(index as u64 + 1), fixture.source.to_owned());
        let mut region_ids = BTreeSet::new();

        for region in fixture.regions {
            assert!(
                region_ids.insert(region.id),
                "duplicate region id {} in {}",
                region.id,
                fixture.id
            );
            if let Some(block) = region.authored_block {
                validate_anchor(&source, block);
            }
        }

        for region in fixture.regions {
            if let Some(parent) = region.parent {
                assert!(
                    region_ids.contains(parent),
                    "missing parent region {parent} in {}",
                    fixture.id
                );
            }
        }

        for binding in fixture.bindings {
            validate_anchor(&source, binding.anchor);
            assert!(region_ids.contains(binding.region));
        }

        for reference in fixture.references {
            validate_anchor(&source, reference.containing_binding);
            validate_anchor(&source, reference.reference);
            assert!(region_ids.contains(reference.region));
            assert_eq!(
                decoded_expected_name(reference.semantic_code_points),
                reference.semantic_name
            );

            if let ExpectedTarget::SameSourceSelectedLexicalBinding { binding, region } =
                reference.target
            {
                validate_anchor(&source, binding);
                assert!(region_ids.contains(region));
            }
        }
    }

    assert_eq!(ids.len(), 11);
    assert_eq!(sources.len(), 11);
}

#[test]
fn inner_shadowing_and_outer_fallback_have_distinct_nearest_targets() {
    let inner = fixture("inner-shadowing").references[0];
    assert_eq!(inner.region, "block-1");
    assert_eq!(
        inner.target,
        ExpectedTarget::SameSourceSelectedLexicalBinding {
            binding: ExpectedAnchor::new(15, 16, "a"),
            region: "block-1",
        }
    );

    let outer = fixture("outer-fallback").references[0];
    assert_eq!(outer.region, "block-1");
    assert_eq!(
        outer.target,
        ExpectedTarget::SameSourceSelectedLexicalBinding {
            binding: ExpectedAnchor::new(4, 5, "a"),
            region: "script",
        }
    );
}

#[test]
fn sibling_blocks_are_distinct_even_at_equal_lexical_depth() {
    let fixture = fixture("sibling-regions");
    assert_eq!(fixture.regions[1].parent, Some("script"));
    assert_eq!(fixture.regions[2].parent, Some("script"));
    assert_ne!(fixture.regions[1].id, fixture.regions[2].id);
    assert_ne!(
        fixture.regions[1].authored_block,
        fixture.regions[2].authored_block
    );
    assert!(matches!(
        fixture.references[0].target,
        ExpectedTarget::NoSameSourceSelectedLexicalBinding
    ));
}

#[test]
fn block_duplicate_and_legal_shadowing_have_different_static_meaning() {
    let duplicate = fixture("inner-duplicate-ee14-r01");
    assert_eq!(
        duplicate.processing,
        ExpectedProcessing::StaticRejected {
            authority: EE14_R01
        }
    );
    assert_eq!(duplicate.bindings[0].region, "block-1");
    assert_eq!(duplicate.bindings[1].region, "block-1");

    let legal = fixture("legal-outer-inner-shadow");
    assert_eq!(legal.processing, ExpectedProcessing::SelectedFrontierAccepted);
    assert_eq!(legal.bindings[0].semantic_name, "a");
    assert_eq!(legal.bindings[1].semantic_name, "a");
    assert_ne!(legal.bindings[0].region, legal.bindings[1].region);
}

#[test]
fn no_match_escaped_equality_and_non_normalization_remain_explicit() {
    let no_match = fixture("no-same-source-target").references[0];
    assert!(matches!(
        no_match.target,
        ExpectedTarget::NoSameSourceSelectedLexicalBinding
    ));
    assert!(NO_MATCH_BOUNDARY.contains("runtime unbound"));
    assert!(NO_MATCH_BOUNDARY.contains("ReferenceError"));

    let escaped = fixture("escaped-outer-fallback").references[0];
    assert_eq!(escaped.reference, ExpectedAnchor::new(17, 23, "\\u0061"));
    assert_eq!(escaped.semantic_name, "a");
    assert_eq!(
        escaped.target,
        ExpectedTarget::SameSourceSelectedLexicalBinding {
            binding: ExpectedAnchor::new(4, 5, "a"),
            region: "script",
        }
    );

    let canonical = fixture("canonical-distinct").references[0];
    assert_eq!(canonical.reference, ExpectedAnchor::new(18, 25, "e\\u0301"));
    assert_eq!(canonical.semantic_name, "e\u{301}");
    assert!(matches!(
        canonical.target,
        ExpectedTarget::NoSameSourceSelectedLexicalBinding
    ));
}

#[test]
fn recursive_block_and_asi_before_brace_are_frontier_unsupported() {
    for id in [
        "recursive-block-unsupported",
        "asi-before-brace-unsupported",
    ] {
        let fixture = fixture(id);
        assert_eq!(fixture.processing, ExpectedProcessing::UnsupportedCoverage);
        assert!(fixture.regions.is_empty());
        assert!(fixture.bindings.is_empty());
        assert!(fixture.references.is_empty());
    }
}

#[test]
fn historical_flat_fixture_has_no_synthetic_block_region() {
    let flat = fixture("historical-flat");
    assert_eq!(flat.processing, ExpectedProcessing::SelectedFrontierAccepted);
    assert_eq!(flat.regions.len(), 1);
    assert_eq!(flat.regions[0].id, "script");
    assert!(flat.regions[0].authored_block.is_none());
    assert_eq!(
        flat.references[0].target,
        ExpectedTarget::SameSourceSelectedLexicalBinding {
            binding: ExpectedAnchor::new(4, 5, "a"),
            region: "script",
        }
    );
}

#[test]
fn oracle_is_candidate_independent_from_future_block_and_scope_production() {
    assert!(!THIS_SOURCE.contains(concat!("selected_", "lexical_slice::")));
    assert!(!THIS_SOURCE.contains(concat!("selected_", "static_semantics::")));
    assert!(!THIS_SOURCE.contains(concat!("selected_", "binding_scope::")));
    assert!(!THIS_SOURCE.contains(concat!("parse_selected_", "block")));
    assert!(!THIS_SOURCE.contains(concat!("analyze_hierarchical_", "binding_scope")));
}
