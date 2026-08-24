//! Cross-layer validation of the TC-S1 document shell construction capability
//! against the existing #112/#113 tokenizer gold corpus.
//!
//! This gate sits beside the existing parser and Core-analysis gates and adds
//! no fixture, generator, or gold of its own: it reuses
//! [`super::corpus::all_candidate_independent_corpus`] (72 initial plus 4
//! supplemental `REG-` fixtures) and the existing bounded, deterministic,
//! dependency-free 4,096-case generator without editing or copying either.
//!
//! What it does **not** do is as important as what it does. It does not
//! reinterpret every tokenizer fixture as supported TC-S1 input, and it never
//! derives an expected tree shape from production output. TC-S1 is a bounded
//! slice: most of this corpus lies outside it, and the honest expected
//! outcome for those cases is explicit tree-unsupported or retained
//! lower-layer incompleteness.
//!
//! The properties checked here are the ones that hold for *every* input,
//! supported or not:
//!
//! - panic freedom (no `catch_unwind`, so a production panic fails naturally);
//! - a deterministic semantic result across repeats and source identities;
//! - no false effective `Complete`, cross-checked against the fixture's own
//!   independently authored tokenizer completion;
//! - valid frozen checkpoint relationships;
//! - exact retained source binding, cross-checked against the fixture gold's
//!   own authored spans rather than against production; and
//! - honest unsupported and resource propagation, including that committed
//!   tree coverage never runs past an unsupported trigger and never claims
//!   more than the tokenizer produced.

use crate::html::tree_construction::driver::construct_html_document_shell;
use crate::html::tree_construction::result::{
    HtmlAuthoredSource, HtmlDocumentShellAnalysis, HtmlShellElementName, HtmlTreeCompletion,
    HtmlTreeIncompleteCause, HtmlTreeNode, HtmlTreeNodeKind,
};
use crate::{SourceId, SourceText};

use super::super::producer::tokenize;
use super::super::resource::HtmlTokenizerLimits;
use super::corpus::all_candidate_independent_corpus;
use super::expected::{Limits, Token, TokenKind};
use super::fixture::HtmlTokenizerFixture;
use super::generated::{MAX_GENERATED_CASES, MAX_SOURCE_BYTES, generated_inputs};
use super::observe::observe;

fn to_html_limits(limits: Limits) -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(
        limits.source_bytes,
        limits.transition_steps,
        limits.emitted_tokens,
        limits.diagnostics,
        limits.attributes_per_tag,
        limits.retained_interpreted_bytes,
        limits.temporary_buffer_bytes,
    )
}

/// A structural result signature: constructed tree meaning, provenance spans,
/// action and diagnostic meaning, coverage, and completion — never authored
/// source content and never the caller-supplied [`SourceId`], so two runs over
/// equal bytes under different identities can be compared directly.
fn signature(analysis: &HtmlDocumentShellAnalysis) -> String {
    let mut rendered = String::new();
    render_node(analysis, analysis.root(), &mut rendered);
    rendered.push('|');
    for action in analysis.actions() {
        rendered.push_str(&format!(
            "{:?}@{}{:?};",
            action.kind(),
            action.trigger().token_index(),
            action.trigger().authored_boundary().map(|a| a.range())
        ));
    }
    rendered.push('|');
    for diagnostic in analysis.diagnostics() {
        rendered.push_str(&format!(
            "{:?}@{}{:?};",
            diagnostic.code(),
            diagnostic.trigger().token_index(),
            diagnostic.recovery()
        ));
    }
    rendered.push('|');
    rendered.push_str(&format!(
        "{}:{}|",
        analysis.coverage().committed_end(),
        analysis.coverage().processed_tokens()
    ));
    rendered.push_str(&match analysis.completion() {
        HtmlTreeCompletion::Complete => "complete".to_owned(),
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete) => {
            "lower-layer".to_owned()
        }
        HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
            unsupported,
        )) => format!(
            "unsupported {:?}@{}{:?}",
            unsupported.capability(),
            unsupported.trigger().token_index(),
            unsupported.trigger().authored_boundary().map(|a| a.range())
        ),
    });
    rendered
}

