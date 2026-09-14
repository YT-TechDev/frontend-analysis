//! Candidate-independent successor oracle for the researched one-level
//! Block-contained `var` declarator-cardinality theorem accepted by Issue
//! #688 (research) / frozen for Issue #689 (validation) / widened by Issue
//! #691 / PR #692 (production), and independently frozen here for Issue
//! #693.
//!
//! This oracle freezes exactly:
//!
//! ```text
//! SelectedBlockItem ::=
//!     SelectedLexicalDeclaration
//!   | SelectedBlockVarStatement
//! SelectedBlockVarStatement ::=
//!     var SelectedBindingIdentifier
//!         ( , SelectedBindingIdentifier )*
//!     ;
//! ```
//!
//! with declarator cardinality `1..N`, each declarator a single simple
//! selected `BindingIdentifier`, no initializer, an authored semicolon
//! terminator, one-level top-level `Block` only, no comments, no deeper
//! `Block`, no `for (var ...)`, and no `BindingPattern`. This successor
//! widens only Block `var` contributor cardinality (`1` -> `1..N`); it does
//! not widen placement, initializer grammar, ASI, source-name
//! correspondence, Binding / Scope, or runtime semantics, and it does not
//! change the Early Error identity set frozen by #689.
//!
//! It independently derives Block-local and Script-propagated declared-name
//! effects from literal fixture facts and freezes the resulting Early Error
//! identity, evidence-order precedence, and unsupported-coverage
//! boundaries. It deliberately does not call, import, or derive expected
//! results from any production selected lexical recognizer, Block
//! recognizer, static semantics, qualification integration, Binding /
//! Scope, var-correspondence, or runtime capability. It also does not
//! search, rescan, retokenize, or reparse fixture source to recover
//! evidence: every authored range below is literal fixture authority, only
//! ever verified by slicing the exact stated range.
//!
//! The historical single-declarator oracle
//! (`qualification_selected_one_level_block_bare_var_validation_tests.rs`)
//! remains frozen and unmodified; it continues to describe the narrower
//! pre-#693 checkpoint it froze, and some of its fixtures intentionally
//! describe multi-declarator source as unsupported for that earlier leaf.
//! That is correct historical authority and is not "fixed" here.

#![allow(clippy::assertions_on_constants)]

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

const THIS_SOURCE: &str = include_str!("selected_one_level_block_var_multi_declarator_frontier.rs");

const SELECTED_BLOCK_ITEM_GRAMMAR: &str = "SelectedLexicalDeclaration | SelectedBlockVarStatement";
const SELECTED_BLOCK_VAR_STATEMENT_GRAMMAR: &str =
    "var SelectedBindingIdentifier ( , SelectedBindingIdentifier )* ;";
const SELECTED_LIST_CARDINALITY: &str = "1..N";
const SELECTED_PARSE_GOAL: &str = "Script";
const SELECTED_IS_STRICT: bool = false;

/// Project evidence-ordering tiers, frozen unchanged from #688/#689. This is
/// project diagnostic/validation policy, not an ECMA-262 diagnostic-order
/// claim. Tier 1 (binding-local) and Tier 3 (`EE-36-R01`, Script duplicate
/// lexical) are inherited unchanged from #689 and are not independently
/// re-derived by this successor: this leaf widens only Block `var`
/// contributor cardinality, and no #693 fixture needs to re-prove a
/// precedence relationship #689 already froze for those two tiers.
const EVIDENCE_TIER_ORDER: &[&str] = &["EE-14-R01", "EE-14-R02", "EE-36-R01", "EE-36-R02"];
const EVIDENCE_ORDER_POLICY_NOTE: &str = "Frontend Analysis project evidence-selection policy, not an ECMA-262 mandated diagnostic order";

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

/// A single authored binding fact. `authored` carries the exact authored
/// source range and raw authored spelling. `semantic_name` and
/// `semantic_code_points` are independently stated (never decoded from the
/// authored spelling by this oracle) so that authored-spelling identity and
/// semantic-name identity remain distinguishable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BindingFact {
    authored: ExpectedAnchor,
    semantic_name: &'static str,
    semantic_code_points: &'static [u32],
}

