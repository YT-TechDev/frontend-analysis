//! Focused #170 validation-gap coverage: category 32/33 (synthetic
//! upstream-tokenizer-incomplete while a `PageRuleBlock`/`PageMarginRuleBlock`
//! is already active) and category 34/35 (a generic parser-resource stop
//! after a `PageRuleBlock`/`PageMarginRuleBlock` has already been entered),
//! kept as dedicated tests rather than folded into `page_fixtures.rs`'s
//! generic gold-comparison inventory: categories 32/33 require a genuine
//! tokenizer-produced lexical prefix, and categories 34/35 require a
//! production-executed proof rather than a contract-only gold fixture --
//! mirroring how #169 covered its own analogous lifecycle gaps in
//! `descriptor_lifecycle_validation_tests.rs`.
//!
//! Production code must never import this module.

use crate::css::parser::context::{
    CssParserContextKind, CssParserContextTermination, CssParserPageMarginRuleKind,
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
/// `terminal_offset` and wraps it in a contract-valid `Incomplete` /
/// `ResourceLimit` [`CssTokenizerRunResult`], so the parser sees the exact
/// real `AtKeyword`/`LeftCurlyBracket`/etc. token kinds a genuine tokenizer
/// run over this source would produce up to that point -- mirrors #169's
/// identically named helper.
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

/// Category 32: a synthetic upstream-incomplete run over `PageRuleBlock` --
/// structurally equivalent to `@page{size:auto`, retained short of the
/// tokenizer's genuine source end -- must already retain the already-entered
/// Page context, honestly terminated `UpstreamTokenizerIncomplete` at the
/// exact bounded terminal, with no fabricated `}`.
#[test]
fn upstream_incomplete_retains_already_entered_page_context() {
    let source = SourceText::new(SourceId::new(199_201), "@page{size:auto;}".to_owned());
    // Truncate right after the Page block opener `{` (offset 6): the
    // context is entered, but its body has not yet observed any content.
    let terminal_offset = 6;
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

    assert_eq!(result.context_records().len(), 1);
    let page = &result.context_records()[0];
    assert_eq!(page.kind(), CssParserContextKind::PageRuleBlock);
    assert!(page.parent().is_none());
    assert!(page.page_selector_list().is_none());

    let at_keyword = page
        .at_keyword()
        .expect("retained @page context must keep its exact at-keyword evidence");
    assert_eq!(at_keyword.range().start(), 0);
    assert_eq!(at_keyword.range().end(), 5);
    assert_eq!(at_keyword.fragment(), "@page");

    match page.termination() {
        CssParserContextTermination::UpstreamTokenizerIncomplete { terminal } => {
            assert_eq!(terminal.range().start(), terminal_offset);
            assert_eq!(terminal.range().end(), terminal_offset);
        }
        other => panic!("expected UpstreamTokenizerIncomplete, got {other:?}"),
    }

    assert!(result.page_occurrences().is_empty());
    assert!(result.page_margin_occurrences().is_empty());
}

/// Category 33: a synthetic upstream-incomplete run over `PageMarginRuleBlock`
/// -- structurally equivalent to `@page{@top-center{content:'x'`, retained
/// short of the tokenizer's genuine source end -- must retain both the Page
/// ancestor and the Margin context, both honestly terminated
/// `UpstreamTokenizerIncomplete` at the same exact bounded terminal.
#[test]
fn upstream_incomplete_retains_both_page_and_margin_contexts() {
    let source = SourceText::new(
        SourceId::new(199_202),
        "@page{@top-center{content:'x';}}".to_owned(),
    );
    // Truncate right after the Margin block opener `{` (offset 18): both
    // contexts are entered, but the Margin's body has not yet observed any
    // content.
    let terminal_offset = 18;
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
    let page = &result.context_records()[0];
    let margin = &result.context_records()[1];

    assert_eq!(page.kind(), CssParserContextKind::PageRuleBlock);
    assert!(page.parent().is_none());
    match page.termination() {
        CssParserContextTermination::UpstreamTokenizerIncomplete { terminal } => {
            assert_eq!(terminal.range().start(), terminal_offset);
        }
        other => panic!("expected UpstreamTokenizerIncomplete, got {other:?}"),
    }

    assert_eq!(
        margin.kind(),
        CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::TopCenter)
    );
    assert_eq!(margin.parent(), Some(page.id()));
    match margin.termination() {
        CssParserContextTermination::UpstreamTokenizerIncomplete { terminal } => {
            assert_eq!(terminal.range().start(), terminal_offset);
            assert_eq!(terminal.range().end(), terminal_offset);
        }
        other => panic!("expected UpstreamTokenizerIncomplete, got {other:?}"),
    }

    assert!(result.page_occurrences().is_empty());
    assert!(result.page_margin_occurrences().is_empty());
}

