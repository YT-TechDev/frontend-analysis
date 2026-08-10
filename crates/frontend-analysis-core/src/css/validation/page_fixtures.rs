//! Independent #170 `@page`/page-margin evidence gold fixtures, covering all
//! 40 required validation categories from the focused implementation/
//! evidence plan's section 14.
//!
//! Authored independently from the approved architecture record, never
//! derived from production `CssParserContextRecord`, `CssPageDeclarationOccurrence`,
//! `CssPageMarginDeclarationOccurrence`, a producer, an external parser,
//! CSSOM, or browser output. Kept a wholly separate inventory from the fixed
//! #166/#168 inventories and the #169 descriptor inventory.
//!
//! Categories 32/33 (synthetic upstream-tokenizer-incomplete while a Page/
//! Margin context is already active) and 34/35 (a generic parser-resource
//! stop reached only after a Page/Margin context has already been entered)
//! require either a genuine tokenizer-produced lexical prefix or a
//! non-deterministic-exact-step resource stop, so they are covered by
//! dedicated tests in `page_lifecycle_validation_tests.rs`, mirroring #169's
//! own precedent for its analogous categories 16/17. Category 40
//! (deterministic repeat / cross-`SourceId` equivalence) is a cross-cutting
//! property proved in `page_conformance_tests.rs` against this whole
//! inventory.

use super::gold::GoldRange;
use super::page_gold::{
    PageGoldCategory, PageGoldContext, PageGoldDescriptorCompanion, PageGoldFixture,
    PageGoldMarginContext, PageGoldMarginKind, PageGoldNestedUnsupportedAtRule, PageGoldOccurrence,
    PageGoldOccurrenceTermination, PageGoldQualifiedCompanion, PageGoldRecovery,
    PageGoldRecoveryTermination, PageGoldResourceExpectation, PageGoldResourceKind, PageGoldRoot,
    PageGoldTermination,
};

fn range(start: usize, end: usize) -> GoldRange {
    GoldRange::new(start, end)
}

fn semi(complete_end: usize) -> PageGoldOccurrenceTermination {
    PageGoldOccurrenceTermination::AuthoredSemicolon(range(complete_end - 1, complete_end))
}

