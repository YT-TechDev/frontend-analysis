//! Candidate-independent validation for Issue #874.
//!
//! This module validates the selected Data-state Named Character Reference
//! successor semantics before any production placement or implementation. It
//! is **test-only** and changes no production behavior.
//!
//! # Independent oracle boundary
//!
//! This module imports nothing from the production HTML tokenizer, the
//! tree-construction driver, session, or result, the Named Character Reference
//! production matcher, the CLI, or any browser, WPT, or html5lib output. Every
//! expected value below is hand-authored from the pinned WHATWG obligations
//! recorded by the Issue #348 post-v0.1.1 checkpoint. The only crate items used
//! are the `SourceText` / `SourceId` anchoring primitives, so that authored
//! evidence ranges are real.
//!
//! Pins recorded by Issue #348 and carried here as freshness markers only:
//! WHATWG HTML `5facb53a8b013f209aa789ea0726e404f12c83d9`, source blob
//! `96420094e52cadc7ceea3c630b246d200b534527`; WPT challenge head
//! `588420c752cb4988fb2fc193583637a4b6452406`; html5lib-tests challenge head
//! `c777c408b61078ea2eb4acefc2535f54dbc8b28a`. WPT and html5lib are challenge
//! evidence, never the oracle.
//!
//! # Pinned obligations the gold is derived from
//!
//! - Data `&` sets the return state to **Data** and switches to the Character
//!   Reference state. RCDATA `&` sets the return state to RCDATA. The two share
//!   Character Reference / Named semantics but not return-state ownership;
//!   this model owns only the Data return state.
//! - Character Reference: ASCII alphanumeric reconsumes in Named; `#` selects
//!   Numeric; anything else (including EOF) flushes the `&` and reconsumes in
//!   the return state.
//! - Named: consume the *maximum* number of characters that form a name from
//!   the table. A match not ending in `;` records
//!   `missing-semicolon-after-character-reference`. The historical exception
//!   for a following ASCII alphanumeric or `=` applies **only when consumed as
//!   part of an attribute**, so it must not apply in Data. No match flushes the
//!   `&` and enters the ambiguous ampersand state.
//! - Ambiguous ampersand: ASCII alphanumerics are emitted; `;` records
//!   `unknown-named-character-reference` and is reconsumed in the return state;
//!   anything else (including EOF) reconsumes in the return state.
//! - Decoded characters are output only. They never re-enter tokenizer input,
//!   so there is no recursive decoding and a decoded `</body>` is character
//!   data, not an authored end tag.
//! - One authored reference may decode to more than one Unicode scalar.
//! - After Body, whitespace is processed with the in-body rules and stays in
//!   After Body; any other character is a parse error that switches to In Body
//!   and reprocesses it. `&nbsp;` decodes to U+00A0, which is not HTML
//!   whitespace; `&Tab;` and `&NewLine;` decode to HTML whitespace.
//!
//! # Deliberately bounded model
//!
//! The model is a Data-only tokenizer plus a four-position tree
//! ([`Mode`]). It recognises only the exact tags `<body>`, `</body>`, and
//! `</html>`; everything else is refused as an explicit boundary. It is not a
//! second HTML parser, a proposed production tokenizer state layout, cursor
//! API, diagnostic enum, anchor encoding, resource representation, or
//! coordinator contract, and it selects no production type or variant
//! spelling for "Numeric Character Reference in Data".
//!
//! Character-reference diagnostics carry a test-local *site* and no raw range,
//! because Issue #874 does not freeze durable diagnostic anchors.
//!
//! # Named data
//!
//! [`NAMED_REFERENCES`] is a small hand-authored **test-local** subset of the
//! WHATWG table. It is faithful for every authored cell here: for each, no
//! omitted WHATWG name is a prefix of the relevant remaining input. It is not
//! a production lookup table and never calls the production matcher.
//!
//! # Separate existing defect
//!
//! Data U+0000 handling is a separate existing project defect recorded by
//! #348 and is neither corrected nor absorbed here. The model refuses Data NUL
//! as an outside-candidate input and makes no claim about its output.

use crate::{SourceId, SourceText};

const WHATWG_HEAD: &str = "5facb53a8b013f209aa789ea0726e404f12c83d9";
const WHATWG_SOURCE_BLOB: &str = "96420094e52cadc7ceea3c630b246d200b534527";
const WPT_CHALLENGE_HEAD: &str = "588420c752cb4988fb2fc193583637a4b6452406";
const HTML5LIB_CHALLENGE_HEAD: &str = "c777c408b61078ea2eb4acefc2535f54dbc8b28a";

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
// Test-local Named Character Reference data and maximum matching
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NamedEntry {
    name: &'static str,
    value: &'static str,
}

const fn entry(name: &'static str, value: &'static str) -> NamedEntry {
    NamedEntry { name, value }
}

