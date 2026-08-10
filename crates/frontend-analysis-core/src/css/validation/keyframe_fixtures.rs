//! Independently authored #171 keyframes fixtures.
//!
//! Byte ranges are written directly from the fixture source strings and the
//! approved keyframes evidence contract; they are never projected from the
//! production parser.

use super::gold::GoldRange;
use super::keyframe_gold::{
    KeyframeGoldCategory as C, KeyframeGoldContext as K, KeyframeGoldContextTermination as CT,
    KeyframeGoldFixture as F, KeyframeGoldOccurrence as O, KeyframeGoldOccurrenceTermination as OT,
    KeyframeGoldPriority as P, KeyframeGoldRecovery as R, KeyframeGoldRecoveryTermination as RT,
    KeyframeGoldUnsupported as U,
};

const fn r(start: usize, end: usize) -> GoldRange {
    GoldRange::new(start, end)
}

fn occurrence(
    context_id: usize,
    item_ordinal: usize,
    complete: (usize, usize),
    name: (usize, usize),
    colon: (usize, usize),
    value: (usize, usize),
    semicolon: (usize, usize),
) -> O {
    O {
        context_id,
        item_ordinal,
        complete: r(complete.0, complete.1),
        name: r(name.0, name.1),
        colon: r(colon.0, colon.1),
        value: r(value.0, value.1),
        priority: None,
        termination: OT::AuthoredSemicolon(r(semicolon.0, semicolon.1)),
    }
}

fn outer(
    id: usize,
    root_item_ordinal: usize,
    name: (usize, usize),
    header_end: usize,
    body: (usize, usize),
    right_curly: (usize, usize),
) -> K {
    K::Keyframes {
        id,
        root_item_ordinal,
        at_keyword: r(0, 10),
        keyframes_name: r(name.0, name.1),
        header: r(0, header_end),
        block_opener: r(header_end, header_end + 1),
        body: r(body.0, body.1),
        termination: CT::AuthoredRightCurly(r(right_curly.0, right_curly.1)),
    }
}

fn block(
    id: usize,
    parent_id: usize,
    item_ordinal: usize,
    selector: (usize, usize),
    body: (usize, usize),
    right_curly: (usize, usize),
) -> K {
    K::KeyframeBlock {
        id,
        parent_id,
        item_ordinal,
        selector_list: r(selector.0, selector.1),
        header: r(selector.0, selector.1),
        block_opener: r(selector.1, selector.1 + 1),
        body: r(body.0, body.1),
        termination: CT::AuthoredRightCurly(r(right_curly.0, right_curly.1)),
    }
}

fn basic_from_to() -> F {
    F {
        id: "CSS-KEYFRAMES-BASIC-FROM-TO-001",
        source_id: 171_001,
        source: "@keyframes spin{from{opacity:0;}to{opacity:1;}}",
        categories: vec![C::OuterNameQualification, C::FromToSelectors],
        contexts: vec![
            outer(0, 0, (11, 15), 15, (16, 46), (46, 47)),
            block(1, 0, 0, (16, 20), (21, 31), (31, 32)),
            block(2, 0, 1, (32, 34), (35, 45), (45, 46)),
        ],
        keyframe_occurrences: vec![
            occurrence(1, 0, (21, 31), (21, 28), (28, 29), (29, 30), (30, 31)),
            occurrence(2, 0, (35, 45), (35, 42), (42, 43), (43, 44), (44, 45)),
        ],
        ordinary_occurrences: vec![],
        unsupported: vec![],
        recoveries: vec![],
        diagnostics: vec![],
    }
}

fn percentage_comma_and_duplicate() -> F {
    F {
        id: "CSS-KEYFRAMES-PERCENTAGE-COMMA-DUPLICATE-001",
        source_id: 171_002,
        source: "@keyframes pulse{0%, 50%,100%{opacity:.5;}0%{opacity:0;}}",
        categories: vec![C::PercentageCommaSelectors, C::MultipleAndDuplicateBlocks],
        contexts: vec![
            outer(0, 0, (11, 16), 16, (17, 56), (56, 57)),
            block(1, 0, 0, (17, 29), (30, 41), (41, 42)),
            block(2, 0, 1, (42, 44), (45, 55), (55, 56)),
        ],
        keyframe_occurrences: vec![
            occurrence(1, 0, (30, 41), (30, 37), (37, 38), (38, 40), (40, 41)),
            occurrence(2, 0, (45, 55), (45, 52), (52, 53), (53, 54), (54, 55)),
        ],
        ordinary_occurrences: vec![],
        unsupported: vec![],
        recoveries: vec![],
        diagnostics: vec![],
    }
}

