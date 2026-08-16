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

use super::unicode::{is_id_continue, is_id_start, is_space_separator};

#[derive(Debug)]
pub(super) enum SelectedLexicalSliceOutcome {
    RecognizedSelectedSlice(SelectedLexicalScript),
    UnsupportedCoverage,
    DefinitiveGrammarRejectionEvidence,
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

#[derive(Debug)]
pub(super) struct SelectedLexicalBinding {
    binding: SourceAnchor,
    initializer: SelectedInitializerState,
}

impl SelectedLexicalBinding {
    pub(super) fn binding(&self) -> &SourceAnchor {
        &self.binding
    }

    pub(super) fn initializer(&self) -> SelectedInitializerState {
        self.initializer
    }
}

#[derive(Debug)]
pub(super) struct SelectedLexicalDeclaration {
    kind: SelectedLexicalDeclarationKind,
    declaration: SourceAnchor,
    bindings: Vec<SelectedLexicalBinding>,
    semicolon: SourceAnchor,
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

    pub(super) fn semicolon(&self) -> &SourceAnchor {
        &self.semicolon
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseFailure {
    UnsupportedCoverage,
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

        self.skip_selected_trivia();
        let mut bindings = Vec::new();

        loop {
            let (binding_start, binding_end) = self
                .parse_unescaped_binding_identifier()
                .ok_or(ParseFailure::UnsupportedCoverage)?;

            self.skip_selected_trivia();
            let initializer = if self.consume_initializer_equals() {
                self.skip_selected_trivia();
                if !self.consume_selected_decimal_integer() {
                    return Err(ParseFailure::UnsupportedCoverage);
                }
                self.skip_selected_trivia();
                SelectedInitializerState::SelectedPresent
            } else {
                SelectedInitializerState::Absent
            };

            let binding = self.anchor(binding_start, binding_end)?;
            bindings
                .try_reserve(1)
                .map_err(|_| ParseFailure::ResourceLimited)?;
            bindings.push(SelectedLexicalBinding {
                binding,
                initializer,
            });

            if self.consume_ascii(',') {
                self.skip_selected_trivia();
                continue;
            }
            break;
        }

        let semicolon_start = self.offset;
        if !self.consume_ascii(';') {
            return Err(ParseFailure::UnsupportedCoverage);
        }
        let semicolon_end = self.offset;

        let declaration = self.anchor(declaration_start, semicolon_end)?;
        let semicolon = self.anchor(semicolon_start, semicolon_end)?;

        Ok(SelectedLexicalDeclaration {
            kind,
            declaration,
            bindings,
            semicolon,
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
        if let Some(next) = self.text[after_keyword..].chars().next() {
            if is_identifier_part(next) || next == '\\' {
                return false;
            }
        }

        self.offset = after_keyword;
        true
    }

    fn parse_unescaped_binding_identifier(&mut self) -> Option<(usize, usize)> {
        let start = self.offset;
        let first = self.peek_char()?;
        if !is_identifier_start(first) {
            return None;
        }
        let _ = self.advance_char();

        while let Some(next) = self.peek_char() {
            if !is_identifier_part(next) {
                break;
            }
            let _ = self.advance_char();
        }

        if self.peek_char() == Some('\\') {
            return None;
        }

        let end = self.offset;
        let spelling = &self.text[start..end];
        if is_unconditionally_reserved_word(spelling) {
            return None;
        }

        Some((start, end))
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

fn is_identifier_start(code_point: char) -> bool {
    code_point == '$' || code_point == '_' || is_id_start(code_point as u32)
}

fn is_identifier_part(code_point: char) -> bool {
    code_point == '$' || is_id_continue(code_point as u32)
}

fn is_unconditionally_reserved_word(spelling: &str) -> bool {
    matches!(
        spelling,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
    )
}