const fn simple_binding(start: usize, end: usize, name: &'static str) -> BindingFact {
    // For a simple (non-escaped) BindingIdentifier the authored spelling and
    // semantic name coincide by construction; the fixture still states both
    // independently rather than deriving one from the other.
    BindingFact {
        authored: ExpectedAnchor::new(start, end, name),
        semantic_name: name,
        semantic_code_points: &[],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockItem {
    Lexical(BindingFact),
    /// One `SelectedBlockVarStatement`'s ordered, duplicate-preserving
    /// `VariableDeclarationList` of 1..N declarators.
    VarStatement(&'static [BindingFact]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockFixture {
    items: &'static [BlockItem],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopLevelItem {
    Lexical(BindingFact),
    /// A single-declarator top-level `var` (cardinality widening at the
    /// top level is #322's concern, not #693's; no fixture in this
    /// successor needs more than one top-level `var` contributor).
    Var(BindingFact),
    Block(BlockFixture),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedReason {
    NonEofAsi,
    Initializer,
    CommentTrivia,
    /// A later `VariableDeclarationList` position that is missing a
    /// definitive `BindingIdentifier` (e.g. a trailing comma before `;`),
    /// where current independently-owned grammar evidence does not yield a
    /// definitive subject.
    IncompleteDeclaratorList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedDisposition {
    /// No Block-local or Script-level lexical/var collision is reachable.
    AcceptedIncomplete,
    /// Block `LexicallyDeclaredNames` contains a duplicate BoundName.
    Ee14R01 { primary: ExpectedAnchor },
    /// Block `LexicallyDeclaredNames` intersects Block `VarDeclaredNames`.
    Ee14R02 { primary: ExpectedAnchor },
    /// Script `LexicallyDeclaredNames` (top-level only) intersects Script
    /// `VarDeclaredNames`, including a propagated Block-var contributor
    /// from any declarator position within a Block `var` statement's list.
    Ee36R02 { primary: ExpectedAnchor },
    /// A definitive selected-grammar rejection, stated literally.
    SyntaxRejected { subject: ExpectedAnchor },
    /// Valid or plausibly valid ECMAScript outside this leaf's supported
    /// grammar. `UnsupportedCoverage` is not evidence of invalid source.
    UnsupportedCoverage(UnsupportedReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fixture {
    id: &'static str,
    source: &'static str,
    top_level_items: &'static [TopLevelItem],
    expected: ExpectedDisposition,
}

const A_CODE_POINTS: &[u32] = &[0x61];
const E_ACUTE_CODE_POINTS: &[u32] = &[0x00e9];
const E_COMBINING_CODE_POINTS: &[u32] = &[0x65, 0x0301];

const fn escaped_a(start: usize, end: usize) -> BindingFact {
    BindingFact {
        authored: ExpectedAnchor::new(start, end, "\\u0061"),
        semantic_name: "a",
        semantic_code_points: A_CODE_POINTS,
    }
}

// --- P1: `{ var x, y; }` (two declarators, accepted) -----------------------
const P1_VAR: &[BindingFact] = &[simple_binding(6, 7, "x"), simple_binding(9, 10, "y")];
const P1_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P1_VAR)],
})];

// --- P2: `{ var x, y, z; }` (three declarators, accepted) ------------------
const P2_VAR: &[BindingFact] = &[
    simple_binding(6, 7, "x"),
    simple_binding(9, 10, "y"),
    simple_binding(12, 13, "z"),
];
const P2_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P2_VAR)],
})];

// --- P3: `{ var x, x; }` (repeated direct name, accepted) ------------------
const P3_VAR: &[BindingFact] = &[simple_binding(6, 7, "x"), simple_binding(9, 10, "x")];
const P3_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P3_VAR)],
})];

// --- P14: `{ var a, a; }` (repeated direct/escaped same name) --------
const P14_VAR: &[BindingFact] = &[simple_binding(6, 7, "a"), escaped_a(9, 15)];
const P14_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P14_VAR)],
})];

// --- P4: `{ let x; var x, y; }` (first-declarator collision) ---------------
const P4_VAR: &[BindingFact] = &[simple_binding(13, 14, "x"), simple_binding(16, 17, "y")];
const P4_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "x")),
    BlockItem::VarStatement(P4_VAR),
];
const P4_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: P4_ITEMS })];

// --- P5: `{ let y; var x, y; }` (second/last-declarator + lexical-before) --
const P5_VAR: &[BindingFact] = &[simple_binding(13, 14, "x"), simple_binding(16, 17, "y")];
const P5_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "y")),
    BlockItem::VarStatement(P5_VAR),
];
const P5_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: P5_ITEMS })];

// --- P6: `{ let y; var x, y, z; }` (interior-declarator collision) ---------
const P6_VAR: &[BindingFact] = &[
    simple_binding(13, 14, "x"),
    simple_binding(16, 17, "y"),
    simple_binding(19, 20, "z"),
];
const P6_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "y")),
    BlockItem::VarStatement(P6_VAR),
];
const P6_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: P6_ITEMS })];

// --- O3: `{ let z; var x, y, z; }` (final-declarator collision) -----------
const O3_VAR: &[BindingFact] = &[
    simple_binding(13, 14, "x"),
    simple_binding(16, 17, "y"),
    simple_binding(19, 20, "z"),
];
const O3_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "z")),
    BlockItem::VarStatement(O3_VAR),
];
const O3_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: O3_ITEMS })];

// --- P7: `{ var x, y; let y; }` (lexical-after-list collision) ------------
const P7_VAR: &[BindingFact] = &[simple_binding(6, 7, "x"), simple_binding(9, 10, "y")];
const P7_ITEMS: &[BlockItem] = &[
    BlockItem::VarStatement(P7_VAR),
    BlockItem::Lexical(simple_binding(16, 17, "y")),
];
const P7_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: P7_ITEMS })];

// --- Offset-discriminating: `{ var x, y, z; let y; }` ----------------------
const ODISC_VAR: &[BindingFact] = &[
    simple_binding(6, 7, "x"),
    simple_binding(9, 10, "y"),
    simple_binding(12, 13, "z"),
];
const ODISC_ITEMS: &[BlockItem] = &[
    BlockItem::VarStatement(ODISC_VAR),
    BlockItem::Lexical(simple_binding(19, 20, "y")),
];
const ODISC_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: ODISC_ITEMS })];

// --- P9: `let y; { var x, y; }` (Script propagation, non-first declarator) -
const P9_VAR: &[BindingFact] = &[simple_binding(13, 14, "x"), simple_binding(16, 17, "y")];
const P9_TOP: &[TopLevelItem] = &[
    TopLevelItem::Lexical(simple_binding(4, 5, "y")),
    TopLevelItem::Block(BlockFixture {
        items: &[BlockItem::VarStatement(P9_VAR)],
    }),
];

// --- P10: `{ var x, y; } let y;` (Script propagation, lexical-after-Block) -
const P10_VAR: &[BindingFact] = &[simple_binding(6, 7, "x"), simple_binding(9, 10, "y")];
const P10_TOP: &[TopLevelItem] = &[
    TopLevelItem::Block(BlockFixture {
        items: &[BlockItem::VarStatement(P10_VAR)],
    }),
    TopLevelItem::Lexical(simple_binding(18, 19, "y")),
];

