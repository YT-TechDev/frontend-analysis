//! Candidate-independent validation for Issue #390.
//!
//! This module validates the selected In-Head `<title>` RCDATA + Named
//! Character-Reference causal lifecycle without importing the production
//! tree-construction driver, session, result, tokenizer producer, or tokenizer
//! state representation as a semantic oracle. Every expected answer below is
//! hand-authored from the pinned normative obligations.
//!
//! Freshness pins at candidate selection:
//! WHATWG HTML `9ead9de8f6751ccb98e91972e580ed6e3314c64a`, source blob
//! `c090774473c6b2bc77f48e94167f43f469bba14e`;
//! WPT `dd432b1d351796d3f25e1d1f243ba52da16c3a0a`;
//! html5lib-tests `224991ec10db04f056a89eed8b0bd8695fd2950e`.
//!
//! The hand-authored GOLD is derived from these pinned obligations:
//!
//! - `title` in the `in head` insertion mode uses the generic RCDATA element
//!   parsing algorithm;
//! - that algorithm inserts the element, switches the tokenizer to RCDATA,
//!   retains the original insertion mode, and switches tree construction to
//!   `text`;
//! - RCDATA appropriate-end-tag recognition returns the tokenizer to Data
//!   before the corresponding end-tag token is consumed by the tree;
//! - the non-script end-tag rule in `text` pops the current node and restores
//!   the retained original insertion mode;
//! - EOF in `text` pops the current node, restores the original insertion mode,
//!   and requires the same EOF token to be reprocessed, without fabricating
//!   authored closing-tag evidence. This bounded model exposes that reprocess
//!   as an explicit non-complete checkpoint rather than widening into later
//!   modes;
//! - the RCDATA return state reaches the character reference state on `&`;
//!   the named character reference state consumes the *maximum* number of
//!   characters that form a named-reference identifier, reports
//!   `missing-semicolon-after-character-reference` when the match does not end
//!   in `;`, and otherwise falls through to the ambiguous ampersand state,
//!   which reports `unknown-named-character-reference` on `;`;
//! - decoded characters are output only. They are never reintroduced as
//!   tokenizer input, so recursive decoding cannot occur and a decoded
//!   `</title>` is not an authored end tag;
//! - one authored named reference may decode to more than one Unicode scalar.
//!
//! Scope boundaries deliberately preserved by this bounded machine:
//!
//! - Numeric Character References are **not** selected. The numeric cell below
//!   proves Title entry, RCDATA entry, and character-reference entry all
//!   succeed and that the single remaining unsupported requirement is
//!   specifically the Numeric branch.
//! - `textarea` is an RCDATA element that this candidate does **not** select.
//!   The bounded machine refuses it, so a durable RCDATA tokenizer-mode
//!   vocabulary cannot be read as authorization for every RCDATA element.
//! - The tree->tokenizer control modelled here is the title-specific
//!   [`Feedback::EnterRcdataForTitle`]. It carries no tokenizer-mode operand
//!   and is not a proposed production type or API name.
//!
//! The named-reference data below is a deliberately small **test-local**
//! subset of the WHATWG named character references table, chosen only to
//! exercise the maximum-match, multi-scalar, and ambiguous-ampersand
//! obligations. It is `#[cfg(test)]`-only, it is not a production lookup
//! table, and it is faithful for every authored cell in this module: for each
//! of those cells no omitted WHATWG identifier is a prefix of the relevant
//! remaining input.
//!
//! WPT and html5lib-family parsing fixtures are challenge/corroboration
//! evidence only. This test-only machine models exactly the semantic states
//! needed to falsify the selected theorem; it is not a proposed production
//! tokenizer state layout, cursor API, diagnostic enum, entity-table
//! placement, or coordinator contract.

use crate::{SourceId, SourceText};

const FRESH_WHATWG_HEAD: &str = "9ead9de8f6751ccb98e91972e580ed6e3314c64a";
const PINNED_WHATWG_SOURCE_BLOB: &str = "c090774473c6b2bc77f48e94167f43f469bba14e";
const FRESH_WPT_HEAD: &str = "dd432b1d351796d3f25e1d1f243ba52da16c3a0a";
const FRESH_HTML5LIB_TESTS_HEAD: &str = "224991ec10db04f056a89eed8b0bd8695fd2950e";

// ---------------------------------------------------------------------------
// Authored evidence
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Test-local named character-reference data and maximum matching
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NamedEntry {
    name: &'static str,
    value: &'static str,
}

