//! Executable, source-driven resource-limit tests for the #139/#158 parser
//! resource kinds, mirroring the #152 `checked_resource_add` committed-
//! versus-attempted contract already proven for the tokenizer. The #166
//! `PeakContextDepth`/`ContextRecords` dimensions are exercised only through
//! synthetic contract-valid construction (`parser/resource.rs`,
//! `parser/result.rs`) because this #166 producer never allocates either.

use crate::css::parser::producer::run;
use crate::css::parser::resource::{CssParserLimits, CssParserResourceKind};
use crate::css::parser::result::{CssParserExecutionCompletion, CssParserTermination};
use crate::css::tokenizer::producer::run as run_tokenizer;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::{SourceId, SourceText};

fn generous_tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
}

#[allow(clippy::too_many_arguments)]
fn parser_limits(
    max_algorithm_steps: usize,
    max_peak_component_depth: usize,
    max_declaration_occurrences: usize,
    max_parser_diagnostics: usize,
    max_recovery_records: usize,
    max_unsupported_regions: usize,
    max_discard_records: usize,
) -> CssParserLimits {
    CssParserLimits::new(
        max_algorithm_steps,
        max_peak_component_depth,
        // #166: generous, since this producer never allocates a context.
        1000,
        max_declaration_occurrences,
        max_parser_diagnostics,
        max_recovery_records,
        max_unsupported_regions,
        max_discard_records,
        1000,
    )
    .unwrap()
}

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), text.to_owned())
}

#[test]
fn algorithm_steps_limit_terminates_and_preserves_last_committed_boundary() {
    let text = source("a{color:red;}");
    let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
    let limits = parser_limits(1, 1000, 1000, 1000, 1000, 1000, 1000);
    let result = run(&text, tokenizer_result, limits)
        .expect("resource-limited runs remain a normal Ok result");

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), CssParserResourceKind::AlgorithmSteps);
            assert_eq!(evidence.limit(), 1);
            assert_eq!(evidence.attempted(), 2);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }
    assert_eq!(result.terminal().range().start(), 0);
    assert_eq!(result.terminal().range().end(), 0);
    assert!(result.occurrences().is_empty());
    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::AlgorithmSteps),
        1
    );
}

#[test]
fn peak_component_depth_limit_preserves_the_highest_successfully_entered_depth() {
    let text = source("a{color:f(g(1));}");
    let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
    let limits = parser_limits(10_000, 1, 1000, 1000, 1000, 1000, 1000);
    let result = run(&text, tokenizer_result, limits)
        .expect("resource-limited runs remain a normal Ok result");

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), CssParserResourceKind::PeakComponentDepth);
            assert_eq!(evidence.limit(), 1);
            assert_eq!(evidence.attempted(), 2);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }
    assert!(
        result.occurrences().is_empty(),
        "no half-built declaration may be published"
    );
    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::PeakComponentDepth),
        1
    );
}

#[test]
fn declaration_occurrences_limit_commits_the_first_and_refuses_the_second() {
    let text = source("a{color:red;background:blue;}");
    let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
    let limits = parser_limits(10_000, 1000, 1, 1000, 1000, 1000, 1000);
    let result = run(&text, tokenizer_result, limits)
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
    assert_eq!(result.occurrences().len(), 1);
    assert_eq!(result.occurrences()[0].complete().range().start(), 2);
    assert_eq!(result.occurrences()[0].complete().range().end(), 12);
    assert_eq!(result.terminal().range().start(), 12);
    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::DeclarationOccurrences),
        1
    );
}

#[test]
fn parser_diagnostics_limit_refusal_leaves_diagnostics_and_recovery_both_uncommitted() {
    let text = source("a{color red;}");
    let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
    let limits = parser_limits(10_000, 1000, 1000, 0, 1000, 1000, 1000);
    let result = run(&text, tokenizer_result, limits)
        .expect("resource-limited runs remain a normal Ok result");

    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), CssParserResourceKind::ParserDiagnostics);
            assert_eq!(evidence.limit(), 0);
            assert_eq!(evidence.attempted(), 1);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }
    assert!(result.parser_diagnostics().is_empty());
    assert!(result.recovery_records().is_empty());
    assert_eq!(result.terminal().range().start(), 2);
}

#[test]
fn recovery_records_limit_refusal_leaves_diagnostics_and_recovery_both_uncommitted() {
    let text = source("a{color red;}");
    let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
    let limits = parser_limits(10_000, 1000, 1000, 1000, 0, 1000, 1000);
    let result = run(&text, tokenizer_result, limits)
        .expect("resource-limited runs remain a normal Ok result");

    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), CssParserResourceKind::RecoveryRecords);
            assert_eq!(evidence.limit(), 0);
            assert_eq!(evidence.attempted(), 1);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }
    assert!(
        result.parser_diagnostics().is_empty(),
        "diagnostic must not commit when its atomic recovery counterpart is refused"
    );
    assert!(result.recovery_records().is_empty());
}

#[test]
fn unsupported_regions_limit_refusal_publishes_no_partial_record() {
    let text = source("@font-face{font-family:x;}");
    let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
    let limits = parser_limits(10_000, 1000, 1000, 1000, 1000, 0, 1000);
    let result = run(&text, tokenizer_result, limits)
        .expect("resource-limited runs remain a normal Ok result");

    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), CssParserResourceKind::UnsupportedRegions);
            assert_eq!(evidence.limit(), 0);
            assert_eq!(evidence.attempted(), 1);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }
    assert!(result.unsupported_regions().is_empty());
    assert_eq!(result.terminal().range().start(), 0);
    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::UnsupportedRegions),
        0
    );
}

#[test]
fn discard_records_limit_refusal_is_commit_honest() {
    let text = source("--foo:bar{color:red;}");
    let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
    let limits = parser_limits(10_000, 1000, 1000, 1000, 1000, 1000, 0);
    let result = run(&text, tokenizer_result, limits)
        .expect("resource-limited runs remain a normal Ok result");

    match result.termination() {
        CssParserTermination::ParserResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), CssParserResourceKind::DiscardRecords);
            assert_eq!(evidence.limit(), 0);
            assert_eq!(evidence.attempted(), 1);
        }
        other => panic!("expected ParserResourceLimit, got {other:?}"),
    }
    assert!(result.discard_records().is_empty());
    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::DiscardRecords),
        0
    );
    assert!(result.occurrences().is_empty());
    assert!(result.parser_diagnostics().is_empty());
    assert!(result.recovery_records().is_empty());
    assert!(result.unsupported_regions().is_empty());
    // The last committed boundary is the structurally accepted qualified-rule
    // opening (`{`.end), the same commit granularity a normal declaration
    // block establishes before any block-content resource check.
    assert_eq!(result.terminal().range().start(), 10);
    assert_eq!(result.terminal().range().end(), 10);
}

#[test]
fn earlier_committed_declarations_survive_a_later_resource_refusal() {
    let text = source("a{color:red;background:blue;color:green;}");
    let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
    let limits = parser_limits(10_000, 1000, 2, 1000, 1000, 1000, 1000);
    let result = run(&text, tokenizer_result, limits)
        .expect("resource-limited runs remain a normal Ok result");

    assert_eq!(result.occurrences().len(), 2);
    assert_eq!(
        result
            .resources()
            .value(CssParserResourceKind::DeclarationOccurrences),
        2
    );
}
