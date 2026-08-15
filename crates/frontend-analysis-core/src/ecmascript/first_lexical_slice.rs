//! First source-backed ECMAScript lexical declaration slice for Issue #204.
//!
//! This module recognizes only the bounded top-level Script subset accepted by
//! #203/#204. Recognition is transactional for the whole authoritative
//! `SourceText`: tentative declaration facts are returned only when the entire
//! source is consumed by selected declarations plus selected trivia.
//!
//! This is not aggregate ECMAScript qualification and cannot construct
//! `QualificationOutcome::Qualified`.

use crate::{SourceAnchor, SourceText};

use super::unicode::{is_id_continue, is_id_start, is_space_separator};

/// Slice-local processing result.
///
/// `DefinitiveGrammarRejectionEvidence` is retained as an explicit future
/// meaning boundary, but this first implementation does not construct it. A
/// selected-grammar failure is conservatively `UnsupportedCoverage` unless a
/// future leaf adds an independently reviewed full-Script invalidity proof.
/// `ResourceLimited` is reserved for an actual fallible retained-evidence
/// allocation, not an invented numeric parser budget.
#[derive(Debug)]
pub(super) enum FirstLexicalSliceOutcome {
    RecognizedSelectedSlice(SelectedLexicalScript),
    UnsupportedCoverage,
    DefinitiveGrammarRejectionEvidence,
    ResourceLimited,
    InternalFailure,
}

/// Minimal retained facts for the selected whole-source Script slice.
#[derive(Debug)]
pub(super) struct SelectedLexicalScript {
    declarations: Vec<LexicalDeclarationFact>,
}

impl SelectedLexicalScript {
    pub(super) fn declarations(&self) -> &[LexicalDeclarationFact] {
        &self.declarations
    }
}

/// Source-backed syntax facts required by the next named static-semantic owner.
#[derive(Debug)]
pub(super) struct LexicalDeclarationFact {
    declaration: SourceAnchor,
    binding: SourceAnchor,
    semicolon: SourceAnchor,
}

impl LexicalDeclarationFact {
    pub(super) fn declaration(&self) -> &SourceAnchor {
        &self.declaration
    }

    pub(super) fn binding(&self) -> &SourceAnchor {
        &self.binding
    }

    pub(super) fn semicolon(&self) -> &SourceAnchor {
        &self.semicolon
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseFailure {
    UnsupportedCoverage,
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

    fn parse_declaration(&mut self) -> Result<LexicalDeclarationFact, ParseFailure> {
        let declaration_start = self.offset;

        if !self.consume_let_keyword() {
            return Err(ParseFailure::UnsupportedCoverage);
        }

        self.skip_selected_trivia();
        let (binding_start, binding_end) = self
            .parse_unescaped_binding_identifier()
            .ok_or(ParseFailure::UnsupportedCoverage)?;

        self.skip_selected_trivia();
        if self.consume_initializer_equals() {
            self.skip_selected_trivia();
            if !self.consume_selected_decimal_integer() {
                return Err(ParseFailure::UnsupportedCoverage);
            }
            self.skip_selected_trivia();
        }

        let semicolon_start = self.offset;
        if !self.consume_ascii(';') {
            return Err(ParseFailure::UnsupportedCoverage);
        }
        let semicolon_end = self.offset;

        let declaration = self.anchor(declaration_start, semicolon_end)?;
        let binding = self.anchor(binding_start, binding_end)?;
        let semicolon = self.anchor(semicolon_start, semicolon_end)?;

        Ok(LexicalDeclarationFact {
            declaration,
            binding,
            semicolon,
        })
    }

    fn consume_let_keyword(&mut self) -> bool {
        const LET: &str = "let";

        if !self.remaining().starts_with(LET) {
            return false;
        }

        let after_let = self.offset + LET.len();
        if let Some(next) = self.text[after_let..].chars().next() {
            // A following IdentifierPart or Unicode escape introducer means
            // `let` is only a prefix of an IdentifierName, not the keyword.
            if is_identifier_part(next) || next == '\\' {
                return false;
            }
        }

        self.offset = after_let;
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

        // An escape would be part of the same IdentifierName in the full
        // lexical grammar. Escaped identifiers are deferred by this Leaf, so
        // do not expose the unescaped prefix as a selected binding.
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

        // `=`, `==`, `===`, and `=>` are distinct punctuator tokens. The
        // selected initializer owns only the single `=` token and must not
        // split a longer punctuator prefix before reporting unsupported
        // coverage.
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

/// Recognizes the first bounded source-backed lexical declaration Script slice.
///
/// The authoritative source is accepted only when every non-trivia byte belongs
/// to one or more selected `let` lexical declarations. No tentative fact escapes
/// on whole-source coverage failure.
pub(super) fn recognize_first_lexical_slice(source: &SourceText) -> FirstLexicalSliceOutcome {
    let mut cursor = Cursor::new(source);
    cursor.skip_selected_trivia();

    if cursor.is_eof() {
        return FirstLexicalSliceOutcome::UnsupportedCoverage;
    }

    let mut declarations = Vec::new();

    loop {
        match cursor.parse_declaration() {
            Ok(declaration) => {
                if declarations.try_reserve(1).is_err() {
                    return FirstLexicalSliceOutcome::ResourceLimited;
                }
                declarations.push(declaration);
            }
            Err(ParseFailure::UnsupportedCoverage) => {
                return FirstLexicalSliceOutcome::UnsupportedCoverage;
            }
            Err(ParseFailure::InternalFailure) => {
                return FirstLexicalSliceOutcome::InternalFailure;
            }
        }

        cursor.skip_selected_trivia();
        if cursor.is_eof() {
            break;
        }
    }

    FirstLexicalSliceOutcome::RecognizedSelectedSlice(SelectedLexicalScript { declarations })
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
    // ES2026 ReservedWord except `await` and `yield`, which are explicitly
    // permitted by BindingIdentifier[~Yield, ~Await] in this top-level Script
    // context. `let` is not in ReservedWord and remains a syntactically
    // recognizable Identifier here; its LexicalDeclaration Early Error belongs
    // to the later static-semantic owner.
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
