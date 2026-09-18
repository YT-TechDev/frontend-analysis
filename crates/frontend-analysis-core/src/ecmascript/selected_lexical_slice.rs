//! Selected source-backed ECMAScript lexical declaration slice for Issue #218.
//!
//! This module recognizes only the bounded top-level Script subset accepted by
//! #215/#218, the additive one-level selected Block frontier accepted by
//! #283/#291, and the distinct top-level VariableStatement frontier fixed by
//! #310, widened to `1..N` declarators by #324, widened to optional selected
//! decimal-integer initializers by #328, widened to direct-authored selected
//! IdentifierReference initializers by #334, widened to selected escaped
//! non-ReservedWord IdentifierReference initializers by #338, widened to
//! retain escaped ReservedWord initializer evidence for EE-04-R08 by #342,
//! widened the one-level Block `var` production leaf from exactly one
//! declarator (#688/#691) to ordered `1..N` simple selected
//! `BindingIdentifier` declarators by #695, and widened each Block `var`
//! declarator to admit an optional selected decimal-integer initializer by
//! #699, widened each Block `var` declarator to additionally admit a
//! direct-authored, escape-free selected IdentifierReference initializer by
//! #710, widened each Block `var` declarator to additionally admit a
//! selected escaped non-ReservedWord IdentifierReference initializer by
//! #713 (escaped ReservedWord spellings remain outside this leaf), and
//! widened both the top-level and one-level Block `var` initializer position
//! to additionally admit a direct-authored selected `BooleanLiteral` by
//! #719, and widened the selected `LexicalDeclaration` initializer position
//! (only; top-level and Block `var` remained outside that leaf) to
//! additionally admit a direct-authored plain fractional `DecimalLiteral`
//! (`SelectedDecimalInteger "." DecimalDigits?` or `"." DecimalDigits`) by
//! #730, and widened both the top-level and one-level Block `var`
//! initializer position to additionally admit the same direct-authored plain
//! fractional `DecimalLiteral`, reusing the unmodified
//! `consume_selected_plain_fractional_decimal_literal` helper tried before
//! the unmodified `consume_selected_decimal_integer` predecessor at both
//! call sites, by #732, widened the selected `LexicalDeclaration`
//! initializer position (only; top-level and Block `var` remained outside
//! that leaf) to additionally admit a direct-authored, separator-free
//! exponent `DecimalLiteral` (`SelectedDecimalInteger SelectedPlainExponentPart`
//! or `SelectedPlainFractionalDecimalLiteral SelectedPlainExponentPart`,
//! where `SelectedPlainExponentPart ::= ("e" | "E") ("+" | "-")?
//! DecimalDigits`), via a new bounded `consume_selected_plain_exponent_decimal_literal`
//! helper tried before the unmodified `consume_selected_plain_fractional_decimal_literal`
//! and `consume_selected_decimal_integer` predecessors at that one call
//! site only, by #738, and widened both the top-level and one-level Block
//! `var` initializer position to additionally admit the same
//! direct-authored, separator-free exponent `DecimalLiteral`, reusing the
//! unmodified `consume_selected_plain_exponent_decimal_literal` helper
//! tried before the unmodified `consume_selected_plain_fractional_decimal_literal`
//! and `consume_selected_decimal_integer` predecessors at both call sites,
//! by #740, and composed a bounded, placement-neutral leading `+`/`-`
//! decimal `UnaryExpression` (`SelectedUnaryPlusMinus
//! SelectedNumericOperandTrivia SelectedAcceptedPlainDecimalAtom`) into all
//! three mature initializer owners (`parse_declaration`,
//! `parse_variable_statement`, `parse_selected_block_var_statement`) in one
//! leaf, via a new bounded
//! `consume_selected_leading_plus_minus_decimal_unary_expression` helper
//! tried before the unmodified exponent/fractional/decimal-integer
//! predecessors at all three call sites, reusing those helpers and the
//! unmodified `skip_selected_trivia` unchanged, per the
//! candidate-independent theorem accepted by #742/#743, by #744, and
//! composed a bounded, placement-neutral leading `+`/`-` direct
//! `IdentifierReference` `UnaryExpression` (`SelectedUnaryPlusMinus
//! SelectedUnaryOperandTrivia SelectedDirectIdentifierReference`) into the
//! same three initializer owners, tried immediately after the unmodified
//! decimal-unary predecessor and before the exponent/fractional/decimal-integer
//! predecessors at all three call sites, via a new bounded
//! `consume_selected_leading_plus_minus_identifier_reference_unary_expression`
//! helper reusing the unmodified `skip_selected_trivia` and shared
//! `consume_selected_identifier_reference` recognizer unchanged and
//! returning the existing inner `SelectedIdentifierReferenceFact`
//! unmodified, per the candidate-independent theorem accepted by #746/#747,
//! by #748, and generalized that same bounded helper (renamed from its
//! original direct-only name to the truthful
//! `consume_selected_leading_plus_minus_identifier_reference_unary_expression`
//! name shown above) to additionally accept a selected escaped
//! non-ReservedWord `IdentifierReference` operand recognized by the same
//! unmodified shared `consume_selected_identifier_reference` recognizer
//! call, at the same three call sites, without a second escaped-operand
//! scanner or decoder; an escaped ReservedWord operand continues to decline
//! this wrapper and restore the cursor, per the candidate-independent
//! theorem accepted by #241/#242 and this leaf's own frontier selection at
//! #688 comment 5724869567, by #750.
//! Recognition is transactional for the whole authoritative
//! `SourceText`: tentative declaration/binding/Block/var facts are returned
//! only when the entire source is consumed by selected items plus selected
//! trivia.
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
    items: Vec<SelectedBlockItem>,
}

