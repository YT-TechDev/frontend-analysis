//! Candidate-independent validation oracle for the researched one-level
//! Block-contained bare `var` candidate accepted by Issue #688 / frozen for
//! Issue #689.
//!
//! This oracle proves the exact #688 theorem:
//!
//! ```text
//! SelectedBlockItem ::=
//!     SelectedLexicalDeclaration
//!   | SelectedBareBlockVar
//!
//! SelectedBareBlockVar ::=
//!     var SelectedBindingIdentifier ;
//! ```
//!
//! It independently derives Block-local and Script-propagated declared-name
//! effects from literal fixture facts and freezes the resulting Early Error
//! identity, evidence-order precedence, and unsupported-coverage boundaries.
//! It deliberately does not call, import, or derive expected results from any
//! production selected lexical recognizer, Block recognizer, static
//! semantics, qualification integration, Binding / Scope, var-correspondence,
//! or runtime capability. It also does not search, rescan, retokenize, or
//! reparse fixture source to recover evidence: every authored range below is
//! literal fixture authority, only ever verified by slicing the exact stated
//! range.

#![allow(clippy::assertions_on_constants)]

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

const THIS_SOURCE: &str =
    include_str!("qualification_selected_one_level_block_bare_var_validation_tests.rs");

const SELECTED_BLOCK_ITEM_GRAMMAR: &str = "SelectedLexicalDeclaration | SelectedBareBlockVar";
const SELECTED_BARE_BLOCK_VAR_GRAMMAR: &str = "var SelectedBindingIdentifier ;";
const SELECTED_PARSE_GOAL: &str = "Script";
const SELECTED_IS_STRICT: bool = false;

/// Project evidence-ordering tiers frozen by #688/#689. This is project
/// diagnostic/validation policy, not an ECMA-262 diagnostic-order claim.
const EVIDENCE_TIER_ORDER: &[&str] = &["EE-14-R01", "EE-14-R02", "EE-36-R01", "EE-36-R02"];

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
/// semantic-name identity remain distinguishable, as required for the
/// escaped-vs-direct-spelling sentinel (authored six-character Unicode
/// escape spelling for lowercase "a", semantic one-character name "a").
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
    BareVar(BindingFact),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockFixture {
    items: &'static [BlockItem],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopLevelItem {
    Lexical(BindingFact),
    Var(BindingFact),
    Block(BlockFixture),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedReason {
    NonEofAsi,
    Initializer,
    MultiDeclarator,
    CommentTrivia,
    DeeperNesting,
    ForVar,
    BindingPattern,
    ModuleGoal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedDisposition {
    /// No Block-local or Script-level lexical/var collision is reachable.
    AcceptedIncomplete,
    /// Block `LexicallyDeclaredNames` contains a duplicate BoundName.
    Ee14R01 { primary: ExpectedAnchor },
    /// Block `LexicallyDeclaredNames` intersects Block `VarDeclaredNames`.
    Ee14R02 { primary: ExpectedAnchor },
    /// Script `LexicallyDeclaredNames` (top-level only) contains a duplicate
    /// BoundName.
    Ee36R01 { primary: ExpectedAnchor },
    /// Script `LexicallyDeclaredNames` intersects Script `VarDeclaredNames`,
    /// including a propagated Block-var contributor.
    Ee36R02 { primary: ExpectedAnchor },
    /// A binding-local Early Error inherited from existing frozen project
    /// authority. This disposition is stated literally; it is not derived
    /// from any Block/Script declared-name intersection in this oracle.
    StaticSemanticsRejected {
        rule_id: &'static str,
        subject: ExpectedAnchor,
    },
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

// --- Fixture: `{ let x; var x; }` -----------------------------------------
const LEXICAL_THEN_VAR_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "x")),
    BlockItem::BareVar(simple_binding(13, 14, "x")),
];
const LEXICAL_THEN_VAR_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: LEXICAL_THEN_VAR_ITEMS,
})];

// --- Fixture: `{ var x; let x; }` -----------------------------------------
const VAR_THEN_LEXICAL_ITEMS: &[BlockItem] = &[
    BlockItem::BareVar(simple_binding(6, 7, "x")),
    BlockItem::Lexical(simple_binding(13, 14, "x")),
];
const VAR_THEN_LEXICAL_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: VAR_THEN_LEXICAL_ITEMS,
})];