fn render_node(
    analysis: &HtmlDocumentShellAnalysis,
    id: crate::html::tree_construction::result::HtmlConstructedNodeId,
    rendered: &mut String,
) {
    let Some(node) = analysis.node(id) else {
        rendered.push_str("<unresolved>");
        return;
    };
    match node.kind() {
        HtmlTreeNodeKind::Document => rendered.push_str("#document"),
        HtmlTreeNodeKind::Element(element) => {
            rendered.push_str(match element.name() {
                HtmlShellElementName::Html => "html",
                HtmlShellElementName::Head => "head",
                HtmlShellElementName::Body => "body",
            });
            match node.authored_source() {
                Some(HtmlAuthoredSource::StartTag { complete, raw_name }) => {
                    rendered.push_str(&format!("[{:?},{:?}]", complete.range(), raw_name.range()))
                }
                _ => rendered.push_str("[synthesized]"),
            }
        }
        HtmlTreeNodeKind::Text(text) => {
            rendered.push_str(&format!("#text({})[", text.interpreted().len()));
            for contribution in text.contributions() {
                rendered.push_str(&format!("{:?},", contribution.source().range()));
            }
            rendered.push(']');
        }
    }
    rendered.push('(');
    for child in node.children() {
        render_node(analysis, *child, rendered);
        rendered.push(' ');
    }
    rendered.push(')');
}

/// Checks the frozen-result relationships that must hold for every input.
fn check_frozen_relationships(
    label: &str,
    source: &SourceText,
    analysis: &HtmlDocumentShellAnalysis,
    failures: &mut Vec<String>,
) {
    let root = analysis
        .node(analysis.root())
        .expect("the frozen root resolves");
    if !matches!(root.kind(), HtmlTreeNodeKind::Document) || root.parent().is_some() {
        failures.push(format!(
            "{label}: the frozen root is not a parentless document"
        ));
    }
    if root.authored_source().is_some() {
        failures.push(format!(
            "{label}: the document root carries authored source"
        ));
    }

    let ordered = analysis.nodes_in_creation_order();
    if ordered.len() != analysis.node_count() {
        failures.push(format!("{label}: creation ordering lost a node"));
    }
    for pair in ordered.windows(2) {
        if pair[0].id() >= pair[1].id() {
            failures.push(format!(
                "{label}: constructed identities are not unique and strictly creation-ordered"
            ));
        }
    }
    for node in &ordered {
        if let Some(parent_id) = node.parent() {
            if parent_id >= node.id() {
                failures.push(format!("{label}: a child committed before its parent"));
            }
            match analysis.node(parent_id) {
                None => failures.push(format!("{label}: a parent relationship does not resolve")),
                Some(parent) => {
                    if parent
                        .children()
                        .iter()
                        .filter(|id| **id == node.id())
                        .count()
                        != 1
                    {
                        failures.push(format!("{label}: a relationship is not mutual"));
                    }
                }
            }
        } else if node.id() != analysis.root() {
            failures.push(format!("{label}: a non-root node has no parent"));
        }
        for child_id in node.children() {
            match analysis.node(*child_id) {
                None => failures.push(format!("{label}: a child relationship does not resolve")),
                Some(child) => {
                    if child.parent() != Some(node.id()) {
                        failures.push(format!("{label}: a relationship is not mutual"));
                    }
                }
            }
        }
        check_source_binding(label, source, node, failures);
    }
}

