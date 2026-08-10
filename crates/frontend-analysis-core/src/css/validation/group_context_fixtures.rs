//! Independent group-rule context/order/resource gold fixtures for the #168
//! nested group-rule producer capability.
//!
//! Kept structurally separate from the fixed #166 16-fixture inventory
//! (`context_fixtures.rs`): this module extends the shared #166 context-gold
//! vocabulary (`context_gold.rs`) with the finite #168 group-rule kinds,
//! authored independently from the approved #168 architecture record
//! (comment `5235527147` on Issue #168), never derived from production
//! `CssParserGroupRuleKind`, `CssParserContextKind`, a production parser,
//! external parser, CSSOM, or browser output. Production code must never
//! import this module.

use super::context_gold::{
    ContextGoldDeclarationItem, ContextGoldFixture, ContextGoldGroup, ContextGoldGroupRuleKind,
    ContextGoldKind, ContextGoldRecord, ContextGoldResourceExpectation, ContextGoldResourceKind,
    ContextGoldTermination, ContextGoldUnsupportedAtRule,
};
use super::gold::GoldRange;

fn range(start: usize, end: usize) -> GoldRange {
    GoldRange::new(start, end)
}

fn decl(
    item_ordinal: usize,
    run_ordinal: usize,
    span: (usize, usize),
) -> ContextGoldDeclarationItem {
    ContextGoldDeclarationItem {
        item_ordinal,
        run_ordinal,
        span: range(span.0, span.1),
    }
}

#[allow(clippy::too_many_arguments)]
fn qualified_record(
    id: usize,
    parent: Option<usize>,
    item_ordinal: usize,
    nearest_qualified_ancestor: Option<usize>,
    header: (usize, usize),
    block_opener: (usize, usize),
    body: (usize, usize),
    termination: ContextGoldTermination,
    declarations: Vec<ContextGoldDeclarationItem>,
) -> ContextGoldRecord {
    ContextGoldRecord {
        id,
        parent,
        nearest_qualified_ancestor,
        item_ordinal,
        kind: ContextGoldKind::QualifiedRuleBlock,
        at_keyword: None,
        header: range(header.0, header.1),
        block_opener: range(block_opener.0, block_opener.1),
        body: range(body.0, body.1),
        termination,
        declarations,
    }
}

#[allow(clippy::too_many_arguments)]
fn group_record(
    id: usize,
    parent: Option<usize>,
    item_ordinal: usize,
    nearest_qualified_ancestor: Option<usize>,
    group_kind: ContextGoldGroupRuleKind,
    at_keyword: (usize, usize),
    header: (usize, usize),
    block_opener: (usize, usize),
    body: (usize, usize),
    termination: ContextGoldTermination,
    declarations: Vec<ContextGoldDeclarationItem>,
) -> ContextGoldRecord {
    ContextGoldRecord {
        id,
        parent,
        nearest_qualified_ancestor,
        item_ordinal,
        kind: ContextGoldKind::GroupRuleBlock(group_kind),
        at_keyword: Some(range(at_keyword.0, at_keyword.1)),
        header: range(header.0, header.1),
        block_opener: range(block_opener.0, block_opener.1),
        body: range(body.0, body.1),
        termination,
        declarations,
    }
}

fn right_curly(range_tuple: (usize, usize)) -> ContextGoldTermination {
    ContextGoldTermination::AuthoredRightCurly(range(range_tuple.0, range_tuple.1))
}

fn unsupported_at_rule(
    complete: (usize, usize),
    at_keyword: (usize, usize),
    owner_context: usize,
    item_ordinal: usize,
) -> ContextGoldUnsupportedAtRule {
    ContextGoldUnsupportedAtRule {
        complete: range(complete.0, complete.1),
        at_keyword: range(at_keyword.0, at_keyword.1),
        owner_context,
        item_ordinal,
    }
}

// -- 1-3: `@media` bounded subset (empty / known type / case-insensitive) ---

