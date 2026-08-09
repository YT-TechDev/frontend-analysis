//! Independent context/order/resource gold fixtures for the future #167
//! nested-context producer (#166).
//!
//! These are contract fixtures, not assertions about the current #166
//! producer: the current producer always emits `context_records = []` (see
//! `css::parser::producer`). Authored independently from the approved #166
//! architecture record, never derived from a producer, production types, or
//! external tooling.

use super::context_gold::{
    ContextGoldDeclarationItem, ContextGoldFixture, ContextGoldGroup, ContextGoldKind,
    ContextGoldParent, ContextGoldRecord, ContextGoldResourceExpectation, ContextGoldResourceKind,
    ContextGoldTermination,
};
use super::gold::GoldRange;

fn range(start: usize, end: usize) -> GoldRange {
    GoldRange::new(start, end)
}

/// Category 1: one top-level qualified-rule context with one declaration.
fn top_level_single_declaration() -> ContextGoldFixture {
    ContextGoldFixture {
        id: "CSS-CONTEXT-TOP-LEVEL-SINGLE-DECLARATION-001",
        group: ContextGoldGroup::TopLevel,
        source_id: 90_001,
        source: "a{color:red;}",
        byte_len: 13,
        contexts: vec![ContextGoldRecord {
            id: 0,
            parent: None,
            kind: ContextGoldKind::QualifiedRuleBlock,
            header: range(0, 1),
            block_opener: range(1, 2),
            body: range(2, 12),
            termination: ContextGoldTermination::AuthoredRightCurly(range(12, 13)),
            declarations: vec![ContextGoldDeclarationItem {
                item_ordinal: 0,
                run_ordinal: 0,
                span: range(2, 12),
            }],
        }],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
    }
}

/// Category 2: declaration-only block with multiple declarations in run 0.
fn declaration_only_block_multiple_declarations() -> ContextGoldFixture {
    ContextGoldFixture {
        id: "CSS-CONTEXT-DECLARATION-ONLY-MULTIPLE-001",
        group: ContextGoldGroup::DeclarationOnly,
        source_id: 90_002,
        source: "a{color:red;background:blue;}",
        byte_len: 29,
        contexts: vec![ContextGoldRecord {
            id: 0,
            parent: None,
            kind: ContextGoldKind::QualifiedRuleBlock,
            header: range(0, 1),
            block_opener: range(1, 2),
            body: range(2, 28),
            termination: ContextGoldTermination::AuthoredRightCurly(range(28, 29)),
            declarations: vec![
                ContextGoldDeclarationItem {
                    item_ordinal: 0,
                    run_ordinal: 0,
                    span: range(2, 12),
                },
                ContextGoldDeclarationItem {
                    item_ordinal: 1,
                    run_ordinal: 0,
                    span: range(12, 28),
                },
            ],
        }],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
    }
}

/// Category 3: declaration -> child qualified rule -> declaration (runs 0
/// and 1), matching the architecture record's own canonical example.
fn declaration_child_rule_declaration() -> ContextGoldFixture {
    let outer = ContextGoldRecord {
        id: 0,
        parent: None,
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 43),
        termination: ContextGoldTermination::AuthoredRightCurly(range(43, 44)),
        declarations: vec![
            ContextGoldDeclarationItem {
                item_ordinal: 0,
                run_ordinal: 0,
                span: range(2, 12),
            },
            ContextGoldDeclarationItem {
                item_ordinal: 2,
                run_ordinal: 1,
                span: range(26, 43),
            },
        ],
    };
    let child_b = ContextGoldRecord {
        id: 1,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 1,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(12, 13),
        block_opener: range(13, 14),
        body: range(14, 25),
        termination: ContextGoldTermination::AuthoredRightCurly(range(25, 26)),
        declarations: vec![ContextGoldDeclarationItem {
            item_ordinal: 0,
            run_ordinal: 0,
            span: range(14, 25),
        }],
    };
    ContextGoldFixture {
        id: "CSS-CONTEXT-DECLARATION-CHILD-DECLARATION-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 90_003,
        source: "a{color:red;b{color:blue;}background:black;}",
        byte_len: 44,
        contexts: vec![outer, child_b],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
    }
}