/// Bounded test-local subset of the pinned named character references table.
///
/// Both semicolonless and semicolon-terminated identifiers are present so that
/// maximum matching, the missing-semicolon obligation, and the multi-scalar
/// obligation are all reachable.
const NAMED_REFERENCES: &[NamedEntry] = &[
    NamedEntry {
        name: "acE;",
        value: "\u{223e}\u{0333}",
    },
    NamedEntry {
        name: "amp",
        value: "\u{0026}",
    },
    NamedEntry {
        name: "amp;",
        value: "\u{0026}",
    },
    NamedEntry {
        name: "gt",
        value: "\u{003e}",
    },
    NamedEntry {
        name: "gt;",
        value: "\u{003e}",
    },
    NamedEntry {
        name: "lt",
        value: "\u{003c}",
    },
    NamedEntry {
        name: "lt;",
        value: "\u{003c}",
    },
    NamedEntry {
        name: "ne;",
        value: "\u{2260}",
    },
    NamedEntry {
        name: "not",
        value: "\u{00ac}",
    },
    NamedEntry {
        name: "not;",
        value: "\u{00ac}",
    },
    NamedEntry {
        name: "notin;",
        value: "\u{2209}",
    },
    NamedEntry {
        name: "notni;",
        value: "\u{220c}",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NamedLookahead {
    matched: Option<NamedEntry>,
    /// Byte length, from the first identifier scalar, that a prefix walk must
    /// look at before it can know the maximum match is final. The failing
    /// scalar is included: that is exactly the scalar a committing
    /// implementation would wrongly preprocess early.
    examined_len: usize,
}

/// Maximum-match discovery.
///
/// This is a pure function of the remaining input. It performs a forward
/// prefix walk only: no cursor exists here to roll back, no second tokenizer
/// is constructed, no source is searched or rescanned, and no already-produced
/// output is reinterpreted.
fn named_lookahead(rest: &str) -> NamedLookahead {
    let mut examined_len = 0usize;
    let mut walked = 0usize;
    for scalar in rest.chars() {
        let next = walked + scalar.len_utf8();
        examined_len = next;
        if !NAMED_REFERENCES
            .iter()
            .any(|entry| entry.name.starts_with(&rest[..next]))
        {
            break;
        }
        walked = next;
    }

    let matched = NAMED_REFERENCES
        .iter()
        .filter(|entry| rest.starts_with(entry.name))
        .max_by_key(|entry| entry.name.len())
        .copied();

    NamedLookahead {
        matched,
        examined_len,
    }
}

/// Falsification probe: recognize only a complete `name;` run.
fn exact_whole_string_lookup(rest: &str) -> Option<NamedEntry> {
    let semicolon = rest.find(';')?;
    let whole = &rest[..=semicolon];
    NAMED_REFERENCES
        .iter()
        .find(|entry| entry.name == whole)
        .copied()
}

/// Falsification probe: a tiny hard-coded whitelist.
fn tiny_whitelist_lookup(rest: &str) -> Option<NamedEntry> {
    const WHITELIST: &[&str] = &["amp;", "lt;", "gt;"];
    NAMED_REFERENCES
        .iter()
        .find(|entry| WHITELIST.contains(&entry.name) && rest.starts_with(entry.name))
        .copied()
}

// ---------------------------------------------------------------------------
// Bounded semantic states
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexState {
    Data,
    Rcdata,
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
    Title,
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

/// How one authored source range contributed to interpreted Title text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContributionOrigin {
    /// Ordinary RCDATA characters. Interpreted text equals authored text.
    RawTextRun,
    /// A resolved named character reference. Interpreted text is decoded
    /// output and is never re-tokenized.
    ResolvedNamedReference { name: &'static str },
    /// An ampersand run that did not resolve. Interpreted text equals authored
    /// text; this is not a syntax token and not a resolved reference.
    UnresolvedAmpersandRun,
    /// A NUL replaced by U+FFFD. Interpreted text differs from authored text
    /// without any reference having been resolved.
    NullReplacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Contribution {
    origin: ContributionOrigin,
    authored: Evidence,
    interpreted: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TitleRecord {
    id: NodeId,
    start: Evidence,
    /// The single coalesced interpreted text node.
    text: String,
    /// Ordered, distinct authored contributions behind `text`.
    contributions: Vec<Contribution>,
    close: Option<Evidence>,
    eof_closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TreeState {
    elements: Vec<Element>,
    open: Vec<NodeId>,
    mode: InsertionMode,
    original_mode: Option<InsertionMode>,
    titles: Vec<TitleRecord>,
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
            titles: Vec::new(),
            next_id: 2,
            eof_in_text_diagnostic: false,
        }
    }

    fn open_title(&self) -> Option<&TitleRecord> {
        self.titles
            .last()
            .filter(|record| record.close.is_none() && !record.eof_closed)
    }

    fn all_titles_closed(&self) -> bool {
        !self.titles.is_empty() && self.open_title().is_none()
    }

    fn insert_title(&mut self, start: Evidence) -> Feedback {
        assert_eq!(self.mode, InsertionMode::InHead);
        assert_eq!(self.open, vec![NodeId(0), NodeId(1)]);
        assert!(self.original_mode.is_none());
        assert!(self.open_title().is_none());

        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.elements.push(Element {
            id,
            name: Name::Title,
            parent: Some(NodeId(1)),
            origin: Origin::Authored(start.clone()),
        });
        self.open.push(id);
        self.titles.push(TitleRecord {
            id,
            start,
            text: String::new(),
            contributions: Vec::new(),
            close: None,
            eof_closed: false,
        });
        Feedback::EnterRcdataForTitle
    }

    fn enter_text_after_feedback(&mut self) {
        assert_eq!(self.mode, InsertionMode::InHead);
        assert!(self.original_mode.is_none());
        let title = self.open_title().expect("inserted title before Text");
        assert_eq!(self.open.last(), Some(&title.id));
        self.original_mode = Some(self.mode);
        self.mode = InsertionMode::Text;
    }

    fn insert_contribution(&mut self, contribution: Contribution) {
        assert_eq!(self.mode, InsertionMode::Text);
        let title = self.titles.last_mut().expect("open title record");
        assert!(title.close.is_none() && !title.eof_closed);
        title.text.push_str(&contribution.interpreted);
        title.contributions.push(contribution);
    }

    fn close_title(&mut self, close: Evidence) {
        assert_eq!(self.mode, InsertionMode::Text);
        let title = self.titles.last_mut().expect("open title record");
        title.close = Some(close);
        let id = title.id;
        assert_eq!(self.open.pop(), Some(id));
        self.mode = self.original_mode.take().expect("retained original mode");
    }

    fn recover_eof_in_text(&mut self) {
        assert_eq!(self.mode, InsertionMode::Text);
        let title = self.titles.last_mut().expect("open title record");
        title.eof_closed = true;
        let id = title.id;
        assert_eq!(self.open.pop(), Some(id));
        self.mode = self.original_mode.take().expect("retained original mode");
        self.eof_in_text_diagnostic = true;
    }
}

/// Title-specific semantic feedback.
///
/// Deliberately *not* a generic `SwitchTokenizer(mode)` control: the candidate
/// discharges one selected element's RCDATA requirement and carries no
/// tokenizer-mode operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Feedback {
    None,
    EnterRcdataForTitle,
}

// ---------------------------------------------------------------------------
// Diagnostics (test-local vocabulary)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticKind {
    MissingSemicolonAfterNamedReference,
    UnknownNamedCharacterReference,
    UnexpectedNullCharacter,
    ControlCharacterInInputStream,
    EofInTitleText,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    kind: DiagnosticKind,
    evidence: Evidence,
}

fn preprocessing_diagnostic_kind(scalar: char) -> Option<DiagnosticKind> {
    let code = scalar as u32;
    let c0_control = code <= 0x1f && !matches!(scalar, '\0' | '\t' | '\n' | '\u{000c}' | '\r');
    let c1_control = (0x7f..=0x9f).contains(&code);
    (c0_control || c1_control).then_some(DiagnosticKind::ControlCharacterInInputStream)
}

// ---------------------------------------------------------------------------
// Bounded tokenizer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenClass {
    StartTitle,
    StartBodySentinel,
    RawTextRun,
    NamedReference {
        name: &'static str,
        value: &'static str,
    },
    AmpersandFlush,
    NullReplacement,
    NumericReferenceStart,
    EndTitle,
    Eof,
    UnsupportedTitleShape,
    UnsupportedTextareaShape,
    OtherData,
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

/// Observation instrumentation for one maximum-match discovery.
///
/// This records what a prefix walk had to look at. It is harness evidence, not
/// retained tokenizer evidence: the assertions in [`CandidateTokenizer`] prove
/// the discovery advanced no cursor, committed no preprocessing diagnostic,
/// advanced no coverage, and produced no token.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LookaheadRecord {
    at: usize,
    examined_end: usize,
    matched: Option<&'static str>,
    cursor_before: usize,
    cursor_after: usize,
    coverage_before: usize,
    coverage_after: usize,
    diagnostics_before: usize,
    diagnostics_after: usize,
}

#[derive(Debug, Clone)]
struct CandidateTokenizer {
    source: SourceText,
    cursor: usize,
    /// Committed processed boundary. Speculative discovery may never move it.
    coverage_end: usize,
    state: LexState,
    diagnostics: Vec<Diagnostic>,
    character_reference_entries: Vec<Evidence>,
    lookaheads: Vec<LookaheadRecord>,
    byte_limit: Option<usize>,
}

impl CandidateTokenizer {
    fn new(source: &SourceText, cursor: usize, byte_limit: Option<usize>) -> Self {
        Self {
            source: source.clone(),
            cursor,
            coverage_end: cursor,
            state: LexState::Data,
            diagnostics: Vec::new(),
            character_reference_entries: Vec::new(),
            lookaheads: Vec::new(),
            byte_limit,
        }
    }

    fn enter_rcdata_for_title(&mut self) {
        self.state = LexState::Rcdata;
    }

    /// Advance the single authoritative cursor forward and run the committed
    /// input-preprocessing lifecycle over exactly the selected source units.
    fn commit(&mut self, end: usize) {
        assert!(end >= self.cursor, "the cursor never rolls back");
        let start = self.cursor;
        let committed = self.source.as_str()[start..end].to_owned();
        let mut offset = start;
        for scalar in committed.chars() {
            if let Some(kind) = preprocessing_diagnostic_kind(scalar) {
                let span = evidence(&self.source, offset, offset + scalar.len_utf8());
                self.diagnostics.push(Diagnostic {
                    kind,
                    evidence: span,
                });
            }
            offset += scalar.len_utf8();
        }
        self.cursor = end;
        self.coverage_end = end;
    }

    /// Non-committing maximum-match discovery.
    ///
    /// The `&self` receiver makes non-commitment structural: this call cannot
    /// touch the cursor, coverage, diagnostics, or emitted tokens.
    fn peek_named_lookahead(&self, at: usize) -> NamedLookahead {
        named_lookahead(&self.source.as_str()[at..])
    }

    fn commit_state(&self) -> (usize, usize, usize) {
        (self.cursor, self.coverage_end, self.diagnostics.len())
    }

    fn record_lookahead(
        &mut self,
        at: usize,
        lookahead: NamedLookahead,
        before: (usize, usize, usize),
        after: (usize, usize, usize),
    ) {
        assert_eq!(
            before, after,
            "maximum-match discovery must be non-committing"
        );
        self.lookaheads.push(LookaheadRecord {
            at,
            examined_end: at + lookahead.examined_len,
            matched: lookahead.matched.map(|entry| entry.name),
            cursor_before: before.0,
            cursor_after: after.0,
            coverage_before: before.1,
            coverage_after: after.1,
            diagnostics_before: before.2,
            diagnostics_after: after.2,
        });
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
            LexState::Rcdata => self.next_rcdata(),
        }
    }

    fn next_data(&mut self) -> Result<Token, LowerLayerStop> {
        let text = self.source.as_str().to_owned();
        if self.cursor == text.len() {
            return Ok(self.token(TokenClass::Eof, self.cursor, self.cursor, LexState::Data));
        }

        let rest = &text[self.cursor..];
        if rest.starts_with("<title>") {
            return self.consume(TokenClass::StartTitle, 7, LexState::Data);
        }
        if rest.starts_with("<body>") {
            return self.consume(TokenClass::StartBodySentinel, 6, LexState::Data);
        }
        if ascii_case_prefix(rest, "<title") {
            let end = tag_shape_end(&text, self.cursor);
            let start = self.cursor;
            self.commit(end);
            return Ok(self.token(
                TokenClass::UnsupportedTitleShape,
                start,
                end,
                LexState::Data,
            ));
        }
        if ascii_case_prefix(rest, "<textarea") {
            let end = tag_shape_end(&text, self.cursor);
            let start = self.cursor;
            self.commit(end);
            return Ok(self.token(
                TokenClass::UnsupportedTextareaShape,
                start,
                end,
                LexState::Data,
            ));
        }
        if let Some(end) = appropriate_title_end_at(&text, self.cursor) {
            let start = self.cursor;
            self.commit(end);
            return Ok(self.token(TokenClass::EndTitle, start, end, LexState::Data));
        }
        if rest.starts_with('<') {
            let end = tag_shape_end(&text, self.cursor);
            let start = self.cursor;
            self.commit(end);
            return Ok(self.token(TokenClass::OtherData, start, end, LexState::Data));
        }

        let end = rest
            .find('<')
            .map_or(text.len(), |offset| self.cursor + offset);
        let start = self.cursor;
        self.commit(end);
        Ok(self.token(TokenClass::OtherData, start, end, LexState::Data))
    }

    fn next_rcdata(&mut self) -> Result<Token, LowerLayerStop> {
        let text = self.source.as_str().to_owned();
        if self.cursor == text.len() {
            return Ok(self.token(TokenClass::Eof, self.cursor, self.cursor, LexState::Rcdata));
        }

        if let Some(end) = appropriate_title_end_at(&text, self.cursor) {
            let start = self.cursor;
            self.commit(end);
            self.state = LexState::Data;
            return Ok(self.token(TokenClass::EndTitle, start, end, LexState::Rcdata));
        }

        let rest = &text[self.cursor..];
        if rest.starts_with('\0') {
            let start = self.cursor;
            self.commit(start + 1);
            let span = evidence(&self.source, start, start + 1);
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::UnexpectedNullCharacter,
                evidence: span,
            });
            return Ok(self.token(
                TokenClass::NullReplacement,
                start,
                start + 1,
                LexState::Rcdata,
            ));
        }

        if rest.starts_with('&') {
            return self.character_reference(&text);
        }

        let start = self.cursor;
        let mut end = start;
        while end < text.len() {
            let scalar = text[end..].chars().next().expect("scalar boundary");
            if scalar == '&' || scalar == '\0' {
                break;
            }
            if scalar == '<' && appropriate_title_end_at(&text, end).is_some() {
                break;
            }
            end += scalar.len_utf8();
        }
        assert!(
            end > start,
            "an RCDATA run always consumes at least one scalar"
        );
        self.commit(end);
        Ok(self.token(TokenClass::RawTextRun, start, end, LexState::Rcdata))
    }

    /// Character reference state with the RCDATA return state.
    fn character_reference(&mut self, text: &str) -> Result<Token, LowerLayerStop> {
        let ampersand = self.cursor;
        let after = ampersand + 1;
        let entry_span = evidence(&self.source, ampersand, after);
        self.character_reference_entries.push(entry_span);

        let next = text[after..].chars().next();
        match next {
            Some('#') => {
                // Numeric branch: reached, deliberately not selected. Nothing
                // is committed, so coverage stops at the ampersand.
                let end = after + 1;
                Ok(self.token(
                    TokenClass::NumericReferenceStart,
                    ampersand,
                    end,
                    LexState::Rcdata,
                ))
            }
            Some(scalar) if scalar.is_ascii_alphanumeric() => {
                let before = self.commit_state();
                let lookahead = self.peek_named_lookahead(after);
                let settled = self.commit_state();
                self.record_lookahead(after, lookahead, before, settled);
                match lookahead.matched {
                    Some(entry) => {
                        let end = after + entry.name.len();
                        self.commit(end);
                        if !entry.name.ends_with(';') {
                            let span = evidence(&self.source, ampersand, end);
                            self.diagnostics.push(Diagnostic {
                                kind: DiagnosticKind::MissingSemicolonAfterNamedReference,
                                evidence: span,
                            });
                        }
                        Ok(self.token(
                            TokenClass::NamedReference {
                                name: entry.name,
                                value: entry.value,
                            },
                            ampersand,
                            end,
                            LexState::Rcdata,
                        ))
                    }
                    None => {
                        // Flush the ampersand, then the ambiguous ampersand
                        // state's alphanumeric run.
                        let mut end = after;
                        while let Some(scalar) = text[end..].chars().next() {
                            if scalar.is_ascii_alphanumeric() {
                                end += scalar.len_utf8();
                            } else {
                                break;
                            }
                        }
                        self.commit(end);
                        if text[end..].starts_with(';') {
                            let span = evidence(&self.source, end, end + 1);
                            self.diagnostics.push(Diagnostic {
                                kind: DiagnosticKind::UnknownNamedCharacterReference,
                                evidence: span,
                            });
                        }
                        Ok(
                            self.token(
                                TokenClass::AmpersandFlush,
                                ampersand,
                                end,
                                LexState::Rcdata,
                            ),
                        )
                    }
                }
            }
            _ => {
                self.commit(after);
                Ok(self.token(
                    TokenClass::AmpersandFlush,
                    ampersand,
                    after,
                    LexState::Rcdata,
                ))
            }
        }
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
        self.commit(end);
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

fn tag_shape_end(text: &str, start: usize) -> usize {
    text[start..]
        .find('>')
        .map_or(text.len(), |offset| start + offset + 1)
}

/// A plain `</name>` end tag whose name is an ASCII case-insensitive `title`.
fn appropriate_title_end_at(text: &str, start: usize) -> Option<usize> {
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
    name.eq_ignore_ascii_case(b"title").then_some(cursor + 1)
}

// ---------------------------------------------------------------------------
// Causal event log
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Produced {
        state: LexState,
        class: TokenClass,
        evidence: Evidence,
    },
    TitleInserted {
        id: NodeId,
        start: Evidence,
    },
    FeedbackRequested {
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
    TextContributed {
        origin: ContributionOrigin,
        authored: Evidence,
        interpreted: String,
    },
    NamedReferenceResolved {
        name: &'static str,
        authored: Evidence,
        interpreted: String,
    },
    CharacterReferenceEntered {
        ampersand: Evidence,
    },
    TokenizerReturnedToData {
        close: Evidence,
    },
    TitleClosed {
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
    SameTokenReprocessPending {
        class: TokenClass,
        state: LexState,
        evidence: Evidence,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventKind {
    ProducedStartTitleData,
    TitleInserted,
    FeedbackRequested,
    FeedbackApplied,
    EnteredText,
    ProducedRawTextRunRcdata,
    ProducedNamedReferenceRcdata,
    CharacterReferenceEntered,
    NamedReferenceResolved,
    TextContributed,
    ProducedEndTitleRcdata,
    TokenizerReturnedToData,
    TitleClosed,
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
                class: TokenClass::StartTitle,
                ..
            } => EventKind::ProducedStartTitleData,
            Event::TitleInserted { .. } => EventKind::TitleInserted,
            Event::FeedbackRequested { .. } => EventKind::FeedbackRequested,
            Event::FeedbackApplied {
                from: LexState::Data,
                to: LexState::Rcdata,
                ..
            } => EventKind::FeedbackApplied,
            Event::EnteredText {
                original: InsertionMode::InHead,
            } => EventKind::EnteredText,
            Event::Produced {
                state: LexState::Rcdata,
                class: TokenClass::RawTextRun,
                ..
            } => EventKind::ProducedRawTextRunRcdata,
            Event::Produced {
                state: LexState::Rcdata,
                class: TokenClass::NamedReference { .. },
                ..
            } => EventKind::ProducedNamedReferenceRcdata,
            Event::CharacterReferenceEntered { .. } => EventKind::CharacterReferenceEntered,
            Event::NamedReferenceResolved { .. } => EventKind::NamedReferenceResolved,
            Event::TextContributed { .. } => EventKind::TextContributed,
            Event::Produced {
                state: LexState::Rcdata,
                class: TokenClass::EndTitle,
                ..
            } => EventKind::ProducedEndTitleRcdata,
            Event::TokenizerReturnedToData { .. } => EventKind::TokenizerReturnedToData,
            Event::TitleClosed { .. } => EventKind::TitleClosed,
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

// ---------------------------------------------------------------------------
// Observation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Completion {
    Complete,
    PendingSameTokenReprocess,
    LowerLayerIncomplete,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminal {
    AppropriateCloseThenEof,
    PostCloseSentinel,
    EofInRcdataReprocessPending,
    LowerLayerStop,
    UnsupportedShape,
    UnsupportedNumericCharacterReference,
}

/// Narrow, honest unsupported requirements.
///
/// The Numeric variant deliberately names only the numeric branch. A coarser
/// "all RCDATA character references are unsupported" claim would be false, and
/// [`coarse_all_rcdata_character_references_unsupported`] proves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedRequirement {
    NumericCharacterReferenceInRcdata,
    NonSelectedTitleStartShape,
    NonTitleRcdataElement,
    TitleOutsideSelectedInHeadContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    source_id: SourceId,
    tree: TreeState,
    tokenizer_state: LexState,
    tokenizer_cursor: usize,
    tokenizer_coverage_end: usize,
    diagnostics: Vec<Diagnostic>,
    character_reference_entries: Vec<Evidence>,
    lookaheads: Vec<LookaheadRecord>,
    pending_feedback: Option<Feedback>,
    pending_reprocess: Option<Token>,
    events: Vec<Event>,
    completion: Completion,
    terminal: Terminal,
    unsupported_requirement: Option<UnsupportedRequirement>,
}

struct Machine {
    tokenizer: CandidateTokenizer,
    tree: TreeState,
    events: Vec<Event>,
    pending_feedback: Option<Feedback>,
}

impl Machine {
    fn observe(
        self,
        source: &SourceText,
        pending_reprocess: Option<Token>,
        completion: Completion,
        terminal: Terminal,
        unsupported_requirement: Option<UnsupportedRequirement>,
    ) -> Observation {
        let mut diagnostics = self.tokenizer.diagnostics.clone();
        if self.tree.eof_in_text_diagnostic
            && let Some(token) = pending_reprocess.as_ref()
        {
            diagnostics.push(Diagnostic {
                kind: DiagnosticKind::EofInTitleText,
                evidence: token.evidence.clone(),
            });
        }
        Observation {
            source_id: source.id(),
            tree: self.tree,
            tokenizer_state: self.tokenizer.state,
            tokenizer_cursor: self.tokenizer.cursor,
            tokenizer_coverage_end: self.tokenizer.coverage_end,
            diagnostics,
            character_reference_entries: self.tokenizer.character_reference_entries,
            lookaheads: self.tokenizer.lookaheads,
            pending_feedback: self.pending_feedback,
            pending_reprocess,
            events: self.events,
            completion,
            terminal,
            unsupported_requirement,
        }
    }
}

fn run_candidate(
    source: &SourceText,
    start_cursor: usize,
    byte_limit: Option<usize>,
) -> Observation {
    let mut machine = Machine {
        tokenizer: CandidateTokenizer::new(source, start_cursor, byte_limit),
        tree: TreeState::candidate_prestate(),
        events: Vec::new(),
        pending_feedback: None,
    };

    loop {
        let token = match machine.tokenizer.next_token() {
            Ok(token) => token,
            Err(LowerLayerStop::ResourceLimit) => {
                return machine.observe(
                    source,
                    None,
                    Completion::LowerLayerIncomplete,
                    Terminal::LowerLayerStop,
                    None,
                );
            }
        };

        machine.events.push(Event::Produced {
            state: token.state_at_emission,
            class: token.class,
            evidence: token.evidence.clone(),
        });

        if token.state_at_emission == LexState::Rcdata
            && matches!(
                token.class,
                TokenClass::NamedReference { .. }
                    | TokenClass::AmpersandFlush
                    | TokenClass::NumericReferenceStart
            )
            && let Some(ampersand) = machine.tokenizer.character_reference_entries.last()
        {
            machine.events.push(Event::CharacterReferenceEntered {
                ampersand: ampersand.clone(),
            });
        }

        if token.class == TokenClass::EndTitle && token.state_at_emission == LexState::Rcdata {
            assert_eq!(machine.tokenizer.state, LexState::Data);
            machine.events.push(Event::TokenizerReturnedToData {
                close: token.evidence.clone(),
            });
        }

        let feedback = match (machine.tree.mode, token.class) {
            (InsertionMode::InHead, TokenClass::StartTitle) => {
                let feedback = machine.tree.insert_title(token.evidence.clone());
                let title = machine.tree.open_title().expect("title after insertion");
                machine.events.push(Event::TitleInserted {
                    id: title.id,
                    start: token.evidence.clone(),
                });
                feedback
            }
            (InsertionMode::Text, TokenClass::RawTextRun) => {
                contribute(
                    &mut machine,
                    ContributionOrigin::RawTextRun,
                    token.evidence.clone(),
                    token.evidence.raw.clone(),
                );
                Feedback::None
            }
            (InsertionMode::Text, TokenClass::AmpersandFlush) => {
                contribute(
                    &mut machine,
                    ContributionOrigin::UnresolvedAmpersandRun,
                    token.evidence.clone(),
                    token.evidence.raw.clone(),
                );
                Feedback::None
            }
            (InsertionMode::Text, TokenClass::NullReplacement) => {
                contribute(
                    &mut machine,
                    ContributionOrigin::NullReplacement,
                    token.evidence.clone(),
                    "\u{fffd}".to_owned(),
                );
                Feedback::None
            }
            (InsertionMode::Text, TokenClass::NamedReference { name, value }) => {
                contribute(
                    &mut machine,
                    ContributionOrigin::ResolvedNamedReference { name },
                    token.evidence.clone(),
                    value.to_owned(),
                );
                machine.events.push(Event::NamedReferenceResolved {
                    name,
                    authored: token.evidence.clone(),
                    interpreted: value.to_owned(),
                });
                Feedback::None
            }
            (InsertionMode::Text, TokenClass::NumericReferenceStart) => {
                return machine.observe(
                    source,
                    None,
                    Completion::Unsupported,
                    Terminal::UnsupportedNumericCharacterReference,
                    Some(UnsupportedRequirement::NumericCharacterReferenceInRcdata),
                );
            }
            (InsertionMode::Text, TokenClass::EndTitle) => {
                machine.tree.close_title(token.evidence.clone());
                machine.events.push(Event::TitleClosed {
                    close: token.evidence.clone(),
                });
                machine.events.push(Event::RestoredMode {
                    mode: InsertionMode::InHead,
                });
                Feedback::None
            }
            (InsertionMode::Text, TokenClass::Eof) => {
                machine.tree.recover_eof_in_text();
                machine.events.push(Event::EofInTextRecovery);
                machine.events.push(Event::RestoredMode {
                    mode: InsertionMode::InHead,
                });
                machine.events.push(Event::SameTokenReprocessPending {
                    class: token.class,
                    state: token.state_at_emission,
                    evidence: token.evidence.clone(),
                });
                return machine.observe(
                    source,
                    Some(token),
                    Completion::PendingSameTokenReprocess,
                    Terminal::EofInRcdataReprocessPending,
                    None,
                );
            }
            (InsertionMode::InHead, TokenClass::StartBodySentinel)
                if machine.tree.all_titles_closed() =>
            {
                machine.events.push(Event::PostCloseSentinel {
                    state: token.state_at_emission,
                    evidence: token.evidence,
                });
                return machine.observe(
                    source,
                    None,
                    Completion::Complete,
                    Terminal::PostCloseSentinel,
                    None,
                );
            }
            (InsertionMode::InHead, TokenClass::Eof) if machine.tree.all_titles_closed() => {
                return machine.observe(
                    source,
                    None,
                    Completion::Complete,
                    Terminal::AppropriateCloseThenEof,
                    None,
                );
            }
            (InsertionMode::InHead, TokenClass::UnsupportedTitleShape) => {
                return machine.observe(
                    source,
                    None,
                    Completion::Unsupported,
                    Terminal::UnsupportedShape,
                    Some(UnsupportedRequirement::NonSelectedTitleStartShape),
                );
            }
            (InsertionMode::InHead, TokenClass::UnsupportedTextareaShape) => {
                return machine.observe(
                    source,
                    None,
                    Completion::Unsupported,
                    Terminal::UnsupportedShape,
                    Some(UnsupportedRequirement::NonTitleRcdataElement),
                );
            }
            (InsertionMode::InHead, TokenClass::StartBodySentinel) => {
                return machine.observe(
                    source,
                    None,
                    Completion::Unsupported,
                    Terminal::UnsupportedShape,
                    Some(UnsupportedRequirement::TitleOutsideSelectedInHeadContext),
                );
            }
            _ => panic!("fixture escaped closed candidate: {token:?}"),
        };

        if feedback == Feedback::EnterRcdataForTitle {
            assert!(machine.pending_feedback.is_none());
            machine.pending_feedback = Some(feedback);
            machine.events.push(Event::FeedbackRequested {
                cursor: machine.tokenizer.cursor,
            });
            let applied = machine
                .pending_feedback
                .take()
                .expect("outstanding feedback request");
            assert_eq!(applied, Feedback::EnterRcdataForTitle);
            let from = machine.tokenizer.state;
            machine.tokenizer.enter_rcdata_for_title();
            machine.events.push(Event::FeedbackApplied {
                from,
                to: machine.tokenizer.state,
                cursor: machine.tokenizer.cursor,
            });
            machine.tree.enter_text_after_feedback();
            machine.events.push(Event::EnteredText {
                original: InsertionMode::InHead,
            });
        }
    }
}

fn contribute(
    machine: &mut Machine,
    origin: ContributionOrigin,
    authored: Evidence,
    interpreted: String,
) {
    machine.tree.insert_contribution(Contribution {
        origin,
        authored: authored.clone(),
        interpreted: interpreted.clone(),
    });
    machine.events.push(Event::TextContributed {
        origin,
        authored,
        interpreted,
    });
}

// ---------------------------------------------------------------------------
// Hand-authored GOLD
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoldEvidence {
    start: usize,
    end: usize,
    raw: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoldContribution {
    origin: ContributionOrigin,
    authored: GoldEvidence,
    interpreted: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoldTitle {
    start: GoldEvidence,
    text: &'static str,
    contributions: Vec<GoldContribution>,
    close: Option<GoldEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoldDiagnostic {
    kind: DiagnosticKind,
    at: GoldEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Gold {
    titles: Vec<GoldTitle>,
    diagnostics: Vec<GoldDiagnostic>,
    final_lex: LexState,
    completion: Completion,
    terminal: Terminal,
    pending_reprocess: Option<(LexState, TokenClass)>,
    eof_in_text_diagnostic: bool,
    sentinel: Option<GoldEvidence>,
}

fn assert_evidence(actual: &Evidence, source_id: SourceId, expected: &GoldEvidence) {
    assert_eq!(actual.source_id, source_id);
    assert_eq!((actual.start, actual.end), (expected.start, expected.end));
    assert_eq!(actual.raw, expected.raw);
}

fn assert_gold(observation: &Observation, gold: &Gold) {
    assert_eq!(observation.completion, gold.completion);
    assert_eq!(observation.terminal, gold.terminal);
    assert_eq!(observation.tokenizer_state, gold.final_lex);
    assert_eq!(observation.pending_feedback, None);
    assert_eq!(
        observation
            .pending_reprocess
            .as_ref()
            .map(|token| (token.state_at_emission, token.class)),
        gold.pending_reprocess
    );
    assert_eq!(observation.tree.mode, InsertionMode::InHead);
    assert_eq!(observation.tree.original_mode, None);
    assert_eq!(observation.tree.open, vec![NodeId(0), NodeId(1)]);
    assert_eq!(
        observation.tree.eof_in_text_diagnostic,
        gold.eof_in_text_diagnostic
    );

    assert_eq!(
        observation.tree.next_id,
        2 + gold.titles.len(),
        "only selected Title start tags allocate element identity"
    );
    assert_eq!(observation.tree.elements.len(), 2 + gold.titles.len());
    assert_eq!(observation.tree.titles.len(), gold.titles.len());

    for (index, expected) in gold.titles.iter().enumerate() {
        let element = &observation.tree.elements[2 + index];
        assert_eq!(element.id, NodeId(2 + index));
        assert_eq!(element.name, Name::Title);
        assert_eq!(element.parent, Some(NodeId(1)));
        match &element.origin {
            Origin::Authored(origin) => {
                assert_evidence(origin, observation.source_id, &expected.start);
            }
            Origin::CandidateContext => panic!("title must have authored origin"),
        }

        let title = &observation.tree.titles[index];
        assert_eq!(title.id, NodeId(2 + index));
        assert_evidence(&title.start, observation.source_id, &expected.start);
        assert_eq!(title.text, expected.text);
        assert_eq!(title.contributions.len(), expected.contributions.len());
        for (contribution, gold_contribution) in
            title.contributions.iter().zip(&expected.contributions)
        {
            assert_eq!(contribution.origin, gold_contribution.origin);
            assert_evidence(
                &contribution.authored,
                observation.source_id,
                &gold_contribution.authored,
            );
            assert_eq!(contribution.interpreted, gold_contribution.interpreted);
        }
        match (&title.close, &expected.close) {
            (Some(actual), Some(want)) => assert_evidence(actual, observation.source_id, want),
            (None, None) => {}
            pair => panic!("close evidence mismatch: {pair:?}"),
        }
        assert_eq!(
            title.eof_closed,
            expected.close.is_none() && gold.terminal == Terminal::EofInRcdataReprocessPending
        );
    }

    assert_eq!(
        observation.diagnostics.len(),
        gold.diagnostics.len(),
        "diagnostics: {:?}",
        observation.diagnostics
    );
    for (actual, expected) in observation.diagnostics.iter().zip(&gold.diagnostics) {
        assert_eq!(actual.kind, expected.kind);
        assert_evidence(&actual.evidence, observation.source_id, &expected.at);
    }

    if let Some(expected) = &gold.sentinel {
        let found = observation.events.iter().find_map(|event| match event {
            Event::PostCloseSentinel { state, evidence } => Some((*state, evidence)),
            _ => None,
        });
        let (state, found_evidence) = found.expect("post-close sentinel event");
        assert_eq!(state, LexState::Data);
        assert_evidence(found_evidence, observation.source_id, expected);
    }
}

// ---------------------------------------------------------------------------
// Causal-order and freeze validation
// ---------------------------------------------------------------------------

fn assert_core_causal_events(observation: &Observation) {
    let first = observation.events.first().expect("first produced token");
    match first {
        Event::Produced {
            state,
            class,
            evidence,
        } => {
            assert_eq!(*state, LexState::Data);
            assert_eq!(*class, TokenClass::StartTitle);
            assert_eq!(evidence.raw, "<title>");
        }
        other => panic!("unexpected first causal event: {other:?}"),
    }

    let inserted = observation.events.iter().find_map(|event| match event {
        Event::TitleInserted { id, start } => Some((*id, start)),
        _ => None,
    });
    let (id, start) = inserted.expect("title insertion event");
    assert_eq!(id, NodeId(2));
    assert_eq!(start.raw, "<title>");

    let request = observation.events.iter().find_map(|event| match event {
        Event::FeedbackRequested { cursor } => Some(*cursor),
        _ => None,
    });
    assert_eq!(
        request,
        Some(start.end),
        "feedback is requested before any later source production"
    );

    let applied = observation.events.iter().find_map(|event| match event {
        Event::FeedbackApplied { from, to, cursor } => Some((*from, *to, *cursor)),
        _ => None,
    });
    assert_eq!(applied, Some((LexState::Data, LexState::Rcdata, start.end)));

    let entered = observation.events.iter().find_map(|event| match event {
        Event::EnteredText { original } => Some(*original),
        _ => None,
    });
    assert_eq!(entered, Some(InsertionMode::InHead));

    if let Some(title) = observation.tree.titles.first()
        && let Some(close) = &title.close
    {
        let tokenizer_close = observation.events.iter().find_map(|event| match event {
            Event::TokenizerReturnedToData { close } => Some(close),
            _ => None,
        });
        assert_eq!(tokenizer_close, Some(close));
        let tree_close = observation.events.iter().find_map(|event| match event {
            Event::TitleClosed { close } => Some(close),
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
    OutstandingSameTokenReprocess,
    ActiveTextWithoutOriginal,
    ClosedPathTokenizerNotData,
    ClosedPathTreeNotRestored,
    ClosedPathTitleStillOpen,
    ClosedPathCloseEvidenceMismatch,
    ClosedPathImpossibleConstructedIdentity,
    EofPathClaimsAuthoredClose,
    FabricatedContributionEvidence,
    OverlappingContributionEvidence,
    ContributionOriginContradictsInterpretation,
    CoalescedTextContradictsContributions,
}

fn selected_constructed_identity_is_valid(tree: &TreeState) -> bool {
    let title_count = tree.titles.len();
    if tree.next_id != 2 + title_count || tree.elements.len() != 2 + title_count {
        return false;
    }
    let Some((html, rest)) = tree.elements.split_first() else {
        return false;
    };
    let Some((head, titles)) = rest.split_first() else {
        return false;
    };
    if html.id != NodeId(0) || html.name != Name::Html || html.parent.is_some() {
        return false;
    }
    if head.id != NodeId(1) || head.name != Name::Head || head.parent != Some(NodeId(0)) {
        return false;
    }
    titles.iter().enumerate().all(|(index, element)| {
        element.id == NodeId(2 + index)
            && element.name == Name::Title
            && element.parent == Some(NodeId(1))
            && tree
                .titles
                .get(index)
                .is_some_and(|record| record.id == element.id)
    })
}

fn retained_closes_match_emitted_closes(observation: &Observation) -> bool {
    let retained: Vec<&Evidence> = observation
        .tree
        .titles
        .iter()
        .filter_map(|title| title.close.as_ref())
        .collect();
    let emitted: Vec<&Evidence> = observation
        .events
        .iter()
        .filter_map(|event| match event {
            Event::Produced {
                state: LexState::Rcdata,
                class: TokenClass::EndTitle,
                evidence,
            } => Some(evidence),
            _ => None,
        })
        .collect();
    retained == emitted
}

/// Authored contributions must be real, ordered, non-overlapping, inside the
/// Title lifetime, and honest about whether interpretation changed the text.
fn validate_contributions(tree: &TreeState) -> Result<(), FreezeError> {
    for title in &tree.titles {
        let mut previous_end = title.start.end;
        let mut coalesced = String::new();
        for contribution in &title.contributions {
            let authored = &contribution.authored;
            if authored.end <= authored.start || authored.end - authored.start != authored.raw.len()
            {
                return Err(FreezeError::FabricatedContributionEvidence);
            }
            if authored.start < previous_end {
                return Err(FreezeError::OverlappingContributionEvidence);
            }
            if let Some(close) = &title.close
                && authored.end > close.start
            {
                return Err(FreezeError::OverlappingContributionEvidence);
            }
            previous_end = authored.end;

            let honest = match contribution.origin {
                ContributionOrigin::RawTextRun | ContributionOrigin::UnresolvedAmpersandRun => {
                    contribution.interpreted == authored.raw
                }
                ContributionOrigin::ResolvedNamedReference { name } => {
                    !name.is_empty()
                        && authored.raw.starts_with('&')
                        && authored.raw.len() == name.len() + 1
                        && authored.raw[1..] == *name
                }
                ContributionOrigin::NullReplacement => {
                    authored.raw == "\0" && contribution.interpreted == "\u{fffd}"
                }
            };
            if !honest {
                return Err(FreezeError::ContributionOriginContradictsInterpretation);
            }
            coalesced.push_str(&contribution.interpreted);
        }
        if coalesced != title.text {
            return Err(FreezeError::CoalescedTextContradictsContributions);
        }
    }
    Ok(())
}

fn validate_candidate_freeze(observation: &Observation) -> Result<(), FreezeError> {
    if observation.pending_feedback.is_some() {
        return Err(FreezeError::PendingFeedback);
    }
    if observation.completion == Completion::Complete && observation.pending_reprocess.is_some() {
        return Err(FreezeError::OutstandingSameTokenReprocess);
    }
    if observation.tree.mode == InsertionMode::Text && observation.tree.original_mode.is_none() {
        return Err(FreezeError::ActiveTextWithoutOriginal);
    }
    validate_contributions(&observation.tree)?;

    match observation.terminal {
        Terminal::AppropriateCloseThenEof | Terminal::PostCloseSentinel => {
            if observation.completion != Completion::Complete {
                return Ok(());
            }
            if observation.tokenizer_state != LexState::Data {
                return Err(FreezeError::ClosedPathTokenizerNotData);
            }
            if observation.tree.mode != InsertionMode::InHead
                || observation.tree.original_mode.is_some()
            {
                return Err(FreezeError::ClosedPathTreeNotRestored);
            }
            if observation.tree.open != vec![NodeId(0), NodeId(1)]
                || !observation.tree.all_titles_closed()
            {
                return Err(FreezeError::ClosedPathTitleStillOpen);
            }
            if !retained_closes_match_emitted_closes(observation) {
                return Err(FreezeError::ClosedPathCloseEvidenceMismatch);
            }
            if !selected_constructed_identity_is_valid(&observation.tree) {
                return Err(FreezeError::ClosedPathImpossibleConstructedIdentity);
            }
        }
        Terminal::EofInRcdataReprocessPending => {
            let title = observation
                .tree
                .titles
                .last()
                .expect("EOF path retains a title record");
            if title.close.is_some() {
                return Err(FreezeError::EofPathClaimsAuthoredClose);
            }
            assert_eq!(observation.tokenizer_state, LexState::Rcdata);
            assert_eq!(observation.tree.mode, InsertionMode::InHead);
            assert_eq!(observation.tree.open, vec![NodeId(0), NodeId(1)]);
        }
        Terminal::LowerLayerStop
        | Terminal::UnsupportedShape
        | Terminal::UnsupportedNumericCharacterReference => {}
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Falsification probes
// ---------------------------------------------------------------------------

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

fn diagnostic_kinds(observation: &Observation) -> Vec<DiagnosticKind> {
    observation
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.kind)
        .collect()
}

fn eager_all_data_classes(source: &SourceText, start_cursor: usize) -> Vec<TokenClass> {
    let mut tokenizer = CandidateTokenizer::new(source, start_cursor, None);
    let mut classes = Vec::new();
    loop {
        tokenizer.state = LexState::Data;
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
    let title = tokenizer.next_token().expect("title token");
    assert_eq!(title.class, TokenClass::StartTitle);
    let feedback = tree.insert_title(title.evidence);
    assert_eq!(feedback, Feedback::EnterRcdataForTitle);

    // Deliberately violate Candidate C sequencing: produce the next token
    // before applying the feedback request.
    let too_early = tokenizer.next_token().expect("late-feedback probe token");
    (too_early.class, too_early.state_at_emission)
}

/// Falsification probe for the non-committing lookahead theorem.
///
/// Emulates an implementation that runs the committed input-preprocessing
/// lifecycle over every scalar the maximum-match walk examines, before the
/// match is known. The observable damage is diagnostic *reordering*.
fn committing_lookahead_diagnostic_order(text: &str, ampersand: usize) -> Vec<DiagnosticKind> {
    let after = ampersand + 1;
    let rest = &text[after..];
    let lookahead = named_lookahead(rest);
    let mut order = Vec::new();
    for scalar in rest[..lookahead.examined_len].chars() {
        if let Some(kind) = preprocessing_diagnostic_kind(scalar) {
            order.push(kind);
        }
    }
    if let Some(entry) = lookahead.matched
        && !entry.name.ends_with(';')
    {
        order.push(DiagnosticKind::MissingSemicolonAfterNamedReference);
    }
    order
}

/// A coarse "every RCDATA character reference is unsupported" claim.
///
/// True only if no selected named-reference cell can complete.
fn coarse_all_rcdata_character_references_unsupported(observations: &[&Observation]) -> bool {
    observations
        .iter()
        .all(|observation| observation.completion != Completion::Complete)
}

fn semantic_projection(
    observation: &Observation,
) -> (Vec<String>, Terminal, LexState, InsertionMode) {
    (
        observation
            .tree
            .titles
            .iter()
            .map(|title| title.text.clone())
            .collect(),
        observation.terminal,
        observation.tokenizer_state,
        observation.tree.mode,
    )
}

fn source(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

fn gold_evidence(start: usize, end: usize, raw: &'static str) -> GoldEvidence {
    GoldEvidence { start, end, raw }
}

fn raw_run(start: usize, end: usize, raw: &'static str) -> GoldContribution {
    GoldContribution {
        origin: ContributionOrigin::RawTextRun,
        authored: gold_evidence(start, end, raw),
        interpreted: raw,
    }
}

fn resolved(
    name: &'static str,
    start: usize,
    end: usize,
    raw: &'static str,
    interpreted: &'static str,
) -> GoldContribution {
    GoldContribution {
        origin: ContributionOrigin::ResolvedNamedReference { name },
        authored: gold_evidence(start, end, raw),
        interpreted,
    }
}

fn ampersand_run(start: usize, end: usize, raw: &'static str) -> GoldContribution {
    GoldContribution {
        origin: ContributionOrigin::UnresolvedAmpersandRun,
        authored: gold_evidence(start, end, raw),
        interpreted: raw,
    }
}

fn closed_title(
    start: GoldEvidence,
    text: &'static str,
    contributions: Vec<GoldContribution>,
    close: GoldEvidence,
) -> GoldTitle {
    GoldTitle {
        start,
        text,
        contributions,
        close: Some(close),
    }
}

fn complete_gold(titles: Vec<GoldTitle>, diagnostics: Vec<GoldDiagnostic>) -> Gold {
    Gold {
        titles,
        diagnostics,
        final_lex: LexState::Data,
        completion: Completion::Complete,
        terminal: Terminal::AppropriateCloseThenEof,
        pending_reprocess: None,
        eof_in_text_diagnostic: false,
        sentinel: None,
    }
}

// ---------------------------------------------------------------------------
// RCDATA lifecycle cells
// ---------------------------------------------------------------------------

#[test]
fn r1_empty_title_proves_complete_feedback_round_trip() {
    let fixture = source(1, "<title></title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "",
                Vec::new(),
                gold_evidence(7, 15, "</title>"),
            )],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
    assert_core_causal_events(&actual);
    assert_eq!(actual.tokenizer_cursor, 15);
    assert_eq!(actual.tokenizer_coverage_end, 15);
    assert!(actual.character_reference_entries.is_empty());
    assert!(actual.lookaheads.is_empty());
}

#[test]
fn r2_tag_shaped_rcdata_remains_text_and_falsifies_a_completed_data_vector() {
    let fixture = source(2, "<title><b>x</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "<b>x",
                vec![raw_run(7, 11, "<b>x")],
                gold_evidence(11, 19, "</title>"),
            )],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));

    // An eager all-Data history would have produced a separate `<b>` token
    // boundary at [7, 10). The coordinated history has one RCDATA run at
    // [7, 11), so repairing the eager history requires retokenization, not a
    // downstream label change.
    let eager = eager_all_data_classes(&fixture, 0);
    assert!(eager.contains(&TokenClass::OtherData));
    assert!(
        !produced_classes(&actual)
            .iter()
            .any(|(_, class)| *class == TokenClass::OtherData)
    );
    assert_eq!(actual.tree.next_id, 3, "no element identity for `<b>`");
}

#[test]
fn r3_non_appropriate_end_tag_candidate_remains_text() {
    let fixture = source(3, "<title>x</titler>y</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "x</titler>y",
                vec![raw_run(7, 18, "x</titler>y")],
                gold_evidence(18, 26, "</title>"),
            )],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
}

#[test]
fn r4_mixed_case_appropriate_close_preserves_authored_raw_spelling() {
    let fixture = source(4, "<title>x</TiTlE>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "x",
                vec![raw_run(7, 8, "x")],
                gold_evidence(8, 16, "</TiTlE>"),
            )],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
}

#[test]
fn r5_full_source_sentinel_proves_feedback_is_early_and_the_round_trip_is_two_way() {
    let fixture = source(5, "<head><title>x</title><body>");
    let actual = run_candidate(&fixture, 6, None);
    assert_gold(
        &actual,
        &Gold {
            titles: vec![closed_title(
                gold_evidence(6, 13, "<title>"),
                "x",
                vec![raw_run(13, 14, "x")],
                gold_evidence(14, 22, "</title>"),
            )],
            diagnostics: Vec::new(),
            final_lex: LexState::Data,
            completion: Completion::Complete,
            terminal: Terminal::PostCloseSentinel,
            pending_reprocess: None,
            eof_in_text_diagnostic: false,
            sentinel: Some(gold_evidence(22, 28, "<body>")),
        },
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));

    assert_eq!(
        event_kinds(&actual.events),
        vec![
            EventKind::ProducedStartTitleData,
            EventKind::TitleInserted,
            EventKind::FeedbackRequested,
            EventKind::FeedbackApplied,
            EventKind::EnteredText,
            EventKind::ProducedRawTextRunRcdata,
            EventKind::TextContributed,
            EventKind::ProducedEndTitleRcdata,
            EventKind::TokenizerReturnedToData,
            EventKind::TitleClosed,
            EventKind::RestoredInHead,
            EventKind::ProducedBodyData,
            EventKind::PostCloseBodyData,
        ],
        "the selected round trip is a causal ordering theorem, not only a final-state theorem"
    );

    let late = source(55, "<title><b>x</title>");
    assert_eq!(
        late_feedback_probe(&late),
        (TokenClass::OtherData, LexState::Data),
        "producing before feedback observes the wrong lexical history"
    );
}

#[test]
fn r6_textarea_end_tag_is_not_appropriate_while_title_is_active() {
    let fixture = source(6, "<title>x</textarea>y</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "x</textarea>y",
                vec![raw_run(7, 20, "x</textarea>y")],
                gold_evidence(20, 28, "</title>"),
            )],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
    assert_eq!(
        actual.tree.next_id, 3,
        "no element identity for `</textarea>`"
    );
}

// ---------------------------------------------------------------------------
// Named character-reference cells
// ---------------------------------------------------------------------------

#[test]
fn n1_ordered_authored_contributions_survive_a_single_coalesced_text_node() {
    let fixture = source(11, "<title>a&amp;b</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "a&b",
                vec![
                    raw_run(7, 8, "a"),
                    resolved("amp;", 8, 13, "&amp;", "&"),
                    raw_run(13, 14, "b"),
                ],
                gold_evidence(14, 22, "</title>"),
            )],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));

    let title = &actual.tree.titles[0];
    assert_eq!(title.text, "a&b", "one final interpreted text node");
    assert_eq!(
        title
            .contributions
            .iter()
            .map(|contribution| (contribution.authored.start, contribution.authored.end))
            .collect::<Vec<_>>(),
        vec![(7, 8), (8, 13), (13, 14)],
        "coalescing must not destroy authored contribution locality"
    );
    // Ordinary character origin, resolved-reference origin, and authored
    // syntax are three different things.
    assert_ne!(title.contributions[0].origin, title.contributions[1].origin);
    assert_eq!(title.contributions[1].authored.raw, "&amp;");
    assert_eq!(title.contributions[1].interpreted, "&");
    assert!(
        !title
            .contributions
            .iter()
            .any(|contribution| contribution.authored.raw == "</title>"),
        "the authored close is syntax, never a text contribution"
    );
}

#[test]
fn n2_one_authored_reference_may_decode_to_multiple_unicode_scalars() {
    let fixture = source(12, "<title>&acE;</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "\u{223e}\u{0333}",
                vec![resolved("acE;", 7, 12, "&acE;", "\u{223e}\u{0333}")],
                gold_evidence(12, 20, "</title>"),
            )],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));

    let contribution = &actual.tree.titles[0].contributions[0];
    assert_eq!(
        contribution.interpreted.chars().count(),
        2,
        "the reference decodes to two scalars"
    );
    assert_eq!(
        actual.tree.titles[0].contributions.len(),
        1,
        "and retains exactly one authored origin"
    );
    assert_eq!(
        tiny_whitelist_lookup("acE;</title>"),
        None,
        "a tiny hard-coded whitelist cannot represent this cell"
    );
}

