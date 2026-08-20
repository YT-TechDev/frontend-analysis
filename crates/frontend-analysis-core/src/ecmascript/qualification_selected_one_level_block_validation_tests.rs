//! Candidate-independent validation for the first one-level selected Block
//! qualification and lexical-region frontier accepted by Issue #283.
//!
//! This oracle owns expected source ranges, lexical-region structure, Block
//! Early Error applicability, and unsupported-frontier boundaries independently
//! from any future production Block recognition or hierarchical Binding / Scope
//! implementation.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

const THIS_SOURCE: &str =
    include_str!("qualification_selected_one_level_block_validation_tests.rs");
const HISTORICAL_FLAT_COMPLETION_SOURCE: &str =
    include_str!("qualification_validation_tests/selected_slice_completion.rs");

const TOP_LEVEL_REGION: &str = "top";
const SELECTED_BLOCK_BODY: &str = "LexicalDeclaration+";
const SELECTED_RECURSIVE_BLOCKS: bool = false;
const SELECTED_EMPTY_BLOCK: bool = false;
const SELECTED_VAR_DECLARATION: bool = false;
const SELECTED_FUNCTION_DECLARATION: bool = false;
const SELECTED_EXPRESSION_STATEMENT: bool = false;
const SELECTED_COMMENT_TRIVIA: bool = false;
const SELECTED_NON_EOF_ASI_BEFORE_CLOSE: bool = false;
const SELECTED_CAN_FORM_DIRECTIVE_PROLOGUE: bool = false;
const SELECTED_IS_STRICT: bool = false;
const SELECTED_INCLUDES_NORMATIVE_OPTIONAL_BLOCK_FUNCTIONS: bool = false;

const REGION_BOUNDARY: &str = "lexical-region source structure != hierarchical Binding / Scope target selection != runtime ResolveBinding != runtime Environment Record != TDZ state";
const UNSUPPORTED_BOUNDARY: &str =
    "UnsupportedCoverage in the selected Block frontier != invalid ECMAScript";
const HISTORICAL_COMPLETION_BOUNDARY: &str =
    "historical flat selected-slice completion != Block-enabled selected-slice completion";

