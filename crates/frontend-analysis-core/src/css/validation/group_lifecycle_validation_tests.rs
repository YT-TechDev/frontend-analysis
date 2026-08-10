//! Focused #168 validation-gap remediation (independent BLOCKED review,
//! comment `5235995822` on Issue #168): two lifecycle cases the 28-fixture
//! `group_context_fixtures.rs` inventory did not cover. Kept as dedicated
//! tests rather than folded into that inventory, per the accepted
//! remediation scope.
//!
//! Production code must never import this module.

use crate::css::parser::context::{
    CssParserContextKind, CssParserContextTermination, CssParserGroupRuleKind,
};
use crate::css::parser::producer::run;
use crate::css::parser::resource::{CssParserLimits, CssParserResourceKind};
use crate::css::parser::result::{CssParserExecutionCompletion, CssParserTermination};
use crate::css::token::CssLexicalItem;
use crate::css::tokenizer::producer::run as run_tokenizer;
use crate::css::tokenizer::resource::{
    CssTokenizerLimits, CssTokenizerResourceKind, CssTokenizerResourceLimitEvidence,
    CssTokenizerResourceUsage,
};
use crate::css::tokenizer::result::{
    CssTokenizerCompletion, CssTokenizerRunResult, CssTokenizerTermination,
};
use crate::{SourceId, SourceText};

fn generous_tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
}

fn generous_parser_limits() -> CssParserLimits {
    CssParserLimits::new(
        1 << 20,
        1 << 12,
        1 << 12,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
    )
    .unwrap()
}

/// Truncates a genuine, production-tokenizer-produced lexical-item prefix at
/// `terminal_offset` and wraps it in a contract-valid `Incomplete`/
/// `ResourceLimit` [`CssTokenizerRunResult`], so the parser sees the exact
/// real `AtKeyword`/`LeftCurlyBracket`/etc. token kinds a genuine tokenizer
/// run over this source would produce up to that point -- never synthetic
/// single-byte `Ident` runs, which would destroy the `@media` `AtKeyword`
/// semantics a group-rule context requires to be entered at all.
///
/// `terminal_offset` must land exactly on a boundary between two real
/// lexical items (never mid-token); this is asserted, not assumed.
fn real_prefix_as_upstream_incomplete(
    source: &SourceText,
    terminal_offset: usize,
) -> CssTokenizerRunResult {
    let complete = run_tokenizer(source, generous_tokenizer_limits())
        .expect("production tokenizer must accept this source under generous limits");

    let mut lexical_items: Vec<CssLexicalItem> = Vec::new();
    let mut cursor = 0usize;
    for item in complete.lexical_items() {
        let end = item.source().range().end();
        if end > terminal_offset {
            break;
        }
        lexical_items.push(item.clone());
        cursor = end;
    }
    assert_eq!(
        cursor, terminal_offset,
        "synthetic terminal must land exactly on a real tokenizer item boundary"
    );

    let diagnostics: Vec<_> = complete
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.location().range().end() <= terminal_offset)
        .cloned()
        .collect();

    let item_count = lexical_items.len();
    let diagnostic_count = diagnostics.len();
    let resource_limit = CssTokenizerResourceLimitEvidence::new(
        source,
        CssTokenizerResourceKind::AlgorithmSteps,
        1,
        2,
        source.anchor(terminal_offset, terminal_offset).unwrap(),
    )
    .unwrap();

    CssTokenizerRunResult::new(
        source,
        None,
        lexical_items,
        diagnostics,
        source.anchor(0, terminal_offset).unwrap(),
        source
            .anchor(terminal_offset, source.as_str().len())
            .unwrap(),
        source.anchor(terminal_offset, terminal_offset).unwrap(),
        CssTokenizerCompletion::Incomplete,
        CssTokenizerTermination::ResourceLimit(resource_limit),
        CssTokenizerResourceUsage::new(
            source.as_str().len(),
            1,
            item_count,
            diagnostic_count,
            0,
            0,
        ),
    )
    .unwrap()
}