#[test]
fn n3_semicolonless_maximum_match_leaves_the_remainder_ordinary() {
    let fixture = source(13, "<title>&notit;</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "\u{00ac}it;",
                vec![
                    resolved("not", 7, 11, "&not", "\u{00ac}"),
                    raw_run(11, 14, "it;"),
                ],
                gold_evidence(14, 22, "</title>"),
            )],
            vec![GoldDiagnostic {
                kind: DiagnosticKind::MissingSemicolonAfterNamedReference,
                at: gold_evidence(7, 11, "&not"),
            }],
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));

    assert_eq!(
        exact_whole_string_lookup("notit;</title>"),
        None,
        "exact whole-string lookup cannot resolve this cell"
    );
    assert_eq!(
        named_lookahead("notit;</title>").matched.map(|e| e.name),
        Some("not"),
        "maximum matching resolves the semicolonless identifier"
    );
    assert_eq!(
        tiny_whitelist_lookup("notit;</title>"),
        None,
        "a tiny hard-coded whitelist cannot represent this cell"
    );
}

#[test]
fn n4_a_longer_exact_named_reference_wins_the_maximum_match() {
    let fixture = source(14, "<title>&notin;</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "\u{2209}",
                vec![resolved("notin;", 7, 14, "&notin;", "\u{2209}")],
                gold_evidence(14, 22, "</title>"),
            )],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));

    // The required contrast: the same three leading scalars, two different
    // maximum matches.
    assert_eq!(
        named_lookahead("notin;</title>").matched.map(|e| e.name),
        Some("notin;")
    );
    assert_eq!(
        named_lookahead("notit;</title>").matched.map(|e| e.name),
        Some("not")
    );
    assert_eq!(
        tiny_whitelist_lookup("notin;</title>"),
        None,
        "a tiny hard-coded whitelist cannot represent this cell"
    );
}