const _: () = {
    assert!(!SELECTED_RECURSIVE_BLOCKS);
    assert!(!SELECTED_EMPTY_BLOCK);
    assert!(!SELECTED_VAR_DECLARATION);
    assert!(!SELECTED_FUNCTION_DECLARATION);
    assert!(!SELECTED_EXPRESSION_STATEMENT);
    assert!(!SELECTED_COMMENT_TRIVIA);
    assert!(!SELECTED_NON_EOF_ASI_BEFORE_CLOSE);
    assert!(!SELECTED_CAN_FORM_DIRECTIVE_PROLOGUE);
    assert!(!SELECTED_IS_STRICT);
    assert!(!SELECTED_INCLUDES_NORMATIVE_OPTIONAL_BLOCK_FUNCTIONS);
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
enum UnsupportedReason {
    NestedBlock,
    NonEofAsiBeforeClose,
    EmptyBlock,
    CommentTrivia,
    VarDeclaration,
    FunctionDeclaration,
    ExpressionStatement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedProcessing {
    Complete,
    StaticRejected {
        authority: &'static str,
        first_binding: ExpectedAnchor,
        duplicate_binding: ExpectedAnchor,
    },
    UnsupportedCoverage(UnsupportedReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedBlockRegion {
    id: &'static str,
    block: ExpectedAnchor,
    open_brace: ExpectedAnchor,
    close_brace: ExpectedAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedDeclaration {
    region: &'static str,
    declaration: ExpectedAnchor,
    bindings: &'static [ExpectedAnchor],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedSemanticObservation {
    authored: ExpectedAnchor,
    semantic_name: &'static str,
    semantic_code_points: &'static [u32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockFixture {
    id: &'static str,
    source: &'static str,
    processing: ExpectedProcessing,
    blocks: &'static [ExpectedBlockRegion],
    declarations: &'static [ExpectedDeclaration],
    semantic_observations: &'static [ExpectedSemanticObservation],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockRuleDisposition {
    ReachableRejecting { fixture_id: &'static str },
    StructurallyNonTriggering { reason: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedBlockRule {
    rule_id: &'static str,
    disposition: BlockRuleDisposition,
}

const BLOCK_RULES: &[ExpectedBlockRule] = &[
    ExpectedBlockRule {
        rule_id: "EE-14-R01",
        disposition: BlockRuleDisposition::ReachableRejecting {
            fixture_id: "block-local-duplicate",
        },
    },
    ExpectedBlockRule {
        rule_id: "EE-14-R02",
        disposition: BlockRuleDisposition::StructurallyNonTriggering {
            reason: "selected Block contains no VarDeclaredNames contributor",
        },
    },
];

const A_CODE_POINTS: &[u32] = &[0x61];
const Y_CODE_POINTS: &[u32] = &[0x79];
const E_COMBINING_ACUTE_CODE_POINTS: &[u32] = &[0x65, 0x0301];

const SHADOW_BLOCKS: &[ExpectedBlockRegion] = &[ExpectedBlockRegion {
    id: "block-0",
    block: ExpectedAnchor::new(9, 30, "{ let a=2; let x=a; }"),
    open_brace: ExpectedAnchor::new(9, 10, "{"),
    close_brace: ExpectedAnchor::new(29, 30, "}"),
}];
const SHADOW_TOP_A_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 5, "a")];
const SHADOW_INNER_A_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(15, 16, "a")];
const SHADOW_INNER_X_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(24, 25, "x")];
const SHADOW_TOP_Y_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(35, 36, "y")];
const SHADOW_DECLARATIONS: &[ExpectedDeclaration] = &[
    ExpectedDeclaration {
        region: TOP_LEVEL_REGION,
        declaration: ExpectedAnchor::new(0, 8, "let a=1;"),
        bindings: SHADOW_TOP_A_BINDINGS,
    },
    ExpectedDeclaration {
        region: "block-0",
        declaration: ExpectedAnchor::new(11, 19, "let a=2;"),
        bindings: SHADOW_INNER_A_BINDINGS,
    },
    ExpectedDeclaration {
        region: "block-0",
        declaration: ExpectedAnchor::new(20, 28, "let x=a;"),
        bindings: SHADOW_INNER_X_BINDINGS,
    },
    ExpectedDeclaration {
        region: TOP_LEVEL_REGION,
        declaration: ExpectedAnchor::new(31, 39, "let y=a;"),
        bindings: SHADOW_TOP_Y_BINDINGS,
    },
];

const OUTER_FALLBACK_BLOCKS: &[ExpectedBlockRegion] = &[ExpectedBlockRegion {
    id: "block-0",
    block: ExpectedAnchor::new(9, 21, "{ let x=a; }"),
    open_brace: ExpectedAnchor::new(9, 10, "{"),
    close_brace: ExpectedAnchor::new(20, 21, "}"),
}];
const OUTER_FALLBACK_TOP_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 5, "a")];
const OUTER_FALLBACK_INNER_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(15, 16, "x")];
const OUTER_FALLBACK_DECLARATIONS: &[ExpectedDeclaration] = &[
    ExpectedDeclaration {
        region: TOP_LEVEL_REGION,
        declaration: ExpectedAnchor::new(0, 8, "let a=1;"),
        bindings: OUTER_FALLBACK_TOP_BINDINGS,
    },
    ExpectedDeclaration {
        region: "block-0",
        declaration: ExpectedAnchor::new(11, 19, "let x=a;"),
        bindings: OUTER_FALLBACK_INNER_BINDINGS,
    },
];

