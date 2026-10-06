//! Production freeze sealing for the SelectedOrdinary authored attribute
//! association (Issue #902).
//!
//! The public correspondence of the projected attribute evidence is sealed
//! against hand-counted offsets in `html::tree::tests`. This module seals the
//! other half: that [`freeze`] itself, not the session, rejects every
//! impossible cross-layer mismatch between a selected insertion action, its
//! exact retained StartTag token, and the constructed node. Each candidate is
//! assembled by hand from a valid baseline and perturbed in exactly one way.
//!
//! This is production sealing only. It does not model the attribute theorem
//! (that is the accepted #900/#901 validation) and does not exercise any
//! tokenizer-owned attribute syntax invariant.

use crate::{SourceId, SourceText};

use super::result::{
    HtmlConstructedIdentityCounter, HtmlConstructedNodeId, HtmlDocumentShellAnalysis,
    HtmlDocumentShellParts, HtmlElement, HtmlSelectedOrdinaryElement,
    HtmlSelectedOrdinaryElementName, HtmlShellElement, HtmlShellElementName,
    HtmlShellElementOrigin, HtmlSynthesisCause, HtmlTreeAction, HtmlTreeActionKind,
    HtmlTreeCompletion, HtmlTreeFreezeError, HtmlTreeIncompleteCause, HtmlTreeNode,
    HtmlTreeNodeKind, HtmlTreeTokenTrigger, freeze,
};

use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;

const DIV: HtmlSelectedOrdinaryElementName = HtmlSelectedOrdinaryElementName::Div;

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

/// `<body>` (token 0, 0..6) then a selected `div` start tag (token 1,
/// `6..tag_end`) and its end tag (token 2), with the five committed identities
/// root, html, head, body, div.
struct Fixture {
    source: SourceText,
    run: HtmlTokenizerRunResult,
    ids: Vec<HtmlConstructedNodeId>,
    tag_end: usize,
}

fn fixture_with(text: &str, source_id: u64, run_source_id: u64) -> Fixture {
    let source = SourceText::new(SourceId::new(source_id), text.to_owned());
    let run_source = SourceText::new(SourceId::new(run_source_id), text.to_owned());
    let run = tokenize(&run_source, limits());
    let mut counter = HtmlConstructedIdentityCounter::new();
    let ids = (0..6)
        .map(|_| {
            let reserved = counter.reserve().expect("identity headroom");
            counter.commit(reserved);
            reserved
        })
        .collect();
    let tag_end = text[6..].find('>').expect("a start tag") + 7;
    Fixture {
        source,
        run,
        ids,
        tag_end,
    }
}

fn fixture(text: &str) -> Fixture {
    fixture_with(text, 1, 1)
}

/// A valid open-at-hand-off candidate: one authored selected `div` whose single
/// insertion action is triggered by the exact start-tag token.
fn parts(fixture: &Fixture) -> HtmlDocumentShellParts {
    let [root, html, head, body, selected, _] = fixture.ids[..] else {
        panic!("six minted identities")
    };
    let anchor = |start: usize, end: usize| fixture.source.anchor(start, end).expect("valid range");
    let synthesized = |name| {
        HtmlTreeNodeKind::Element(HtmlElement::Shell(HtmlShellElement::new(
            name,
            HtmlShellElementOrigin::Synthesized(HtmlSynthesisCause::ImpliedByDocumentStructure),
        )))
    };
    HtmlDocumentShellParts {
        nodes: vec![
            HtmlTreeNode::new(root, None, vec![html], HtmlTreeNodeKind::Document),
            HtmlTreeNode::new(
                html,
                Some(root),
                vec![head, body],
                synthesized(HtmlShellElementName::Html),
            ),
            HtmlTreeNode::new(
                head,
                Some(html),
                vec![],
                synthesized(HtmlShellElementName::Head),
            ),
            HtmlTreeNode::new(
                body,
                Some(html),
                vec![selected],
                HtmlTreeNodeKind::Element(HtmlElement::Shell(HtmlShellElement::new(
                    HtmlShellElementName::Body,
                    HtmlShellElementOrigin::Authored {
                        complete: anchor(0, 6),
                        raw_name: anchor(1, 5),
                    },
                ))),
            ),
            HtmlTreeNode::new(
                selected,
                Some(body),
                vec![],
                HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(
                    HtmlSelectedOrdinaryElement::new(
                        DIV,
                        anchor(6, fixture.tag_end),
                        anchor(7, 10),
                    ),
                )),
            ),
        ],
        root,
        admitted_creation_events: 5,
        diagnostics: vec![],
        actions: vec![HtmlTreeAction::new(
            HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement {
                node: selected,
                name: DIV,
            },
            HtmlTreeTokenTrigger::authored(1, anchor(6, fixture.tag_end)),
        )],
        processed_tokens: 2,
        committed_prefix_end: fixture.tag_end,
        completion: HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete),
        final_open_selected_ordinary: vec![selected],
        final_open_paragraph: None,
        final_open_style: None,
        final_open_title: None,
        final_text_mode_active: false,
        final_original_insertion_mode_retained: false,
        pending_tokenizer_feedback: false,
        coordinated_raw_text_entry_tokens: Vec::new(),
        coordinated_raw_text_close_tokens: Vec::new(),
        coordinated_rcdata_entry_tokens: Vec::new(),
        coordinated_rcdata_close_tokens: Vec::new(),
    }
}