#[test]
fn n5_unresolved_ampersand_run_is_neither_resolved_output_nor_authored_syntax() {
    let fixture = source(15, "<title>&bogus;</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "&bogus;",
                vec![ampersand_run(7, 13, "&bogus"), raw_run(13, 14, ";")],
                gold_evidence(14, 22, "</title>"),
            )],
            vec![GoldDiagnostic {
                kind: DiagnosticKind::UnknownNamedCharacterReference,
                at: gold_evidence(13, 14, ";"),
            }],
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));

    let contributions = &actual.tree.titles[0].contributions;
    assert_eq!(
        contributions[0].origin,
        ContributionOrigin::UnresolvedAmpersandRun
    );
    assert_eq!(contributions[0].interpreted, contributions[0].authored.raw);
    assert!(
        !contributions.iter().any(|contribution| matches!(
            contribution.origin,
            ContributionOrigin::ResolvedNamedReference { .. }
        )),
        "an unresolved ampersand must never be recorded as a resolved reference"
    );
    assert_eq!(
        actual.character_reference_entries.len(),
        1,
        "the character reference state was entered exactly once"
    );
}

#[test]
fn n6_decoded_end_tag_spelling_is_output_and_never_an_authored_close() {
    let fixture = source(16, "<title>&lt;/title></title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "</title>",
                vec![
                    resolved("lt;", 7, 11, "&lt;", "<"),
                    raw_run(11, 18, "/title>"),
                ],
                gold_evidence(18, 26, "</title>"),
            )],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));

    let title = &actual.tree.titles[0];
    assert_eq!(
        title.text, "</title>",
        "the interpreted text spells an end tag"
    );
    let close = title.close.as_ref().expect("authored close");
    assert_eq!(
        (close.start, close.end),
        (18, 26),
        "the authored close is the last source `</title>`, not the decoded text"
    );
    assert_eq!(
        title.contributions[0].authored.raw, "&lt;",
        "the decoded `<` retains its authored reference spelling"
    );
    assert_eq!(
        produced_classes(&actual)
            .iter()
            .filter(|(_, class)| *class == TokenClass::EndTitle)
            .count(),
        1,
        "decoded output is never re-tokenized into a second end tag"
    );
}

