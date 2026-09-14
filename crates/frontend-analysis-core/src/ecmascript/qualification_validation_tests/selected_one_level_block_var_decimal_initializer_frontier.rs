//! Candidate-independent successor oracle composing the accepted one-level
//! Block-contained `var` declarator-cardinality theorem (Issue #688 research
//! / #689-#690 validation / #691-#692 production / #693-#694 1..N
//! validation / #695-#696 1..N production) with the already-accepted
//! bounded top-level decimal-integer initializer leaf (Issue #326-#327
//! validation / #328-#329 top-level production), frozen for Issue #697.
//!
//! This oracle freezes exactly:
//!
//! ```text
//! SelectedBlockVarStatement ::=
//!     var SelectedBlockVarDeclaration
//!         ( , SelectedBlockVarDeclaration )*
//!     ;
//!
//! SelectedBlockVarDeclaration ::=
//!     SelectedBindingIdentifier
//!   | SelectedBindingIdentifier = SelectedDecimalInteger
//!
//! SelectedDecimalInteger ::=
//!     0
//!   | [1-9][0-9]*
//! ```
//!
//! with declarator cardinality `1..N`, an optional selected decimal-integer
//! initializer *per declarator*, an authored semicolon terminator,
//! one-level top-level `Block` only, no comments, no deeper `Block`, no
//! `for (var ...)`, and no `BindingPattern`. This successor widens only the
//! Block `var` declarator *source family* (bare identifier -> identifier
//! with an optional bounded decimal initializer); it does not widen
//! placement, declarator cardinality semantics, ASI, source-name
//! correspondence, Binding / Scope, or runtime semantics, and it does not
//! change the Early Error identity set frozen by #689/#693.
//!
//! Every positive fixture below contains at least one selected decimal
//! initializer: the accepted initializer-free predecessor family remains
//! valid through #693/#696 and is not re-proven here.
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
//! The historical oracles (`selected_one_level_block_var_multi_declarator_frontier.rs`,
//! `selected_variable_statement_decimal_initializer_frontier.rs`,
//! `qualification_selected_one_level_block_bare_var_validation_tests.rs`,
//! `selected_post_688_bare_block_var_slice_completion.rs`,
//! `selected_post_693_block_var_multi_declarator_slice_completion.rs`)
//! remain frozen and unmodified; some of their fixtures intentionally
//! describe an initializer-bearing Block `var` source as unsupported for
//! their own narrower frontier. That is correct historical authority and is
//! not "fixed" here.

#![allow(clippy::assertions_on_constants)]

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

const THIS_SOURCE: &str =
    include_str!("selected_one_level_block_var_decimal_initializer_frontier.rs");
const HISTORICAL_BLOCK_VAR_MULTI_DECLARATOR: &str =
    include_str!("selected_one_level_block_var_multi_declarator_frontier.rs");
const HISTORICAL_TOP_LEVEL_DECIMAL_INITIALIZER: &str =
    include_str!("selected_variable_statement_decimal_initializer_frontier.rs");
const HISTORICAL_CORRESPONDENCE: &str =
    include_str!("../selected_variable_statement_name_correspondence_validation_tests.rs");

const SELECTED_BLOCK_VAR_STATEMENT_GRAMMAR: &str =
    "var SelectedBlockVarDeclaration ( , SelectedBlockVarDeclaration )* ;";
const SELECTED_BLOCK_VAR_DECLARATION_GRAMMAR: &str =
    "SelectedBindingIdentifier | SelectedBindingIdentifier = SelectedDecimalInteger";
const SELECTED_DECIMAL_INTEGER_GRAMMAR: &str = "0 | [1-9][0-9]*";
const SELECTED_LIST_CARDINALITY: &str = "1..N";
const SELECTED_PARSE_GOAL: &str = "Script";
const SELECTED_IS_STRICT: bool = false;
const POSITIVE_LIFECYCLE: &str = "SelectedAcceptedIncomplete";

/// This successor requires no source-name correspondence widening: selected
/// decimal initializers contain no `IdentifierReference` and create no new
/// correspondence query or relation.
const CORRESPONDENCE_WIDENING_REQUIRED: bool = false;

/// Project evidence-ordering tiers, frozen unchanged from #688/#689/#693.
/// This is project diagnostic/validation policy, not an ECMA-262
/// diagnostic-order claim. Tier 1 (binding-local) and Tier 3 (`EE-36-R01`)
/// are inherited unchanged and are not independently re-derived by this
/// successor's declared-name model; the Tier-1-vs-Tier-2b race is instead
/// pinned as a literal frozen fact (see `TIER1_RACE`) because Tier 1 itself
/// remains outside this leaf's derivation responsibility.
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
/// semantic-name identity remain distinguishable. `decimal_rhs` is
/// validation-only provenance for the optional selected decimal initializer
/// and never participates in `BoundNames`, `VarDeclaredNames`, or collision
/// evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BindingFact {
    authored: ExpectedAnchor,
    semantic_name: &'static str,
    semantic_code_points: &'static [u32],
    decimal_rhs: Option<ExpectedAnchor>,
}

const fn simple_binding(start: usize, end: usize, name: &'static str) -> BindingFact {
    BindingFact {
        authored: ExpectedAnchor::new(start, end, name),
        semantic_name: name,
        semantic_code_points: &[],
        decimal_rhs: None,
    }
}

const fn simple_binding_initialized(
    start: usize,
    end: usize,
    name: &'static str,
    rhs_start: usize,
    rhs_end: usize,
    rhs_fragment: &'static str,
) -> BindingFact {
    BindingFact {
        authored: ExpectedAnchor::new(start, end, name),
        semantic_name: name,
        semantic_code_points: &[],
        decimal_rhs: Some(ExpectedAnchor::new(rhs_start, rhs_end, rhs_fragment)),
    }
}

const A_CODE_POINTS: &[u32] = &[0x61];
const E_COMBINING_CODE_POINTS: &[u32] = &[0x65, 0x0301];