const NAMED_REFERENCES: &[NamedEntry] = &[
    entry("NewLine;", "\u{000a}"),
    entry("Not;", "\u{2aec}"),
    entry("NotEqual;", "\u{2260}"),
    entry("NotEqualTilde;", "\u{2242}\u{0338}"),
    entry("Tab;", "\u{0009}"),
    entry("amp", "\u{0026}"),
    entry("amp;", "\u{0026}"),
    entry("gt", "\u{003e}"),
    entry("gt;", "\u{003e}"),
    entry("lt", "\u{003c}"),
    entry("lt;", "\u{003c}"),
    entry("nbsp", "\u{00a0}"),
    entry("nbsp;", "\u{00a0}"),
    entry("not", "\u{00ac}"),
    entry("not;", "\u{00ac}"),
    entry("notin;", "\u{2209}"),
    entry("notinva;", "\u{2209}"),
    entry("notni;", "\u{220c}"),
    entry("notniva;", "\u{220c}"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NamedLookahead {
    matched: Option<NamedEntry>,
    /// Byte length from the first identifier scalar that the prefix walk had
    /// to examine before the maximum match was known final. The failing scalar
    /// is included: it is exactly the scalar a committing implementation would
    /// wrongly preprocess early.
    examined_len: usize,
}

/// Maximum-match discovery: a pure forward prefix walk over the remaining
/// input. No cursor exists here, nothing is searched after recognition, and no
/// produced output is reinterpreted.
fn named_lookahead(rest: &str) -> NamedLookahead {
    let mut examined_len = 0usize;
    let mut walked = 0usize;
    for scalar in rest.chars() {
        let next = walked + scalar.len_utf8();
        examined_len = next;
        if !NAMED_REFERENCES
            .iter()
            .any(|candidate| candidate.name.starts_with(&rest[..next]))
        {
            break;
        }
        walked = next;
    }
    let matched = NAMED_REFERENCES
        .iter()
        .filter(|candidate| rest.starts_with(candidate.name))
        .max_by_key(|candidate| candidate.name.len())
        .copied();
    NamedLookahead {
        matched,
        examined_len,
    }
}

// ---------------------------------------------------------------------------
// Falsification probes: deliberately wrong designs the gold must reject
// ---------------------------------------------------------------------------

/// Probe: recognise only a complete `name;` run.
fn exact_whole_string_lookup(rest: &str) -> Option<NamedEntry> {
    let semicolon = rest.find(';')?;
    let whole = &rest[..=semicolon];
    NAMED_REFERENCES
        .iter()
        .find(|candidate| candidate.name == whole)
        .copied()
}

/// Probe: a tiny hard-coded whitelist.
fn tiny_whitelist_lookup(rest: &str) -> Option<NamedEntry> {
    const WHITELIST: &[&str] = &["amp;", "lt;", "gt;"];
    NAMED_REFERENCES
        .iter()
        .find(|candidate| WHITELIST.contains(&candidate.name) && rest.starts_with(candidate.name))
        .copied()
}

/// Probe: the AttributeValue historical exception applied to Data. A
/// semicolonless match followed by an ASCII alphanumeric or `=` is flushed
/// unresolved.
fn attribute_exception_lookup(rest: &str) -> Option<NamedEntry> {
    let matched = named_lookahead(rest).matched?;
    if !matched.name.ends_with(';') {
        let after = rest[matched.name.len()..].chars().next();
        if after.is_some_and(|scalar| scalar.is_ascii_alphanumeric() || scalar == '=') {
            return None;
        }
    }
    Some(matched)
}

/// Probe: recursive decoding. Re-decodes decoded output until it is stable.
fn recursive_decode(text: &str) -> String {
    let mut current = decode_once(text);
    loop {
        let next = decode_once(&current);
        if next == current {
            return current;
        }
        current = next;
    }
}

/// One pass of Data character-reference decoding, for the probe only.
fn decode_once(text: &str) -> String {
    let mut output = String::new();
    let mut index = 0usize;
    while let Some(scalar) = text[index..].chars().next() {
        let after = index + scalar.len_utf8();
        let matched = (scalar == '&')
            .then(|| named_lookahead(&text[after..]).matched)
            .flatten();
        match matched {
            Some(found) => {
                output.push_str(found.value);
                index = after + found.name.len();
            }
            None => {
                output.push(scalar);
                index = after;
            }
        }
    }
    output
}

/// Probe: decoded output re-enters the tokenizer, so decoded syntax becomes
/// authored syntax.
fn retokenized_end_body_count(text: &str) -> usize {
    let decoded = lex(text).interpreted();
    count_tokens(&lex(&decoded), |token| matches!(token, Token::EndBody(_)))
}

/// Probe: a committing lookahead preprocesses the scalar that ended the match
/// before the reference's own observation.
fn committing_lookahead_diagnostic_order(text: &str, ampersand: usize) -> Vec<DiagnosticKind> {
    let after = ampersand + 1;
    let lookahead = named_lookahead(&text[after..]);
    let mut kinds = Vec::new();
    let mut offset = after;
    for scalar in text[after..after + lookahead.examined_len].chars() {
        if control_diagnostic(scalar) {
            kinds.push(DiagnosticKind::ControlCharacterInInputStream);
        }
        offset += scalar.len_utf8();
    }
    assert!(offset >= after);
    if lookahead
        .matched
        .is_some_and(|matched| !matched.name.ends_with(';'))
    {
        kinds.push(DiagnosticKind::MissingSemicolonAfterNamedReference);
    }
    kinds
}

/// Probe: classify decoded whitespace by authored spelling instead of by the
/// decoded value.
fn spelling_based_is_whitespace(raw: &str) -> bool {
    !(raw.starts_with('&') && raw.ends_with(';'))
        && raw.chars().all(is_html_whitespace)
        && !raw.is_empty()
}

/// Probe: recover a reference's source span by decoded length.
fn span_from_decoded_len(start: usize, interpreted: &str) -> (usize, usize) {
    (start, start + interpreted.len())
}

/// Probe: recover a reference's source span by searching the source for the
/// decoded output.
fn span_from_later_search(source: &str, interpreted: &str) -> Option<(usize, usize)> {
    source
        .find(interpreted)
        .map(|start| (start, start + interpreted.len()))
}

/// Probe: what accidental Numeric support would decode.
fn accidental_numeric_decode(digits: &str) -> Option<char> {
    digits.parse::<u32>().ok().and_then(char::from_u32)
}

/// Probe: a design that treats every Data character reference as deferred.
fn coarse_every_ampersand_is_deferred(text: &str) -> bool {
    text.contains('&')
}

fn is_html_whitespace(scalar: char) -> bool {
    matches!(
        scalar,
        '\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' | '\u{0020}'
    )
}

// ---------------------------------------------------------------------------
// Diagnostics (test-local vocabulary)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticKind {
    MissingSemicolonAfterNamedReference,
    UnknownNamedCharacterReference,
    ControlCharacterInInputStream,
}

fn control_diagnostic(scalar: char) -> bool {
    let code = scalar as u32;
    let c0 = code <= 0x1f && !matches!(scalar, '\0' | '\t' | '\n' | '\u{000c}' | '\r');
    let c1 = (0x7f..=0x9f).contains(&code);
    c0 || c1
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DiagnosticSite {
    /// The resolved reference whose maximum match did not end in `;`, related
    /// to its reference by entry ordinal and never by a raw range.
    ResolvedReference { entry_index: usize },
    /// The `;` scalar that ended an unresolved ampersand run, observed by the
    /// ambiguous ampersand state before Data reconsumes it.
    AmbiguousAmpersandSemicolon(Evidence),
    /// One authored input-preprocessing scalar.
    PreprocessingScalar(Evidence),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    kind: DiagnosticKind,
    source_id: SourceId,
    site: DiagnosticSite,
}

// ---------------------------------------------------------------------------
// Bounded Data tokenizer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Origin {
    /// Ordinary Data characters. Interpreted text equals authored text.
    Literal,
    /// A resolved Named reference. Interpreted text is decoded output.
    ResolvedNamed { name: &'static str },
    /// A flushed `&`, optionally with the ambiguous-ampersand alphanumeric
    /// run. Interpreted text equals authored text; this is neither a
    /// resolved reference nor markup.
    UnresolvedAmpersandRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Contribution {
    origin: Origin,
    authored: Evidence,
    interpreted: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    StartBody(Evidence),
    EndBody(Evidence),
    EndHtml(Evidence),
    Characters(Contribution),
    EndOfFile,
}

fn count_tokens(lexed: &Lexed, predicate: impl Fn(&Token) -> bool) -> usize {
    lexed.tokens.iter().filter(|token| predicate(token)).count()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outside {
    /// Any `<` shape other than the three exact tags, including every tag
    /// with attributes and every RCDATA element start (TC-S10 owns Title).
    TagShape,
    /// Data U+0000 — a separate existing defect, deliberately not modelled.
    DataNul,
    /// CR needs input-stream newline normalisation, outside this candidate.
    CarriageReturn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailStep {
    Preflight,
    PrepareEvidence,
}

/// Test-local semantic failpoint, keyed by the character-reference entry
/// ordinal whose Named commit must refuse at the given step. Not a production
/// resource strategy and not a new resource dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Failpoint {
    entry_index: usize,
    step: FailStep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenizerStop {
    /// `&#` — the Numeric branch was reached and is not selected. `trigger`
    /// identifies the branch only and is not committed coverage.
    NumericBranch {
        trigger: Evidence,
    },
    Outside(Outside),
    ResourceRefusal(Failpoint),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Entry(usize),
    Diagnostic(usize),
    Token(usize),
}

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
    tokens_before: usize,
    tokens_after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Lexed {
    source_id: SourceId,
    source_text: String,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    events: Vec<Event>,
    entries: Vec<Evidence>,
    lookaheads: Vec<LookaheadRecord>,
    cursor: usize,
    coverage_end: usize,
    stop: Option<TokenizerStop>,
}

impl Lexed {
    fn contributions(&self) -> Vec<&Contribution> {
        self.tokens
            .iter()
            .filter_map(|token| match token {
                Token::Characters(contribution) => Some(contribution),
                _ => None,
            })
            .collect()
    }

    fn interpreted(&self) -> String {
        self.contributions()
            .iter()
            .map(|contribution| contribution.interpreted.as_str())
            .collect()
    }

    fn diagnostic_kinds(&self) -> Vec<DiagnosticKind> {
        self.diagnostics.iter().map(|d| d.kind).collect()
    }

    fn reached_eof(&self) -> bool {
        self.tokens.last() == Some(&Token::EndOfFile)
    }

    /// `(origin, start, end, interpreted)` per contribution.
    fn projection(&self) -> Vec<(Origin, usize, usize, String)> {
        self.contributions()
            .iter()
            .map(|c| {
                (
                    c.origin.clone(),
                    c.authored.start,
                    c.authored.end,
                    c.interpreted.clone(),
                )
            })
            .collect()
    }
}

struct Tokenizer {
    source: SourceText,
    cursor: usize,
    coverage_end: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    events: Vec<Event>,
    entries: Vec<Evidence>,
    lookaheads: Vec<LookaheadRecord>,
    failpoint: Option<Failpoint>,
}

fn lex(text: &str) -> Lexed {
    lex_in(&SourceText::new(SourceId::new(1), text.to_owned()), None)
}

fn lex_in(source: &SourceText, failpoint: Option<Failpoint>) -> Lexed {
    let mut tokenizer = Tokenizer {
        source: source.clone(),
        cursor: 0,
        coverage_end: 0,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
        events: Vec::new(),
        entries: Vec::new(),
        lookaheads: Vec::new(),
        failpoint,
    };
    let stop = tokenizer.run().err();
    Lexed {
        source_id: tokenizer.source.id(),
        source_text: tokenizer.source.as_str().to_owned(),
        tokens: tokenizer.tokens,
        diagnostics: tokenizer.diagnostics,
        events: tokenizer.events,
        entries: tokenizer.entries,
        lookaheads: tokenizer.lookaheads,
        cursor: tokenizer.cursor,
        coverage_end: tokenizer.coverage_end,
        stop,
    }
}

impl Tokenizer {
    fn text(&self) -> String {
        self.source.as_str().to_owned()
    }

    fn run(&mut self) -> Result<(), TokenizerStop> {
        loop {
            let text = self.text();
            let Some(scalar) = text[self.cursor..].chars().next() else {
                self.push_token(Token::EndOfFile);
                return Ok(());
            };
            match scalar {
                '&' => self.character_reference(&text)?,
                '<' => self.tag(&text)?,
                '\0' => return Err(TokenizerStop::Outside(Outside::DataNul)),
                '\r' => return Err(TokenizerStop::Outside(Outside::CarriageReturn)),
                _ => self.literal_run(&text),
            }
        }
    }

    fn push_token(&mut self, token: Token) {
        self.tokens.push(token);
        self.events.push(Event::Token(self.tokens.len() - 1));
    }

    fn push_diagnostic(&mut self, kind: DiagnosticKind, site: DiagnosticSite) {
        self.diagnostics.push(Diagnostic {
            kind,
            source_id: self.source.id(),
            site,
        });
        self.events
            .push(Event::Diagnostic(self.diagnostics.len() - 1));
    }

    /// The single authoritative cursor advance. It runs input-preprocessing
    /// observation over exactly the selected source units, so a scalar that is
    /// only *examined* by lookahead is never observed here.
    fn commit(&mut self, end: usize) {
        assert!(end >= self.cursor, "the cursor never rolls back");
        let start = self.cursor;
        let mut offset = start;
        let selected = self.source.as_str()[start..end].to_owned();
        for scalar in selected.chars() {
            if control_diagnostic(scalar) {
                let span = evidence(&self.source, offset, offset + scalar.len_utf8());
                self.push_diagnostic(
                    DiagnosticKind::ControlCharacterInInputStream,
                    DiagnosticSite::PreprocessingScalar(span),
                );
            }
            offset += scalar.len_utf8();
        }
        self.cursor = end;
        self.coverage_end = end;
    }

    fn contribution(&self, origin: Origin, start: usize, end: usize, interpreted: &str) -> Token {
        Token::Characters(Contribution {
            origin,
            authored: evidence(&self.source, start, end),
            interpreted: interpreted.to_owned(),
        })
    }

    fn literal_run(&mut self, text: &str) {
        let start = self.cursor;
        let mut end = start;
        for scalar in text[start..].chars() {
            if matches!(scalar, '&' | '<' | '\0' | '\r') {
                break;
            }
            end += scalar.len_utf8();
        }
        self.commit(end);
        let token = self.contribution(Origin::Literal, start, end, &text[start..end]);
        self.push_token(token);
    }

    fn tag(&mut self, text: &str) -> Result<(), TokenizerStop> {
        let start = self.cursor;
        let rest = &text[start..];
        let (width, build): (usize, fn(Evidence) -> Token) = if rest.starts_with("<body>") {
            (6, Token::StartBody)
        } else if rest.starts_with("</body>") {
            (7, Token::EndBody)
        } else if rest.starts_with("</html>") {
            (7, Token::EndHtml)
        } else {
            return Err(TokenizerStop::Outside(Outside::TagShape));
        };
        self.commit(start + width);
        let token = build(evidence(&self.source, start, start + width));
        self.push_token(token);
        Ok(())
    }

    fn state(&self) -> (usize, usize, usize, usize) {
        (
            self.cursor,
            self.coverage_end,
            self.diagnostics.len(),
            self.tokens.len(),
        )
    }

    /// Data `&`: return state = Data, then the Character Reference state.
    fn character_reference(&mut self, text: &str) -> Result<(), TokenizerStop> {
        let ampersand = self.cursor;
        let after = ampersand + 1;

        // The Data return state consumes the authored `&` into the Character
        // Reference state. Committing it here keeps entry evidence from ever
        // preceding committed coverage.
        self.commit(after);
        self.entries.push(evidence(&self.source, ampersand, after));
        let entry_index = self.entries.len() - 1;
        self.events.push(Event::Entry(entry_index));

        match text[after..].chars().next() {
            Some('#') => Err(TokenizerStop::NumericBranch {
                trigger: evidence(&self.source, after, after + 1),
            }),
            Some(scalar) if scalar.is_ascii_alphanumeric() => {
                let before = self.state();
                let lookahead = named_lookahead(&text[after..]);
                let settled = self.state();
                assert_eq!(before, settled, "discovery must be non-committing");
                self.lookaheads.push(LookaheadRecord {
                    at: after,
                    examined_end: after + lookahead.examined_len,
                    matched: lookahead.matched.map(|matched| matched.name),
                    cursor_before: before.0,
                    cursor_after: settled.0,
                    coverage_before: before.1,
                    coverage_after: settled.1,
                    diagnostics_before: before.2,
                    diagnostics_after: settled.2,
                    tokens_before: before.3,
                    tokens_after: settled.3,
                });
                match lookahead.matched {
                    Some(matched) => self.commit_named(entry_index, ampersand, matched),
                    None => {
                        self.ambiguous_ampersand(text, ampersand, after);
                        Ok(())
                    }
                }
            }
            _ => {
                // Flush the `&` and reconsume in Data.
                let token =
                    self.contribution(Origin::UnresolvedAmpersandRun, ampersand, after, "&");
                self.push_token(token);
                Ok(())
            }
        }
    }

    fn fail_if(&self, entry_index: usize, step: FailStep) -> Result<(), TokenizerStop> {
        match self.failpoint {
            Some(point) if point.entry_index == entry_index && point.step == step => {
                Err(TokenizerStop::ResourceRefusal(point))
            }
            _ => Ok(()),
        }
    }

    /// preflight -> prepare evidence -> authoritative consumption ->
    /// non-refusing commit. Every fallible step precedes consumption, so a
    /// refusal leaves nothing to undo and no rollback exists.
    fn commit_named(
        &mut self,
        entry_index: usize,
        ampersand: usize,
        matched: NamedEntry,
    ) -> Result<(), TokenizerStop> {
        self.fail_if(entry_index, FailStep::Preflight)?;
        let end = ampersand + 1 + matched.name.len();
        let prepared = self.contribution(
            Origin::ResolvedNamed { name: matched.name },
            ampersand,
            end,
            matched.value,
        );
        self.fail_if(entry_index, FailStep::PrepareEvidence)?;

        // Authoritative matched-source consumption.
        self.commit(end);

        // Non-refusing commit: no `Result` below this line.
        if !matched.name.ends_with(';') {
            self.push_diagnostic(
                DiagnosticKind::MissingSemicolonAfterNamedReference,
                DiagnosticSite::ResolvedReference { entry_index },
            );
        }
        self.push_token(prepared);
        Ok(())
    }

    fn ambiguous_ampersand(&mut self, text: &str, ampersand: usize, after: usize) {
        let mut end = after;
        while let Some(scalar) = text[end..].chars().next() {
            if scalar.is_ascii_alphanumeric() {
                end += scalar.len_utf8();
            } else {
                break;
            }
        }
        self.commit(end);
        let run = self.contribution(
            Origin::UnresolvedAmpersandRun,
            ampersand,
            end,
            &text[ampersand..end],
        );
        self.push_token(run);
        if text[end..].starts_with(';') {
            let semicolon = evidence(&self.source, end, end + 1);
            self.push_diagnostic(
                DiagnosticKind::UnknownNamedCharacterReference,
                DiagnosticSite::AmbiguousAmpersandSemicolon(semicolon),
            );
        }
    }
}

/// Probe: consume the matched source first, then fail at the final effect.
/// Returns the partial state a resource refusal would expose.
fn consume_before_preflight_probe(text: &str, ampersand: usize) -> (usize, usize, usize) {
    let source = SourceText::new(SourceId::new(1), text.to_owned());
    let mut tokenizer = Tokenizer {
        source: source.clone(),
        cursor: 0,
        coverage_end: 0,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
        events: Vec::new(),
        entries: Vec::new(),
        lookaheads: Vec::new(),
        failpoint: None,
    };
    tokenizer.commit(ampersand + 1);
    let matched = named_lookahead(&text[ampersand + 1..])
        .matched
        .expect("probe requires a match");
    tokenizer.commit(ampersand + 1 + matched.name.len());
    if !matched.name.ends_with(';') {
        tokenizer.push_diagnostic(
            DiagnosticKind::MissingSemicolonAfterNamedReference,
            DiagnosticSite::ResolvedReference { entry_index: 0 },
        );
    }
    // The refusal arrives here, after consumption and diagnostics.
    (
        tokenizer.coverage_end,
        tokenizer.diagnostics.len(),
        tokenizer.tokens.len(),
    )
}

// ---------------------------------------------------------------------------
// Bounded tree over the existing selected document-tree positions
// ---------------------------------------------------------------------------

/// Exactly the four selected positions. No new insertion mode is introduced:
/// `EarlyPositions` collapses the existing early positions (Initial through After
/// Head) that stay honest unsupported boundaries for character data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    EarlyPositions,
    InBody,
    AfterBody,
    AfterAfterBody,
}

const ALL_MODES: [Mode; 4] = [
    Mode::EarlyPositions,
    Mode::InBody,
    Mode::AfterBody,
    Mode::AfterAfterBody,
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum Recovery {
    /// A non-whitespace scalar in After Body switched to In Body and was
    /// reprocessed there. Classified from the interpreted scalar only.
    AfterBodyNonWhitespaceToInBody { scalar: char },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextNode {
    text: String,
    contributions: Vec<Contribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Body {
    start: Evidence,
    text_nodes: Vec<TextNode>,
    close: Option<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Tree {
    /// Source-less `html` and `head` synthesized by the existing shell rules.
    shell_synthesized: bool,
    body: Option<Body>,
    html_close: Option<Evidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TreeBoundary {
    /// Character data before the body exists: early whitespace-sensitive
    /// positions stay unsupported in this first slice.
    CharacterDataBeforeBody,
    /// Character data after `</html>`: the existing AfterAfterBody boundary.
    CharacterDataInAfterAfterBody,
    /// Any other token the bounded model does not select.
    OutsideModelledCells,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Unsupported {
    /// The single remaining requirement after Named selection: the Numeric
    /// branch in Data. Deliberately not a production variant spelling.
    NumericCharacterReferenceInData,
    OutsideCandidate(Outside),
    Tree(TreeBoundary),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Completion {
    Complete,
    ResourceLimit,
    Unsupported(Unsupported),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    lexed: Lexed,
    tree: Tree,
    mode: Mode,
    recoveries: Vec<Recovery>,
    completion: Completion,
}

fn construct(lexed: Lexed) -> Observation {
    let mut tree = Tree {
        shell_synthesized: false,
        body: None,
        html_close: None,
    };
    let mut mode = Mode::EarlyPositions;
    let mut recoveries = Vec::new();
    let mut refusal: Option<Unsupported> = None;

    for token in &lexed.tokens {
        let outcome = match (mode, token) {
            (Mode::EarlyPositions, Token::StartBody(start)) => {
                tree.shell_synthesized = true;
                tree.body = Some(Body {
                    start: start.clone(),
                    text_nodes: Vec::new(),
                    close: None,
                });
                mode = Mode::InBody;
                Ok(())
            }
            (Mode::EarlyPositions, Token::Characters(_)) => {
                Err(TreeBoundary::CharacterDataBeforeBody)
            }
            (Mode::EarlyPositions, Token::EndOfFile) => Err(TreeBoundary::OutsideModelledCells),
            (Mode::EarlyPositions, _) => Err(TreeBoundary::OutsideModelledCells),
            (Mode::InBody, Token::Characters(contribution)) => {
                insert(&mut tree, contribution);
                Ok(())
            }
            (Mode::InBody, Token::EndBody(close)) => {
                tree.body.as_mut().expect("body").close = Some(close.clone());
                mode = Mode::AfterBody;
                Ok(())
            }
            (Mode::InBody | Mode::AfterBody, Token::EndOfFile) => Ok(()),
            (Mode::AfterBody, Token::Characters(contribution)) => {
                for scalar in contribution.interpreted.chars() {
                    if mode == Mode::AfterBody && !is_html_whitespace(scalar) {
                        recoveries.push(Recovery::AfterBodyNonWhitespaceToInBody { scalar });
                        mode = Mode::InBody;
                    }
                }
                insert(&mut tree, contribution);
                Ok(())
            }
            (Mode::AfterBody, Token::EndHtml(close)) => {
                tree.html_close = Some(close.clone());
                mode = Mode::AfterAfterBody;
                Ok(())
            }
            (Mode::AfterAfterBody, Token::EndOfFile) => Ok(()),
            (Mode::AfterAfterBody, Token::Characters(_)) => {
                Err(TreeBoundary::CharacterDataInAfterAfterBody)
            }
            _ => Err(TreeBoundary::OutsideModelledCells),
        };
        if let Err(boundary) = outcome {
            refusal = Some(Unsupported::Tree(boundary));
            break;
        }
    }

    let completion = match (refusal, &lexed.stop) {
        (Some(unsupported), _) => Completion::Unsupported(unsupported),
        (None, Some(TokenizerStop::NumericBranch { .. })) => {
            Completion::Unsupported(Unsupported::NumericCharacterReferenceInData)
        }
        (None, Some(TokenizerStop::Outside(outside))) => {
            Completion::Unsupported(Unsupported::OutsideCandidate(*outside))
        }
        (None, Some(TokenizerStop::ResourceRefusal(_))) => Completion::ResourceLimit,
        (None, None) => Completion::Complete,
    };

    Observation {
        lexed,
        tree,
        mode,
        recoveries,
        completion,
    }
}

/// Inserts one authored contribution into the body, coalescing with an
/// adjacent text node.
fn insert(tree: &mut Tree, contribution: &Contribution) {
    let body = tree.body.as_mut().expect("body");
    match body.text_nodes.last_mut() {
        Some(node) => {
            node.text.push_str(&contribution.interpreted);
            node.contributions.push(contribution.clone());
        }
        None => body.text_nodes.push(TextNode {
            text: contribution.interpreted.clone(),
            contributions: vec![contribution.clone()],
        }),
    }
}

fn run(text: &str) -> Observation {
    construct(lex(text))
}

fn run_with(text: &str, failpoint: Failpoint) -> Observation {
    let source = SourceText::new(SourceId::new(1), text.to_owned());
    construct(lex_in(&source, Some(failpoint)))
}

impl Observation {
    fn body_text(&self) -> String {
        self.tree
            .body
            .as_ref()
            .map(|body| body.text_nodes.iter().map(|n| n.text.as_str()).collect())
            .unwrap_or_default()
    }

    fn body_contributions(&self) -> Vec<(Origin, usize, usize, String)> {
        self.tree
            .body
            .iter()
            .flat_map(|body| body.text_nodes.iter())
            .flat_map(|node| node.contributions.iter())
            .map(|c| {
                (
                    c.origin.clone(),
                    c.authored.start,
                    c.authored.end,
                    c.interpreted.clone(),
                )
            })
            .collect()
    }

    fn text_node_count(&self) -> usize {
        self.tree
            .body
            .as_ref()
            .map_or(0, |body| body.text_nodes.len())
    }
}

// ---------------------------------------------------------------------------
// Freeze validation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum FreezeError {
    CompleteWithoutFullCoverage,
    NonCompleteHasEndOfFile,
    ContributionOutOfOrderOrOverlapping,
    ContributionRawMismatch,
    LiteralInterpretationMismatch,
    ContributionBeyondCoverage,
    NamedValueMismatch,
    TreeContributionNotFromLexed,
    TextNodeDoesNotEqualContributions,
    FabricatedSubdivision,
}

fn validate_freeze(observation: &Observation) -> Result<(), FreezeError> {
    let lexed = &observation.lexed;
    let complete = observation.completion == Completion::Complete;
    if complete && (lexed.coverage_end != lexed.source_text.len() || !lexed.reached_eof()) {
        return Err(FreezeError::CompleteWithoutFullCoverage);
    }
    if lexed.stop.is_some() && lexed.reached_eof() {
        return Err(FreezeError::NonCompleteHasEndOfFile);
    }

    let mut previous_end = 0usize;
    for contribution in lexed.contributions() {
        let span = &contribution.authored;
        if span.start < previous_end || span.start >= span.end {
            return Err(FreezeError::ContributionOutOfOrderOrOverlapping);
        }
        previous_end = span.end;
        if span.end > lexed.coverage_end {
            return Err(FreezeError::ContributionBeyondCoverage);
        }
        if lexed.source_text[span.start..span.end] != span.raw {
            return Err(FreezeError::ContributionRawMismatch);
        }
        if matches!(
            contribution.origin,
            Origin::Literal | Origin::UnresolvedAmpersandRun
        ) && contribution.interpreted != span.raw
        {
            return Err(FreezeError::LiteralInterpretationMismatch);
        }
        if let Origin::ResolvedNamed { name } = contribution.origin {
            let expected = NAMED_REFERENCES
                .iter()
                .find(|candidate| candidate.name == name)
                .map(|candidate| candidate.value);
            if expected != Some(contribution.interpreted.as_str()) {
                return Err(FreezeError::NamedValueMismatch);
            }
            // One authored source span per reference, however many scalars.
            if span.raw != format!("&{name}") {
                return Err(FreezeError::FabricatedSubdivision);
            }
        }
    }

    let lexed_contributions = lexed.contributions();
    let placed_contributions: Vec<&Contribution> = observation
        .tree
        .body
        .iter()
        .flat_map(|body| body.text_nodes.iter())
        .flat_map(|node| node.contributions.iter())
        .collect();

    let mut next_lexed = 0usize;
    for placed in &placed_contributions {
        while next_lexed < lexed_contributions.len()
            && lexed_contributions[next_lexed] != *placed
        {
            next_lexed += 1;
        }
        if next_lexed == lexed_contributions.len() {
            return Err(FreezeError::TreeContributionNotFromLexed);
        }
        next_lexed += 1;
    }
    if complete && placed_contributions.len() != lexed_contributions.len() {
        return Err(FreezeError::TreeContributionNotFromLexed);
    }

    for node in observation
        .tree
        .body
        .iter()
        .flat_map(|body| body.text_nodes.iter())
    {
        let joined: String = node
            .contributions
            .iter()
            .map(|c| c.interpreted.as_str())
            .collect();
        if joined != node.text {
            return Err(FreezeError::TextNodeDoesNotEqualContributions);
        }
    }
    Ok(())
}

fn assert_valid(observation: &Observation) {
    assert_eq!(validate_freeze(observation), Ok(()));
}

fn lit(start: usize, end: usize, text: &str) -> (Origin, usize, usize, String) {
    (Origin::Literal, start, end, text.to_owned())
}

fn named(
    name: &'static str,
    start: usize,
    end: usize,
    value: &str,
) -> (Origin, usize, usize, String) {
    (Origin::ResolvedNamed { name }, start, end, value.to_owned())
}

fn unresolved(start: usize, end: usize, text: &str) -> (Origin, usize, usize, String) {
    (Origin::UnresolvedAmpersandRun, start, end, text.to_owned())
}

const MISSING: DiagnosticKind = DiagnosticKind::MissingSemicolonAfterNamedReference;
const UNKNOWN: DiagnosticKind = DiagnosticKind::UnknownNamedCharacterReference;
const CONTROL: DiagnosticKind = DiagnosticKind::ControlCharacterInInputStream;

// ---------------------------------------------------------------------------
// Character Reference / Named lexical cells
// ---------------------------------------------------------------------------

#[test]
fn k1_bare_ampersand_flushes_literally_and_reaches_eof() {
    let lexed = lex("&");
    assert_eq!(lexed.projection(), vec![unresolved(0, 1, "&")]);
    assert!(lexed.diagnostics.is_empty());
    assert!(lexed.reached_eof() && lexed.stop.is_none());
    assert_eq!(lexed.interpreted(), "&");
}

#[test]
fn k2_ampersand_semicolon_flushes_and_reconsumes_the_semicolon_in_data() {
    let lexed = lex("&;");
    assert_eq!(
        lexed.projection(),
        vec![unresolved(0, 1, "&"), lit(1, 2, ";")]
    );
    assert!(
        lexed.diagnostics.is_empty(),
        "no alphanumeric, no ambiguity"
    );
    assert_eq!(lexed.interpreted(), "&;");
}

#[test]
fn k3_unknown_alphanumeric_at_eof_is_unresolved_without_observation() {
    let lexed = lex("&x");
    assert_eq!(lexed.projection(), vec![unresolved(0, 2, "&x")]);
    assert!(lexed.diagnostics.is_empty());
    assert!(lexed.reached_eof());
}

#[test]
fn k4_unknown_name_with_semicolon_observes_at_the_semicolon_and_reconsumes_it() {
    let lexed = lex("&x;");
    assert_eq!(
        lexed.projection(),
        vec![unresolved(0, 2, "&x"), lit(2, 3, ";")]
    );
    assert_eq!(lexed.diagnostic_kinds(), vec![UNKNOWN]);
    match &lexed.diagnostics[0].site {
        DiagnosticSite::AmbiguousAmpersandSemicolon(span) => {
            assert_eq!((span.start, span.end, span.raw.as_str()), (2, 3, ";"));
        }
        other => panic!("unexpected site {other:?}"),
    }
    assert_eq!(lexed.interpreted(), "&x;");
}

#[test]
fn k5_exact_semicolon_terminated_name_resolves_without_observation() {
    let lexed = lex("&amp;");
    assert_eq!(lexed.projection(), vec![named("amp;", 0, 5, "&")]);
    assert!(lexed.diagnostics.is_empty());
}

#[test]
fn k6_semicolonless_legacy_name_resolves_with_missing_semicolon() {
    let lexed = lex("&amp");
    assert_eq!(lexed.projection(), vec![named("amp", 0, 4, "&")]);
    assert_eq!(lexed.diagnostic_kinds(), vec![MISSING]);
}

#[test]
fn k7_data_does_not_inherit_the_attribute_value_equals_exception() {
    let lexed = lex("&amp=");
    assert_eq!(
        lexed.projection(),
        vec![named("amp", 0, 4, "&"), lit(4, 5, "=")]
    );
    assert_eq!(lexed.diagnostic_kinds(), vec![MISSING]);
    assert_eq!(lexed.interpreted(), "&=");
    assert_eq!(attribute_exception_lookup("amp="), None);
    assert_ne!(
        attribute_exception_lookup("amp="),
        named_lookahead("amp=").matched,
        "the AttributeValue rule would leave this unresolved"
    );
}

#[test]
fn k8_semicolonless_maximum_prefix_leaves_the_remainder_ordinary() {
    let lexed = lex("&ampx;");
    assert_eq!(
        lexed.projection(),
        vec![named("amp", 0, 4, "&"), lit(4, 6, "x;")]
    );
    assert_eq!(lexed.diagnostic_kinds(), vec![MISSING]);
    assert_eq!(lexed.interpreted(), "&x;");
    assert_eq!(attribute_exception_lookup("ampx;"), None);
}

#[test]
fn k9_a_shorter_semicolonless_prefix_wins_over_a_failed_longer_name() {
    let lexed = lex("&notit;");
    assert_eq!(
        lexed.projection(),
        vec![named("not", 0, 4, "\u{00ac}"), lit(4, 7, "it;")]
    );
    assert_eq!(lexed.diagnostic_kinds(), vec![MISSING]);
    assert_eq!(lexed.interpreted(), "\u{00ac}it;");
}

#[test]
fn k10_a_longer_exact_name_wins_the_maximum_match() {
    let lexed = lex("&notin;");
    assert_eq!(lexed.projection(), vec![named("notin;", 0, 7, "\u{2209}")]);
    assert!(lexed.diagnostics.is_empty());
    let longer = lex("&notinva;");
    assert_eq!(
        longer.projection(),
        vec![named("notinva;", 0, 9, "\u{2209}")]
    );
    let partial = lex("&notinv");
    assert_eq!(
        partial.projection(),
        vec![named("not", 0, 4, "\u{00ac}"), lit(4, 7, "inv")]
    );
    assert_eq!(partial.diagnostic_kinds(), vec![MISSING]);
}

#[test]
fn k11_one_authored_reference_may_decode_to_multiple_scalars() {
    let lexed = lex("&NotEqualTilde;");
    assert_eq!(
        lexed.projection(),
        vec![named("NotEqualTilde;", 0, 15, "\u{2242}\u{0338}")]
    );
    assert_eq!(lexed.contributions().len(), 1);
    assert_eq!(lexed.interpreted().chars().count(), 2);
    assert!(lexed.diagnostics.is_empty());
    // `NotEqual;` is a different name and is not a prefix of this input.
    assert_eq!(
        named_lookahead("NotEqualTilde;").matched.map(|m| m.name),
        Some("NotEqualTilde;")
    );
}

#[test]
fn k12_unknown_name_with_semicolon_is_an_unresolved_run_plus_observation() {
    let lexed = lex("&bogus;");
    assert_eq!(
        lexed.projection(),
        vec![unresolved(0, 6, "&bogus"), lit(6, 7, ";")]
    );
    assert_eq!(lexed.diagnostic_kinds(), vec![UNKNOWN]);
    assert_eq!(lexed.interpreted(), "&bogus;");
    assert!(
        lexed
            .contributions()
            .iter()
            .all(|c| !matches!(c.origin, Origin::ResolvedNamed { .. }))
    );
}

#[test]
fn k13_unknown_name_at_eof_has_no_semicolon_triggered_observation() {
    let lexed = lex("&bogus");
    assert_eq!(lexed.projection(), vec![unresolved(0, 6, "&bogus")]);
    assert!(lexed.diagnostics.is_empty());
    assert!(lexed.reached_eof());
    assert_ne!(
        lex("&bogus;").diagnostic_kinds(),
        lex("&bogus").diagnostic_kinds()
    );
}

#[test]
fn k14_a_second_ampersand_reenters_character_reference() {
    let lexed = lex("&&amp;");
    assert_eq!(
        lexed.projection(),
        vec![unresolved(0, 1, "&"), named("amp;", 1, 6, "&")]
    );
    assert_eq!(lexed.entries.len(), 2);
}

// ---------------------------------------------------------------------------
// Maximum-match falsifiers
// ---------------------------------------------------------------------------

#[test]
fn m1_whole_name_only_lookup_is_falsified() {
    for (rest, expected) in [
        ("amp", "amp"),
        ("notit;", "not"),
        ("ampx;", "amp"),
        ("nbsp ", "nbsp"),
    ] {
        assert_eq!(
            named_lookahead(rest).matched.map(|m| m.name),
            Some(expected)
        );
        assert_ne!(
            exact_whole_string_lookup(rest).map(|m| m.name),
            Some(expected),
            "whole-name lookup cannot explain `{rest}`"
        );
    }
}

#[test]
fn m2_a_small_whitelist_is_falsified() {
    for rest in [
        "nbsp;",
        "notin;",
        "NotEqualTilde;",
        "Tab;",
        "NewLine;",
        "amp",
    ] {
        assert!(named_lookahead(rest).matched.is_some());
        assert_eq!(
            tiny_whitelist_lookup(rest),
            None,
            "a whitelist cannot explain `{rest}`"
        );
    }
}

#[test]
fn m3_the_attribute_exception_leaking_into_data_is_falsified() {
    for rest in ["amp=", "ampx;", "not1", "lt="] {
        assert!(named_lookahead(rest).matched.is_some());
        assert_eq!(attribute_exception_lookup(rest), None);
        assert!(
            lex(&format!("&{rest}"))
                .contributions()
                .iter()
                .any(|c| matches!(c.origin, Origin::ResolvedNamed { .. }))
        );
    }
    // Semicolon-terminated names are unaffected by that exception.
    assert!(attribute_exception_lookup("amp;").is_some());
}

#[test]
fn m4_named_table_is_hand_authored_and_never_decodes_to_nul() {
    assert!(
        NAMED_REFERENCES
            .iter()
            .all(|candidate| !candidate.value.contains('\u{0000}'))
    );
    let mut sorted: Vec<_> = NAMED_REFERENCES.iter().map(|c| c.name).collect();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), NAMED_REFERENCES.len());
}

// ---------------------------------------------------------------------------
// Non-recursion / authored-versus-interpreted
// ---------------------------------------------------------------------------

#[test]
fn n1_decoded_output_is_not_recursively_decoded() {
    let lexed = lex("&amp;lt;");
    assert_eq!(
        lexed.projection(),
        vec![named("amp;", 0, 5, "&"), lit(5, 8, "lt;")]
    );
    assert_eq!(lexed.interpreted(), "&lt;");
    assert_ne!(
        recursive_decode("&amp;lt;"),
        lexed.interpreted(),
        "recursive decoding would produce `<`"
    );
    assert_eq!(recursive_decode("&amp;lt;"), "<");
    assert_eq!(lexed.entries.len(), 1, "only the authored `&` entered");
}

#[test]
fn n2_decoded_syntax_is_character_data_and_never_an_authored_end_tag() {
    let lexed = lex("&lt;/body>");
    assert_eq!(
        lexed.projection(),
        vec![named("lt;", 0, 4, "<"), lit(4, 10, "/body>")]
    );
    assert_eq!(lexed.interpreted(), "</body>");
    assert_eq!(count_tokens(&lexed, |t| matches!(t, Token::EndBody(_))), 0);
    assert_eq!(
        retokenized_end_body_count("&lt;/body>"),
        1,
        "retokenizing decoded output would fabricate an authored end tag"
    );
}

// ---------------------------------------------------------------------------
// Lookahead purity
// ---------------------------------------------------------------------------

#[test]
fn l1_speculative_lookahead_cannot_commit_a_later_preprocessing_diagnostic() {
    let text = "&not\u{0001}";
    let lexed = lex(text);
    assert_eq!(
        lexed.projection(),
        vec![named("not", 0, 4, "\u{00ac}"), lit(4, 5, "\u{0001}")]
    );
    assert_eq!(lexed.diagnostic_kinds(), vec![MISSING, CONTROL]);

    let lookahead = &lexed.lookaheads[0];
    assert_eq!(lookahead.at, 1);
    assert_eq!(lookahead.matched, Some("not"));
    assert_eq!(
        lookahead.examined_end, 5,
        "discovery examined the scalar that ended the match"
    );
    assert_eq!(lookahead.cursor_before, lookahead.cursor_after);
    assert_eq!(lookahead.coverage_before, lookahead.coverage_after);
    assert_eq!(lookahead.diagnostics_before, lookahead.diagnostics_after);
    assert_eq!(lookahead.tokens_before, lookahead.tokens_after);
    assert_eq!((lookahead.cursor_before, lookahead.coverage_before), (1, 1));

    // Ordering: entry, missing-semicolon, reference, control, literal, EOF.
    assert_eq!(
        lexed.events,
        vec![
            Event::Entry(0),
            Event::Diagnostic(0),
            Event::Token(0),
            Event::Diagnostic(1),
            Event::Token(1),
            Event::Token(2),
        ]
    );
    match &lexed.diagnostics[1].site {
        DiagnosticSite::PreprocessingScalar(span) => {
            assert_eq!((span.start, span.end), (4, 5));
        }
        other => panic!("unexpected site {other:?}"),
    }
    assert_eq!(
        committing_lookahead_diagnostic_order(text, 0),
        vec![CONTROL, MISSING],
        "a committing lookahead reorders the diagnostics"
    );
    assert_ne!(
        lexed.diagnostic_kinds(),
        committing_lookahead_diagnostic_order(text, 0)
    );
}

#[test]
fn l2_unmatched_lookahead_is_also_non_committing() {
    let lexed = lex("&bogus\u{0001}");
    assert_eq!(lexed.lookaheads[0].matched, None);
    assert_eq!(
        lexed.lookaheads[0].coverage_before,
        lexed.lookaheads[0].coverage_after
    );
    assert_eq!(lexed.diagnostic_kinds(), vec![CONTROL]);
}

// ---------------------------------------------------------------------------
// Resource atomicity
// ---------------------------------------------------------------------------

const PROVENANCE_FIXTURE: &str = "<body>a&amp;b</body>";

#[test]
fn a1_refusal_at_either_fallible_step_leaves_no_partial_reference() {
    for step in [FailStep::Preflight, FailStep::PrepareEvidence] {
        let failpoint = Failpoint {
            entry_index: 0,
            step,
        };
        let observation = run_with("<body>a&amp</body>", failpoint);
        assert_eq!(observation.completion, Completion::ResourceLimit);
        let lexed = &observation.lexed;
        assert_eq!(lexed.stop, Some(TokenizerStop::ResourceRefusal(failpoint)));
        // Only the authored `&` entry is committed; the matched name is not.
        assert_eq!((lexed.cursor, lexed.coverage_end), (8, 8));
        assert!(
            lexed.diagnostics.is_empty(),
            "no diagnostic without its effect"
        );
        assert!(
            lexed
                .contributions()
                .iter()
                .all(|c| !matches!(c.origin, Origin::ResolvedNamed { .. }))
        );
        assert_eq!(
            observation.body_text(),
            "a",
            "no partial interpreted output"
        );
        assert_eq!(observation.tree.body.as_ref().expect("body").close, None);
        assert!(!lexed.reached_eof(), "completion is never upgraded");
        assert_ne!(observation.completion, Completion::Complete);
        assert_valid(&observation);
    }
}

#[test]
fn a2_refusal_of_a_later_reference_keeps_the_earlier_committed_one() {
    let failpoint = Failpoint {
        entry_index: 1,
        step: FailStep::PrepareEvidence,
    };
    let observation = run_with("<body>&amp;&lt;</body>", failpoint);
    assert_eq!(observation.completion, Completion::ResourceLimit);
    assert_eq!(
        observation.body_contributions(),
        vec![named("amp;", 6, 11, "&")]
    );
    assert_eq!(observation.lexed.coverage_end, 12);
    assert_valid(&observation);
}

#[test]
fn a3_consume_before_preflight_would_expose_a_partial_commit() {
    let (coverage, diagnostics, tokens) = consume_before_preflight_probe("&amp", 0);
    let refused = run_with(
        "&amp",
        Failpoint {
            entry_index: 0,
            step: FailStep::Preflight,
        },
    );
    assert_eq!((coverage, diagnostics, tokens), (4, 1, 0));
    assert_eq!(refused.lexed.coverage_end, 1);
    assert_eq!(refused.lexed.diagnostics.len(), 0);
    assert_ne!(
        (coverage, diagnostics),
        (refused.lexed.coverage_end, refused.lexed.diagnostics.len()),
        "the broken order consumes source and records a diagnostic with no effect"
    );
}

#[test]
fn a4_a_run_without_a_failpoint_is_unaffected_by_the_failpoint_machinery() {
    let plain = run(PROVENANCE_FIXTURE);
    let unmatched = run_with(
        PROVENANCE_FIXTURE,
        Failpoint {
            entry_index: 5,
            step: FailStep::Preflight,
        },
    );
    assert_eq!(plain.completion, Completion::Complete);
    assert_eq!(plain.body_text(), unmatched.body_text());
}

// ---------------------------------------------------------------------------
// Numeric boundary
// ---------------------------------------------------------------------------

#[test]
fn x1_numeric_entry_is_reached_and_the_remaining_requirement_is_only_numeric() {
    for text in ["<body>&#65;</body>", "<body>&#x41;</body>"] {
        let observation = run(text);
        assert_eq!(
            observation.completion,
            Completion::Unsupported(Unsupported::NumericCharacterReferenceInData)
        );
        let lexed = &observation.lexed;
        assert_eq!(
            lexed.entries.len(),
            1,
            "Data Character Reference entry reached"
        );
        assert_eq!((lexed.entries[0].start, lexed.entries[0].end), (6, 7));
        assert_eq!(
            lexed.coverage_end, 7,
            "the trigger is not committed coverage"
        );
        match &lexed.stop {
            Some(TokenizerStop::NumericBranch { trigger }) => {
                assert_eq!(
                    (trigger.start, trigger.end, trigger.raw.as_str()),
                    (7, 8, "#")
                );
            }
            other => panic!("unexpected stop {other:?}"),
        }
        assert!(
            lexed.contributions().is_empty(),
            "no decoded or fabricated output"
        );
        assert_eq!(observation.body_text(), "");
        assert_valid(&observation);
    }
    assert_eq!(accidental_numeric_decode("65"), Some('A'));
    assert_ne!(run("<body>&#65;</body>").body_text(), "A");
}

#[test]
fn x2_the_durable_stop_is_not_the_coarse_all_data_references_claim() {
    let named_run = run("<body>&amp;</body>");
    let numeric_run = run("<body>&#65;</body>");
    assert_eq!(named_run.completion, Completion::Complete);
    assert!(coarse_every_ampersand_is_deferred("<body>&amp;</body>"));
    assert_ne!(
        named_run.completion, numeric_run.completion,
        "a coarse Data-reference stop would wrongly cover Named"
    );
    // The only Unsupported variant that concerns character references is the
    // Numeric one; there is no variant meaning "any Data reference".
    let all = [
        Unsupported::NumericCharacterReferenceInData,
        Unsupported::OutsideCandidate(Outside::TagShape),
        Unsupported::Tree(TreeBoundary::OutsideModelledCells),
    ];
    assert_eq!(
        all.iter()
            .filter(|u| matches!(u, Unsupported::NumericCharacterReferenceInData))
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------
// Tree and provenance
// ---------------------------------------------------------------------------

#[test]
fn t1_interleaved_authored_contributions_survive_one_interpreted_text_node() {
    let observation = run(PROVENANCE_FIXTURE);
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.text_node_count(), 1);
    assert_eq!(observation.body_text(), "a&b");
    assert_eq!(
        observation.body_contributions(),
        vec![lit(6, 7, "a"), named("amp;", 7, 12, "&"), lit(12, 13, "b")]
    );
    let body = observation.tree.body.as_ref().expect("body");
    assert_eq!((body.start.start, body.start.end), (0, 6));
    let close = body.close.as_ref().expect("authored close");
    assert_eq!(
        (close.start, close.end, close.raw.as_str()),
        (13, 20, "</body>")
    );
    assert!(observation.tree.shell_synthesized);
    assert_eq!(observation.mode, Mode::AfterBody);
    assert_valid(&observation);
}

#[test]
fn t2_one_reference_one_span_many_scalars_without_fabricated_subdivision() {
    let observation = run("<body>&NotEqualTilde;</body>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(
        observation.body_contributions(),
        vec![named("NotEqualTilde;", 6, 21, "\u{2242}\u{0338}")]
    );
    assert_eq!(observation.body_contributions().len(), 1);
    assert_eq!(observation.body_text().chars().count(), 2);
    assert_valid(&observation);

    // One-reference-one-scalar assumption: a per-scalar span would not be real.
    let (start, end) = span_from_decoded_len(6, &observation.body_text());
    assert_ne!((start, end), (6, 21), "decoded-length inference is wrong");
}

#[test]
fn t3_source_is_not_recovered_by_later_search_or_length() {
    let observation = run(PROVENANCE_FIXTURE);
    let source = PROVENANCE_FIXTURE;
    let actual = observation.body_contributions()[1].clone();
    assert_eq!((actual.1, actual.2), (7, 12));
    assert_eq!(span_from_later_search(source, "&"), Some((7, 8)));
    assert_ne!(
        span_from_later_search(source, "&"),
        Some((actual.1, actual.2))
    );
    assert_ne!(
        span_from_decoded_len(actual.1, &actual.3),
        (actual.1, actual.2)
    );
}

#[test]
fn t4_flattening_provenance_loses_the_ordered_causality() {
    let observation = run(PROVENANCE_FIXTURE);
    let flattened = vec![lit(6, 13, "a&b")];
    assert_ne!(observation.body_contributions(), flattened);
    assert_eq!(observation.body_contributions().len(), 3);
}

#[test]
fn t5_tab_is_html_whitespace_in_body() {
    let observation = run("<body>&Tab;</body>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.body_text(), "\t");
    assert_eq!(
        observation.body_contributions(),
        vec![named("Tab;", 6, 11, "\t")]
    );
    assert!(observation.body_text().chars().all(is_html_whitespace));
    assert_valid(&observation);
}

#[test]
fn t6_newline_after_body_is_whitespace_and_stays_after_body() {
    let observation = run("<body></body>&NewLine;");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.body_text(), "\n");
    assert_eq!(observation.mode, Mode::AfterBody);
    assert!(observation.recoveries.is_empty());
    assert_valid(&observation);
}

#[test]
fn t7_nbsp_after_body_is_not_whitespace_and_recovers_to_in_body() {
    let observation = run("<body></body>&nbsp;");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.body_text(), "\u{00a0}");
    assert_eq!(observation.mode, Mode::InBody);
    assert_eq!(
        observation.recoveries,
        vec![Recovery::AfterBodyNonWhitespaceToInBody { scalar: '\u{00a0}' }]
    );
    assert!(!is_html_whitespace('\u{00a0}'));
    assert_valid(&observation);
}

#[test]
fn t8_placement_follows_the_decoded_value_not_the_authored_spelling() {
    let via_reference = run("<body></body>&Tab;");
    let via_literal = run("<body></body>\t");
    assert_eq!(via_reference.mode, via_literal.mode);
    assert_eq!(via_reference.body_text(), via_literal.body_text());
    assert_eq!(via_reference.recoveries, via_literal.recoveries);
    // Provenance still differs: the authored causes are not the same.
    assert_ne!(
        via_reference.body_contributions(),
        via_literal.body_contributions()
    );
    // A spelling-based classifier misreads the reference.
    assert!(!spelling_based_is_whitespace("&Tab;"));
    assert!(is_html_whitespace('\t'));
}

#[test]
fn t9_decoded_end_tag_text_is_body_text_and_closes_nothing() {
    let unclosed = run("<body>&lt;/body>");
    assert_eq!(unclosed.completion, Completion::Complete);
    assert_eq!(unclosed.body_text(), "</body>");
    assert_eq!(unclosed.tree.body.as_ref().expect("body").close, None);
    assert_eq!(unclosed.mode, Mode::InBody);

    let closed = run("<body>&lt;/body></body>");
    assert_eq!(closed.body_text(), "</body>");
    let close = closed
        .tree
        .body
        .as_ref()
        .expect("body")
        .close
        .clone()
        .expect("close");
    assert_eq!(
        (close.start, close.end),
        (16, 23),
        "only the authored tag closes"
    );
    assert_valid(&closed);
}

#[test]
fn t10_tree_frontier_is_not_widened() {
    // Early character data stays an honest unsupported boundary.
    for text in ["&Tab;<body>", "&amp;<body>", "&nbsp;<body>"] {
        let observation = run(text);
        assert_eq!(
            observation.completion,
            Completion::Unsupported(Unsupported::Tree(TreeBoundary::CharacterDataBeforeBody)),
            "{text}"
        );
        assert!(observation.tree.body.is_none());
        assert_valid(&observation);
    }
    // After `</html>` the existing boundary is preserved.
    let observation = run("<body></body></html>&nbsp;");
    assert_eq!(
        observation.completion,
        Completion::Unsupported(Unsupported::Tree(
            TreeBoundary::CharacterDataInAfterAfterBody
        ))
    );
    assert_eq!(observation.mode, Mode::AfterAfterBody);
    assert!(observation.tree.html_close.is_some());
    assert_eq!(observation.body_text(), "");
    // No insertion mode was added to the bounded model.
    assert_eq!(ALL_MODES.len(), 4);
    for mode in ALL_MODES {
        match mode {
            Mode::EarlyPositions | Mode::InBody | Mode::AfterBody | Mode::AfterAfterBody => {}
        }
    }
}

#[test]
fn t11_eof_in_selected_positions_completes_without_fabricating_closes() {
    let observation = run("<body>a&amp");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.body_text(), "a&");
    assert_eq!(observation.tree.body.as_ref().expect("body").close, None);
    assert_eq!(observation.lexed.diagnostic_kinds(), vec![MISSING]);
    assert_valid(&observation);
}

// ---------------------------------------------------------------------------
// Predecessor supersession and protected boundaries
// ---------------------------------------------------------------------------

struct SupersededPredecessor {
    id: &'static str,
    source: &'static str,
    claim: &'static str,
}

/// The predecessor expectation Issue #874 names as intentionally challenged.
/// It is recorded here, not edited: this validation touches no production
/// gold.
const SUPERSEDED: SupersededPredecessor = SupersededPredecessor {
    id: "UNSUP-001",
    source: "&x",
    claim: "CharacterReference(Data), Deferred, coverage empty",
};

const PROTECTED_ATTRIBUTE_VALUE: (&str, &str) = ("UNSUP-002", "<a x=&x>");

#[test]
fn s1_unsup_001_is_explicitly_superseded_for_data_only() {
    assert_eq!(SUPERSEDED.id, "UNSUP-001");
    assert!(SUPERSEDED.claim.contains("Deferred"));
    let lexed = lex(SUPERSEDED.source);
    // Superseded: Data `&x` is no longer a deferred stop.
    assert!(lexed.stop.is_none());
    assert!(lexed.reached_eof());
    assert_eq!(lexed.projection(), vec![unresolved(0, 2, "&x")]);
    // The new frontier commits the authored `&` entry, so the predecessor's
    // empty-coverage trigger shape is part of what is superseded.
    assert_eq!(lexed.coverage_end, 2);
}

#[test]
fn s2_attribute_value_references_remain_outside_the_candidate() {
    let (id, source) = PROTECTED_ATTRIBUTE_VALUE;
    assert_eq!(id, "UNSUP-002");
    for text in [source, "<body class=\"&amp;\">"] {
        let observation = run(text);
        assert_eq!(
            observation.completion,
            Completion::Unsupported(Unsupported::OutsideCandidate(Outside::TagShape))
        );
        assert_eq!(observation.lexed.coverage_end, 0);
        assert!(observation.lexed.contributions().is_empty());
        assert!(observation.lexed.entries.is_empty());
    }
}

#[test]
fn s3_title_rcdata_remains_outside_this_data_candidate() {
    let observation = run("<title>&amp;</title>");
    assert_eq!(
        observation.completion,
        Completion::Unsupported(Unsupported::OutsideCandidate(Outside::TagShape))
    );
    assert!(observation.lexed.entries.is_empty());
}

#[test]
fn s4_data_nul_is_neither_corrected_nor_modelled() {
    let observation = run("<body>\u{0000}</body>");
    assert_eq!(
        observation.completion,
        Completion::Unsupported(Unsupported::OutsideCandidate(Outside::DataNul))
    );
    // No claim about NUL output either way: nothing was produced for it.
    assert!(observation.lexed.contributions().is_empty());
    assert!(!observation.body_text().contains('\u{fffd}'));
    assert!(!observation.body_text().contains('\u{0000}'));
}

// ---------------------------------------------------------------------------
// Controls
// ---------------------------------------------------------------------------

#[test]
fn c1_source_id_perturbation_changes_provenance_not_semantics() {
    let first = construct(lex_in(
        &SourceText::new(SourceId::new(1), PROVENANCE_FIXTURE.to_owned()),
        None,
    ));
    let second = construct(lex_in(
        &SourceText::new(SourceId::new(2), PROVENANCE_FIXTURE.to_owned()),
        None,
    ));
    assert_eq!(first.body_text(), second.body_text());
    assert_eq!(first.body_contributions(), second.body_contributions());
    assert_eq!(first.completion, second.completion);
    assert_ne!(
        first.lexed.entries[0].source_id,
        second.lexed.entries[0].source_id
    );
    assert_ne!(first.lexed.source_id, second.lexed.source_id);
}

#[test]
fn c2_repeated_runs_are_deterministic() {
    assert_eq!(run(PROVENANCE_FIXTURE), run(PROVENANCE_FIXTURE));
    assert_eq!(run("&notit;"), run("&notit;"));
}

#[test]
fn c3_lower_layer_stops_are_never_upgraded_to_complete() {
    for text in ["<body>&#65;", "<body x=1>", "<body>\u{0000}", "&Tab;<body>"] {
        let observation = run(text);
        assert_ne!(observation.completion, Completion::Complete, "{text}");
        assert!(
            !observation.lexed.reached_eof()
                || matches!(
                    observation.completion,
                    Completion::Unsupported(Unsupported::Tree(_))
                )
        );
    }
}

#[test]
fn c4_freeze_rejects_impossible_observations() {
    let good = run(PROVENANCE_FIXTURE);
    assert_valid(&good);

    let mut short = good.clone();
    short.lexed.coverage_end = 3;
    assert_eq!(
        validate_freeze(&short),
        Err(FreezeError::CompleteWithoutFullCoverage)
    );

    let mut eof_on_refusal = run_with(
        "<body>&amp;",
        Failpoint {
            entry_index: 0,
            step: FailStep::Preflight,
        },
    );
    eof_on_refusal.lexed.tokens.push(Token::EndOfFile);
    assert_eq!(
        validate_freeze(&eof_on_refusal),
        Err(FreezeError::NonCompleteHasEndOfFile)
    );

    let mut overlapping = good.clone();
    if let Token::Characters(contribution) = &mut overlapping.lexed.tokens[1] {
        contribution.authored = evidence(
            &SourceText::new(SourceId::new(1), PROVENANCE_FIXTURE.to_owned()),
            6,
            9,
        );
    }
    assert!(validate_freeze(&overlapping).is_err());

    let mut wrong_literal = good.clone();
    if let Token::Characters(contribution) = &mut wrong_literal.lexed.tokens[1] {
        assert!(matches!(contribution.origin, Origin::Literal));
        contribution.interpreted = "x".to_owned();
    }
    assert_eq!(
        validate_freeze(&wrong_literal),
        Err(FreezeError::LiteralInterpretationMismatch)
    );

    let mut wrong_value = good.clone();
    for token in &mut wrong_value.lexed.tokens {
        if let Token::Characters(contribution) = token
            && matches!(contribution.origin, Origin::ResolvedNamed { .. })
        {
            contribution.interpreted = "<".to_owned();
        }
    }
    assert_eq!(
        validate_freeze(&wrong_value),
        Err(FreezeError::NamedValueMismatch)
    );

    let mut fabricated_tree = good.clone();
    {
        let node = &mut fabricated_tree
            .tree
            .body
            .as_mut()
            .expect("body")
            .text_nodes[0];
        node.contributions[0].interpreted = "x".to_owned();
        node.text = node
            .contributions
            .iter()
            .map(|c| c.interpreted.as_str())
            .collect();
    }
    assert_eq!(
        validate_freeze(&fabricated_tree),
        Err(FreezeError::TreeContributionNotFromLexed)
    );

    let mut omitted_tree = good.clone();
    {
        let node = &mut omitted_tree
            .tree
            .body
            .as_mut()
            .expect("body")
            .text_nodes[0];
        node.contributions.remove(1);
        node.text = node
            .contributions
            .iter()
            .map(|c| c.interpreted.as_str())
            .collect();
    }
    assert_eq!(
        validate_freeze(&omitted_tree),
        Err(FreezeError::TreeContributionNotFromLexed)
    );

    let mut divergent = good;
    divergent.tree.body.as_mut().expect("body").text_nodes[0].text = "a&&b".to_owned();
    assert_eq!(
        validate_freeze(&divergent),
        Err(FreezeError::TextNodeDoesNotEqualContributions)
    );
}

#[test]
fn c5_every_contribution_range_is_real_across_the_corpus() {
    for text in [
        "&",
        "&;",
        "&x",
        "&x;",
        "&amp;",
        "&amp",
        "&amp=",
        "&ampx;",
        "&notit;",
        "&notin;",
        "&NotEqualTilde;",
        "&bogus;",
        "&bogus",
        "&amp;lt;",
        "&lt;/body>",
        "&&amp;",
        "&not\u{0001}",
        "<body>a&amp;b</body>",
        "<body></body>&nbsp;",
    ] {
        let observation = run(text);
        for contribution in observation.lexed.contributions() {
            let span = &contribution.authored;
            assert_eq!(&text[span.start..span.end], span.raw);
            assert!(span.end <= observation.lexed.coverage_end);
        }
        // Authoritative coverage is monotonic and never exceeds the source.
        assert!(observation.lexed.coverage_end <= text.len());
        assert_eq!(observation.lexed.cursor, observation.lexed.coverage_end);
    }
}

#[test]
fn c6_gold_is_hand_authored_and_external_heads_are_markers_only() {
    for pin in [
        WHATWG_HEAD,
        WHATWG_SOURCE_BLOB,
        WPT_CHALLENGE_HEAD,
        HTML5LIB_CHALLENGE_HEAD,
    ] {
        assert_eq!(pin.len(), 40);
        assert!(pin.bytes().all(|b| b.is_ascii_hexdigit()));
    }
    // This module's source admits exactly the two source-identity primitives
    // and no production HTML path. Keep the guard independent of a particular
    // import spelling by checking both the exact import surface and normalized
    // fully-qualified paths.
    let this_file = include_str!("data_state_named_reference_successor_validation.rs");
    let imports: Vec<&str> = this_file
        .lines()
        .map(str::trim_start)
        .filter(|line| *line == "use" || line.starts_with("use "))
        .collect();
    assert_eq!(imports, ["use crate::{SourceId, SourceText};"]);

    let normalized: String = this_file
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect();
    for forbidden in [
        ["html", "::tokenizer"].concat(),
        ["tree_construction", "::driver"].concat(),
        ["tree_construction", "::session"].concat(),
        ["tree_construction", "::result"].concat(),
        ["super", "::driver"].concat(),
        ["super", "::session"].concat(),
        ["super", "::result"].concat(),
        ["super", "::super::tokenizer"].concat(),
        ["frontend_analysis", "_cli"].concat(),
    ] {
        assert!(
            !normalized.contains(&forbidden),
            "production path escaped the oracle boundary: {forbidden}"
        );
    }
}