/// `@media {}` with an empty prelude.
fn media_empty_prelude() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 14),
        right_curly((14, 15)),
        vec![],
    );
    let media = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Media,
        (2, 8),
        (2, 8),
        (8, 9),
        (9, 13),
        right_curly((13, 14)),
        vec![decl(0, 0, (9, 13))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-MEDIA-EMPTY-PRELUDE-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_101,
        source: "a{@media{p:v;}}",
        byte_len: 15,
        contexts: vec![outer, media],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

/// `@media screen {}`.
fn media_screen_prelude() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 21),
        right_curly((21, 22)),
        vec![],
    );
    let media = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Media,
        (2, 8),
        (2, 15),
        (15, 16),
        (16, 20),
        right_curly((20, 21)),
        vec![decl(0, 0, (16, 20))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-MEDIA-SCREEN-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_102,
        source: "a{@media screen{p:v;}}",
        byte_len: 22,
        contexts: vec![outer, media],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

/// `@media PRINT {}`: the known media-type match is ASCII-case-insensitive.
fn media_print_case_insensitive() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 20),
        right_curly((20, 21)),
        vec![],
    );
    let media = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Media,
        (2, 8),
        (2, 14),
        (14, 15),
        (15, 19),
        right_curly((19, 20)),
        vec![decl(0, 0, (15, 19))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-MEDIA-PRINT-CASE-INSENSITIVE-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_103,
        source: "a{@media PRINT{p:v;}}",
        byte_len: 21,
        contexts: vec![outer, media],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 4-5: `@supports` bounded subset -----------------------------------------

/// `@supports (display:grid) {}`.
fn supports_basic() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 32),
        right_curly((32, 33)),
        vec![],
    );
    let supports = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Supports,
        (2, 11),
        (2, 26),
        (26, 27),
        (27, 31),
        right_curly((31, 32)),
        vec![decl(0, 0, (27, 31))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-SUPPORTS-DECLARATION-SHAPED-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_104,
        source: "a{@supports (display:grid){p:v;}}",
        byte_len: 33,
        contexts: vec![outer, supports],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

/// `@supports (x:) {}`: an empty value is structurally permitted.
fn supports_empty_value() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 22),
        right_curly((22, 23)),
        vec![],
    );
    let supports = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Supports,
        (2, 11),
        (2, 16),
        (16, 17),
        (17, 21),
        right_curly((21, 22)),
        vec![decl(0, 0, (17, 21))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-SUPPORTS-EMPTY-VALUE-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_105,
        source: "a{@supports (x:){p:v;}}",
        byte_len: 23,
        contexts: vec![outer, supports],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 6: `@container` name-only form ------------------------------------------

/// `@container sidebar {}`.
fn container_name_only() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 26),
        right_curly((26, 27)),
        vec![],
    );
    let container = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Container,
        (2, 12),
        (2, 20),
        (20, 21),
        (21, 25),
        right_curly((25, 26)),
        vec![decl(0, 0, (21, 25))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-CONTAINER-NAME-ONLY-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_106,
        source: "a{@container sidebar{p:v;}}",
        byte_len: 27,
        contexts: vec![outer, container],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 7-8: `@layer` anonymous / dotted-name -----------------------------------

/// `@layer {}` (anonymous).
fn layer_anonymous() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 14),
        right_curly((14, 15)),
        vec![],
    );
    let layer = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Layer,
        (2, 8),
        (2, 8),
        (8, 9),
        (9, 13),
        right_curly((13, 14)),
        vec![decl(0, 0, (9, 13))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-LAYER-ANONYMOUS-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_107,
        source: "a{@layer{p:v;}}",
        byte_len: 15,
        contexts: vec![outer, layer],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

/// `@layer a.b.c {}` (dotted layer name).
fn layer_dotted_name() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 20),
        right_curly((20, 21)),
        vec![],
    );
    let layer = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Layer,
        (2, 8),
        (2, 14),
        (14, 15),
        (15, 19),
        right_curly((19, 20)),
        vec![decl(0, 0, (15, 19))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-LAYER-DOTTED-NAME-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_108,
        source: "a{@layer a.b.c{p:v;}}",
        byte_len: 21,
        contexts: vec![outer, layer],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 9-10: `@scope` empty prelude / `(&)` ------------------------------------

/// `@scope {}` (empty prelude).
fn scope_empty_prelude() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 14),
        right_curly((14, 15)),
        vec![],
    );
    let scope = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Scope,
        (2, 8),
        (2, 8),
        (8, 9),
        (9, 13),
        right_curly((13, 14)),
        vec![decl(0, 0, (9, 13))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-SCOPE-EMPTY-PRELUDE-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_109,
        source: "a{@scope{p:v;}}",
        byte_len: 15,
        contexts: vec![outer, scope],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

/// `@scope (&) {}`.
fn scope_amp_shape() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 18),
        right_curly((18, 19)),
        vec![],
    );
    let scope = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Scope,
        (2, 8),
        (2, 12),
        (12, 13),
        (13, 17),
        right_curly((17, 18)),
        vec![decl(0, 0, (13, 17))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-SCOPE-AMP-SHAPE-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_110,
        source: "a{@scope (&){p:v;}}",
        byte_len: 19,
        contexts: vec![outer, scope],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 11: declaration before / inside / after a group -------------------------

/// `a{p0:v0;@media{p1:v1;}p2:v2;}`: outer run 0, group item, outer run 1.
fn declaration_before_inside_after_group() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 28),
        right_curly((28, 29)),
        vec![decl(0, 0, (2, 8)), decl(2, 1, (22, 28))],
    );
    let media = group_record(
        1,
        Some(0),
        1,
        Some(0),
        ContextGoldGroupRuleKind::Media,
        (8, 14),
        (8, 14),
        (14, 15),
        (15, 21),
        right_curly((21, 22)),
        vec![decl(0, 0, (15, 21))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-DECLARATION-BEFORE-INSIDE-AFTER-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 91_111,
        source: "a{p0:v0;@media{p1:v1;}p2:v2;}",
        byte_len: 29,
        contexts: vec![outer, media],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 12: qualified-rule child inside a supported group -----------------------

/// `a{@media{b{p:v;}}}`: a plain qualified-rule `b{...}` nested inside a
/// supported `@media` group. `b`'s nearest qualified ancestor is the
/// top-level `a`, inherited through the interposing group.
fn qualified_rule_child_inside_group() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 17),
        right_curly((17, 18)),
        vec![],
    );
    let media = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Media,
        (2, 8),
        (2, 8),
        (8, 9),
        (9, 16),
        right_curly((16, 17)),
        vec![],
    );
    let b = qualified_record(
        2,
        Some(1),
        0,
        Some(0),
        (9, 10),
        (10, 11),
        (11, 15),
        right_curly((15, 16)),
        vec![decl(0, 0, (11, 15))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-QUALIFIED-CHILD-INSIDE-MEDIA-001",
        group: ContextGoldGroup::RecursiveNesting,
        source_id: 91_112,
        source: "a{@media{b{p:v;}}}",
        byte_len: 18,
        contexts: vec![outer, media, b],
        expected_context_record_count: 3,
        expected_peak_context_depth: 3,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 13: supported group nested inside supported group -----------------------

/// `a{@media{@layer{p:v;}}}`: `@layer` nested inside `@media`. Both group
/// contexts inherit the same nearest qualified ancestor (`a`).
fn group_nested_inside_group() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 22),
        right_curly((22, 23)),
        vec![],
    );
    let media = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Media,
        (2, 8),
        (2, 8),
        (8, 9),
        (9, 21),
        right_curly((21, 22)),
        vec![],
    );
    let layer = group_record(
        2,
        Some(1),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Layer,
        (9, 15),
        (9, 15),
        (15, 16),
        (16, 20),
        right_curly((20, 21)),
        vec![decl(0, 0, (16, 20))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-NESTED-INSIDE-GROUP-001",
        group: ContextGoldGroup::RecursiveNesting,
        source_id: 91_113,
        source: "a{@media{@layer{p:v;}}}",
        byte_len: 23,
        contexts: vec![outer, media, layer],
        expected_context_record_count: 3,
        expected_peak_context_depth: 3,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 14: qualified-rule child under nested groups -----------------------------

/// `a{@media{@layer{b{p:v;}}}}`: exact nearest-qualified-ancestor derivation
/// through two interposing group levels.
fn qualified_child_under_nested_groups() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 25),
        right_curly((25, 26)),
        vec![],
    );
    let media = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Media,
        (2, 8),
        (2, 8),
        (8, 9),
        (9, 24),
        right_curly((24, 25)),
        vec![],
    );
    let layer = group_record(
        2,
        Some(1),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Layer,
        (9, 15),
        (9, 15),
        (15, 16),
        (16, 23),
        right_curly((23, 24)),
        vec![],
    );
    let b = qualified_record(
        3,
        Some(2),
        0,
        Some(0),
        (16, 17),
        (17, 18),
        (18, 22),
        right_curly((22, 23)),
        vec![decl(0, 0, (18, 22))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-QUALIFIED-CHILD-UNDER-NESTED-GROUPS-001",
        group: ContextGoldGroup::RecursiveNesting,
        source_id: 91_114,
        source: "a{@media{@layer{b{p:v;}}}}",
        byte_len: 26,
        contexts: vec![outer, media, layer, b],
        expected_context_record_count: 4,
        expected_peak_context_depth: 4,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 15-16: unknown at-rule (block / semicolon) resumes parent ---------------

/// An unknown block at-rule consumes exactly one unsupported direct item;
/// parent parsing resumes afterward.
fn unknown_block_at_rule_resumes_parent() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 28),
        right_curly((28, 29)),
        vec![decl(0, 0, (2, 8)), decl(2, 1, (22, 28))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-UNKNOWN-BLOCK-AT-RULE-RESUMES-001",
        group: ContextGoldGroup::Malformed,
        source_id: 91_115,
        source: "a{p0:v0;@unknown{x:y;}p1:v1;}",
        byte_len: 29,
        contexts: vec![outer],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![unsupported_at_rule((8, 22), (8, 16), 0, 1)],
    }
}