/// Category 4: multiple child rules between declaration runs (runs 0, 1, 2).
fn multiple_child_rules_between_runs() -> ContextGoldFixture {
    let outer = ContextGoldRecord {
        id: 0,
        parent: None,
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 38),
        termination: ContextGoldTermination::AuthoredRightCurly(range(38, 39)),
        declarations: vec![
            ContextGoldDeclarationItem {
                item_ordinal: 0,
                run_ordinal: 0,
                span: range(2, 8),
            },
            ContextGoldDeclarationItem {
                item_ordinal: 2,
                run_ordinal: 1,
                span: range(17, 23),
            },
            ContextGoldDeclarationItem {
                item_ordinal: 4,
                run_ordinal: 2,
                span: range(32, 38),
            },
        ],
    };
    let child_b = ContextGoldRecord {
        id: 1,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 1,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(8, 9),
        block_opener: range(9, 10),
        body: range(10, 16),
        termination: ContextGoldTermination::AuthoredRightCurly(range(16, 17)),
        declarations: vec![ContextGoldDeclarationItem {
            item_ordinal: 0,
            run_ordinal: 0,
            span: range(10, 16),
        }],
    };
    let child_c = ContextGoldRecord {
        id: 2,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 3,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(23, 24),
        block_opener: range(24, 25),
        body: range(25, 31),
        termination: ContextGoldTermination::AuthoredRightCurly(range(31, 32)),
        declarations: vec![ContextGoldDeclarationItem {
            item_ordinal: 0,
            run_ordinal: 0,
            span: range(25, 31),
        }],
    };
    ContextGoldFixture {
        id: "CSS-CONTEXT-MULTIPLE-CHILD-RULES-BETWEEN-RUNS-001",
        group: ContextGoldGroup::MixedDeclarationsAndRules,
        source_id: 90_004,
        source: "a{p0:v0;b{p1:v1;}p2:v2;c{p3:v3;}p4:v4;}",
        byte_len: 39,
        contexts: vec![outer, child_b, child_c],
        expected_context_record_count: 3,
        expected_peak_context_depth: 2,
        resource_expectation: None,
    }
}