/// Category 34: a generic parser resource stop (`RecoveryRecords`) reached
/// only *after* `PageRuleBlock` has already been entered and one page
/// declaration has already committed.
///
/// `@page{src:y;***;###;}`: `src:y;` is a valid page declaration (item
/// ordinal 0); `***;` is a malformed item whose shared
/// `InvalidBlockItem`/`MalformedBlockItem` evidence commits under a
/// `max_recovery_records` limit of 1, exhausting the resource; `###;` is a
/// second malformed item whose recovery/diagnostic commit is prospectively
/// refused before any mutation, so it never commits and the run stops there.
#[test]
fn generic_parser_resource_stop_after_page_context_entry() {
    let source = SourceText::new(SourceId::new(199_203), "@page{src:y;***;###;}".to_owned());
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits())
        .expect("production tokenizer must accept this source under generous limits");
    let limits = CssParserLimits::new(
        1 << 20,
        1 << 12,
        1 << 12,
        1 << 16,
        1 << 16,
        1, // max_recovery_records: exhausted by the first malformed item.
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
            assert_eq!(evidence.kind(), CssParserResourceKind::RecoveryRecords);
            assert_eq!(evidence.limit(), 1);
            assert_eq!(evidence.attempted(), 2);
            assert_eq!(evidence.location().range().start(), 16);
            assert_eq!(evidence.location().range().end(), 16);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }

    assert_eq!(result.context_records().len(), 1);
    let page = &result.context_records()[0];
    assert_eq!(page.kind(), CssParserContextKind::PageRuleBlock);
    match page.termination() {
        CssParserContextTermination::ParserResourceLimit { terminal } => {
            assert_eq!(terminal.range().start(), 16);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }

    assert_eq!(result.page_occurrences().len(), 1);
    let occurrence = &result.page_occurrences()[0];
    assert_eq!(occurrence.complete().range().start(), 6);
    assert_eq!(occurrence.complete().range().end(), 12);
    assert_eq!(occurrence.name().fragment(), "src");
    assert_eq!(occurrence.placement().context_id(), page.id());
    assert_eq!(occurrence.placement().item_ordinal().value(), 0);

    assert_eq!(result.recovery_records().len(), 1);
    assert_eq!(result.recovery_records()[0].region().range().start(), 12);
    assert_eq!(result.recovery_records()[0].region().range().end(), 16);
    assert_eq!(result.parser_diagnostics().len(), 1);

    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::RecoveryRecords),
        1
    );
}

/// Category 35: a generic parser resource stop (`RecoveryRecords`) reached
/// only *after* `PageMarginRuleBlock` has already been entered and one
/// page-margin declaration has already committed. Mirrors category 34's
/// shape one level deeper: `@page{@top-center{src:y;***;###;}}`.
#[test]
fn generic_parser_resource_stop_after_margin_context_entry() {
    let source = SourceText::new(
        SourceId::new(199_204),
        "@page{@top-center{src:y;***;###;}}".to_owned(),
    );
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits())
        .expect("production tokenizer must accept this source under generous limits");
    let limits = CssParserLimits::new(
        1 << 20,
        1 << 12,
        1 << 12,
        1 << 16,
        1 << 16,
        1, // max_recovery_records: exhausted by the first malformed item.
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
            assert_eq!(evidence.kind(), CssParserResourceKind::RecoveryRecords);
            assert_eq!(evidence.limit(), 1);
            assert_eq!(evidence.attempted(), 2);
            assert_eq!(evidence.location().range().start(), 28);
            assert_eq!(evidence.location().range().end(), 28);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }

    assert_eq!(result.context_records().len(), 2);
    let page = &result.context_records()[0];
    let margin = &result.context_records()[1];
    assert_eq!(page.kind(), CssParserContextKind::PageRuleBlock);
    assert_eq!(
        margin.kind(),
        CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::TopCenter)
    );
    match page.termination() {
        CssParserContextTermination::ParserResourceLimit { terminal } => {
            assert_eq!(terminal.range().start(), 28);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }
    match margin.termination() {
        CssParserContextTermination::ParserResourceLimit { terminal } => {
            assert_eq!(terminal.range().start(), 28);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }

    assert_eq!(result.page_margin_occurrences().len(), 1);
    let occurrence = &result.page_margin_occurrences()[0];
    assert_eq!(occurrence.name().fragment(), "src");
    assert_eq!(occurrence.placement().context_id(), margin.id());
    assert_eq!(occurrence.placement().item_ordinal().value(), 0);
    assert!(result.page_occurrences().is_empty());

    assert_eq!(result.recovery_records().len(), 1);
    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::RecoveryRecords),
        1
    );
}
