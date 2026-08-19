//! Selected source-backed ECMAScript lexical declaration slice for Issue #218.
//!
//! This module recognizes only the bounded top-level Script subset accepted by
//! #215/#218. Recognition is transactional for the whole authoritative
//! `SourceText`: tentative declaration/binding facts are returned only when the
//! entire source is consumed by selected declarations plus selected trivia.
//!
//! This is not aggregate ECMAScript qualification and cannot construct
//! `QualificationOutcome::Qualified`.

use crate::{SourceAnchor, SourceText};

use super::selected_binding_identifier::{
    formed_unicode_escape_at, is_selected_identifier_part, is_selected_identifier_start,
    is_unconditionally_reserved_word, selected_grammar_escape_subject_end,
    selected_keyword_adjacent_grammar_escape_subject_end,
};
use super::unicode::is_space_separator;

#[derive(Debug)]
pub(super) enum SelectedLexicalSliceOutcome {
    RecognizedSelectedSlice(SelectedLexicalScript),
    UnsupportedCoverage,
    DefinitiveGrammarRejectionEvidence { subject: SourceAnchor },
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug)]
pub(super) struct SelectedLexicalScript {
    declarations: Vec<SelectedLexicalDeclaration>,
}

impl SelectedLexicalScript {
    pub(super) fn declarations(&self) -> &[SelectedLexicalDeclaration] {
        &self.declarations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedLexicalDeclarationKind {
    Let,
    Const,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedInitializerState {
    Absent,
    SelectedPresent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedInvalidEscapePosition {
    Start,
    Part,
}

#[derive(Debug)]
pub(super) enum SelectedBindingNameState {
    Unescaped,
    EscapedValid {
        decoded: String,
    },
    InvalidEscapedPosition {
        position: SelectedInvalidEscapePosition,
        escape: SourceAnchor,
    },
}

#[derive(Debug)]
pub(super) struct SelectedLexicalBinding {
    binding: SourceAnchor,
    name_state: SelectedBindingNameState,
    initializer: SelectedInitializerState,
}

impl SelectedLexicalBinding {
    pub(super) fn binding(&self) -> &SourceAnchor {
        &self.binding
    }

    pub(super) fn name_state(&self) -> &SelectedBindingNameState {
        &self.name_state
    }

    pub(super) fn semantic_name(&self) -> Option<&str> {
        match &self.name_state {
            SelectedBindingNameState::Unescaped => Some(self.binding.fragment()),
            SelectedBindingNameState::EscapedValid { decoded } => Some(decoded.as_str()),
            SelectedBindingNameState::InvalidEscapedPosition { .. } => None,
        }
    }

    pub(super) fn initializer(&self) -> SelectedInitializerState {
        self.initializer
    }
}

#[derive(Debug)]
pub(super) enum SelectedDeclarationTerminator {
    AuthoredSemicolon(SourceAnchor),
    AutomaticAtEof,
}

#[derive(Debug)]
pub(super) struct SelectedLexicalDeclaration {
    kind: SelectedLexicalDeclarationKind,
    declaration: SourceAnchor,
    bindings: Vec<SelectedLexicalBinding>,
    terminator: SelectedDeclarationTerminator,
}

impl SelectedLexicalDeclaration {
    pub(super) fn kind(&self) -> SelectedLexicalDeclarationKind {
        self.kind
    }

    pub(super) fn declaration(&self) -> &SourceAnchor {
        &self.declaration
    }

    pub(super) fn bindings(&self) -> &[SelectedLexicalBinding] {
        &self.bindings
    }

    pub(super) fn terminator(&self) -> &SelectedDeclarationTerminator {
        &self.terminator
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedGrammarEvidenceContext {
    General,
    KeywordAdjacentLet,
    UnsupportedKeywordAdjacent,
}

#[derive(Debug)]
enum ParseFailure {
    UnsupportedCoverage,
    DefinitiveGrammarRejectionEvidence { subject: SourceAnchor },
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedIdentifierReferenceRecognition {
    Matched,
    NotSelected,
    ResourceLimited,
    InternalFailure,
}

struct Cursor<'source> {
    source: &'source SourceText,
    text: &'source str,
    offset: usize,
}

impl<'source> Cursor<'source> {
    fn new(source: &'source SourceText) -> Self {
        Self {
            source,
            text: source.as_str(),
            offset: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.offset == self.text.len()
    }

    fn remaining(&self) -> &'source str {
        &self.text[self.offset..]
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let next = self.peek_char()?;
        self.offset += next.len_utf8();
        Some(next)
    }

    fn skip_selected_trivia(&mut self) {
        while let Some(next) = self.peek_char() {
            if !is_selected_trivia(next) {
                break;
            }
            let _ = self.advance_char();
        }
    }

    fn parse_declaration(&mut self) -> Result<SelectedLexicalDeclaration, ParseFailure> {
        let declaration_start = self.offset;
        let kind = self
            .consume_declaration_kind()
            .ok_or(ParseFailure::UnsupportedCoverage)?;

        let after_keyword = self.offset;
        self.skip_selected_trivia();
        let first_binding_is_keyword_adjacent = self.offset == after_keyword;
        let mut bindings = Vec::new();
        let mut first_binding = true;

        let final_significant_end = loop {
            let grammar_context = if first_binding && first_binding_is_keyword_adjacent {
                match kind {
                    SelectedLexicalDeclarationKind::Let => {
                        SelectedGrammarEvidenceContext::KeywordAdjacentLet
                    }
                    SelectedLexicalDeclarationKind::Const => {
                        SelectedGrammarEvidenceContext::UnsupportedKeywordAdjacent
                    }
                }
            } else {
                SelectedGrammarEvidenceContext::General
            };

            let (binding_start, binding_end, name_state) =
                self.parse_selected_binding_identifier(grammar_context)?;

            self.skip_selected_trivia();
            let (initializer, significant_end) = if self.consume_initializer_equals() {
                self.skip_selected_trivia();
                if !self.consume_selected_decimal_integer()
                    && !self.consume_selected_boolean_literal()
                {
                    match self.consume_selected_identifier_reference() {
                        SelectedIdentifierReferenceRecognition::Matched => {}
                        SelectedIdentifierReferenceRecognition::NotSelected => {
                            return Err(ParseFailure::UnsupportedCoverage);
                        }
                        SelectedIdentifierReferenceRecognition::ResourceLimited => {
                            return Err(ParseFailure::ResourceLimited);
                        }
                        SelectedIdentifierReferenceRecognition::InternalFailure => {
                            return Err(ParseFailure::InternalFailure);
                        }
                    }
                }
                let initializer_end = self.offset;
                self.skip_selected_trivia();
                (SelectedInitializerState::SelectedPresent, initializer_end)
            } else {
                (SelectedInitializerState::Absent, binding_end)
            };

            let binding = self.anchor(binding_start, binding_end)?;
            bindings
                .try_reserve(1)
                .map_err(|_| ParseFailure::ResourceLimited)?;
            bindings.push(SelectedLexicalBinding {
                binding,
                name_state,
                initializer,
            });
            first_binding = false;

            if self.consume_ascii(',') {
                self.skip_selected_trivia();
                continue;
            }
            break significant_end;
        };

        let semicolon_start = self.offset;
        let (declaration_end, terminator) = if self.consume_ascii(';') {
            let semicolon_end = self.offset;
            (
                semicolon_end,
                SelectedDeclarationTerminator::AuthoredSemicolon(
                    self.anchor(semicolon_start, semicolon_end)?,
                ),
            )
        } else if self.is_eof() {
            (
                final_significant_end,
                SelectedDeclarationTerminator::AutomaticAtEof,
            )
        } else {
            return Err(ParseFailure::UnsupportedCoverage);
        };

        let declaration = self.anchor(declaration_start, declaration_end)?;

        Ok(SelectedLexicalDeclaration {
            kind,
            declaration,
            bindings,
            terminator,
        })
    }

    fn consume_declaration_kind(&mut self) -> Option<SelectedLexicalDeclarationKind> {
        if self.consume_keyword("let") {
            return Some(SelectedLexicalDeclarationKind::Let);
        }
        if self.consume_keyword("const") {
            return Some(SelectedLexicalDeclarationKind::Const);
        }
        None
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        if !self.remaining().starts_with(keyword) {
            return false;
        }

        let after_keyword = self.offset + keyword.len();
        if let Some(next) = self.text[after_keyword..].chars().next()
            && (is_selected_identifier_part(next as u32)
                || (next == '\\' && formed_unicode_escape_at(self.text, after_keyword).is_some()))
        {
            return false;
        }

        self.offset = after_keyword;
        true
    }

    fn parse_selected_binding_identifier(
        &mut self,
        grammar_context: SelectedGrammarEvidenceContext,
    ) -> Result<(usize, usize, SelectedBindingNameState), ParseFailure> {
        let start = self.offset;
        let mut first_element = true;
        let mut saw_escape = false;
        let mut decoded: Option<String> = None;
        let mut first_invalid: Option<(SelectedInvalidEscapePosition, SourceAnchor)> = None;

        loop {
            if self.peek_char() == Some('\\') {
                let escape_start = self.offset;
                let Some(formation) = formed_unicode_escape_at(self.text, escape_start) else {
                    let grammar_end = if first_element {
                        match grammar_context {
                            SelectedGrammarEvidenceContext::General => {
                                selected_grammar_escape_subject_end(self.text, escape_start)
                            }
                            SelectedGrammarEvidenceContext::KeywordAdjacentLet => {
                                selected_keyword_adjacent_grammar_escape_subject_end(
                                    self.text,
                                    escape_start,
                                )
                            }
                            SelectedGrammarEvidenceContext::UnsupportedKeywordAdjacent => None,
                        }
                    } else {
                        selected_grammar_escape_subject_end(self.text, escape_start)
                    };

                    let Some(grammar_end) = grammar_end else {
                        return Err(ParseFailure::UnsupportedCoverage);
                    };
                    let subject = self.anchor(escape_start, grammar_end)?;
                    return Err(ParseFailure::DefinitiveGrammarRejectionEvidence { subject });
                };

                self.offset = formation.end;
                saw_escape = true;

                let position = if first_element {
                    SelectedInvalidEscapePosition::Start
                } else {
                    SelectedInvalidEscapePosition::Part
                };
                let valid_position = match position {
                    SelectedInvalidEscapePosition::Start => {
                        is_selected_identifier_start(formation.code_point)
                    }
                    SelectedInvalidEscapePosition::Part => {
                        is_selected_identifier_part(formation.code_point)
                    }
                };

                if !valid_position {
                    if first_invalid.is_none() {
                        first_invalid = Some((position, self.anchor(escape_start, formation.end)?));
                        decoded = None;
                    }
                } else if first_invalid.is_none() {
                    if decoded.is_none() {
                        let prefix = &self.text[start..escape_start];
                        let mut name = String::new();
                        name.try_reserve(prefix.len())
                            .map_err(|_| ParseFailure::ResourceLimited)?;
                        name.push_str(prefix);
                        decoded = Some(name);
                    }

                    let scalar = char::from_u32(formation.code_point)
                        .ok_or(ParseFailure::InternalFailure)?;
                    let name = decoded.as_mut().ok_or(ParseFailure::InternalFailure)?;
                    name.try_reserve(scalar.len_utf8())
                        .map_err(|_| ParseFailure::ResourceLimited)?;
                    name.push(scalar);
                }

                first_element = false;
                continue;
            }

            let Some(next) = self.peek_char() else {
                break;
            };
            let valid_position = if first_element {
                is_selected_identifier_start(next as u32)
            } else {
                is_selected_identifier_part(next as u32)
            };
            if !valid_position {
                if first_element {
                    return Err(ParseFailure::UnsupportedCoverage);
                }
                break;
            }

            let _ = self.advance_char();
            if first_invalid.is_none()
                && let Some(name) = decoded.as_mut()
            {
                name.try_reserve(next.len_utf8())
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                name.push(next);
            }
            first_element = false;
        }

        if first_element {
            return Err(ParseFailure::UnsupportedCoverage);
        }

        let end = self.offset;
        let name_state = if let Some((position, escape)) = first_invalid {
            SelectedBindingNameState::InvalidEscapedPosition { position, escape }
        } else if saw_escape {
            SelectedBindingNameState::EscapedValid {
                decoded: decoded.ok_or(ParseFailure::InternalFailure)?,
            }
        } else {
            let spelling = &self.text[start..end];
            if is_unconditionally_reserved_word(spelling) {
                return Err(ParseFailure::UnsupportedCoverage);
            }
            SelectedBindingNameState::Unescaped
        };

        Ok((start, end, name_state))
    }

    /// Recognizes exactly one direct-authored `BooleanLiteral` in the selected
    /// initializer position without widening into a general Literal owner.
    ///
    /// Reusing the existing keyword boundary prevents a direct `true` / `false`
    /// prefix from committing when a direct IdentifierPart or formed authored
    /// Unicode escape continues the same maximal IdentifierName. Malformed UES
    /// continuation remains a whole-source transaction concern; no local commit
    /// policy for that class is retained as domain state.
    fn consume_selected_boolean_literal(&mut self) -> bool {
        self.consume_keyword("true") || self.consume_keyword("false")
    }

    /// Recognizes one selected `IdentifierReference` atom in the fixed
    /// non-strict Script envelope (`Yield=false`, `Await=false`).
    ///
    /// This is a position-specific recognizer, not a general ECMAScript
    /// Identifier abstraction. It scans one maximal direct/escaped
    /// `IdentifierName` with local offsets and commits `self.offset` only after
    /// the complete name is selected. Direct-only spelling stays borrowed and
    /// allocation-free. Once a formed, position-valid authored escape appears,
    /// a temporary decoded `StringValue` is built only for complete-name policy
    /// and discarded after recognition.
    ///
    /// Direct authored `yield` / `await` and escaped Identifier spellings that
    /// decode to those names use different grammar routes, as fixed by #242.
    /// This fixed context selects both routes, so the production domain model
    /// needs only successful applicability and retains no route state.
    fn consume_selected_identifier_reference(&mut self) -> SelectedIdentifierReferenceRecognition {
        let start = self.offset;
        let mut end = start;
        let mut first_element = true;
        let mut decoded: Option<String> = None;

        loop {
            if self.text.as_bytes().get(end) == Some(&b'\\') {
                let escape_start = end;
                let Some(formation) = formed_unicode_escape_at(self.text, escape_start) else {
                    return SelectedIdentifierReferenceRecognition::NotSelected;
                };

                let valid_position = if first_element {
                    is_selected_identifier_start(formation.code_point)
                } else {
                    is_selected_identifier_part(formation.code_point)
                };
                if !valid_position {
                    return SelectedIdentifierReferenceRecognition::NotSelected;
                }

                let Some(scalar) = char::from_u32(formation.code_point) else {
                    return SelectedIdentifierReferenceRecognition::InternalFailure;
                };

                if decoded.is_none() {
                    let prefix = &self.text[start..escape_start];
                    let mut name = String::new();
                    if name.try_reserve(prefix.len()).is_err() {
                        return SelectedIdentifierReferenceRecognition::ResourceLimited;
                    }
                    name.push_str(prefix);
                    decoded = Some(name);
                }

                let Some(name) = decoded.as_mut() else {
                    return SelectedIdentifierReferenceRecognition::InternalFailure;
                };
                if name.try_reserve(scalar.len_utf8()).is_err() {
                    return SelectedIdentifierReferenceRecognition::ResourceLimited;
                }
                name.push(scalar);
                end = formation.end;
                first_element = false;
                continue;
            }

            let Some(next) = self.text[end..].chars().next() else {
                break;
            };
            let valid_position = if first_element {
                is_selected_identifier_start(next as u32)
            } else {
                is_selected_identifier_part(next as u32)
            };
            if !valid_position {
                if first_element {
                    return SelectedIdentifierReferenceRecognition::NotSelected;
                }
                break;
            }

            end += next.len_utf8();
            if let Some(name) = decoded.as_mut() {
                if name.try_reserve(next.len_utf8()).is_err() {
                    return SelectedIdentifierReferenceRecognition::ResourceLimited;
                }
                name.push(next);
            }
            first_element = false;
        }

        if first_element {
            return SelectedIdentifierReferenceRecognition::NotSelected;
        }

        let semantic_name = decoded.as_deref().unwrap_or_else(|| &self.text[start..end]);
        if is_unconditionally_reserved_word(semantic_name) {
            return SelectedIdentifierReferenceRecognition::NotSelected;
        }

        self.offset = end;
        SelectedIdentifierReferenceRecognition::Matched
    }

    fn consume_initializer_equals(&mut self) -> bool {
        if !self.remaining().starts_with('=') {
            return false;
        }

        if self.remaining().starts_with("==") || self.remaining().starts_with("=>") {
            return false;
        }

        self.offset += 1;
        true
    }

    fn consume_selected_decimal_integer(&mut self) -> bool {
        let start = self.offset;
        let bytes = self.text.as_bytes();

        let Some(first) = bytes.get(self.offset).copied() else {
            return false;
        };

        match first {
            b'0' => {
                self.offset += 1;
                if matches!(bytes.get(self.offset), Some(next) if next.is_ascii_digit()) {
                    self.offset = start;
                    return false;
                }
            }
            b'1'..=b'9' => {
                self.offset += 1;
                while matches!(bytes.get(self.offset), Some(next) if next.is_ascii_digit()) {
                    self.offset += 1;
                }
            }
            _ => return false,
        }

        true
    }

    fn consume_ascii(&mut self, expected: char) -> bool {
        debug_assert!(expected.is_ascii());
        if self.peek_char() == Some(expected) {
            self.offset += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn anchor(&self, start: usize, end: usize) -> Result<SourceAnchor, ParseFailure> {
        self.source
            .anchor(start, end)
            .map_err(|_| ParseFailure::InternalFailure)
    }
}

pub(super) fn recognize_selected_lexical_slice(source: &SourceText) -> SelectedLexicalSliceOutcome {
    let mut cursor = Cursor::new(source);
    cursor.skip_selected_trivia();

    if cursor.is_eof() {
        return SelectedLexicalSliceOutcome::UnsupportedCoverage;
    }

    let mut declarations = Vec::new();

    loop {
        match cursor.parse_declaration() {
            Ok(declaration) => {
                if declarations.try_reserve(1).is_err() {
                    return SelectedLexicalSliceOutcome::ResourceLimited;
                }
                declarations.push(declaration);
            }
            Err(ParseFailure::UnsupportedCoverage) => {
                return SelectedLexicalSliceOutcome::UnsupportedCoverage;
            }
            Err(ParseFailure::DefinitiveGrammarRejectionEvidence { subject }) => {
                return SelectedLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence { subject };
            }
            Err(ParseFailure::ResourceLimited) => {
                return SelectedLexicalSliceOutcome::ResourceLimited;
            }
            Err(ParseFailure::InternalFailure) => {
                return SelectedLexicalSliceOutcome::InternalFailure;
            }
        }

        cursor.skip_selected_trivia();
        if cursor.is_eof() {
            break;
        }
    }

    SelectedLexicalSliceOutcome::RecognizedSelectedSlice(SelectedLexicalScript { declarations })
}

fn is_selected_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}