// --- Fixture: `{ let x; var y; }` -----------------------------------------
const LEXICAL_VAR_DISTINCT_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "x")),
    BlockItem::BareVar(simple_binding(13, 14, "y")),
];
const LEXICAL_VAR_DISTINCT_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: LEXICAL_VAR_DISTINCT_ITEMS,
})];

// --- Fixture: `{ var x; let y; }` -----------------------------------------
const VAR_LEXICAL_DISTINCT_ITEMS: &[BlockItem] = &[
    BlockItem::BareVar(simple_binding(6, 7, "x")),
    BlockItem::Lexical(simple_binding(13, 14, "y")),
];
const VAR_LEXICAL_DISTINCT_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: VAR_LEXICAL_DISTINCT_ITEMS,
})];

// --- Fixture: `let x; { var x; }` -----------------------------------------
const SCRIPT_LEXICAL_THEN_BLOCK_VAR_ITEMS: &[BlockItem] =
    &[BlockItem::BareVar(simple_binding(13, 14, "x"))];
const SCRIPT_LEXICAL_THEN_BLOCK_VAR_TOP: &[TopLevelItem] = &[
    TopLevelItem::Lexical(simple_binding(4, 5, "x")),
    TopLevelItem::Block(BlockFixture {
        items: SCRIPT_LEXICAL_THEN_BLOCK_VAR_ITEMS,
    }),
];

// --- Fixture: `{ var x; } let x;` -----------------------------------------
const SCRIPT_BLOCK_VAR_THEN_LEXICAL_ITEMS: &[BlockItem] =
    &[BlockItem::BareVar(simple_binding(6, 7, "x"))];
const SCRIPT_BLOCK_VAR_THEN_LEXICAL_TOP: &[TopLevelItem] = &[
    TopLevelItem::Block(BlockFixture {
        items: SCRIPT_BLOCK_VAR_THEN_LEXICAL_ITEMS,
    }),
    TopLevelItem::Lexical(simple_binding(15, 16, "x")),
];

// --- Fixture: `let x; { var y; }` -----------------------------------------
const SCRIPT_ACCEPTED_PROPAGATION_ITEMS: &[BlockItem] =
    &[BlockItem::BareVar(simple_binding(13, 14, "y"))];
const SCRIPT_ACCEPTED_PROPAGATION_TOP: &[TopLevelItem] = &[
    TopLevelItem::Lexical(simple_binding(4, 5, "x")),
    TopLevelItem::Block(BlockFixture {
        items: SCRIPT_ACCEPTED_PROPAGATION_ITEMS,
    }),
];

// --- Fixture: `var x; { let x; }` (asymmetry sentinel) --------------------
const ASYMMETRY_BLOCK_ITEMS: &[BlockItem] = &[BlockItem::Lexical(simple_binding(13, 14, "x"))];
const ASYMMETRY_TOP: &[TopLevelItem] = &[
    TopLevelItem::Var(simple_binding(4, 5, "x")),
    TopLevelItem::Block(BlockFixture {
        items: ASYMMETRY_BLOCK_ITEMS,
    }),
];

// --- Fixture: `{ let a; var a; }` (semantic-name / spelling split) ---
const A_CODE_POINTS: &[u32] = &[0x61];
const SEMANTIC_SPLIT_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(BindingFact {
        authored: ExpectedAnchor::new(6, 7, "a"),
        semantic_name: "a",
        semantic_code_points: A_CODE_POINTS,
    }),
    BlockItem::BareVar(BindingFact {
        authored: ExpectedAnchor::new(13, 19, "\\u0061"),
        semantic_name: "a",
        semantic_code_points: A_CODE_POINTS,
    }),
];
const SEMANTIC_SPLIT_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: SEMANTIC_SPLIT_ITEMS,
})];