// --- P11: `var y; { let y; }` (top-level-var / Block-lexical asymmetry) ---
const P11_ITEMS: &[BlockItem] = &[BlockItem::Lexical(simple_binding(13, 14, "y"))];
const P11_TOP: &[TopLevelItem] = &[
    TopLevelItem::Var(simple_binding(4, 5, "y")),
    TopLevelItem::Block(BlockFixture { items: P11_ITEMS }),
];

// --- P13: `{ let a; var a, x; }` (escaped first-declarator collision) -
const P13_VAR: &[BindingFact] = &[escaped_a(13, 19), simple_binding(21, 22, "x")];
const P13_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "a")),
    BlockItem::VarStatement(P13_VAR),
];
const P13_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: P13_ITEMS })];

// --- P12: `{ let a; var x, a; }` (escaped later-declarator collision) -
const P12_VAR: &[BindingFact] = &[simple_binding(13, 14, "x"), escaped_a(16, 22)];
const P12_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "a")),
    BlockItem::VarStatement(P12_VAR),
];
const P12_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: P12_ITEMS })];

// --- P15: `{ var é, é; }` (no Unicode normalization, accepted) ------
const P15_VAR: &[BindingFact] = &[
    BindingFact {
        authored: ExpectedAnchor::new(6, 8, "é"),
        semantic_name: "é",
        semantic_code_points: E_ACUTE_CODE_POINTS,
    },
    BindingFact {
        authored: ExpectedAnchor::new(10, 17, "e\\u0301"),
        semantic_name: "e\u{301}",
        semantic_code_points: E_COMBINING_CODE_POINTS,
    },
];
const P15_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P15_VAR)],
})];

// --- O1: `{ let a; var x, a; }\n{ let b; var y, b; }` (same-tier order) ----
const O1_FIRST_VAR: &[BindingFact] = &[simple_binding(13, 14, "x"), simple_binding(16, 17, "a")];
const O1_FIRST_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "a")),
    BlockItem::VarStatement(O1_FIRST_VAR),
];
const O1_SECOND_VAR: &[BindingFact] = &[simple_binding(34, 35, "y"), simple_binding(37, 38, "b")];
const O1_SECOND_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(27, 28, "b")),
    BlockItem::VarStatement(O1_SECOND_VAR),
];
const O1_TOP: &[TopLevelItem] = &[
    TopLevelItem::Block(BlockFixture {
        items: O1_FIRST_ITEMS,
    }),
    TopLevelItem::Block(BlockFixture {
        items: O1_SECOND_ITEMS,
    }),
];

// --- O2: `{ let a; var x, a; } { let b; let b; }` (cross-tier priority) ----
const O2_FIRST_VAR: &[BindingFact] = &[simple_binding(13, 14, "x"), simple_binding(16, 17, "a")];
const O2_FIRST_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "a")),
    BlockItem::VarStatement(O2_FIRST_VAR),
];
const O2_SECOND_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(27, 28, "b")),
    BlockItem::Lexical(simple_binding(34, 35, "b")),
];
const O2_TOP: &[TopLevelItem] = &[
    TopLevelItem::Block(BlockFixture {
        items: O2_FIRST_ITEMS,
    }),
    TopLevelItem::Block(BlockFixture {
        items: O2_SECOND_ITEMS,
    }),
];

const FIXTURES: &[Fixture] = &[
    Fixture {
        id: "block-two-declarators-accepted",
        source: "{ var x, y; }",
        top_level_items: P1_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-three-declarators-accepted",
        source: "{ var x, y, z; }",
        top_level_items: P2_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-repeated-direct-var-accepted",
        source: "{ var x, x; }",
        top_level_items: P3_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-repeated-direct-escaped-var-accepted",
        source: "{ var a, \\u0061; }",
        top_level_items: P14_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-first-declarator-collision",
        source: "{ let x; var x, y; }",
        top_level_items: P4_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 14, "x"),
        },
    },
    Fixture {
        id: "block-second-declarator-collision",
        source: "{ let y; var x, y; }",
        top_level_items: P5_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(16, 17, "y"),
        },
    },
    Fixture {
        id: "block-interior-declarator-collision",
        source: "{ let y; var x, y, z; }",
        top_level_items: P6_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(16, 17, "y"),
        },
    },
    Fixture {
        id: "block-final-declarator-collision",
        source: "{ let z; var x, y, z; }",
        top_level_items: O3_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(19, 20, "z"),
        },
    },
    Fixture {
        id: "block-lexical-after-list-collision",
        source: "{ var x, y; let y; }",
        top_level_items: P7_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(16, 17, "y"),
        },
    },
    Fixture {
        id: "block-lexical-after-list-offset-discriminating",
        source: "{ var x, y, z; let y; }",
        top_level_items: ODISC_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(19, 20, "y"),
        },
    },
    Fixture {
        id: "script-propagation-non-first-block-declarator",
        source: "let y; { var x, y; }",
        top_level_items: P9_TOP,
        expected: ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(16, 17, "y"),
        },
    },
    Fixture {
        id: "script-propagation-lexical-after-block",
        source: "{ var x, y; } let y;",
        top_level_items: P10_TOP,
        expected: ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(18, 19, "y"),
        },
    },
    Fixture {
        id: "asymmetry-top-level-var-block-lexical-accepted",
        source: "var y; { let y; }",
        top_level_items: P11_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-escaped-first-declarator-collision",
        source: "{ let a; var \\u0061, x; }",
        top_level_items: P13_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 19, "\\u0061"),
        },
    },
    Fixture {
        id: "block-escaped-later-declarator-collision",
        source: "{ let a; var x, \\u0061; }",
        top_level_items: P12_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(16, 22, "\\u0061"),
        },
    },
    Fixture {
        id: "block-no-unicode-normalization-accepted",
        source: "{ var é, e\\u0301; }",
        top_level_items: P15_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "same-tier-sibling-block-source-order",
        source: "{ let a; var x, a; }\n{ let b; var y, b; }",
        top_level_items: O1_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(16, 17, "a"),
        },
    },
    Fixture {
        id: "cross-tier-priority-over-source-position",
        source: "{ let a; var x, a; } { let b; let b; }",
        top_level_items: O2_TOP,
        expected: ExpectedDisposition::Ee14R01 {
            primary: ExpectedAnchor::new(34, 35, "b"),
        },
    },
    Fixture {
        id: "malformed-later-declarator-syntax-rejected",
        source: "{ var x, \\u{}; }",
        top_level_items: &[],
        expected: ExpectedDisposition::SyntaxRejected {
            subject: ExpectedAnchor::new(9, 13, "\\u{}"),
        },
    },
    Fixture {
        id: "incomplete-declarator-list-trailing-comma-two-unsupported",
        source: "{ var x, ; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::IncompleteDeclaratorList,
        ),
    },
    Fixture {
        id: "incomplete-declarator-list-trailing-comma-three-unsupported",
        source: "{ var x, y, ; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::IncompleteDeclaratorList,
        ),
    },
    Fixture {
        id: "initializer-firewall-unsupported",
        source: "{ var x, y = 1; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::Initializer),
    },
    Fixture {
        id: "non-eof-asi-firewall-unsupported",
        source: "{ var x, y }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NonEofAsi),
    },
    Fixture {
        id: "comment-firewall-before-declarator-unsupported",
        source: "{ var x, /* c */ y; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::CommentTrivia),
    },
    Fixture {
        id: "comment-firewall-after-declarator-unsupported",
        source: "{ var x, y /* c */; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::CommentTrivia),
    },
];

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing Block var multi-declarator fixture {id}"))
}

