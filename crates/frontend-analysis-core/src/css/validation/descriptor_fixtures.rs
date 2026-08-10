//! Independent #169 descriptor-context evidence gold fixtures (validation
//! phase authorization comment `5236795522`), covering all 24 required
//! architecture categories from proposal comment `5236345421` section 15.
//!
//! Authored independently from the approved architecture record, never
//! derived from production `CssParserContextRecord`, `CssDescriptorOccurrence`,
//! a producer, an external parser, CSSOM, or browser output. Kept a wholly
//! separate inventory from the fixed 16 `#166` context fixtures
//! (`context_fixtures.rs`) and the fixed 28 `#168` group fixtures
//! (`group_context_fixtures.rs`): those inventories are preserved exact and
//! unrepurposed by this Leaf.
//!
//! Category 16 (synthetic upstream-tokenizer-incomplete while a descriptor
//! context is already active) requires a genuine tokenizer-produced lexical
//! prefix rather than this module's generic gold-fixture execution recipe,
//! so it is covered by a dedicated test in
//! `descriptor_lifecycle_validation_tests.rs`, mirroring how #168 covered its
//! own analogous lifecycle gap outside its 28-fixture inventory. Category 22
//! (deterministic repeat / cross-`SourceId` equivalence) and category 24
//! (existing #167/#168 behavior remains green) are cross-cutting properties
//! proved in `descriptor_conformance_tests.rs` against this whole inventory
//! and the existing fixed inventories, not by a single dedicated fixture.

use super::descriptor_gold::{
    DescriptorGoldCategory, DescriptorGoldContext, DescriptorGoldFixture,
    DescriptorGoldNestedUnsupportedAtRule, DescriptorGoldOccurrence,
    DescriptorGoldOccurrenceTermination, DescriptorGoldQualifiedCompanion, DescriptorGoldRecovery,
    DescriptorGoldRecoveryTermination, DescriptorGoldResourceExpectation,
    DescriptorGoldResourceKind, DescriptorGoldRoot, DescriptorGoldRuleKind,
    DescriptorGoldTermination,
};
use super::gold::GoldRange;

fn range(start: usize, end: usize) -> GoldRange {
    GoldRange::new(start, end)
}

fn semi(complete_end: usize) -> DescriptorGoldOccurrenceTermination {
    DescriptorGoldOccurrenceTermination::AuthoredSemicolon(range(complete_end - 1, complete_end))
}