// --- Fixture: `let x; { let x; var x; }` (Block R02 before Script R02) ----
const BLOCK_BEFORE_SCRIPT_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(13, 14, "x")),
    BlockItem::BareVar(simple_binding(20, 21, "x")),
];
const BLOCK_BEFORE_SCRIPT_TOP: &[TopLevelItem] = &[
    TopLevelItem::Lexical(simple_binding(4, 5, "x")),
    TopLevelItem::Block(BlockFixture {
        items: BLOCK_BEFORE_SCRIPT_ITEMS,
    }),
];

// --- Fixture: `{ let x; let x; var x; }` (R01 before R02) -----------------
const R01_BEFORE_R02_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "x")),
    BlockItem::Lexical(simple_binding(13, 14, "x")),
    BlockItem::BareVar(simple_binding(20, 21, "x")),
];
const R01_BEFORE_R02_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: R01_BEFORE_R02_ITEMS,
})];

// --- Fixture: `{ var x; var x; }` (repeated bare var control) -------------
const REPEATED_VAR_ITEMS: &[BlockItem] = &[
    BlockItem::BareVar(simple_binding(6, 7, "x")),
    BlockItem::BareVar(simple_binding(13, 14, "x")),
];
const REPEATED_VAR_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: REPEATED_VAR_ITEMS,
})];

// --- Fixture: `{ let x; var x; var 0; }` (Tier 1 outranks Block R02) -
// The third Block item (an escaped BindingIdentifier that decodes to an
// invalid IdentifierStart) is a binding-local Tier-1 fact stated literally
// via `ExpectedDisposition::StaticSemanticsRejected`; it is intentionally
// not modeled as a `BlockItem` because a Tier-1-invalid identifier can never
// also be a valid BoundName that could collide with anything. The two valid
// items below (`let x` / `var x`) still independently derive a true
// `EE-14-R02` collision, proving the fixture's frozen Tier-1 disposition
// outranks that derivable Tier-2b result rather than merely coexisting with
// an absent one.
const TIER1_OUTRANKS_BLOCK_R02_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "x")),
    BlockItem::BareVar(simple_binding(13, 14, "x")),
];
const TIER1_OUTRANKS_BLOCK_R02_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: TIER1_OUTRANKS_BLOCK_R02_ITEMS,
})];

// --- Fixture: `let x; let x; { var x; }` (Tier 3 outranks Tier 4) ---------
const TIER3_OUTRANKS_TIER4_ITEMS: &[BlockItem] = &[BlockItem::BareVar(simple_binding(20, 21, "x"))];
const TIER3_OUTRANKS_TIER4_TOP: &[TopLevelItem] = &[
    TopLevelItem::Lexical(simple_binding(4, 5, "x")),
    TopLevelItem::Lexical(simple_binding(11, 12, "x")),
    TopLevelItem::Block(BlockFixture {
        items: TIER3_OUTRANKS_TIER4_ITEMS,
    }),
];

// --- Fixture: `{ let x; var x; } { let y; var y; }` (first Block wins) ----
const TWO_BLOCKS_FIRST_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "x")),
    BlockItem::BareVar(simple_binding(13, 14, "x")),
];
const TWO_BLOCKS_SECOND_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(24, 25, "y")),
    BlockItem::BareVar(simple_binding(31, 32, "y")),
];
const TWO_BLOCKS_TOP: &[TopLevelItem] = &[
    TopLevelItem::Block(BlockFixture {
        items: TWO_BLOCKS_FIRST_ITEMS,
    }),
    TopLevelItem::Block(BlockFixture {
        items: TWO_BLOCKS_SECOND_ITEMS,
    }),
];

