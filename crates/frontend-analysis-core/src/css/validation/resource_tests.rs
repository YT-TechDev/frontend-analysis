//! Executable resource-limit tests for #136.
//!
//! The six #135 `resource_limit_contract_fixtures()` are CONTRACT-ONLY (they
//! validate the gold model's own resource-lifecycle shape) and must not be
//! executed as tokenizer recognition oracles. These tests instead drive the
//! production tokenizer directly with real triggering source inputs and
//! deliberately tight `CssTokenizerLimits` for each of the six resource
//! kinds.

use crate::css::tokenizer::producer::run;
use crate::css::tokenizer::resource::{CssTokenizerLimits, CssTokenizerResourceKind};
use crate::css::tokenizer::result::{CssTokenizerCompletion, CssTokenizerTermination};
use crate::{SourceId, SourceText};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), text.to_owned())
}

fn assert_resource_limited(
    text: &SourceText,
    limits: CssTokenizerLimits,
    expected_kind: CssTokenizerResourceKind,
    expected_limit: usize,
    expected_attempted: usize,
) {
    let result = run(text, limits).expect("resource-limited runs remain a normal Ok result");
    assert_eq!(result.completion(), CssTokenizerCompletion::Incomplete);
    match result.termination() {
        CssTokenizerTermination::ResourceLimit(evidence) => {
            assert_eq!(evidence.kind(), expected_kind);
            assert_eq!(evidence.limit(), expected_limit);
            assert_eq!(evidence.attempted(), expected_attempted);
        }
        CssTokenizerTermination::EndOfInput => panic!("expected a resource-limited termination"),
    }
    assert_eq!(result.terminal().range().start(), 0);
    assert_eq!(result.terminal().range().end(), 0);
    assert!(result.lexical_items().is_empty());
}

#[test]
fn source_bytes_limit_terminates_before_any_processing() {
    let text = source("a");
    let limits = CssTokenizerLimits::new(0, 1000, 1000, 1000, 1000, 1000).unwrap();
    assert_resource_limited(&text, limits, CssTokenizerResourceKind::SourceBytes, 0, 1);
}

#[test]
fn algorithm_steps_limit_terminates_mid_recognition() {
    let text = source("a");
    let limits = CssTokenizerLimits::new(1000, 1, 1000, 1000, 1000, 1000).unwrap();
    assert_resource_limited(
        &text,
        limits,
        CssTokenizerResourceKind::AlgorithmSteps,
        1,
        2,
    );
}

#[test]
fn lexical_items_limit_preflights_before_recognition() {
    let text = source("a");
    let limits = CssTokenizerLimits::new(1000, 1000, 0, 1000, 1000, 1000).unwrap();
    assert_resource_limited(&text, limits, CssTokenizerResourceKind::LexicalItems, 0, 1);
}

#[test]
fn diagnostics_limit_terminates_before_atomic_item_commit() {
    let text = source("\"a");
    let limits = CssTokenizerLimits::new(1000, 1000, 1000, 0, 1000, 1000).unwrap();
    assert_resource_limited(&text, limits, CssTokenizerResourceKind::Diagnostics, 0, 1);
}

#[test]
fn retained_interpreted_bytes_limit_terminates_before_atomic_item_commit() {
    let text = source("a");
    let limits = CssTokenizerLimits::new(1000, 1000, 1000, 1000, 0, 1000).unwrap();
    assert_resource_limited(
        &text,
        limits,
        CssTokenizerResourceKind::RetainedInterpretedBytes,
        0,
        1,
    );
}

#[test]
fn temporary_buffer_bytes_limit_terminates_before_scratch_growth() {
    let text = source("a");
    let limits = CssTokenizerLimits::new(1000, 1000, 1000, 1000, 1000, 0).unwrap();
    assert_resource_limited(
        &text,
        limits,
        CssTokenizerResourceKind::TemporaryBufferBytes,
        0,
        1,
    );
}
