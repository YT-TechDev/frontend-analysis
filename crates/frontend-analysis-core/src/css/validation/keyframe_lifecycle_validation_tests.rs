//! Focused #171 lifecycle/resource validation that is intentionally separate
//! from the source-driven independent fixture inventory.

use crate::css::parser::context::{CssParserContextKind, CssParserContextTermination};
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

fn parser_limits(
    max_peak_context_depth: usize,
    max_declaration_occurrences: usize,
    max_unsupported_regions: usize,
    max_context_records: usize,
) -> CssParserLimits {
    CssParserLimits::new(
        1 << 20,
        1 << 12,
        max_peak_context_depth,
        max_declaration_occurrences,
        1 << 16,
        1 << 16,
        max_unsupported_regions,
        1 << 16,
        max_context_records,
    )
    .unwrap()
}

fn generous_parser_limits() -> CssParserLimits {
    parser_limits(1 << 12, 1 << 16, 1 << 16, 1 << 16)
}

fn real_prefix_as_upstream_incomplete(
    source: &SourceText,
    terminal_offset: usize,
) -> CssTokenizerRunResult {
    let complete = run_tokenizer(source, generous_tokenizer_limits()).unwrap();
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
    assert_eq!(cursor, terminal_offset);
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

#[test]
fn upstream_incomplete_retains_entered_keyframes_context() {
    let source = SourceText::new(
        SourceId::new(171_101),
        "@keyframes x{from{x:y;}}".to_owned(),
    );
    let terminal_offset = 13; // immediately after the outer `{`.
    let upstream = real_prefix_as_upstream_incomplete(&source, terminal_offset);
    let result = run(&source, upstream, generous_parser_limits()).unwrap();

    assert_eq!(
        result.termination(),
        &CssParserTermination::UpstreamTokenizerIncomplete
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(result.context_records().len(), 1);
    let outer = &result.context_records()[0];
    assert_eq!(outer.kind(), CssParserContextKind::KeyframesRuleBlock);
    assert!(outer.parent().is_none());
    match outer.termination() {
        CssParserContextTermination::UpstreamTokenizerIncomplete { terminal } => {
            assert_eq!(terminal.range().start(), terminal_offset);
            assert_eq!(terminal.range().end(), terminal_offset);
        }
        other => panic!("expected upstream-incomplete keyframes context, got {other:?}"),
    }
    assert!(result.keyframe_occurrences().is_empty());
}

#[test]
fn upstream_incomplete_retains_outer_and_keyframe_block() {
    let source = SourceText::new(
        SourceId::new(171_102),
        "@keyframes x{from{x:y;}}".to_owned(),
    );
    let terminal_offset = 18; // immediately after `from{`.
    let upstream = real_prefix_as_upstream_incomplete(&source, terminal_offset);
    let result = run(&source, upstream, generous_parser_limits()).unwrap();

    assert_eq!(result.context_records().len(), 2);
    let outer = &result.context_records()[0];
    let block = &result.context_records()[1];
    assert_eq!(outer.kind(), CssParserContextKind::KeyframesRuleBlock);
    assert!(outer.parent().is_none());
    assert_eq!(block.kind(), CssParserContextKind::KeyframeBlock);
    assert_eq!(block.parent(), Some(outer.id()));
    for context in [outer, block] {
        match context.termination() {
            CssParserContextTermination::UpstreamTokenizerIncomplete { terminal } => {
                assert_eq!(terminal.range().start(), terminal_offset);
                assert_eq!(terminal.range().end(), terminal_offset);
            }
            other => panic!("expected upstream-incomplete context, got {other:?}"),
        }
    }
}

#[test]
fn declaration_occurrence_cap_is_shared_with_keyframe_occurrences() {
    let source = SourceText::new(
        SourceId::new(171_103),
        "a{x:y;}@keyframes k{from{x:z;}}".to_owned(),
    );
    let tokenizer = run_tokenizer(&source, generous_tokenizer_limits()).unwrap();
    let result = run(
        &source,
        tokenizer,
        parser_limits(1 << 12, 1, 1 << 16, 1 << 16),
    )
    .unwrap();

    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(
                evidence.kind(),
                CssParserResourceKind::DeclarationOccurrences
            );
            assert_eq!(evidence.limit(), 1);
            assert_eq!(evidence.attempted(), 2);
        }
        other => panic!("expected DeclarationOccurrences stop, got {other:?}"),
    }
    assert_eq!(result.occurrences().len(), 1);
    assert!(result.keyframe_occurrences().is_empty());
    assert!(
        result
            .context_records()
            .iter()
            .any(|context| matches!(context.kind(), CssParserContextKind::KeyframesRuleBlock))
    );
    assert!(
        result
            .context_records()
            .iter()
            .any(|context| matches!(context.kind(), CssParserContextKind::KeyframeBlock))
    );
}

#[test]
fn context_record_refusal_after_outer_entry_keeps_outer_partial() {
    let source = SourceText::new(
        SourceId::new(171_104),
        "@keyframes spin{from{opacity:0;}}".to_owned(),
    );
    let tokenizer = run_tokenizer(&source, generous_tokenizer_limits()).unwrap();
    let result = run(
        &source,
        tokenizer,
        parser_limits(1 << 12, 1 << 16, 1 << 16, 1),
    )
    .unwrap();

    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), CssParserResourceKind::ContextRecords);
            assert_eq!(evidence.limit(), 1);
            assert_eq!(evidence.attempted(), 2);
        }
        other => panic!("expected ContextRecords stop, got {other:?}"),
    }
    assert_eq!(result.context_records().len(), 1);
    let outer = &result.context_records()[0];
    assert_eq!(outer.kind(), CssParserContextKind::KeyframesRuleBlock);
    assert!(outer.parent().is_none());
    assert!(matches!(
        outer.termination(),
        CssParserContextTermination::ParserResourceLimit { .. }
    ));
    assert!(result.keyframe_occurrences().is_empty());
}

#[test]
fn unsupported_region_refusal_for_unqualified_child_is_commit_honest() {
    let source = SourceText::new(
        SourceId::new(171_105),
        "@keyframes x{bogus{x:y;}from{a:b;}}".to_owned(),
    );
    let tokenizer = run_tokenizer(&source, generous_tokenizer_limits()).unwrap();
    let result = run(
        &source,
        tokenizer,
        parser_limits(1 << 12, 1 << 16, 0, 1 << 16),
    )
    .unwrap();

    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), CssParserResourceKind::UnsupportedRegions);
            assert_eq!(evidence.limit(), 0);
            assert_eq!(evidence.attempted(), 1);
        }
        other => panic!("expected UnsupportedRegions stop, got {other:?}"),
    }
    assert_eq!(result.context_records().len(), 1);
    assert_eq!(
        result.context_records()[0].kind(),
        CssParserContextKind::KeyframesRuleBlock
    );
    assert!(result.unsupported_regions().is_empty());
    assert!(result.keyframe_occurrences().is_empty());
}
