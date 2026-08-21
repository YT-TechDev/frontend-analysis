//! Selected source-backed ECMAScript lexical declaration slice for Issue #218.
//!
//! This module recognizes only the bounded top-level Script subset accepted by
//! #215/#218, the additive one-level selected Block frontier accepted by
//! #283/#291, and the distinct top-level VariableStatement frontier fixed by
//! #310. Recognition is transactional for the whole authoritative `SourceText`:
//! tentative declaration/binding/Block/var facts are returned only when the
//! entire source is consumed by selected items plus selected trivia.
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
    RecognizedOneLevelBlockSlice(SelectedOneLevelBlockScript),
    RecognizedVariableStatementSlice(SelectedVariableStatementScript),
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

#[derive(Debug)]
pub(super) struct SelectedOneLevelBlockScript {
    items: Vec<SelectedTopLevelItem>,
}

impl SelectedOneLevelBlockScript {
    pub(super) fn items(&self) -> &[SelectedTopLevelItem] {
        &self.items
    }
}

#[derive(Debug)]
pub(super) enum SelectedTopLevelItem {
    LexicalDeclaration(SelectedLexicalDeclaration),
    Block(SelectedBlock),
}

#[derive(Debug)]
pub(super) struct SelectedVariableStatementScript {
    items: Vec<SelectedVariableTopLevelItem>,
}

impl SelectedVariableStatementScript {
    pub(super) fn items(&self) -> &[SelectedVariableTopLevelItem] {
        &self.items
    }
}

#[derive(Debug)]
pub(super) enum SelectedVariableTopLevelItem {
    LexicalDeclaration(SelectedLexicalDeclaration),
    Block(SelectedBlock),
    VariableStatement(SelectedVariableStatement),
}

#[derive(Debug)]
pub(super) struct SelectedBlock {
    block: SourceAnchor,
    declarations: Vec<SelectedLexicalDeclaration>,
}

impl SelectedBlock {
    pub(super) fn block(&self) -> &SourceAnchor {
        &self.block
    }

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
pub(super) enum SelectedIdentifierReferenceNameState {
    Direct,
    Escaped { decoded: String },
}

#[derive(Debug)]
pub(super) struct SelectedIdentifierReferenceFact {
    reference: SourceAnchor,
    name_state: SelectedIdentifierReferenceNameState,
}

impl SelectedIdentifierReferenceFact {
    pub(super) fn reference(&self) -> &SourceAnchor {
        &self.reference
    }

    pub(super) fn name_state(&self) -> &SelectedIdentifierReferenceNameState {
        &self.name_state
    }

    pub(super) fn semantic_name(&self) -> &str {
        match &self.name_state {
            SelectedIdentifierReferenceNameState::Direct => self.reference.fragment(),
            SelectedIdentifierReferenceNameState::Escaped { decoded } => decoded.as_str(),
        }
    }
}

#[derive(Debug)]
pub(super) struct SelectedLexicalBinding {
    binding: SourceAnchor,
    name_state: SelectedBindingNameState,
    initializer: SelectedInitializerState,
    identifier_reference_initializer: Option<SelectedIdentifierReferenceFact>,
    escaped_reserved_initializer_identifier: Option<SourceAnchor>,
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

    pub(super) fn identifier_reference_initializer(
        &self,
    ) -> Option<&SelectedIdentifierReferenceFact> {
        self.identifier_reference_initializer.as_ref()
    }