fn validate_anchor(source: &SourceText, expected: ExpectedAnchor) {
    let anchor = source
        .anchor(expected.start, expected.end)
        .unwrap_or_else(|error| panic!("invalid fixture anchor {expected:?}: {error}"));
    assert_eq!(anchor.fragment(), expected.fragment);
}

fn decoded_expected_name(code_points: &[u32]) -> Option<String> {
    if code_points.is_empty() {
        return None;
    }
    let mut decoded = String::new();
    for code_point in code_points {
        let scalar = char::from_u32(*code_point)
            .unwrap_or_else(|| panic!("fixture contains non-scalar U+{code_point:04X}"));
        decoded.push(scalar);
    }
    Some(decoded)
}

/// Independent `LexicallyDeclaredNames` for a Block, derived only from
/// `BlockItem::Lexical` contributors.
fn block_lexically_declared_names(block: &BlockFixture) -> Vec<&'static str> {
    block
        .items
        .iter()
        .filter_map(|item| match item {
            BlockItem::Lexical(fact) => Some(fact.semantic_name),
            BlockItem::VarStatement(_) => None,
        })
        .collect()
}

/// Independent `VarDeclaredNames` for a Block: the ordered,
/// duplicate-preserving concatenation of every `VarStatement`'s 1..N
/// declarators, preserving authored `VariableDeclarationList` order. Every
/// declarator in every position participates; none are dropped.
fn block_var_declared_names(block: &BlockFixture) -> Vec<&'static str> {
    let mut names = Vec::new();
    for item in block.items {
        if let BlockItem::VarStatement(declarators) = item {
            for declarator in *declarators {
                names.push(declarator.semantic_name);
            }
        }
    }
    names
}

/// Independent Script `TopLevelLexicallyDeclaredNames`. A Block's own
/// lexical names are intentionally excluded: they do not propagate to
/// Script lexical-declared-name evidence.
fn script_top_level_lexically_declared_names(items: &[TopLevelItem]) -> Vec<&'static str> {
    items
        .iter()
        .filter_map(|item| match item {
            TopLevelItem::Lexical(fact) => Some(fact.semantic_name),
            TopLevelItem::Var(_) | TopLevelItem::Block(_) => None,
        })
        .collect()
}

/// Independent Script `VarDeclaredNames` / `TopLevelVarDeclaredNames`. Every
/// declarator of every top-level Block `var` statement propagates into this
/// set, not only the first; a Block's lexical names never do.
fn script_var_declared_names(items: &[TopLevelItem]) -> Vec<&'static str> {
    let mut names = Vec::new();
    for item in items {
        match item {
            TopLevelItem::Var(fact) => names.push(fact.semantic_name),
            TopLevelItem::Block(block) => names.extend(block_var_declared_names(block)),
            TopLevelItem::Lexical(_) => {}
        }
    }
    names
}

fn has_duplicate(names: &[&str]) -> bool {
    let unique: BTreeSet<_> = names.iter().collect();
    unique.len() != names.len()
}

fn intersects(a: &[&str], b: &[&str]) -> bool {
    let set_b: BTreeSet<_> = b.iter().collect();
    a.iter().any(|name| set_b.contains(name))
}

fn any_block(items: &[TopLevelItem]) -> impl Iterator<Item = &BlockFixture> {
    items.iter().filter_map(|item| match item {
        TopLevelItem::Block(block) => Some(block),
        TopLevelItem::Lexical(_) | TopLevelItem::Var(_) => None,
    })
}

/// Tier 2a: any Block's `LexicallyDeclaredNames` contains a duplicate
/// BoundName (existing `EE-14-R01`).
fn any_block_has_duplicate_lexical(items: &[TopLevelItem]) -> bool {
    any_block(items).any(|block| has_duplicate(&block_lexically_declared_names(block)))
}

/// Tier 2b: any Block's `LexicallyDeclaredNames` intersects that Block's own
/// `VarDeclaredNames` (`EE-14-R02`, now considering all 1..N declarators).
fn any_block_has_lexical_var_collision(items: &[TopLevelItem]) -> bool {
    any_block(items).any(|block| {
        intersects(
            &block_lexically_declared_names(block),
            &block_var_declared_names(block),
        )
    })
}