#[test]
fn n7_recursive_entity_decoding_does_not_occur() {
    let fixture = source(17, "<title>&amp;lt;</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "&lt;",
                vec![
                    resolved("amp;", 7, 12, "&amp;", "&"),
                    raw_run(12, 15, "lt;"),
                ],
                gold_evidence(15, 23, "</title>"),
            )],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
    assert_eq!(
        actual.tree.titles[0].text, "&lt;",
        "the decoded `&` must not be reintroduced as tokenizer input"
    );
    assert_eq!(
        actual.character_reference_entries.len(),
        1,
        "exactly one authored reference was entered"
    );
}

// ---------------------------------------------------------------------------
// Non-committing lookahead
// ---------------------------------------------------------------------------

#[test]
fn l1_speculative_maximum_match_cannot_reorder_a_later_preprocessing_diagnostic() {
    let fixture = source(18, "<title>&not\u{0001}</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "\u{00ac}\u{0001}",
                vec![
                    resolved("not", 7, 11, "&not", "\u{00ac}"),
                    raw_run(11, 12, "\u{0001}"),
                ],
                gold_evidence(12, 20, "</title>"),
            )],
            vec![
                GoldDiagnostic {
                    kind: DiagnosticKind::MissingSemicolonAfterNamedReference,
                    at: gold_evidence(7, 11, "&not"),
                },
                GoldDiagnostic {
                    kind: DiagnosticKind::ControlCharacterInInputStream,
                    at: gold_evidence(11, 12, "\u{0001}"),
                },
            ],
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));

    // Discovery had to look at the control scalar to know `not` was maximal.
    let lookahead = actual
        .lookaheads
        .first()
        .expect("one maximum-match discovery");
    assert_eq!(lookahead.at, 8);
    assert_eq!(lookahead.matched, Some("not"));
    assert_eq!(
        lookahead.examined_end, 12,
        "the walk examined the scalar that ended the match"
    );

    // It nevertheless committed nothing.
    assert_eq!(lookahead.cursor_before, lookahead.cursor_after);
    assert_eq!(lookahead.coverage_before, lookahead.coverage_after);
    assert_eq!(lookahead.diagnostics_before, lookahead.diagnostics_after);
    assert_eq!(
        lookahead.cursor_before, 7,
        "speculative discovery never advanced the authoritative cursor"
    );
    assert_eq!(
        lookahead.coverage_before, 7,
        "speculative discovery never advanced coverage"
    );

    assert_eq!(
        diagnostic_kinds(&actual),
        vec![
            DiagnosticKind::MissingSemicolonAfterNamedReference,
            DiagnosticKind::ControlCharacterInInputStream,
        ]
    );
    assert_eq!(
        committing_lookahead_diagnostic_order(fixture.as_str(), 7),
        vec![
            DiagnosticKind::ControlCharacterInInputStream,
            DiagnosticKind::MissingSemicolonAfterNamedReference,
        ],
        "a committing lookahead reorders the later diagnostic before the reference"
    );
    assert_ne!(
        diagnostic_kinds(&actual),
        committing_lookahead_diagnostic_order(fixture.as_str(), 7)
    );
    assert_eq!(actual.tokenizer_coverage_end, 20);
}