const fn escaped_a_initialized(
    start: usize,
    end: usize,
    rhs_start: usize,
    rhs_end: usize,
    rhs_fragment: &'static str,
) -> BindingFact {
    BindingFact {
        authored: ExpectedAnchor::new(start, end, "\\u0061"),
        semantic_name: "a",
        semantic_code_points: A_CODE_POINTS,
        decimal_rhs: Some(ExpectedAnchor::new(rhs_start, rhs_end, rhs_fragment)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockItem {
    Lexical(BindingFact),
    /// One `SelectedBlockVarStatement`'s ordered, duplicate-preserving
    /// `VariableDeclarationList` of 1..N declarators, each with an
    /// independently stated optional decimal initializer.
    VarStatement(&'static [BindingFact]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockFixture {
    items: &'static [BlockItem],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopLevelItem {
    Lexical(BindingFact),
    Block(BlockFixture),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedReason {
    NonEofAsi,
    CommentTrivia,
    /// A later `VariableDeclarationList` position missing a definitive
    /// declarator entirely (e.g. a trailing comma before `;`).
    IncompleteDeclaratorList,
    /// A declarator whose `=` is present but whose right-hand side is
    /// absent or incomplete.
    IncompleteInitializerExpression,
    /// A numeric literal spelling outside the exact selected
    /// `0 | [1-9][0-9]*` decimal grammar (leading zero, separator,
    /// fraction, exponent, BigInt suffix, unary sign).
    NumericNeighbor,
    /// A valid ECMAScript initializer outside the selected decimal family
    /// (Boolean/null/`this`/String/IdentifierReference, direct or escaped).
    NonDecimalInitializer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedDisposition {
    /// No Block-local or Script-level lexical/var collision is reachable.
    AcceptedIncomplete,
    /// Block `LexicallyDeclaredNames` intersects Block `VarDeclaredNames`.
    Ee14R02 { primary: ExpectedAnchor },
    /// Block `LexicallyDeclaredNames` contains a duplicate BoundName.
    Ee14R01 { primary: ExpectedAnchor },
    /// Script `LexicallyDeclaredNames` (top-level only) intersects Script
    /// `VarDeclaredNames`, including a propagated Block-var contributor
    /// from any declarator position (initialized or not).
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

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing Block var decimal-initializer fixture {id}"))
}

// --- Positive fixtures (every fixture below carries >=1 initializer) ------

// `{ var a=0; }`
const P1_VAR: &[BindingFact] = &[simple_binding_initialized(6, 7, "a", 8, 9, "0")];
const P1_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P1_VAR)],
})];

// `{ var a=12345; }`
const P2_VAR: &[BindingFact] = &[simple_binding_initialized(6, 7, "a", 8, 13, "12345")];
const P2_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P2_VAR)],
})];

// `{ var a=1,b; }` (first declarator initialized)
const P3_VAR: &[BindingFact] = &[
    simple_binding_initialized(6, 7, "a", 8, 9, "1"),
    simple_binding(10, 11, "b"),
];
const P3_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P3_VAR)],
})];

// `{ var a,b=2; }` (final declarator initialized)
const P4_VAR: &[BindingFact] = &[
    simple_binding(6, 7, "a"),
    simple_binding_initialized(8, 9, "b", 10, 11, "2"),
];
const P4_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P4_VAR)],
})];

// `{ var a=1,b=2; }` (both declarators initialized)
const P5_VAR: &[BindingFact] = &[
    simple_binding_initialized(6, 7, "a", 8, 9, "1"),
    simple_binding_initialized(10, 11, "b", 12, 13, "2"),
];
const P5_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P5_VAR)],
})];

// `{ var a=1,b,c=2; }` (first and final initialized, interior bare)
const P6_VAR: &[BindingFact] = &[
    simple_binding_initialized(6, 7, "a", 8, 9, "1"),
    simple_binding(10, 11, "b"),
    simple_binding_initialized(12, 13, "c", 14, 15, "2"),
];
const P6_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P6_VAR)],
})];

// `{ var a,b=2,c; }` (interior initialized, first and final bare)
const P7_VAR: &[BindingFact] = &[
    simple_binding(6, 7, "a"),
    simple_binding_initialized(8, 9, "b", 10, 11, "2"),
    simple_binding(12, 13, "c"),
];
const P7_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P7_VAR)],
})];

// `{ var a=1,b=2,c=3; }` (all three initialized)
const P8_VAR: &[BindingFact] = &[
    simple_binding_initialized(6, 7, "a", 8, 9, "1"),
    simple_binding_initialized(10, 11, "b", 12, 13, "2"),
    simple_binding_initialized(14, 15, "c", 16, 17, "3"),
];
const P8_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P8_VAR)],
})];

// `{ var a=1,a; }` (repeated direct name, first initialized)
const P9_VAR: &[BindingFact] = &[
    simple_binding_initialized(6, 7, "a", 8, 9, "1"),
    simple_binding(10, 11, "a"),
];
const P9_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P9_VAR)],
})];

// `{ var a=1,a=2; }` (repeated direct/escaped same name, both initialized)
const P10_VAR: &[BindingFact] = &[
    simple_binding_initialized(6, 7, "a", 8, 9, "1"),
    escaped_a_initialized(10, 16, 17, 18, "2"),
];
const P10_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P10_VAR)],
})];

const E_ACUTE_CODE_POINTS: &[u32] = &[0x00e9];

// `{ var é=1,é=2; }` (no Unicode normalization, both initialized)
const P11_VAR: &[BindingFact] = &[
    BindingFact {
        authored: ExpectedAnchor::new(6, 8, "é"),
        semantic_name: "é",
        semantic_code_points: E_ACUTE_CODE_POINTS,
        decimal_rhs: Some(ExpectedAnchor::new(9, 10, "1")),
    },
    BindingFact {
        authored: ExpectedAnchor::new(11, 18, "e\\u0301"),
        semantic_name: "e\u{301}",
        semantic_code_points: E_COMBINING_CODE_POINTS,
        decimal_rhs: Some(ExpectedAnchor::new(19, 20, "2")),
    },
];
const P11_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture {
    items: &[BlockItem::VarStatement(P11_VAR)],
})];

// --- Block EE-14-R02 collision fixtures (initializer-bearing) -------------

// `{ let a; var a=1,b; }` (first declarator collision, initialized)
const C1_VAR: &[BindingFact] = &[
    simple_binding_initialized(13, 14, "a", 15, 16, "1"),
    simple_binding(17, 18, "b"),
];
const C1_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "a")),
    BlockItem::VarStatement(C1_VAR),
];
const C1_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: C1_ITEMS })];

// `{ let b; var a=1,b=2; }` (second/final-of-two declarator collision, initialized)
const C2_VAR: &[BindingFact] = &[
    simple_binding_initialized(13, 14, "a", 15, 16, "1"),
    simple_binding_initialized(17, 18, "b", 19, 20, "2"),
];
const C2_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "b")),
    BlockItem::VarStatement(C2_VAR),
];
const C2_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: C2_ITEMS })];

// `{ let b; var a,b=2,c; }` (interior declarator collision, initialized)
const C3_VAR: &[BindingFact] = &[
    simple_binding(13, 14, "a"),
    simple_binding_initialized(15, 16, "b", 17, 18, "2"),
    simple_binding(19, 20, "c"),
];
const C3_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "b")),
    BlockItem::VarStatement(C3_VAR),
];
const C3_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: C3_ITEMS })];

