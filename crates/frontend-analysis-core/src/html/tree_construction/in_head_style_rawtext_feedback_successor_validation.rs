//! Candidate-independent validation for Issue #384.
//!
//! This module validates the selected In-Head `<style>` RAWTEXT feedback
//! lifecycle without importing production tree-construction driver, session,
//! result, or tokenizer-state representation as a semantic oracle.
//!
//! Freshness pins at candidate selection:
//! WHATWG HTML `df14ce3085887cc99d821d238c5192857904de58`, source blob
//! `16cdaecdb5f3db29eac0753a49f401b221ba9247`;
//! WPT `5ce815a83b2601ce920e39f001cd7e77642ea860`.
//!
//! The hand-authored GOLD below is derived from these pinned normative
//! obligations, not from browser, WPT, html5lib, or production output:
//!
//! - `style` in the `in head` insertion mode uses the generic raw-text element
//!   parsing algorithm;
//! - that algorithm inserts the element, switches the tokenizer to RAWTEXT,
//!   retains the original insertion mode, and switches tree construction to
//!   `text`;
//! - RAWTEXT appropriate-end-tag recognition switches the tokenizer back to
//!   Data before the corresponding end-tag token is consumed by the tree;
//! - the non-script end-tag rule in `text` pops the current node and restores
//!   the retained original insertion mode;
//! - EOF in `text` pops the current node and restores the original insertion
//!   mode without fabricating authored closing-tag evidence.
//!
//! WPT/html5lib-family parsing fixtures are challenge/corroboration evidence
//! only. This test-only machine intentionally models only the semantic states
//! needed to falsify the selected theorem; it is not a proposed production
//! tokenizer state layout or coordinator API.

use crate::{SourceId, SourceText};

const FRESH_WHATWG_HEAD: &str = "df14ce3085887cc99d821d238c5192857904de58";
const PINNED_WHATWG_SOURCE_BLOB: &str = "16cdaecdb5f3db29eac0753a49f401b221ba9247";
const FRESH_WPT_HEAD: &str = "5ce815a83b2601ce920e39f001cd7e77642ea860";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Evidence {
    source_id: SourceId,
    start: usize,
    end: usize,
    raw: String,
}