const FIXTURES: &[Fixture] = &[
    Fixture {
        id: "block-lexical-then-var-collision",
        source: "{ let x; var x; }",
        top_level_items: LEXICAL_THEN_VAR_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 14, "x"),
        },
    },
    Fixture {
        id: "block-var-then-lexical-collision",
        source: "{ var x; let x; }",
        top_level_items: VAR_THEN_LEXICAL_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 14, "x"),
        },
    },
    Fixture {
        id: "block-lexical-var-distinct-accepted",
        source: "{ let x; var y; }",
        top_level_items: LEXICAL_VAR_DISTINCT_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-var-lexical-distinct-accepted",
        source: "{ var x; let y; }",
        top_level_items: VAR_LEXICAL_DISTINCT_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "script-propagation-lexical-then-block-var",
        source: "let x; { var x; }",
        top_level_items: SCRIPT_LEXICAL_THEN_BLOCK_VAR_TOP,
        expected: ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(13, 14, "x"),
        },
    },
    Fixture {
        id: "script-propagation-block-var-then-lexical",
        source: "{ var x; } let x;",
        top_level_items: SCRIPT_BLOCK_VAR_THEN_LEXICAL_TOP,
        expected: ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(15, 16, "x"),
        },
    },
    Fixture {
        id: "script-propagation-accepted",
        source: "let x; { var y; }",
        top_level_items: SCRIPT_ACCEPTED_PROPAGATION_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "asymmetry-outer-var-inner-lexical-accepted",
        source: "var x; { let x; }",
        top_level_items: ASYMMETRY_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "semantic-name-authored-spelling-separation",
        source: "{ let a; var \\u0061; }",
        top_level_items: SEMANTIC_SPLIT_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 19, "\\u0061"),
        },
    },
    Fixture {
        id: "block-r02-outranks-script-r02",
        source: "let x; { let x; var x; }",
        top_level_items: BLOCK_BEFORE_SCRIPT_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(20, 21, "x"),
        },
    },
    Fixture {
        id: "block-r01-outranks-block-r02",
        source: "{ let x; let x; var x; }",
        top_level_items: R01_BEFORE_R02_TOP,
        expected: ExpectedDisposition::Ee14R01 {
            primary: ExpectedAnchor::new(13, 14, "x"),
        },
    },
    Fixture {
        id: "repeated-bare-var-accepted",
        source: "{ var x; var x; }",
        top_level_items: REPEATED_VAR_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "tier1-outranks-block-r02",
        source: "{ let x; var x; var \\u0030; }",
        top_level_items: TIER1_OUTRANKS_BLOCK_R02_TOP,
        expected: ExpectedDisposition::StaticSemanticsRejected {
            rule_id: "EE-01-R01",
            subject: ExpectedAnchor::new(20, 26, "\\u0030"),
        },
    },
    Fixture {
        id: "tier3-outranks-tier4",
        source: "let x; let x; { var x; }",
        top_level_items: TIER3_OUTRANKS_TIER4_TOP,
        expected: ExpectedDisposition::Ee36R01 {
            primary: ExpectedAnchor::new(11, 12, "x"),
        },
    },
    Fixture {
        id: "two-blocks-first-block-owns-r02-primary",
        source: "{ let x; var x; } { let y; var y; }",
        top_level_items: TWO_BLOCKS_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 14, "x"),
        },
    },
    Fixture {
        id: "binding-local-escaped-start-digit-rejected",
        source: "{var \\u0030;}",
        top_level_items: &[],
        expected: ExpectedDisposition::StaticSemanticsRejected {
            rule_id: "EE-01-R01",
            subject: ExpectedAnchor::new(5, 11, "\\u0030"),
        },
    },
    Fixture {
        id: "definitive-grammar-empty-braced-escape-rejected",
        source: "{var \\u{};}",
        top_level_items: &[],
        expected: ExpectedDisposition::SyntaxRejected {
            subject: ExpectedAnchor::new(5, 9, "\\u{}"),
        },
    },
    Fixture {
        id: "unsupported-non-eof-asi",
        source: "{var x}",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NonEofAsi),
    },
    Fixture {
        id: "unsupported-initializer",
        source: "{var x=x;}",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::Initializer),
    },
    Fixture {
        id: "unsupported-multi-declarator",
        source: "{var x,y;}",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::MultiDeclarator),
    },
    Fixture {
        id: "unsupported-comment-trivia",
        source: "{var x; /*c*/}",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::CommentTrivia),
    },
    Fixture {
        id: "unsupported-deeper-nesting",
        source: "{ { var x; } }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::DeeperNesting),
    },
    Fixture {
        id: "unsupported-for-var",
        source: "for (var x;;) {}",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::ForVar),
    },
    Fixture {
        id: "unsupported-binding-pattern",
        source: "{var [x]=y;}",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::BindingPattern),
    },
    Fixture {
        id: "unsupported-module-goal",
        source: "{ var x; } export {};",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::ModuleGoal),
    },
];

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing bare Block var fixture {id}"))
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