// ---------------------------------------------------------------------------
// EOF cells
// ---------------------------------------------------------------------------

#[test]
fn e1_eof_after_a_bare_ampersand_restores_the_tree_without_fabricating_a_close() {
    let fixture = source(21, "<title>&");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &Gold {
            titles: vec![GoldTitle {
                start: gold_evidence(0, 7, "<title>"),
                text: "&",
                contributions: vec![ampersand_run(7, 8, "&")],
                close: None,
            }],
            diagnostics: vec![GoldDiagnostic {
                kind: DiagnosticKind::EofInTitleText,
                at: gold_evidence(8, 8, ""),
            }],
            final_lex: LexState::Rcdata,
            completion: Completion::PendingSameTokenReprocess,
            terminal: Terminal::EofInRcdataReprocessPending,
            pending_reprocess: Some((LexState::Rcdata, TokenClass::Eof)),
            eof_in_text_diagnostic: true,
            sentinel: None,
        },
    );
    assert_ne!(actual.completion, Completion::Complete);
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));

    let pending = actual
        .pending_reprocess
        .as_ref()
        .expect("the same EOF token remains pending for reprocess");
    assert_evidence(&pending.evidence, fixture.id(), &gold_evidence(8, 8, ""));
    assert!(actual.events.iter().any(|event| matches!(
        event,
        Event::SameTokenReprocessPending {
            class: TokenClass::Eof,
            state: LexState::Rcdata,
            evidence,
        } if evidence.start == 8 && evidence.end == 8 && evidence.raw.is_empty()
    )));
    assert!(
        !actual
            .events
            .iter()
            .any(|event| matches!(event, Event::TokenizerReturnedToData { .. })),
        "EOF recovery must not claim the tokenizer returned to Data"
    );
}