/// Category 1: selector-less `@page`.
fn selector_less_page() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 16),
        termination: PageGoldTermination::AuthoredRightCurly(range(16, 17)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(6, 16),
            name: range(6, 10),
            colon: range(10, 11),
            value: range(11, 15),
            priority: None,
            termination: semi(16),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-SELECTOR-LESS-001",
        categories: &[
            PageGoldCategory::SelectorLessPage,
            PageGoldCategory::AuthoredClosePage,
        ],
        source_id: 92_001,
        source: "@page{size:auto;}",
        byte_len: 17,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 2: named page (`<ident-token>` page-type selector).
fn named_page() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: Some(range(6, 9)),
        header: range(0, 9),
        block_opener: range(9, 10),
        body: range(10, 20),
        termination: PageGoldTermination::AuthoredRightCurly(range(20, 21)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(10, 20),
            name: range(10, 14),
            colon: range(14, 15),
            value: range(15, 19),
            priority: None,
            termination: semi(20),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-NAMED-001",
        categories: &[PageGoldCategory::NamedPage],
        source_id: 92_002,
        source: "@page foo{size:auto;}",
        byte_len: 21,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 3: pseudo-only page.
fn pseudo_only_page() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: Some(range(6, 12)),
        header: range(0, 12),
        block_opener: range(12, 13),
        body: range(13, 23),
        termination: PageGoldTermination::AuthoredRightCurly(range(23, 24)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(13, 23),
            name: range(13, 17),
            colon: range(17, 18),
            value: range(18, 22),
            priority: None,
            termination: semi(23),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-PSEUDO-ONLY-001",
        categories: &[PageGoldCategory::PseudoOnlyPage],
        source_id: 92_003,
        source: "@page :first{size:auto;}",
        byte_len: 24,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 4: named page-type ident followed by a pseudo-page.
fn named_and_pseudo_page() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: Some(range(6, 15)),
        header: range(0, 15),
        block_opener: range(15, 16),
        body: range(16, 26),
        termination: PageGoldTermination::AuthoredRightCurly(range(26, 27)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(16, 26),
            name: range(16, 20),
            colon: range(20, 21),
            value: range(21, 25),
            priority: None,
            termination: semi(26),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-NAMED-AND-PSEUDO-001",
        categories: &[PageGoldCategory::NamedAndPseudoPage],
        source_id: 92_004,
        source: "@page foo:first{size:auto;}",
        byte_len: 27,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 5: repeated pseudo-pages on one selector.
fn repeated_pseudo() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: Some(range(6, 18)),
        header: range(0, 18),
        block_opener: range(18, 19),
        body: range(19, 29),
        termination: PageGoldTermination::AuthoredRightCurly(range(29, 30)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(19, 29),
            name: range(19, 23),
            colon: range(23, 24),
            value: range(24, 28),
            priority: None,
            termination: semi(29),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-REPEATED-PSEUDO-001",
        categories: &[PageGoldCategory::RepeatedPseudo],
        source_id: 92_005,
        source: "@page :first:blank{size:auto;}",
        byte_len: 30,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 6: comma-separated selector list.
fn comma_separated_selector_list() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: Some(range(6, 20)),
        header: range(0, 20),
        block_opener: range(20, 21),
        body: range(21, 31),
        termination: PageGoldTermination::AuthoredRightCurly(range(31, 32)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(21, 31),
            name: range(21, 25),
            colon: range(25, 26),
            value: range(26, 30),
            priority: None,
            termination: semi(31),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-COMMA-SEPARATED-001",
        categories: &[PageGoldCategory::CommaSeparatedSelectorList],
        source_id: 92_006,
        source: "@page foo, bar:first{size:auto;}",
        byte_len: 32,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 7: escaped page-type ident selector component.
fn escaped_selector_component() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: Some(range(6, 12)),
        header: range(0, 12),
        block_opener: range(12, 13),
        body: range(13, 23),
        termination: PageGoldTermination::AuthoredRightCurly(range(23, 24)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(13, 23),
            name: range(13, 17),
            colon: range(17, 18),
            value: range(18, 22),
            priority: None,
            termination: semi(23),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-ESCAPED-COMPONENT-001",
        categories: &[PageGoldCategory::EscapedSelectorComponent],
        source_id: 92_007,
        source: r"@page f\6F o{size:auto;}",
        byte_len: 24,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 8: `foo/**/:first` must qualify (comments are grammar-invisible).
fn comment_adjacent_pseudo_qualifies() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: Some(range(6, 19)),
        header: range(0, 19),
        block_opener: range(19, 20),
        body: range(20, 30),
        termination: PageGoldTermination::AuthoredRightCurly(range(30, 31)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(20, 30),
            name: range(20, 24),
            colon: range(24, 25),
            value: range(25, 29),
            priority: None,
            termination: semi(30),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-COMMENT-ADJACENT-PSEUDO-001",
        categories: &[PageGoldCategory::CommentAdjacentPseudoQualifies],
        source_id: 92_008,
        source: "@page foo/**/:first{size:auto;}",
        byte_len: 31,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 9: `foo/**/ :first` must NOT qualify (real whitespace before the
/// pseudo-page colon disqualifies).
fn real_whitespace_before_pseudo_disqualifies() -> PageGoldFixture {
    PageGoldFixture {
        id: "CSS-PAGE-REAL-WHITESPACE-DISQUALIFIES-001",
        categories: &[PageGoldCategory::RealWhitespaceBeforePseudoDisqualifies],
        source_id: 92_009,
        source: "@page foo/**/ :first{size:auto;}",
        byte_len: 32,
        roots: vec![PageGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 32),
            at_keyword: range(0, 5),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 10: leading comma disqualifies.
fn leading_comma_disqualifies() -> PageGoldFixture {
    PageGoldFixture {
        id: "CSS-PAGE-LEADING-COMMA-001",
        categories: &[PageGoldCategory::LeadingCommaDisqualifies],
        source_id: 92_010,
        source: "@page ,foo{size:auto;}",
        byte_len: 22,
        roots: vec![PageGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 22),
            at_keyword: range(0, 5),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 11: trailing comma disqualifies.
fn trailing_comma_disqualifies() -> PageGoldFixture {
    PageGoldFixture {
        id: "CSS-PAGE-TRAILING-COMMA-001",
        categories: &[PageGoldCategory::TrailingCommaDisqualifies],
        source_id: 92_011,
        source: "@page foo,{size:auto;}",
        byte_len: 22,
        roots: vec![PageGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 22),
            at_keyword: range(0, 5),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 12: empty member (double comma) disqualifies.
fn empty_member_disqualifies() -> PageGoldFixture {
    PageGoldFixture {
        id: "CSS-PAGE-EMPTY-MEMBER-001",
        categories: &[PageGoldCategory::EmptyMemberDisqualifies],
        source_id: 92_012,
        source: "@page foo,,bar{size:auto;}",
        byte_len: 26,
        roots: vec![PageGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 26),
            at_keyword: range(0, 5),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 13: unknown pseudo-page name disqualifies.
fn unknown_pseudo_disqualifies() -> PageGoldFixture {
    PageGoldFixture {
        id: "CSS-PAGE-UNKNOWN-PSEUDO-001",
        categories: &[PageGoldCategory::UnknownPseudoDisqualifies],
        source_id: 92_013,
        source: "@page :active{size:auto;}",
        byte_len: 25,
        roots: vec![PageGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 25),
            at_keyword: range(0, 5),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 14: `auto` is a structurally valid page-type ident (no
/// reserved-word exclusion, unlike `@container`).
fn auto_structurally_valid() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: Some(range(6, 10)),
        header: range(0, 10),
        block_opener: range(10, 11),
        body: range(11, 21),
        termination: PageGoldTermination::AuthoredRightCurly(range(21, 22)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(11, 21),
            name: range(11, 15),
            colon: range(15, 16),
            value: range(16, 20),
            priority: None,
            termination: semi(21),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-AUTO-VALID-001",
        categories: &[PageGoldCategory::AutoStructurallyValid],
        source_id: 92_014,
        source: "@page auto{size:auto;}",
        byte_len: 22,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 15: one margin child.
fn one_margin_child() -> PageGoldFixture {
    let margin = PageGoldMarginContext {
        id: 1,
        item_ordinal: 0,
        kind: PageGoldMarginKind::TopCenter,
        at_keyword: range(6, 17),
        header: range(6, 17),
        block_opener: range(17, 18),
        body: range(18, 30),
        termination: PageGoldTermination::AuthoredRightCurly(range(30, 31)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(18, 30),
            name: range(18, 25),
            colon: range(25, 26),
            value: range(26, 29),
            priority: None,
            termination: semi(30),
        }],
        nested_unsupported: vec![],
    };
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 31),
        termination: PageGoldTermination::AuthoredRightCurly(range(31, 32)),
        declarations: vec![],
        margins: vec![margin],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-ONE-MARGIN-001",
        categories: &[PageGoldCategory::OneMarginChild],
        source_id: 92_015,
        source: "@page{@top-center{content:'x';}}",
        byte_len: 32,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 16: all sixteen margin kinds qualify, each with an empty body.
fn all_sixteen_margin_kinds() -> PageGoldFixture {
    let entries: [(PageGoldMarginKind, usize, usize, usize, usize, usize, usize); 16] = [
        (PageGoldMarginKind::TopLeftCorner, 6, 22, 22, 23, 23, 24),
        (PageGoldMarginKind::TopLeft, 24, 33, 33, 34, 34, 35),
        (PageGoldMarginKind::TopCenter, 35, 46, 46, 47, 47, 48),
        (PageGoldMarginKind::TopRight, 48, 58, 58, 59, 59, 60),
        (PageGoldMarginKind::TopRightCorner, 60, 77, 77, 78, 78, 79),
        (
            PageGoldMarginKind::BottomLeftCorner,
            79,
            98,
            98,
            99,
            99,
            100,
        ),
        (PageGoldMarginKind::BottomLeft, 100, 112, 112, 113, 113, 114),
        (
            PageGoldMarginKind::BottomCenter,
            114,
            128,
            128,
            129,
            129,
            130,
        ),
        (
            PageGoldMarginKind::BottomRight,
            130,
            143,
            143,
            144,
            144,
            145,
        ),
        (
            PageGoldMarginKind::BottomRightCorner,
            145,
            165,
            165,
            166,
            166,
            167,
        ),
        (PageGoldMarginKind::LeftTop, 167, 176, 176, 177, 177, 178),
        (PageGoldMarginKind::LeftMiddle, 178, 190, 190, 191, 191, 192),
        (PageGoldMarginKind::LeftBottom, 192, 204, 204, 205, 205, 206),
        (PageGoldMarginKind::RightTop, 206, 216, 216, 217, 217, 218),
        (
            PageGoldMarginKind::RightMiddle,
            218,
            231,
            231,
            232,
            232,
            233,
        ),
        (
            PageGoldMarginKind::RightBottom,
            233,
            246,
            246,
            247,
            247,
            248,
        ),
    ];
    let margins: Vec<PageGoldMarginContext> = entries
        .into_iter()
        .enumerate()
        .map(
            |(
                index,
                (kind, at_start, at_end, opener_start, opener_end, close_start, close_end),
            )| {
                PageGoldMarginContext {
                    id: index + 1,
                    item_ordinal: index,
                    kind,
                    at_keyword: range(at_start, at_end),
                    header: range(at_start, at_end),
                    block_opener: range(opener_start, opener_end),
                    body: range(opener_end, opener_end),
                    termination: PageGoldTermination::AuthoredRightCurly(range(
                        close_start,
                        close_end,
                    )),
                    declarations: vec![],
                    nested_unsupported: vec![],
                }
            },
        )
        .collect();
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 248),
        termination: PageGoldTermination::AuthoredRightCurly(range(248, 249)),
        declarations: vec![],
        margins,
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-ALL-SIXTEEN-MARGIN-KINDS-001",
        categories: &[PageGoldCategory::AllSixteenMarginKinds],
        source_id: 92_016,
        source: "@page{@top-left-corner{}@top-left{}@top-center{}@top-right{}@top-right-corner{}@bottom-left-corner{}@bottom-left{}@bottom-center{}@bottom-right{}@bottom-right-corner{}@left-top{}@left-middle{}@left-bottom{}@right-top{}@right-middle{}@right-bottom{}}",
        byte_len: 249,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 17: page declaration -> margin child -> page declaration
/// ordering, without any run semantic (Amendment A).
fn page_declaration_margin_page_declaration_ordering() -> PageGoldFixture {
    let margin = PageGoldMarginContext {
        id: 1,
        item_ordinal: 1,
        kind: PageGoldMarginKind::TopCenter,
        at_keyword: range(16, 27),
        header: range(16, 27),
        block_opener: range(27, 28),
        body: range(28, 40),
        termination: PageGoldTermination::AuthoredRightCurly(range(40, 41)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(28, 40),
            name: range(28, 35),
            colon: range(35, 36),
            value: range(36, 39),
            priority: None,
            termination: semi(40),
        }],
        nested_unsupported: vec![],
    };
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 51),
        termination: PageGoldTermination::AuthoredRightCurly(range(51, 52)),
        declarations: vec![
            PageGoldOccurrence {
                item_ordinal: 0,
                complete: range(6, 16),
                name: range(6, 10),
                colon: range(10, 11),
                value: range(11, 15),
                priority: None,
                termination: semi(16),
            },
            PageGoldOccurrence {
                item_ordinal: 2,
                complete: range(41, 51),
                name: range(41, 46),
                colon: range(46, 47),
                value: range(47, 50),
                priority: None,
                termination: semi(51),
            },
        ],
        margins: vec![margin],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-DECL-MARGIN-DECL-ORDERING-001",
        categories: &[PageGoldCategory::PageDeclarationMarginPageDeclarationOrdering],
        source_id: 92_017,
        source: "@page{size:auto;@top-center{content:'x';}color:red;}",
        byte_len: 52,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 18: multiple margin children.
fn multiple_margin_children() -> PageGoldFixture {
    let margin_a = PageGoldMarginContext {
        id: 1,
        item_ordinal: 0,
        kind: PageGoldMarginKind::TopCenter,
        at_keyword: range(6, 17),
        header: range(6, 17),
        block_opener: range(17, 18),
        body: range(18, 30),
        termination: PageGoldTermination::AuthoredRightCurly(range(30, 31)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(18, 30),
            name: range(18, 25),
            colon: range(25, 26),
            value: range(26, 29),
            priority: None,
            termination: semi(30),
        }],
        nested_unsupported: vec![],
    };
    let margin_b = PageGoldMarginContext {
        id: 2,
        item_ordinal: 1,
        kind: PageGoldMarginKind::BottomCenter,
        at_keyword: range(31, 45),
        header: range(31, 45),
        block_opener: range(45, 46),
        body: range(46, 58),
        termination: PageGoldTermination::AuthoredRightCurly(range(58, 59)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(46, 58),
            name: range(46, 53),
            colon: range(53, 54),
            value: range(54, 57),
            priority: None,
            termination: semi(58),
        }],
        nested_unsupported: vec![],
    };
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 59),
        termination: PageGoldTermination::AuthoredRightCurly(range(59, 60)),
        declarations: vec![],
        margins: vec![margin_a, margin_b],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-MULTIPLE-MARGINS-001",
        categories: &[PageGoldCategory::MultipleMarginChildren],
        source_id: 92_018,
        source: "@page{@top-center{content:'a';}@bottom-center{content:'b';}}",
        byte_len: 60,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 19: same property spelling as both a page declaration and a
/// page-margin declaration remains two semantically distinct occurrence
/// types.
fn same_property_spelling_page_vs_margin() -> PageGoldFixture {
    let margin = PageGoldMarginContext {
        id: 1,
        item_ordinal: 1,
        kind: PageGoldMarginKind::TopCenter,
        at_keyword: range(16, 27),
        header: range(16, 27),
        block_opener: range(27, 28),
        body: range(28, 39),
        termination: PageGoldTermination::AuthoredRightCurly(range(39, 40)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(28, 39),
            name: range(28, 33),
            colon: range(33, 34),
            value: range(34, 38),
            priority: None,
            termination: semi(39),
        }],
        nested_unsupported: vec![],
    };
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 40),
        termination: PageGoldTermination::AuthoredRightCurly(range(40, 41)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(6, 16),
            name: range(6, 11),
            colon: range(11, 12),
            value: range(12, 15),
            priority: None,
            termination: semi(16),
        }],
        margins: vec![margin],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-SAME-SPELLING-PAGE-VS-MARGIN-001",
        categories: &[PageGoldCategory::SamePropertySpellingPageVsMargin],
        source_id: 92_019,
        source: "@page{color:red;@top-center{color:blue;}}",
        byte_len: 41,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 20: duplicate spelling under different `ContextId`s (two
/// distinct `@page` contexts).
fn duplicate_spelling_distinct_context_ids() -> PageGoldFixture {
    let page0 = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 10),
        termination: PageGoldTermination::AuthoredRightCurly(range(10, 11)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(6, 10),
            name: range(6, 7),
            colon: range(7, 8),
            value: range(8, 9),
            priority: None,
            termination: semi(10),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    let page1 = PageGoldContext {
        id: 1,
        root_item_ordinal: 1,
        at_keyword: range(11, 16),
        page_selector_list: Some(range(17, 23)),
        header: range(11, 23),
        block_opener: range(23, 24),
        body: range(24, 28),
        termination: PageGoldTermination::AuthoredRightCurly(range(28, 29)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(24, 28),
            name: range(24, 25),
            colon: range(25, 26),
            value: range(26, 27),
            priority: None,
            termination: semi(28),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-DUPLICATE-SPELLING-DISTINCT-CONTEXTS-001",
        categories: &[PageGoldCategory::DuplicateSpellingDistinctContextIds],
        source_id: 92_020,
        source: "@page{x:1;}@page :first{x:1;}",
        byte_len: 29,
        roots: vec![PageGoldRoot::Page(page0), PageGoldRoot::Page(page1)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 21: a margin candidate with a non-empty prelude remains
/// unsupported, never a `PageMarginRuleBlock`.
fn non_empty_margin_prelude_unsupported() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 35),
        termination: PageGoldTermination::AuthoredRightCurly(range(35, 36)),
        declarations: vec![],
        margins: vec![],
        nested_unsupported: vec![PageGoldNestedUnsupportedAtRule {
            item_ordinal: 0,
            complete: range(6, 35),
            at_keyword: range(6, 17),
        }],
    };
    PageGoldFixture {
        id: "CSS-PAGE-NONEMPTY-MARGIN-PRELUDE-001",
        categories: &[PageGoldCategory::NonEmptyMarginPreludeUnsupported],
        source_id: 92_021,
        source: "@page{@top-center foo{content:'x';}}",
        byte_len: 36,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 22: a top-level page-margin at-rule remains unsupported.
fn top_level_margin_rule_unsupported() -> PageGoldFixture {
    PageGoldFixture {
        id: "CSS-PAGE-TOP-LEVEL-MARGIN-001",
        categories: &[PageGoldCategory::TopLevelMarginRuleUnsupported],
        source_id: 92_022,
        source: "@top-center{content:'x';}",
        byte_len: 25,
        roots: vec![PageGoldRoot::UnsupportedAtRule {
            root_item_ordinal: 0,
            complete: range(0, 25),
            at_keyword: range(0, 11),
        }],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 23: `@page` nested directly inside a non-root supported
/// context (an ordinary qualified rule) remains unsupported: Page is
/// root-owned in #170, never inferred from any ancestor.
fn page_nested_in_non_root_supported_context_unsupported() -> PageGoldFixture {
    let companion = PageGoldQualifiedCompanion {
        id: 0,
        root_item_ordinal: 0,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 19),
        termination: PageGoldTermination::AuthoredRightCurly(range(19, 20)),
        declarations: vec![],
        nested_unsupported: vec![PageGoldNestedUnsupportedAtRule {
            item_ordinal: 0,
            complete: range(2, 19),
            at_keyword: range(2, 7),
        }],
    };
    PageGoldFixture {
        id: "CSS-PAGE-NESTED-IN-NON-ROOT-SUPPORTED-001",
        categories: &[PageGoldCategory::PageNestedInNonRootSupportedContextUnsupported],
        source_id: 92_023,
        source: "a{@page{size:auto;}}",
        byte_len: 20,
        roots: vec![PageGoldRoot::Qualified(companion)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 24: a #168 group-rule name (`@media`) inside a `PageRuleBlock`
/// remains unsupported: never inferred as a child `GroupRuleBlock`.
fn group_rule_name_inside_page_unsupported() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 31),
        termination: PageGoldTermination::AuthoredRightCurly(range(31, 32)),
        declarations: vec![],
        margins: vec![],
        nested_unsupported: vec![PageGoldNestedUnsupportedAtRule {
            item_ordinal: 0,
            complete: range(6, 31),
            at_keyword: range(6, 12),
        }],
    };
    PageGoldFixture {
        id: "CSS-PAGE-GROUP-RULE-NAME-INSIDE-PAGE-001",
        categories: &[PageGoldCategory::GroupRuleNameInsidePageUnsupported],
        source_id: 92_024,
        source: "@page{@media screen{color:red;}}",
        byte_len: 32,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 25: qualified-rule-shaped malformed content directly inside a
/// `PageRuleBlock` body is malformed/recovery evidence, never a child
/// `QualifiedRuleBlock`; a following valid item still recovers cleanly
/// (also covers category 28's "valid parent + malformed child + following
/// valid item" shape at the Page level).
fn qualified_rule_shaped_malformed_page_content() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 30),
        termination: PageGoldTermination::AuthoredRightCurly(range(30, 31)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(20, 30),
            name: range(20, 24),
            colon: range(24, 25),
            value: range(25, 29),
            priority: None,
            termination: semi(30),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-QUALIFIED-SHAPED-MALFORMED-001",
        categories: &[
            PageGoldCategory::QualifiedRuleShapedMalformedPageContent,
            PageGoldCategory::ValidParentMalformedChildFollowingValidItemRecovery,
        ],
        source_id: 92_025,
        source: "@page{a{color:red;};size:auto;}",
        byte_len: 31,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![range(6, 20)],
        recovery: vec![PageGoldRecovery {
            region: range(6, 20),
            termination: PageGoldRecoveryTermination::AuthoredSemicolon(range(19, 20)),
        }],
    }
}

/// Category 26: qualified-rule-shaped malformed content directly inside a
/// `PageMarginRuleBlock` body is malformed/recovery evidence, never a child
/// context.
fn qualified_rule_shaped_malformed_margin_content() -> PageGoldFixture {
    let margin = PageGoldMarginContext {
        id: 1,
        item_ordinal: 0,
        kind: PageGoldMarginKind::TopCenter,
        at_keyword: range(6, 17),
        header: range(6, 17),
        block_opener: range(17, 18),
        body: range(18, 44),
        termination: PageGoldTermination::AuthoredRightCurly(range(44, 45)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(32, 44),
            name: range(32, 39),
            colon: range(39, 40),
            value: range(40, 43),
            priority: None,
            termination: semi(44),
        }],
        nested_unsupported: vec![],
    };
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 45),
        termination: PageGoldTermination::AuthoredRightCurly(range(45, 46)),
        declarations: vec![],
        margins: vec![margin],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-QUALIFIED-SHAPED-MALFORMED-MARGIN-001",
        categories: &[PageGoldCategory::QualifiedRuleShapedMalformedMarginContent],
        source_id: 92_026,
        source: "@page{@top-center{a{color:red;};content:'x';}}",
        byte_len: 46,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![range(18, 32)],
        recovery: vec![PageGoldRecovery {
            region: range(18, 32),
            termination: PageGoldRecoveryTermination::AuthoredSemicolon(range(31, 32)),
        }],
    }
}

/// Category 27: a nested at-rule inside a `PageMarginRuleBlock` body remains
/// one explicit unsupported direct item, followed by a valid margin
/// declaration.
fn nested_at_rule_in_margin() -> PageGoldFixture {
    let margin = PageGoldMarginContext {
        id: 1,
        item_ordinal: 0,
        kind: PageGoldMarginKind::TopCenter,
        at_keyword: range(6, 17),
        header: range(6, 17),
        block_opener: range(17, 18),
        body: range(18, 55),
        termination: PageGoldTermination::AuthoredRightCurly(range(55, 56)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 1,
            complete: range(43, 55),
            name: range(43, 50),
            colon: range(50, 51),
            value: range(51, 54),
            priority: None,
            termination: semi(55),
        }],
        nested_unsupported: vec![PageGoldNestedUnsupportedAtRule {
            item_ordinal: 0,
            complete: range(18, 43),
            at_keyword: range(18, 24),
        }],
    };
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 56),
        termination: PageGoldTermination::AuthoredRightCurly(range(56, 57)),
        declarations: vec![],
        margins: vec![margin],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-NESTED-AT-RULE-IN-MARGIN-001",
        categories: &[PageGoldCategory::NestedAtRuleInMargin],
        source_id: 92_027,
        source: "@page{@top-center{@media screen{color:red;}content:'x';}}",
        byte_len: 57,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 28 (further coverage): a malformed non-brace item (`###;`)
/// between two valid Page declarations recovers cleanly and the following
/// valid item is retained.
fn recovery_ordering() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 30),
        termination: PageGoldTermination::AuthoredRightCurly(range(30, 31)),
        declarations: vec![
            PageGoldOccurrence {
                item_ordinal: 0,
                complete: range(6, 16),
                name: range(6, 10),
                colon: range(10, 11),
                value: range(11, 15),
                priority: None,
                termination: semi(16),
            },
            PageGoldOccurrence {
                item_ordinal: 1,
                complete: range(20, 30),
                name: range(20, 25),
                colon: range(25, 26),
                value: range(26, 29),
                priority: None,
                termination: semi(30),
            },
        ],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-RECOVERY-ORDERING-001",
        categories: &[PageGoldCategory::ValidParentMalformedChildFollowingValidItemRecovery],
        source_id: 92_028,
        source: "@page{size:auto;###;color:red;}",
        byte_len: 31,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![range(16, 20)],
        recovery: vec![PageGoldRecovery {
            region: range(16, 20),
            termination: PageGoldRecoveryTermination::AuthoredSemicolon(range(19, 20)),
        }],
    }
}

/// Category 30: true EOF while `PageRuleBlock` is active -- never fabricates
/// a closing `}`.
fn true_eof_page() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 15),
        termination: PageGoldTermination::EndOfInput(range(15, 15)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(6, 15),
            name: range(6, 10),
            colon: range(10, 11),
            value: range(11, 15),
            priority: None,
            termination: PageGoldOccurrenceTermination::OmittedAtEndOfInput(range(15, 15)),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-TRUE-EOF-001",
        categories: &[PageGoldCategory::TrueEofPage],
        source_id: 92_030,
        source: "@page{size:auto",
        byte_len: 15,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 31: true EOF while `PageMarginRuleBlock` is active preserves
/// both the Page and Margin contexts' honest partial evidence.
fn true_eof_inside_margin() -> PageGoldFixture {
    let margin = PageGoldMarginContext {
        id: 1,
        item_ordinal: 0,
        kind: PageGoldMarginKind::TopCenter,
        at_keyword: range(6, 17),
        header: range(6, 17),
        block_opener: range(17, 18),
        body: range(18, 29),
        termination: PageGoldTermination::EndOfInput(range(29, 29)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(18, 29),
            name: range(18, 25),
            colon: range(25, 26),
            value: range(26, 29),
            priority: None,
            termination: PageGoldOccurrenceTermination::OmittedAtEndOfInput(range(29, 29)),
        }],
        nested_unsupported: vec![],
    };
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 29),
        termination: PageGoldTermination::EndOfInput(range(29, 29)),
        declarations: vec![],
        margins: vec![margin],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-TRUE-EOF-INSIDE-MARGIN-001",
        categories: &[PageGoldCategory::TrueEofInsideMarginPreservesBothContexts],
        source_id: 92_031,
        source: "@page{@top-center{content:'x'",
        byte_len: 29,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: None,
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 36: `PeakContextDepth` refusal exactly at margin-context entry;
/// the Page ancestor remains retained, the Margin allocates no id.
fn peak_context_depth_refusal() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 18),
        termination: PageGoldTermination::ParserResourceLimit(range(18, 18)),
        declarations: vec![],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-PEAK-CONTEXT-DEPTH-REFUSAL-001",
        categories: &[PageGoldCategory::PeakContextDepthRefusal],
        source_id: 92_036,
        source: "@page{@top-center{content:'x';}}",
        byte_len: 32,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: Some(PageGoldResourceExpectation {
            kind: PageGoldResourceKind::PeakContextDepth,
            limit: 1,
            attempted: 2,
        }),
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 37: `ContextRecords` refusal at a second top-level `@page`
/// candidate's entry; the first Page remains retained and closed.
fn context_records_refusal() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 10),
        termination: PageGoldTermination::AuthoredRightCurly(range(10, 11)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(6, 10),
            name: range(6, 7),
            colon: range(7, 8),
            value: range(8, 9),
            priority: None,
            termination: semi(10),
        }],
        margins: vec![],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-CONTEXT-RECORDS-REFUSAL-001",
        categories: &[PageGoldCategory::ContextRecordsRefusal],
        source_id: 92_037,
        source: "@page{x:1;}@page :first{y:2;}",
        byte_len: 29,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: Some(PageGoldResourceExpectation {
            kind: PageGoldResourceKind::ContextRecords,
            limit: 1,
            attempted: 2,
        }),
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 38: the `DeclarationOccurrences` aggregate cap sums ordinary,
/// descriptor, page, and page-margin occurrences together; the fourth
/// (page-margin) commit is refused, the first three remain retained.
fn aggregate_declaration_occurrences_across_all_four() -> PageGoldFixture {
    let qualified = PageGoldQualifiedCompanion {
        id: 0,
        root_item_ordinal: 0,
        header: range(0, 1),
        block_opener: range(1, 2),
        body: range(2, 6),
        termination: PageGoldTermination::AuthoredRightCurly(range(6, 7)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(2, 6),
            name: range(2, 3),
            colon: range(3, 4),
            value: range(4, 5),
            priority: None,
            termination: semi(6),
        }],
        nested_unsupported: vec![],
    };
    let descriptor = PageGoldDescriptorCompanion {
        id: 1,
        root_item_ordinal: 1,
        at_keyword: range(7, 17),
        header: range(7, 17),
        block_opener: range(17, 18),
        body: range(18, 22),
        termination: PageGoldTermination::AuthoredRightCurly(range(22, 23)),
        occurrences: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(18, 22),
            name: range(18, 19),
            colon: range(19, 20),
            value: range(20, 21),
            priority: None,
            termination: semi(22),
        }],
    };
    let margin = PageGoldMarginContext {
        id: 3,
        item_ordinal: 1,
        kind: PageGoldMarginKind::TopCenter,
        at_keyword: range(33, 44),
        header: range(33, 44),
        block_opener: range(44, 45),
        body: range(45, 45),
        termination: PageGoldTermination::ParserResourceLimit(range(45, 45)),
        declarations: vec![],
        nested_unsupported: vec![],
    };
    let page = PageGoldContext {
        id: 2,
        root_item_ordinal: 2,
        at_keyword: range(23, 28),
        page_selector_list: None,
        header: range(23, 28),
        block_opener: range(28, 29),
        body: range(29, 45),
        termination: PageGoldTermination::ParserResourceLimit(range(45, 45)),
        declarations: vec![PageGoldOccurrence {
            item_ordinal: 0,
            complete: range(29, 33),
            name: range(29, 30),
            colon: range(30, 31),
            value: range(31, 32),
            priority: None,
            termination: semi(33),
        }],
        margins: vec![margin],
        nested_unsupported: vec![],
    };
    PageGoldFixture {
        id: "CSS-PAGE-AGGREGATE-ALL-FOUR-001",
        categories: &[PageGoldCategory::AggregateDeclarationOccurrencesAcrossAllFour],
        source_id: 92_038,
        source: "a{p:1;}@font-face{q:2;}@page{r:3;@top-center{s:4;}}",
        byte_len: 51,
        roots: vec![
            PageGoldRoot::Qualified(qualified),
            PageGoldRoot::Descriptor(descriptor),
            PageGoldRoot::Page(page),
        ],
        resource_expectation: Some(PageGoldResourceExpectation {
            kind: PageGoldResourceKind::DeclarationOccurrences,
            limit: 3,
            attempted: 4,
        }),
        diagnostics: vec![],
        recovery: vec![],
    }
}

/// Category 39: `UnsupportedRegions` refusal inside `PageRuleBlock` leaks no
/// ordinal for the refused second nested at-rule.
fn unsupported_regions_refusal_no_ordinal_leak() -> PageGoldFixture {
    let context = PageGoldContext {
        id: 0,
        root_item_ordinal: 0,
        at_keyword: range(0, 5),
        page_selector_list: None,
        header: range(0, 5),
        block_opener: range(5, 6),
        body: range(6, 25),
        termination: PageGoldTermination::ParserResourceLimit(range(25, 25)),
        declarations: vec![],
        margins: vec![],
        nested_unsupported: vec![PageGoldNestedUnsupportedAtRule {
            item_ordinal: 0,
            complete: range(6, 25),
            at_keyword: range(6, 12),
        }],
    };
    PageGoldFixture {
        id: "CSS-PAGE-UNSUPPORTED-REGIONS-REFUSAL-001",
        categories: &[PageGoldCategory::UnsupportedRegionsRefusalNoOrdinalLeak],
        source_id: 92_039,
        source: "@page{@media screen{c:1;}@layer x;}",
        byte_len: 35,
        roots: vec![PageGoldRoot::Page(context)],
        resource_expectation: Some(PageGoldResourceExpectation {
            kind: PageGoldResourceKind::UnsupportedRegions,
            limit: 1,
            attempted: 2,
        }),
        diagnostics: vec![],
        recovery: vec![],
    }
}

pub(super) fn independent_page_fixtures() -> Vec<PageGoldFixture> {
    vec![
        selector_less_page(),
        named_page(),
        pseudo_only_page(),
        named_and_pseudo_page(),
        repeated_pseudo(),
        comma_separated_selector_list(),
        escaped_selector_component(),
        comment_adjacent_pseudo_qualifies(),
        real_whitespace_before_pseudo_disqualifies(),
        leading_comma_disqualifies(),
        trailing_comma_disqualifies(),
        empty_member_disqualifies(),
        unknown_pseudo_disqualifies(),
        auto_structurally_valid(),
        one_margin_child(),
        all_sixteen_margin_kinds(),
        page_declaration_margin_page_declaration_ordering(),
        multiple_margin_children(),
        same_property_spelling_page_vs_margin(),
        duplicate_spelling_distinct_context_ids(),
        non_empty_margin_prelude_unsupported(),
        top_level_margin_rule_unsupported(),
        page_nested_in_non_root_supported_context_unsupported(),
        group_rule_name_inside_page_unsupported(),
        qualified_rule_shaped_malformed_page_content(),
        qualified_rule_shaped_malformed_margin_content(),
        nested_at_rule_in_margin(),
        recovery_ordering(),
        true_eof_page(),
        true_eof_inside_margin(),
        peak_context_depth_refusal(),
        context_records_refusal(),
        aggregate_declaration_occurrences_across_all_four(),
        unsupported_regions_refusal_no_ordinal_leak(),
    ]
}