const SIBLING_BLOCKS: &[ExpectedBlockRegion] = &[
    ExpectedBlockRegion {
        id: "block-0",
        block: ExpectedAnchor::new(0, 12, "{ let a=1; }"),
        open_brace: ExpectedAnchor::new(0, 1, "{"),
        close_brace: ExpectedAnchor::new(11, 12, "}"),
    },
    ExpectedBlockRegion {
        id: "block-1",
        block: ExpectedAnchor::new(13, 25, "{ let x=a; }"),
        open_brace: ExpectedAnchor::new(13, 14, "{"),
        close_brace: ExpectedAnchor::new(24, 25, "}"),
    },
];
const SIBLING_A_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(6, 7, "a")];
const SIBLING_X_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(19, 20, "x")];
const SIBLING_DECLARATIONS: &[ExpectedDeclaration] = &[
    ExpectedDeclaration {
        region: "block-0",
        declaration: ExpectedAnchor::new(2, 10, "let a=1;"),
        bindings: SIBLING_A_BINDINGS,
    },
    ExpectedDeclaration {
        region: "block-1",
        declaration: ExpectedAnchor::new(15, 23, "let x=a;"),
        bindings: SIBLING_X_BINDINGS,
    },
];

const DUPLICATE_BLOCKS: &[ExpectedBlockRegion] = &[ExpectedBlockRegion {
    id: "block-0",
    block: ExpectedAnchor::new(0, 21, "{ let a=1; let a=2; }"),
    open_brace: ExpectedAnchor::new(0, 1, "{"),
    close_brace: ExpectedAnchor::new(20, 21, "}"),
}];
const DUPLICATE_FIRST_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(6, 7, "a")];
const DUPLICATE_SECOND_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(15, 16, "a")];
const DUPLICATE_DECLARATIONS: &[ExpectedDeclaration] = &[
    ExpectedDeclaration {
        region: "block-0",
        declaration: ExpectedAnchor::new(2, 10, "let a=1;"),
        bindings: DUPLICATE_FIRST_BINDINGS,
    },
    ExpectedDeclaration {
        region: "block-0",
        declaration: ExpectedAnchor::new(11, 19, "let a=2;"),
        bindings: DUPLICATE_SECOND_BINDINGS,
    },
];

const LEGAL_SHADOW_BLOCKS: &[ExpectedBlockRegion] = &[ExpectedBlockRegion {
    id: "block-0",
    block: ExpectedAnchor::new(9, 21, "{ let a=2; }"),
    open_brace: ExpectedAnchor::new(9, 10, "{"),
    close_brace: ExpectedAnchor::new(20, 21, "}"),
}];
const LEGAL_SHADOW_TOP_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 5, "a")];
const LEGAL_SHADOW_INNER_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(15, 16, "a")];
const LEGAL_SHADOW_DECLARATIONS: &[ExpectedDeclaration] = &[
    ExpectedDeclaration {
        region: TOP_LEVEL_REGION,
        declaration: ExpectedAnchor::new(0, 8, "let a=1;"),
        bindings: LEGAL_SHADOW_TOP_BINDINGS,
    },
    ExpectedDeclaration {
        region: "block-0",
        declaration: ExpectedAnchor::new(11, 19, "let a=2;"),
        bindings: LEGAL_SHADOW_INNER_BINDINGS,
    },
];

const NO_MATCH_BLOCKS: &[ExpectedBlockRegion] = &[ExpectedBlockRegion {
    id: "block-0",
    block: ExpectedAnchor::new(0, 12, "{ let x=y; }"),
    open_brace: ExpectedAnchor::new(0, 1, "{"),
    close_brace: ExpectedAnchor::new(11, 12, "}"),
}];
const NO_MATCH_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(6, 7, "x")];
const NO_MATCH_DECLARATIONS: &[ExpectedDeclaration] = &[ExpectedDeclaration {
    region: "block-0",
    declaration: ExpectedAnchor::new(2, 10, "let x=y;"),
    bindings: NO_MATCH_BINDINGS,
}];
const NO_MATCH_SEMANTIC: &[ExpectedSemanticObservation] = &[ExpectedSemanticObservation {
    authored: ExpectedAnchor::new(8, 9, "y"),
    semantic_name: "y",
    semantic_code_points: Y_CODE_POINTS,
}];