/// An unknown semicolon-terminated at-rule consumes exactly one unsupported
/// direct item; parent parsing resumes afterward.
fn unknown_semicolon_at_rule_resumes_parent() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 23),
        right_curly((23, 24)),
        vec![decl(0, 0, (2, 8)), decl(2, 1, (17, 23))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-UNKNOWN-SEMICOLON-AT-RULE-RESUMES-001",
        group: ContextGoldGroup::Malformed,
        source_id: 91_116,
        source: "a{p0:v0;@unknown;p1:v1;}",
        byte_len: 24,
        contexts: vec![outer],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![unsupported_at_rule((8, 17), (8, 16), 0, 1)],
    }
}

// -- 17-19: recognized registry member outside the bounded subset -----------

/// `@media screen and (min-width:1px)` falls outside the bounded #168
/// subset and remains explicit unsupported evidence, not invalidity.
fn media_outside_bounded_subset() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 41),
        right_curly((41, 42)),
        vec![],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-MEDIA-OUTSIDE-SUBSET-001",
        group: ContextGoldGroup::Malformed,
        source_id: 91_117,
        source: "a{@media screen and (min-width:1px){x:y;}}",
        byte_len: 42,
        contexts: vec![outer],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![unsupported_at_rule((2, 41), (2, 8), 0, 0)],
    }
}