fn freeze_parts(
    fixture: &Fixture,
    parts: HtmlDocumentShellParts,
) -> Result<HtmlDocumentShellAnalysis, HtmlTreeFreezeError> {
    freeze(&fixture.source, fixture.run.clone(), parts)
}

fn insertion(
    node: HtmlConstructedNodeId,
    name: HtmlSelectedOrdinaryElementName,
    trigger: HtmlTreeTokenTrigger,
) -> HtmlTreeAction {
    HtmlTreeAction::new(
        HtmlTreeActionKind::InsertedAuthoredSelectedOrdinaryElement { node, name },
        trigger,
    )
}

const ATTRIBUTED: &str = "<body><div id=x></div>";

#[test]
fn freeze_accepts_zero_one_and_a_closed_attributed_selected_start() {
    for text in [
        "<body><div></div>",
        ATTRIBUTED,
        "<body><div id=\"x\"></div>",
    ] {
        let fixture = fixture(text);
        let analysis = freeze_parts(&fixture, parts(&fixture)).expect("valid parts freeze");
        assert_eq!(analysis.node_count(), 5, "{text:?}");
    }
}

#[test]
fn freeze_rejects_an_insertion_triggered_by_a_different_token() {
    let fixture = fixture(ATTRIBUTED);
    let selected = fixture.ids[4];
    let anchor = |start, end| fixture.source.anchor(start, end).expect("valid range");
    // Token 0 is the `<body>` start tag; token 2 is the `div` end tag.
    for (token_index, trigger) in [(0, anchor(0, 6)), (2, anchor(16, 22))] {
        let mut parts = parts(&fixture);
        parts.actions[0] = insertion(
            selected,
            DIV,
            HtmlTreeTokenTrigger::authored(token_index, trigger),
        );
        assert_eq!(
            freeze_parts(&fixture, parts).expect_err("freeze must reject this"),
            HtmlTreeFreezeError::SelectedOrdinaryInsertionTriggerIsNotItsStartTag {
                node: selected,
                token_index,
            },
            "token {token_index}"
        );
    }
}

#[test]
fn freeze_rejects_an_insertion_triggered_by_the_end_of_file_token() {
    let fixture = fixture(ATTRIBUTED);
    let selected = fixture.ids[4];
    let mut parts = parts(&fixture);
    parts.actions[0] = insertion(selected, DIV, HtmlTreeTokenTrigger::end_of_file(3));
    assert_eq!(
        freeze_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::SelectedOrdinaryInsertionTriggerIsNotItsStartTag {
            node: selected,
            token_index: 3,
        }
    );
}

#[test]
fn freeze_rejects_a_trigger_anchor_that_is_not_the_exact_token_anchor() {
    let fixture = fixture(ATTRIBUTED);
    let selected = fixture.ids[4];
    let mut parts = parts(&fixture);
    parts.actions[0] = insertion(
        selected,
        DIV,
        HtmlTreeTokenTrigger::authored(1, fixture.source.anchor(6, fixture.tag_end - 1).unwrap()),
    );
    assert_eq!(
        freeze_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::SelectedOrdinaryInsertionTriggerIsNotItsStartTag {
            node: selected,
            token_index: 1,
        }
    );
}

#[test]
fn freeze_rejects_a_selected_name_that_the_token_does_not_carry() {
    // Node and action agree on `section`, but the exact token is a `div`.
    let fixture = fixture(ATTRIBUTED);
    let selected = fixture.ids[4];
    let anchor = |start, end| fixture.source.anchor(start, end).expect("valid range");
    let mut parts = parts(&fixture);
    parts.nodes[4] = HtmlTreeNode::new(
        selected,
        Some(fixture.ids[3]),
        vec![],
        HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(
            HtmlSelectedOrdinaryElement::new(
                HtmlSelectedOrdinaryElementName::Section,
                anchor(6, fixture.tag_end),
                anchor(7, 10),
            ),
        )),
    );
    parts.actions[0] = insertion(
        selected,
        HtmlSelectedOrdinaryElementName::Section,
        HtmlTreeTokenTrigger::authored(1, anchor(6, fixture.tag_end)),
    );
    assert_eq!(
        freeze_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::SelectedOrdinaryInsertionTriggerIsNotItsStartTag {
            node: selected,
            token_index: 1,
        }
    );
}