fn quoted_name() -> F {
    F {
        id: "CSS-KEYFRAMES-QUOTED-NAME-001",
        source_id: 171_003,
        source: "@keyframes \"none\"{from{x:y;}}",
        categories: vec![C::StringNameQualification],
        contexts: vec![
            outer(0, 0, (11, 17), 17, (18, 28), (28, 29)),
            block(1, 0, 0, (18, 22), (23, 27), (27, 28)),
        ],
        keyframe_occurrences: vec![occurrence(
            1,
            0,
            (23, 27),
            (23, 24),
            (24, 25),
            (25, 26),
            (26, 27),
        )],
        ordinary_occurrences: vec![],
        unsupported: vec![],
        recoveries: vec![],
        diagnostics: vec![],
    }
}

fn invalid_outer_name() -> F {
    F {
        id: "CSS-KEYFRAMES-INVALID-OUTER-NAME-001",
        source_id: 171_004,
        source: "@keyframes none{from{x:y;}}",
        categories: vec![C::InvalidOuterIsUnsupported],
        contexts: vec![],
        keyframe_occurrences: vec![],
        ordinary_occurrences: vec![],
        unsupported: vec![U::TopLevelAtRule {
            complete: r(0, 27),
            at_keyword: r(0, 10),
        }],
        recoveries: vec![],
        diagnostics: vec![],
    }
}

fn invalid_child_selector() -> F {
    F {
        id: "CSS-KEYFRAMES-INVALID-CHILD-001",
        source_id: 171_005,
        source: "@keyframes x{bogus{x:y;}from{a:b;}}",
        categories: vec![C::InvalidChildIsUnsupported],
        contexts: vec![
            outer(0, 0, (11, 12), 12, (13, 34), (34, 35)),
            block(1, 0, 1, (24, 28), (29, 33), (33, 34)),
        ],
        keyframe_occurrences: vec![occurrence(
            1,
            0,
            (29, 33),
            (29, 30),
            (30, 31),
            (31, 32),
            (32, 33),
        )],
        ordinary_occurrences: vec![],
        unsupported: vec![U::UnqualifiedKeyframeBlock {
            complete: r(13, 24),
            context_id: 0,
            item_ordinal: 0,
        }],
        recoveries: vec![],
        diagnostics: vec![],
    }
}

fn malformed_declaration_recovery() -> F {
    F {
        id: "CSS-KEYFRAMES-MALFORMED-DECLARATION-001",
        source_id: 171_006,
        source: "@keyframes x{from{broken;opacity:1;}}",
        categories: vec![C::MalformedDeclarationRecovery],
        contexts: vec![
            outer(0, 0, (11, 12), 12, (13, 36), (36, 37)),
            block(1, 0, 0, (13, 17), (18, 35), (35, 36)),
        ],
        keyframe_occurrences: vec![occurrence(
            1,
            0,
            (25, 35),
            (25, 32),
            (32, 33),
            (33, 34),
            (34, 35),
        )],
        ordinary_occurrences: vec![],
        unsupported: vec![],
        recoveries: vec![R {
            region: r(18, 25),
            termination: RT::AuthoredSemicolon(r(24, 25)),
        }],
        diagnostics: vec![r(18, 25)],
    }
}