impl SelectedBlock {
    pub(super) fn block(&self) -> &SourceAnchor {
        &self.block
    }

    pub(super) fn items(&self) -> &[SelectedBlockItem] {
        &self.items
    }

    /// Every Block-local lexical declaration, in authored item order, with
    /// `var`-statement items filtered out. Existing lexical-only consumers
    /// (Block duplicate-lexical checks, Binding/Scope, var-name
    /// correspondence) keep this exact accessor rather than widening to raw
    /// items.
    pub(super) fn declarations(&self) -> impl Iterator<Item = &SelectedLexicalDeclaration> {
        self.items.iter().filter_map(|item| match item {
            SelectedBlockItem::LexicalDeclaration(declaration) => Some(declaration),
            SelectedBlockItem::Var(_) => None,
        })
    }

    /// Every Block-local `var` binding contributor, in authored Block item
    /// order and, within each Block `var` statement, exact authored
    /// `VariableDeclarationList` order, with lexical declarations filtered
    /// out. Consumed only by the Block `VarDeclaredNames` / EE-14-R02 /
    /// Script-propagation static semantics introduced for Issue #691 and
    /// widened from exactly one contributor per statement to ordered `1..N`
    /// contributors per statement by Issue #695.
    pub(super) fn block_var_bindings(&self) -> impl Iterator<Item = &SelectedBlockVarBinding> {
        self.items
            .iter()
            .filter_map(|item| match item {
                SelectedBlockItem::Var(statement) => Some(statement.bindings().iter()),
                SelectedBlockItem::LexicalDeclaration(_) => None,
            })
            .flatten()
    }
}

/// One authored item of the one-level selected Block body, widened by
/// Issue #691 from lexical-declaration-only to admit exactly one additional
/// `var`-statement production leaf. Item order is retained because Tier-2b
/// (`EE-14-R02`) evidence selection and same-tier sibling-Block ordering are
/// authored-source-order sensitive.
#[derive(Debug)]
pub(super) enum SelectedBlockItem {
    LexicalDeclaration(SelectedLexicalDeclaration),
    Var(SelectedBlockVarStatement),
}

/// `SelectedBlockVarStatement ::= var SelectedBlockVarDeclaration
/// ( , SelectedBlockVarDeclaration )* ;` where `SelectedBlockVarDeclaration
/// ::= SelectedBindingIdentifier | SelectedBindingIdentifier =
/// SelectedDecimalInteger | SelectedBindingIdentifier =
/// SelectedDirectIdentifierReference | SelectedBindingIdentifier =
/// SelectedEscapedNonReservedIdentifierReference | SelectedBindingIdentifier =
/// SelectedEscapedReservedWordIdentifierName`, accepted by
/// Issue #688/#691/#695/#699/#710/#713/#715: one selected `VariableStatement`
/// owning ordered `1..N` selected `BindingIdentifier` declarators, each
/// independently optionally carrying a selected decimal-integer initializer,
/// a selected direct-authored or escaped non-ReservedWord
/// `IdentifierReference` initializer, or a classification-only escaped
/// ReservedWord `IdentifierName` initializer anchor for the later
/// `EE-04-R08` Tier-1 consumer, and one statement-owned terminator (Issue
/// #717 widens this from authored-semicolon-only to additionally admit
/// bounded close-brace ASI; see `SelectedBlockVarStatementTerminator`). No
/// comma or whole-statement span is retained because no proven consumer
/// needs it.
#[derive(Debug)]
pub(super) struct SelectedBlockVarStatement {
    bindings: Vec<SelectedBlockVarBinding>,
    terminator: SelectedBlockVarStatementTerminator,
}

impl SelectedBlockVarStatement {
    pub(super) fn bindings(&self) -> &[SelectedBlockVarBinding] {
        &self.bindings
    }

    pub(super) fn terminator(&self) -> SelectedBlockVarStatementTerminator {
        self.terminator
    }
}

/// Payload-free statement-owned termination provenance for
/// `SelectedBlockVarStatement` (Issue #717). `AuthoredSemicolon` proves only
/// that the statement was terminated by an authored `;`.
/// `AutomaticBeforeBlockClose` proves only that the statement's declarator
/// list completed and the next significant source position, after currently
/// selected trivia, is the containing Block's closing `}`; it carries no
/// authored or synthetic `SourceAnchor` for the inserted semicolon and no
/// anchor for `}` itself, which remains the enclosing Block's own authored
/// syntax and is left unconsumed by this statement's parser. This is
/// intentionally not shared with `SelectedVariableStatementTerminator`
/// (`AutomaticAtEof` is a distinct termination fact from
/// `AutomaticBeforeBlockClose`) or `SelectedDeclarationTerminator` (which
/// retains an authored-semicolon `SourceAnchor` this leaf has no consumer
/// for).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedBlockVarStatementTerminator {
    AuthoredSemicolon,
    AutomaticBeforeBlockClose,
}