fn block_binding_facts(block: &BlockFixture) -> impl Iterator<Item = (&BlockItem, BindingFact)> {
    block.items.iter().map(|item| {
        let fact = match item {
            BlockItem::Lexical(fact) | BlockItem::BareVar(fact) => *fact,
        };
        (item, fact)
    })
}

/// Independent `LexicallyDeclaredNames` for a Block, derived only from
/// `BlockItem::Lexical` contributors.
fn block_lexically_declared_names(block: &BlockFixture) -> Vec<&'static str> {
    block
        .items
        .iter()
        .filter_map(|item| match item {
            BlockItem::Lexical(fact) => Some(fact.semantic_name),
            BlockItem::BareVar(_) => None,
        })
        .collect()
}

/// Independent `VarDeclaredNames` for a Block, derived only from
/// `BlockItem::BareVar` contributors.
fn block_var_declared_names(block: &BlockFixture) -> Vec<&'static str> {
    block
        .items
        .iter()
        .filter_map(|item| match item {
            BlockItem::BareVar(fact) => Some(fact.semantic_name),
            BlockItem::Lexical(_) => None,
        })
        .collect()
}

/// Independent Script `TopLevelLexicallyDeclaredNames`. A Block's own
/// lexical names are intentionally excluded: they do not propagate to Script
/// lexical-declared-name evidence.
fn script_top_level_lexically_declared_names(items: &[TopLevelItem]) -> Vec<&'static str> {
    items
        .iter()
        .filter_map(|item| match item {
            TopLevelItem::Lexical(fact) => Some(fact.semantic_name),
            TopLevelItem::Var(_) | TopLevelItem::Block(_) => None,
        })
        .collect()
}

/// Independent Script `VarDeclaredNames` / `TopLevelVarDeclaredNames`. A
/// top-level Block's `VarDeclaredNames` contributors propagate into this
/// set; a Block's lexical names never do.
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
/// `VarDeclaredNames` (newly reachable `EE-14-R02`).
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
/// a propagated Block-var contributor.
fn script_has_lexical_var_collision(items: &[TopLevelItem]) -> bool {
    intersects(
        &script_top_level_lexically_declared_names(items),
        &script_var_declared_names(items),
    )
}

/// Tier 3: Script `TopLevelLexicallyDeclaredNames` contains a duplicate
/// BoundName (existing `EE-36-R01`).
fn script_has_duplicate_lexical(items: &[TopLevelItem]) -> bool {
    has_duplicate(&script_top_level_lexically_declared_names(items))
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

/// Project evidence-order decision across the tiers this oracle models.
/// Tier 1 (binding-local) is not derived here: a Tier-1-invalid identifier
/// can never also be a valid BoundName that could collide with anything, so
/// no fixture can construct a genuine Tier-1/Tier-2/Tier-4 name collision;
/// Tier-1 primacy is instead frozen as a literal per-fixture fact (see
/// `tier1-outranks-block-r02`). Tiers 2a, 2b, 3, and 4 are all independently
/// derived below, in project evidence-tier order.
fn expected_primary_rule_id(items: &[TopLevelItem]) -> Option<&'static str> {
    if any_block_has_duplicate_lexical(items) {
        return Some("EE-14-R01");
    }
    if any_block_has_lexical_var_collision(items) {
        return Some("EE-14-R02");
    }
    if script_has_duplicate_lexical(items) {
        return Some("EE-36-R01");
    }
    if script_has_lexical_var_collision(items) {
        return Some("EE-36-R02");
    }
    None
}