/// `@supports not (display:grid)` falls outside the bounded #168 subset
/// (only a bare parenthesized declaration-shaped condition is supported).
fn supports_outside_bounded_subset() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 36),
        right_curly((36, 37)),
        vec![],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-SUPPORTS-OUTSIDE-SUBSET-001",
        group: ContextGoldGroup::Malformed,
        source_id: 91_118,
        source: "a{@supports not (display:grid){x:y;}}",
        byte_len: 37,
        contexts: vec![outer],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![unsupported_at_rule((2, 36), (2, 11), 0, 0)],
    }
}

/// `@container none` uses a reserved container-name keyword and falls
/// outside the bounded #168 subset.
fn container_reserved_name_outside_subset() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 23),
        right_curly((23, 24)),
        vec![],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-CONTAINER-RESERVED-NAME-001",
        group: ContextGoldGroup::Malformed,
        source_id: 91_119,
        source: "a{@container none{x:y;}}",
        byte_len: 24,
        contexts: vec![outer],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![unsupported_at_rule((2, 23), (2, 12), 0, 0)],
    }
}

// -- 20: `@layer` statement form remains unsupported -------------------------

/// `@layer foo;` is a statement, not a group context (#168 explicit
/// exclusion), and remains one unsupported nested direct item.
fn layer_statement_form_remains_unsupported() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 25),
        right_curly((25, 26)),
        vec![decl(0, 0, (2, 8)), decl(2, 1, (19, 25))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-LAYER-STATEMENT-FORM-001",
        group: ContextGoldGroup::Malformed,
        source_id: 91_120,
        source: "a{p0:v0;@layer foo;p1:v1;}",
        byte_len: 26,
        contexts: vec![outer],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![unsupported_at_rule((8, 19), (8, 14), 0, 1)],
    }
}

// -- 21: broader `@scope` selector-boundary form remains unsupported --------