/// Tier 4: Script `TopLevelLexicallyDeclaredNames` intersects Script
/// `VarDeclaredNames` (existing `EE-36-R02`), where the var side may include
/// a propagated Block-var contributor from any declarator position.
fn script_has_lexical_var_collision(items: &[TopLevelItem]) -> bool {
    intersects(
        &script_top_level_lexically_declared_names(items),
        &script_var_declared_names(items),
    )
}

/// The index, within `items`, of the first top-level Block (in authored
/// source order) whose own `LexicallyDeclaredNames` intersects its own
/// `VarDeclaredNames`. Project evidence-order policy requires the earliest
/// such Block to supply the primary `EE-14-R02` evidence when more than one
/// Block independently qualifies.
fn first_block_index_with_lexical_var_collision(items: &[TopLevelItem]) -> Option<usize> {
    items.iter().position(|item| match item {
        TopLevelItem::Block(block) => intersects(
            &block_lexically_declared_names(block),
            &block_var_declared_names(block),
        ),
        TopLevelItem::Lexical(_) | TopLevelItem::Var(_) => false,
    })
}

/// Project evidence-order decision across the tiers this oracle
/// independently derives. Tier 1 (binding-local) and Tier 3 (`EE-36-R01`)
/// are inherited unchanged from #689 (see `EVIDENCE_TIER_ORDER`); this
/// successor derives only Tier 2a, Tier 2b, and Tier 4, in project
/// evidence-tier order, since no #693 fixture needs to re-exercise Tier 1
/// or Tier 3 precedence.
fn expected_primary_rule_id(items: &[TopLevelItem]) -> Option<&'static str> {
    if any_block_has_duplicate_lexical(items) {
        return Some("EE-14-R01");
    }
    if any_block_has_lexical_var_collision(items) {
        return Some("EE-14-R02");
    }
    if script_has_lexical_var_collision(items) {
        return Some("EE-36-R02");
    }
    None
}

#[test]
fn selected_block_var_multi_declarator_grammar_and_envelope_are_exact() {
    assert_eq!(
        SELECTED_BLOCK_ITEM_GRAMMAR,
        "SelectedLexicalDeclaration | SelectedBlockVarStatement"
    );
    assert_eq!(
        SELECTED_BLOCK_VAR_STATEMENT_GRAMMAR,
        "var SelectedBindingIdentifier ( , SelectedBindingIdentifier )* ;"
    );
    assert_eq!(SELECTED_LIST_CARDINALITY, "1..N");
    assert_eq!(SELECTED_PARSE_GOAL, "Script");
    assert!(!SELECTED_IS_STRICT);
    assert_eq!(
        EVIDENCE_TIER_ORDER,
        ["EE-14-R01", "EE-14-R02", "EE-36-R01", "EE-36-R02"]
    );
    assert!(!EVIDENCE_ORDER_POLICY_NOTE.is_empty());
    assert!(EVIDENCE_ORDER_POLICY_NOTE.contains("Frontend Analysis project"));
}

#[test]
fn fixture_ids_and_sources_are_stable_and_unique() {
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
    }
    assert_eq!(ids.len(), FIXTURES.len());
    assert_eq!(sources.len(), FIXTURES.len());
    assert_eq!(FIXTURES.len(), 25);
}

#[test]
fn all_expected_anchors_are_valid_utf8_source_anchors_and_slice_to_their_fragment() {
    for (index, fixture) in FIXTURES.iter().enumerate() {
        let source = SourceText::new(SourceId::new(index as u64 + 1), fixture.source.to_owned());

        for item in fixture.top_level_items {
            match item {
                TopLevelItem::Lexical(fact) | TopLevelItem::Var(fact) => {
                    validate_anchor(&source, fact.authored);
                }
                TopLevelItem::Block(block) => {
                    for block_item in block.items {
                        match block_item {
                            BlockItem::Lexical(fact) => validate_anchor(&source, fact.authored),
                            BlockItem::VarStatement(declarators) => {
                                for declarator in *declarators {
                                    validate_anchor(&source, declarator.authored);
                                }
                            }
                        }
                    }
                }
            }
        }

        match fixture.expected {
            ExpectedDisposition::Ee14R01 { primary }
            | ExpectedDisposition::Ee14R02 { primary }
            | ExpectedDisposition::Ee36R02 { primary } => validate_anchor(&source, primary),
            ExpectedDisposition::SyntaxRejected { subject } => validate_anchor(&source, subject),
            ExpectedDisposition::AcceptedIncomplete
            | ExpectedDisposition::UnsupportedCoverage(_) => {}
        }
    }
}

#[test]
fn independent_block_and_script_declared_name_derivation_matches_every_modeled_fixture() {
    for fixture in FIXTURES {
        if matches!(
            fixture.expected,
            ExpectedDisposition::SyntaxRejected { .. }
                | ExpectedDisposition::UnsupportedCoverage(_)
        ) {
            // Grammar and unsupported-coverage fixtures are frozen literal
            // facts, not independently derivable declared-name outcomes.
            continue;
        }

        let derived = expected_primary_rule_id(fixture.top_level_items);
        match fixture.expected {
            ExpectedDisposition::AcceptedIncomplete => {
                assert_eq!(derived, None, "{} must not derive any rule", fixture.id);
            }
            ExpectedDisposition::Ee14R01 { .. } => {
                assert_eq!(
                    derived,
                    Some("EE-14-R01"),
                    "{} must independently derive EE-14-R01",
                    fixture.id
                );
            }
            ExpectedDisposition::Ee14R02 { .. } => {
                assert_eq!(
                    derived,
                    Some("EE-14-R02"),
                    "{} must independently derive EE-14-R02",
                    fixture.id
                );
            }
            ExpectedDisposition::Ee36R02 { .. } => {
                assert_eq!(
                    derived,
                    Some("EE-36-R02"),
                    "{} must independently derive EE-36-R02",
                    fixture.id
                );
            }
            ExpectedDisposition::SyntaxRejected { .. }
            | ExpectedDisposition::UnsupportedCoverage(_) => unreachable!(),
        }
    }
}