/// Case 1 (independent BLOCKED review, comment `5235995822`): a synthetic
/// upstream-incomplete run over `QualifiedRuleBlock └─ GroupRuleBlock(@media)`
/// -- structurally equivalent to `a{@media{...`, retained short of the
/// tokenizer's genuine source end -- must already retain both the outer
/// qualified context and the already-entered inner `@media` group context,
/// both honestly terminated `UpstreamTokenizerIncomplete` at the exact same
/// bounded terminal, with no fabricated `}`.
#[test]
fn upstream_incomplete_retains_already_entered_group_context() {
    let source = SourceText::new(SourceId::new(199_001), "a{@media{p:v;}}".to_owned());
    // Truncate right after the media block opener `{` (offset 9): both
    // contexts are entered, but the group body has not yet observed any
    // content. Offset 9 is well short of the full 15-byte source, proving
    // the terminal need not sit at the retained source's end.
    let terminal_offset = 9;
    assert!(terminal_offset < source.as_str().len());

    let upstream = real_prefix_as_upstream_incomplete(&source, terminal_offset);
    let result = run(&source, upstream, generous_parser_limits())
        .expect("production parser must accept this synthetic upstream result");

    assert_eq!(
        result.termination(),
        &CssParserTermination::UpstreamTokenizerIncomplete
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );

    assert_eq!(result.context_records().len(), 2);
    let outer = &result.context_records()[0];
    let media = &result.context_records()[1];

    assert_eq!(outer.kind(), CssParserContextKind::QualifiedRuleBlock);
    assert!(outer.parent().is_none());

    assert_eq!(
        media.kind(),
        CssParserContextKind::GroupRuleBlock(CssParserGroupRuleKind::Media)
    );
    assert_eq!(media.parent(), Some(outer.id()));
    assert_eq!(media.nearest_qualified_ancestor(), Some(outer.id()));

    let at_keyword = media
        .group_at_keyword()
        .expect("retained @media group context must keep its exact at-keyword evidence");
    assert_eq!(at_keyword.range().start(), 2);
    assert_eq!(at_keyword.range().end(), 8);
    assert_eq!(at_keyword.fragment(), "@media");

    for (label, context) in [("outer", outer), ("media", media)] {
        match context.termination() {
            CssParserContextTermination::UpstreamTokenizerIncomplete { terminal } => {
                assert_eq!(
                    terminal.range().start(),
                    terminal_offset,
                    "{label}: terminal start"
                );
                assert_eq!(
                    terminal.range().end(),
                    terminal_offset,
                    "{label}: terminal end"
                );
            }
            other => panic!("{label}: expected UpstreamTokenizerIncomplete, got {other:?}"),
        }
    }
}

/// Case 2 (independent BLOCKED review, comment `5235995822`): a
/// `DeclarationOccurrences` refusal after a `GroupRuleBlock(@media)` has
/// already been entered must leave both the outer qualified context and the
/// inner group context retained, both honestly terminated
/// `ParserResourceLimit` at the exact same committed parser terminal, with
/// the first declaration surviving and the second never committed.
#[test]
fn declaration_occurrences_limit_stops_after_group_entry() {
    let source = SourceText::new(SourceId::new(199_002), "a{@media{p:v;q:w;}}".to_owned());
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits()).unwrap();
    let limits = CssParserLimits::new(
        1 << 20,
        1 << 12,
        1 << 12,
        1, // max_declaration_occurrences
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
    )
    .unwrap();

    let result = run(&source, tokenizer_result, limits)
        .expect("resource-limited runs remain a normal Ok result");

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(
                evidence.kind(),
                CssParserResourceKind::DeclarationOccurrences
            );
            assert_eq!(evidence.limit(), 1);
            assert_eq!(evidence.attempted(), 2);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }

    assert_eq!(result.context_records().len(), 2);
    let outer = &result.context_records()[0];
    let media = &result.context_records()[1];

    assert_eq!(outer.kind(), CssParserContextKind::QualifiedRuleBlock);
    assert_eq!(
        media.kind(),
        CssParserContextKind::GroupRuleBlock(CssParserGroupRuleKind::Media)
    );
    assert_eq!(media.parent(), Some(outer.id()));
    assert_eq!(media.nearest_qualified_ancestor(), Some(outer.id()));

    let at_keyword = media
        .group_at_keyword()
        .expect("retained @media group context must keep its exact at-keyword evidence");
    assert_eq!(at_keyword.range().start(), 2);
    assert_eq!(at_keyword.range().end(), 8);
    assert_eq!(at_keyword.fragment(), "@media");

    for (label, context) in [("outer", outer), ("media", media)] {
        match context.termination() {
            CssParserContextTermination::ParserResourceLimit { terminal } => {
                assert_eq!(terminal.range().start(), 13, "{label}: terminal start");
                assert_eq!(terminal.range().end(), 13, "{label}: terminal end");
            }
            other => panic!("{label}: expected ParserResourceLimit, got {other:?}"),
        }
    }

    assert_eq!(
        result.occurrences().len(),
        1,
        "second declaration must not commit"
    );
    let first = &result.occurrences()[0];
    assert_eq!(first.placement().context_id(), media.id());
    assert_eq!(first.placement().item_ordinal().value(), 0);
    assert_eq!(first.placement().run_ordinal().value(), 0);
    assert_eq!(first.complete().range().start(), 9);
    assert_eq!(first.complete().range().end(), 13);

    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::DeclarationOccurrences),
        1
    );
}
