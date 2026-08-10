//! Candidate-independent normative parser-level gold fixtures (#138).
//!
//! Every fixture is authored directly from the pinned normative specs
//! (CSS Syntax Module Level 3, Editor's Draft 10 June 2026; CSS Nesting
//! Module Level 1, Editor's Draft 5 February 2026) and the #137/#138
//! approved architecture. None is derived from a production parser,
//! external parser, CSSOM, or browser output.

use super::gold::GoldRange;
use super::parser_gold::{
    ParserGoldDeclaration, ParserGoldDiagnostic, ParserGoldDiagnosticCode, ParserGoldDiscard,
    ParserGoldFixture, ParserGoldGroup, ParserGoldPriority, ParserGoldRecovery,
    ParserGoldRecoveryTermination, ParserGoldTermination, ParserGoldUnsupportedRegion,
};

fn r(start: usize, end: usize) -> GoldRange {
    GoldRange::new(start, end)
}

#[allow(clippy::too_many_arguments)]
fn decl_semi(
    complete: (usize, usize),
    property_name: (usize, usize),
    colon: (usize, usize),
    value: (usize, usize),
    semicolon: (usize, usize),
) -> ParserGoldDeclaration {
    ParserGoldDeclaration {
        complete: r(complete.0, complete.1),
        property_name: r(property_name.0, property_name.1),
        colon: r(colon.0, colon.1),
        value: r(value.0, value.1),
        priority: None,
        termination: ParserGoldTermination::AuthoredSemicolon(r(semicolon.0, semicolon.1)),
    }
}

#[allow(clippy::too_many_arguments)]
fn decl_omit_curly(
    complete: (usize, usize),
    property_name: (usize, usize),
    colon: (usize, usize),
    value: (usize, usize),
    right_curly: (usize, usize),
) -> ParserGoldDeclaration {
    ParserGoldDeclaration {
        complete: r(complete.0, complete.1),
        property_name: r(property_name.0, property_name.1),
        colon: r(colon.0, colon.1),
        value: r(value.0, value.1),
        priority: None,
        termination: ParserGoldTermination::OmittedBeforeRightCurly(r(
            right_curly.0,
            right_curly.1,
        )),
    }
}

fn decl_omit_eof(
    complete: (usize, usize),
    property_name: (usize, usize),
    colon: (usize, usize),
    value: (usize, usize),
    eof_point: usize,
) -> ParserGoldDeclaration {
    ParserGoldDeclaration {
        complete: r(complete.0, complete.1),
        property_name: r(property_name.0, property_name.1),
        colon: r(colon.0, colon.1),
        value: r(value.0, value.1),
        priority: None,
        termination: ParserGoldTermination::OmittedAtEndOfInput(r(eof_point, eof_point)),
    }
}

#[allow(clippy::too_many_arguments)]
fn decl_with_priority(
    complete: (usize, usize),
    property_name: (usize, usize),
    colon: (usize, usize),
    value: (usize, usize),
    priority: (usize, usize, usize, usize, usize, usize),
    semicolon: (usize, usize),
) -> ParserGoldDeclaration {
    let (p_complete_start, p_complete_end, bang_start, bang_end, ident_start, ident_end) = priority;
    ParserGoldDeclaration {
        complete: r(complete.0, complete.1),
        property_name: r(property_name.0, property_name.1),
        colon: r(colon.0, colon.1),
        value: r(value.0, value.1),
        priority: Some(ParserGoldPriority {
            complete: r(p_complete_start, p_complete_end),
            bang: r(bang_start, bang_end),
            important_ident: r(ident_start, ident_end),
        }),
        termination: ParserGoldTermination::AuthoredSemicolon(r(semicolon.0, semicolon.1)),
    }
}

fn recovery_semi(region: (usize, usize), semicolon: (usize, usize)) -> ParserGoldRecovery {
    ParserGoldRecovery {
        region: r(region.0, region.1),
        termination: ParserGoldRecoveryTermination::AuthoredSemicolon(r(semicolon.0, semicolon.1)),
    }
}