const ESCAPED_BLOCKS: &[ExpectedBlockRegion] = &[ExpectedBlockRegion {
    id: "block-0",
    block: ExpectedAnchor::new(9, 26, "{ let x=\\u0061; }"),
    open_brace: ExpectedAnchor::new(9, 10, "{"),
    close_brace: ExpectedAnchor::new(25, 26, "}"),
}];
const ESCAPED_TOP_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 5, "a")];
const ESCAPED_INNER_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(15, 16, "x")];
const ESCAPED_DECLARATIONS: &[ExpectedDeclaration] = &[
    ExpectedDeclaration {
        region: TOP_LEVEL_REGION,
        declaration: ExpectedAnchor::new(0, 8, "let a=1;"),
        bindings: ESCAPED_TOP_BINDINGS,
    },
    ExpectedDeclaration {
        region: "block-0",
        declaration: ExpectedAnchor::new(11, 24, "let x=\\u0061;"),
        bindings: ESCAPED_INNER_BINDINGS,
    },
];
const ESCAPED_SEMANTIC: &[ExpectedSemanticObservation] = &[ExpectedSemanticObservation {
    authored: ExpectedAnchor::new(17, 23, "\\u0061"),
    semantic_name: "a",
    semantic_code_points: A_CODE_POINTS,
}];

const CANONICAL_BLOCKS: &[ExpectedBlockRegion] = &[ExpectedBlockRegion {
    id: "block-0",
    block: ExpectedAnchor::new(10, 28, "{ let x=e\\u0301; }"),
    open_brace: ExpectedAnchor::new(10, 11, "{"),
    close_brace: ExpectedAnchor::new(27, 28, "}"),
}];
const CANONICAL_TOP_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(4, 6, "é")];
const CANONICAL_INNER_BINDINGS: &[ExpectedAnchor] = &[ExpectedAnchor::new(16, 17, "x")];
const CANONICAL_DECLARATIONS: &[ExpectedDeclaration] = &[
    ExpectedDeclaration {
        region: TOP_LEVEL_REGION,
        declaration: ExpectedAnchor::new(0, 9, "let é=1;"),
        bindings: CANONICAL_TOP_BINDINGS,
    },
    ExpectedDeclaration {
        region: "block-0",
        declaration: ExpectedAnchor::new(12, 26, "let x=e\\u0301;"),
        bindings: CANONICAL_INNER_BINDINGS,
    },
];
const CANONICAL_SEMANTIC: &[ExpectedSemanticObservation] = &[ExpectedSemanticObservation {
    authored: ExpectedAnchor::new(18, 25, "e\\u0301"),
    semantic_name: "e\u{301}",
    semantic_code_points: E_COMBINING_ACUTE_CODE_POINTS,
}];