fn true_eof_partial_contexts() -> F {
    F {
        id: "CSS-KEYFRAMES-TRUE-EOF-001",
        source_id: 171_007,
        source: "@keyframes x{from{opacity:0",
        categories: vec![C::TrueEofRetainsPartialContexts],
        contexts: vec![
            K::Keyframes {
                id: 0,
                root_item_ordinal: 0,
                at_keyword: r(0, 10),
                keyframes_name: r(11, 12),
                header: r(0, 12),
                block_opener: r(12, 13),
                body: r(13, 27),
                termination: CT::EndOfInput(r(27, 27)),
            },
            K::KeyframeBlock {
                id: 1,
                parent_id: 0,
                item_ordinal: 0,
                selector_list: r(13, 17),
                header: r(13, 17),
                block_opener: r(17, 18),
                body: r(18, 27),
                termination: CT::EndOfInput(r(27, 27)),
            },
        ],
        keyframe_occurrences: vec![O {
            context_id: 1,
            item_ordinal: 0,
            complete: r(18, 27),
            name: r(18, 25),
            colon: r(25, 26),
            value: r(26, 27),
            priority: None,
            termination: OT::OmittedAtEndOfInput(r(27, 27)),
        }],
        ordinary_occurrences: vec![],
        unsupported: vec![],
        recoveries: vec![],
        diagnostics: vec![],
    }
}

fn distinct_ordinary_and_keyframe_meaning() -> F {
    F {
        id: "CSS-KEYFRAMES-DISTINCT-OCCURRENCE-MEANING-001",
        source_id: 171_008,
        source: "a{x:y;}@keyframes k{from{x:y;}}",
        categories: vec![C::DistinctDeclarationMeaning],
        contexts: vec![
            K::Keyframes {
                id: 1,
                root_item_ordinal: 1,
                at_keyword: r(7, 17),
                keyframes_name: r(18, 19),
                header: r(7, 19),
                block_opener: r(19, 20),
                body: r(20, 30),
                termination: CT::AuthoredRightCurly(r(30, 31)),
            },
            block(2, 1, 0, (20, 24), (25, 29), (29, 30)),
        ],
        keyframe_occurrences: vec![occurrence(
            2,
            0,
            (25, 29),
            (25, 26),
            (26, 27),
            (27, 28),
            (28, 29),
        )],
        ordinary_occurrences: vec![occurrence(0, 0, (2, 6), (2, 3), (3, 4), (4, 5), (5, 6))],
        unsupported: vec![],
        recoveries: vec![],
        diagnostics: vec![],
    }
}

fn nested_keyframes_not_promoted() -> F {
    F {
        id: "CSS-KEYFRAMES-NESTED-NOT-PROMOTED-001",
        source_id: 171_009,
        source: "a{@media screen{@keyframes x{from{x:y;}}}}",
        categories: vec![C::NestedKeyframesNotPromoted],
        contexts: vec![],
        keyframe_occurrences: vec![],
        ordinary_occurrences: vec![],
        unsupported: vec![U::NestedAtRule {
            complete: r(16, 40),
            at_keyword: r(16, 26),
            context_id: 1,
            item_ordinal: 0,
        }],
        recoveries: vec![],
        diagnostics: vec![],
    }
}

fn important_is_syntax_evidence() -> F {
    F {
        id: "CSS-KEYFRAMES-IMPORTANT-SYNTAX-EVIDENCE-001",
        source_id: 171_010,
        source: "@keyframes k{from{opacity:0!important;}}",
        categories: vec![C::ImportantRemainsSyntaxEvidence],
        contexts: vec![
            outer(0, 0, (11, 12), 12, (13, 39), (39, 40)),
            block(1, 0, 0, (13, 17), (18, 38), (38, 39)),
        ],
        keyframe_occurrences: vec![O {
            context_id: 1,
            item_ordinal: 0,
            complete: r(18, 38),
            name: r(18, 25),
            colon: r(25, 26),
            value: r(26, 27),
            priority: Some(P {
                complete: r(27, 37),
                bang: r(27, 28),
                important_ident: r(28, 37),
            }),
            termination: OT::AuthoredSemicolon(r(37, 38)),
        }],
        ordinary_occurrences: vec![],
        unsupported: vec![],
        recoveries: vec![],
        diagnostics: vec![],
    }
}

pub(super) fn independent_keyframe_fixtures() -> Vec<F> {
    vec![
        basic_from_to(),
        percentage_comma_and_duplicate(),
        quoted_name(),
        invalid_outer_name(),
        invalid_child_selector(),
        malformed_declaration_recovery(),
        true_eof_partial_contexts(),
        distinct_ordinary_and_keyframe_meaning(),
        nested_keyframes_not_promoted(),
        important_is_syntax_evidence(),
    ]
}