// `{ let c; var a=1,b=2,c=3; }` (final declarator collision, initialized)
const C4_VAR: &[BindingFact] = &[
    simple_binding_initialized(13, 14, "a", 15, 16, "1"),
    simple_binding_initialized(17, 18, "b", 19, 20, "2"),
    simple_binding_initialized(21, 22, "c", 23, 24, "3"),
];
const C4_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "c")),
    BlockItem::VarStatement(C4_VAR),
];
const C4_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: C4_ITEMS })];

// `{ var a=1,b=2; let b; }` (lexical-after-list collision, initialized)
const C5_VAR: &[BindingFact] = &[
    simple_binding_initialized(6, 7, "a", 8, 9, "1"),
    simple_binding_initialized(10, 11, "b", 12, 13, "2"),
];
const C5_ITEMS: &[BlockItem] = &[
    BlockItem::VarStatement(C5_VAR),
    BlockItem::Lexical(simple_binding(19, 20, "b")),
];
const C5_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: C5_ITEMS })];

// `{ let a; var a=1,x=2; }` (escaped first-declarator collision, initialized)
const C6_VAR: &[BindingFact] = &[
    escaped_a_initialized(13, 19, 20, 21, "1"),
    simple_binding_initialized(22, 23, "x", 24, 25, "2"),
];
const C6_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "a")),
    BlockItem::VarStatement(C6_VAR),
];
const C6_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: C6_ITEMS })];

// `{ let a; var x=1,a=2; }` (escaped later-declarator collision, initialized)
const C7_VAR: &[BindingFact] = &[
    simple_binding_initialized(13, 14, "x", 15, 16, "1"),
    escaped_a_initialized(17, 23, 24, 25, "2"),
];
const C7_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "a")),
    BlockItem::VarStatement(C7_VAR),
];
const C7_TOP: &[TopLevelItem] = &[TopLevelItem::Block(BlockFixture { items: C7_ITEMS })];

// --- Script EE-36-R02 propagation fixtures (initializer-bearing) ----------

// `let b; { var a=1,b=2; }` (Script propagation, non-first declarator)
const S1_VAR: &[BindingFact] = &[
    simple_binding_initialized(10, 11, "a", 15, 16, "1"),
    simple_binding_initialized(17, 18, "b", 19, 20, "2"),
];
const S1_TOP: &[TopLevelItem] = &[
    TopLevelItem::Lexical(simple_binding(4, 5, "b")),
    TopLevelItem::Block(BlockFixture {
        items: &[BlockItem::VarStatement(S1_VAR)],
    }),
];

// `{ var a=1,b=2; } let b;` (Script propagation, lexical-after-Block)
const S2_VAR: &[BindingFact] = &[
    simple_binding_initialized(6, 7, "a", 8, 9, "1"),
    simple_binding_initialized(10, 11, "b", 12, 13, "2"),
];
const S2_TOP: &[TopLevelItem] = &[
    TopLevelItem::Block(BlockFixture {
        items: &[BlockItem::VarStatement(S2_VAR)],
    }),
    TopLevelItem::Lexical(simple_binding(21, 22, "b")),
];

// --- Same-tier / cross-tier multi-Block ordering (initializer-bearing) ----

// `{ let a; var x=1,a=2; }\n{ let b; var y=3,b=4; }`
const O1_FIRST_VAR: &[BindingFact] = &[
    simple_binding_initialized(13, 14, "x", 15, 16, "1"),
    simple_binding_initialized(17, 18, "a", 19, 20, "2"),
];
const O1_FIRST_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "a")),
    BlockItem::VarStatement(O1_FIRST_VAR),
];
const O1_SECOND_VAR: &[BindingFact] = &[
    simple_binding_initialized(37, 38, "y", 39, 40, "3"),
    simple_binding_initialized(41, 42, "b", 43, 44, "4"),
];
const O1_SECOND_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(30, 31, "b")),
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