const FIXTURES: &[BlockFixture] = &[
    BlockFixture {
        id: "mixed-shadowing",
        source: "let a=1; { let a=2; let x=a; } let y=a;",
        processing: ExpectedProcessing::Complete,
        blocks: SHADOW_BLOCKS,
        declarations: SHADOW_DECLARATIONS,
        semantic_observations: &[],
    },
    BlockFixture {
        id: "outer-fallback-shape",
        source: "let a=1; { let x=a; }",
        processing: ExpectedProcessing::Complete,
        blocks: OUTER_FALLBACK_BLOCKS,
        declarations: OUTER_FALLBACK_DECLARATIONS,
        semantic_observations: &[],
    },
    BlockFixture {
        id: "sibling-regions",
        source: "{ let a=1; } { let x=a; }",
        processing: ExpectedProcessing::Complete,
        blocks: SIBLING_BLOCKS,
        declarations: SIBLING_DECLARATIONS,
        semantic_observations: &[],
    },
    BlockFixture {
        id: "block-local-duplicate",
        source: "{ let a=1; let a=2; }",
        processing: ExpectedProcessing::StaticRejected {
            authority: "EE-14-R01",
            first_binding: ExpectedAnchor::new(6, 7, "a"),
            duplicate_binding: ExpectedAnchor::new(15, 16, "a"),
        },
        blocks: DUPLICATE_BLOCKS,
        declarations: DUPLICATE_DECLARATIONS,
        semantic_observations: &[],
    },
    BlockFixture {
        id: "legal-outer-inner-shadowing",
        source: "let a=1; { let a=2; }",
        processing: ExpectedProcessing::Complete,
        blocks: LEGAL_SHADOW_BLOCKS,
        declarations: LEGAL_SHADOW_DECLARATIONS,
        semantic_observations: &[],
    },
    BlockFixture {
        id: "no-match-shape",
        source: "{ let x=y; }",
        processing: ExpectedProcessing::Complete,
        blocks: NO_MATCH_BLOCKS,
        declarations: NO_MATCH_DECLARATIONS,
        semantic_observations: NO_MATCH_SEMANTIC,
    },
    BlockFixture {
        id: "escaped-spelling",
        source: "let a=1; { let x=\\u0061; }",
        processing: ExpectedProcessing::Complete,
        blocks: ESCAPED_BLOCKS,
        declarations: ESCAPED_DECLARATIONS,
        semantic_observations: ESCAPED_SEMANTIC,
    },
    BlockFixture {
        id: "canonical-distinct",
        source: "let é=1; { let x=e\\u0301; }",
        processing: ExpectedProcessing::Complete,
        blocks: CANONICAL_BLOCKS,
        declarations: CANONICAL_DECLARATIONS,
        semantic_observations: CANONICAL_SEMANTIC,
    },
    BlockFixture {
        id: "nested-block",
        source: "{ { let a=1; } }",
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::NestedBlock),
        blocks: &[],
        declarations: &[],
        semantic_observations: &[],
    },
    BlockFixture {
        id: "non-eof-asi-before-close",
        source: "{ let a=1 }",
        processing: ExpectedProcessing::UnsupportedCoverage(
            UnsupportedReason::NonEofAsiBeforeClose,
        ),
        blocks: &[],
        declarations: &[],
        semantic_observations: &[],
    },
    BlockFixture {
        id: "empty-block",
        source: "{}",
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::EmptyBlock),
        blocks: &[],
        declarations: &[],
        semantic_observations: &[],
    },
    BlockFixture {
        id: "comment-trivia",
        source: "{ let a=1; /*c*/ let x=a; }",
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::CommentTrivia),
        blocks: &[],
        declarations: &[],
        semantic_observations: &[],
    },
    BlockFixture {
        id: "var-declaration",
        source: "{ var a=1; }",
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::VarDeclaration),
        blocks: &[],
        declarations: &[],
        semantic_observations: &[],
    },
    BlockFixture {
        id: "function-declaration",
        source: "{ function f(){} }",
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::FunctionDeclaration),
        blocks: &[],
        declarations: &[],
        semantic_observations: &[],
    },
    BlockFixture {
        id: "expression-statement",
        source: "{ 1; }",
        processing: ExpectedProcessing::UnsupportedCoverage(UnsupportedReason::ExpressionStatement),
        blocks: &[],
        declarations: &[],
        semantic_observations: &[],
    },
];

fn fixture(id: &str) -> &'static BlockFixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing one-level Block fixture {id}"))
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
            .unwrap_or_else(|| panic!("fixture contains non-scalar U+{code_point:04X}"));
        decoded.push(scalar);
    }
    decoded
}

fn region_exists(fixture: &BlockFixture, region: &str) -> bool {
    region == TOP_LEVEL_REGION || fixture.blocks.iter().any(|block| block.id == region)
}

#[test]
fn one_level_block_frontier_and_qualification_dispositions_are_explicit() {
    assert_eq!(SELECTED_BLOCK_BODY, "LexicalDeclaration+");
    assert_eq!(BLOCK_RULES.len(), 2);
    assert_eq!(BLOCK_RULES[0].rule_id, "EE-14-R01");
    assert!(matches!(
        BLOCK_RULES[0].disposition,
        BlockRuleDisposition::ReachableRejecting {
            fixture_id: "block-local-duplicate"
        }
    ));
    assert_eq!(BLOCK_RULES[1].rule_id, "EE-14-R02");
    assert!(matches!(
        BLOCK_RULES[1].disposition,
        BlockRuleDisposition::StructurallyNonTriggering { .. }
    ));
}