/// `terminal_point` is the exact empty-point offset at retained source end.
fn recovery_eof(region: (usize, usize), terminal_point: usize) -> ParserGoldRecovery {
    ParserGoldRecovery {
        region: r(region.0, region.1),
        termination: ParserGoldRecoveryTermination::EndOfInput(r(terminal_point, terminal_point)),
    }
}

fn diagnostic(code: ParserGoldDiagnosticCode, location: (usize, usize)) -> ParserGoldDiagnostic {
    ParserGoldDiagnostic {
        code,
        location: r(location.0, location.1),
    }
}

fn top_level_at_rule(
    complete: (usize, usize),
    at_keyword: (usize, usize),
) -> ParserGoldUnsupportedRegion {
    ParserGoldUnsupportedRegion::TopLevelAtRule {
        complete: r(complete.0, complete.1),
        at_keyword: r(at_keyword.0, at_keyword.1),
    }
}

fn nested_remainder(region: (usize, usize)) -> ParserGoldUnsupportedRegion {
    ParserGoldUnsupportedRegion::NestedContentRemainder {
        region: r(region.0, region.1),
    }
}

fn discard_top_level_custom_property_like(
    region: (usize, usize),
    property_name: (usize, usize),
    colon: (usize, usize),
) -> ParserGoldDiscard {
    ParserGoldDiscard::TopLevelCustomPropertyLikeQualifiedRule {
        region: r(region.0, region.1),
        property_name: r(property_name.0, property_name.1),
        colon: r(colon.0, colon.1),
    }
}

/// The #99 escaped-property provenance regression.
fn historical_provenance_fixture() -> ParserGoldFixture {
    ParserGoldFixture::complete(
        "CSS-PARSER-HIST-PROVENANCE-001",
        ParserGoldGroup::Historical,
        2701,
        "a{c\\6F lor/**/:red;}",
        20,
        vec![decl_semi((2, 19), (2, 10), (14, 15), (15, 18), (18, 19))],
        vec![],
        vec![],
        vec![],
    )
}

/// The #102 G1-04 silent-priority-loss regression.
fn historical_priority_fixture() -> ParserGoldFixture {
    ParserGoldFixture::complete(
        "CSS-PARSER-HIST-PRIORITY-001",
        ParserGoldGroup::Historical,
        2702,
        "a{color:red !/**/Im\\70 ortant;}",
        31,
        vec![decl_with_priority(
            (2, 30),
            (2, 7),
            (7, 8),
            (8, 11),
            (12, 29, 12, 13, 17, 29),
            (29, 30),
        )],
        vec![],
        vec![],
        vec![],
    )
}

/// #158: `consume a qualified rule` with `nested = false`, whose prelude's
/// first two non-whitespace values are a custom-property-shaped Ident
/// followed by a Colon. The qualified rule is not returned; its block is
/// structurally consumed as discard evidence rather than becoming a
/// supported declaration context, an unsupported region, or recovery.
fn discard_top_level_custom_property_like_qualified_rule_fixture() -> ParserGoldFixture {
    ParserGoldFixture::complete(
        "CSS-PARSER-DISCARD-TOP-LEVEL-CUSTOM-PROPERTY-LIKE-001",
        ParserGoldGroup::Discard,
        2033,
        "--foo:bar{color:red;}",
        21,
        vec![],
        vec![],
        vec![],
        vec![],
    )
    .with_discard(vec![discard_top_level_custom_property_like(
        (0, 21),
        (0, 5),
        (5, 6),
    )])
}