// `{ let a; var x=1,a=2; } { let b; let b; }`
const O2_FIRST_VAR: &[BindingFact] = &[
    simple_binding_initialized(13, 14, "x", 15, 16, "1"),
    simple_binding_initialized(17, 18, "a", 19, 20, "2"),
];
const O2_FIRST_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(6, 7, "a")),
    BlockItem::VarStatement(O2_FIRST_VAR),
];
const O2_SECOND_ITEMS: &[BlockItem] = &[
    BlockItem::Lexical(simple_binding(30, 31, "b")),
    BlockItem::Lexical(simple_binding(37, 38, "b")),
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
        id: "block-single-zero-initializer",
        source: "{ var a=0; }",
        top_level_items: P1_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-single-multidigit-initializer",
        source: "{ var a=12345; }",
        top_level_items: P2_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-first-declarator-initialized",
        source: "{ var a=1,b; }",
        top_level_items: P3_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-final-declarator-initialized",
        source: "{ var a,b=2; }",
        top_level_items: P4_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-both-declarators-initialized",
        source: "{ var a=1,b=2; }",
        top_level_items: P5_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-mixed-first-and-final-initialized-three",
        source: "{ var a=1,b,c=2; }",
        top_level_items: P6_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-interior-declarator-initialized-three",
        source: "{ var a,b=2,c; }",
        top_level_items: P7_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-three-declarators-all-initialized",
        source: "{ var a=1,b=2,c=3; }",
        top_level_items: P8_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-repeated-direct-var-initialized",
        source: "{ var a=1,a; }",
        top_level_items: P9_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-repeated-direct-escaped-var-initialized",
        source: "{ var a=1,\\u0061=2; }",
        top_level_items: P10_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-no-unicode-normalization-initialized",
        source: "{ var é=1,e\\u0301=2; }",
        top_level_items: P11_TOP,
        expected: ExpectedDisposition::AcceptedIncomplete,
    },
    Fixture {
        id: "block-first-declarator-collision-initialized",
        source: "{ let a; var a=1,b; }",
        top_level_items: C1_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 14, "a"),
        },
    },
    Fixture {
        id: "block-second-declarator-collision-initialized",
        source: "{ let b; var a=1,b=2; }",
        top_level_items: C2_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(17, 18, "b"),
        },
    },
    Fixture {
        id: "block-interior-declarator-collision-initialized",
        source: "{ let b; var a,b=2,c; }",
        top_level_items: C3_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(15, 16, "b"),
        },
    },
    Fixture {
        id: "block-final-declarator-collision-initialized",
        source: "{ let c; var a=1,b=2,c=3; }",
        top_level_items: C4_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(21, 22, "c"),
        },
    },
    Fixture {
        id: "block-lexical-after-list-collision-initialized",
        source: "{ var a=1,b=2; let b; }",
        top_level_items: C5_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(19, 20, "b"),
        },
    },
    Fixture {
        id: "block-escaped-first-declarator-collision-initialized",
        source: "{ let a; var \\u0061=1,x=2; }",
        top_level_items: C6_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(13, 19, "\\u0061"),
        },
    },
    Fixture {
        id: "block-escaped-later-declarator-collision-initialized",
        source: "{ let a; var x=1,\\u0061=2; }",
        top_level_items: C7_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(17, 23, "\\u0061"),
        },
    },
    Fixture {
        id: "script-propagation-non-first-initialized-declarator",
        source: "let b; { var a=1,b=2; }",
        top_level_items: S1_TOP,
        expected: ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(17, 18, "b"),
        },
    },
    Fixture {
        id: "script-propagation-lexical-after-block-initialized",
        source: "{ var a=1,b=2; } let b;",
        top_level_items: S2_TOP,
        expected: ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(21, 22, "b"),
        },
    },
    Fixture {
        id: "same-tier-sibling-block-source-order-initialized",
        source: "{ let a; var x=1,a=2; }\n{ let b; var y=3,b=4; }",
        top_level_items: O1_TOP,
        expected: ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(17, 18, "a"),
        },
    },
    Fixture {
        id: "cross-tier-priority-over-source-position-initialized",
        source: "{ let a; var x=1,a=2; } { let b; let b; }",
        top_level_items: O2_TOP,
        expected: ExpectedDisposition::Ee14R01 {
            primary: ExpectedAnchor::new(37, 38, "b"),
        },
    },
    Fixture {
        id: "numeric-neighbor-leading-zero-unsupported",
        source: "{ var a=01; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NumericNeighbor),
    },
    Fixture {
        id: "numeric-neighbor-separator-unsupported",
        source: "{ var a=1_0; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NumericNeighbor),
    },
    Fixture {
        id: "numeric-neighbor-fraction-unsupported",
        source: "{ var a=1.0; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NumericNeighbor),
    },
    Fixture {
        id: "numeric-neighbor-exponent-unsupported",
        source: "{ var a=1e2; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NumericNeighbor),
    },
    Fixture {
        id: "numeric-neighbor-bigint-unsupported",
        source: "{ var a=1n; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NumericNeighbor),
    },
    Fixture {
        id: "numeric-neighbor-unary-plus-unsupported",
        source: "{ var a=+1; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NumericNeighbor),
    },
    Fixture {
        id: "numeric-neighbor-unary-minus-unsupported",
        source: "{ var a=-1; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NumericNeighbor),
    },
    Fixture {
        id: "non-decimal-initializer-boolean-unsupported",
        source: "{ var a=true; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::NonDecimalInitializer,
        ),
    },
    Fixture {
        id: "non-decimal-initializer-null-unsupported",
        source: "{ var a=null; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::NonDecimalInitializer,
        ),
    },
    Fixture {
        id: "non-decimal-initializer-this-unsupported",
        source: "{ var a=this; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::NonDecimalInitializer,
        ),
    },
    Fixture {
        id: "non-decimal-initializer-string-unsupported",
        source: "{ var a=\"x\"; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::NonDecimalInitializer,
        ),
    },
    Fixture {
        id: "non-decimal-initializer-identifier-reference-unsupported",
        source: "{ var a=foo; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::NonDecimalInitializer,
        ),
    },
    Fixture {
        id: "non-decimal-initializer-escaped-identifier-reference-unsupported",
        source: "{ var a=\\u0066oo; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::NonDecimalInitializer,
        ),
    },
    Fixture {
        id: "non-decimal-initializer-escaped-reserved-identifier-unsupported",
        source: "{ var a=\\u0069f; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::NonDecimalInitializer,
        ),
    },
    Fixture {
        id: "non-eof-asi-single-declarator-unsupported",
        source: "{ var a=1 }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NonEofAsi),
    },
    Fixture {
        id: "non-eof-asi-two-declarators-unsupported",
        source: "{ var a=1,b=2 }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NonEofAsi),
    },
    Fixture {
        id: "comment-before-declarator-unsupported",
        source: "{ var a=1, /* c */ b; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::CommentTrivia),
    },
    Fixture {
        id: "comment-inside-initializer-unsupported",
        source: "{ var a=/* c */1; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::CommentTrivia),
    },
    Fixture {
        id: "comment-after-initializer-unsupported",
        source: "{ var a=1 /* c */; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::CommentTrivia),
    },
    Fixture {
        id: "transactionality-trailing-comma-no-declarator-unsupported",
        source: "{ var a=1, ; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::IncompleteDeclaratorList,
        ),
    },
    Fixture {
        id: "transactionality-trailing-equals-no-rhs-unsupported",
        source: "{ var a=1,b= ; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::IncompleteInitializerExpression,
        ),
    },
    Fixture {
        id: "transactionality-later-boolean-initializer-unsupported",
        source: "{ var a=1,b=true; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(
            UnsupportedReason::NonDecimalInitializer,
        ),
    },
    Fixture {
        id: "transactionality-later-fraction-initializer-unsupported",
        source: "{ var a=1,b=1.0; }",
        top_level_items: &[],
        expected: ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NumericNeighbor),
    },
    Fixture {
        id: "transactionality-later-malformed-binding-syntax-rejected",
        source: "{ var a=1,b,\\u{}=2; }",
        top_level_items: &[],
        expected: ExpectedDisposition::SyntaxRejected {
            subject: ExpectedAnchor::new(12, 16, "\\u{}"),
        },
    },
];

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

/// Frozen decimal-integer grammar check, used only to confirm (never
/// construct) that every fixture-owned `decimal_rhs` fragment satisfies
/// `0 | [1-9][0-9]*`.
fn is_selected_decimal_integer(candidate: &str) -> bool {
    if candidate == "0" {
        return true;
    }
    let mut bytes = candidate.bytes();
    match bytes.next() {
        Some(b'1'..=b'9') => bytes.all(|byte| byte.is_ascii_digit()),
        _ => false,
    }
}

fn block_declarators(block: &BlockFixture) -> Vec<&'static BindingFact> {
    let mut declarators = Vec::new();
    for item in block.items {
        if let BlockItem::VarStatement(list) = item {
            declarators.extend(list.iter());
        }
    }
    declarators
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
/// declarators, preserving authored `VariableDeclarationList` order.
/// Initializer presence never affects this derivation: every declarator in
/// every position participates regardless of whether it carries a decimal
/// initializer.
fn block_var_declared_names(block: &BlockFixture) -> Vec<&'static str> {
    block_declarators(block)
        .into_iter()
        .map(|fact| fact.semantic_name)
        .collect()
}