/// `@scope (.a) to (.b)` uses general selector-boundary syntax outside the
/// bounded `(&)`-only #168 subset.
fn scope_broader_form_remains_unsupported() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 27),
        right_curly((27, 28)),
        vec![],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-SCOPE-BROADER-FORM-001",
        group: ContextGoldGroup::Malformed,
        source_id: 91_121,
        source: "a{@scope (.a) to (.b){x:y;}}",
        byte_len: 28,
        contexts: vec![outer],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![unsupported_at_rule((2, 27), (2, 8), 0, 0)],
    }
}

// -- 22: custom-property value containing an at-keyword/block ----------------

/// `--x:@media screen{color:blue;};`: the at-keyword and its block are
/// ordinary declaration-value content for a custom property; group dispatch
/// occurs only at block-content item dispatch, never inside declaration
/// value scanning.
fn custom_property_value_does_not_create_group_context() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 33),
        right_curly((33, 34)),
        vec![decl(0, 0, (2, 33))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-CUSTOM-PROPERTY-AT-KEYWORD-VALUE-001",
        group: ContextGoldGroup::CustomProperty,
        source_id: 91_122,
        source: "a{--x:@media screen{color:blue;};}",
        byte_len: 34,
        contexts: vec![outer],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 23: malformed item before a group does not leak an ordinal or run ------

/// `"color red;"` fails declaration recognition and qualified-rule fallback,
/// materializing no direct item; the following `@media` group is item
/// ordinal 0, not 1.
fn malformed_recovery_before_group_leaks_no_ordinal() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 24),
        right_curly((24, 25)),
        vec![],
    );
    let media = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Media,
        (12, 18),
        (12, 18),
        (18, 19),
        (19, 23),
        right_curly((23, 24)),
        vec![decl(0, 0, (19, 23))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-MALFORMED-BEFORE-GROUP-NO-LEAK-001",
        group: ContextGoldGroup::Malformed,
        source_id: 91_123,
        source: "a{color red;@media{p:v;}}",
        byte_len: 25,
        contexts: vec![outer, media],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 24: identical authored group spelling under distinct contexts ----------

/// Two independent top-level qualified rules each nest an identically
/// spelled `@media{p:v;}` group; both receive distinct `ContextId`s.
fn duplicate_group_spelling_distinct_identities() -> ContextGoldFixture {
    let a = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 14),
        right_curly((14, 15)),
        vec![],
    );
    let media_in_a = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Media,
        (2, 8),
        (2, 8),
        (8, 9),
        (9, 13),
        right_curly((13, 14)),
        vec![decl(0, 0, (9, 13))],
    );
    let b = qualified_record(
        2,
        None,
        1,
        None,
        (15, 16),
        (16, 17),
        (17, 29),
        right_curly((29, 30)),
        vec![],
    );
    let media_in_b = group_record(
        3,
        Some(2),
        0,
        Some(2),
        ContextGoldGroupRuleKind::Media,
        (17, 23),
        (17, 23),
        (23, 24),
        (24, 28),
        right_curly((28, 29)),
        vec![decl(0, 0, (24, 28))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-DUPLICATE-SPELLING-DISTINCT-IDENTITIES-001",
        group: ContextGoldGroup::DuplicateSpelling,
        source_id: 91_124,
        source: "a{@media{p:v;}}b{@media{p:v;}}",
        byte_len: 30,
        contexts: vec![a, media_in_a, b, media_in_b],
        expected_context_record_count: 4,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 25: true EOF while qualified + group ancestry are active ---------------

/// `a{@media{p:v` (unterminated): both the outer qualified-rule context and
/// the inner group context honestly retain `EndOfInput` evidence at the
/// exact same true source-end terminal; the inner unterminated declaration
/// is completed under the already-approved `OmittedAtEndOfInput` rule.
fn true_eof_while_group_ancestry_active() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 12),
        ContextGoldTermination::EndOfInput(range(12, 12)),
        vec![],
    );
    let media = group_record(
        1,
        Some(0),
        0,
        Some(0),
        ContextGoldGroupRuleKind::Media,
        (2, 8),
        (2, 8),
        (8, 9),
        (9, 12),
        ContextGoldTermination::EndOfInput(range(12, 12)),
        vec![decl(0, 0, (9, 12))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-TRUE-EOF-ANCESTRY-001",
        group: ContextGoldGroup::Termination,
        source_id: 91_125,
        source: "a{@media{p:v",
        byte_len: 12,
        contexts: vec![outer, media],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
        unsupported: vec![],
    }
}

// -- 26-27: `PeakContextDepth` / `ContextRecords` refusal on group entry ----