/// Category 1: `@font-face` with multiple descriptor-shaped items.
fn font_face_multiple_descriptors() -> DescriptorGoldFixture {
    let context = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(0, 10),
        property_name: None,
        header: range(0, 10),
        block_opener: range(10, 11),
        body: range(11, 31),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(31, 32)),
        occurrences: vec![
            DescriptorGoldOccurrence {
                item_ordinal: 0,
                complete: range(11, 25),
                name: range(11, 22),
                colon: range(22, 23),
                value: range(23, 24),
                priority: None,
                termination: semi(25),
            },
            DescriptorGoldOccurrence {
                item_ordinal: 1,
                complete: range(25, 31),
                name: range(25, 28),
                colon: range(28, 29),
                value: range(29, 30),
                priority: None,
                termination: semi(31),
            },
        ],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-FONT-FACE-MULTIPLE-001",
        categories: &[DescriptorGoldCategory::FontFaceMultipleDescriptors],
        source_id: 91_001,
        source: "@font-face{font-family:x;src:y;}",
        byte_len: 32,
        roots: vec![DescriptorGoldRoot::Descriptor(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 2: `@property --x` with multiple descriptor-shaped items,
/// retaining the exact authored custom-property-name anchor.
fn property_multiple_descriptors() -> DescriptorGoldFixture {
    let context = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::Property,
        at_keyword: range(0, 9),
        property_name: Some(range(10, 13)),
        header: range(0, 13),
        block_opener: range(13, 14),
        body: range(14, 38),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(38, 39)),
        occurrences: vec![
            DescriptorGoldOccurrence {
                item_ordinal: 0,
                complete: range(14, 23),
                name: range(14, 20),
                colon: range(20, 21),
                value: range(21, 22),
                priority: None,
                termination: semi(23),
            },
            DescriptorGoldOccurrence {
                item_ordinal: 1,
                complete: range(23, 38),
                name: range(23, 31),
                colon: range(31, 32),
                value: range(32, 37),
                priority: None,
                termination: semi(38),
            },
        ],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-PROPERTY-MULTIPLE-001",
        categories: &[DescriptorGoldCategory::PropertyMultipleDescriptors],
        source_id: 91_002,
        source: "@property --x{syntax:y;inherits:false;}",
        byte_len: 39,
        roots: vec![DescriptorGoldRoot::Descriptor(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 3: ASCII-case-insensitive at-rule spelling for both `@FONT-FACE`
/// and `@PROPERTY`.
fn case_insensitive_at_keyword() -> DescriptorGoldFixture {
    let font_face = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(0, 10),
        property_name: None,
        header: range(0, 10),
        block_opener: range(10, 11),
        body: range(11, 17),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(17, 18)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(11, 17),
            name: range(11, 14),
            colon: range(14, 15),
            value: range(15, 16),
            priority: None,
            termination: semi(17),
        }],
        nested_unsupported: vec![],
    };
    let property = DescriptorGoldContext {
        id: 1,
        root_item_ordinal: 1,
        kind: DescriptorGoldRuleKind::Property,
        at_keyword: range(18, 27),
        property_name: Some(range(28, 31)),
        header: range(18, 31),
        block_opener: range(31, 32),
        body: range(32, 41),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(41, 42)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(32, 41),
            name: range(32, 38),
            colon: range(38, 39),
            value: range(39, 40),
            priority: None,
            termination: semi(41),
        }],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-CASE-INSENSITIVE-AT-KEYWORD-001",
        categories: &[DescriptorGoldCategory::CaseInsensitiveAtKeyword],
        source_id: 91_003,
        source: "@FONT-FACE{src:y;}@PROPERTY --x{syntax:z;}",
        byte_len: 42,
        roots: vec![
            DescriptorGoldRoot::Descriptor(font_face),
            DescriptorGoldRoot::Descriptor(property),
        ],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 4: an escaped authored custom-property spelling
/// (`--sp\61 cing`, tokenizer-decoded to `--spacing`) qualifies, while the
/// exact raw authored name evidence (including the escape) is retained as
/// parent evidence -- never a decoded reconstruction.
fn escaped_custom_property_name() -> DescriptorGoldFixture {
    let context = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::Property,
        at_keyword: range(0, 9),
        property_name: Some(range(10, 22)),
        header: range(0, 22),
        block_opener: range(22, 23),
        body: range(23, 32),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(32, 33)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(23, 32),
            name: range(23, 29),
            colon: range(29, 30),
            value: range(30, 31),
            priority: None,
            termination: semi(32),
        }],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-PROPERTY-ESCAPED-NAME-001",
        categories: &[DescriptorGoldCategory::EscapedCustomPropertyName],
        source_id: 91_004,
        source: "@property --sp\\61 cing{syntax:y;}",
        byte_len: 33,
        roots: vec![DescriptorGoldRoot::Descriptor(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 5: identical `name:value` spelling in an ordinary qualified-rule
/// declaration versus a descriptor context produces semantically distinct
/// occurrence types.
fn same_spelling_distinct_occurrence_types() -> DescriptorGoldFixture {
    let companion = DescriptorGoldQualifiedCompanion {
        id: 0,
        root_item_ordinal: 0,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 12),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(12, 13)),
        declarations: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(2, 12),
            name: range(2, 7),
            colon: range(7, 8),
            value: range(8, 11),
            priority: None,
            termination: semi(12),
        }],
    };
    let context = DescriptorGoldContext {
        id: 1,
        root_item_ordinal: 1,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(13, 23),
        property_name: None,
        header: range(13, 23),
        block_opener: range(23, 24),
        body: range(24, 34),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(34, 35)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(24, 34),
            name: range(24, 29),
            colon: range(29, 30),
            value: range(30, 33),
            priority: None,
            termination: semi(34),
        }],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-SAME-SPELLING-DISTINCT-TYPES-001",
        categories: &[DescriptorGoldCategory::SameSpellingDistinctOccurrenceTypes],
        source_id: 91_005,
        source: "a{color:red;}@font-face{color:red;}",
        byte_len: 35,
        roots: vec![
            DescriptorGoldRoot::Qualified(companion),
            DescriptorGoldRoot::Descriptor(context),
        ],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 6: duplicate raw descriptor spelling (`unicode-range:a;`)