#[test]
fn freeze_rejects_a_node_whose_authored_start_is_not_the_trigger_token() {
    let fixture = fixture(ATTRIBUTED);
    let selected = fixture.ids[4];
    let anchor = |start, end| fixture.source.anchor(start, end).expect("valid range");
    let mut parts = parts(&fixture);
    // The node claims a different (still contained, still valid) raw name.
    parts.nodes[4] = HtmlTreeNode::new(
        selected,
        Some(fixture.ids[3]),
        vec![],
        HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(
            HtmlSelectedOrdinaryElement::new(DIV, anchor(6, fixture.tag_end), anchor(7, 9)),
        )),
    );
    assert_eq!(
        freeze_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::SelectedOrdinaryInsertionAuthoredStartMismatch {
            node: selected,
            token_index: 1,
        }
    );
}

#[test]
fn freeze_rejects_an_insertion_whose_subject_is_not_the_selected_node() {
    let fixture = fixture(ATTRIBUTED);
    let body = fixture.ids[3];
    let mut parts = parts(&fixture);
    let trigger = parts.actions[0].trigger().clone();
    parts.actions[0] = insertion(body, DIV, trigger);
    assert_eq!(
        freeze_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::ClosureSubjectIsNotTheSelectedOrdinaryElement {
            node: body,
            name: DIV,
        }
    );
}

#[test]
fn freeze_rejects_a_self_closing_start_token() {
    let fixture = fixture("<body><div id=\"x\"/></div>");
    let selected = fixture.ids[4];
    assert_eq!(
        freeze_parts(&fixture, parts(&fixture)).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::SelectedOrdinaryInsertionTokenIsSelfClosing {
            node: selected,
            token_index: 1,
        }
    );
}

#[test]
fn freeze_rejects_a_token_beyond_the_zero_or_one_attribute_theorem() {
    let fixture = fixture("<body><div a b></div>");
    let selected = fixture.ids[4];
    assert_eq!(
        freeze_parts(&fixture, parts(&fixture)).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::SelectedOrdinaryInsertionAttributeCountExceedsTheorem {
            node: selected,
            token_index: 1,
            attributes: 2,
        }
    );
}

#[test]
fn freeze_rejects_a_token_whose_source_identity_is_not_the_report_source() {
    // The retained run was tokenized from source 2; the candidate is frozen
    // against source 1 with otherwise identical text.
    let fixture = fixture_with(ATTRIBUTED, 1, 2);
    let selected = fixture.ids[4];
    assert_eq!(
        freeze_parts(&fixture, parts(&fixture)).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::SelectedOrdinaryInsertionTokenSourceMismatch {
            node: selected,
            token_index: 1,
        }
    );
}

#[test]
fn freeze_rejects_a_selected_node_without_its_insertion_action() {
    let fixture = fixture(ATTRIBUTED);
    let selected = fixture.ids[4];
    let mut parts = parts(&fixture);
    parts.actions.clear();
    parts.final_open_selected_ordinary.clear();
    assert_eq!(
        freeze_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::SelectedOrdinaryNodeWithoutInsertion(selected)
    );
}

#[test]
fn freeze_rejects_two_nodes_attached_to_one_consumed_token() {
    let fixture = fixture(ATTRIBUTED);
    let [_, _, _, body, first, second] = fixture.ids[..] else {
        panic!("six minted identities")
    };
    let anchor = |start, end| fixture.source.anchor(start, end).expect("valid range");
    let mut parts = parts(&fixture);
    parts.admitted_creation_events = 6;
    parts.nodes[3] = HtmlTreeNode::new(
        body,
        Some(fixture.ids[1]),
        vec![first, second],
        parts.nodes[3].kind().clone(),
    );
    parts.nodes.push(HtmlTreeNode::new(
        second,
        Some(body),
        vec![],
        HtmlTreeNodeKind::Element(HtmlElement::SelectedOrdinary(
            HtmlSelectedOrdinaryElement::new(DIV, anchor(6, fixture.tag_end), anchor(7, 10)),
        )),
    ));
    parts.actions.push(insertion(
        second,
        DIV,
        HtmlTreeTokenTrigger::authored(1, anchor(6, fixture.tag_end)),
    ));
    parts.final_open_selected_ordinary = vec![first, second];
    assert_eq!(
        freeze_parts(&fixture, parts).expect_err("freeze must reject this"),
        HtmlTreeFreezeError::DuplicateSelectedOrdinaryInsertionToken { token_index: 1 }
    );
}