fn evidence(source: &SourceText, start: usize, end: usize) -> Evidence {
    let anchor = source.anchor(start, end).expect("candidate byte range");
    Evidence {
        source_id: anchor.source_id(),
        start: anchor.range().start(),
        end: anchor.range().end(),
        raw: anchor.fragment().to_owned(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexState {
    Data,
    RawText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertionMode {
    InHead,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Name {
    Html,
    Head,
    Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeId(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Origin {
    CandidateContext,
    Authored(Evidence),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Element {
    id: NodeId,
    name: Name,
    parent: Option<NodeId>,
    origin: Origin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StyleRecord {
    id: NodeId,
    start: Evidence,
    text: String,
    contributions: Vec<Evidence>,
    close: Option<Evidence>,
    eof_closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TreeState {
    elements: Vec<Element>,
    open: Vec<NodeId>,
    mode: InsertionMode,
    original_mode: Option<InsertionMode>,
    style: Option<StyleRecord>,
    next_id: usize,
    eof_in_text_diagnostic: bool,
}

impl TreeState {
    fn candidate_prestate() -> Self {
        let html = Element {
            id: NodeId(0),
            name: Name::Html,
            parent: None,
            origin: Origin::CandidateContext,
        };
        let head = Element {
            id: NodeId(1),
            name: Name::Head,
            parent: Some(NodeId(0)),
            origin: Origin::CandidateContext,
        };
        Self {
            elements: vec![html, head],
            open: vec![NodeId(0), NodeId(1)],
            mode: InsertionMode::InHead,
            original_mode: None,
            style: None,
            next_id: 2,
            eof_in_text_diagnostic: false,
        }
    }

    fn insert_style(&mut self, start: Evidence) -> Feedback {
        assert_eq!(self.mode, InsertionMode::InHead);
        assert_eq!(self.open, vec![NodeId(0), NodeId(1)]);
        assert!(self.original_mode.is_none());
        assert!(self.style.is_none());

        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.elements.push(Element {
            id,
            name: Name::Style,
            parent: Some(NodeId(1)),
            origin: Origin::Authored(start.clone()),
        });
        self.open.push(id);
        self.style = Some(StyleRecord {
            id,
            start,
            text: String::new(),
            contributions: Vec::new(),
            close: None,
            eof_closed: false,
        });
        self.original_mode = Some(self.mode);
        self.mode = InsertionMode::Text;
        Feedback::SwitchTokenizer(LexState::RawText)
    }

    fn insert_raw_text(&mut self, contribution: Evidence) {
        assert_eq!(self.mode, InsertionMode::Text);
        let style = self.style.as_mut().expect("open style record");
        style.text.push_str(&contribution.raw);
        style.contributions.push(contribution);
    }

    fn close_style(&mut self, close: Evidence) {
        assert_eq!(self.mode, InsertionMode::Text);
        let style = self.style.as_mut().expect("open style record");
        style.close = Some(close);
        assert_eq!(self.open.pop(), Some(style.id));
        self.mode = self.original_mode.take().expect("retained original mode");
    }

    fn recover_eof_in_text(&mut self) {
        assert_eq!(self.mode, InsertionMode::Text);
        let style = self.style.as_mut().expect("open style record");
        style.eof_closed = true;
        assert_eq!(self.open.pop(), Some(style.id));
        self.mode = self.original_mode.take().expect("retained original mode");
        self.eof_in_text_diagnostic = true;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Feedback {
    None,
    SwitchTokenizer(LexState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenClass {
    StartStyle,
    StartBodySentinel,
    StartOtherB,
    EndStyle,
    Characters,
    Eof,
    UnsupportedStyleStart,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    class: TokenClass,
    evidence: Evidence,
    state_at_emission: LexState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LowerLayerStop {
    ResourceLimit,
}

#[derive(Debug, Clone)]
struct CandidateTokenizer {
    source: SourceText,
    cursor: usize,
    state: LexState,
    byte_limit: Option<usize>,
}

impl CandidateTokenizer {
    fn new(source: &SourceText, cursor: usize, byte_limit: Option<usize>) -> Self {
        Self {
            source: source.clone(),
            cursor,
            state: LexState::Data,
            byte_limit,
        }
    }

    fn set_state(&mut self, state: LexState) {
        self.state = state;
    }

    fn next_token(&mut self) -> Result<Token, LowerLayerStop> {
        if self
            .byte_limit
            .is_some_and(|limit| self.cursor >= limit && self.cursor < self.source.as_str().len())
        {
            return Err(LowerLayerStop::ResourceLimit);
        }
        match self.state {
            LexState::Data => self.next_data(),
            LexState::RawText => self.next_raw_text(),
        }
    }

    fn next_data(&mut self) -> Result<Token, LowerLayerStop> {
        let text = self.source.as_str();
        if self.cursor == text.len() {
            return Ok(self.token(TokenClass::Eof, self.cursor, self.cursor, LexState::Data));
        }

        let rest = &text[self.cursor..];
        if rest.starts_with("<style>") {
            return self.consume(TokenClass::StartStyle, 7, LexState::Data);
        }
        if rest.starts_with("<body>") {
            return self.consume(TokenClass::StartBodySentinel, 6, LexState::Data);
        }
        if rest.starts_with("<b>") {
            return self.consume(TokenClass::StartOtherB, 3, LexState::Data);
        }
        if let Some(end) = plain_style_end_at(text, self.cursor) {
            let start = self.cursor;
            self.cursor = end;
            return Ok(self.token(TokenClass::EndStyle, start, end, LexState::Data));
        }
        if ascii_case_prefix(rest, "<style") {
            let end = rest
                .find('>')
                .map_or(text.len(), |offset| self.cursor + offset + 1);
            let start = self.cursor;
            self.cursor = end;
            return Ok(self.token(
                TokenClass::UnsupportedStyleStart,
                start,
                end,
                LexState::Data,
            ));
        }
        if rest.starts_with('<') {
            let end = rest
                .find('>')
                .map_or(text.len(), |offset| self.cursor + offset + 1);
            let start = self.cursor;
            self.cursor = end;
            return Ok(self.token(TokenClass::Other, start, end, LexState::Data));
        }

        let next = rest
            .find('<')
            .map_or(text.len(), |offset| self.cursor + offset);
        let start = self.cursor;
        self.cursor = next;
        Ok(self.token(TokenClass::Characters, start, next, LexState::Data))
    }

    fn next_raw_text(&mut self) -> Result<Token, LowerLayerStop> {
        let text = self.source.as_str();
        if self.cursor == text.len() {
            return Ok(self.token(TokenClass::Eof, self.cursor, self.cursor, LexState::RawText));
        }

        if let Some(end) = plain_style_end_at(text, self.cursor) {
            let start = self.cursor;
            self.cursor = end;
            self.state = LexState::Data;
            return Ok(self.token(TokenClass::EndStyle, start, end, LexState::RawText));
        }

        let end = find_next_plain_style_end(text, self.cursor).unwrap_or(text.len());
        let start = self.cursor;
        self.cursor = end;
        Ok(self.token(TokenClass::Characters, start, end, LexState::RawText))
    }

    fn consume(
        &mut self,
        class: TokenClass,
        width: usize,
        state: LexState,
    ) -> Result<Token, LowerLayerStop> {
        let start = self.cursor;
        let end = start + width;
        if self.byte_limit.is_some_and(|limit| end > limit) {
            return Err(LowerLayerStop::ResourceLimit);
        }
        self.cursor = end;
        Ok(self.token(class, start, end, state))
    }

    fn token(&self, class: TokenClass, start: usize, end: usize, state: LexState) -> Token {
        Token {
            class,
            evidence: evidence(&self.source, start, end),
            state_at_emission: state,
        }
    }
}

fn ascii_case_prefix(value: &str, prefix: &str) -> bool {
    value.len() >= prefix.len()
        && value.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
}

fn plain_style_end_at(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if start + 3 > bytes.len()
        || bytes.get(start) != Some(&b'<')
        || bytes.get(start + 1) != Some(&b'/')
    {
        return None;
    }
    let mut cursor = start + 2;
    while cursor < bytes.len() && bytes[cursor].is_ascii_alphabetic() {
        cursor += 1;
    }
    if cursor == start + 2 || bytes.get(cursor) != Some(&b'>') {
        return None;
    }
    let name = &bytes[start + 2..cursor];
    name.eq_ignore_ascii_case(b"style").then_some(cursor + 1)
}

fn find_next_plain_style_end(text: &str, start: usize) -> Option<usize> {
    let mut cursor = start + 1;
    while cursor < text.len() {
        if text.as_bytes()[cursor] == b'<' && plain_style_end_at(text, cursor).is_some() {
            return Some(cursor);
        }
        cursor += 1;
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Produced {
        state: LexState,
        class: TokenClass,
        evidence: Evidence,
    },
    StyleInserted {
        id: NodeId,
        start: Evidence,
    },
    FeedbackRequested {
        requested: LexState,
        cursor: usize,
    },
    FeedbackApplied {
        from: LexState,
        to: LexState,
        cursor: usize,
    },
    EnteredText {
        original: InsertionMode,
    },
    RawTextInserted {
        evidence: Evidence,
    },
    TokenizerReturnedToData {
        close: Evidence,
    },
    StyleClosed {
        close: Evidence,
    },
    RestoredMode {
        mode: InsertionMode,
    },
    PostCloseSentinel {
        state: LexState,
        evidence: Evidence,
    },
    EofInTextRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventKind {
    ProducedStartStyleData,
    StyleInserted,
    EnteredText,
    FeedbackRequestedRawText,
    FeedbackAppliedRawText,
    ProducedCharactersRawText,
    RawTextInserted,
    ProducedEndStyleRawText,
    TokenizerReturnedToData,
    StyleClosed,
    RestoredInHead,
    ProducedBodyData,
    PostCloseBodyData,
    Other,
}

fn event_kinds(events: &[Event]) -> Vec<EventKind> {
    events
        .iter()
        .map(|event| match event {
            Event::Produced {
                state: LexState::Data,
                class: TokenClass::StartStyle,
                ..
            } => EventKind::ProducedStartStyleData,
            Event::StyleInserted { .. } => EventKind::StyleInserted,
            Event::EnteredText {
                original: InsertionMode::InHead,
            } => EventKind::EnteredText,
            Event::FeedbackRequested {
                requested: LexState::RawText,
                ..
            } => EventKind::FeedbackRequestedRawText,
            Event::FeedbackApplied {
                from: LexState::Data,
                to: LexState::RawText,
                ..
            } => EventKind::FeedbackAppliedRawText,
            Event::Produced {
                state: LexState::RawText,
                class: TokenClass::Characters,
                ..
            } => EventKind::ProducedCharactersRawText,
            Event::RawTextInserted { .. } => EventKind::RawTextInserted,
            Event::Produced {
                state: LexState::RawText,
                class: TokenClass::EndStyle,
                ..
            } => EventKind::ProducedEndStyleRawText,
            Event::TokenizerReturnedToData { .. } => EventKind::TokenizerReturnedToData,
            Event::StyleClosed { .. } => EventKind::StyleClosed,
            Event::RestoredMode {
                mode: InsertionMode::InHead,
            } => EventKind::RestoredInHead,
            Event::Produced {
                state: LexState::Data,
                class: TokenClass::StartBodySentinel,
                ..
            } => EventKind::ProducedBodyData,
            Event::PostCloseSentinel {
                state: LexState::Data,
                ..
            } => EventKind::PostCloseBodyData,
            _ => EventKind::Other,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Completion {
    Complete,
    LowerLayerIncomplete,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminal {
    AppropriateCloseThenEof,
    PostCloseSentinel,
    EofInRawText,
    LowerLayerStop,
    UnsupportedStyleShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    source_id: SourceId,
    tree: TreeState,
    tokenizer_state: LexState,
    tokenizer_cursor: usize,
    pending_feedback: Option<LexState>,
    events: Vec<Event>,
    completion: Completion,
    terminal: Terminal,
}

fn run_candidate(
    source: &SourceText,
    start_cursor: usize,
    byte_limit: Option<usize>,
) -> Observation {
    let mut tokenizer = CandidateTokenizer::new(source, start_cursor, byte_limit);
    let mut tree = TreeState::candidate_prestate();
    let mut events = Vec::new();
    let mut pending_feedback = None;

    loop {
        let token = match tokenizer.next_token() {
            Ok(token) => token,
            Err(LowerLayerStop::ResourceLimit) => {
                return Observation {
                    source_id: source.id(),
                    tree,
                    tokenizer_state: tokenizer.state,
                    tokenizer_cursor: tokenizer.cursor,
                    pending_feedback,
                    events,
                    completion: Completion::LowerLayerIncomplete,
                    terminal: Terminal::LowerLayerStop,
                };
            }
        };

        events.push(Event::Produced {
            state: token.state_at_emission,
            class: token.class,
            evidence: token.evidence.clone(),
        });

        if token.class == TokenClass::EndStyle && token.state_at_emission == LexState::RawText {
            assert_eq!(tokenizer.state, LexState::Data);
            events.push(Event::TokenizerReturnedToData {
                close: token.evidence.clone(),
            });
        }

        let feedback = match (tree.mode, token.class) {
            (InsertionMode::InHead, TokenClass::StartStyle) => {
                let feedback = tree.insert_style(token.evidence.clone());
                let style = tree.style.as_ref().expect("style after insertion");
                events.push(Event::StyleInserted {
                    id: style.id,
                    start: token.evidence.clone(),
                });
                events.push(Event::EnteredText {
                    original: InsertionMode::InHead,
                });
                feedback
            }
            (InsertionMode::Text, TokenClass::Characters) => {
                tree.insert_raw_text(token.evidence.clone());
                events.push(Event::RawTextInserted {
                    evidence: token.evidence.clone(),
                });
                Feedback::None
            }
            (InsertionMode::Text, TokenClass::EndStyle) => {
                tree.close_style(token.evidence.clone());
                events.push(Event::StyleClosed {
                    close: token.evidence.clone(),
                });
                events.push(Event::RestoredMode {
                    mode: InsertionMode::InHead,
                });
                Feedback::None
            }
            (InsertionMode::Text, TokenClass::Eof) => {
                tree.recover_eof_in_text();
                events.push(Event::EofInTextRecovery);
                events.push(Event::RestoredMode {
                    mode: InsertionMode::InHead,
                });
                return Observation {
                    source_id: source.id(),
                    tree,
                    tokenizer_state: tokenizer.state,
                    tokenizer_cursor: tokenizer.cursor,
                    pending_feedback,
                    events,
                    completion: Completion::Complete,
                    terminal: Terminal::EofInRawText,
                };
            }
            (InsertionMode::InHead, TokenClass::StartBodySentinel)
                if tree
                    .style
                    .as_ref()
                    .is_some_and(|style| style.close.is_some()) =>
            {
                events.push(Event::PostCloseSentinel {
                    state: token.state_at_emission,
                    evidence: token.evidence,
                });
                return Observation {
                    source_id: source.id(),
                    tree,
                    tokenizer_state: tokenizer.state,
                    tokenizer_cursor: tokenizer.cursor,
                    pending_feedback,
                    events,
                    completion: Completion::Complete,
                    terminal: Terminal::PostCloseSentinel,
                };
            }
            (InsertionMode::InHead, TokenClass::Eof)
                if tree
                    .style
                    .as_ref()
                    .is_some_and(|style| style.close.is_some()) =>
            {
                return Observation {
                    source_id: source.id(),
                    tree,
                    tokenizer_state: tokenizer.state,
                    tokenizer_cursor: tokenizer.cursor,
                    pending_feedback,
                    events,
                    completion: Completion::Complete,
                    terminal: Terminal::AppropriateCloseThenEof,
                };
            }
            (InsertionMode::InHead, TokenClass::UnsupportedStyleStart) => {
                return Observation {
                    source_id: source.id(),
                    tree,
                    tokenizer_state: tokenizer.state,
                    tokenizer_cursor: tokenizer.cursor,
                    pending_feedback,
                    events,
                    completion: Completion::Unsupported,
                    terminal: Terminal::UnsupportedStyleShape,
                };
            }
            _ => panic!("fixture escaped closed candidate: {token:?}"),
        };

        if let Feedback::SwitchTokenizer(requested) = feedback {
            assert!(pending_feedback.is_none());
            pending_feedback = Some(requested);
            events.push(Event::FeedbackRequested {
                requested,
                cursor: tokenizer.cursor,
            });
            let applied = pending_feedback
                .take()
                .expect("outstanding feedback request");
            let from = tokenizer.state;
            tokenizer.set_state(applied);
            events.push(Event::FeedbackApplied {
                from,
                to: applied,
                cursor: tokenizer.cursor,
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoldEvidence {
    start: usize,
    end: usize,
    raw: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Gold {
    style_start: GoldEvidence,
    text: &'static str,
    text_contribution: Option<GoldEvidence>,
    close: Option<GoldEvidence>,
    final_lex: LexState,
    terminal: Terminal,
    eof_in_text_diagnostic: bool,
    sentinel: Option<GoldEvidence>,
}

fn assert_gold(observation: &Observation, gold: &Gold) {
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.terminal, gold.terminal);
    assert_eq!(observation.tokenizer_state, gold.final_lex);
    assert_eq!(observation.pending_feedback, None);
    assert_eq!(observation.tree.mode, InsertionMode::InHead);
    assert_eq!(observation.tree.original_mode, None);
    assert_eq!(observation.tree.open, vec![NodeId(0), NodeId(1)]);
    assert_eq!(
        observation.tree.next_id, 3,
        "only style allocates an element"
    );
    assert_eq!(observation.tree.elements.len(), 3);

    let style_element = &observation.tree.elements[2];
    assert_eq!(style_element.id, NodeId(2));
    assert_eq!(style_element.name, Name::Style);
    assert_eq!(style_element.parent, Some(NodeId(1)));
    match &style_element.origin {
        Origin::Authored(origin) => {
            assert_evidence(origin, observation.source_id, &gold.style_start)
        }
        Origin::CandidateContext => panic!("style must have authored origin"),
    }

    let style = observation.tree.style.as_ref().expect("style record");
    assert_evidence(&style.start, observation.source_id, &gold.style_start);
    assert_eq!(style.text, gold.text);
    assert_eq!(
        style.contributions.len(),
        usize::from(gold.text_contribution.is_some())
    );
    if let Some(expected) = &gold.text_contribution {
        assert_evidence(&style.contributions[0], observation.source_id, expected);
    }
    match (&style.close, &gold.close) {
        (Some(actual), Some(expected)) => assert_evidence(actual, observation.source_id, expected),
        (None, None) => {}
        pair => panic!("close evidence mismatch: {pair:?}"),
    }
    assert_eq!(style.eof_closed, gold.terminal == Terminal::EofInRawText);
    assert_eq!(
        observation.tree.eof_in_text_diagnostic,
        gold.eof_in_text_diagnostic
    );

    if let Some(expected) = &gold.sentinel {
        let actual = observation.events.iter().find_map(|event| match event {
            Event::PostCloseSentinel { state, evidence } => Some((*state, evidence)),
            _ => None,
        });
        let (state, evidence) = actual.expect("post-close sentinel event");
        assert_eq!(state, LexState::Data);
        assert_evidence(evidence, observation.source_id, expected);
    }
}

fn assert_evidence(actual: &Evidence, source_id: SourceId, expected: &GoldEvidence) {
    assert_eq!(actual.source_id, source_id);
    assert_eq!((actual.start, actual.end), (expected.start, expected.end));
    assert_eq!(actual.raw, expected.raw);
}

fn assert_core_causal_events(observation: &Observation) {
    let first = observation.events.first().expect("first produced token");
    match first {
        Event::Produced {
            state,
            class,
            evidence,
        } => {
            assert_eq!(*state, LexState::Data);
            assert_eq!(*class, TokenClass::StartStyle);
            assert_eq!(evidence.raw, "<style>");
        }
        other => panic!("unexpected first causal event: {other:?}"),
    }

    let inserted = observation.events.iter().find_map(|event| match event {
        Event::StyleInserted { id, start } => Some((*id, start)),
        _ => None,
    });
    let (id, start) = inserted.expect("style insertion event");
    assert_eq!(id, NodeId(2));
    assert_eq!(start.raw, "<style>");

    let request = observation.events.iter().find_map(|event| match event {
        Event::FeedbackRequested { requested, cursor } => Some((*requested, *cursor)),
        _ => None,
    });
    assert_eq!(request, Some((LexState::RawText, start.end)));

    let entered = observation.events.iter().find_map(|event| match event {
        Event::EnteredText { original } => Some(*original),
        _ => None,
    });
    assert_eq!(entered, Some(InsertionMode::InHead));

    if let Some(style) = &observation.tree.style
        && let Some(close) = &style.close
    {
        let tokenizer_close = observation.events.iter().find_map(|event| match event {
            Event::TokenizerReturnedToData { close } => Some(close),
            _ => None,
        });
        assert_eq!(tokenizer_close, Some(close));
        let tree_close = observation.events.iter().find_map(|event| match event {
            Event::StyleClosed { close } => Some(close),
            _ => None,
        });
        assert_eq!(tree_close, Some(close));
        let restored = observation.events.iter().find_map(|event| match event {
            Event::RestoredMode { mode } => Some(*mode),
            _ => None,
        });
        assert_eq!(restored, Some(InsertionMode::InHead));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FreezeError {
    PendingFeedback,
    ActiveTextWithoutOriginal,
    ClosedPathTokenizerNotData,
    ClosedPathTreeNotRestored,
    ClosedPathStyleStillOpen,
    EofPathClaimsAuthoredClose,
}

fn validate_candidate_freeze(observation: &Observation) -> Result<(), FreezeError> {
    if observation.completion != Completion::Complete {
        return Ok(());
    }
    if observation.pending_feedback.is_some() {
        return Err(FreezeError::PendingFeedback);
    }
    if observation.tree.mode == InsertionMode::Text && observation.tree.original_mode.is_none() {
        return Err(FreezeError::ActiveTextWithoutOriginal);
    }

    match observation.terminal {
        Terminal::AppropriateCloseThenEof | Terminal::PostCloseSentinel => {
            if observation.tokenizer_state != LexState::Data {
                return Err(FreezeError::ClosedPathTokenizerNotData);
            }
            if observation.tree.mode != InsertionMode::InHead
                || observation.tree.original_mode.is_some()
            {
                return Err(FreezeError::ClosedPathTreeNotRestored);
            }
            if observation.tree.open != vec![NodeId(0), NodeId(1)] {
                return Err(FreezeError::ClosedPathStyleStillOpen);
            }
        }
        Terminal::EofInRawText => {
            let style = observation.tree.style.as_ref().expect("EOF style record");
            if style.close.is_some() {
                return Err(FreezeError::EofPathClaimsAuthoredClose);
            }
            assert_eq!(observation.tokenizer_state, LexState::RawText);
            assert_eq!(observation.tree.mode, InsertionMode::InHead);
            assert_eq!(observation.tree.open, vec![NodeId(0), NodeId(1)]);
        }
        Terminal::LowerLayerStop | Terminal::UnsupportedStyleShape => {}
    }
    Ok(())
}

fn produced_classes(observation: &Observation) -> Vec<(LexState, TokenClass)> {
    observation
        .events
        .iter()
        .filter_map(|event| match event {
            Event::Produced { state, class, .. } => Some((*state, *class)),
            _ => None,
        })
        .collect()
}

fn eager_all_data_classes(source: &SourceText, start_cursor: usize) -> Vec<TokenClass> {
    let mut tokenizer = CandidateTokenizer::new(source, start_cursor, None);
    let mut classes = Vec::new();
    loop {
        tokenizer.set_state(LexState::Data);
        let token = tokenizer.next_token().expect("unbounded eager probe");
        classes.push(token.class);
        if token.class == TokenClass::Eof {
            return classes;
        }
    }
}

fn late_feedback_probe(source: &SourceText) -> (TokenClass, LexState) {
    let mut tokenizer = CandidateTokenizer::new(source, 0, None);
    let mut tree = TreeState::candidate_prestate();
    let style = tokenizer.next_token().expect("style token");
    assert_eq!(style.class, TokenClass::StartStyle);
    let feedback = tree.insert_style(style.evidence);
    assert_eq!(feedback, Feedback::SwitchTokenizer(LexState::RawText));

    // Deliberately violate Candidate C sequencing: produce the next token
    // before applying the feedback request.
    let too_early = tokenizer.next_token().expect("late-feedback probe token");
    (too_early.class, too_early.state_at_emission)
}

fn semantic_projection(observation: &Observation) -> (String, Terminal, LexState, InsertionMode) {
    (
        observation
            .tree
            .style
            .as_ref()
            .map_or_else(String::new, |style| style.text.clone()),
        observation.terminal,
        observation.tokenizer_state,
        observation.tree.mode,
    )
}

fn source(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

#[test]
fn r1_empty_style_proves_complete_feedback_round_trip() {
    let source = source(1, "<style></style>");
    let actual = run_candidate(&source, 0, None);
    assert_gold(
        &actual,
        &Gold {
            style_start: GoldEvidence {
                start: 0,
                end: 7,
                raw: "<style>",
            },
            text: "",
            text_contribution: None,
            close: Some(GoldEvidence {
                start: 7,
                end: 15,
                raw: "</style>",
            }),
            final_lex: LexState::Data,
            terminal: Terminal::AppropriateCloseThenEof,
            eof_in_text_diagnostic: false,
            sentinel: None,
        },
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
    assert_core_causal_events(&actual);
    assert_eq!(actual.tokenizer_cursor, 15);
    assert!(actual.events.iter().any(|event| matches!(
        event,
        Event::FeedbackApplied {
            from: LexState::Data,
            to: LexState::RawText,
            cursor: 7
        }
    )));
    assert!(
        actual
            .events
            .iter()
            .any(|event| matches!(event, Event::TokenizerReturnedToData { .. }))
    );
}

#[test]
fn r2_f1_f3_tag_shaped_raw_text_falsifies_completed_data_vector_and_reinterpretation() {
    let source = source(2, "<style><b>x</style>");
    let actual = run_candidate(&source, 0, None);
    assert_gold(
        &actual,
        &Gold {
            style_start: GoldEvidence {
                start: 0,
                end: 7,
                raw: "<style>",
            },
            text: "<b>x",
            text_contribution: Some(GoldEvidence {
                start: 7,
                end: 11,
                raw: "<b>x",
            }),
            close: Some(GoldEvidence {
                start: 11,
                end: 19,
                raw: "</style>",
            }),
            final_lex: LexState::Data,
            terminal: Terminal::AppropriateCloseThenEof,
            eof_in_text_diagnostic: false,
            sentinel: None,
        },
    );

    let eager = eager_all_data_classes(&source, 0);
    assert!(eager.contains(&TokenClass::StartOtherB));
    assert!(
        !produced_classes(&actual)
            .iter()
            .any(|(_, class)| *class == TokenClass::StartOtherB)
    );

    // The eager history has a `<b>` token boundary [7, 10), while the
    // coordinated history has one RAWTEXT contribution [7, 11). Repairing the
    // eager history therefore requires retokenization/reconstruction, not a
    // semantics-preserving downstream label change.
    let raw = actual
        .events
        .iter()
        .find_map(|event| match event {
            Event::RawTextInserted { evidence } => Some(evidence),
            _ => None,
        })
        .expect("rawtext contribution");
    assert_eq!((raw.start, raw.end), (7, 11));
    assert_eq!(actual.tree.next_id, 3, "no element identity for `<b>`");
}

#[test]
fn r3_non_appropriate_end_tag_candidate_remains_raw_text() {
    let source = source(3, "<style>x</styler>y</style>");
    let actual = run_candidate(&source, 0, None);
    assert_gold(
        &actual,
        &Gold {
            style_start: GoldEvidence {
                start: 0,
                end: 7,
                raw: "<style>",
            },
            text: "x</styler>y",
            text_contribution: Some(GoldEvidence {
                start: 7,
                end: 18,
                raw: "x</styler>y",
            }),
            close: Some(GoldEvidence {
                start: 18,
                end: 26,
                raw: "</style>",
            }),
            final_lex: LexState::Data,
            terminal: Terminal::AppropriateCloseThenEof,
            eof_in_text_diagnostic: false,
            sentinel: None,
        },
    );
}

#[test]
fn r4_mixed_case_appropriate_close_preserves_raw_spelling() {
    let source = source(4, "<style>x</StYlE>");
    let actual = run_candidate(&source, 0, None);
    assert_gold(
        &actual,
        &Gold {
            style_start: GoldEvidence {
                start: 0,
                end: 7,
                raw: "<style>",
            },
            text: "x",
            text_contribution: Some(GoldEvidence {
                start: 7,
                end: 8,
                raw: "x",
            }),
            close: Some(GoldEvidence {
                start: 8,
                end: 16,
                raw: "</StYlE>",
            }),
            final_lex: LexState::Data,
            terminal: Terminal::AppropriateCloseThenEof,
            eof_in_text_diagnostic: false,
            sentinel: None,
        },
    );
}

#[test]
fn r5_f2_f4_full_source_sentinel_proves_feedback_is_early_and_round_trip_is_two_way() {
    let fixture_source = source(5, "<head><style><b>x</style><body>");
    let actual = run_candidate(&fixture_source, 6, None);
    assert_gold(
        &actual,
        &Gold {
            style_start: GoldEvidence {
                start: 6,
                end: 13,
                raw: "<style>",
            },
            text: "<b>x",
            text_contribution: Some(GoldEvidence {
                start: 13,
                end: 17,
                raw: "<b>x",
            }),
            close: Some(GoldEvidence {
                start: 17,
                end: 25,
                raw: "</style>",
            }),
            final_lex: LexState::Data,
            terminal: Terminal::PostCloseSentinel,
            eof_in_text_diagnostic: false,
            sentinel: Some(GoldEvidence {
                start: 25,
                end: 31,
                raw: "<body>",
            }),
        },
    );

    assert_eq!(
        event_kinds(&actual.events),
        vec![
            EventKind::ProducedStartStyleData,
            EventKind::StyleInserted,
            EventKind::EnteredText,
            EventKind::FeedbackRequestedRawText,
            EventKind::FeedbackAppliedRawText,
            EventKind::ProducedCharactersRawText,
            EventKind::RawTextInserted,
            EventKind::ProducedEndStyleRawText,
            EventKind::TokenizerReturnedToData,
            EventKind::StyleClosed,
            EventKind::RestoredInHead,
            EventKind::ProducedBodyData,
            EventKind::PostCloseBodyData,
        ],
        "the selected round trip is a causal ordering theorem, not only a final-state theorem"
    );

    let late_source = source(55, "<style><b>x</style>");
    assert_eq!(
        late_feedback_probe(&late_source),
        (TokenClass::StartOtherB, LexState::Data),
        "producing before feedback observes the wrong lexical history"
    );

    let classes = produced_classes(&actual);
    assert!(classes.contains(&(LexState::RawText, TokenClass::Characters)));
    assert!(classes.contains(&(LexState::RawText, TokenClass::EndStyle)));
    assert!(classes.contains(&(LexState::Data, TokenClass::StartBodySentinel)));
}

#[test]
fn r6_eof_in_raw_text_restores_tree_without_fabricating_close_or_data_transition() {
    let source = source(6, "<style><b>x");
    let actual = run_candidate(&source, 0, None);
    assert_gold(
        &actual,
        &Gold {
            style_start: GoldEvidence {
                start: 0,
                end: 7,
                raw: "<style>",
            },
            text: "<b>x",
            text_contribution: Some(GoldEvidence {
                start: 7,
                end: 11,
                raw: "<b>x",
            }),
            close: None,
            final_lex: LexState::RawText,
            terminal: Terminal::EofInRawText,
            eof_in_text_diagnostic: true,
            sentinel: None,
        },
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
    assert!(
        !actual
            .events
            .iter()
            .any(|event| matches!(event, Event::TokenizerReturnedToData { .. }))
    );
}

#[test]
fn r7_raw_text_less_than_and_non_name_end_open_fallback_stays_text() {
    let source = source(7, "<style>a</!b<c</style>");
    let actual = run_candidate(&source, 0, None);
    assert_gold(
        &actual,
        &Gold {
            style_start: GoldEvidence {
                start: 0,
                end: 7,
                raw: "<style>",
            },
            text: "a</!b<c",
            text_contribution: Some(GoldEvidence {
                start: 7,
                end: 14,
                raw: "a</!b<c",
            }),
            close: Some(GoldEvidence {
                start: 14,
                end: 22,
                raw: "</style>",
            }),
            final_lex: LexState::Data,
            terminal: Terminal::AppropriateCloseThenEof,
            eof_in_text_diagnostic: false,
            sentinel: None,
        },
    );
}

#[test]
fn r8_source_id_perturbation_changes_provenance_not_semantics() {
    let first = source(80, "<style><b>x</style>");
    let second = source(81, "<style><b>x</style>");
    let first_observation = run_candidate(&first, 0, None);
    let second_observation = run_candidate(&second, 0, None);

    assert_eq!(
        semantic_projection(&first_observation),
        semantic_projection(&second_observation)
    );
    assert_ne!(first_observation.source_id, second_observation.source_id);
    assert_ne!(
        first_observation
            .tree
            .style
            .as_ref()
            .unwrap()
            .start
            .source_id,
        second_observation
            .tree
            .style
            .as_ref()
            .unwrap()
            .start
            .source_id
    );
}

#[test]
fn r9_repeated_runs_are_deterministic() {
    let source = source(9, "<style>x</styler>y</style>");
    assert_eq!(
        run_candidate(&source, 0, None),
        run_candidate(&source, 0, None)
    );
}

#[test]
fn r10_lower_layer_stop_is_never_upgraded_to_complete() {
    let source = source(10, "<style><b>x</style>");
    let actual = run_candidate(&source, 0, Some(7));
    assert_eq!(actual.completion, Completion::LowerLayerIncomplete);
    assert_eq!(actual.terminal, Terminal::LowerLayerStop);
    assert_eq!(actual.tokenizer_state, LexState::RawText);
    assert_eq!(actual.tokenizer_cursor, 7);
    assert_eq!(actual.tree.mode, InsertionMode::Text);
    assert_eq!(actual.tree.original_mode, Some(InsertionMode::InHead));
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
}

#[test]
fn r11_excluded_style_shape_refuses_before_tree_identity_or_feedback_mutation() {
    let source = source(11, "<style media=x>");
    let actual = run_candidate(&source, 0, None);
    assert_eq!(actual.completion, Completion::Unsupported);
    assert_eq!(actual.terminal, Terminal::UnsupportedStyleShape);
    assert_eq!(actual.tree, TreeState::candidate_prestate());
    assert_eq!(actual.tokenizer_state, LexState::Data);
    assert_eq!(actual.tokenizer_cursor, 15);
    assert_eq!(actual.pending_feedback, None);
    assert!(!actual.events.iter().any(|event| matches!(
        event,
        Event::FeedbackRequested { .. }
            | Event::FeedbackApplied { .. }
            | Event::StyleInserted { .. }
    )));
}

#[test]
fn r12_f5_f6_freeze_rejects_inconsistent_feedback_and_restoration_state() {
    let fixture_source = source(12, "<style>x</style>");
    let valid = run_candidate(&fixture_source, 0, None);
    assert_eq!(validate_candidate_freeze(&valid), Ok(()));

    let mut pending = valid.clone();
    pending.pending_feedback = Some(LexState::RawText);
    assert_eq!(
        validate_candidate_freeze(&pending),
        Err(FreezeError::PendingFeedback)
    );

    let mut wrong_lex = valid.clone();
    wrong_lex.tokenizer_state = LexState::RawText;
    assert_eq!(
        validate_candidate_freeze(&wrong_lex),
        Err(FreezeError::ClosedPathTokenizerNotData)
    );

    let mut missing_original = valid.clone();
    missing_original.tree.mode = InsertionMode::Text;
    missing_original.tree.original_mode = None;
    assert_eq!(
        validate_candidate_freeze(&missing_original),
        Err(FreezeError::ActiveTextWithoutOriginal)
    );

    let mut unrestored = valid.clone();
    unrestored.tree.mode = InsertionMode::Text;
    unrestored.tree.original_mode = Some(InsertionMode::InHead);
    assert_eq!(
        validate_candidate_freeze(&unrestored),
        Err(FreezeError::ClosedPathTreeNotRestored)
    );

    let mut style_still_open = valid.clone();
    let style_id = style_still_open.tree.style.as_ref().unwrap().id;
    style_still_open.tree.open.push(style_id);
    assert_eq!(
        validate_candidate_freeze(&style_still_open),
        Err(FreezeError::ClosedPathStyleStillOpen)
    );

    let eof_source = source(120, "<style>x");
    let mut fake_close = run_candidate(&eof_source, 0, None);
    fake_close.tree.style.as_mut().unwrap().close = Some(evidence(&eof_source, 8, 8));
    assert_eq!(
        validate_candidate_freeze(&fake_close),
        Err(FreezeError::EofPathClaimsAuthoredClose)
    );
}

#[test]
fn f7_closed_candidate_needs_no_generic_scope_or_implied_end_domain() {
    let source = source(13, "<style><b>x</style>");
    let actual = run_candidate(&source, 0, None);
    assert_eq!(
        actual
            .tree
            .elements
            .iter()
            .map(|element| element.name)
            .collect::<Vec<_>>(),
        vec![Name::Html, Name::Head, Name::Style]
    );
    assert_eq!(actual.tree.open, vec![NodeId(0), NodeId(1)]);
    assert_eq!(actual.tree.next_id, 3);
}

#[test]
fn f8_gold_is_normative_hand_authored_and_external_heads_are_freshness_markers_only() {
    assert_eq!(FRESH_WHATWG_HEAD.len(), 40);
    assert_eq!(PINNED_WHATWG_SOURCE_BLOB.len(), 40);
    assert_eq!(FRESH_WPT_HEAD.len(), 40);
    assert_ne!(FRESH_WHATWG_HEAD, FRESH_WPT_HEAD);

    let source = source(14, "<style>x</style>");
    let actual = run_candidate(&source, 0, None);
    let hand_authored = Gold {
        style_start: GoldEvidence {
            start: 0,
            end: 7,
            raw: "<style>",
        },
        text: "x",
        text_contribution: Some(GoldEvidence {
            start: 7,
            end: 8,
            raw: "x",
        }),
        close: Some(GoldEvidence {
            start: 8,
            end: 16,
            raw: "</style>",
        }),
        final_lex: LexState::Data,
        terminal: Terminal::AppropriateCloseThenEof,
        eof_in_text_diagnostic: false,
        sentinel: None,
    };
    assert_gold(&actual, &hand_authored);
}