#[test]
fn block_ee14_r02_reaches_first_second_interior_and_final_declarator_positions() {
    // Each fixture below independently derives EE-14-R02 from a collision
    // at a *different* declarator position within the VariableDeclarationList,
    // defeating both a "first declarator only" and a "last declarator only"
    // wrong oracle.
    let first = fixture("block-first-declarator-collision");
    let second = fixture("block-second-declarator-collision");
    let interior = fixture("block-interior-declarator-collision");
    let final_position = fixture("block-final-declarator-collision");

    for (fixture, expected_primary) in [
        (first, ExpectedAnchor::new(13, 14, "x")),
        (second, ExpectedAnchor::new(16, 17, "y")),
        (interior, ExpectedAnchor::new(16, 17, "y")),
        (final_position, ExpectedAnchor::new(19, 20, "z")),
    ] {
        assert_eq!(
            expected_primary_rule_id(fixture.top_level_items),
            Some("EE-14-R02"),
            "{} must independently derive EE-14-R02",
            fixture.id
        );
        assert_eq!(
            fixture.expected,
            ExpectedDisposition::Ee14R02 {
                primary: expected_primary
            },
            "{} primary evidence mismatch",
            fixture.id
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct EvidenceTripleFixture {
    fixture_id: &'static str,
    lexical_binding: ExpectedAnchor,
    var_binding: ExpectedAnchor,
}

const EVIDENCE_TRIPLE_FIXTURES: &[EvidenceTripleFixture] = &[
    EvidenceTripleFixture {
        fixture_id: "block-second-declarator-collision",
        lexical_binding: ExpectedAnchor::new(6, 7, "y"),
        var_binding: ExpectedAnchor::new(16, 17, "y"),
    },
    EvidenceTripleFixture {
        fixture_id: "block-lexical-after-list-collision",
        lexical_binding: ExpectedAnchor::new(16, 17, "y"),
        var_binding: ExpectedAnchor::new(9, 10, "y"),
    },
    EvidenceTripleFixture {
        fixture_id: "block-lexical-after-list-offset-discriminating",
        lexical_binding: ExpectedAnchor::new(19, 20, "y"),
        var_binding: ExpectedAnchor::new(9, 10, "y"),
    },
];

#[test]
fn lexical_before_and_after_list_evidence_triples_are_independently_asserted() {
    // Accepted project policy: the primary authored binding is the binding
    // occurrence that *completes* the collision, i.e. whichever of the
    // lexical / var sides occurs later in authored source order. This test
    // asserts the complete (lexical, var, primary) triple, not only rule
    // identity or only the primary range, and includes an
    // offset-discriminating case so a hard-coded numeric offset cannot
    // accidentally pass.
    for triple in EVIDENCE_TRIPLE_FIXTURES {
        let fixture = fixture(triple.fixture_id);
        let ExpectedDisposition::Ee14R02 { primary } = fixture.expected else {
            panic!("{} must be an EE-14-R02 fixture", triple.fixture_id);
        };
        assert_ne!(
            triple.lexical_binding, triple.var_binding,
            "{} lexical and var evidence must be distinct occurrences",
            triple.fixture_id
        );
        let later = if triple.lexical_binding.start > triple.var_binding.start {
            triple.lexical_binding
        } else {
            triple.var_binding
        };
        assert_eq!(
            primary, later,
            "{} primary evidence must be the later-in-source occurrence that completes the collision",
            triple.fixture_id
        );
    }

    // The lexical-before fixture completes on the var side; the two
    // lexical-after fixtures complete on the lexical side, and use
    // different numeric offsets for that lexical side (17 vs 20),
    // preventing a hard-coded primary offset from accidentally passing.
    assert_eq!(
        fixture("block-second-declarator-collision").expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(16, 17, "y")
        }
    );
    assert_eq!(
        fixture("block-lexical-after-list-collision").expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(16, 17, "y")
        }
    );
    assert_eq!(
        fixture("block-lexical-after-list-offset-discriminating").expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(19, 20, "y")
        }
    );
}

#[test]
fn repeated_var_names_and_escaped_semantic_equality_do_not_collapse_authored_multiplicity() {
    let direct = fixture("block-repeated-direct-var-accepted");
    let TopLevelItem::Block(direct_block) = direct.top_level_items[0] else {
        panic!("fixture's only item must be a Block");
    };
    assert_eq!(block_var_declared_names(&direct_block), vec!["x", "x"]);
    assert!(!any_block_has_lexical_var_collision(direct.top_level_items));
    assert_eq!(direct.expected, ExpectedDisposition::AcceptedIncomplete);

    let escaped = fixture("block-repeated-direct-escaped-var-accepted");
    let TopLevelItem::Block(escaped_block) = escaped.top_level_items[0] else {
        panic!("fixture's only item must be a Block");
    };
    let BlockItem::VarStatement(declarators) = escaped_block.items[0] else {
        panic!("fixture's only Block item must be a var statement");
    };
    assert_ne!(
        declarators[0].authored.fragment, declarators[1].authored.fragment,
        "authored spellings must differ (\"a\" vs escaped \\u0061)"
    );
    assert_eq!(declarators[0].semantic_name, declarators[1].semantic_name);
    assert_eq!(
        decoded_expected_name(declarators[1].semantic_code_points),
        Some(declarators[1].semantic_name.to_owned())
    );
    assert_eq!(block_var_declared_names(&escaped_block), vec!["a", "a"]);
    assert!(!any_block_has_lexical_var_collision(
        escaped.top_level_items
    ));
    assert_eq!(escaped.expected, ExpectedDisposition::AcceptedIncomplete);
}