#[test]
fn e2_eof_after_a_semicolonless_match_keeps_the_reference_and_the_checkpoint() {
    let fixture = source(22, "<title>&amp");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &Gold {
            titles: vec![GoldTitle {
                start: gold_evidence(0, 7, "<title>"),
                text: "&",
                contributions: vec![resolved("amp", 7, 11, "&amp", "&")],
                close: None,
            }],
            diagnostics: vec![
                GoldDiagnostic {
                    kind: DiagnosticKind::MissingSemicolonAfterNamedReference,
                    at: gold_evidence(7, 11, "&amp"),
                },
                GoldDiagnostic {
                    kind: DiagnosticKind::EofInTitleText,
                    at: gold_evidence(11, 11, ""),
                },
            ],
            final_lex: LexState::Rcdata,
            completion: Completion::PendingSameTokenReprocess,
            terminal: Terminal::EofInRcdataReprocessPending,
            pending_reprocess: Some((LexState::Rcdata, TokenClass::Eof)),
            eof_in_text_diagnostic: true,
            sentinel: None,
        },
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
    assert_eq!(
        actual.tree.titles[0].contributions[0].interpreted, "&",
        "the resolved value differs from the authored spelling"
    );
    assert_eq!(actual.tree.titles[0].contributions[0].authored.raw, "&amp");
}

// ---------------------------------------------------------------------------
// Repeated lifecycle
// ---------------------------------------------------------------------------

#[test]
fn p1_repeated_title_lifecycles_reuse_the_same_round_trip() {
    let fixture = source(31, "<title>a</title><title>b</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![
                closed_title(
                    gold_evidence(0, 7, "<title>"),
                    "a",
                    vec![raw_run(7, 8, "a")],
                    gold_evidence(8, 16, "</title>"),
                ),
                closed_title(
                    gold_evidence(16, 23, "<title>"),
                    "b",
                    vec![raw_run(23, 24, "b")],
                    gold_evidence(24, 32, "</title>"),
                ),
            ],
            Vec::new(),
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
    assert_eq!(actual.tree.next_id, 4);
    assert_eq!(
        actual
            .events
            .iter()
            .filter(|event| matches!(event, Event::FeedbackApplied { .. }))
            .count(),
        2,
        "each Title lifecycle requests and applies its own feedback"
    );
    assert_eq!(actual.tokenizer_state, LexState::Data);
}

// ---------------------------------------------------------------------------
// Negative and control cells
// ---------------------------------------------------------------------------

#[test]
fn x1_numeric_branch_is_reached_and_deferred_without_a_false_coarse_claim() {
    let numeric_source = source(41, "<title>&#60;</title>");
    let numeric = run_candidate(&numeric_source, 0, None);

    assert_eq!(numeric.completion, Completion::Unsupported);
    assert_eq!(
        numeric.terminal,
        Terminal::UnsupportedNumericCharacterReference
    );
    assert_eq!(
        numeric.unsupported_requirement,
        Some(UnsupportedRequirement::NumericCharacterReferenceInRcdata),
        "the single remaining requirement is specifically the Numeric branch"
    );
    assert_eq!(validate_candidate_freeze(&numeric), Ok(()));

    // Title entry succeeded.
    assert_eq!(numeric.tree.titles.len(), 1);
    assert_evidence(
        &numeric.tree.titles[0].start,
        numeric.source_id,
        &gold_evidence(0, 7, "<title>"),
    );
    // RCDATA entry succeeded.
    assert_eq!(numeric.tokenizer_state, LexState::Rcdata);
    assert_eq!(numeric.tree.mode, InsertionMode::Text);
    assert_eq!(numeric.tree.original_mode, Some(InsertionMode::InHead));
    // Character reference entry was reached.
    assert_eq!(numeric.character_reference_entries.len(), 1);
    assert_evidence(
        &numeric.character_reference_entries[0],
        numeric.source_id,
        &gold_evidence(7, 8, "&"),
    );
    assert!(
        numeric
            .events
            .iter()
            .any(|event| matches!(event, Event::CharacterReferenceEntered { .. }))
    );
    // Nothing beyond the ampersand was committed.
    assert_eq!(numeric.tokenizer_coverage_end, 7);
    assert!(numeric.tree.titles[0].contributions.is_empty());
    assert!(
        numeric.lookaheads.is_empty(),
        "no named match was attempted"
    );

    // A coarse "all RCDATA character references are unsupported" claim would
    // be false: selected named references complete on the same machine.
    let amp = run_candidate(&source(42, "<title>&amp;</title>"), 0, None);
    let ace = run_candidate(&source(43, "<title>&acE;</title>"), 0, None);
    let notin = run_candidate(&source(44, "<title>&notin;</title>"), 0, None);
    assert_eq!(amp.completion, Completion::Complete);
    assert_eq!(ace.completion, Completion::Complete);
    assert_eq!(notin.completion, Completion::Complete);
    assert!(
        !coarse_all_rcdata_character_references_unsupported(&[&numeric, &amp, &ace, &notin]),
        "the durable distinction must be narrow enough to name only Numeric"
    );
    assert!(coarse_all_rcdata_character_references_unsupported(&[
        &numeric
    ]));
}

#[test]
fn x2_attributed_title_start_refuses_transactionally() {
    let fixture = source(45, "<title id=x>x</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_unsupported_shape_refusal(
        &actual,
        UnsupportedRequirement::NonSelectedTitleStartShape,
        12,
    );
}

#[test]
fn x3_self_closing_title_shape_refuses_transactionally() {
    let fixture = source(46, "<title/>x");
    let actual = run_candidate(&fixture, 0, None);
    assert_unsupported_shape_refusal(
        &actual,
        UnsupportedRequirement::NonSelectedTitleStartShape,
        8,
    );
}

#[test]
fn x4_title_outside_the_selected_in_head_context_refuses_transactionally() {
    let fixture = source(47, "<body><title>x</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_unsupported_shape_refusal(
        &actual,
        UnsupportedRequirement::TitleOutsideSelectedInHeadContext,
        6,
    );
}

#[test]
fn x5_null_in_rcdata_is_recovered_output_not_a_resolved_reference() {
    let fixture = source(48, "<title>\0</title>");
    let actual = run_candidate(&fixture, 0, None);
    assert_gold(
        &actual,
        &complete_gold(
            vec![closed_title(
                gold_evidence(0, 7, "<title>"),
                "\u{fffd}",
                vec![GoldContribution {
                    origin: ContributionOrigin::NullReplacement,
                    authored: gold_evidence(7, 8, "\0"),
                    interpreted: "\u{fffd}",
                }],
                gold_evidence(8, 16, "</title>"),
            )],
            vec![GoldDiagnostic {
                kind: DiagnosticKind::UnexpectedNullCharacter,
                at: gold_evidence(7, 8, "\0"),
            }],
        ),
    );
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
    assert!(
        actual.character_reference_entries.is_empty(),
        "NUL recovery is not character-reference decoding"
    );
}

#[test]
fn x6_non_title_rcdata_element_is_not_authorized_by_the_selected_candidate() {
    let fixture = source(49, "<textarea>x");
    let actual = run_candidate(&fixture, 0, None);
    assert_unsupported_shape_refusal(&actual, UnsupportedRequirement::NonTitleRcdataElement, 10);
    assert!(
        !actual
            .events
            .iter()
            .any(|event| matches!(event, Event::FeedbackRequested { .. })),
        "a durable RCDATA vocabulary does not authorize coordination for every RCDATA element"
    );
}