#[test]
fn selected_bare_block_var_grammar_and_envelope_are_exact() {
    assert_eq!(
        SELECTED_BLOCK_ITEM_GRAMMAR,
        "SelectedLexicalDeclaration | SelectedBareBlockVar"
    );
    assert_eq!(
        SELECTED_BARE_BLOCK_VAR_GRAMMAR,
        "var SelectedBindingIdentifier ;"
    );
    assert_eq!(SELECTED_PARSE_GOAL, "Script");
    assert!(!SELECTED_IS_STRICT);
    assert_eq!(
        EVIDENCE_TIER_ORDER,
        ["EE-14-R01", "EE-14-R02", "EE-36-R01", "EE-36-R02"]
    );
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
                    for (_, fact) in block_binding_facts(block) {
                        validate_anchor(&source, fact.authored);
                    }
                }
            }
        }

        match fixture.expected {
            ExpectedDisposition::Ee14R01 { primary }
            | ExpectedDisposition::Ee14R02 { primary }
            | ExpectedDisposition::Ee36R01 { primary }
            | ExpectedDisposition::Ee36R02 { primary } => validate_anchor(&source, primary),
            ExpectedDisposition::StaticSemanticsRejected { subject, .. }
            | ExpectedDisposition::SyntaxRejected { subject } => validate_anchor(&source, subject),
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
            ExpectedDisposition::StaticSemanticsRejected { .. }
                | ExpectedDisposition::SyntaxRejected { .. }
                | ExpectedDisposition::UnsupportedCoverage(_)
        ) {
            // Binding-local, grammar, and unsupported-coverage fixtures are
            // frozen literal facts. Some (like `tier1-outranks-block-r02`)
            // still carry modeled top-level items to prove their literal
            // disposition outranks an independently derivable tier; that
            // precedence is exercised by a dedicated test instead of this
            // generic declared-name-derivation loop.
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
            ExpectedDisposition::Ee36R01 { .. } => {
                assert_eq!(
                    derived,
                    Some("EE-36-R01"),
                    "{} must independently derive EE-36-R01",
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
            ExpectedDisposition::StaticSemanticsRejected { .. }
            | ExpectedDisposition::SyntaxRejected { .. }
            | ExpectedDisposition::UnsupportedCoverage(_) => unreachable!(),
        }
    }
}

#[test]
fn block_local_collision_is_source_order_independent() {
    let lexical_then_var = fixture("block-lexical-then-var-collision");
    let var_then_lexical = fixture("block-var-then-lexical-collision");

    assert_eq!(
        expected_primary_rule_id(lexical_then_var.top_level_items),
        Some("EE-14-R02")
    );
    assert_eq!(
        expected_primary_rule_id(var_then_lexical.top_level_items),
        Some("EE-14-R02")
    );

    // Both orderings place the primary evidence at the later-in-source
    // occurrence among the two colliding bindings, at the same offsets,
    // because both fixtures collide over one-character identifiers at
    // symmetric positions.
    assert_eq!(
        lexical_then_var.expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 14, "x")
        }
    );
    assert_eq!(
        var_then_lexical.expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 14, "x")
        }
    );
}

#[test]
fn asymmetry_sentinel_is_not_falsely_symmetric_with_script_propagation() {
    let asymmetry = fixture("asymmetry-outer-var-inner-lexical-accepted");
    let propagation = fixture("script-propagation-lexical-then-block-var");

    assert_eq!(asymmetry.expected, ExpectedDisposition::AcceptedIncomplete);
    assert_eq!(
        propagation.expected,
        ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(13, 14, "x")
        }
    );

    // Structural reason for the asymmetry: the outer top-level var does not
    // appear in the inner Block's own VarDeclaredNames, and the inner
    // Block's lexical name does not appear in Script's
    // TopLevelLexicallyDeclaredNames.
    let TopLevelItem::Block(inner_block) = asymmetry.top_level_items[1] else {
        panic!("asymmetry fixture's second item must be a Block");
    };
    assert!(block_var_declared_names(&inner_block).is_empty());
    assert!(script_top_level_lexically_declared_names(asymmetry.top_level_items).is_empty());
    assert!(!script_has_lexical_var_collision(asymmetry.top_level_items));
}