/// retained in two distinct descriptor contexts keeps distinct owner
/// `ContextId`s.
fn duplicate_spelling_distinct_contexts() -> DescriptorGoldFixture {
    let font_face = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(0, 10),
        property_name: None,
        header: range(0, 10),
        block_opener: range(10, 11),
        body: range(11, 27),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(27, 28)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(11, 27),
            name: range(11, 24),
            colon: range(24, 25),
            value: range(25, 26),
            priority: None,
            termination: semi(27),
        }],
        nested_unsupported: vec![],
    };
    let property = DescriptorGoldContext {
        id: 1,
        root_item_ordinal: 1,
        kind: DescriptorGoldRuleKind::Property,
        at_keyword: range(28, 37),
        property_name: Some(range(38, 41)),
        header: range(28, 41),
        block_opener: range(41, 42),
        body: range(42, 58),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(58, 59)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(42, 58),
            name: range(42, 55),
            colon: range(55, 56),
            value: range(56, 57),
            priority: None,
            termination: semi(58),
        }],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-DUPLICATE-SPELLING-DISTINCT-CONTEXTS-001",
        categories: &[DescriptorGoldCategory::DuplicateSpellingDistinctContexts],
        source_id: 91_006,
        source: "@font-face{unicode-range:a;}@property --x{unicode-range:a;}",
        byte_len: 59,
        roots: vec![
            DescriptorGoldRoot::Descriptor(font_face),
            DescriptorGoldRoot::Descriptor(property),
        ],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 7: `@property color { ... }` does not qualify a