/// One authored declarator within a `SelectedBlockVarStatement`'s
/// `VariableDeclarationList`: exactly one selected `BindingIdentifier` and an
/// independently optional initializer. A selected decimal-integer initializer
/// (Issue #699) is consumed and discarded without retaining any
/// initializer-specific fact. A selected direct-authored, escape-free
/// `IdentifierReference` initializer (Issue #710) or a selected escaped
/// non-ReservedWord `IdentifierReference` initializer (Issue #713) retains
/// the existing exact source-backed reference fact for the source-name
/// correspondence consumer. A selected escaped spelling that the shared
/// recognizer classifies as a ReservedWord (Issue #715) retains only its
/// exact authored initializer anchor, not a `SelectedIdentifierReferenceFact`
/// and not the decoded semantic name, for the later Tier-1 `EE-04-R08`
/// consumer. Distinct declarator occurrences remain distinct even when their
/// semantic names coincide (Issue #695 repeated-name requirement).
#[derive(Debug)]
pub(super) struct SelectedBlockVarBinding {
    binding: SourceAnchor,
    name_state: SelectedBindingNameState,
    identifier_reference_initializer: Option<SelectedIdentifierReferenceFact>,
    escaped_reserved_initializer_identifier: Option<SourceAnchor>,
}

impl SelectedBlockVarBinding {
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

    pub(super) fn identifier_reference_initializer(
        &self,
    ) -> Option<&SelectedIdentifierReferenceFact> {
        self.identifier_reference_initializer.as_ref()
    }