#[test]
fn script_ee36_r02_propagation_covers_non_first_block_declarator_and_lexical_after_block() {
    let non_first = fixture("script-propagation-non-first-block-declarator");
    let TopLevelItem::Block(non_first_block) = non_first.top_level_items[1] else {
        panic!("fixture's second item must be a Block");
    };
    // The colliding contributor ("y") is the *second* declarator of the
    // Block's own VariableDeclarationList, not the first ("x"), proving a
    // "first declarator only" propagation model would miss this collision.
    assert_eq!(block_var_declared_names(&non_first_block), vec!["x", "y"]);
    assert_eq!(
        expected_primary_rule_id(non_first.top_level_items),
        Some("EE-36-R02")
    );
    assert_eq!(
        non_first.expected,
        ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(16, 17, "y")
        }
    );

    let lexical_after = fixture("script-propagation-lexical-after-block");
    assert_eq!(
        expected_primary_rule_id(lexical_after.top_level_items),
        Some("EE-36-R02")
    );
    assert_eq!(
        lexical_after.expected,
        ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(18, 19, "y")
        }
    );
}

#[test]
fn asymmetry_between_top_level_var_and_block_lexical_names_remains_explicit() {
    let asymmetry = fixture("asymmetry-top-level-var-block-lexical-accepted");
    assert_eq!(asymmetry.expected, ExpectedDisposition::AcceptedIncomplete);

    let TopLevelItem::Block(inner_block) = asymmetry.top_level_items[1] else {
        panic!("asymmetry fixture's second item must be a Block");
    };
    assert!(block_var_declared_names(&inner_block).is_empty());
    assert!(script_top_level_lexically_declared_names(asymmetry.top_level_items).is_empty());
    assert!(!script_has_lexical_var_collision(asymmetry.top_level_items));
}

#[test]
fn escaped_first_and_later_declarator_collisions_use_semantic_equality_not_spelling() {
    let first = fixture("block-escaped-first-declarator-collision");
    assert_eq!(
        expected_primary_rule_id(first.top_level_items),
        Some("EE-14-R02")
    );
    assert_eq!(
        first.expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 19, "\\u0061")
        }
    );

    let later = fixture("block-escaped-later-declarator-collision");
    assert_eq!(
        expected_primary_rule_id(later.top_level_items),
        Some("EE-14-R02")
    );
    assert_eq!(
        later.expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(16, 22, "\\u0061")
        }
    );
}

#[test]
fn no_unicode_normalization_preserves_distinct_code_point_sequences() {
    let fixture = fixture("block-no-unicode-normalization-accepted");
    let TopLevelItem::Block(block) = fixture.top_level_items[0] else {
        panic!("fixture's only item must be a Block");
    };
    let BlockItem::VarStatement(declarators) = block.items[0] else {
        panic!("fixture's only Block item must be a var statement");
    };
    assert_ne!(
        declarators[0].semantic_code_points,
        declarators[1].semantic_code_points
    );
    assert_ne!(declarators[0].semantic_name, declarators[1].semantic_name);
    assert!(!any_block_has_lexical_var_collision(
        fixture.top_level_items
    ));
    assert_eq!(fixture.expected, ExpectedDisposition::AcceptedIncomplete);
}

#[test]
fn same_tier_sibling_blocks_select_first_qualifying_block_in_source_order() {
    let fixture = fixture("same-tier-sibling-block-source-order");
    let TopLevelItem::Block(first_block) = fixture.top_level_items[0] else {
        panic!("fixture's first item must be a Block");
    };
    let TopLevelItem::Block(second_block) = fixture.top_level_items[1] else {
        panic!("fixture's second item must be a Block");
    };

    // Both Blocks independently qualify: this is not a vacuous ordering
    // proof where only one Block could ever have been chosen. In both
    // Blocks the collision is at the *second* declarator, not the first.
    assert!(intersects(
        &block_lexically_declared_names(&first_block),
        &block_var_declared_names(&first_block)
    ));
    assert!(intersects(
        &block_lexically_declared_names(&second_block),
        &block_var_declared_names(&second_block)
    ));
    assert_eq!(block_var_declared_names(&first_block), vec!["x", "a"]);
    assert_eq!(block_var_declared_names(&second_block), vec!["y", "b"]);

    assert_eq!(
        first_block_index_with_lexical_var_collision(fixture.top_level_items),
        Some(0),
        "project evidence-order policy selects the first qualifying Block in source order"
    );
    assert_eq!(
        fixture.expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(16, 17, "a"),
        },
        "primary evidence must belong to the first Block's own var binding, not the second Block's"
    );
}

#[test]
fn evidence_tier_outranks_cross_block_source_position_for_the_successor_theorem() {
    let fixture = fixture("cross-tier-priority-over-source-position");
    let TopLevelItem::Block(first_block) = fixture.top_level_items[0] else {
        panic!("fixture's first item must be a Block");
    };
    let TopLevelItem::Block(second_block) = fixture.top_level_items[1] else {
        panic!("fixture's second item must be a Block");
    };

    // The earlier Block independently reaches only Tier 2b (EE-14-R02); the
    // later Block independently reaches Tier 2a (EE-14-R01). Neither
    // condition is vacuous.
    assert!(intersects(
        &block_lexically_declared_names(&first_block),
        &block_var_declared_names(&first_block)
    ));
    assert!(!has_duplicate(&block_lexically_declared_names(
        &first_block
    )));
    assert!(has_duplicate(&block_lexically_declared_names(
        &second_block
    )));

    assert!(any_block_has_lexical_var_collision(fixture.top_level_items));
    assert!(any_block_has_duplicate_lexical(fixture.top_level_items));

    // Tier priority is evaluated before Block source position: Tier 2a
    // (from the later Block) must win over Tier 2b (from the earlier
    // Block), never "whichever error's Block comes first in the file".
    assert_eq!(
        expected_primary_rule_id(fixture.top_level_items),
        Some("EE-14-R01"),
        "a later Tier-2a error must outrank an earlier Tier-2b error"
    );
    assert_eq!(
        fixture.expected,
        ExpectedDisposition::Ee14R01 {
            primary: ExpectedAnchor::new(34, 35, "b"),
        },
        "primary evidence must belong to the second Block's duplicate lexical binding"
    );
}