/// `DescriptorRuleBlock` context and emits no descriptor occurrence; the
/// whole at-rule remains structurally consumed unsupported evidence.
fn property_color_does_not_qualify() -> DescriptorGoldFixture {
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-PROPERTY-COLOR-UNQUALIFIED-001",
        categories: &[DescriptorGoldCategory::PropertyColorDoesNotQualify],
        source_id: 91_007,
        source: "@property color{syntax:y;}",
        byte_len: 26,
        roots: vec![DescriptorGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 26),
            at_keyword: range(0, 9),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 8: the reserved exact spelling `@property -- { ... }` never
/// qualifies as a custom-property name.
fn property_reserved_double_hyphen_does_not_qualify() -> DescriptorGoldFixture {
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-PROPERTY-RESERVED-DOUBLE-HYPHEN-001",
        categories: &[DescriptorGoldCategory::PropertyReservedDoubleHyphenDoesNotQualify],
        source_id: 91_008,
        source: "@property --{syntax:y;}",
        byte_len: 23,
        roots: vec![DescriptorGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 23),
            at_keyword: range(0, 9),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 9: the syntactically valid multi-name `@property --a, --b { ... }`
/// prelude remains explicit unsupported capability evidence in this bounded
/// Leaf, never an `Invalid*` diagnostic asserting the CSS itself is invalid.
fn property_multi_name_remains_unsupported() -> DescriptorGoldFixture {
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-PROPERTY-MULTI-NAME-UNSUPPORTED-001",
        categories: &[DescriptorGoldCategory::PropertyMultiNameRemainsUnsupported],
        source_id: 91_009,
        source: "@property --a, --b{syntax:y;}",
        byte_len: 29,
        roots: vec![DescriptorGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 29),
            at_keyword: range(0, 9),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 10: a non-empty `@font-face` prelude remains unsupported.
fn font_face_non_empty_prelude_remains_unsupported() -> DescriptorGoldFixture {
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-FONT-FACE-NON-EMPTY-PRELUDE-001",
        categories: &[DescriptorGoldCategory::FontFaceNonEmptyPreludeRemainsUnsupported],
        source_id: 91_010,
        source: "@font-face x{font-family:y;}",
        byte_len: 28,
        roots: vec![DescriptorGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 28),
            at_keyword: range(0, 10),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 11: an unknown descriptor-bearing top-level at-rule (neither
/// `font-face` nor `property`) remains unsupported and produces no
/// descriptor occurrences.
fn unknown_descriptor_bearing_at_rule_unsupported() -> DescriptorGoldFixture {
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-UNKNOWN-AT-RULE-UNSUPPORTED-001",
        categories: &[DescriptorGoldCategory::UnknownDescriptorBearingAtRuleUnsupported],
        source_id: 91_011,
        source: "@counter-style foo{system:cyclic;}",
        byte_len: 34,
        roots: vec![DescriptorGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 34),
            at_keyword: range(0, 14),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 12: a malformed descriptor-shaped item (`***;`) produces the
/// existing shared `InvalidBlockItem`/`MalformedBlockItem` diagnostic/
/// recovery evidence and consumes no direct-item ordinal, so the later valid
/// descriptor `src:y;` survives at item ordinal 0, not 1.
fn malformed_descriptor_item_recovers() -> DescriptorGoldFixture {
    let context = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(0, 10),
        property_name: None,
        header: range(0, 10),
        block_opener: range(10, 11),
        body: range(11, 21),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(21, 22)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(15, 21),
            name: range(15, 18),
            colon: range(18, 19),
            value: range(19, 20),
            priority: None,
            termination: semi(21),
        }],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-MALFORMED-ITEM-RECOVERS-001",
        categories: &[DescriptorGoldCategory::MalformedDescriptorItemRecovers],
        source_id: 91_012,
        source: "@font-face{***;src:y;}",
        byte_len: 22,
        roots: vec![DescriptorGoldRoot::Descriptor(context)],
        resource_expectation: None,
        diagnostics: vec![range(11, 15)],
        recovery: vec![DescriptorGoldRecovery {
            region: range(11, 15),
            termination: DescriptorGoldRecoveryTermination::AuthoredSemicolon(range(14, 15)),
        }],
    }
}

/// Category 13: a qualified-rule-shaped fragment (`.foo{color:red;}`) inside
/// a descriptor block is malformed descriptor-list input, not a nested-rule
/// trigger: it never becomes a child `QualifiedRuleBlock`, and the later
/// valid descriptor `src:y;` still recovers as item ordinal 0.
fn qualified_rule_shaped_fragment_not_child_context() -> DescriptorGoldFixture {
    let context = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(0, 10),
        property_name: None,
        header: range(0, 10),
        block_opener: range(10, 11),
        body: range(11, 34),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(34, 35)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(28, 34),
            name: range(28, 31),
            colon: range(31, 32),
            value: range(32, 33),
            priority: None,
            termination: semi(34),
        }],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-QUALIFIED-SHAPE-NOT-CHILD-CONTEXT-001",
        categories: &[DescriptorGoldCategory::QualifiedRuleShapedFragmentNotChildContext],
        source_id: 91_013,
        source: "@font-face{.foo{color:red;};src:y;}",
        byte_len: 35,
        roots: vec![DescriptorGoldRoot::Descriptor(context)],
        resource_expectation: None,
        diagnostics: vec![range(11, 28)],
        recovery: vec![DescriptorGoldRecovery {
            region: range(11, 28),
            termination: DescriptorGoldRecoveryTermination::AuthoredSemicolon(range(27, 28)),
        }],
    }
}