    pub(super) fn escaped_reserved_initializer_identifier(&self) -> Option<&SourceAnchor> {
        self.escaped_reserved_initializer_identifier.as_ref()
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
    identifier_reference_initializer: Option<SelectedIdentifierReferenceFact>,
    escaped_reserved_initializer_identifier: Option<SourceAnchor>,
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

    pub(super) fn identifier_reference_initializer(
        &self,
    ) -> Option<&SelectedIdentifierReferenceFact> {
        self.identifier_reference_initializer.as_ref()
    }

    pub(super) fn escaped_reserved_initializer_identifier(&self) -> Option<&SourceAnchor> {
        self.escaped_reserved_initializer_identifier.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedVariableStatementTerminator {
    AuthoredSemicolon,
    AutomaticAtEof,
}

/// One selected top-level `VariableStatement` owning its authored
/// `VariableDeclarationList` bindings and exactly one terminator.
///
/// `bindings` holds `1..N` selected `VariableDeclaration` binding facts in exact
/// authored list order. Each declarator may carry a parser-local selected
/// decimal-integer initializer, which retains no initializer-specific fact; one
/// selected direct/escaped non-ReservedWord IdentifierReference initializer,
/// which retains the existing exact source-backed reference fact for the source-
/// name correspondence consumer; or one classification-only exact authored
/// escaped-ReservedWord initializer anchor for the later EE-04-R08 Tier-1
/// consumer. Non-empty cardinality is a private recognizer construction
/// invariant: `parse_variable_statement` pushes the first binding before any
/// list continuation is considered and returns a `ParseFailure` instead of an
/// empty list. `Vec` does not encode that invariant at the type level, and no
/// generic non-empty collection is introduced for it.
///
/// The terminator is statement-owned and independent of list cardinality. No
/// comma, initializer kind, `VariableDeclarationList`, or whole-`VariableStatement`
/// extent is retained.
#[derive(Debug)]
pub(super) struct SelectedVariableStatement {
    bindings: Vec<SelectedVariableBinding>,
    terminator: SelectedVariableStatementTerminator,
}

impl SelectedVariableStatement {
    pub(super) fn bindings(&self) -> &[SelectedVariableBinding] {
        &self.bindings
    }

    pub(super) fn terminator(&self) -> SelectedVariableStatementTerminator {
        self.terminator
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
            (
                Self::VariableEnabled(items),
                SelectedTopLevelItem::LexicalDeclaration(declaration),
            ) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedVariableTopLevelItem::LexicalDeclaration(
                    declaration,
                ));
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
                    items.push(SelectedVariableTopLevelItem::LexicalDeclaration(
                        declaration,
                    ));
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

/// Result of the bounded leading `+`/`-` `IdentifierReference`
/// `UnaryExpression` helper (generalized by Issue #750 from the Issue #748
/// direct-only helper). `Matched` carries the exact existing
/// `SelectedIdentifierReferenceFact` produced by the shared
/// `consume_selected_identifier_reference()` recognizer for the operand,
/// unchanged, for either a direct-authored or an escaped non-ReservedWord
/// operand. `NotSelected` covers every decline: no leading `+`/`-`, an
/// escaped `ReservedWord` operand, or no `IdentifierReference` operand at
/// all. `ResourceLimited` and `InternalFailure` preserve the shared
/// recognizer's own processing-failure classes without collapsing them
/// into `NotSelected`.
#[derive(Debug)]
enum SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition {
    Matched(SelectedIdentifierReferenceFact),
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

        let mut items = Vec::new();
        loop {
            let item = if self.remaining().starts_with("var") {
                SelectedBlockItem::Var(self.parse_selected_block_var_statement()?)
            } else {
                SelectedBlockItem::LexicalDeclaration(self.parse_declaration()?)
            };
            items
                .try_reserve(1)
                .map_err(|_| ParseFailure::ResourceLimited)?;
            items.push(item);

            self.skip_selected_trivia();
            if self.consume_ascii('}') {
                break;
            }
            if self.is_eof() {
                return Err(ParseFailure::UnsupportedCoverage);
            }
        }

        let block = self.anchor(block_start, self.offset)?;
        Ok(SelectedBlock { block, items })
    }

    /// Recognizes exactly:
    ///
    /// ```text
    /// SelectedBlockVarStatement ::=
    ///     var SelectedBlockVarDeclaration
    ///         ( , SelectedBlockVarDeclaration )*
    ///     SelectedBlockVarTerminator
    ///
    /// SelectedBlockVarDeclaration ::=
    ///     SelectedBindingIdentifier
    ///   | SelectedBindingIdentifier = SelectedLeadingPlusMinusDecimalUnaryExpression
    ///   | SelectedBindingIdentifier = SelectedPlainExponentDecimalLiteral
    ///   | SelectedBindingIdentifier = SelectedPlainFractionalDecimalLiteral
    ///   | SelectedBindingIdentifier = SelectedDecimalInteger
    ///   | SelectedBindingIdentifier = SelectedDirectThisExpression
    ///   | SelectedBindingIdentifier = SelectedDirectEscapeFreeStringLiteral
    ///   | SelectedBindingIdentifier = SelectedDirectIdentifierReference
    ///   | SelectedBindingIdentifier = SelectedEscapedNonReservedIdentifierReference
    ///   | SelectedBindingIdentifier = SelectedEscapedReservedWordIdentifierName
    ///
    /// SelectedBlockVarTerminator ::=
    ///     ;
    ///   | [lookahead == `}`]
    /// ```
    ///
    /// (Issue #688/#691, widened to ordered `1..N` declarators by #695,
    /// widened to an optional selected decimal-integer initializer per
    /// declarator by #699, widened to an optional selected direct-authored,
    /// escape-free `IdentifierReference` initializer per declarator by
    /// #710, widened to an optional selected escaped non-ReservedWord
    /// `IdentifierReference` initializer per declarator by #713, widened to
    /// an optional selected escaped ReservedWord `IdentifierName`
    /// initializer per declarator by #715, widened to admit bounded
    /// close-brace ASI as an additional terminator route by #717, widened to
    /// an optional direct-authored `PrimaryExpression : this` initializer
    /// per declarator by #723, widened to an optional direct-authored plain
    /// fractional `DecimalLiteral` initializer per declarator, reusing the
    /// unmodified `consume_selected_plain_fractional_decimal_literal`
    /// helper tried before the unmodified decimal-integer predecessor, by
    /// #732, widened to an optional direct-authored, separator-free
    /// exponent `DecimalLiteral` initializer per declarator, reusing the
    /// unmodified `consume_selected_plain_exponent_decimal_literal` helper
    /// tried before the unmodified fractional and decimal-integer
    /// predecessors, by #740, and widened to an optional direct-authored
    /// leading `+`/`-` decimal `UnaryExpression` initializer per declarator,
    /// via the new bounded
    /// `consume_selected_leading_plus_minus_decimal_unary_expression` helper
    /// tried before the unmodified exponent/fractional/decimal-integer
    /// predecessors, by #744): one or
    /// more selected `BindingIdentifier` declarators separated by commas,
    /// each independently optionally followed by
    /// `= SelectedLeadingPlusMinusDecimalUnaryExpression`,
    /// `= SelectedPlainExponentDecimalLiteral`,
    /// `= SelectedPlainFractionalDecimalLiteral`, `= SelectedDecimalInteger`,
    /// `= SelectedDirectThisExpression`,
    /// `= SelectedIdentifierReference` (direct or escaped non-ReservedWord),
    /// or `= SelectedEscapedReservedWordIdentifierName` (classification-only
    /// source position for the later Tier-1 `EE-04-R08` consumer, not an
    /// accepted `SelectedIdentifierReference`), and one statement-owned
    /// terminator: an authored semicolon, or, when no semicolon is authored
    /// and the next significant source position (after currently selected
    /// trivia) is the containing Block's closing `}`, selected close-brace
    /// ASI (Issue #717). The `}` itself is never consumed here; it remains
    /// owned and consumed by `parse_selected_block`.
    ///
    /// This intentionally does not reuse `parse_variable_statement`: that
    /// owner's EOF-only ASI belongs to the distinct top-level
    /// `VariableStatement` capability and is a materially different
    /// termination fact from this Block-item's close-brace ASI, so it must
    /// not leak into this narrower Block-item placement. Only the keyword,
    /// `BindingIdentifier`, initializer-equals/decimal-integer/direct
    /// `BooleanLiteral`/`this`/IdentifierReference, and comma-continuation
    /// recognition mechanics are shared. A direct-authored or escaped
    /// non-ReservedWord `IdentifierReference` initializer is admitted
    /// (Issues #710/#713) through the existing source-backed
    /// `SelectedIdentifierReferenceFact`. An escaped spelling that the shared
    /// recognizer classifies as a ReservedWord (Issue #715) is admitted only
    /// as a classification-only exact authored initializer anchor for the
    /// later Tier-1 `EE-04-R08` consumer; it is not represented as an
    /// accepted `IdentifierReference` and its decoded semantic name is not
    /// persisted. A direct-authored `BooleanLiteral` initializer (Issue
    /// #719) is admitted through the existing accepted
    /// `consume_selected_boolean_literal` helper. A direct-authored
    /// `NullLiteral` initializer (Issue #721) is admitted through the
    /// existing accepted `consume_selected_null_literal` helper. A
    /// direct-authored `PrimaryExpression : this` initializer (Issue #723)
    /// is admitted through the existing accepted
    /// `consume_selected_this_expression` helper; an escaped spelling whose
    /// decoded semantic name is the reserved word `this` is not this route
    /// and continues through the classification-only escaped-ReservedWord
    /// C6 route above. A direct-authored, escape-free `StringLiteral`
    /// initializer (Issue #725) is admitted through the existing accepted
    /// `consume_selected_escape_free_string_literal` helper; an authored
    /// reverse solidus, raw LF, raw CR, or unclosed quote before the
    /// matching authored quote remains outside this route and continues to
    /// be reported as `UnsupportedCoverage`. Any other
    /// non-exponent, non-fractional, non-decimal, non-Boolean, non-Null, non-`this`,
    /// non-escape-free-StringLiteral, non-IdentifierReference
    /// initializer, comment trivia, EOF (i.e. non-EOF ASI before the
    /// enclosing `}`), or terminator
    /// whose next significant token is neither `;` nor `}` (including
    /// LineTerminator-triggered ASI before another statement) is left
    /// entirely unrecognized here and reported as `UnsupportedCoverage`.
    ///
    /// The direct-authored, separator-free exponent `DecimalLiteral` (Issue
    /// #740, reusing the existing accepted
    /// `consume_selected_plain_exponent_decimal_literal` helper, tried
    /// before the fractional and decimal-integer predecessors), the
    /// direct-authored plain fractional `DecimalLiteral` (Issue #732,
    /// reusing the existing accepted
    /// `consume_selected_plain_fractional_decimal_literal` helper), decimal,
    /// direct `BooleanLiteral`, direct `NullLiteral`, direct
    /// `this`, and direct escape-free `StringLiteral` initializers are consumed
    /// inside this owning cursor lifecycle and discarded: no
    /// initializer-specific fact (presence, anchor, or value) is retained on
    /// `SelectedBlockVarBinding` for any of them. A
    /// selected direct or escaped non-ReservedWord `IdentifierReference`
    /// initializer retains the existing exact source-backed
    /// `SelectedIdentifierReferenceFact` for the source-name correspondence
    /// consumer. A selected escaped ReservedWord `IdentifierName` initializer
    /// retains only its exact authored `SourceAnchor` for the later Tier-1
    /// `EE-04-R08` static-semantics consumer.
    ///
    /// The declarator list is accumulated in a purely local `Vec` and this
    /// function returns `Err` before constructing `SelectedBlockVarStatement`
    /// on any later declarator, initializer, or terminator failure, so a
    /// valid declarator prefix (e.g. `x` in `{ var x, ; }`, or `a=1` in
    /// `{ var a=1, ; }`, or `x=a` in `{ var x=a, ; }`) never escapes as a
    /// committed contributor when a later list element prevents statement
    /// completion.
    fn parse_selected_block_var_statement(
        &mut self,
    ) -> Result<SelectedBlockVarStatement, ParseFailure> {
        if !self.consume_keyword("var") {
            return Err(ParseFailure::UnsupportedCoverage);
        }

        let after_keyword = self.offset;
        self.skip_selected_trivia();
        let mut grammar_context = if self.offset == after_keyword {
            SelectedGrammarEvidenceContext::UnsupportedKeywordAdjacent
        } else {
            SelectedGrammarEvidenceContext::General
        };

        let mut bindings = Vec::new();
        loop {
            let (binding_start, binding_end, name_state) =
                self.parse_selected_binding_identifier(grammar_context)?;
            self.skip_selected_trivia();

            let (identifier_reference_initializer, escaped_reserved_initializer_identifier) =
                if self.consume_initializer_equals() {
                    self.skip_selected_trivia();
                    let facts = if self
                        .consume_selected_leading_plus_minus_decimal_unary_expression()
                    {
                        (None, None)
                    } else {
                        match self
                            .consume_selected_leading_plus_minus_identifier_reference_unary_expression()
                        {
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::Matched(reference) => {
                                (Some(reference), None)
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::ResourceLimited => {
                                return Err(ParseFailure::ResourceLimited);
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::InternalFailure => {
                                return Err(ParseFailure::InternalFailure);
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::NotSelected => {
                                if self.consume_selected_plain_exponent_decimal_literal()
                                    || self.consume_selected_plain_fractional_decimal_literal()
                                    || self.consume_selected_decimal_integer()
                                    || self.consume_selected_boolean_literal()
                                    || self.consume_selected_null_literal()
                                    || self.consume_selected_this_expression()
                                    || self.consume_selected_escape_free_string_literal()
                                {
                                    (None, None)
                                } else {
                                    match self.consume_selected_identifier_reference() {
                                        SelectedIdentifierReferenceRecognition::Matched(reference) => {
                                            (Some(reference), None)
                                        }
                                        SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName {
                                            identifier,
                                        } => (None, Some(identifier)),
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
                            }
                        }
                    };
                    self.skip_selected_trivia();
                    facts
                } else {
                    (None, None)
                };

            let binding = self.anchor(binding_start, binding_end)?;
            bindings
                .try_reserve(1)
                .map_err(|_| ParseFailure::ResourceLimited)?;
            bindings.push(SelectedBlockVarBinding {
                binding,
                name_state,
                identifier_reference_initializer,
                escaped_reserved_initializer_identifier,
            });

            if !self.consume_ascii(',') {
                break;
            }
            self.skip_selected_trivia();
            grammar_context = SelectedGrammarEvidenceContext::General;
        }

        let terminator = if self.consume_ascii(';') {
            SelectedBlockVarStatementTerminator::AuthoredSemicolon
        } else if self.peek_char() == Some('}') {
            SelectedBlockVarStatementTerminator::AutomaticBeforeBlockClose
        } else {
            return Err(ParseFailure::UnsupportedCoverage);
        };

        Ok(SelectedBlockVarStatement {
            bindings,
            terminator,
        })
    }

    /// Recognizes one selected top-level `VariableStatement` covering the
    /// inductive `VariableDeclarationList` base and successor productions with
    /// `1..N` simple bindings and optional selected direct-authored leading
    /// `+`/`-` decimal `UnaryExpression`, separator-free exponent
    /// `DecimalLiteral`, plain
    /// fractional `DecimalLiteral`, decimal-integer, direct
    /// `BooleanLiteral`, direct `NullLiteral`, direct `PrimaryExpression :
    /// this`, direct-authored escape-free `StringLiteral`, selected direct/escaped
    /// non-ReservedWord IdentifierReference, or selected escaped ReservedWord
    /// initializer source positions.
    ///
    /// The direct-authored leading `+`/`-` decimal `UnaryExpression` (Issue
    /// #744) is tried before the exponent, fractional, and decimal-integer
    /// predecessors, via the new bounded
    /// `consume_selected_leading_plus_minus_decimal_unary_expression` helper,
    /// which itself reuses those unmodified predecessor helpers unchanged.
    /// The direct-authored, separator-free exponent `DecimalLiteral` (Issue
    /// #740) is tried before the fractional and decimal-integer
    /// predecessors, reusing the existing accepted
    /// `consume_selected_plain_exponent_decimal_literal` helper unchanged.
    /// The direct-authored plain fractional `DecimalLiteral` (Issue #732) is
    /// tried before the decimal-integer predecessor, reusing the existing
    /// accepted `consume_selected_plain_fractional_decimal_literal` helper
    /// unchanged. Exponent, fractional, decimal, direct `BooleanLiteral`, direct
    /// `NullLiteral`, direct
    /// `this`, and direct escape-free `StringLiteral` initializer
    /// syntax are consumed inside this owning cursor lifecycle and discarded
    /// through the existing accepted `consume_selected_this_expression`
    /// helper (Issue #723) for the `this` case and the existing accepted
    /// `consume_selected_escape_free_string_literal` helper (Issue #725) for
    /// the `StringLiteral` case; an authored reverse solidus, raw LF, raw CR,
    /// or unclosed quote before the matching authored quote remains outside
    /// this route and continues to be reported as `UnsupportedCoverage`. An
    /// escaped spelling whose
    /// decoded semantic name is the reserved word `this` is not this route
    /// and continues through the existing escaped-ReservedWord C6 route. A
    /// selected IdentifierReference retains the
    /// complete existing source-backed fact on the containing binding for the
    /// source-name correspondence consumer. An escaped spelling already
    /// classified by the shared IdentifierName recognizer as a ReservedWord
    /// retains only its exact authored anchor on the containing binding for the
    /// later EE-04-R08 Tier-1 consumer; decoded identity is not persisted.
    ///
    /// Only the keyword-adjacent first declarator keeps the existing restricted
    /// grammar-evidence context; every later declarator uses the general
    /// selected `BindingIdentifier` route, so no new grammar-evidence category
    /// is introduced. Bindings stay local until the whole list plus its single
    /// terminator are selected: any failure returns before the statement is
    /// constructed and commits no partial selected-success state.
    fn parse_variable_statement(&mut self) -> Result<SelectedVariableStatement, ParseFailure> {
        if !self.consume_keyword("var") {
            return Err(ParseFailure::UnsupportedCoverage);
        }

        let after_keyword = self.offset;
        self.skip_selected_trivia();
        let mut grammar_context = if self.offset == after_keyword {
            SelectedGrammarEvidenceContext::UnsupportedKeywordAdjacent
        } else {
            SelectedGrammarEvidenceContext::General
        };

        let mut bindings = Vec::new();
        loop {
            let (binding_start, binding_end, name_state) =
                self.parse_selected_binding_identifier(grammar_context)?;

            self.skip_selected_trivia();
            let (identifier_reference_initializer, escaped_reserved_initializer_identifier) =
                if self.consume_initializer_equals() {
                    self.skip_selected_trivia();
                    let facts = if self
                        .consume_selected_leading_plus_minus_decimal_unary_expression()
                    {
                        (None, None)
                    } else {
                        match self
                            .consume_selected_leading_plus_minus_identifier_reference_unary_expression()
                        {
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::Matched(reference) => {
                                (Some(reference), None)
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::ResourceLimited => {
                                return Err(ParseFailure::ResourceLimited);
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::InternalFailure => {
                                return Err(ParseFailure::InternalFailure);
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::NotSelected => {
                                if self.consume_selected_plain_exponent_decimal_literal()
                                    || self.consume_selected_plain_fractional_decimal_literal()
                                    || self.consume_selected_decimal_integer()
                                    || self.consume_selected_boolean_literal()
                                    || self.consume_selected_null_literal()
                                    || self.consume_selected_this_expression()
                                    || self.consume_selected_escape_free_string_literal()
                                {
                                    (None, None)
                                } else {
                                    match self.consume_selected_identifier_reference() {
                                        SelectedIdentifierReferenceRecognition::Matched(reference) => {
                                            (Some(reference), None)
                                        }
                                        SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName {
                                            identifier,
                                        } => (None, Some(identifier)),
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
                            }
                        }
                    };
                    self.skip_selected_trivia();
                    facts
                } else {
                    (None, None)
                };

            let binding = self.anchor(binding_start, binding_end)?;
            bindings
                .try_reserve(1)
                .map_err(|_| ParseFailure::ResourceLimited)?;
            bindings.push(SelectedVariableBinding {
                binding,
                name_state,
                identifier_reference_initializer,
                escaped_reserved_initializer_identifier,
            });

            if !self.consume_ascii(',') {
                break;
            }
            self.skip_selected_trivia();
            grammar_context = SelectedGrammarEvidenceContext::General;
        }

        let terminator = if self.consume_ascii(';') {
            SelectedVariableStatementTerminator::AuthoredSemicolon
        } else if self.is_eof() {
            SelectedVariableStatementTerminator::AutomaticAtEof
        } else {
            return Err(ParseFailure::UnsupportedCoverage);
        };

        Ok(SelectedVariableStatement {
            bindings,
            terminator,
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
                if !self.consume_selected_leading_plus_minus_decimal_unary_expression() {
                    match self
                        .consume_selected_leading_plus_minus_identifier_reference_unary_expression()
                    {
                        SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::Matched(reference) => {
                            identifier_reference_initializer = Some(reference);
                        }
                        SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::ResourceLimited => {
                            return Err(ParseFailure::ResourceLimited);
                        }
                        SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::InternalFailure => {
                            return Err(ParseFailure::InternalFailure);
                        }
                        SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::NotSelected => {
                            if !self.consume_selected_plain_exponent_decimal_literal()
                                && !self.consume_selected_plain_fractional_decimal_literal()
                                && !self.consume_selected_decimal_integer()
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
    /// initializer position without widening into a generic
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

    /// Recognizes exactly one direct-authored leading `+` or `-` decimal
    /// `UnaryExpression` (`SelectedUnaryPlusMinus SelectedNumericOperandTrivia
    /// SelectedAcceptedPlainDecimalAtom`) in the selected initializer
    /// position, per the candidate-independent theorem accepted by
    /// #742/#743, without retaining any operator, trivia, or operand fact
    /// beyond the existing `SelectedInitializerState::SelectedPresent`.
    ///
    /// This bounded local scan commits `self.offset` only after a complete
    /// accepted decimal operand atom is recognized following the operator
    /// and any intervening existing selected trivia (`skip_selected_trivia`,
    /// unchanged); on decline (no leading `+`/`-`, or no accepted
    /// exponent/fractional/integer operand match after the operator), the
    /// cursor is restored to its starting offset and this helper returns
    /// `false`, leaving the unmodified unsigned predecessors free to
    /// recognize their own atoms. The operand chain reuses the existing
    /// accepted `consume_selected_plain_exponent_decimal_literal`,
    /// `consume_selected_plain_fractional_decimal_literal`, and
    /// `consume_selected_decimal_integer` helpers unchanged, in that exact
    /// order; no decimal grammar is duplicated here. Consequently `-1e-2` is
    /// recognized as this outer unary `-` composed with the complete
    /// exponent atom `1e-2`, whose own internal `-` remains owned entirely
    /// by the exponent helper, never as a signed `NumericLiteral`. A locally
    /// complete unary atom does not itself authorize any broader source;
    /// the enclosing declaration/statement/source transaction remains
    /// authoritative for any unowned trailing source (e.g. `-1 + x`,
    /// `-1 ** 2`, `-1e`).
    fn consume_selected_leading_plus_minus_decimal_unary_expression(&mut self) -> bool {
        let start = self.offset;

        if !self.consume_ascii('+') && !self.consume_ascii('-') {
            return false;
        }

        self.skip_selected_trivia();

        if self.consume_selected_plain_exponent_decimal_literal()
            || self.consume_selected_plain_fractional_decimal_literal()
            || self.consume_selected_decimal_integer()
        {
            true
        } else {
            self.offset = start;
            false
        }
    }

    /// Recognizes exactly one leading `+` or `-` `IdentifierReference`
    /// `UnaryExpression` (`SelectedUnaryPlusMinus SelectedUnaryOperandTrivia
    /// SelectedAcceptedIdentifierReference`, where
    /// `SelectedAcceptedIdentifierReference ::= SelectedDirectIdentifierReference
    /// | SelectedEscapedNonReservedIdentifierReference`) in the selected
    /// initializer position. The direct-operand case is the
    /// candidate-independent theorem accepted by #746/#747; the escaped
    /// non-Reserved operand case generalizes that accepted helper per Issue
    /// #750. Neither case retains any operator or trivia fact: on a match,
    /// the existing inner `SelectedIdentifierReferenceFact` produced by the
    /// unmodified shared `consume_selected_identifier_reference()`
    /// recognizer is returned unchanged, for either operand spelling. The
    /// operand is recognized and decoded exactly once by that unmodified
    /// shared recognizer; no second escaped-operand scanner or decoder is
    /// introduced.
    ///
    /// This bounded local scan commits `self.offset` only after a complete
    /// direct-authored or escaped non-ReservedWord `IdentifierReference`
    /// operand is recognized following the operator and any intervening
    /// existing selected trivia (`skip_selected_trivia`, unchanged). An
    /// escaped `ReservedWord` operand (e.g. `-\u{69}f`, decoding to `if`) or
    /// no `IdentifierReference` operand at all restores the cursor to its
    /// starting offset and returns `NotSelected`, leaving the unmodified
    /// decimal-unary predecessor and the unsigned atom/IdentifierReference
    /// predecessors free to recognize their own atoms; the restored cursor
    /// keeps an escaped-ReservedWord operand outside this wrapper and
    /// outside the existing plain escaped-ReservedWord C6 / EE-04-R08
    /// initializer route, which owns only its own unwrapped source
    /// position, not this wrapper's source position. A processing failure
    /// from the shared recognizer (`ResourceLimited` / `InternalFailure`) is
    /// preserved unchanged and is never downgraded to `NotSelected`. A
    /// locally complete unary-reference atom does not itself authorize any
    /// broader source; the enclosing declaration/statement/source
    /// transaction remains authoritative for any unowned trailing source
    /// (e.g. `-a.b`, `-a + b`).
    fn consume_selected_leading_plus_minus_identifier_reference_unary_expression(
        &mut self,
    ) -> SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition {
        let start = self.offset;

        if !self.consume_ascii('+') && !self.consume_ascii('-') {
            return SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::NotSelected;
        }

        self.skip_selected_trivia();

        match self.consume_selected_identifier_reference() {
            SelectedIdentifierReferenceRecognition::Matched(reference) => {
                SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::Matched(
                    reference,
                )
            }
            SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName { .. }
            | SelectedIdentifierReferenceRecognition::NotSelected => {
                self.offset = start;
                SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::NotSelected
            }
            SelectedIdentifierReferenceRecognition::ResourceLimited => {
                SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::ResourceLimited
            }
            SelectedIdentifierReferenceRecognition::InternalFailure => {
                SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::InternalFailure
            }
        }
    }

    /// Recognizes exactly one direct-authored, separator-free exponent
    /// `DecimalLiteral` (`SelectedDecimalInteger SelectedPlainExponentPart`
    /// or `SelectedPlainFractionalDecimalLiteral SelectedPlainExponentPart`,
    /// where `SelectedPlainExponentPart ::= ("e" | "E") ("+" | "-")?
    /// DecimalDigits`) in the selected `LexicalDeclaration` initializer
    /// position, per the candidate-independent theorem accepted by
    /// #735/#736, without retaining any numeric value, digit, or
    /// source-anchor fact beyond the existing
    /// `SelectedInitializerState::SelectedPresent`.
    ///
    /// This bounded local scan commits `self.offset` only after the complete
    /// selected exponent atom (mantissa plus exponent part) is recognized;
    /// on any decline, including an incomplete exponent tail such as `1e`,
    /// `1e+`, or `1e-`, the cursor is left entirely unchanged. The mantissa
    /// is rescanned locally with the same leading-zero and fraction rules as
    /// the unmodified `consume_selected_decimal_integer` and
    /// `consume_selected_plain_fractional_decimal_literal` predecessors,
    /// rather than calling either of them, because either predecessor would
    /// commit its own partial match (e.g. the `1` inside `1e2`, or the `1.0`
    /// inside `1.0e2`) before the exponent part is known to be present. A
    /// miss here leaves those unmodified predecessors free to recognize
    /// their own mantissa-only atoms (`1`, `1.0`, `.5`) exactly as before. A
    /// locally complete exponent atom does not itself authorize any broader
    /// source; the enclosing declaration/source transaction remains
    /// authoritative for any unowned trailing source (e.g. `1e2.foo`).
    fn consume_selected_plain_exponent_decimal_literal(&mut self) -> bool {
        let bytes = self.text.as_bytes();
        let mut offset = self.offset;

        let has_integer_part = match bytes.get(offset).copied() {
            Some(b'0') => {
                offset += 1;
                true
            }
            Some(b'1'..=b'9') => {
                offset += 1;
                while matches!(bytes.get(offset), Some(next) if next.is_ascii_digit()) {
                    offset += 1;
                }
                true
            }
            _ => false,
        };

        if bytes.get(offset) == Some(&b'.') {
            offset += 1;
            let fraction_digits_start = offset;
            while matches!(bytes.get(offset), Some(next) if next.is_ascii_digit()) {
                offset += 1;
            }
            let has_fraction_digits = offset > fraction_digits_start;
            if !has_integer_part && !has_fraction_digits {
                return false;
            }
        } else if !has_integer_part {
            return false;
        }

        if !matches!(bytes.get(offset), Some(b'e' | b'E')) {
            return false;
        }
        offset += 1;

        if matches!(bytes.get(offset), Some(b'+' | b'-')) {
            offset += 1;
        }

        let exponent_digits_start = offset;
        while matches!(bytes.get(offset), Some(next) if next.is_ascii_digit()) {
            offset += 1;
        }
        if offset == exponent_digits_start {
            return false;
        }

        self.offset = offset;
        true
    }

    /// Recognizes exactly one direct-authored, plain fractional
    /// `DecimalLiteral` (`SelectedDecimalInteger "." DecimalDigits?` or
    /// `"." DecimalDigits`) in the selected `LexicalDeclaration` initializer
    /// position, per the candidate-independent theorem accepted by #727/#728,
    /// without retaining any numeric value, digit, or source-anchor fact
    /// beyond the existing `SelectedInitializerState::SelectedPresent`.
    ///
    /// This bounded local scan commits `self.offset` only after the complete
    /// selected fractional atom is recognized; on decline the cursor is left
    /// unchanged, so the unmodified `consume_selected_decimal_integer`
    /// predecessor still owns plain integers such as `1`. The scan reuses the
    /// same leading-zero boundary as that predecessor: a `0` integer part
    /// must be immediately followed by `.`, or this helper declines without
    /// commit, so a leading-zero spelling such as `01.0` is never selected
    /// here. A locally complete fractional prefix (e.g. the `1.` inside
    /// `1..foo`, or the `1.0` inside `1.0e2`) does not itself authorize any
    /// broader source; the enclosing declaration/source transaction remains
    /// authoritative for any unowned trailing source.
    fn consume_selected_plain_fractional_decimal_literal(&mut self) -> bool {
        let bytes = self.text.as_bytes();
        let mut offset = self.offset;

        let has_integer_part = match bytes.get(offset).copied() {
            Some(b'0') => {
                offset += 1;
                true
            }
            Some(b'1'..=b'9') => {
                offset += 1;
                while matches!(bytes.get(offset), Some(next) if next.is_ascii_digit()) {
                    offset += 1;
                }
                true
            }
            _ => false,
        };

        if bytes.get(offset) != Some(&b'.') {
            return false;
        }
        offset += 1;

        let fraction_digits_start = offset;
        while matches!(bytes.get(offset), Some(next) if next.is_ascii_digit()) {
            offset += 1;
        }
        let has_fraction_digits = offset > fraction_digits_start;

        if !has_integer_part && !has_fraction_digits {
            return false;
        }

        self.offset = offset;
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
