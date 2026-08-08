//! Executable resource-limit tests for #136.
//!
//! The six #135 `resource_limit_contract_fixtures()` are CONTRACT-ONLY (they
//! validate the gold model's own resource-lifecycle shape) and must not be
//! executed as tokenizer recognition oracles. These tests instead drive the
//! production tokenizer directly with real triggering source inputs and
//! deliberately tight `CssTokenizerLimits` for each of the six resource
//! kinds, and assert both the `ResourceLimit` evidence and the committed
//! `CssTokenizerResourceUsage` left behind by the failed prospective
//! operation, per the #152 `checked_resource_add` contract.

use crate::css::tokenizer::producer::run;
use crate::css::tokenizer::resource::{CssTokenizerLimits, CssTokenizerResourceKind};
use crate::css::tokenizer::result::{CssTokenizerCompletion, CssTokenizerTermination};
use crate::{SourceId, SourceText};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), text.to_owned())
}

#[allow(clippy::too_many_arguments)]
fn assert_resource_limited(
    text: &SourceText,
    limits: CssTokenizerLimits,
    expected_kind: CssTokenizerResourceKind,
    expected_limit: usize,
    expected_attempted: usize,
    expected_terminal: usize,
    expected_committed_items: usize,
    expected_committed_usage: usize,
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
    assert_eq!(result.terminal().range().start(), expected_terminal);
    assert_eq!(result.terminal().range().end(), expected_terminal);
    assert_eq!(result.lexical_items().len(), expected_committed_items);
    assert_eq!(
        result.resources().value(expected_kind),
        expected_committed_usage,
        "committed usage for {expected_kind:?} must reflect only successfully committed work"
    );
}

#[test]
fn source_bytes_limit_terminates_before_any_processing() {
    let text = source("a");
    let limits = CssTokenizerLimits::new(0, 1000, 1000, 1000, 1000, 1000).unwrap();
    // SourceBytes is the entry-established retained byte length, not an
    // incremental counter: usage remains the real source length even though
    // the run terminates before any token processing.
    assert_resource_limited(
        &text,
        limits,
        CssTokenizerResourceKind::SourceBytes,
        0,
        1,
        0,
        0,
        1,
    );
}

#[test]
fn algorithm_steps_limit_terminates_mid_recognition_and_preserves_committed_usage() {
    let text = source("a");
    let limits = CssTokenizerLimits::new(1000, 1, 1000, 1000, 1000, 1000).unwrap();
    // The first step (outer dispatch) commits successfully; the second step
    // (inside ident-sequence consumption) is the failed prospective attempt.
    assert_resource_limited(
        &text,
        limits,
        CssTokenizerResourceKind::AlgorithmSteps,
        1,
        2,
        0,
        0,
        1,
    );
}

#[test]
fn lexical_items_limit_preflights_before_recognition_and_never_commits() {
    let text = source("a");
    let limits = CssTokenizerLimits::new(1000, 1000, 0, 1000, 1000, 1000).unwrap();
    // The failed prospective item preflight must not increment committed
    // item usage: zero items are ever pushed.
    assert_resource_limited(
        &text,
        limits,
        CssTokenizerResourceKind::LexicalItems,
        0,
        1,
        0,
        0,
        0,
    );
}

#[test]
fn diagnostics_limit_terminates_before_atomic_item_commit_and_stays_at_zero() {
    let text = source("\"a");
    let limits = CssTokenizerLimits::new(1000, 1000, 1000, 0, 1000, 1000).unwrap();
    // The pending EofInString diagnostic is provisional until its owning
    // String item commits atomically; committed Diagnostics usage remains 0.
    assert_resource_limited(
        &text,
        limits,
        CssTokenizerResourceKind::Diagnostics,
        0,
        1,
        0,
        0,
        0,
    );
}

#[test]
fn retained_interpreted_bytes_limit_preserves_previously_committed_usage() {
    let text = source("a b");
    let limits = CssTokenizerLimits::new(1000, 1000, 1000, 1000, 1, 1000).unwrap();
    // "a" commits first (retained usage 0 -> 1). The whitespace token
    // retains no interpreted bytes (usage stays 1). "b" would raise
    // prospective usage to 2, exceeding the limit of 1; the failed
    // prospective attempt must not disturb the 1 byte already committed.
    assert_resource_limited(
        &text,
        limits,
        CssTokenizerResourceKind::RetainedInterpretedBytes,
        1,
        2,
        2,
        2,
        1,
    );
}

#[test]
fn temporary_buffer_bytes_limit_preserves_the_highest_successfully_reached_peak() {
    let text = source("ab");
    let limits = CssTokenizerLimits::new(1000, 1000, 1000, 1000, 1000, 1).unwrap();
    // Growing scratch by the first byte ('a') succeeds and immediately
    // raises the observed peak to 1. Growing by the second byte ('b') is
    // the failed prospective attempt (attempted = 2); the observed peak
    // must remain 1, the highest value actually reached, not 0.
    assert_resource_limited(
        &text,
        limits,
        CssTokenizerResourceKind::TemporaryBufferBytes,
        1,
        2,
        0,
        0,
        1,
    );
}