/// Category 14: an `AtKeyword` inside a descriptor block -- including a
/// registry member like `@media` that would otherwise qualify as a
/// `GroupRuleBlock` in a qualified/group context -- becomes exactly one
/// unsupported direct item, never a `GroupRuleBlock`; later descriptor
/// parsing resumes with exact ordinal ordering (`@media` at ordinal 0,
/// `src:y;` at ordinal 1, sharing the same direct-item ordinal space).
fn nested_at_keyword_single_unsupported_item() -> DescriptorGoldFixture {
    let context = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(0, 10),
        property_name: None,
        header: range(0, 10),
        block_opener: range(10, 11),
        body: range(11, 42),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(42, 43)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 1,
            complete: range(36, 42),
            name: range(36, 39),
            colon: range(39, 40),
            value: range(40, 41),
            priority: None,
            termination: semi(42),
        }],
        nested_unsupported: vec![DescriptorGoldNestedUnsupportedAtRule {
            item_ordinal: 0,
            complete: range(11, 36),
            at_keyword: range(11, 17),
        }],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-NESTED-AT-KEYWORD-UNSUPPORTED-ITEM-001",
        categories: &[DescriptorGoldCategory::NestedAtKeywordSingleUnsupportedItem],
        source_id: 91_014,
        source: "@font-face{@media screen{color:red;}src:y;}",
        byte_len: 43,
        roots: vec![DescriptorGoldRoot::Descriptor(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 15: true EOF while `DescriptorRuleBlock` is already active
/// retains the context with `EndOfInput` and no fabricated `}`; the
/// unterminated `font-family:x` descriptor is still recognized under the
/// pre-#169 `OmittedAtEndOfInput` declaration-syntax rule.
fn true_eof_while_descriptor_active() -> DescriptorGoldFixture {
    let context = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(0, 10),
        property_name: None,
        header: range(0, 10),
        block_opener: range(10, 11),
        body: range(11, 24),
        termination: DescriptorGoldTermination::EndOfInput(range(24, 24)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(11, 24),
            name: range(11, 22),
            colon: range(22, 23),
            value: range(23, 24),
            priority: None,
            termination: DescriptorGoldOccurrenceTermination::OmittedAtEndOfInput(range(24, 24)),
        }],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-TRUE-EOF-ACTIVE-001",
        categories: &[DescriptorGoldCategory::TrueEofWhileDescriptorActive],
        source_id: 91_015,
        source: "@font-face{font-family:x",
        byte_len: 24,
        roots: vec![DescriptorGoldRoot::Descriptor(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 17: a generic parser-resource stop (`AlgorithmSteps`, distinct
/// from the descriptor-specific resource categories 18-21) after a
/// `DescriptorRuleBlock` has already entered and one descriptor occurrence
/// has already committed. Contract/gold self-validation only, mirroring the
/// established precedent for the fixed 16-fixture inventory's own generic
/// parser-resource partial-ancestry case
/// (`CSS-CONTEXT-PARSER-RESOURCE-LIMITED-TERMINATION-001`): a generic
/// step-based trigger's exact internal terminal is not an architecture-
/// specified contract, so this fixture is not claimed as an exact
/// production execution recipe.
fn generic_parser_resource_stop_after_entry() -> DescriptorGoldFixture {
    let context = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(0, 10),
        property_name: None,
        header: range(0, 10),
        block_opener: range(10, 11),
        body: range(11, 25),
        termination: DescriptorGoldTermination::ParserResourceLimit(range(25, 25)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(11, 25),
            name: range(11, 22),
            colon: range(22, 23),
            value: range(23, 24),
            priority: None,
            termination: semi(25),
        }],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-GENERIC-PARSER-RESOURCE-LIMITED-TERMINATION-001",
        categories: &[DescriptorGoldCategory::GenericParserResourceStopAfterEntry],
        source_id: 91_017,
        source: "@font-face{font-family:x;src:y;}",
        byte_len: 32,
        roots: vec![DescriptorGoldRoot::Descriptor(context)],
        resource_expectation: Some(DescriptorGoldResourceExpectation {
            kind: DescriptorGoldResourceKind::AlgorithmSteps,
            limit: 100,
            attempted: 101,
        }),
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 18: `PeakContextDepth` refusal at descriptor entry commits no
/// `ContextId`/root ordinal/evidence -- `roots` is empty.
fn peak_context_depth_refusal_at_entry() -> DescriptorGoldFixture {
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-PEAK-CONTEXT-DEPTH-REFUSAL-001",
        categories: &[DescriptorGoldCategory::PeakContextDepthRefusalAtEntry],
        source_id: 91_018,
        source: "@font-face{font-family:x;}",
        byte_len: 26,
        roots: vec![],
        resource_expectation: Some(DescriptorGoldResourceExpectation {
            kind: DescriptorGoldResourceKind::PeakContextDepth,
            limit: 0,
            attempted: 1,
        }),
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 19: `ContextRecords` refusal at descriptor entry carries the
/// same commit-honesty guarantee as category 18 -- `roots` is empty.
fn context_records_refusal_at_entry() -> DescriptorGoldFixture {
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-CONTEXT-RECORDS-REFUSAL-001",
        categories: &[DescriptorGoldCategory::ContextRecordsRefusalAtEntry],
        source_id: 91_019,
        source: "@font-face{font-family:x;}",
        byte_len: 26,
        roots: vec![],
        resource_expectation: Some(DescriptorGoldResourceExpectation {
            kind: DescriptorGoldResourceKind::ContextRecords,
            limit: 0,
            attempted: 1,
        }),
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 20: the aggregate `DeclarationOccurrences` cap is shared across
/// ordinary and descriptor occurrences in one run. One ordinary declaration
/// commits (usage 1); the descriptor context itself is still entered
/// (context entry is gated by `PeakContextDepth`/`ContextRecords` only, never
/// by `DeclarationOccurrences`), but its one descriptor-occurrence attempt is
/// prospectively refused -- neither the occurrence nor its item ordinal
/// commits, and the already-entered context retains honest
/// `ParserResourceLimit` termination at its own body start.
fn declaration_occurrences_aggregate_limit() -> DescriptorGoldFixture {
    let companion = DescriptorGoldQualifiedCompanion {
        id: 0,
        root_item_ordinal: 0,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 12),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(12, 13)),
        declarations: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(2, 12),
            name: range(2, 7),
            colon: range(7, 8),
            value: range(8, 11),
            priority: None,
            termination: semi(12),
        }],
    };
    let context = DescriptorGoldContext {
        id: 1,
        root_item_ordinal: 1,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(13, 23),
        property_name: None,
        header: range(13, 23),
        block_opener: range(23, 24),
        body: range(24, 24),
        termination: DescriptorGoldTermination::ParserResourceLimit(range(24, 24)),
        occurrences: vec![],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-DECLARATION-OCCURRENCES-AGGREGATE-LIMIT-001",
        categories: &[DescriptorGoldCategory::DeclarationOccurrencesAggregateLimit],
        source_id: 91_020,
        source: "a{color:red;}@font-face{font-family:x;}",
        byte_len: 39,
        roots: vec![
            DescriptorGoldRoot::Qualified(companion),
            DescriptorGoldRoot::Descriptor(context),
        ],
        resource_expectation: Some(DescriptorGoldResourceExpectation {
            kind: DescriptorGoldResourceKind::DeclarationOccurrences,
            limit: 1,
            attempted: 2,
        }),
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 21: `UnsupportedRegions` refusal inside an already-entered
/// `DescriptorRuleBlock` -- the nested `@media` unsupported item does not
/// commit, its item ordinal is never allocated, and the descriptor context
/// retains honest `ParserResourceLimit` termination at its own body start
/// with no fabricated run/item boundary.
fn unsupported_regions_refusal_inside_descriptor() -> DescriptorGoldFixture {
    let context = DescriptorGoldContext {
        id: 0,
        root_item_ordinal: 0,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(0, 10),
        property_name: None,
        header: range(0, 10),
        block_opener: range(10, 11),
        body: range(11, 11),
        termination: DescriptorGoldTermination::ParserResourceLimit(range(11, 11)),
        occurrences: vec![],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-UNSUPPORTED-REGIONS-REFUSAL-001",
        categories: &[DescriptorGoldCategory::UnsupportedRegionsRefusalInsideDescriptor],
        source_id: 91_021,
        source: "@font-face{@media screen{color:red;}}",
        byte_len: 37,
        roots: vec![DescriptorGoldRoot::Descriptor(context)],
        resource_expectation: Some(DescriptorGoldResourceExpectation {
            kind: DescriptorGoldResourceKind::UnsupportedRegions,
            limit: 0,
            attempted: 1,
        }),
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 23: existing custom-property brace-value declaration behavior
/// (`--x:{y:1;};`) outside any descriptor context remains ordinary
/// `CssDeclarationOccurrence` behavior, unaffected by the presence of a real
/// descriptor context later in the same run.
fn custom_property_brace_value_outside_descriptor_unaffected() -> DescriptorGoldFixture {
    let companion = DescriptorGoldQualifiedCompanion {
        id: 0,
        root_item_ordinal: 0,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 13),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(13, 14)),
        declarations: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(2, 13),
            name: range(2, 5),
            colon: range(5, 6),
            value: range(6, 12),
            priority: None,
            termination: semi(13),
        }],
    };
    let context = DescriptorGoldContext {
        id: 1,
        root_item_ordinal: 1,
        kind: DescriptorGoldRuleKind::FontFace,
        at_keyword: range(14, 24),
        property_name: None,
        header: range(14, 24),
        block_opener: range(24, 25),
        body: range(25, 31),
        termination: DescriptorGoldTermination::AuthoredRightCurly(range(31, 32)),
        occurrences: vec![DescriptorGoldOccurrence {
            item_ordinal: 0,
            complete: range(25, 31),
            name: range(25, 28),
            colon: range(28, 29),
            value: range(29, 30),
            priority: None,
            termination: semi(31),
        }],
        nested_unsupported: vec![],
    };
    DescriptorGoldFixture {
        id: "CSS-DESCRIPTOR-CUSTOM-PROPERTY-BRACE-VALUE-OUTSIDE-001",
        categories: &[DescriptorGoldCategory::CustomPropertyBraceValueOutsideDescriptorUnaffected],
        source_id: 91_023,
        source: "a{--x:{y:1;};}@font-face{src:z;}",
        byte_len: 32,
        roots: vec![
            DescriptorGoldRoot::Qualified(companion),
            DescriptorGoldRoot::Descriptor(context),
        ],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

pub(super) fn independent_descriptor_fixtures() -> Vec<DescriptorGoldFixture> {
    vec![
        font_face_multiple_descriptors(),
        property_multiple_descriptors(),
        case_insensitive_at_keyword(),
        escaped_custom_property_name(),
        same_spelling_distinct_occurrence_types(),
        duplicate_spelling_distinct_contexts(),
        property_color_does_not_qualify(),
        property_reserved_double_hyphen_does_not_qualify(),
        property_multi_name_remains_unsupported(),
        font_face_non_empty_prelude_remains_unsupported(),
        unknown_descriptor_bearing_at_rule_unsupported(),
        malformed_descriptor_item_recovers(),
        qualified_rule_shaped_fragment_not_child_context(),
        nested_at_keyword_single_unsupported_item(),
        true_eof_while_descriptor_active(),
        generic_parser_resource_stop_after_entry(),
        peak_context_depth_refusal_at_entry(),
        context_records_refusal_at_entry(),
        declaration_occurrences_aggregate_limit(),
        unsupported_regions_refusal_inside_descriptor(),
        custom_property_brace_value_outside_descriptor_unaffected(),
    ]
}