#[test]
fn fixture_ids_sources_and_region_ids_are_stable_and_unique() {
    let mut ids = BTreeSet::new();
    let mut sources = BTreeSet::new();

    for fixture in FIXTURES {
        assert!(
            ids.insert(fixture.id),
            "duplicate fixture id {}",
            fixture.id
        );
        assert!(
            sources.insert(fixture.source),
            "duplicate fixture source {}",
            fixture.source
        );

        let mut region_ids = BTreeSet::new();
        for block in fixture.blocks {
            assert!(
                region_ids.insert(block.id),
                "duplicate region id {} in {}",
                block.id,
                fixture.id
            );
        }
        for declaration in fixture.declarations {
            assert!(
                region_exists(fixture, declaration.region),
                "unknown region {} in {}",
                declaration.region,
                fixture.id
            );
        }
    }

    assert_eq!(ids.len(), FIXTURES.len());
    assert_eq!(sources.len(), FIXTURES.len());
}

#[test]
fn all_expected_ranges_are_valid_utf8_source_anchors() {
    for (index, fixture) in FIXTURES.iter().enumerate() {
        let source = SourceText::new(SourceId::new(index as u64 + 1), fixture.source.to_owned());

        for block in fixture.blocks {
            validate_anchor(&source, block.block);
            validate_anchor(&source, block.open_brace);
            validate_anchor(&source, block.close_brace);
        }
        for declaration in fixture.declarations {
            validate_anchor(&source, declaration.declaration);
            for binding in declaration.bindings {
                validate_anchor(&source, *binding);
            }
        }
        for observation in fixture.semantic_observations {
            validate_anchor(&source, observation.authored);
            assert_eq!(
                decoded_expected_name(observation.semantic_code_points),
                observation.semantic_name
            );
        }
        if let ExpectedProcessing::StaticRejected {
            first_binding,
            duplicate_binding,
            ..
        } = fixture.processing
        {
            validate_anchor(&source, first_binding);
            validate_anchor(&source, duplicate_binding);
        }
    }
}

#[test]
fn mixed_shadowing_fixture_pins_region_structure_without_target_selection() {
    let mixed = fixture("mixed-shadowing");
    assert_eq!(mixed.processing, ExpectedProcessing::Complete);
    assert_eq!(mixed.blocks, SHADOW_BLOCKS);
    assert_eq!(mixed.declarations, SHADOW_DECLARATIONS);
    assert_eq!(
        mixed.blocks[0].block,
        ExpectedAnchor::new(9, 30, "{ let a=2; let x=a; }")
    );
    assert_eq!(mixed.declarations[0].region, TOP_LEVEL_REGION);
    assert_eq!(mixed.declarations[1].region, "block-0");
    assert_eq!(mixed.declarations[2].region, "block-0");
    assert_eq!(mixed.declarations[3].region, TOP_LEVEL_REGION);
    assert!(REGION_BOUNDARY.contains("hierarchical Binding / Scope target selection"));
}

#[test]
fn sibling_blocks_have_distinct_region_identity_even_at_equal_depth() {
    let siblings = fixture("sibling-regions");
    assert_eq!(siblings.processing, ExpectedProcessing::Complete);
    assert_eq!(siblings.blocks.len(), 2);
    assert_ne!(siblings.blocks[0].id, siblings.blocks[1].id);
    assert_eq!(
        siblings.blocks[0].block,
        ExpectedAnchor::new(0, 12, "{ let a=1; }")
    );
    assert_eq!(
        siblings.blocks[1].block,
        ExpectedAnchor::new(13, 25, "{ let x=a; }")
    );
    assert_eq!(siblings.declarations[0].region, "block-0");
    assert_eq!(siblings.declarations[1].region, "block-1");
}

#[test]
fn block_duplicate_and_outer_inner_shadowing_are_distinct_expected_states() {
    let duplicate = fixture("block-local-duplicate");
    assert!(matches!(
        duplicate.processing,
        ExpectedProcessing::StaticRejected {
            authority: "EE-14-R01",
            first_binding: ExpectedAnchor {
                start: 6,
                end: 7,
                ..
            },
            duplicate_binding: ExpectedAnchor {
                start: 15,
                end: 16,
                ..
            },
        }
    ));

    let legal_shadow = fixture("legal-outer-inner-shadowing");
    assert_eq!(legal_shadow.processing, ExpectedProcessing::Complete);
    assert_eq!(legal_shadow.declarations[0].region, TOP_LEVEL_REGION);
    assert_eq!(legal_shadow.declarations[1].region, "block-0");
    assert_eq!(legal_shadow.declarations[0].bindings[0].fragment, "a");
    assert_eq!(legal_shadow.declarations[1].bindings[0].fragment, "a");
}