/// Category 5: recursive qualified-rule nesting with depth > 2.
fn recursive_nesting_depth_three() -> ContextGoldFixture {
    let a = ContextGoldRecord {
        id: 0,
        parent: None,
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 12),
        termination: ContextGoldTermination::AuthoredRightCurly(range(12, 13)),
        declarations: vec![],
    };
    let b = ContextGoldRecord {
        id: 1,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(2, 3),
        block_opener: range(3, 4),
        body: range(4, 11),
        termination: ContextGoldTermination::AuthoredRightCurly(range(11, 12)),
        declarations: vec![],
    };
    let c = ContextGoldRecord {
        id: 2,
        parent: Some(ContextGoldParent {
            parent_id: 1,
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(4, 5),
        block_opener: range(5, 6),
        body: range(6, 10),
        termination: ContextGoldTermination::AuthoredRightCurly(range(10, 11)),
        declarations: vec![ContextGoldDeclarationItem {
            item_ordinal: 0,
            run_ordinal: 0,
            span: range(6, 10),
        }],
    };
    ContextGoldFixture {
        id: "CSS-CONTEXT-RECURSIVE-NESTING-DEPTH-THREE-001",
        group: ContextGoldGroup::RecursiveNesting,
        source_id: 90_005,
        source: "a{b{c{p:v;}}}",
        byte_len: 13,
        contexts: vec![a, b, c],
        expected_context_record_count: 3,
        expected_peak_context_depth: 3,
        resource_expectation: None,
    }
}

/// Category 6: true-EOF context termination (no authored `}`, no fabricated
/// closer).
fn true_eof_termination() -> ContextGoldFixture {
    ContextGoldFixture {
        id: "CSS-CONTEXT-TRUE-EOF-TERMINATION-001",
        group: ContextGoldGroup::Termination,
        source_id: 90_006,
        source: "a{",
        byte_len: 2,
        contexts: vec![ContextGoldRecord {
            id: 0,
            parent: None,
            kind: ContextGoldKind::QualifiedRuleBlock,
            header: range(0, 1),
            block_opener: range(1, 2),
            body: range(2, 2),
            termination: ContextGoldTermination::EndOfInput(range(2, 2)),
            declarations: vec![],
        }],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
    }
}

/// Category 7: upstream-incomplete context termination.
fn upstream_incomplete_termination() -> ContextGoldFixture {
    ContextGoldFixture {
        id: "CSS-CONTEXT-UPSTREAM-INCOMPLETE-TERMINATION-001",
        group: ContextGoldGroup::Termination,
        source_id: 90_007,
        source: "a{color:red",
        byte_len: 11,
        contexts: vec![ContextGoldRecord {
            id: 0,
            parent: None,
            kind: ContextGoldKind::QualifiedRuleBlock,
            header: range(0, 1),
            block_opener: range(1, 2),
            body: range(2, 11),
            termination: ContextGoldTermination::UpstreamTokenizerIncomplete(range(11, 11)),
            declarations: vec![],
        }],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
    }
}

/// Category 8: a context whose own termination is `ParserResourceLimit`
/// because the run stopped (on some unrelated resource) while it was still
/// active -- distinct from categories 12/13, which model the
/// `PeakContextDepth`/`ContextRecords` refusal itself.
fn parser_resource_limited_termination() -> ContextGoldFixture {
    ContextGoldFixture {
        id: "CSS-CONTEXT-PARSER-RESOURCE-LIMITED-TERMINATION-001",
        group: ContextGoldGroup::Termination,
        source_id: 90_008,
        source: "a{color:red;}",
        byte_len: 13,
        contexts: vec![ContextGoldRecord {
            id: 0,
            parent: None,
            kind: ContextGoldKind::QualifiedRuleBlock,
            header: range(0, 1),
            block_opener: range(1, 2),
            body: range(2, 8),
            termination: ContextGoldTermination::ParserResourceLimit(range(8, 8)),
            declarations: vec![],
        }],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
    }
}

/// Category 9: identical raw child spelling nested under two distinct
/// top-level parents must still receive distinct context identities.
fn duplicate_authored_spelling_distinct_identities() -> ContextGoldFixture {
    let a = ContextGoldRecord {
        id: 0,
        parent: None,
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 9),
        termination: ContextGoldTermination::AuthoredRightCurly(range(9, 10)),
        declarations: vec![],
    };
    let x_in_a = ContextGoldRecord {
        id: 1,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(2, 3),
        block_opener: range(3, 4),
        body: range(4, 8),
        termination: ContextGoldTermination::AuthoredRightCurly(range(8, 9)),
        declarations: vec![ContextGoldDeclarationItem {
            item_ordinal: 0,
            run_ordinal: 0,
            span: range(4, 8),
        }],
    };
    let b = ContextGoldRecord {
        id: 2,
        parent: None,
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(10, 11),
        block_opener: range(11, 12),
        body: range(12, 19),
        termination: ContextGoldTermination::AuthoredRightCurly(range(19, 20)),
        declarations: vec![],
    };
    let x_in_b = ContextGoldRecord {
        id: 3,
        parent: Some(ContextGoldParent {
            parent_id: 2,
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(12, 13),
        block_opener: range(13, 14),
        body: range(14, 18),
        termination: ContextGoldTermination::AuthoredRightCurly(range(18, 19)),
        declarations: vec![ContextGoldDeclarationItem {
            item_ordinal: 0,
            run_ordinal: 0,
            span: range(14, 18),
        }],
    };
    ContextGoldFixture {
        id: "CSS-CONTEXT-DUPLICATE-SPELLING-DISTINCT-IDENTITIES-001",
        group: ContextGoldGroup::DuplicateSpelling,
        source_id: 90_009,
        source: "a{x{p:v;}}b{x{p:v;}}",
        byte_len: 20,
        contexts: vec![a, x_in_a, b, x_in_b],
        expected_context_record_count: 4,
        expected_peak_context_depth: 2,
        resource_expectation: None,
    }
}

/// Category 10: a custom-property declaration whose value is a brace-shaped
/// simple block must never itself become a child context.
fn custom_property_brace_value_is_not_a_child_context() -> ContextGoldFixture {
    ContextGoldFixture {
        id: "CSS-CONTEXT-CUSTOM-PROPERTY-BRACE-VALUE-NOT-CONTEXT-001",
        group: ContextGoldGroup::CustomProperty,
        source_id: 90_010,
        source: "a{--x:{y:1;};}",
        byte_len: 14,
        contexts: vec![ContextGoldRecord {
            id: 0,
            parent: None,
            kind: ContextGoldKind::QualifiedRuleBlock,
            header: range(0, 1),
            block_opener: range(1, 2),
            body: range(2, 13),
            termination: ContextGoldTermination::AuthoredRightCurly(range(13, 14)),
            declarations: vec![ContextGoldDeclarationItem {
                item_ordinal: 0,
                run_ordinal: 0,
                span: range(2, 13),
            }],
        }],
        expected_context_record_count: 1,
        expected_peak_context_depth: 1,
        resource_expectation: None,
    }
}

/// Category 11: malformed/recovered syntax around a context transition.
/// `"color red;"` fails declaration recognition and qualified-rule
/// fallback, so it materializes no direct item; the following child rule
/// `b{...}` is item ordinal 0, not 1, because the abandoned span never
/// consumed an ordinal.
fn malformed_recovery_does_not_consume_an_item_ordinal() -> ContextGoldFixture {
    let outer = ContextGoldRecord {
        id: 0,
        parent: None,
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 19),
        termination: ContextGoldTermination::AuthoredRightCurly(range(19, 20)),
        declarations: vec![],
    };
    let child_b = ContextGoldRecord {
        id: 1,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            // Not 1: the preceding malformed "color red;" span never
            // materialized a direct item.
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(12, 13),
        block_opener: range(13, 14),
        body: range(14, 18),
        termination: ContextGoldTermination::AuthoredRightCurly(range(18, 19)),
        declarations: vec![ContextGoldDeclarationItem {
            item_ordinal: 0,
            run_ordinal: 0,
            span: range(14, 18),
        }],
    };
    ContextGoldFixture {
        id: "CSS-CONTEXT-MALFORMED-RECOVERY-NO-SYNTHETIC-ITEM-001",
        group: ContextGoldGroup::Malformed,
        source_id: 90_011,
        source: "a{color red;b{p:v;}}",
        byte_len: 20,
        contexts: vec![outer, child_b],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: None,
    }
}

/// Category 12: a `PeakContextDepth` refusal. `a{b{c{...` would enter a
/// third simultaneously active context; the ancestors remain committed with
/// honest `ParserResourceLimit` termination and no fabricated closure, and
/// the refused third identity never appears (no ID gap).
fn peak_context_depth_limit_case() -> ContextGoldFixture {
    let a = ContextGoldRecord {
        id: 0,
        parent: None,
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 6),
        termination: ContextGoldTermination::ParserResourceLimit(range(6, 6)),
        declarations: vec![],
    };
    let b = ContextGoldRecord {
        id: 1,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(2, 3),
        block_opener: range(3, 4),
        body: range(4, 6),
        termination: ContextGoldTermination::ParserResourceLimit(range(6, 6)),
        declarations: vec![],
    };
    ContextGoldFixture {
        id: "CSS-CONTEXT-PEAK-CONTEXT-DEPTH-LIMIT-001",
        group: ContextGoldGroup::ResourceLimit,
        source_id: 90_012,
        source: "a{b{c{p:v;}}}",
        byte_len: 13,
        contexts: vec![a, b],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: Some(ContextGoldResourceExpectation {
            kind: ContextGoldResourceKind::PeakContextDepth,
            limit: 2,
            attempted: 3,
        }),
    }
}