fn assert_unsupported_shape_refusal(
    observation: &Observation,
    requirement: UnsupportedRequirement,
    cursor: usize,
) {
    assert_eq!(observation.completion, Completion::Unsupported);
    assert_eq!(observation.terminal, Terminal::UnsupportedShape);
    assert_eq!(observation.unsupported_requirement, Some(requirement));
    assert_eq!(
        observation.tree,
        TreeState::candidate_prestate(),
        "an excluded shape mutates no candidate state and allocates no identity"
    );
    assert_eq!(observation.tokenizer_state, LexState::Data);
    assert_eq!(observation.tokenizer_cursor, cursor);
    assert_eq!(observation.pending_feedback, None);
    assert_eq!(observation.pending_reprocess, None);
    assert!(observation.diagnostics.is_empty());
    assert!(observation.character_reference_entries.is_empty());
    assert!(!observation.events.iter().any(|event| matches!(
        event,
        Event::FeedbackRequested { .. }
            | Event::FeedbackApplied { .. }
            | Event::TitleInserted { .. }
    )));
    assert_eq!(validate_candidate_freeze(observation), Ok(()));
}

// ---------------------------------------------------------------------------
// Identity, determinism, incompleteness, and freeze corruption
// ---------------------------------------------------------------------------

#[test]
fn c1_source_id_perturbation_changes_provenance_not_semantics() {
    let first = source(80, "<title>a&amp;b</title>");
    let second = source(81, "<title>a&amp;b</title>");
    let first_observation = run_candidate(&first, 0, None);
    let second_observation = run_candidate(&second, 0, None);

    assert_eq!(
        semantic_projection(&first_observation),
        semantic_projection(&second_observation)
    );
    assert_ne!(first_observation.source_id, second_observation.source_id);
    assert_ne!(
        first_observation.tree.titles[0].start.source_id,
        second_observation.tree.titles[0].start.source_id
    );
    assert_ne!(
        first_observation.tree.titles[0].contributions[1]
            .authored
            .source_id,
        second_observation.tree.titles[0].contributions[1]
            .authored
            .source_id
    );
}

#[test]
fn c2_repeated_runs_are_deterministic() {
    let fixture = source(82, "<title>&notit;</title><title>&bogus;</title>");
    assert_eq!(
        run_candidate(&fixture, 0, None),
        run_candidate(&fixture, 0, None)
    );
}

#[test]
fn c3_lower_layer_stop_is_never_upgraded_to_complete() {
    let fixture = source(83, "<title>a&amp;b</title>");
    let actual = run_candidate(&fixture, 0, Some(7));
    assert_eq!(actual.completion, Completion::LowerLayerIncomplete);
    assert_eq!(actual.terminal, Terminal::LowerLayerStop);
    assert_eq!(actual.tokenizer_state, LexState::Rcdata);
    assert_eq!(actual.tokenizer_cursor, 7);
    assert_eq!(actual.tokenizer_coverage_end, 7);
    assert_eq!(actual.tree.mode, InsertionMode::Text);
    assert_eq!(actual.tree.original_mode, Some(InsertionMode::InHead));
    assert_eq!(actual.pending_reprocess, None);
    assert!(actual.tree.titles[0].contributions.is_empty());
    assert!(actual.tree.titles[0].close.is_none());
    assert!(!actual.tree.titles[0].eof_closed);
    assert_eq!(validate_candidate_freeze(&actual), Ok(()));
}

#[test]
fn c4_freeze_rejects_impossible_terminal_states() {
    let fixture = source(84, "<title>a&amp;b</title>");
    let valid = run_candidate(&fixture, 0, None);
    assert_eq!(validate_candidate_freeze(&valid), Ok(()));

    let mut pending = valid.clone();
    pending.pending_feedback = Some(Feedback::EnterRcdataForTitle);
    assert_eq!(
        validate_candidate_freeze(&pending),
        Err(FreezeError::PendingFeedback)
    );

    let mut wrong_lex = valid.clone();
    wrong_lex.tokenizer_state = LexState::Rcdata;
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

    let mut still_open = valid.clone();
    let title_id = still_open.tree.titles[0].id;
    still_open.tree.open.push(title_id);
    assert_eq!(
        validate_candidate_freeze(&still_open),
        Err(FreezeError::ClosedPathTitleStillOpen)
    );

    // A non-overlapping but wrong close: it still contradicts the emitted
    // end-tag evidence.
    let mut mismatched_close = valid.clone();
    mismatched_close.tree.titles[0].close = Some(evidence(&fixture, 14, 21));
    assert_eq!(
        validate_candidate_freeze(&mismatched_close),
        Err(FreezeError::ClosedPathCloseEvidenceMismatch)
    );

    let mut impossible_identity = valid.clone();
    impossible_identity.tree.elements.push(Element {
        id: NodeId(3),
        name: Name::Title,
        parent: Some(NodeId(1)),
        origin: Origin::Authored(evidence(&fixture, 0, 7)),
    });
    impossible_identity.tree.next_id = 4;
    assert_eq!(
        validate_candidate_freeze(&impossible_identity),
        Err(FreezeError::ClosedPathImpossibleConstructedIdentity)
    );

    let mut fabricated = valid.clone();
    fabricated.tree.titles[0].contributions[0].authored.end = 9;
    assert_eq!(
        validate_candidate_freeze(&fabricated),
        Err(FreezeError::FabricatedContributionEvidence)
    );

    let mut overlapping = valid.clone();
    overlapping.tree.titles[0].contributions[2].authored = evidence(&fixture, 12, 13);
    assert_eq!(
        validate_candidate_freeze(&overlapping),
        Err(FreezeError::OverlappingContributionEvidence)
    );

    let mut relabelled = valid.clone();
    relabelled.tree.titles[0].contributions[1].origin = ContributionOrigin::RawTextRun;
    assert_eq!(
        validate_candidate_freeze(&relabelled),
        Err(FreezeError::ContributionOriginContradictsInterpretation)
    );

    let mut fake_reference = valid.clone();
    fake_reference.tree.titles[0].contributions[0].origin =
        ContributionOrigin::ResolvedNamedReference { name: "amp;" };
    assert_eq!(
        validate_candidate_freeze(&fake_reference),
        Err(FreezeError::ContributionOriginContradictsInterpretation)
    );

    let mut broken_coalescing = valid.clone();
    broken_coalescing.tree.titles[0].text = "a<b".to_owned();
    assert_eq!(
        validate_candidate_freeze(&broken_coalescing),
        Err(FreezeError::CoalescedTextContradictsContributions)
    );

    let eof_source = source(85, "<title>&amp");
    let mut false_complete = run_candidate(&eof_source, 0, None);
    false_complete.completion = Completion::Complete;
    assert_eq!(
        validate_candidate_freeze(&false_complete),
        Err(FreezeError::OutstandingSameTokenReprocess)
    );

    let mut fake_close = run_candidate(&eof_source, 0, None);
    fake_close.tree.titles[0].close = Some(evidence(&eof_source, 11, 11));
    assert_eq!(
        validate_candidate_freeze(&fake_close),
        Err(FreezeError::EofPathClaimsAuthoredClose)
    );
}

#[test]
fn c5_every_retained_range_is_real_and_never_fabricated() {
    for text in [
        "<title></title>",
        "<title>a&amp;b</title>",
        "<title>&acE;</title>",
        "<title>&notit;</title>",
        "<title>&bogus;</title>",
        "<title>&lt;/title></title>",
        "<title>&amp;lt;</title>",
        "<title>&not\u{0001}</title>",
        "<title>\0</title>",
        "<title>a</title><title>b</title>",
        "<title>&",
        "<title>&amp",
    ] {
        let fixture = source(90, text);
        let observation = run_candidate(&fixture, 0, None);
        let mut checked = 0usize;
        for title in &observation.tree.titles {
            let mut spans = vec![&title.start];
            spans.extend(title.contributions.iter().map(|c| &c.authored));
            if let Some(close) = &title.close {
                spans.push(close);
            }
            for span in spans {
                assert_eq!(span.source_id, fixture.id());
                assert_eq!(
                    span.raw,
                    &fixture.as_str()[span.start..span.end],
                    "retained evidence must quote the real source"
                );
                checked += 1;
            }
        }
        for diagnostic in &observation.diagnostics {
            let span = &diagnostic.evidence;
            assert_eq!(span.raw, &fixture.as_str()[span.start..span.end]);
        }
        assert!(checked > 0, "every cell retains at least the Title start");
    }
}

#[test]
fn c6_the_tree_to_tokenizer_control_is_title_specific() {
    // The only feedback this candidate can request is the Title-specific
    // RCDATA entry. It carries no tokenizer-mode operand, so it cannot stand
    // in for a generic mode-switch control.
    let mut tree = TreeState::candidate_prestate();
    let fixture = source(91, "<title>x</title>");
    assert_eq!(
        tree.insert_title(evidence(&fixture, 0, 7)),
        Feedback::EnterRcdataForTitle
    );

    // The other selected-RCDATA element in the predecessor tokenizer GOLD
    // (`textarea`) produces no feedback at all.
    let textarea = run_candidate(&source(92, "<textarea>x"), 0, None);
    assert!(
        !textarea
            .events
            .iter()
            .any(|event| matches!(event, Event::FeedbackApplied { .. }))
    );
    assert_eq!(textarea.tokenizer_state, LexState::Data);
}

#[test]
fn c7_gold_is_hand_authored_and_external_heads_are_freshness_markers_only() {
    for pin in [
        FRESH_WHATWG_HEAD,
        PINNED_WHATWG_SOURCE_BLOB,
        FRESH_WPT_HEAD,
        FRESH_HTML5LIB_TESTS_HEAD,
    ] {
        assert_eq!(pin.len(), 40);
        assert!(pin.chars().all(|c| c.is_ascii_hexdigit()));
    }
    assert_ne!(FRESH_WHATWG_HEAD, PINNED_WHATWG_SOURCE_BLOB);
    assert_ne!(FRESH_WPT_HEAD, FRESH_HTML5LIB_TESTS_HEAD);

    let fixture = source(93, "<title>&notin;</title>");
    let actual = run_candidate(&fixture, 0, None);
    let hand_authored = complete_gold(
        vec![closed_title(
            gold_evidence(0, 7, "<title>"),
            "\u{2209}",
            vec![resolved("notin;", 7, 14, "&notin;", "\u{2209}")],
            gold_evidence(14, 22, "</title>"),
        )],
        Vec::new(),
    );
    assert_gold(&actual, &hand_authored);
}