#[derive(Debug, Clone, Copy)]
struct FailedDeclaratorListFixture {
    fixture_id: &'static str,
}

const FAILED_DECLARATOR_LIST_FIXTURES: &[FailedDeclaratorListFixture] = &[
    FailedDeclaratorListFixture {
        fixture_id: "malformed-later-declarator-syntax-rejected",
    },
    FailedDeclaratorListFixture {
        fixture_id: "incomplete-declarator-list-trailing-comma-two-unsupported",
    },
    FailedDeclaratorListFixture {
        fixture_id: "incomplete-declarator-list-trailing-comma-three-unsupported",
    },
    FailedDeclaratorListFixture {
        fixture_id: "initializer-firewall-unsupported",
    },
];

#[test]
fn malformed_and_incomplete_declarator_lists_never_commit_a_partial_prefix() {
    for failed in FAILED_DECLARATOR_LIST_FIXTURES {
        let fixture = fixture(failed.fixture_id);
        // Every failed-list fixture models zero committed top-level items:
        // no valid declarator prefix escapes as committed selected success
        // when a later list element prevents statement completion.
        assert!(
            fixture.top_level_items.is_empty(),
            "{} must commit no partial selected-success state",
            fixture.id
        );
        assert_ne!(
            fixture.expected,
            ExpectedDisposition::AcceptedIncomplete,
            "{} must not be accepted",
            fixture.id
        );
    }
    assert!(
        FAILED_DECLARATOR_LIST_FIXTURES
            .iter()
            .any(|failed| matches!(
                fixture(failed.fixture_id).expected,
                ExpectedDisposition::SyntaxRejected { .. }
            ))
    );
    assert!(
        FAILED_DECLARATOR_LIST_FIXTURES
            .iter()
            .any(|failed| matches!(
                fixture(failed.fixture_id).expected,
                ExpectedDisposition::UnsupportedCoverage(_)
            ))
    );
}

#[test]
fn classification_firewalls_keep_syntax_rejection_and_unsupported_coverage_distinct() {
    assert_eq!(
        fixture("malformed-later-declarator-syntax-rejected").expected,
        ExpectedDisposition::SyntaxRejected {
            subject: ExpectedAnchor::new(9, 13, "\\u{}"),
        }
    );

    let expected_unsupported = [
        (
            "incomplete-declarator-list-trailing-comma-two-unsupported",
            UnsupportedReason::IncompleteDeclaratorList,
        ),
        (
            "incomplete-declarator-list-trailing-comma-three-unsupported",
            UnsupportedReason::IncompleteDeclaratorList,
        ),
        (
            "initializer-firewall-unsupported",
            UnsupportedReason::Initializer,
        ),
        (
            "non-eof-asi-firewall-unsupported",
            UnsupportedReason::NonEofAsi,
        ),
        (
            "comment-firewall-before-declarator-unsupported",
            UnsupportedReason::CommentTrivia,
        ),
        (
            "comment-firewall-after-declarator-unsupported",
            UnsupportedReason::CommentTrivia,
        ),
    ];
    for (id, reason) in expected_unsupported {
        assert_eq!(
            fixture(id).expected,
            ExpectedDisposition::UnsupportedCoverage(reason)
        );
    }

    // `{ var x, y }` is valid ECMAScript through non-EOF ASI; it must remain
    // UnsupportedCoverage in this leaf, never SyntaxRejected -- this leaf
    // does not implement ASI.
    assert_ne!(
        fixture("non-eof-asi-firewall-unsupported").expected,
        ExpectedDisposition::SyntaxRejected {
            subject: ExpectedAnchor::new(0, 0, "")
        }
    );
}

#[test]
fn oracle_source_is_candidate_independent_and_does_not_call_production_owners() {
    for forbidden in [
        concat!("selected_lexical_", "slice::"),
        concat!("selected_static_", "semantics::"),
        concat!("selected_one_level_block_static_", "semantics::"),
        concat!("selected_variable_statement_static_", "semantics::"),
        concat!("selected_qualification_", "integration::"),
        concat!("selected_binding_", "scope::"),
        concat!("selected_one_level_block_binding_", "scope::"),
        concat!("selected_variable_statement_name_", "correspondence::"),
        concat!("recognize_selected_", "lexical_slice("),
        concat!("evaluate_selected_", "static_semantics("),
        concat!("evaluate_selected_one_level_block_", "static_semantics("),
        concat!("evaluate_selected_variable_statement_", "static_semantics("),
        concat!("attempt_selected_", "qualification("),
        concat!("analyze_selected_", "binding_scope("),
        concat!("analyze_selected_one_level_block_", "binding_scope("),
        concat!(
            "analyze_selected_variable_statement_name_",
            "correspondence("
        ),
        concat!("crate::ecma", "script::"),
        concat!("super::selected", "_"),
        concat!("SelectedStaticSemantics", "Outcome"),
        concat!("SelectedOneLevelBlockStaticSemantics", "Outcome"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "Block var multi-declarator oracle must remain candidate-independent: found {forbidden}"
        );
    }
}