/// A `PeakContextDepth` refusal while attempting to enter a group context:
/// the outer qualified-rule ancestor remains committed with honest
/// `ParserResourceLimit` termination; the refused group never appears (no
/// ID gap, no leaked declaration inside it).
fn peak_context_depth_refusal_on_group_entry() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 9),
        ContextGoldTermination::ParserResourceLimit(range(9, 9)),
        vec![],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-PEAK-CONTEXT-DEPTH-REFUSAL-001",
        group: ContextGoldGroup::ResourceLimit,
        source_id: 91_126,
        source: "a{@media{p:v;}}",
        byte_len: 15,
        contexts: vec![outer],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: Some(ContextGoldResourceExpectation {
            kind: ContextGoldResourceKind::PeakContextDepth,
            limit: 1,
            attempted: 2,
        }),
        unsupported: vec![],
    }
}

/// A `ContextRecords` refusal while attempting to enter a group context: an
/// already-closed sibling `QualifiedRuleBlock` keeps its honest
/// `AuthoredRightCurly` termination; the outer context (still active)
/// retains `ParserResourceLimit` termination; the refused group never
/// appears.
fn context_records_refusal_on_group_entry() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 16),
        ContextGoldTermination::ParserResourceLimit(range(16, 16)),
        vec![],
    );
    let b = qualified_record(
        1,
        Some(0),
        0,
        Some(0),
        (2, 3),
        (3, 4),
        (4, 8),
        right_curly((8, 9)),
        vec![decl(0, 0, (4, 8))],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-CONTEXT-RECORDS-REFUSAL-001",
        group: ContextGoldGroup::ResourceLimit,
        source_id: 91_127,
        source: "a{b{p:v;}@media{q:w;}}",
        byte_len: 22,
        contexts: vec![outer, b],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: Some(ContextGoldResourceExpectation {
            kind: ContextGoldResourceKind::ContextRecords,
            limit: 2,
            attempted: 3,
        }),
        unsupported: vec![],
    }
}

// -- 28: `UnsupportedRegions` refusal is commit-honest -----------------------

/// An `UnsupportedRegions` refusal while committing a nested unsupported
/// at-rule: no unsupported record, no item-ordinal advancement, no run
/// boundary leak. The owning context stops with honest `ParserResourceLimit`
/// termination at the exact boundary reached before the refused commit.
fn unsupported_regions_refusal_is_commit_honest() -> ContextGoldFixture {
    let outer = qualified_record(
        0,
        None,
        0,
        None,
        (0, 1),
        (1, 2),
        (2, 2),
        ContextGoldTermination::ParserResourceLimit(range(2, 2)),
        vec![],
    );
    ContextGoldFixture {
        id: "CSS-CONTEXT-GROUP-UNSUPPORTED-REGIONS-REFUSAL-001",
        group: ContextGoldGroup::ResourceLimit,
        source_id: 91_128,
        source: "a{@unknown{x:y;}}",
        byte_len: 17,
        contexts: vec![outer],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: Some(ContextGoldResourceExpectation {
            kind: ContextGoldResourceKind::UnsupportedRegions,
            limit: 0,
            attempted: 1,
        }),
        unsupported: vec![],
    }
}

pub(super) fn independent_group_context_fixtures() -> Vec<ContextGoldFixture> {
    vec![
        media_empty_prelude(),
        media_screen_prelude(),
        media_print_case_insensitive(),
        supports_basic(),
        supports_empty_value(),
        container_name_only(),
        layer_anonymous(),
        layer_dotted_name(),
        scope_empty_prelude(),
        scope_amp_shape(),
        declaration_before_inside_after_group(),
        qualified_rule_child_inside_group(),
        group_nested_inside_group(),
        qualified_child_under_nested_groups(),
        unknown_block_at_rule_resumes_parent(),
        unknown_semicolon_at_rule_resumes_parent(),
        media_outside_bounded_subset(),
        supports_outside_bounded_subset(),
        container_reserved_name_outside_subset(),
        layer_statement_form_remains_unsupported(),
        scope_broader_form_remains_unsupported(),
        custom_property_value_does_not_create_group_context(),
        malformed_recovery_before_group_leaks_no_ordinal(),
        duplicate_group_spelling_distinct_identities(),
        true_eof_while_group_ancestry_active(),
        peak_context_depth_refusal_on_group_entry(),
        context_records_refusal_on_group_entry(),
        unsupported_regions_refusal_is_commit_honest(),
    ]
}