    pub(super) fn escaped_reserved_initializer_identifier(&self) -> Option<&SourceAnchor> {
        self.escaped_reserved_initializer_identifier.as_ref()
    }
}

#[derive(Debug)]
pub(super) struct SelectedVariableBinding {
    binding: SourceAnchor,
    name_state: SelectedBindingNameState,
}

impl SelectedVariableBinding {
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
}

#[derive(Debug)]
pub(super) struct SelectedVariableStatement {
    binding: SelectedVariableBinding,
}

impl SelectedVariableStatement {
    pub(super) fn binding(&self) -> &SelectedVariableBinding {
        &self.binding
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

#[derive(Debug)]
enum SelectedScriptBuilder {
    Flat(Vec<SelectedLexicalDeclaration>),
    BlockEnabled(Vec<SelectedTopLevelItem>),
    VariableEnabled(Vec<SelectedVariableTopLevelItem>),
}

impl SelectedScriptBuilder {
    fn push_item(&mut self, item: SelectedTopLevelItem) -> Result<(), ParseFailure> {
        match (self, item) {
            (Self::Flat(declarations), SelectedTopLevelItem::LexicalDeclaration(declaration)) => {
                declarations
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                declarations.push(declaration);
                Ok(())
            }
            (builder @ Self::Flat(_), SelectedTopLevelItem::Block(block)) => {
                let Self::Flat(declarations) = builder else {
                    return Err(ParseFailure::InternalFailure);
                };
                let item_count = declarations
                    .len()
                    .checked_add(1)
                    .ok_or(ParseFailure::InternalFailure)?;
                let mut items = Vec::new();
                items
                    .try_reserve(item_count)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                for declaration in std::mem::take(declarations) {
                    items.push(SelectedTopLevelItem::LexicalDeclaration(declaration));
                }
                items.push(SelectedTopLevelItem::Block(block));
                *builder = Self::BlockEnabled(items);
                Ok(())
            }
            (Self::BlockEnabled(items), item) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(item);
                Ok(())
            }
            (Self::VariableEnabled(items), SelectedTopLevelItem::LexicalDeclaration(declaration)) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedVariableTopLevelItem::LexicalDeclaration(declaration));
                Ok(())
            }
            (Self::VariableEnabled(items), SelectedTopLevelItem::Block(block)) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedVariableTopLevelItem::Block(block));
                Ok(())
            }
        }
    }

    fn push_variable_statement(
        &mut self,
        statement: SelectedVariableStatement,
    ) -> Result<(), ParseFailure> {
        match self {
            builder @ Self::Flat(_) => {
                let Self::Flat(declarations) = builder else {
                    return Err(ParseFailure::InternalFailure);
                };
                let item_count = declarations
                    .len()
                    .checked_add(1)
                    .ok_or(ParseFailure::InternalFailure)?;
                let mut items = Vec::new();
                items
                    .try_reserve(item_count)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                for declaration in std::mem::take(declarations) {
                    items.push(SelectedVariableTopLevelItem::LexicalDeclaration(declaration));
                }
                items.push(SelectedVariableTopLevelItem::VariableStatement(statement));
                *builder = Self::VariableEnabled(items);
                Ok(())
            }
            builder @ Self::BlockEnabled(_) => {
                let Self::BlockEnabled(existing_items) = builder else {
                    return Err(ParseFailure::InternalFailure);
                };
                let item_count = existing_items
                    .len()
                    .checked_add(1)
                    .ok_or(ParseFailure::InternalFailure)?;
                let mut items = Vec::new();
                items
                    .try_reserve(item_count)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                for item in std::mem::take(existing_items) {
                    match item {
                        SelectedTopLevelItem::LexicalDeclaration(declaration) => items.push(
                            SelectedVariableTopLevelItem::LexicalDeclaration(declaration),
                        ),
                        SelectedTopLevelItem::Block(block) => {
                            items.push(SelectedVariableTopLevelItem::Block(block));
                        }
                    }
                }
                items.push(SelectedVariableTopLevelItem::VariableStatement(statement));
                *builder = Self::VariableEnabled(items);
                Ok(())
            }
            Self::VariableEnabled(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedVariableTopLevelItem::VariableStatement(statement));
                Ok(())
            }
        }
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

#[derive(Debug)]
enum SelectedIdentifierReferenceRecognition {
    Matched(SelectedIdentifierReferenceFact),
    EscapedReservedIdentifierName { identifier: SourceAnchor },
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

    fn parse_selected_block(&mut self) -> Result<SelectedBlock, ParseFailure> {
        let block_start = self.offset;
        if !self.consume_ascii('{') {
            return Err(ParseFailure::UnsupportedCoverage);
        }
        self.skip_selected_trivia();

        if self.peek_char() == Some('}') {
            return Err(ParseFailure::UnsupportedCoverage);
        }

        let mut declarations = Vec::new();
        loop {
            let declaration = self.parse_declaration()?;
            declarations
                .try_reserve(1)
                .map_err(|_| ParseFailure::ResourceLimited)?;
            declarations.push(declaration);

            self.skip_selected_trivia();
            if self.consume_ascii('}') {
                break;
            }
            if self.is_eof() {
                return Err(ParseFailure::UnsupportedCoverage);
            }
        }

        let block = self.anchor(block_start, self.offset)?;
        Ok(SelectedBlock {
            block,
            declarations,
        })
    }