fn check_source_binding(
    label: &str,
    source: &SourceText,
    node: &HtmlTreeNode,
    failures: &mut Vec<String>,
) {
    match node.authored_source() {
        None => {}
        Some(HtmlAuthoredSource::StartTag { complete, raw_name }) => {
            for anchor in [complete, raw_name] {
                if anchor.source_id() != source.id() {
                    failures.push(format!(
                        "{label}: authored evidence has a foreign source id"
                    ));
                    continue;
                }
                let range = anchor.range();
                match source.as_str().get(range.start()..range.end()) {
                    Some(fragment) if fragment == anchor.fragment() => {}
                    _ => failures.push(format!("{label}: authored evidence does not revalidate")),
                }
            }
            if complete.range().start() > raw_name.range().start()
                || raw_name.range().end() > complete.range().end()
            {
                failures.push(format!(
                    "{label}: raw-name evidence escapes its complete tag"
                ));
            }
        }
        Some(HtmlAuthoredSource::Characters(contributions)) => {
            if contributions.is_empty() {
                failures.push(format!("{label}: a text node retains no contribution"));
            }
            let mut rebuilt = String::new();
            let mut previous_end = 0usize;
            for contribution in contributions {
                let anchor = contribution.source();
                if anchor.source_id() != source.id() {
                    failures.push(format!("{label}: a contribution has a foreign source id"));
                }
                if anchor.range().start() < previous_end {
                    failures.push(format!("{label}: contributions are not in source order"));
                }
                previous_end = anchor.range().end();
                rebuilt.push_str(contribution.interpreted());
            }
            let HtmlTreeNodeKind::Text(text) = node.kind() else {
                failures.push(format!("{label}: character provenance on a non-text node"));
                return;
            };
            if rebuilt != text.interpreted() {
                failures.push(format!(
                    "{label}: interpreted text is not its ordered contributions"
                ));
            }
        }
    }
}

/// Checks that effective completion is never claimed dishonestly, and that
/// committed tree coverage stays behind its own stop.
fn check_completion_honesty(
    label: &str,
    analysis: &HtmlDocumentShellAnalysis,
    failures: &mut Vec<String>,
) {
    let emitted = analysis.tokenizer_run().tokens().len();
    if analysis.coverage().processed_tokens() > emitted {
        failures.push(format!("{label}: committed tokens exceed emitted tokens"));
    }
    if analysis.is_complete() {
        if analysis.tokenizer_run().is_incomplete() {
            failures.push(format!(
                "{label}: effective Complete over an incomplete tokenizer run"
            ));
        }
        if analysis.coverage().processed_tokens() != emitted {
            failures.push(format!(
                "{label}: effective Complete with unprocessed emitted tokens"
            ));
        }
        if !has_complete_shell(analysis) {
            failures.push(format!(
                "{label}: effective Complete without a complete document shell"
            ));
        }
    }
    if let HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::UnsupportedCapability(
        unsupported,
    )) = analysis.completion()
    {
        if unsupported.trigger().token_index() >= emitted {
            failures.push(format!(
                "{label}: an unsupported trigger names no emitted token"
            ));
        }
        if let Some(boundary) = unsupported.trigger().authored_boundary() {
            if analysis.coverage().committed_end() > boundary.range().start() {
                failures.push(format!(
                    "{label}: committed tree coverage ran past the unsupported trigger"
                ));
            }
            for node in analysis.nodes_in_creation_order() {
                if let Some(HtmlAuthoredSource::StartTag { complete, .. }) = node.authored_source()
                    && complete.range() == boundary.range()
                {
                    failures.push(format!(
                        "{label}: the unsupported trigger leaked as a node's authored origin"
                    ));
                }
            }
        }
    }
}

fn has_complete_shell(analysis: &HtmlDocumentShellAnalysis) -> bool {
    let names: Vec<HtmlShellElementName> = analysis
        .nodes_in_creation_order()
        .iter()
        .filter_map(|node| match node.kind() {
            HtmlTreeNodeKind::Element(element) => Some(element.name()),
            _ => None,
        })
        .collect();
    names.contains(&HtmlShellElementName::Html)
        && names.contains(&HtmlShellElementName::Head)
        && names.contains(&HtmlShellElementName::Body)
}

/// The authored spans a fixture's own gold declares. Independent expected
/// evidence: read from the #112 gold, never from a production run.
struct GoldSpans {
    start_tags: Vec<(usize, usize)>,
    characters: Vec<(usize, usize)>,
}

fn gold_spans(fixture: &HtmlTokenizerFixture) -> GoldSpans {
    let mut spans = GoldSpans {
        start_tags: Vec::new(),
        characters: Vec::new(),
    };
    for token in &fixture.expected.0.tokens {
        match token {
            Token::Tag {
                kind: TokenKind::StartTag,
                complete,
                ..
            } => spans
                .start_tags
                .push((complete.span.start, complete.span.end)),
            Token::Character { source, .. } => {
                spans.characters.push((source.span.start, source.span.end));
            }
            _ => {}
        }
    }
    spans
}

