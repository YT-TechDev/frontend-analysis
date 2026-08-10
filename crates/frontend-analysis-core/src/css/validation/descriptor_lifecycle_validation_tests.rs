//! Focused #169 validation-gap coverage: category 16 (synthetic upstream-
//! tokenizer-incomplete while a `DescriptorRuleBlock` is already active) and
//! category 17 (a generic parser-resource stop after a `DescriptorRuleBlock`
//! has already been entered), kept as dedicated tests rather than folded
//! into `descriptor_fixtures.rs`'s generic gold-comparison inventory:
//! category 16 requires a genuine tokenizer-produced lexical prefix, and
//! category 17 requires a production-executed proof rather than the
//! contract-only gold fixture (`CSS-DESCRIPTOR-GENERIC-PARSER-RESOURCE-
//! LIMITED-TERMINATION-001` in `descriptor_fixtures.rs`) that previously
//! stood in for it -- mirroring how #168 covered its own analogous
//! lifecycle gaps in `group_lifecycle_validation_tests.rs` outside its
//! 28-fixture inventory.
//!
//! Production code must never import this module.

use crate::css::parser::context::{
    CssParserContextKind, CssParserContextTermination, CssParserDescriptorRuleKind,
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
/// single-byte `Ident` runs, which would destroy the descriptor `AtKeyword`
/// semantics a `DescriptorRuleBlock` context requires to be entered at all
/// (mirrors `group_lifecycle_validation_tests.rs`'s identically named
/// helper).
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

/// Category 16: a synthetic upstream-incomplete run over
/// `DescriptorRuleBlock(FontFace)` -- structurally equivalent to
/// `@font-face{font-family:x`, retained short of the tokenizer's genuine
/// source end -- must already retain the already-entered descriptor context,
/// honestly terminated `UpstreamTokenizerIncomplete` at the exact bounded
/// terminal, with no fabricated `}`, and with the shared `at_keyword`
/// evidence surviving.
#[test]
fn upstream_incomplete_retains_already_entered_descriptor_context() {
    let source = SourceText::new(
        SourceId::new(199_101),
        "@font-face{font-family:x;src:y;}".to_owned(),
    );
    // Truncate right after the descriptor block opener `{` (offset 11): the
    // context is entered, but its body has not yet observed any content.
    // Offset 11 is well short of the full 32-byte source, proving the
    // terminal need not sit at the retained source's end.
    let terminal_offset = 11;
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
    let descriptor = &result.context_records()[0];

    assert_eq!(
        descriptor.kind(),
        CssParserContextKind::DescriptorRuleBlock(CssParserDescriptorRuleKind::FontFace)
    );
    assert!(descriptor.parent().is_none());
    assert!(descriptor.descriptor_property_name().is_none());

    let at_keyword = descriptor
        .at_keyword()
        .expect("retained @font-face descriptor context must keep its exact at-keyword evidence");
    assert_eq!(at_keyword.range().start(), 0);
    assert_eq!(at_keyword.range().end(), 10);
    assert_eq!(at_keyword.fragment(), "@font-face");

    match descriptor.termination() {
        CssParserContextTermination::UpstreamTokenizerIncomplete { terminal } => {
            assert_eq!(terminal.range().start(), terminal_offset);
            assert_eq!(terminal.range().end(), terminal_offset);
        }
        other => panic!("expected UpstreamTokenizerIncomplete, got {other:?}"),
    }

    assert!(result.descriptor_occurrences().is_empty());
    assert!(result.occurrences().is_empty());
}

/// Category 17 (validation-review comment `5237181235`): a generic parser
/// resource stop -- `RecoveryRecords`, distinct from the entry-gating
/// `PeakContextDepth`/`ContextRecords` cases (categories 18/19) and from the
/// already-covered `DeclarationOccurrences`/`UnsupportedRegions` cases
/// (categories 20/21) -- reached only *after* `DescriptorRuleBlock(FontFace)`
/// has already been entered and one descriptor occurrence has already
/// committed.
///
/// `@font-face{src:y;***;###;}`: `src:y;` is a valid descriptor occurrence
/// (item ordinal 0); `***;` is a malformed item whose shared
/// `InvalidBlockItem`/`MalformedBlockItem` evidence commits under a
/// `max_recovery_records` limit of 1, exhausting the resource; `###;` is a
/// second malformed item whose recovery/diagnostic commit is prospectively
/// refused (attempted 2 > limit 1) before any mutation, so it never commits
/// and the run stops there -- well short of the authored closing `}`.
#[test]
fn generic_parser_resource_stop_after_descriptor_context_entry() {
    let source = SourceText::new(
        SourceId::new(199_102),
        "@font-face{src:y;***;###;}".to_owned(),
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

    // -- run-level termination -----------------------------------------
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), CssParserResourceKind::RecoveryRecords);
            assert_eq!(evidence.limit(), 1);
            assert_eq!(evidence.attempted(), 2);
            assert_eq!(evidence.location().range().start(), 21);
            assert_eq!(evidence.location().range().end(), 21);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }
    assert_eq!(result.terminal().range().start(), 21);
    assert_eq!(result.terminal().range().end(), 21);

    // -- descriptor context retained, not fabricated ---------------------
    assert_eq!(result.context_records().len(), 1);
    let descriptor = &result.context_records()[0];
    assert_eq!(
        descriptor.kind(),
        CssParserContextKind::DescriptorRuleBlock(CssParserDescriptorRuleKind::FontFace)
    );
    assert!(descriptor.parent().is_none());
    let at_keyword = descriptor
        .at_keyword()
        .expect("retained @font-face descriptor context must keep its exact at-keyword evidence");
    assert_eq!(at_keyword.range().start(), 0);
    assert_eq!(at_keyword.range().end(), 10);
    assert_eq!(at_keyword.fragment(), "@font-face");

    // Same exact terminal as the run's own committed parser terminal, and
    // the same exact resource-limit location -- never a fabricated `}`
    // (the authored closer at byte 25..26 is never reached).
    match descriptor.termination() {
        CssParserContextTermination::ParserResourceLimit { terminal } => {
            assert_eq!(terminal.range().start(), 21);
            assert_eq!(terminal.range().end(), 21);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }
    assert_eq!(descriptor.body().range().start(), 11);
    assert_eq!(descriptor.body().range().end(), 21);

    // -- already-committed descriptor evidence survives -------------------
    assert_eq!(result.descriptor_occurrences().len(), 1);
    let occurrence = &result.descriptor_occurrences()[0];
    assert_eq!(occurrence.complete().range().start(), 11);
    assert_eq!(occurrence.complete().range().end(), 17);
    assert_eq!(occurrence.name().fragment(), "src");
    assert_eq!(occurrence.placement().context_id(), descriptor.id());
    assert_eq!(occurrence.placement().item_ordinal().value(), 0);

    // The first malformed item's shared recovery/diagnostic evidence
    // committed before the refusal and survives; the second (refused)
    // malformed item's evidence never leaks.
    assert_eq!(result.recovery_records().len(), 1);
    assert_eq!(result.recovery_records()[0].region().range().start(), 17);
    assert_eq!(result.recovery_records()[0].region().range().end(), 21);
    assert_eq!(result.parser_diagnostics().len(), 1);
    assert_eq!(
        result.parser_diagnostics()[0].location().range().start(),
        17
    );
    assert_eq!(result.parser_diagnostics()[0].location().range().end(), 21);

    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::RecoveryRecords),
        1
    );
    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::ParserDiagnostics),
        1
    );
}