/// The full normative, candidate-independent #138 parser-level gold matrix.
pub(super) fn normative_parser_fixtures() -> Vec<ParserGoldFixture> {
    vec![
        historical_provenance_fixture(),
        historical_priority_fixture(),
        discard_top_level_custom_property_like_qualified_rule_fixture(),
        // Basic declaration with an authored semicolon.
        ParserGoldFixture::complete(
            "CSS-PARSER-BASIC-SEMICOLON-001",
            ParserGoldGroup::Basic,
            2001,
            "a{color:red;}",
            13,
            vec![decl_semi((2, 12), (2, 7), (7, 8), (8, 11), (11, 12))],
            vec![],
            vec![],
            vec![],
        ),
        // Omitted semicolon before the enclosing right curly.
        ParserGoldFixture::complete(
            "CSS-PARSER-TERM-OMITTED-BEFORE-RIGHT-CURLY-001",
            ParserGoldGroup::Termination,
            2002,
            "a{color:red}",
            12,
            vec![decl_omit_curly((2, 11), (2, 7), (7, 8), (8, 11), (11, 12))],
            vec![],
            vec![],
            vec![],
        ),
        // Omitted semicolon at true end of input; the enclosing block was
        // never authored-closed and closes automatically.
        ParserGoldFixture::complete(
            "CSS-PARSER-TERM-OMITTED-AT-EOF-001",
            ParserGoldGroup::Termination,
            2003,
            "a{color:red",
            11,
            vec![decl_omit_eof((2, 11), (2, 7), (7, 8), (8, 11), 11)],
            vec![],
            vec![],
            vec![],
        ),
        // Multiple declarations in deterministic source order.
        ParserGoldFixture::complete(
            "CSS-PARSER-MULTIPLE-DECLARATIONS-001",
            ParserGoldGroup::Basic,
            2004,
            "a{color:red;background:blue;}",
            29,
            vec![
                decl_semi((2, 12), (2, 7), (7, 8), (8, 11), (11, 12)),
                decl_semi((12, 28), (12, 22), (22, 23), (23, 27), (27, 28)),
            ],
            vec![],
            vec![],
            vec![],
        ),
        // Empty value: nothing but the semicolon after the colon.
        ParserGoldFixture::complete(
            "CSS-PARSER-VALUE-EMPTY-001",
            ParserGoldGroup::Values,
            2005,
            "a{color:;}",
            10,
            vec![decl_semi((2, 9), (2, 7), (7, 8), (8, 8), (8, 9))],
            vec![],
            vec![],
            vec![],
        ),
        // Whitespace/comment-only value collapses to a zero-length value at
        // colon.end while the comment remains inside `complete`.
        ParserGoldFixture::complete(
            "CSS-PARSER-VALUE-WHITESPACE-COMMENT-ONLY-001",
            ParserGoldGroup::Values,
            2006,
            "a{color: /**/ ;}",
            16,
            vec![decl_semi((2, 15), (2, 7), (7, 8), (8, 8), (14, 15))],
            vec![],
            vec![],
            vec![],
        ),
        // Boundary whitespace/comments around name, colon, value, and
        // terminator all remain inside `complete` but outside their
        // neighboring exact-evidence components.
        ParserGoldFixture::complete(
            "CSS-PARSER-BOUNDARY-TRIVIA-001",
            ParserGoldGroup::Values,
            2007,
            "a{ /*c1*/ color /*c2*/ : /*c3*/ red /*c4*/ ;}",
            45,
            vec![decl_semi((10, 44), (10, 15), (23, 24), (32, 35), (43, 44))],
            vec![],
            vec![],
            vec![],
        ),
        // Mixed-case property name.
        ParserGoldFixture::complete(
            "CSS-PARSER-PROPERTY-MIXED-CASE-001",
            ParserGoldGroup::PropertyNames,
            2008,
            "a{Color:red;}",
            13,
            vec![decl_semi((2, 12), (2, 7), (7, 8), (8, 11), (11, 12))],
            vec![],
            vec![],
            vec![],
        ),
        // Vendor-prefixed property name.
        ParserGoldFixture::complete(
            "CSS-PARSER-PROPERTY-VENDOR-PREFIX-001",
            ParserGoldGroup::PropertyNames,
            2009,
            "a{-webkit-transform:none;}",
            26,
            vec![decl_semi((2, 25), (2, 19), (19, 20), (20, 24), (24, 25))],
            vec![],
            vec![],
            vec![],
        ),
        // Unknown/future property-shaped name remains an eligible syntax
        // occurrence.
        ParserGoldFixture::complete(
            "CSS-PARSER-PROPERTY-UNKNOWN-FUTURE-001",
            ParserGoldGroup::PropertyNames,
            2010,
            "a{fooBarBaz:1;}",
            15,
            vec![decl_semi((2, 14), (2, 11), (11, 12), (12, 13), (13, 14))],
            vec![],
            vec![],
            vec![],
        ),
        // Custom property with a `{}` value preserved as ordinary syntax.
        ParserGoldFixture::complete(
            "CSS-PARSER-PROPERTY-CUSTOM-BRACE-VALUE-001",
            ParserGoldGroup::PropertyNames,
            2011,
            "a{--x:{a:b};}",
            13,
            vec![decl_semi((2, 12), (2, 5), (5, 6), (6, 11), (11, 12))],
            vec![],
            vec![],
            vec![],
        ),
        // Unicode/multibyte property and value evidence, byte-exact.
        ParserGoldFixture::complete(
            "CSS-PARSER-UNICODE-MULTIBYTE-001",
            ParserGoldGroup::PropertyNames,
            2012,
            "a{--é:中;}",
            12,
            vec![decl_semi((2, 11), (2, 6), (6, 7), (7, 10), (10, 11))],
            vec![],
            vec![],
            vec![],
        ),
        // A string value containing a semicolon and braces is ordinary,
        // opaque value content, not a terminator or nested block.
        ParserGoldFixture::complete(
            "CSS-PARSER-STRING-WITH-SEMICOLON-BRACE-001",
            ParserGoldGroup::Values,
            2013,
            "a{content:\"a;{b}\";}",
            19,
            vec![decl_semi((2, 18), (2, 9), (9, 10), (10, 17), (17, 18))],
            vec![],
            vec![],
            vec![],
        ),
        // Nested functions and `()`/`[]` component blocks stay inside one
        // contiguous value envelope.
        ParserGoldFixture::complete(
            "CSS-PARSER-NESTED-FUNCTIONS-BRACKETS-001",
            ParserGoldGroup::Values,
            2014,
            "a{grid-template-columns:[full] minmax(min(1px,2px),1fr) [full-end];}",
            68,
            vec![decl_semi((2, 67), (2, 23), (23, 24), (24, 66), (66, 67))],
            vec![],
            vec![],
            vec![],
        ),
        // A non-custom property whose value is solely one top-level `{}`
        // block remains a recognized declaration in the #137 syntax
        // boundary.
        ParserGoldFixture::complete(
            "CSS-PARSER-NONCUSTOM-BRACE-SOLE-VALUE-001",
            ParserGoldGroup::Values,
            2015,
            "a{color:{red};}",
            15,
            vec![decl_semi((2, 14), (2, 7), (7, 8), (8, 13), (13, 14))],
            vec![],
            vec![],
            vec![],
        ),
        // A non-custom `{}` block combined with another top-level value
        // component is not a declaration; declaration interpretation fails
        // and qualified-rule fallback commits with no declaration-failure
        // evidence (rollback no-leak). #167 supersedes the old whole-
        // remainder outcome: the fallback now structurally recognizes
        // "color:green" as a nested qualified-rule prelude and retains a
        // real child context, so the inner "color:blue" declaration is a
        // supported occurrence rather than swallowed unsupported content.
        ParserGoldFixture::complete(
            "CSS-PARSER-NONCUSTOM-BRACE-PLUS-OTHER-001",
            ParserGoldGroup::Rollback,
            2016,
            "a{color:green{color:blue;}}",
            27,
            vec![decl_semi((14, 25), (14, 19), (19, 20), (20, 24), (24, 25))],
            vec![],
            vec![],
            vec![],
        ),
        // A simple valid `!important` with ordinary boundary whitespace.
        ParserGoldFixture::complete(
            "CSS-PARSER-PRIORITY-SIMPLE-001",
            ParserGoldGroup::Priority,
            2017,
            "a{color:red !important;}",
            24,
            vec![decl_with_priority(
                (2, 23),
                (2, 7),
                (7, 8),
                (8, 11),
                (12, 22, 12, 13, 13, 22),
                (22, 23),
            )],
            vec![],
            vec![],
            vec![],
        ),
        // Mixed-case, escaped-code-point valid `!important` (distinct from
        // the mandatory historical fixture, which also has an intervening
        // comment).
        ParserGoldFixture::complete(
            "CSS-PARSER-PRIORITY-MIXED-CASE-ESCAPED-001",
            ParserGoldGroup::Priority,
            2018,
            "a{color:red !iM\\50 ortant;}",
            27,
            vec![decl_with_priority(
                (2, 26),
                (2, 7),
                (7, 8),
                (8, 11),
                (12, 25, 12, 13, 13, 25),
                (25, 26),
            )],
            vec![],
            vec![],
            vec![],
        ),
        // An invalid priority-shaped suffix produces no priority evidence
        // and remains ordinary value syntax.
        ParserGoldFixture::complete(
            "CSS-PARSER-PRIORITY-INVALID-SUFFIX-001",
            ParserGoldGroup::Priority,
            2019,
            "a{color:red !impotent;}",
            23,
            vec![decl_semi((2, 22), (2, 7), (7, 8), (8, 21), (21, 22))],
            vec![],
            vec![],
            vec![],
        ),
        // A malformed block item missing a colon is recovered up through
        // its authored semicolon; a later valid declaration is unaffected.
        ParserGoldFixture::complete(
            "CSS-PARSER-MALFORMED-MISSING-COLON-THEN-VALID-001",
            ParserGoldGroup::Malformed,
            2020,
            "a{color red;background:blue;}",
            29,
            vec![decl_semi((12, 28), (12, 22), (22, 23), (23, 27), (27, 28))],
            vec![diagnostic(
                ParserGoldDiagnosticCode::InvalidBlockItem,
                (2, 12),
            )],
            vec![recovery_semi((2, 12), (11, 12))],
            vec![],
        ),
        // A malformed declaration start (no leading ident) is recovered up
        // through its authored semicolon; a later valid declaration is
        // unaffected.
        ParserGoldFixture::complete(
            "CSS-PARSER-MALFORMED-START-THEN-VALID-001",
            ParserGoldGroup::Malformed,
            2021,
            "a{:red;color:blue;}",
            19,
            vec![decl_semi((7, 18), (7, 12), (12, 13), (13, 17), (17, 18))],
            vec![diagnostic(
                ParserGoldDiagnosticCode::InvalidBlockItem,
                (2, 7),
            )],
            vec![recovery_semi((2, 7), (6, 7))],
            vec![],
        ),
        // A malformed block item that reaches true tokenizer end of input
        // without an authored semicolon or authored enclosing right curly
        // (#159). Recovery preserves the malformed authored bytes through
        // an explicit empty EOF terminal at retained source end; no
        // semicolon or right curly is fabricated.
        ParserGoldFixture::complete(
            "CSS-PARSER-MALFORMED-AT-TRUE-EOF-001",
            ParserGoldGroup::Malformed,
            2034,
            "a{color red",
            11,
            vec![],
            vec![diagnostic(
                ParserGoldDiagnosticCode::InvalidBlockItem,
                (2, 11),
            )],
            vec![recovery_eof((2, 11), 11)],
            vec![],
        ),
        // Leading supported declaration, a nested qualified rule with its
        // own declaration, then a trailing declaration back in the outer
        // context. #167 supersedes the old whole-remainder outcome: the
        // nested rule is now a real child context, so all three
        // declarations are supported occurrences in source order (outer
        // run 0, child run 0, outer run 1).
        ParserGoldFixture::complete(
            "CSS-PARSER-NESTING-DECL-THEN-QUALIFIED-RULE-001",
            ParserGoldGroup::Nesting,
            2022,
            "a{color:red;b{color:blue;}color:green;}",
            39,
            vec![
                decl_semi((2, 12), (2, 7), (7, 8), (8, 11), (11, 12)),
                decl_semi((14, 25), (14, 19), (19, 20), (20, 24), (24, 25)),
                decl_semi((26, 38), (26, 31), (31, 32), (32, 37), (37, 38)),
            ],
            vec![],
            vec![],
            vec![],
        ),
        // #168 supersedes the old whole-remainder outcome: `@media screen`
        // enters the bounded #168 subset (an empty-or-`all`/`screen`/`print`
        // media-type prelude), so the nested at-rule becomes a supported
        // group context rather than unsupported evidence, and its own
        // declaration is a normal supported occurrence in source order.
        // Parser-level gold is flat across contexts (see the companion
        // `CSS-CONTEXT-*` group-rule gold for structural context evidence),
        // so both declarations appear here with no unsupported region.
        ParserGoldFixture::complete(
            "CSS-PARSER-NESTING-DECL-THEN-AT-RULE-001",
            ParserGoldGroup::Nesting,
            2023,
            "a{color:red;@media screen{color:blue;}}",
            39,
            vec![
                decl_semi((2, 12), (2, 7), (7, 8), (8, 11), (11, 12)),
                decl_semi((26, 37), (26, 31), (31, 32), (32, 36), (36, 37)),
            ],
            vec![],
            vec![],
            vec![],
        ),
        // #169 supersedes the old whole-remainder outcome: an empty-prelude
        // top-level `@font-face` now qualifies as a supported descriptor
        // context, so it is no longer top-level-unsupported evidence. The
        // declaration-shaped `font-family:x;` inside it becomes a
        // `CssDescriptorOccurrence`, not an ordinary `CssDeclarationOccurrence`
        // -- this flat parser-level gold has no descriptor-occurrence
        // concept (see the companion `CSS-CONTEXT-*`/descriptor-focused
        // producer tests for that structural evidence), so both remain
        // empty here.
        ParserGoldFixture::complete(
            "CSS-PARSER-UNSUPPORTED-FONT-FACE-001",
            ParserGoldGroup::UnsupportedAtRule,
            2024,
            "@font-face{font-family:x;}",
            26,
            vec![],
            vec![],
            vec![],
            vec![],
        ),
        // A top-level `@keyframes` is structurally consumed but unsupported;
        // no keyframe declaration is extracted.
        ParserGoldFixture::complete(
            "CSS-PARSER-UNSUPPORTED-KEYFRAMES-001",
            ParserGoldGroup::UnsupportedAtRule,
            2025,
            "@keyframes spin{from{opacity:0;}to{opacity:1;}}",
            47,
            vec![],
            vec![],
            vec![],
            vec![top_level_at_rule((0, 47), (0, 10))],
        ),
        // A top-level `@media` containing style rules is unsupported; no
        // descendant occurrence is extracted from its block in this first
        // slice.
        ParserGoldFixture::complete(
            "CSS-PARSER-UNSUPPORTED-MEDIA-WITH-STYLE-RULES-001",
            ParserGoldGroup::UnsupportedAtRule,
            2026,
            "@media screen{a{color:red;}}",
            28,
            vec![],
            vec![],
            vec![],
            vec![top_level_at_rule((0, 28), (0, 6))],
        ),
        // #169 supersedes the old whole-remainder outcome: the leading
        // empty-prelude top-level `@font-face` now qualifies as a supported
        // descriptor context rather than an unsupported region (see the
        // `CSS-PARSER-UNSUPPORTED-FONT-FACE-001` supersession comment
        // above). The later top-level qualified rule's declaration remains
        // a normal supported occurrence at its unchanged source offsets.
        ParserGoldFixture::complete(
            "CSS-PARSER-UNSUPPORTED-AT-RULE-THEN-SUPPORTED-RULE-001",
            ParserGoldGroup::UnsupportedAtRule,
            2027,
            "@font-face{font-family:x;}a{color:red;}",
            39,
            vec![decl_semi((28, 38), (28, 33), (33, 34), (34, 37), (37, 38))],
            vec![],
            vec![],
            vec![],
        ),
        // Duplicate identical declaration spellings at distinct offsets
        // remain distinct occurrences.
        ParserGoldFixture::complete(
            "CSS-PARSER-DUPLICATE-IDENTICAL-DECLARATIONS-001",
            ParserGoldGroup::Basic,
            2028,
            "a{color:red;color:red;}",
            23,
            vec![
                decl_semi((2, 12), (2, 7), (7, 8), (8, 11), (11, 12)),
                decl_semi((12, 22), (12, 17), (17, 18), (18, 21), (21, 22)),
            ],
            vec![],
            vec![],
            vec![],
        ),
        // A BOM precedes the first rule; declaration ranges remain exact
        // raw byte offsets after the 3-byte BOM.
        ParserGoldFixture::complete(
            "CSS-PARSER-BOM-BEFORE-FIRST-RULE-001",
            ParserGoldGroup::InputPreprocessing,
            2029,
            "\u{feff}a{color:red;}",
            16,
            vec![decl_semi((5, 15), (5, 10), (10, 11), (11, 14), (14, 15))],
            vec![],
            vec![],
            vec![],
        ),
        // CR, LF, CRLF, and FF preprocessing-sensitive raw positions between
        // declarations: each declaration's own exact raw range is unaffected
        // by the following raw newline form.
        ParserGoldFixture::complete(
            "CSS-PARSER-CR-LF-CRLF-FF-RAW-MAPPING-001",
            ParserGoldGroup::InputPreprocessing,
            2030,
            "a{a1:1;\ra2:2;\na3:3;\r\na4:4;\x0ca5:5;}",
            33,
            vec![
                decl_semi((2, 7), (2, 4), (4, 5), (5, 6), (6, 7)),
                decl_semi((8, 13), (8, 10), (10, 11), (11, 12), (12, 13)),
                decl_semi((14, 19), (14, 16), (16, 17), (17, 18), (18, 19)),
                decl_semi((21, 26), (21, 23), (23, 24), (24, 25), (25, 26)),
                decl_semi((27, 32), (27, 29), (29, 30), (30, 31), (31, 32)),
            ],
            vec![],
            vec![],
            vec![],
        ),
        // A raw U+0000 byte is replacement-sensitive tokenizer syntax that
        // still yields ordinary byte-exact value evidence.
        ParserGoldFixture::complete(
            "CSS-PARSER-NUL-REPLACEMENT-SENSITIVE-001",
            ParserGoldGroup::Values,
            2031,
            "a{color:\0;}",
            11,
            vec![decl_semi((2, 10), (2, 7), (7, 8), (8, 9), (9, 10))],
            vec![],
            vec![],
            vec![],
        ),
        // Upstream tokenizer incompleteness can never become parser-complete
        // or an ordinary omitted-at-EOF termination.
        ParserGoldFixture::upstream_incomplete(
            "CSS-PARSER-LIFECYCLE-TOKENIZER-INCOMPLETE-001",
            ParserGoldGroup::Lifecycle,
            2032,
            "a{color:red;}",
            13,
            r(5, 5),
        ),
    ]
}