#[test]
fn tree_construction_holds_its_contract_over_the_candidate_independent_corpus() {
    let fixtures = all_candidate_independent_corpus();
    const INITIAL_CORPUS_COUNT: usize = 72;
    const SUPPLEMENTAL_REGRESSION_COUNT: usize = 4;
    assert_eq!(
        fixtures.len(),
        INITIAL_CORPUS_COUNT + SUPPLEMENTAL_REGRESSION_COUNT
    );

    let mut failures = Vec::new();
    let mut supported = 0usize;
    for fixture in &fixtures {
        let text = String::from_utf8(fixture.source_bytes.to_vec())
            .expect("fixture source is valid UTF-8");
        let limits = to_html_limits(fixture.expected.0.limits);
        let source = SourceText::new(SourceId::new(1), text.clone());

        // Retained tokenizer evidence must be exactly what the tokenizer
        // produced, before and after the tree-construction boundary.
        let standalone_run = tokenize(&source, limits);
        let observed_before = observe(&source, &standalone_run);

        let analysis = match construct_html_document_shell(&source, limits) {
            Ok(analysis) => analysis,
            Err(error) => {
                failures.push(format!("{}: boundary failure {error:?}", fixture.id));
                continue;
            }
        };
        if observe(&source, analysis.tokenizer_run()) != observed_before {
            failures.push(format!(
                "{}: retained tokenizer evidence changed through the tree boundary",
                fixture.id
            ));
        }

        check_frozen_relationships(fixture.id, &source, &analysis, &mut failures);
        check_completion_honesty(fixture.id, &analysis, &mut failures);

        // Cross-check effective completion against the fixture's own
        // independently authored tokenizer completion.
        if analysis.is_complete() {
            supported += 1;
            if !fixture.expected.0.completion.is_complete() {
                failures.push(format!(
                    "{}: effective Complete although the gold tokenizer run is not Complete",
                    fixture.id
                ));
            }
        }

        // Exact retained source binding, against the gold's own spans.
        let gold = gold_spans(fixture);
        for node in analysis.nodes_in_creation_order() {
            match node.authored_source() {
                Some(HtmlAuthoredSource::StartTag { complete, .. }) => {
                    let span = (complete.range().start(), complete.range().end());
                    if !gold.start_tags.contains(&span) {
                        failures.push(format!(
                            "{}: authored element span {span:?} is not an authored gold start tag",
                            fixture.id
                        ));
                    }
                }
                Some(HtmlAuthoredSource::Characters(contributions)) => {
                    for contribution in contributions {
                        let span = (
                            contribution.source().range().start(),
                            contribution.source().range().end(),
                        );
                        if !gold.characters.contains(&span) {
                            failures.push(format!(
                                "{}: contribution span {span:?} is not an authored gold character run",
                                fixture.id
                            ));
                        }
                    }
                }
                None => {}
            }
        }

        // Determinism across repeats and across caller-supplied identities.
        let baseline = signature(&analysis);
        for source_id in [1u64, 2u64] {
            let repeat_source = SourceText::new(SourceId::new(source_id), text.clone());
            match construct_html_document_shell(&repeat_source, limits) {
                Ok(repeat) => {
                    if signature(&repeat) != baseline {
                        failures.push(format!(
                            "{}: non-deterministic result under SourceId {source_id}",
                            fixture.id
                        ));
                    }
                }
                Err(error) => {
                    failures.push(format!("{}: repeat boundary failure {error:?}", fixture.id))
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "TC-S1 cross-layer corpus failures:\n{}",
        failures.join("\n")
    );

    // TC-S1 is a bounded slice, not HTML parsing: this corpus was authored for
    // the tokenizer and most of it lies outside the proved document shell.
    // Asserting a strict subset keeps a future change from quietly claiming
    // the whole corpus is supported TC-S1 input.
    assert!(
        supported < fixtures.len(),
        "TC-S1 must not claim the whole tokenizer corpus is supported"
    );
}

#[test]
fn tree_construction_handles_all_generated_inputs_without_panic() {
    let limits = to_html_limits(Limits::generous());
    let inputs = generated_inputs();
    assert_eq!(inputs.len(), MAX_GENERATED_CASES);

    let mut failures = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        assert!(input.len() <= MAX_SOURCE_BYTES);
        let label = format!("case {index} (source byte length {})", input.len());

        // No `catch_unwind`: a production panic on any generated case fails
        // this test naturally rather than being caught and downgraded.
        let source = SourceText::new(SourceId::new(1), input.clone());
        let standalone_run = tokenize(&source, limits);
        let observed_before = observe(&source, &standalone_run);

        let analysis = match construct_html_document_shell(&source, limits) {
            Ok(analysis) => analysis,
            Err(error) => {
                failures.push(format!("{label}: boundary failure {error:?}"));
                continue;
            }
        };
        if observe(&source, analysis.tokenizer_run()) != observed_before {
            failures.push(format!(
                "{label}: retained tokenizer evidence changed through the tree boundary"
            ));
        }

        check_frozen_relationships(&label, &source, &analysis, &mut failures);
        check_completion_honesty(&label, &analysis, &mut failures);

        // Committed tree coverage is a prefix of the retained source and never
        // claims more than the tokenizer itself processed.
        if analysis.coverage().committed_end() > analysis.tokenizer_run().coverage().processed_end()
        {
            failures.push(format!(
                "{label}: committed tree coverage ran past tokenizer coverage"
            ));
        }

        let baseline = signature(&analysis);
        for source_id in [1u64, 3u64] {
            let repeat_source = SourceText::new(SourceId::new(source_id), input.clone());
            match construct_html_document_shell(&repeat_source, limits) {
                Ok(repeat) => {
                    if signature(&repeat) != baseline {
                        failures.push(format!(
                            "{label}: non-deterministic result under SourceId {source_id}"
                        ));
                    }
                }
                Err(error) => failures.push(format!("{label}: repeat boundary failure {error:?}")),
            }
        }
    }
    assert!(
        failures.is_empty(),
        "TC-S1 generated-input failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn tree_construction_propagates_low_limits_and_invalid_configuration_honestly() {
    // Re-running a slice of the same corpus under deliberately hostile
    // tokenizer configuration: a refused or truncated lower layer must never
    // become an effective Complete tree, and must never panic.
    let hostile = [
        // A source-byte limit almost everything exceeds.
        HtmlTokenizerLimits::new(2, 8_192, 1_024, 1_024, 256, 4_096, 1_024),
        // A single emitted token.
        HtmlTokenizerLimits::new(1_024, 8_192, 1, 1_024, 256, 4_096, 1_024),
        // A single transition step.
        HtmlTokenizerLimits::new(1_024, 1, 1_024, 1_024, 256, 4_096, 1_024),
        // Invalid configuration: a zero transition-step limit.
        HtmlTokenizerLimits::new(1_024, 0, 1_024, 1_024, 256, 4_096, 1_024),
        // Invalid configuration: a zero emitted-token limit.
        HtmlTokenizerLimits::new(1_024, 8_192, 0, 1_024, 256, 4_096, 1_024),
    ];

    let mut failures = Vec::new();
    for fixture in &all_candidate_independent_corpus() {
        let text = String::from_utf8(fixture.source_bytes.to_vec())
            .expect("fixture source is valid UTF-8");
        for (limit_index, limits) in hostile.iter().enumerate() {
            let source = SourceText::new(SourceId::new(1), text.clone());
            let label = format!("{} under hostile limits {limit_index}", fixture.id);
            let analysis = match construct_html_document_shell(&source, *limits) {
                Ok(analysis) => analysis,
                Err(error) => {
                    failures.push(format!("{label}: boundary failure {error:?}"));
                    continue;
                }
            };
            if analysis.tokenizer_run().is_incomplete() && analysis.is_complete() {
                failures.push(format!("{label}: lower-layer incompleteness was upgraded"));
            }
            check_frozen_relationships(&label, &source, &analysis, &mut failures);
            check_completion_honesty(&label, &analysis, &mut failures);
        }
    }
    assert!(
        failures.is_empty(),
        "TC-S1 hostile-configuration failures:\n{}",
        failures.join("\n")
    );
}