/// Independent Script `TopLevelLexicallyDeclaredNames`. A Block's own
/// lexical names are intentionally excluded.
fn script_top_level_lexically_declared_names(items: &[TopLevelItem]) -> Vec<&'static str> {
    items
        .iter()
        .filter_map(|item| match item {
            TopLevelItem::Lexical(fact) => Some(fact.semantic_name),
            TopLevelItem::Block(_) => None,
        })
        .collect()
}

/// Independent Script `VarDeclaredNames` / `TopLevelVarDeclaredNames`. Every
/// declarator of every top-level Block `var` statement propagates into this
/// set regardless of initializer presence; a Block's lexical names never do.
fn script_var_declared_names(items: &[TopLevelItem]) -> Vec<&'static str> {
    let mut names = Vec::new();
    for item in items {
        if let TopLevelItem::Block(block) = item {
            names.extend(block_var_declared_names(block));
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
        TopLevelItem::Lexical(_) => None,
    })
}

/// Tier 2a: any Block's `LexicallyDeclaredNames` contains a duplicate
/// BoundName (existing `EE-14-R01`).
fn any_block_has_duplicate_lexical(items: &[TopLevelItem]) -> bool {
    any_block(items).any(|block| has_duplicate(&block_lexically_declared_names(block)))
}

/// Tier 2b: any Block's `LexicallyDeclaredNames` intersects that Block's own
/// `VarDeclaredNames` (`EE-14-R02`), independent of initializer presence.
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
/// a propagated Block-var contributor from any declarator position,
/// initialized or not.
fn script_has_lexical_var_collision(items: &[TopLevelItem]) -> bool {
    intersects(
        &script_top_level_lexically_declared_names(items),
        &script_var_declared_names(items),
    )
}

fn first_block_index_with_lexical_var_collision(items: &[TopLevelItem]) -> Option<usize> {
    items.iter().position(|item| match item {
        TopLevelItem::Block(block) => intersects(
            &block_lexically_declared_names(block),
            &block_var_declared_names(block),
        ),
        TopLevelItem::Lexical(_) => false,
    })
}

/// Project evidence-order decision across the tiers this oracle
/// independently derives. Tier 1 (binding-local) and Tier 3 (`EE-36-R01`)
/// are inherited unchanged (see `EVIDENCE_TIER_ORDER`); this successor
/// derives only Tier 2a, Tier 2b, and Tier 4, in project evidence-tier
/// order.
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

/// The literal frozen fact for the mandatory Tier-1-vs-Block race:
/// `{ let a; var a=1, if=2; }` reaches existing Tier-1 `EE-04-R08`
/// (an escaped ReservedWord `BindingIdentifier`) rather than this leaf's own
/// Block `EE-14-R02` (from the first declarator `a` colliding with `let a`).
/// Tier 1 is inherited, unrederived authority (see `EVIDENCE_TIER_ORDER`);
/// this struct pins the composed outcome as a literal fact rather than
/// deriving it through `expected_primary_rule_id`, so that decimal
/// initializer presence on the *first* declarator cannot be mistaken for
/// having caused the Tier-1 win.
struct Tier1RaceFixture {
    source: &'static str,
    primary_rule_id: &'static str,
    primary_subject: ExpectedAnchor,
    lower_priority_rule_id: &'static str,
    lower_priority_subject: ExpectedAnchor,
}

const TIER1_RACE: Tier1RaceFixture = Tier1RaceFixture {
    source: "{ let a; var a=1, \\u0069f=2; }",
    primary_rule_id: "EE-04-R08",
    primary_subject: ExpectedAnchor::new(18, 25, "\\u0069f"),
    lower_priority_rule_id: "EE-14-R02",
    lower_priority_subject: ExpectedAnchor::new(13, 14, "a"),
};

#[test]
fn selected_grammar_and_envelope_are_exact() {
    assert_eq!(
        SELECTED_BLOCK_VAR_STATEMENT_GRAMMAR,
        "var SelectedBlockVarDeclaration ( , SelectedBlockVarDeclaration )* ;"
    );
    assert_eq!(
        SELECTED_BLOCK_VAR_DECLARATION_GRAMMAR,
        "SelectedBindingIdentifier | SelectedBindingIdentifier = SelectedDecimalInteger"
    );
    assert_eq!(SELECTED_DECIMAL_INTEGER_GRAMMAR, "0 | [1-9][0-9]*");
    assert_eq!(SELECTED_LIST_CARDINALITY, "1..N");
    assert_eq!(SELECTED_PARSE_GOAL, "Script");
    assert!(!SELECTED_IS_STRICT);
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert_ne!(POSITIVE_LIFECYCLE, "Qualified");
    assert!(!CORRESPONDENCE_WIDENING_REQUIRED);
    assert_eq!(
        EVIDENCE_TIER_ORDER,
        ["EE-14-R01", "EE-14-R02", "EE-36-R01", "EE-36-R02"]
    );
    assert!(!EVIDENCE_ORDER_POLICY_NOTE.is_empty());
    assert!(EVIDENCE_ORDER_POLICY_NOTE.contains("Frontend Analysis project"));
}

#[test]
fn decimal_leaf_is_exactly_zero_or_nonzero_ascii_decimal_digits() {
    for accepted in ["0", "1", "9", "10", "42", "12345"] {
        assert!(is_selected_decimal_integer(accepted));
    }
    for rejected in ["", "00", "01", "1_0", "1.0", "1e2", "1n", "+1", "-1"] {
        assert!(!is_selected_decimal_integer(rejected));
    }
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
    assert_eq!(FIXTURES.len(), 46);
}