#[test]
fn outer_fallback_shape_and_no_match_shape_do_not_claim_binding_resolution() {
    let fallback = fixture("outer-fallback-shape");
    assert_eq!(fallback.processing, ExpectedProcessing::Complete);
    assert_eq!(fallback.declarations[0].region, TOP_LEVEL_REGION);
    assert_eq!(fallback.declarations[1].region, "block-0");

    let no_match = fixture("no-match-shape");
    assert_eq!(no_match.processing, ExpectedProcessing::Complete);
    assert_eq!(no_match.semantic_observations[0].semantic_name, "y");
    assert!(REGION_BOUNDARY.contains("runtime ResolveBinding"));
    assert!(REGION_BOUNDARY.contains("TDZ state"));
}

#[test]
fn escaped_spelling_and_canonical_distinction_remain_source_backed() {
    let escaped = fixture("escaped-spelling");
    let escaped_observation = escaped.semantic_observations[0];
    assert_eq!(
        escaped_observation.authored,
        ExpectedAnchor::new(17, 23, "\\u0061")
    );
    assert_eq!(escaped_observation.semantic_name, "a");
    assert_eq!(escaped_observation.semantic_code_points, &[0x61]);

    let canonical = fixture("canonical-distinct");
    let canonical_observation = canonical.semantic_observations[0];
    assert_eq!(
        canonical.declarations[0].bindings[0],
        ExpectedAnchor::new(4, 6, "é")
    );
    assert_eq!(
        canonical_observation.authored,
        ExpectedAnchor::new(18, 25, "e\\u0301")
    );
    assert_eq!(canonical_observation.semantic_name, "e\u{301}");
    assert_eq!(canonical_observation.semantic_code_points, &[0x65, 0x0301]);
    assert_ne!(
        canonical.declarations[0].bindings[0].fragment,
        canonical_observation.semantic_name
    );
}

#[test]
fn unsupported_frontier_cases_remain_coverage_states_not_syntax_verdicts() {
    let expected = [
        ("nested-block", UnsupportedReason::NestedBlock),
        (
            "non-eof-asi-before-close",
            UnsupportedReason::NonEofAsiBeforeClose,
        ),
        ("empty-block", UnsupportedReason::EmptyBlock),
        ("comment-trivia", UnsupportedReason::CommentTrivia),
        ("var-declaration", UnsupportedReason::VarDeclaration),
        (
            "function-declaration",
            UnsupportedReason::FunctionDeclaration,
        ),
        (
            "expression-statement",
            UnsupportedReason::ExpressionStatement,
        ),
    ];

    for (id, reason) in expected {
        assert_eq!(
            fixture(id).processing,
            ExpectedProcessing::UnsupportedCoverage(reason)
        );
    }

    assert!(UNSUPPORTED_BOUNDARY.contains("!= invalid ECMAScript"));
}

#[test]
fn historical_flat_completion_remains_separate_and_candidate_independence_is_visible() {
    assert!(HISTORICAL_FLAT_COMPLETION_SOURCE.contains("SELECTED_TOP_LEVEL_GRAMMAR"));
    assert!(HISTORICAL_FLAT_COMPLETION_SOURCE.contains("LexicalDeclaration+"));
    assert!(HISTORICAL_FLAT_COMPLETION_SOURCE.contains("EE-14"));
    assert!(HISTORICAL_FLAT_COMPLETION_SOURCE.contains("OwningSyntaxAbsent"));
    assert!(HISTORICAL_COMPLETION_BOUNDARY.contains("!= Block-enabled"));

    let sibling_qualifier = concat!("super", "::");
    assert!(
        THIS_SOURCE.lines().all(|line| !line
            .trim_start()
            .starts_with(&format!("use {sibling_qualifier}"))),
        "candidate-independent oracle must not import future sibling production modules"
    );
}