/// Category 13: a `ContextRecords` refusal. A third context (`c{...}`)
/// would exceed the committed-record cap; `a` (still active) retains
/// `ParserResourceLimit` termination and `b` (already closed before the
/// refusal) keeps its honest `AuthoredRightCurly` termination.
fn context_records_limit_case() -> ContextGoldFixture {
    let a = ContextGoldRecord {
        id: 0,
        parent: None,
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 11),
        termination: ContextGoldTermination::ParserResourceLimit(range(11, 11)),
        declarations: vec![],
    };
    let b = ContextGoldRecord {
        id: 1,
        parent: Some(ContextGoldParent {
            parent_id: 0,
            item_ordinal: 0,
        }),
        kind: ContextGoldKind::QualifiedRuleBlock,
        header: range(2, 3),
        block_opener: range(3, 4),
        body: range(4, 8),
        termination: ContextGoldTermination::AuthoredRightCurly(range(8, 9)),
        declarations: vec![ContextGoldDeclarationItem {
            item_ordinal: 0,
            run_ordinal: 0,
            span: range(4, 8),
        }],
    };
    ContextGoldFixture {
        id: "CSS-CONTEXT-CONTEXT-RECORDS-LIMIT-001",
        group: ContextGoldGroup::ResourceLimit,
        source_id: 90_013,
        source: "a{b{p:v;}c{q:w;}}",
        byte_len: 17,
        contexts: vec![a, b],
        expected_context_record_count: 2,
        expected_peak_context_depth: 2,
        resource_expectation: Some(ContextGoldResourceExpectation {
            kind: ContextGoldResourceKind::ContextRecords,
            limit: 2,
            attempted: 3,
        }),
    }
}

pub(super) fn independent_context_fixtures() -> Vec<ContextGoldFixture> {
    vec![
        top_level_single_declaration(),
        declaration_only_block_multiple_declarations(),
        declaration_child_rule_declaration(),
        multiple_child_rules_between_runs(),
        recursive_nesting_depth_three(),
        true_eof_termination(),
        upstream_incomplete_termination(),
        parser_resource_limited_termination(),
        duplicate_authored_spelling_distinct_identities(),
        custom_property_brace_value_is_not_a_child_context(),
        malformed_recovery_does_not_consume_an_item_ordinal(),
        peak_context_depth_limit_case(),
        context_records_limit_case(),
    ]
}