#[test]
fn all_expected_anchors_are_valid_utf8_source_anchors_and_slice_to_their_fragment() {
    for (index, fixture) in FIXTURES.iter().enumerate() {
        let source = SourceText::new(SourceId::new(index as u64 + 1), fixture.source.to_owned());

        for item in fixture.top_level_items {
            match item {
                TopLevelItem::Lexical(fact) => validate_anchor(&source, fact.authored),
                TopLevelItem::Block(block) => {
                    for block_item in block.items {
                        match block_item {
                            BlockItem::Lexical(fact) => {
                                validate_anchor(&source, fact.authored);
                                assert!(fact.decimal_rhs.is_none());
                            }
                            BlockItem::VarStatement(declarators) => {
                                for declarator in *declarators {
                                    validate_anchor(&source, declarator.authored);
                                    if let Some(rhs) = declarator.decimal_rhs {
                                        validate_anchor(&source, rhs);
                                        assert!(rhs.start >= declarator.authored.end);
                                        assert!(is_selected_decimal_integer(rhs.fragment));
                                    }
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

    let tier1_source = SourceText::new(SourceId::new(9_001), TIER1_RACE.source.to_owned());
    validate_anchor(&tier1_source, TIER1_RACE.primary_subject);
    validate_anchor(&tier1_source, TIER1_RACE.lower_priority_subject);
}

#[test]
fn every_positive_fixture_carries_at_least_one_selected_decimal_initializer() {
    for fixture in FIXTURES {
        if !matches!(fixture.expected, ExpectedDisposition::AcceptedIncomplete) {
            continue;
        }
        let has_initializer = fixture.top_level_items.iter().any(|item| {
            let TopLevelItem::Block(block) = item else {
                return false;
            };
            block_declarators(block)
                .iter()
                .any(|fact| fact.decimal_rhs.is_some())
        });
        assert!(
            has_initializer,
            "{} must carry at least one selected decimal initializer (the \
             initializer-free predecessor family is #693/#696's concern, not #697's)",
            fixture.id
        );
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
            continue;
        }

        let derived = expected_primary_rule_id(fixture.top_level_items);
        match fixture.expected {
            ExpectedDisposition::AcceptedIncomplete => {
                assert_eq!(derived, None, "{} must not derive any rule", fixture.id);
            }
            ExpectedDisposition::Ee14R01 { .. } => {
                assert_eq!(derived, Some("EE-14-R01"), "{}", fixture.id);
            }
            ExpectedDisposition::Ee14R02 { .. } => {
                assert_eq!(derived, Some("EE-14-R02"), "{}", fixture.id);
            }
            ExpectedDisposition::Ee36R02 { .. } => {
                assert_eq!(derived, Some("EE-36-R02"), "{}", fixture.id);
            }
            ExpectedDisposition::SyntaxRejected { .. }
            | ExpectedDisposition::UnsupportedCoverage(_) => unreachable!(),
        }
    }
}

#[test]
fn per_declarator_initializer_presence_is_independent_not_statement_wide() {
    // Falsifies: "initializer applies statement-wide", "only first
    // declarator initializer matters", "only last declarator initializer
    // matters", and "two-declarator cap".
    let cases: &[(&str, &[bool])] = &[
        ("block-first-declarator-initialized", &[true, false]),
        ("block-final-declarator-initialized", &[false, true]),
        ("block-both-declarators-initialized", &[true, true]),
        (
            "block-mixed-first-and-final-initialized-three",
            &[true, false, true],
        ),
        (
            "block-interior-declarator-initialized-three",
            &[false, true, false],
        ),
        (
            "block-three-declarators-all-initialized",
            &[true, true, true],
        ),
    ];
    for (id, expected_presence) in cases {
        let fixture = fixture(id);
        let TopLevelItem::Block(block) = fixture.top_level_items[0] else {
            panic!("{id} must be a single Block");
        };
        let declarators = block_declarators(&block);
        assert_eq!(declarators.len(), expected_presence.len(), "{id}");
        for (declarator, expected) in declarators.iter().zip(expected_presence.iter()) {
            assert_eq!(declarator.decimal_rhs.is_some(), *expected, "{id}");
        }
    }
    // The three-declarator witness defeats a hard-coded two-declarator cap.
    assert_eq!(
        fixture("block-three-declarators-all-initialized")
            .top_level_items
            .len(),
        1
    );
}

#[test]
fn block_ee14_r02_reaches_first_interior_and_final_initialized_declarator_positions() {
    for (id, expected_primary) in [
        (
            "block-first-declarator-collision-initialized",
            ExpectedAnchor::new(13, 14, "a"),
        ),
        (
            "block-second-declarator-collision-initialized",
            ExpectedAnchor::new(17, 18, "b"),
        ),
        (
            "block-interior-declarator-collision-initialized",
            ExpectedAnchor::new(15, 16, "b"),
        ),
        (
            "block-final-declarator-collision-initialized",
            ExpectedAnchor::new(21, 22, "c"),
        ),
    ] {
        let fixture = fixture(id);
        assert_eq!(
            expected_primary_rule_id(fixture.top_level_items),
            Some("EE-14-R02"),
            "{id}"
        );
        assert_eq!(
            fixture.expected,
            ExpectedDisposition::Ee14R02 {
                primary: expected_primary
            },
            "{id}"
        );
    }
}

#[test]
fn decimal_initializer_never_becomes_bound_name_var_declared_name_or_collision_evidence() {
    for fixture in FIXTURES {
        let primary = match fixture.expected {
            ExpectedDisposition::Ee14R01 { primary }
            | ExpectedDisposition::Ee14R02 { primary }
            | ExpectedDisposition::Ee36R02 { primary } => primary,
            _ => continue,
        };
        // The primary evidence fragment must be an identifier occurrence
        // (direct or escaped), never a decimal digit sequence: a wrong
        // oracle that let the initializer RHS become collision evidence
        // would produce a primary fragment satisfying the decimal grammar.
        assert!(
            !is_selected_decimal_integer(primary.fragment),
            "{} primary evidence must not be a decimal RHS",
            fixture.id
        );

        for item in fixture.top_level_items {
            let TopLevelItem::Block(block) = item else {
                continue;
            };
            for declarator in block_declarators(block) {
                if let Some(rhs) = declarator.decimal_rhs {
                    assert_ne!(
                        (rhs.start, rhs.end),
                        (primary.start, primary.end),
                        "{} primary evidence must not be a declarator's decimal RHS anchor",
                        fixture.id
                    );
                }
            }
        }
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
        fixture_id: "block-second-declarator-collision-initialized",
        lexical_binding: ExpectedAnchor::new(6, 7, "b"),
        var_binding: ExpectedAnchor::new(17, 18, "b"),
    },
    EvidenceTripleFixture {
        fixture_id: "block-lexical-after-list-collision-initialized",
        lexical_binding: ExpectedAnchor::new(19, 20, "b"),
        var_binding: ExpectedAnchor::new(10, 11, "b"),
    },
];

#[test]
fn lexical_before_and_after_list_evidence_triples_remain_independent_of_initializer_content() {
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
}

#[test]
fn repeated_var_contributors_and_escaped_semantic_equality_do_not_collapse_authored_multiplicity() {
    let direct = fixture("block-repeated-direct-var-initialized");
    let TopLevelItem::Block(direct_block) = direct.top_level_items[0] else {
        panic!("fixture's only item must be a Block");
    };
    assert_eq!(block_var_declared_names(&direct_block), vec!["a", "a"]);
    assert!(!any_block_has_lexical_var_collision(direct.top_level_items));
    assert_eq!(direct.expected, ExpectedDisposition::AcceptedIncomplete);
    // First contributor is initialized, second is bare: repeated names do
    // not collapse initializer presence either.
    let declarators = block_declarators(&direct_block);
    assert!(declarators[0].decimal_rhs.is_some());
    assert!(declarators[1].decimal_rhs.is_none());

    let escaped = fixture("block-repeated-direct-escaped-var-initialized");
    let TopLevelItem::Block(escaped_block) = escaped.top_level_items[0] else {
        panic!("fixture's only item must be a Block");
    };
    let declarators = block_declarators(&escaped_block);
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
fn script_ee36_r02_propagation_covers_non_first_initialized_declarator_and_lexical_after_block() {
    let non_first = fixture("script-propagation-non-first-initialized-declarator");
    let TopLevelItem::Block(non_first_block) = non_first.top_level_items[1] else {
        panic!("fixture's second item must be a Block");
    };
    // The colliding contributor ("b") is the *second* declarator, and it is
    // the initialized one, proving a "first declarator only" or
    // "uninitialized declarators only" propagation model would miss this.
    assert_eq!(block_var_declared_names(&non_first_block), vec!["a", "b"]);
    let declarators = block_declarators(&non_first_block);
    assert!(declarators[1].decimal_rhs.is_some());
    assert_eq!(
        expected_primary_rule_id(non_first.top_level_items),
        Some("EE-36-R02")
    );
    assert_eq!(
        non_first.expected,
        ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(17, 18, "b")
        }
    );

    let lexical_after = fixture("script-propagation-lexical-after-block-initialized");
    assert_eq!(
        expected_primary_rule_id(lexical_after.top_level_items),
        Some("EE-36-R02")
    );
    assert_eq!(
        lexical_after.expected,
        ExpectedDisposition::Ee36R02 {
            primary: ExpectedAnchor::new(21, 22, "b")
        }
    );
}

#[test]
fn escaped_first_and_later_declarator_collisions_use_semantic_equality_not_spelling() {
    let first = fixture("block-escaped-first-declarator-collision-initialized");
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

    let later = fixture("block-escaped-later-declarator-collision-initialized");
    assert_eq!(
        expected_primary_rule_id(later.top_level_items),
        Some("EE-14-R02")
    );
    assert_eq!(
        later.expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(17, 23, "\\u0061")
        }
    );
}

#[test]
fn no_unicode_normalization_preserves_distinct_code_point_sequences_with_initializer() {
    let fixture = fixture("block-no-unicode-normalization-initialized");
    let TopLevelItem::Block(block) = fixture.top_level_items[0] else {
        panic!("fixture's only item must be a Block");
    };
    let declarators = block_declarators(&block);
    assert_ne!(
        declarators[0].semantic_code_points,
        declarators[1].semantic_code_points
    );
    assert_ne!(declarators[0].semantic_name, declarators[1].semantic_name);
    assert!(declarators[0].decimal_rhs.is_some());
    assert!(declarators[1].decimal_rhs.is_some());
    assert!(!any_block_has_lexical_var_collision(
        fixture.top_level_items
    ));
    assert_eq!(fixture.expected, ExpectedDisposition::AcceptedIncomplete);
}

#[test]
fn same_tier_sibling_blocks_select_first_qualifying_block_in_source_order_with_initializer() {
    let fixture = fixture("same-tier-sibling-block-source-order-initialized");
    let TopLevelItem::Block(first_block) = fixture.top_level_items[0] else {
        panic!("fixture's first item must be a Block");
    };
    let TopLevelItem::Block(second_block) = fixture.top_level_items[1] else {
        panic!("fixture's second item must be a Block");
    };

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
        Some(0)
    );
    assert_eq!(
        fixture.expected,
        ExpectedDisposition::Ee14R02 {
            primary: ExpectedAnchor::new(17, 18, "a"),
        }
    );
}

#[test]
fn evidence_tier_outranks_cross_block_source_position_with_initializer_present() {
    let fixture = fixture("cross-tier-priority-over-source-position-initialized");
    let TopLevelItem::Block(first_block) = fixture.top_level_items[0] else {
        panic!("fixture's first item must be a Block");
    };
    let TopLevelItem::Block(second_block) = fixture.top_level_items[1] else {
        panic!("fixture's second item must be a Block");
    };

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

    assert_eq!(
        expected_primary_rule_id(fixture.top_level_items),
        Some("EE-14-R01"),
        "a later Tier-2a error must outrank an earlier initializer-bearing Tier-2b error"
    );
    assert_eq!(
        fixture.expected,
        ExpectedDisposition::Ee14R01 {
            primary: ExpectedAnchor::new(37, 38, "b"),
        }
    );
}

#[test]
fn tier1_ee04_r08_outranks_block_ee14_r02_even_with_initializer_present() {
    assert_eq!(TIER1_RACE.primary_rule_id, "EE-04-R08");
    assert_eq!(TIER1_RACE.lower_priority_rule_id, "EE-14-R02");
    assert_eq!(
        TIER1_RACE.primary_subject,
        ExpectedAnchor::new(18, 25, "\\u0069f")
    );
    assert_eq!(
        TIER1_RACE.lower_priority_subject,
        ExpectedAnchor::new(13, 14, "a")
    );
    // The first declarator ("a=1") that collides with `let a` carries the
    // decimal initializer; the escaped-reserved-word second declarator
    // ("if=2") is what a wrong oracle might imagine "wins" because it
    // is later in source order. Neither declarator's initializer content
    // participates in tier selection: Tier 1 wins because of the tier,
    // not the source position or the initializer.
    assert!(TIER1_RACE.source.contains("a=1"));
    assert!(TIER1_RACE.source.contains("f=2"));
    assert!(TIER1_RACE.primary_subject.start > TIER1_RACE.lower_priority_subject.start);
}

#[test]
fn decimal_numeric_neighbors_remain_unsupported_coverage() {
    for id in [
        "numeric-neighbor-leading-zero-unsupported",
        "numeric-neighbor-separator-unsupported",
        "numeric-neighbor-fraction-unsupported",
        "numeric-neighbor-exponent-unsupported",
        "numeric-neighbor-bigint-unsupported",
        "numeric-neighbor-unary-plus-unsupported",
        "numeric-neighbor-unary-minus-unsupported",
    ] {
        assert_eq!(
            fixture(id).expected,
            ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NumericNeighbor),
            "{id}"
        );
        assert!(fixture(id).top_level_items.is_empty(), "{id}");
    }
}

#[test]
fn non_decimal_initializer_families_remain_unsupported_coverage() {
    for id in [
        "non-decimal-initializer-boolean-unsupported",
        "non-decimal-initializer-null-unsupported",
        "non-decimal-initializer-this-unsupported",
        "non-decimal-initializer-string-unsupported",
        "non-decimal-initializer-identifier-reference-unsupported",
        "non-decimal-initializer-escaped-identifier-reference-unsupported",
        "non-decimal-initializer-escaped-reserved-identifier-unsupported",
    ] {
        assert_eq!(
            fixture(id).expected,
            ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NonDecimalInitializer),
            "{id}"
        );
        assert!(fixture(id).top_level_items.is_empty(), "{id}");
    }
}

#[test]
fn authored_semicolon_is_mandatory_and_non_eof_asi_remains_unsupported() {
    for id in [
        "non-eof-asi-single-declarator-unsupported",
        "non-eof-asi-two-declarators-unsupported",
    ] {
        assert_eq!(
            fixture(id).expected,
            ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::NonEofAsi),
            "{id}"
        );
        assert!(fixture(id).top_level_items.is_empty(), "{id}");
        assert!(!fixture(id).source.trim_end().ends_with(';'));
    }
    // Positive fixtures all end with the authored `; }` terminator shape.
    for fixture in FIXTURES {
        if matches!(fixture.expected, ExpectedDisposition::AcceptedIncomplete) {
            assert!(
                fixture.source.contains("; }"),
                "{} must retain an authored semicolon before the closing brace",
                fixture.id
            );
        }
    }
}

#[test]
fn comment_trivia_remains_unsupported() {
    for id in [
        "comment-before-declarator-unsupported",
        "comment-inside-initializer-unsupported",
        "comment-after-initializer-unsupported",
    ] {
        assert_eq!(
            fixture(id).expected,
            ExpectedDisposition::UnsupportedCoverage(UnsupportedReason::CommentTrivia),
            "{id}"
        );
        assert!(fixture(id).top_level_items.is_empty(), "{id}");
        assert!(fixture(id).source.contains("/*"));
    }
}

#[test]
fn transactionality_never_commits_a_partial_initialized_prefix_on_later_failure() {
    for id in [
        "transactionality-trailing-comma-no-declarator-unsupported",
        "transactionality-trailing-equals-no-rhs-unsupported",
        "transactionality-later-boolean-initializer-unsupported",
        "transactionality-later-fraction-initializer-unsupported",
        "transactionality-later-malformed-binding-syntax-rejected",
    ] {
        let fixture = fixture(id);
        assert!(
            fixture.top_level_items.is_empty(),
            "{id} must commit no partial selected-success state despite a valid, \
             decimal-initialized earlier prefix"
        );
        assert_ne!(
            fixture.expected,
            ExpectedDisposition::AcceptedIncomplete,
            "{id} must not be accepted"
        );
        // Every one of these fixtures has a syntactically valid, correctly
        // decimal-initialized `a=1` prefix before the failure: this is the
        // load-bearing part of the transactionality proof.
        assert!(fixture.source.contains("a=1"), "{id}");
    }
}

#[test]
fn classification_firewalls_keep_syntax_rejection_and_unsupported_coverage_distinct() {
    assert_eq!(
        fixture("transactionality-later-malformed-binding-syntax-rejected").expected,
        ExpectedDisposition::SyntaxRejected {
            subject: ExpectedAnchor::new(12, 16, "\\u{}"),
        }
    );
    // `{ var a=1,b=1.0; }` remains UnsupportedCoverage, not SyntaxRejected:
    // a richer numeric literal is valid-but-unselected ECMAScript, not
    // invalid syntax.
    assert_ne!(
        fixture("transactionality-later-fraction-initializer-unsupported").expected,
        ExpectedDisposition::SyntaxRejected {
            subject: ExpectedAnchor::new(0, 0, "")
        }
    );
    // `{ var a=1,b=2 }` is valid ECMAScript through non-EOF ASI; it must
    // remain UnsupportedCoverage, never SyntaxRejected.
    assert_ne!(
        fixture("non-eof-asi-two-declarators-unsupported").expected,
        ExpectedDisposition::SyntaxRejected {
            subject: ExpectedAnchor::new(0, 0, "")
        }
    );
}

#[test]
fn top_level_variable_statement_parser_is_not_reusable_wholesale_for_block_placement() {
    // This leaf's selected Block var initializer family is strictly
    // narrower than the current top-level selected var initializer family
    // (decimal only, no IdentifierReference/Boolean/null/this/String, and
    // authored-semicolon-only rather than EOF-only ASI). If a future
    // implementation reused the top-level `VariableStatement` parser
    // unchanged for Block placement, it would accidentally accept some of
    // these exact fixtures; this oracle firewalls every one of them as
    // UnsupportedCoverage so that wrong reuse fails these tests.
    let firewalled_by_wholesale_reuse = [
        "non-decimal-initializer-identifier-reference-unsupported",
        "non-decimal-initializer-boolean-unsupported",
        "non-decimal-initializer-null-unsupported",
        "non-decimal-initializer-this-unsupported",
        "non-decimal-initializer-string-unsupported",
        "non-decimal-initializer-escaped-identifier-reference-unsupported",
        "non-eof-asi-single-declarator-unsupported",
        "non-eof-asi-two-declarators-unsupported",
    ];
    for id in firewalled_by_wholesale_reuse {
        assert!(
            matches!(
                fixture(id).expected,
                ExpectedDisposition::UnsupportedCoverage(_)
            ),
            "{id} must remain outside selected Block var coverage"
        );
    }
}

#[test]
fn historical_and_sibling_authority_remains_immutable_and_additive() {
    assert!(
        HISTORICAL_BLOCK_VAR_MULTI_DECLARATOR
            .contains("SELECTED_LIST_CARDINALITY: &str = \"1..N\"")
    );
    assert!(
        HISTORICAL_BLOCK_VAR_MULTI_DECLARATOR
            .contains("var SelectedBindingIdentifier ( , SelectedBindingIdentifier )* ;")
    );
    assert!(HISTORICAL_BLOCK_VAR_MULTI_DECLARATOR.contains("{ var x, y = 1; }"));
    assert!(
        HISTORICAL_TOP_LEVEL_DECIMAL_INITIALIZER
            .contains("SELECTED_DECIMAL_INTEGER_GRAMMAR: &str = \"0 | [1-9][0-9]*\"")
    );
    assert!(HISTORICAL_TOP_LEVEL_DECIMAL_INITIALIZER.contains("{ var a=1,b; }"));
    assert!(HISTORICAL_CORRESPONDENCE.contains("VisibleSelectedLexicalBinding"));
    assert!(HISTORICAL_CORRESPONDENCE.contains("SameSourceSelectedVarNameContributors"));
    assert!(HISTORICAL_CORRESPONDENCE.contains("NoSelectedSameSourceContributor"));
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
            "Block var decimal-initializer oracle must remain candidate-independent: found {forbidden}"
        );
    }
}