#[test]
fn semantic_name_equality_drives_collision_not_authored_spelling_equality() {
    let fixture = fixture("semantic-name-authored-spelling-separation");
    let TopLevelItem::Block(block) = fixture.top_level_items[0] else {
        panic!("fixture's only item must be a Block");
    };

    let lexical = block
        .items
        .iter()
        .find_map(|item| match item {
            BlockItem::Lexical(fact) => Some(*fact),
            BlockItem::BareVar(_) => None,
        })
        .expect("lexical binding must exist");
    let bare_var = block
        .items
        .iter()
        .find_map(|item| match item {
            BlockItem::BareVar(fact) => Some(*fact),
            BlockItem::Lexical(_) => None,
        })
        .expect("bare var binding must exist");

    // Authored spellings differ ("a" vs "a"); a wrong implementation
    // comparing authored spelling instead of semantic name would miss this
    // collision entirely.
    assert_ne!(lexical.authored.fragment, bare_var.authored.fragment);
    assert_eq!(lexical.semantic_name, bare_var.semantic_name);
    assert_eq!(
        decoded_expected_name(bare_var.semantic_code_points),
        Some(bare_var.semantic_name.to_owned())
    );

    assert_eq!(
        expected_primary_rule_id(fixture.top_level_items),
        Some("EE-14-R02")
    );
    assert_eq!(
        fixture.expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 19, "\\u0061")
        }
    );
}

#[test]
fn evidence_order_precedence_sentinels_hold() {
    let block_before_script = fixture("block-r02-outranks-script-r02");
    assert!(any_block_has_lexical_var_collision(
        block_before_script.top_level_items
    ));
    assert!(script_has_lexical_var_collision(
        block_before_script.top_level_items
    ));
    assert_eq!(
        expected_primary_rule_id(block_before_script.top_level_items),
        Some("EE-14-R02"),
        "Block EE-14-R02 must outrank Script EE-36-R02 even though both are true"
    );

    let r01_before_r02 = fixture("block-r01-outranks-block-r02");
    assert!(any_block_has_duplicate_lexical(
        r01_before_r02.top_level_items
    ));
    assert!(any_block_has_lexical_var_collision(
        r01_before_r02.top_level_items
    ));
    assert_eq!(
        expected_primary_rule_id(r01_before_r02.top_level_items),
        Some("EE-14-R01"),
        "existing EE-14-R01 must remain primary over newly reachable EE-14-R02"
    );
}

#[test]
fn tier1_binding_local_fact_outranks_an_independently_derivable_block_r02() {
    let combined = fixture("tier1-outranks-block-r02");

    // The two modeled items alone independently derive a true EE-14-R02,
    // proving this is a real precedence override and not a vacuous one.
    assert!(any_block_has_lexical_var_collision(
        combined.top_level_items
    ));
    assert_eq!(
        expected_primary_rule_id(combined.top_level_items),
        Some("EE-14-R02"),
        "the modeled portion alone must independently derive EE-14-R02"
    );

    // The fixture's frozen disposition is nonetheless the literal Tier-1
    // binding-local fact, never the derivable Tier-2b result.
    assert_eq!(
        combined.expected,
        ExpectedDisposition::StaticSemanticsRejected {
            rule_id: "EE-01-R01",
            subject: ExpectedAnchor::new(20, 26, "\\u0030"),
        }
    );
}

#[test]
fn tier3_script_duplicate_lexical_outranks_tier4_script_lexical_var_collision() {
    let combined = fixture("tier3-outranks-tier4");

    assert!(script_has_duplicate_lexical(combined.top_level_items));
    assert!(script_has_lexical_var_collision(combined.top_level_items));
    assert_eq!(
        expected_primary_rule_id(combined.top_level_items),
        Some("EE-36-R01"),
        "existing EE-36-R01 must remain primary over EE-36-R02 even though both are true"
    );
    assert_eq!(
        combined.expected,
        ExpectedDisposition::Ee36R01 {
            primary: ExpectedAnchor::new(11, 12, "x"),
        },
        "the primary anchor is the duplicate (second) top-level lexical binding, \
         unaffected by the co-occurring Block-var propagation"
    );
}