    fn parse_variable_statement(&mut self) -> Result<SelectedVariableStatement, ParseFailure> {
        if !self.consume_keyword("var") {
            return Err(ParseFailure::UnsupportedCoverage);
        }

        let after_keyword = self.offset;
        self.skip_selected_trivia();
        let grammar_context = if self.offset == after_keyword {
            SelectedGrammarEvidenceContext::UnsupportedKeywordAdjacent
        } else {
            SelectedGrammarEvidenceContext::General
        };
        let (binding_start, binding_end, name_state) =
            self.parse_selected_binding_identifier(grammar_context)?;
        self.skip_selected_trivia();

        if !self.consume_ascii(';') {
            return Err(ParseFailure::UnsupportedCoverage);
        }

        let binding = self.anchor(binding_start, binding_end)?;
        Ok(SelectedVariableStatement {
            binding: SelectedVariableBinding {
                binding,
                name_state,
            },
        })
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
            let (
                initializer,
                identifier_reference_initializer,
                escaped_reserved_initializer_identifier,
                significant_end,
            ) = if self.consume_initializer_equals() {
                self.skip_selected_trivia();
                let mut identifier_reference_initializer = None;
                let mut escaped_reserved_initializer_identifier = None;
                if !self.consume_selected_decimal_integer()
                    && !self.consume_selected_boolean_literal()
                    && !self.consume_selected_null_literal()
                    && !self.consume_selected_this_expression()
                    && !self.consume_selected_escape_free_string_literal()
                {
                    match self.consume_selected_identifier_reference() {
                        SelectedIdentifierReferenceRecognition::Matched(reference) => {
                            identifier_reference_initializer = Some(reference);
                        }
                        SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName {
                            identifier,
                        } => {
                            escaped_reserved_initializer_identifier = Some(identifier);
                        }
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
                (
                    SelectedInitializerState::SelectedPresent,
                    identifier_reference_initializer,
                    escaped_reserved_initializer_identifier,
                    initializer_end,
                )
            } else {
                (SelectedInitializerState::Absent, None, None, binding_end)
            };

            let binding = self.anchor(binding_start, binding_end)?;
            bindings
                .try_reserve(1)
                .map_err(|_| ParseFailure::ResourceLimited)?;
            bindings.push(SelectedLexicalBinding {
                binding,
                name_state,
                initializer,
                identifier_reference_initializer,
                escaped_reserved_initializer_identifier,
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

    /// Recognizes exactly one direct-authored `NullLiteral` in the selected
    /// initializer position without widening into a general Literal owner.
    ///
    /// The shared keyword boundary preserves maximal IdentifierName routing for
    /// direct IdentifierPart and formed authored UES continuations. Malformed or
    /// non-CodePoint UES tails remain owned only by the enclosing whole-source
    /// transaction; this helper retains no local commit policy as domain state.
    fn consume_selected_null_literal(&mut self) -> bool {
        self.consume_keyword("null")
    }

    /// Recognizes exactly one direct-authored `PrimaryExpression : this` in the
    /// selected initializer position without widening into a generic
    /// `PrimaryExpression` or `Expression` owner.
    ///
    /// The shared keyword boundary preserves maximal IdentifierName routing for
    /// direct IdentifierPart and formed authored UES continuations. Malformed or
    /// non-CodePoint UES tails remain owned only by the enclosing whole-source
    /// transaction; this helper retains no local commit policy as domain state.
    /// Runtime `this` Evaluation and `ResolveThisBinding()` are outside this
    /// source-recognition slice.
    fn consume_selected_this_expression(&mut self) -> bool {
        self.consume_keyword("this")
    }

    /// Recognizes exactly one direct-authored, escape-free `StringLiteral` in
    /// the selected initializer position without retaining a `StringValue` or
    /// widening into a generic Literal / PrimaryExpression / Expression owner.
    ///
    /// The matching authored quote terminates the selected atom. Reverse solidus,
    /// raw LF, raw CR, or EOF before that matching quote causes this helper to
    /// restore its starting cursor and decline. ES2026 direct LS / PS characters
    /// remain ordinary direct string content as fixed by #257.
    fn consume_selected_escape_free_string_literal(&mut self) -> bool {
        let start = self.offset;
        let Some(quote) = self.peek_char() else {
            return false;
        };
        if !matches!(quote, '"' | '\'') {
            return false;
        }
        let _ = self.advance_char();

        loop {
            match self.peek_char() {
                Some(next) if next == quote => {
                    let _ = self.advance_char();
                    return true;
                }
                Some('\\' | '\n' | '\r') | None => {
                    self.offset = start;
                    return false;
                }
                Some(_) => {
                    let _ = self.advance_char();
                }
            }
        }
    }

    /// Recognizes one selected `IdentifierReference` atom in the fixed
    /// non-strict Script envelope (`Yield=false`, `Await=false`).
    ///
    /// This is a position-specific recognizer, not a general ECMAScript
    /// Identifier abstraction. It scans one maximal direct/escaped
    /// `IdentifierName` with local offsets and commits `self.offset` only after
    /// the complete name is selected. Direct-only spelling stays source-backed
    /// and allocation-free. Once a formed, position-valid authored escape
    /// appears, the exact decoded semantic name is retained only for the first
    /// Binding / Scope consumer fixed by #270/#273.
    ///
    /// Direct authored `yield` / `await` and escaped Identifier spellings that
    /// decode to those names use different grammar routes, as fixed by #242.
    /// This fixed context selects both routes; route identity is not retained.
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
            if decoded.is_none() {
                return SelectedIdentifierReferenceRecognition::NotSelected;
            }

            let identifier = match self.anchor(start, end) {
                Ok(identifier) => identifier,
                Err(_) => return SelectedIdentifierReferenceRecognition::InternalFailure,
            };
            self.offset = end;
            return SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName {
                identifier,
            };
        }

        let reference = match self.anchor(start, end) {
            Ok(reference) => reference,
            Err(_) => return SelectedIdentifierReferenceRecognition::InternalFailure,
        };
        let name_state = match decoded {
            Some(decoded) => SelectedIdentifierReferenceNameState::Escaped { decoded },
            None => SelectedIdentifierReferenceNameState::Direct,
        };

        self.offset = end;
        SelectedIdentifierReferenceRecognition::Matched(SelectedIdentifierReferenceFact {
            reference,
            name_state,
        })
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

fn parse_failure_to_outcome(failure: ParseFailure) -> SelectedLexicalSliceOutcome {
    match failure {
        ParseFailure::UnsupportedCoverage => SelectedLexicalSliceOutcome::UnsupportedCoverage,
        ParseFailure::DefinitiveGrammarRejectionEvidence { subject } => {
            SelectedLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence { subject }
        }
        ParseFailure::ResourceLimited => SelectedLexicalSliceOutcome::ResourceLimited,
        ParseFailure::InternalFailure => SelectedLexicalSliceOutcome::InternalFailure,
    }
}

pub(super) fn recognize_selected_lexical_slice(source: &SourceText) -> SelectedLexicalSliceOutcome {
    let mut cursor = Cursor::new(source);
    cursor.skip_selected_trivia();

    if cursor.is_eof() {
        return SelectedLexicalSliceOutcome::UnsupportedCoverage;
    }

    let mut builder = SelectedScriptBuilder::Flat(Vec::new());

    loop {
        if cursor.remaining().starts_with("var") {
            let statement = match cursor.parse_variable_statement() {
                Ok(statement) => statement,
                Err(failure) => return parse_failure_to_outcome(failure),
            };
            if let Err(failure) = builder.push_variable_statement(statement) {
                return parse_failure_to_outcome(failure);
            }
        } else {
            let item = if cursor.peek_char() == Some('{') {
                match cursor.parse_selected_block() {
                    Ok(block) => SelectedTopLevelItem::Block(block),
                    Err(failure) => return parse_failure_to_outcome(failure),
                }
            } else {
                match cursor.parse_declaration() {
                    Ok(declaration) => SelectedTopLevelItem::LexicalDeclaration(declaration),
                    Err(failure) => return parse_failure_to_outcome(failure),
                }
            };

            if let Err(failure) = builder.push_item(item) {
                return parse_failure_to_outcome(failure);
            }
        }

        cursor.skip_selected_trivia();
        if cursor.is_eof() {
            break;
        }
    }

    match builder {
        SelectedScriptBuilder::Flat(declarations) => {
            SelectedLexicalSliceOutcome::RecognizedSelectedSlice(SelectedLexicalScript {
                declarations,
            })
        }
        SelectedScriptBuilder::BlockEnabled(items) => {
            SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(SelectedOneLevelBlockScript {
                items,
            })
        }
        SelectedScriptBuilder::VariableEnabled(items) => {
            SelectedLexicalSliceOutcome::RecognizedVariableStatementSlice(
                SelectedVariableStatementScript { items },
            )
        }
    }
}

fn is_selected_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}