#[test]
fn multi_block_source_order_selects_the_first_qualifying_block_for_r02_primary() {
    let two_blocks = fixture("two-blocks-first-block-owns-r02-primary");
    let TopLevelItem::Block(first_block) = two_blocks.top_level_items[0] else {
        panic!("fixture's first item must be a Block");
    };
    let TopLevelItem::Block(second_block) = two_blocks.top_level_items[1] else {
        panic!("fixture's second item must be a Block");
    };

    // Both Blocks independently qualify: this is not a vacuous ordering
    // proof where only one Block could ever have been chosen.
    assert!(intersects(
        &block_lexically_declared_names(&first_block),
        &block_var_declared_names(&first_block)
    ));
    assert!(intersects(
        &block_lexically_declared_names(&second_block),
        &block_var_declared_names(&second_block)
    ));

    assert_eq!(
        first_block_index_with_lexical_var_collision(two_blocks.top_level_items),
        Some(0),
        "project evidence-order policy selects the first qualifying Block in source order"
    );
    assert_eq!(
        two_blocks.expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 14, "x"),
        },
        "primary evidence must belong to the first Block's own var binding, not the second Block's"
    );
}

#[test]
fn repeated_bare_var_names_alone_do_not_trigger_the_lexical_var_rule() {
    let repeated = fixture("repeated-bare-var-accepted");
    let TopLevelItem::Block(block) = repeated.top_level_items[0] else {
        panic!("fixture's only item must be a Block");
    };
    assert_eq!(block_var_declared_names(&block), vec!["x", "x"]);
    assert!(block_lexically_declared_names(&block).is_empty());
    assert!(!any_block_has_lexical_var_collision(
        repeated.top_level_items
    ));
    assert!(!any_block_has_duplicate_lexical(repeated.top_level_items));
    assert_eq!(repeated.expected, ExpectedDisposition::AcceptedIncomplete);
}

#[test]
fn binding_local_and_definitive_grammar_rejections_are_frozen_literal_facts() {
    let binding_local = fixture("binding-local-escaped-start-digit-rejected");
    assert_eq!(
        binding_local.expected,
        ExpectedDisposition::StaticSemanticsRejected {
            rule_id: "EE-01-R01",
            subject: ExpectedAnchor::new(5, 11, "\\u0030"),
        }
    );

    let grammar = fixture("definitive-grammar-empty-braced-escape-rejected");
    assert_eq!(
        grammar.expected,
        ExpectedDisposition::SyntaxRejected {
            subject: ExpectedAnchor::new(5, 9, "\\u{}"),
        }
    );

    // `{var x}` is valid ECMAScript through non-EOF ASI; it must remain
    // UnsupportedCoverage in this leaf, never SyntaxRejected.
    let non_eof_asi = fixture("unsupported-non-eof-asi");
    assert_eq!(
        non_eof_asi.expected,
        ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NonEofAsi)
    );
    assert_ne!(
        non_eof_asi.expected,
        ExpectedDisposition::SyntaxRejected {
            subject: ExpectedAnchor::new(0, 0, "")
        }
    );
}

#[test]
fn unsupported_coverage_boundaries_remain_representative_and_distinct() {
    let expected = [
        ("unsupported-non-eof-asi", UnsupportedReason::NonEofAsi),
        ("unsupported-initializer", UnsupportedReason::Initializer),
        (
            "unsupported-multi-declarator",
            UnsupportedReason::MultiDeclarator,
        ),
        (
            "unsupported-comment-trivia",
            UnsupportedReason::CommentTrivia,
        ),
        (
            "unsupported-deeper-nesting",
            UnsupportedReason::DeeperNesting,
        ),
        ("unsupported-for-var", UnsupportedReason::ForVar),
        (
            "unsupported-binding-pattern",
            UnsupportedReason::BindingPattern,
        ),
        ("unsupported-module-goal", UnsupportedReason::ModuleGoal),
    ];

    for (id, reason) in expected {
        assert_eq!(
            fixture(id).expected,
            ExpectedDisposition::UnsupportedCoverage(reason)
        );
    }
}

#[test]
fn oracle_source_is_candidate_independent_and_does_not_call_production_owners() {
    for forbidden in [
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
        concat!("crate::ecmascript", "::"),
        concat!("super::selected", "_"),
        concat!("SelectedStaticSemantics", "Outcome"),
        concat!("SelectedOneLevelBlockStaticSemantics", "Outcome"),
        concat!("Qualification", "Outcome"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "bare Block var oracle must remain candidate-independent: found {forbidden}"
        );
    }
}
