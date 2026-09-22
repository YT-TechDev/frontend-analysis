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
//! #688 comment 5724869567, by #750, and widened the retained
//! `IdentifierReference` initializer carrier from at-most-one to a bounded
//! `None`/`One`/`Two`-equivalent representation, composing an ordered
//! `SelectedTwoIdentifierReferenceAdditiveInitializer`
//! (`SelectedAcceptedIdentifierReference SelectedAdditiveTrivia ("+" | "-")
//! SelectedAdditiveTrivia SelectedAcceptedIdentifierReference`) into the same
//! three initializer owners via a new bounded
//! `consume_selected_identifier_reference_initializer` helper that absorbs
//! the previous plain single-reference route, reuses the unmodified shared
//! `consume_selected_identifier_reference` recognizer exactly once per
//! operand, and retains no operator or whole-expression fact, per the
//! candidate-independent theorem accepted by #752/#753, by #754, and
//! composed a bounded, transactional free-standing top-level
//! `IdentifierReference` `ExpressionStatement` use-site leaf
//! (`SelectedAcceptedIdentifierReference AuthoredSemicolon`) into a new
//! fourth / broadest selected Script carrier, whose top-level item domain
//! is `LexicalDeclaration | Block | VariableStatement |
//! IdentifierReferenceExpressionStatement`. The bounded use-site probe
//! (`consume_selected_top_level_identifier_reference_expression_statement`)
//! runs transactionally before the existing raw top-level `var` / Block /
//! lexical-declaration dispatch, reusing the unmodified shared
//! `consume_selected_identifier_reference` recognizer exactly once per
//! candidate and retaining the existing `SelectedIdentifierReferenceFact`
//! unmodified as the use-site payload -- no whole Statement, whole
//! Expression, or semicolon anchor is retained, and no second scanner or
//! decoder is introduced. Historical carriers
//! (`SelectedLexicalScript`, `SelectedOneLevelBlockScript`,
//! `SelectedVariableStatementScript`) remain semantically unchanged; the
//! builder transitions monotonically into the new broadest
//! `ReferenceUseEnabled` state exactly once, moving any already-collected
//! `Flat`/`BlockEnabled`/`VariableEnabled` items into the new item
//! representation in authored order, per the candidate-independent theorem
//! accepted by #756/#757, by #758. Recognition is transactional for the
//! whole authoritative
//! `SourceText`: tentative declaration/binding/Block/var/use-site facts are
//! returned only when the entire source is consumed by selected items plus
//! selected trivia, and composed a bounded, transactional Block-contained
//! free-standing `IdentifierReference` `ExpressionStatement` use-site leaf
//! into one owned single-pass selected one-level Block parse
//! (`Cursor::parse_selected_block`), producing the new crate-private
//! `SelectedUseSiteEnabledBlock` representation exactly when at least one
//! Block-contained use-site commits (the exact unchanged historical
//! `SelectedBlock` otherwise), generalizing the #758 bounded use-site probe
//! (renamed to the placement-neutral
//! `consume_selected_identifier_reference_expression_statement_use_site`) to
//! run transactionally before the existing raw Block `var` /
//! lexical-declaration dispatch, and composed a new fifth / broadest
//! selected Script carrier (`SelectedBlockReferenceUseEnabledScript`)
//! distinguishing historical from use-site-enabled Blocks while retaining
//! authored top-level order and never reparsing or reconstructing an
//! already-produced historical Block, per the candidate-independent
//! theorem accepted by #760/#761 and the production representation /
//! placement authority accepted by #688 comment 5734964743, by #762.
//! Historical carriers, `SelectedBlock`, and `SelectedBlockItem` remain
//! semantically unchanged. Widened the free-standing `IdentifierReference`
//! `ExpressionStatement` use-site payload -- shared by
//! `SelectedReferenceUseEnabledTopLevelItem`,
//! `SelectedBlockReferenceUseEnabledTopLevelItem`, and
//! `SelectedUseSiteEnabledBlockItem` -- from exactly one retained
//! `SelectedIdentifierReferenceFact` to a new crate-private bounded
//! `SelectedFreeStandingIdentifierReferenceUseSite` (`One` | `Two`)
//! occurrence carrier, composing an ordered
//! `SelectedTwoIdentifierReferenceAdditiveExpressionStatement`
//! (`SelectedAcceptedIdentifierReference SelectedAdditiveTrivia ("+" | "-")
//! SelectedAdditiveTrivia SelectedAcceptedIdentifierReference`) into the same
//! placement-neutral
//! `consume_selected_identifier_reference_expression_statement_use_site`
//! probe that already owns both TopLevel and Block-contained placements,
//! reusing the unmodified shared `consume_selected_identifier_reference`
//! recognizer exactly once per operand and retaining no operator or
//! whole-expression fact, per the candidate-independent theorem accepted by
//! #764/#765 and the production representation / placement authority
//! accepted by #688 comment 5739718987, by #766. Unlike
//! `consume_selected_identifier_reference_initializer`, a failed additive
//! continuation rolls back the whole probe to its original snapshot instead
//! of degrading to a successful single-reference `One`: a free-standing
//! Statement has no enclosing binding/comma transaction able to recover a
//! left-over unconsumed suffix. Existing Script/Block builder topology,
//! static witnesses, and qualification branches remain unchanged; only the
//! existing use-site item payload widens. Widened the free-standing
//! `IdentifierReference` `ExpressionStatement` use-site family by exactly
//! one statement-level dimension, explicit termination provenance, per the
//! candidate-independent theorem accepted by #233/#234, #318/#319, #688
//! comment 5685046994 / #717, #756/#757, #760/#761, and #764/#765, composed
//! by #688 comment 5741848600, by #768. The placement-neutral body probe
//! (renamed `consume_selected_identifier_reference_expression_statement_use_site_body`)
//! now claims no terminator; two new placement-owned callers,
//! `consume_selected_top_level_identifier_reference_expression_statement_use_site`
//! and
//! `consume_selected_block_identifier_reference_expression_statement_use_site`,
//! each pair the unchanged `One`/`Two` body with a new, owner-specific,
//! crate-private termination enum
//! (`SelectedTopLevelFreeStandingIdentifierReferenceUseSiteTerminator`:
//! `AuthoredSemicolon` | `AutomaticAtEof`; and
//! `SelectedBlockFreeStandingIdentifierReferenceUseSiteTerminator`:
//! `AuthoredSemicolon` | `AutomaticBeforeBlockClose`), making
//! `TopLevel + AutomaticBeforeBlockClose` and `Block + AutomaticAtEof`
//! unrepresentable. Neither new terminator retains a `SourceAnchor`: an
//! automatic semicolon is synthesized structure, never authored source, and
//! the containing Block's `}` remains solely owned and consumed by
//! `Cursor::parse_selected_block`. `SelectedVariableStatementTerminator` and
//! `SelectedBlockVarStatementTerminator` remain distinct, unchanged,
//! unreused precedents. Correspondence, static semantics, qualification
//! branches, Script/Block carrier topology, and completion remain
//! unchanged; only a mechanical `body()`/`terminator()` accessor split
//! replaces the previous bare `SelectedFreeStandingIdentifierReferenceUseSite`
//! use-site item payload. Composed a bounded leading `+`/`-`
//! `IdentifierReference` `UnaryExpression`
//! (`SelectedUnaryPlusMinus SelectedUnaryOperandTrivia
//! SelectedAcceptedIdentifierReference`) into the placement-neutral
//! free-standing body probe
//! (`consume_selected_identifier_reference_expression_statement_use_site_body`)
//! as one additional bounded body form, tried before the existing
//! bare/additive body logic, reusing the unmodified
//! `consume_selected_leading_plus_minus_identifier_reference_unary_expression`
//! helper and mapping a matched result onto the existing
//! `One(SelectedIdentifierReferenceFact)` occurrence -- the same
//! representation an unwrapped bare reference produces -- per the
//! candidate-independent theorem accepted by #746/#747 and this leaf's own
//! frontier selection at #688 comment 5744036288, by #771. No unary-specific
//! item or body variant is introduced; no operator, trivia, or whole-unary
//! `SourceAnchor` is retained; TopLevel/Block placement, terminator
//! provenance, correspondence, static semantics, qualification branches, and
//! Script/Block carrier topology remain unchanged. Widened that same
//! unary-first body route: a matched leading `+`/`-` `IdentifierReference`
//! `UnaryExpression` no longer commits `One` immediately -- selected trivia
//! is skipped and, when exactly one authored binary `+` or `-` follows, one
//! plain accepted `IdentifierReference` is recognized by the same unmodified
//! shared `consume_selected_identifier_reference` recognizer and the whole
//! probe commits the existing `Two { first, second }` occurrence instead;
//! absent that continuation, `One` still commits exactly as before. A
//! declining or escaped-ReservedWord second operand rolls the whole body
//! probe back to its pre-unary snapshot (`NotSelected`), never degrading to
//! `One(first)`, mirroring the existing plain-additive continuation's own
//! transactional decline. No new item, body variant, operator, trivia, or
//! whole-expression `SourceAnchor` is retained; `++`/`--` and three-or-more
//! additive operands remain outside;
//! TopLevel/Block placement, terminator provenance, correspondence, static
//! semantics, qualification branches, and Script/Block carrier topology
//! remain unchanged, per the candidate-independent theorem accepted by
//! #746/#747, #752/#753, #764/#765, and this leaf's own frontier selection
//! at #688 comment 5744425380, by #773.
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
    RecognizedIdentifierReferenceExpressionStatementSlice(
        SelectedIdentifierReferenceExpressionStatementScript,
    ),
    RecognizedBlockReferenceUseEnabledSlice(SelectedBlockReferenceUseEnabledScript),
    UnsupportedCoverage,
    DefinitiveGrammarRejectionEvidence {
        subject: SourceAnchor,
    },
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

/// The new fourth / broadest selected Script carrier (Issue #758),
/// retaining authored top-level item order across every already-accepted
/// item shape plus the new free-standing `IdentifierReference`
/// `ExpressionStatement` use-site leaf. Historical carriers
/// (`SelectedLexicalScript`, `SelectedOneLevelBlockScript`,
/// `SelectedVariableStatementScript`) remain distinct, semantically
/// unchanged accepted authorities; this is a new, separate carrier, never a
/// widening of `SelectedVariableStatementScript` itself.
#[derive(Debug)]
pub(super) struct SelectedIdentifierReferenceExpressionStatementScript {
    items: Vec<SelectedReferenceUseEnabledTopLevelItem>,
}

impl SelectedIdentifierReferenceExpressionStatementScript {
    pub(super) fn items(&self) -> &[SelectedReferenceUseEnabledTopLevelItem] {
        &self.items
    }
}

/// One top-level item of the broadest selected Script carrier. The
/// `IdentifierReferenceExpressionStatement` variant retains the placement-owned
/// `SelectedTopLevelIdentifierReferenceUseSite` (Issue #768; the bounded
/// `SelectedFreeStandingIdentifierReferenceUseSite` occurrence carrier alone,
/// Issue #766, before #768), pairing the existing `One`/`Two` body with this
/// placement's own `AuthoredSemicolon`/`AutomaticAtEof` termination
/// provenance: any authored `+`/`-` operator remains a construction
/// invariant of the body, never a retained anchor or relation payload.
#[derive(Debug)]
pub(super) enum SelectedReferenceUseEnabledTopLevelItem {
    LexicalDeclaration(SelectedLexicalDeclaration),
    Block(SelectedBlock),
    VariableStatement(SelectedVariableStatement),
    IdentifierReferenceExpressionStatement(SelectedTopLevelIdentifierReferenceUseSite),
}

/// New fifth / broadest selected Script carrier (Issue #762), composing the
/// #758 free-standing top-level use-site leaf with Block-contained
/// free-standing use-sites. Historical carriers (`SelectedLexicalScript`,
/// `SelectedOneLevelBlockScript`, `SelectedVariableStatementScript`,
/// `SelectedIdentifierReferenceExpressionStatementScript`) remain distinct,
/// semantically unchanged accepted authorities; this is a new, separate
/// carrier, never a widening of any of them.
#[derive(Debug)]
pub(super) struct SelectedBlockReferenceUseEnabledScript {
    items: Vec<SelectedBlockReferenceUseEnabledTopLevelItem>,
}

impl SelectedBlockReferenceUseEnabledScript {
    pub(super) fn items(&self) -> &[SelectedBlockReferenceUseEnabledTopLevelItem] {
        &self.items
    }
}

/// One top-level item of the fifth / broadest selected Script carrier
/// (Issue #762). `Block` retains the exact historical representation for a
/// selected one-level Block containing no Block-local free-standing
/// use-site; `UseSiteEnabledBlock` is the distinct new representation for a
/// Block containing at least one. `IdentifierReferenceExpressionStatement`
/// retains the placement-owned `SelectedTopLevelIdentifierReferenceUseSite`
/// (Issue #768) for a top-level free-standing use-site -- the same TopLevel
/// placement owner as `SelectedReferenceUseEnabledTopLevelItem`'s variant of
/// the same name, never the Block-owned counterpart.
#[derive(Debug)]
pub(super) enum SelectedBlockReferenceUseEnabledTopLevelItem {
    LexicalDeclaration(SelectedLexicalDeclaration),
    Block(SelectedBlock),
    UseSiteEnabledBlock(SelectedUseSiteEnabledBlock),
    VariableStatement(SelectedVariableStatement),
    IdentifierReferenceExpressionStatement(SelectedTopLevelIdentifierReferenceUseSite),
}

/// New use-site-enabled Block representation (Issue #762): a selected
/// one-level Block containing at least one Block-contained free-standing
/// `IdentifierReference` `ExpressionStatement` use-site, alongside existing
/// selected `LexicalDeclaration` / Block `var` items, in exact authored
/// order. Reuses the existing `SelectedIdentifierReferenceFact` for the
/// use-site payload; retains no whole-Statement, whole-Expression, or
/// semicolon anchor, and no item ordinal beyond authored `Vec` position. Is
/// never produced by reparsing or rescanning an already-produced historical
/// `SelectedBlock`: the owning single left-to-right Block parse in
/// `Cursor::parse_selected_block` decides which of the two representations
/// to construct exactly once, from the same cursor lifecycle.
#[derive(Debug)]
pub(super) struct SelectedUseSiteEnabledBlock {
    block: SourceAnchor,
    items: Vec<SelectedUseSiteEnabledBlockItem>,
}

impl SelectedUseSiteEnabledBlock {
    pub(super) fn block(&self) -> &SourceAnchor {
        &self.block
    }

    pub(super) fn items(&self) -> &[SelectedUseSiteEnabledBlockItem] {
        &self.items
    }

    /// Every Block-local lexical declaration, in authored item order, with
    /// `var`-statement and use-site items filtered out. Mirrors
    /// `SelectedBlock::declarations` exactly for the new representation, so
    /// existing lexical-only consumers (Block duplicate-lexical checks,
    /// var-name correspondence) reuse the identical accessor shape.
    pub(super) fn declarations(&self) -> impl Iterator<Item = &SelectedLexicalDeclaration> {
        self.items.iter().filter_map(|item| match item {
            SelectedUseSiteEnabledBlockItem::LexicalDeclaration(declaration) => Some(declaration),
            SelectedUseSiteEnabledBlockItem::Var(_)
            | SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(_) => None,
        })
    }

    /// Every Block-local `var` binding contributor, in authored Block item
    /// order and, within each Block `var` statement, exact authored
    /// `VariableDeclarationList` order, with lexical declarations and
    /// use-site items filtered out. Mirrors `SelectedBlock::block_var_bindings`
    /// exactly for the new representation.
    pub(super) fn block_var_bindings(&self) -> impl Iterator<Item = &SelectedBlockVarBinding> {
        self.items
            .iter()
            .filter_map(|item| match item {
                SelectedUseSiteEnabledBlockItem::Var(statement) => {
                    Some(statement.bindings().iter())
                }
                SelectedUseSiteEnabledBlockItem::LexicalDeclaration(_)
                | SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(_) => {
                    None
                }
            })
            .flatten()
    }
}

/// `UseSiteEnabledBlockItem ::= existing selected LexicalDeclaration |
/// existing selected Block Var statement | selected IdentifierReference
/// ExpressionStatement use-site` (Issue #762). The use-site variant retains
/// the placement-owned `SelectedBlockIdentifierReferenceUseSite` (Issue
/// #768; the bounded `SelectedFreeStandingIdentifierReferenceUseSite`
/// occurrence carrier alone, Issue #766, before #768), pairing the existing
/// `One`/`Two` body with this Block placement's own
/// `AuthoredSemicolon`/`AutomaticBeforeBlockClose` termination provenance:
/// any authored `+`/`-` operator remains a construction invariant of the
/// body, never a retained anchor or relation payload.
#[derive(Debug)]
pub(super) enum SelectedUseSiteEnabledBlockItem {
    LexicalDeclaration(SelectedLexicalDeclaration),
    Var(SelectedBlockVarStatement),
    IdentifierReferenceExpressionStatement(SelectedBlockIdentifierReferenceUseSite),
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
    identifier_reference_initializer: Option<SelectedIdentifierReferenceInitializer>,
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

    /// The first retained `IdentifierReference` fact, regardless of how many
    /// facts the initializer retains overall (a `One`, `Two`, or `Three`
    /// carrier): either a single-operand initializer's only reference, the
    /// one retained reference of a selected two-syntax-operand
    /// `IdentifierReference`/plain-Decimal additive initializer (Issue
    /// #791), or the authored left operand of a two- or three-reference
    /// additive initializer. Existing single-operand behavior is exactly
    /// preserved; use [`Self::identifier_reference_initializer_facts`] to
    /// observe every retained fact.
    pub(super) fn identifier_reference_initializer(
        &self,
    ) -> Option<&SelectedIdentifierReferenceFact> {
        self.identifier_reference_initializer
            .as_ref()
            .map(SelectedIdentifierReferenceInitializer::first)
    }

    /// Every retained fact, in authored left-to-right order (0, 1, 2, or 3
    /// items).
    pub(super) fn identifier_reference_initializer_facts(
        &self,
    ) -> impl Iterator<Item = &SelectedIdentifierReferenceFact> {
        self.identifier_reference_initializer
            .iter()
            .flat_map(SelectedIdentifierReferenceInitializer::facts)
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

/// Crate-private bounded cardinality carrier for the selected
/// `IdentifierReference` initializer position, widening the previous
/// at-most-one carrier to admit up to two additional selected additive
/// operands (Issue #754; Issue #797 per #688 comment 5771773635). `One`
/// retains exactly one source-backed `IdentifierReference` fact: either a
/// single-operand initializer's only reference, or the sole retained
/// reference of a selected two-syntax-operand `IdentifierReference`/plain-Decimal
/// additive initializer (Issue #791, per #688 comment 5762579228), whose
/// Decimal operand, binary operator, and left/right orientation are never
/// retained. `Two` retains both authored operands of a selected
/// `SelectedTwoIdentifierReferenceAdditiveInitializer` in exact authored
/// left-to-right order (`first` is the left operand, `second` is the right
/// operand). `Three` retains all three authored operands of a selected
/// `SelectedExactlyThreeIdentifierReferenceAdditiveInitializer` (Issue #797)
/// in exact authored left-to-right order; it composes the accepted
/// candidate-independent theorem proven by #795/PR #796 for plain/plain/plain
/// `IdentifierReference` operands only. Four or more facts, a second or third
/// fact without its predecessors, holes, reordering, and deduplication are
/// all unrepresentable by this type. The containing binding's
/// `Option<SelectedIdentifierReferenceInitializer>` field is the sole
/// retained-reference storage: `None` means zero facts, so no separate
/// competing fact channel exists anywhere on the binding.
#[derive(Debug)]
pub(super) enum SelectedIdentifierReferenceInitializer {
    One(SelectedIdentifierReferenceFact),
    Two {
        first: SelectedIdentifierReferenceFact,
        second: SelectedIdentifierReferenceFact,
    },
    Three {
        first: SelectedIdentifierReferenceFact,
        second: SelectedIdentifierReferenceFact,
        third: SelectedIdentifierReferenceFact,
    },
}

impl SelectedIdentifierReferenceInitializer {
    /// The first (and, for `One`, only) retained fact: the sole retained
    /// `IdentifierReference` fact of a `One` initializer (whether its syntax
    /// is a single-operand initializer or a selected two-syntax-operand
    /// `IdentifierReference`/plain-Decimal additive initializer, Issue
    /// #791), or the authored left operand of a two- or three-reference
    /// additive initializer. Always defined and never panics.
    pub(super) fn first(&self) -> &SelectedIdentifierReferenceFact {
        match self {
            Self::One(fact) => fact,
            Self::Two { first, .. } => first,
            Self::Three { first, .. } => first,
        }
    }

    /// Every retained fact, in exact authored left-to-right order: one item
    /// for `One`, two for `Two`, three for `Three`. Backed by a fixed-size
    /// array, never a heap allocation.
    pub(super) fn facts(&self) -> impl Iterator<Item = &SelectedIdentifierReferenceFact> {
        match self {
            Self::One(fact) => [Some(fact), None, None],
            Self::Two { first, second } => [Some(first), Some(second), None],
            Self::Three {
                first,
                second,
                third,
            } => [Some(first), Some(second), Some(third)],
        }
        .into_iter()
        .flatten()
    }
}

#[derive(Debug)]
pub(super) struct SelectedLexicalBinding {
    binding: SourceAnchor,
    name_state: SelectedBindingNameState,
    initializer: SelectedInitializerState,
    identifier_reference_initializer: Option<SelectedIdentifierReferenceInitializer>,
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

    /// The first retained `IdentifierReference` fact, regardless of how many
    /// facts the initializer retains overall (a `One`, `Two`, or `Three`
    /// carrier): either a single-operand initializer's only reference, the
    /// one retained reference of a selected two-syntax-operand
    /// `IdentifierReference`/plain-Decimal additive initializer (Issue
    /// #791), or the authored left operand of a two- or three-reference
    /// additive initializer. Existing single-operand behavior is exactly
    /// preserved; use [`Self::identifier_reference_initializer_facts`] to
    /// observe every retained fact.
    pub(super) fn identifier_reference_initializer(
        &self,
    ) -> Option<&SelectedIdentifierReferenceFact> {
        self.identifier_reference_initializer
            .as_ref()
            .map(SelectedIdentifierReferenceInitializer::first)
    }

    /// Every retained fact, in authored left-to-right order (0, 1, 2, or 3
    /// items).
    pub(super) fn identifier_reference_initializer_facts(
        &self,
    ) -> impl Iterator<Item = &SelectedIdentifierReferenceFact> {
        self.identifier_reference_initializer
            .iter()
            .flat_map(SelectedIdentifierReferenceInitializer::facts)
    }

    pub(super) fn escaped_reserved_initializer_identifier(&self) -> Option<&SourceAnchor> {
        self.escaped_reserved_initializer_identifier.as_ref()
    }
}

#[derive(Debug)]
pub(super) struct SelectedVariableBinding {
    binding: SourceAnchor,
    name_state: SelectedBindingNameState,
    identifier_reference_initializer: Option<SelectedIdentifierReferenceInitializer>,
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

    /// The first retained `IdentifierReference` fact, regardless of how many
    /// facts the initializer retains overall (a `One`, `Two`, or `Three`
    /// carrier): either a single-operand initializer's only reference, the
    /// one retained reference of a selected two-syntax-operand
    /// `IdentifierReference`/plain-Decimal additive initializer (Issue
    /// #791), or the authored left operand of a two- or three-reference
    /// additive initializer. Existing single-operand behavior is exactly
    /// preserved; use [`Self::identifier_reference_initializer_facts`] to
    /// observe every retained fact.
    pub(super) fn identifier_reference_initializer(
        &self,
    ) -> Option<&SelectedIdentifierReferenceFact> {
        self.identifier_reference_initializer
            .as_ref()
            .map(SelectedIdentifierReferenceInitializer::first)
    }

    /// Every retained fact, in authored left-to-right order (0, 1, 2, or 3
    /// items).
    pub(super) fn identifier_reference_initializer_facts(
        &self,
    ) -> impl Iterator<Item = &SelectedIdentifierReferenceFact> {
        self.identifier_reference_initializer
            .iter()
            .flat_map(SelectedIdentifierReferenceInitializer::facts)
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
    ReferenceUseEnabled(Vec<SelectedReferenceUseEnabledTopLevelItem>),
    BlockReferenceUseEnabled(Vec<SelectedBlockReferenceUseEnabledTopLevelItem>),
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
            (
                Self::ReferenceUseEnabled(items),
                SelectedTopLevelItem::LexicalDeclaration(declaration),
            ) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedReferenceUseEnabledTopLevelItem::LexicalDeclaration(
                    declaration,
                ));
                Ok(())
            }
            (Self::ReferenceUseEnabled(items), SelectedTopLevelItem::Block(block)) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedReferenceUseEnabledTopLevelItem::Block(block));
                Ok(())
            }
            (
                Self::BlockReferenceUseEnabled(items),
                SelectedTopLevelItem::LexicalDeclaration(declaration),
            ) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(
                    SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(declaration),
                );
                Ok(())
            }
            (Self::BlockReferenceUseEnabled(items), SelectedTopLevelItem::Block(block)) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedBlockReferenceUseEnabledTopLevelItem::Block(block));
                Ok(())
            }
        }
    }

    /// Commits the transactionally-recognized free-standing top-level
    /// use-site fact (Issue #758). When the builder is still `Flat`,
    /// `BlockEnabled`, or `VariableEnabled`, this is the first selected
    /// use-site: existing items move into the new broadest item
    /// representation in exact authored order before the use-site is
    /// appended, promoting the builder to `ReferenceUseEnabled` exactly
    /// once. Once already `ReferenceUseEnabled`, the use-site appends
    /// directly.
    fn push_use_site(
        &mut self,
        use_site: SelectedTopLevelIdentifierReferenceUseSite,
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
                    items.push(SelectedReferenceUseEnabledTopLevelItem::LexicalDeclaration(
                        declaration,
                    ));
                }
                items.push(
                    SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                        use_site,
                    ),
                );
                *builder = Self::ReferenceUseEnabled(items);
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
                        SelectedTopLevelItem::LexicalDeclaration(declaration) => {
                            items.push(SelectedReferenceUseEnabledTopLevelItem::LexicalDeclaration(
                                declaration,
                            ))
                        }
                        SelectedTopLevelItem::Block(block) => {
                            items.push(SelectedReferenceUseEnabledTopLevelItem::Block(block));
                        }
                    }
                }
                items.push(
                    SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                        use_site,
                    ),
                );
                *builder = Self::ReferenceUseEnabled(items);
                Ok(())
            }
            builder @ Self::VariableEnabled(_) => {
                let Self::VariableEnabled(existing_items) = builder else {
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
                        SelectedVariableTopLevelItem::LexicalDeclaration(declaration) => items
                            .push(SelectedReferenceUseEnabledTopLevelItem::LexicalDeclaration(
                                declaration,
                            )),
                        SelectedVariableTopLevelItem::Block(block) => {
                            items.push(SelectedReferenceUseEnabledTopLevelItem::Block(block));
                        }
                        SelectedVariableTopLevelItem::VariableStatement(statement) => {
                            items.push(SelectedReferenceUseEnabledTopLevelItem::VariableStatement(
                                statement,
                            ));
                        }
                    }
                }
                items.push(
                    SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                        use_site,
                    ),
                );
                *builder = Self::ReferenceUseEnabled(items);
                Ok(())
            }
            Self::ReferenceUseEnabled(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(
                    SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                        use_site,
                    ),
                );
                Ok(())
            }
            Self::BlockReferenceUseEnabled(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(
                    SelectedBlockReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                        use_site,
                    ),
                );
                Ok(())
            }
        }
    }

    /// Commits the transactionally-recognized use-site-enabled Block
    /// (Issue #762). When the builder is not yet `BlockReferenceUseEnabled`,
    /// this is the first selected Block-contained use-site: every
    /// already-owned top-level item moves into the new fifth / broadest item
    /// representation in exact authored order -- without reparsing or
    /// reconstructing any already-produced historical `SelectedBlock` -- and
    /// the builder promotes to `BlockReferenceUseEnabled` exactly once
    /// before the new Block is appended. Once already
    /// `BlockReferenceUseEnabled`, the Block appends directly.
    fn push_use_site_enabled_block(
        &mut self,
        block: SelectedUseSiteEnabledBlock,
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
                    items.push(
                        SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(
                            declaration,
                        ),
                    );
                }
                items
                    .push(SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block));
                *builder = Self::BlockReferenceUseEnabled(items);
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
                            SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(
                                declaration,
                            ),
                        ),
                        SelectedTopLevelItem::Block(existing_block) => {
                            items.push(SelectedBlockReferenceUseEnabledTopLevelItem::Block(
                                existing_block,
                            ));
                        }
                    }
                }
                items
                    .push(SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block));
                *builder = Self::BlockReferenceUseEnabled(items);
                Ok(())
            }
            builder @ Self::VariableEnabled(_) => {
                let Self::VariableEnabled(existing_items) = builder else {
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
                        SelectedVariableTopLevelItem::LexicalDeclaration(declaration) => items
                            .push(
                                SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(
                                    declaration,
                                ),
                            ),
                        SelectedVariableTopLevelItem::Block(existing_block) => {
                            items.push(SelectedBlockReferenceUseEnabledTopLevelItem::Block(
                                existing_block,
                            ));
                        }
                        SelectedVariableTopLevelItem::VariableStatement(statement) => {
                            items.push(
                                SelectedBlockReferenceUseEnabledTopLevelItem::VariableStatement(
                                    statement,
                                ),
                            );
                        }
                    }
                }
                items
                    .push(SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block));
                *builder = Self::BlockReferenceUseEnabled(items);
                Ok(())
            }
            builder @ Self::ReferenceUseEnabled(_) => {
                let Self::ReferenceUseEnabled(existing_items) = builder else {
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
                    let converted = match item {
                        SelectedReferenceUseEnabledTopLevelItem::LexicalDeclaration(declaration) => {
                            SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(
                                declaration,
                            )
                        }
                        SelectedReferenceUseEnabledTopLevelItem::Block(existing_block) => {
                            SelectedBlockReferenceUseEnabledTopLevelItem::Block(existing_block)
                        }
                        SelectedReferenceUseEnabledTopLevelItem::VariableStatement(statement) => {
                            SelectedBlockReferenceUseEnabledTopLevelItem::VariableStatement(
                                statement,
                            )
                        }
                        SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                            fact,
                        ) => {
                            SelectedBlockReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                                fact,
                            )
                        }
                    };
                    items.push(converted);
                }
                items
                    .push(SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block));
                *builder = Self::BlockReferenceUseEnabled(items);
                Ok(())
            }
            Self::BlockReferenceUseEnabled(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items
                    .push(SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block));
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
            Self::ReferenceUseEnabled(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedReferenceUseEnabledTopLevelItem::VariableStatement(
                    statement,
                ));
                Ok(())
            }
            Self::BlockReferenceUseEnabled(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(
                    SelectedBlockReferenceUseEnabledTopLevelItem::VariableStatement(statement),
                );
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

/// Crate-private bounded cardinality carrier for a free-standing
/// `IdentifierReference` `ExpressionStatement` use-site occurrence, widening
/// the previous always-exactly-one-fact use-site payload to admit exactly
/// one additional selected additive operand (Issue #766). `One` represents
/// a free-standing use-site body retaining exactly one `IdentifierReference`
/// fact (Issue #793): a plain single-reference use-site (`a;`), or a
/// selected one-reference / one-plain-decimal heterogeneous additive
/// use-site (`a + 1;`, `1 + a;`) whose consumed-and-discarded Decimal
/// operand, binary operator, and left/right orientation are never retained.
/// `Two` retains
/// both authored operands of a selected
/// `SelectedTwoIdentifierReferenceAdditiveExpressionStatement` in exact
/// authored left-to-right order (`first` is the left operand, `second` is
/// the right operand). Three or more facts, a second fact without a first,
/// reordering, and deduplication are all unrepresentable by this type. This
/// is a distinct owner from `SelectedIdentifierReferenceInitializer`:
/// physical shape equivalence does not establish semantic ownership
/// equivalence, so this type is never constructed from, converted to, or
/// shared with that initializer-owned carrier -- a free-standing use-site
/// has no containing binding.
#[derive(Debug)]
pub(super) enum SelectedFreeStandingIdentifierReferenceUseSite {
    One(SelectedIdentifierReferenceFact),
    Two {
        first: SelectedIdentifierReferenceFact,
        second: SelectedIdentifierReferenceFact,
    },
}

impl SelectedFreeStandingIdentifierReferenceUseSite {
    /// Every retained fact, in exact authored left-to-right order: one item
    /// for `One`, two for `Two`. Backed by a fixed-size array, never a heap
    /// allocation.
    pub(super) fn facts(&self) -> impl Iterator<Item = &SelectedIdentifierReferenceFact> {
        match self {
            Self::One(fact) => [Some(fact), None],
            Self::Two { first, second } => [Some(first), Some(second)],
        }
        .into_iter()
        .flatten()
    }
}

/// Payload-free, placement-owned termination provenance for a TopLevel
/// free-standing `IdentifierReference` `ExpressionStatement` use-site (Issue
/// #768). `AuthoredSemicolon` proves only that the use-site was terminated
/// by an authored `;`. `AutomaticAtEof` proves only that the use-site's body
/// completed and the next source position, after currently selected trivia,
/// is actual end of input; it carries no authored or synthetic
/// `SourceAnchor` for the inserted semicolon and no EOF decision offset.
/// This is a distinct owner from `SelectedVariableStatementTerminator`
/// (physical shape equality does not establish semantic owner equality) and
/// from `SelectedBlockFreeStandingIdentifierReferenceUseSiteTerminator`: a
/// TopLevel use-site can never carry `AutomaticBeforeBlockClose`, which this
/// type cannot represent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedTopLevelFreeStandingIdentifierReferenceUseSiteTerminator {
    AuthoredSemicolon,
    AutomaticAtEof,
}

/// TopLevel free-standing `IdentifierReference` `ExpressionStatement`
/// use-site (Issue #768): the existing bounded `One`/`Two` occurrence body
/// (Issue #766) paired with this placement's own termination provenance.
/// Reference occurrence cardinality (`body`) and statement termination
/// provenance (`terminator`) are kept as two independent fields, never
/// crossed into a Cartesian variant set. Never shared with
/// `SelectedBlockIdentifierReferenceUseSite`: a TopLevel use-site can never
/// carry `AutomaticBeforeBlockClose`.
#[derive(Debug)]
pub(super) struct SelectedTopLevelIdentifierReferenceUseSite {
    body: SelectedFreeStandingIdentifierReferenceUseSite,
    terminator: SelectedTopLevelFreeStandingIdentifierReferenceUseSiteTerminator,
}

impl SelectedTopLevelIdentifierReferenceUseSite {
    pub(super) fn body(&self) -> &SelectedFreeStandingIdentifierReferenceUseSite {
        &self.body
    }

    pub(super) fn terminator(
        &self,
    ) -> SelectedTopLevelFreeStandingIdentifierReferenceUseSiteTerminator {
        self.terminator
    }
}

/// Payload-free, placement-owned termination provenance for a one-level
/// Block free-standing `IdentifierReference` `ExpressionStatement` use-site
/// (Issue #768). `AuthoredSemicolon` proves only that the use-site was
/// terminated by an authored `;`. `AutomaticBeforeBlockClose` proves only
/// that the use-site's body completed and the next significant source
/// position, after currently selected trivia, is the containing Block's
/// closing `}`; it carries no authored or synthetic `SourceAnchor` for the
/// inserted semicolon and no anchor for `}` itself, which remains the
/// enclosing Block's own authored syntax and is left unconsumed by this
/// use-site's parser. This is intentionally not shared with
/// `SelectedBlockVarStatementTerminator` (physical shape equality does not
/// establish semantic owner equality) or with
/// `SelectedTopLevelFreeStandingIdentifierReferenceUseSiteTerminator`: a
/// Block use-site can never carry `AutomaticAtEof`, which this type cannot
/// represent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedBlockFreeStandingIdentifierReferenceUseSiteTerminator {
    AuthoredSemicolon,
    AutomaticBeforeBlockClose,
}

/// One-level Block free-standing `IdentifierReference` `ExpressionStatement`
/// use-site (Issue #768): the existing bounded `One`/`Two` occurrence body
/// (Issue #766) paired with this placement's own termination provenance.
/// Reference occurrence cardinality (`body`) and statement termination
/// provenance (`terminator`) are kept as two independent fields, never
/// crossed into a Cartesian variant set. Never shared with
/// `SelectedTopLevelIdentifierReferenceUseSite`: a Block use-site can never
/// carry `AutomaticAtEof`.
#[derive(Debug)]
pub(super) struct SelectedBlockIdentifierReferenceUseSite {
    body: SelectedFreeStandingIdentifierReferenceUseSite,
    terminator: SelectedBlockFreeStandingIdentifierReferenceUseSiteTerminator,
}

impl SelectedBlockIdentifierReferenceUseSite {
    pub(super) fn body(&self) -> &SelectedFreeStandingIdentifierReferenceUseSite {
        &self.body
    }

    pub(super) fn terminator(
        &self,
    ) -> SelectedBlockFreeStandingIdentifierReferenceUseSiteTerminator {
        self.terminator
    }
}

/// Result of the bounded, transactional, placement-neutral free-standing
/// `IdentifierReference` `ExpressionStatement` use-site **body** probe
/// (Issue #758, generalized from a TopLevel-only name to this
/// placement-neutral leaf by Issue #762 per #688 comment 5734964743, widened
/// from an always-exactly-one retained fact to the bounded
/// `SelectedFreeStandingIdentifierReferenceUseSite` `One`/`Two` occurrence
/// carrier by Issue #766 per #688 comment 5739718987, narrowed from owning
/// the whole use-site including its authored-semicolon terminator to owning
/// only the placement-neutral body -- termination provenance moved entirely
/// to the placement-owned TopLevel/Block callers -- by Issue #768).
/// `NotSelected` covers every declining case uniformly: no candidate
/// reference; an escaped-ReservedWord first or second candidate; or a
/// `+`/`-` continuation whose second operand does not complete the whole
/// two-operand theorem. In every `NotSelected` case the cursor is left
/// exactly where it stood before the probe began -- including a `+`/`-`
/// continuation that begins to match but does not complete (e.g. `a+`) --
/// so the caller's own existing dispatch sees an unperturbed cursor and a
/// locally recognized prefix never authorizes a richer or longer source.
/// This is why this owner must not call
/// `consume_selected_identifier_reference_initializer`: that helper has
/// initializer-owned staged partial-recovery semantics (degrading a failed
/// continuation to a completed `One`/`Two` result) and, for a plain
/// three-operand chain, now fully composes a complete `Three(first, second,
/// third)` (Issue #797 per #688 comment 5771773635), which would incorrectly
/// authorize `a+b+c` as a complete free-standing Statement body; free-standing
/// use-sites use whole-body rollback instead and cannot reuse that owner
/// contract. `ResourceLimited`/
/// `InternalFailure` are propagated exactly, never downgraded to
/// `NotSelected`. `Matched` retains no terminator: a locally complete body
/// (`One` or `Two`) is not yet a complete use-site until the placement owner
/// also proves an authored `;`, actual EOF, or the containing `}` follows
/// (per `consume_selected_top_level_identifier_reference_expression_statement_use_site`
/// / `consume_selected_block_identifier_reference_expression_statement_use_site`);
/// a placement owner that finds none of those must roll back the whole
/// probe to its own pre-call snapshot and decline, exactly reproducing this
/// leaf's pre-#768 all-or-nothing transaction. This type and its producing
/// method carry no placement ownership themselves; TopLevel vs. Block
/// placement belongs entirely to the caller.
#[derive(Debug)]
enum SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition {
    Matched(SelectedFreeStandingIdentifierReferenceUseSite),
    NotSelected,
    ResourceLimited,
    InternalFailure,
}

/// Result of the placement-owned TopLevel free-standing `IdentifierReference`
/// `ExpressionStatement` use-site probe (Issue #768): the placement-neutral
/// body probe above, plus this placement's own
/// `AuthoredSemicolon`/`AutomaticAtEof` termination decision. `NotSelected`
/// covers a declining body probe unchanged, and additionally a body that
/// completed but whose immediate next position (after selected trivia) is
/// neither an authored `;` nor actual end of input -- in that case the
/// cursor is rolled back to exactly where it stood before the whole probe
/// began, so a locally recognized body never authorizes a richer or longer
/// source (e.g. `a.b`, `a+b+c`).
#[derive(Debug)]
enum SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition {
    Matched(SelectedTopLevelIdentifierReferenceUseSite),
    NotSelected,
    ResourceLimited,
    InternalFailure,
}

/// Result of the placement-owned one-level Block free-standing
/// `IdentifierReference` `ExpressionStatement` use-site probe (Issue #768):
/// the placement-neutral body probe above, plus this placement's own
/// `AuthoredSemicolon`/`AutomaticBeforeBlockClose` termination decision.
/// `NotSelected` covers a declining body probe unchanged, and additionally a
/// body that completed but whose immediate next position (after selected
/// trivia) is neither an authored `;` nor the containing Block's closing
/// `}` -- in that case the cursor is rolled back to exactly where it stood
/// before the whole probe began. `AutomaticBeforeBlockClose` never consumes
/// `}`: it remains the enclosing Block parser's own authored syntax.
#[derive(Debug)]
enum SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition {
    Matched(SelectedBlockIdentifierReferenceUseSite),
    NotSelected,
    ResourceLimited,
    InternalFailure,
}

/// Result of the owning single left-to-right Block parse lifecycle (Issue
/// #762): `Legacy` is the exact unchanged historical `SelectedBlock`,
/// produced when no Block-contained use-site commits. `UseSiteEnabled` is
/// the new representation, produced when at least one Block-contained
/// use-site commits. The Block source is read exactly once; this enum
/// distinguishes only which already-built in-memory representation the
/// single pass ends with, never a second parse, rescan, or reparse.
#[derive(Debug)]
enum SelectedBlockParseOutcome {
    Legacy(SelectedBlock),
    UseSiteEnabled(SelectedUseSiteEnabledBlock),
}

/// Block-local monotonic capability builder (Issue #762) driving
/// `Cursor::parse_selected_block`'s single owned pass: `Legacy` accumulates
/// exactly the existing historical `SelectedBlockItem` representation with
/// no extra allocation or conversion. On the first Block-contained
/// free-standing use-site, `push_use_site` promotes `Legacy` to
/// `ReferenceUseEnabled` exactly once, moving every already-owned legacy
/// item into the wider `SelectedUseSiteEnabledBlockItem` representation in
/// exact authored order; every subsequent item -- use-site, lexical
/// declaration, or Block `var` statement -- then appends directly to the
/// already-wide `Vec`. A Block that never exercises the new use-site
/// capability therefore finishes through `Legacy` with no wide staging
/// representation and no post-parse down-conversion allocation.
#[derive(Debug)]
enum SelectedBlockBuilder {
    Legacy(Vec<SelectedBlockItem>),
    ReferenceUseEnabled(Vec<SelectedUseSiteEnabledBlockItem>),
}

impl SelectedBlockBuilder {
    fn push_lexical_declaration(
        &mut self,
        declaration: SelectedLexicalDeclaration,
    ) -> Result<(), ParseFailure> {
        match self {
            Self::Legacy(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedBlockItem::LexicalDeclaration(declaration));
                Ok(())
            }
            Self::ReferenceUseEnabled(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedUseSiteEnabledBlockItem::LexicalDeclaration(
                    declaration,
                ));
                Ok(())
            }
        }
    }

    fn push_var_statement(
        &mut self,
        statement: SelectedBlockVarStatement,
    ) -> Result<(), ParseFailure> {
        match self {
            Self::Legacy(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedBlockItem::Var(statement));
                Ok(())
            }
            Self::ReferenceUseEnabled(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(SelectedUseSiteEnabledBlockItem::Var(statement));
                Ok(())
            }
        }
    }

    /// Commits the transactionally-recognized Block-contained use-site.
    /// When still `Legacy`, this is the first selected Block-contained
    /// use-site: every already-owned legacy item moves into the wider item
    /// representation in exact authored order before the use-site is
    /// appended, promoting to `ReferenceUseEnabled` exactly once. Once
    /// already `ReferenceUseEnabled`, the use-site appends directly.
    fn push_use_site(
        &mut self,
        use_site: SelectedBlockIdentifierReferenceUseSite,
    ) -> Result<(), ParseFailure> {
        match self {
            builder @ Self::Legacy(_) => {
                let Self::Legacy(existing_items) = builder else {
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
                        SelectedBlockItem::LexicalDeclaration(declaration) => {
                            items.push(SelectedUseSiteEnabledBlockItem::LexicalDeclaration(
                                declaration,
                            ));
                        }
                        SelectedBlockItem::Var(statement) => {
                            items.push(SelectedUseSiteEnabledBlockItem::Var(statement));
                        }
                    }
                }
                items.push(
                    SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(
                        use_site,
                    ),
                );
                *builder = Self::ReferenceUseEnabled(items);
                Ok(())
            }
            Self::ReferenceUseEnabled(items) => {
                items
                    .try_reserve(1)
                    .map_err(|_| ParseFailure::ResourceLimited)?;
                items.push(
                    SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(
                        use_site,
                    ),
                );
                Ok(())
            }
        }
    }
}

/// Result of the bounded 1-, 2-, or 3-operand `IdentifierReference`
/// initializer helper (Issue #754; widened to three operands by Issue #797
/// per #688 comment 5771773635), which absorbs the previous plain
/// `consume_selected_identifier_reference()` initializer route. `One`
/// carries the sole retained fact: either the exact existing
/// single-reference behavior unchanged, or (Issue #791) the first operand of
/// a selected two-syntax-operand `IdentifierReference`-then-plain-Decimal
/// additive initializer, whose Decimal second operand is consumed and
/// discarded rather than retained. `Two` carries both authored operands of a
/// selected `SelectedTwoIdentifierReferenceAdditiveInitializer` in exact
/// authored left-to-right order. `Three` carries all three authored operands
/// of a selected `SelectedExactlyThreeIdentifierReferenceAdditiveInitializer`
/// in exact authored left-to-right order, composing the accepted
/// candidate-independent theorem proven by #795/PR #796 for plain/plain/plain
/// operands only; it is reachable only when the second operand was plain
/// (never right-unary-wrapped). `EscapedReservedIdentifierName` is the
/// unchanged existing classification-only route for a first operand whose
/// decoded spelling is a ReservedWord (a ReservedWord is never an accepted
/// operand, so no additive continuation is attempted for it). `NotSelected`
/// covers no `IdentifierReference` operand at all. `ResourceLimited` and
/// `InternalFailure` preserve the shared recognizer's own processing-failure
/// classes for any operand, never downgraded to `NotSelected` or to a
/// completed `One`/`Two` result.
#[derive(Debug)]
enum SelectedIdentifierReferenceInitializerRecognition {
    One(SelectedIdentifierReferenceFact),
    Two {
        first: SelectedIdentifierReferenceFact,
        second: SelectedIdentifierReferenceFact,
    },
    Three {
        first: SelectedIdentifierReferenceFact,
        second: SelectedIdentifierReferenceFact,
        third: SelectedIdentifierReferenceFact,
    },
    EscapedReservedIdentifierName {
        identifier: SourceAnchor,
    },
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

/// Result of the bounded plain-Decimal-atom initializer helper composing
/// the Decimal-left orientation of the one-reference / one-plain-decimal
/// additive initializer theorem (Issue #791, per #688 comment 5762579228).
/// `NotSelected` covers no accepted plain Decimal atom at all, leaving the
/// unmodified Boolean/null/`this`/`String`/`IdentifierReference`
/// predecessors free to recognize their own initializer syntax.
/// `DecimalOnly` is the exact existing presence-only accepted Decimal-atom
/// initializer, unchanged: no additive continuation was authored, or the
/// probed continuation declined. `DecimalWithReference` carries only the
/// second operand's `SelectedIdentifierReferenceFact`; the Decimal atom, the
/// binary operator, and the orientation are never retained. `ResourceLimited`
/// and `InternalFailure` preserve the shared `IdentifierReference`
/// recognizer's own processing-failure classes and are never downgraded to
/// `DecimalOnly`.
#[derive(Debug)]
enum SelectedPlainDecimalAtomInitializerRecognition {
    NotSelected,
    DecimalOnly,
    DecimalWithReference(SelectedIdentifierReferenceFact),
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

    /// Owning single left-to-right Block parse lifecycle (Issue #762,
    /// widened from an authored-semicolon-only Block-contained use-site
    /// terminator to also admit before-`}` automatic termination by Issue
    /// #768). The Block source is read exactly once: inside the loop, the
    /// bounded Block-owned free-standing use-site probe
    /// (`consume_selected_block_identifier_reference_expression_statement_use_site`,
    /// which recognizes the placement-neutral body and then decides this
    /// placement's own `AuthoredSemicolon`/`AutomaticBeforeBlockClose`
    /// termination, only ever peeking `}`) runs transactionally before the
    /// existing raw Block `var` / lexical-declaration dispatch, so
    /// `{ let; }` / `{ varfoo; }` become use-sites while `{ let a; }` /
    /// `{ var a; }` remain owned by the existing declaration dispatch
    /// exactly as before. Every recognized
    /// item is committed directly to the `SelectedBlockBuilder`
    /// Block-local monotonic capability builder: `Legacy` while no
    /// Block-contained use-site has committed, promoting to
    /// `ReferenceUseEnabled` exactly once on the first one. A Block that
    /// never exercises the new capability builds and returns the exact
    /// historical `SelectedBlock` directly from its `Legacy` items, with no
    /// wide staging representation and no post-parse down-conversion
    /// allocation. No second tokenizer, parser, rescan, or source search
    /// recovers the Block or reference endpoints -- both come from the same
    /// owned cursor lifecycle used to anchor every item.
    fn parse_selected_block(&mut self) -> Result<SelectedBlockParseOutcome, ParseFailure> {
        let block_start = self.offset;
        if !self.consume_ascii('{') {
            return Err(ParseFailure::UnsupportedCoverage);
        }
        self.skip_selected_trivia();

        if self.peek_char() == Some('}') {
            return Err(ParseFailure::UnsupportedCoverage);
        }

        let mut builder = SelectedBlockBuilder::Legacy(Vec::new());
        loop {
            match self.consume_selected_block_identifier_reference_expression_statement_use_site() {
                SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition::Matched(
                    use_site,
                ) => {
                    builder.push_use_site(use_site)?;
                }
                SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition::ResourceLimited => {
                    return Err(ParseFailure::ResourceLimited);
                }
                SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition::InternalFailure => {
                    return Err(ParseFailure::InternalFailure);
                }
                SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition::NotSelected => {
                    if self.remaining().starts_with("var") {
                        let statement = self.parse_selected_block_var_statement()?;
                        builder.push_var_statement(statement)?;
                    } else {
                        let declaration = self.parse_declaration()?;
                        builder.push_lexical_declaration(declaration)?;
                    }
                }
            }

            self.skip_selected_trivia();
            if self.consume_ascii('}') {
                break;
            }
            if self.is_eof() {
                return Err(ParseFailure::UnsupportedCoverage);
            }
        }

        let block = self.anchor(block_start, self.offset)?;

        Ok(match builder {
            SelectedBlockBuilder::Legacy(items) => {
                SelectedBlockParseOutcome::Legacy(SelectedBlock { block, items })
            }
            SelectedBlockBuilder::ReferenceUseEnabled(items) => {
                SelectedBlockParseOutcome::UseSiteEnabled(SelectedUseSiteEnabledBlock {
                    block,
                    items,
                })
            }
        })
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
                                let initializer = self
                                    .consume_selected_leading_plus_minus_identifier_reference_left_additive_initializer(reference)?;
                                (Some(initializer), None)
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::ResourceLimited => {
                                return Err(ParseFailure::ResourceLimited);
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::InternalFailure => {
                                return Err(ParseFailure::InternalFailure);
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::NotSelected => {
                                match self.consume_selected_plain_decimal_atom_initializer() {
                                    SelectedPlainDecimalAtomInitializerRecognition::DecimalOnly => {
                                        (None, None)
                                    }
                                    SelectedPlainDecimalAtomInitializerRecognition::DecimalWithReference(reference) => {
                                        (Some(SelectedIdentifierReferenceInitializer::One(reference)), None)
                                    }
                                    SelectedPlainDecimalAtomInitializerRecognition::ResourceLimited => {
                                        return Err(ParseFailure::ResourceLimited);
                                    }
                                    SelectedPlainDecimalAtomInitializerRecognition::InternalFailure => {
                                        return Err(ParseFailure::InternalFailure);
                                    }
                                    SelectedPlainDecimalAtomInitializerRecognition::NotSelected => {
                                        if self.consume_selected_boolean_literal()
                                            || self.consume_selected_null_literal()
                                            || self.consume_selected_this_expression()
                                            || self.consume_selected_escape_free_string_literal()
                                        {
                                            (None, None)
                                        } else {
                                            match self.consume_selected_identifier_reference_initializer() {
                                                SelectedIdentifierReferenceInitializerRecognition::One(reference) => {
                                                    (Some(SelectedIdentifierReferenceInitializer::One(reference)), None)
                                                }
                                                SelectedIdentifierReferenceInitializerRecognition::Two { first, second } => {
                                                    (
                                                        Some(SelectedIdentifierReferenceInitializer::Two { first, second }),
                                                        None,
                                                    )
                                                }
                                                SelectedIdentifierReferenceInitializerRecognition::Three { first, second, third } => {
                                                    (
                                                        Some(SelectedIdentifierReferenceInitializer::Three { first, second, third }),
                                                        None,
                                                    )
                                                }
                                                SelectedIdentifierReferenceInitializerRecognition::EscapedReservedIdentifierName {
                                                    identifier,
                                                } => (None, Some(identifier)),
                                                SelectedIdentifierReferenceInitializerRecognition::NotSelected => {
                                                    return Err(ParseFailure::UnsupportedCoverage);
                                                }
                                                SelectedIdentifierReferenceInitializerRecognition::ResourceLimited => {
                                                    return Err(ParseFailure::ResourceLimited);
                                                }
                                                SelectedIdentifierReferenceInitializerRecognition::InternalFailure => {
                                                    return Err(ParseFailure::InternalFailure);
                                                }
                                            }
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
                                let initializer = self
                                    .consume_selected_leading_plus_minus_identifier_reference_left_additive_initializer(reference)?;
                                (Some(initializer), None)
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::ResourceLimited => {
                                return Err(ParseFailure::ResourceLimited);
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::InternalFailure => {
                                return Err(ParseFailure::InternalFailure);
                            }
                            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::NotSelected => {
                                match self.consume_selected_plain_decimal_atom_initializer() {
                                    SelectedPlainDecimalAtomInitializerRecognition::DecimalOnly => {
                                        (None, None)
                                    }
                                    SelectedPlainDecimalAtomInitializerRecognition::DecimalWithReference(reference) => {
                                        (Some(SelectedIdentifierReferenceInitializer::One(reference)), None)
                                    }
                                    SelectedPlainDecimalAtomInitializerRecognition::ResourceLimited => {
                                        return Err(ParseFailure::ResourceLimited);
                                    }
                                    SelectedPlainDecimalAtomInitializerRecognition::InternalFailure => {
                                        return Err(ParseFailure::InternalFailure);
                                    }
                                    SelectedPlainDecimalAtomInitializerRecognition::NotSelected => {
                                        if self.consume_selected_boolean_literal()
                                            || self.consume_selected_null_literal()
                                            || self.consume_selected_this_expression()
                                            || self.consume_selected_escape_free_string_literal()
                                        {
                                            (None, None)
                                        } else {
                                            match self.consume_selected_identifier_reference_initializer() {
                                                SelectedIdentifierReferenceInitializerRecognition::One(reference) => {
                                                    (Some(SelectedIdentifierReferenceInitializer::One(reference)), None)
                                                }
                                                SelectedIdentifierReferenceInitializerRecognition::Two { first, second } => {
                                                    (
                                                        Some(SelectedIdentifierReferenceInitializer::Two { first, second }),
                                                        None,
                                                    )
                                                }
                                                SelectedIdentifierReferenceInitializerRecognition::Three { first, second, third } => {
                                                    (
                                                        Some(SelectedIdentifierReferenceInitializer::Three { first, second, third }),
                                                        None,
                                                    )
                                                }
                                                SelectedIdentifierReferenceInitializerRecognition::EscapedReservedIdentifierName {
                                                    identifier,
                                                } => (None, Some(identifier)),
                                                SelectedIdentifierReferenceInitializerRecognition::NotSelected => {
                                                    return Err(ParseFailure::UnsupportedCoverage);
                                                }
                                                SelectedIdentifierReferenceInitializerRecognition::ResourceLimited => {
                                                    return Err(ParseFailure::ResourceLimited);
                                                }
                                                SelectedIdentifierReferenceInitializerRecognition::InternalFailure => {
                                                    return Err(ParseFailure::InternalFailure);
                                                }
                                            }
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
                            identifier_reference_initializer = Some(
                                self.consume_selected_leading_plus_minus_identifier_reference_left_additive_initializer(reference)?,
                            );
                        }
                        SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::ResourceLimited => {
                            return Err(ParseFailure::ResourceLimited);
                        }
                        SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::InternalFailure => {
                            return Err(ParseFailure::InternalFailure);
                        }
                        SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::NotSelected => {
                            match self.consume_selected_plain_decimal_atom_initializer() {
                                SelectedPlainDecimalAtomInitializerRecognition::DecimalOnly => {}
                                SelectedPlainDecimalAtomInitializerRecognition::DecimalWithReference(reference) => {
                                    identifier_reference_initializer =
                                        Some(SelectedIdentifierReferenceInitializer::One(reference));
                                }
                                SelectedPlainDecimalAtomInitializerRecognition::ResourceLimited => {
                                    return Err(ParseFailure::ResourceLimited);
                                }
                                SelectedPlainDecimalAtomInitializerRecognition::InternalFailure => {
                                    return Err(ParseFailure::InternalFailure);
                                }
                                SelectedPlainDecimalAtomInitializerRecognition::NotSelected => {
                                    if !self.consume_selected_boolean_literal()
                                        && !self.consume_selected_null_literal()
                                        && !self.consume_selected_this_expression()
                                        && !self.consume_selected_escape_free_string_literal()
                                    {
                                        match self.consume_selected_identifier_reference_initializer() {
                                            SelectedIdentifierReferenceInitializerRecognition::One(reference) => {
                                                identifier_reference_initializer =
                                                    Some(SelectedIdentifierReferenceInitializer::One(reference));
                                            }
                                            SelectedIdentifierReferenceInitializerRecognition::Two { first, second } => {
                                                identifier_reference_initializer =
                                                    Some(SelectedIdentifierReferenceInitializer::Two { first, second });
                                            }
                                            SelectedIdentifierReferenceInitializerRecognition::Three { first, second, third } => {
                                                identifier_reference_initializer =
                                                    Some(SelectedIdentifierReferenceInitializer::Three { first, second, third });
                                            }
                                            SelectedIdentifierReferenceInitializerRecognition::EscapedReservedIdentifierName {
                                                identifier,
                                            } => {
                                                escaped_reserved_initializer_identifier = Some(identifier);
                                            }
                                            SelectedIdentifierReferenceInitializerRecognition::NotSelected => {
                                                return Err(ParseFailure::UnsupportedCoverage);
                                            }
                                            SelectedIdentifierReferenceInitializerRecognition::ResourceLimited => {
                                                return Err(ParseFailure::ResourceLimited);
                                            }
                                            SelectedIdentifierReferenceInitializerRecognition::InternalFailure => {
                                                return Err(ParseFailure::InternalFailure);
                                            }
                                        }
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

    /// Bounded, transactional, placement-neutral free-standing
    /// `IdentifierReference` `ExpressionStatement` use-site **body** probe
    /// (Issue #758, generalized to this placement-neutral leaf by Issue #762
    /// per #688 comment 5734964743 -- both TopLevel and Block placement are
    /// now independently justified callers of this exact same bounded
    /// grammar -- widened by Issue #766 per #688 comment 5739718987 to
    /// additionally accept exactly one authored `+`/`-` continuation,
    /// narrowed by Issue #768 to own only the body -- termination provenance
    /// moved entirely to the placement-owned TopLevel/Block callers -- widened
    /// by Issue #771 per #688 comment 5744036288 to additionally accept a
    /// leading `+`/`-` `IdentifierReference` `UnaryExpression`, composing the
    /// already-accepted bounded
    /// `consume_selected_leading_plus_minus_identifier_reference_unary_expression`
    /// helper -- unchanged, never reimplemented here -- as one additional
    /// bounded body form tried before the existing bare/additive body logic
    /// -- widened by Issue #773 per #688 comment 5744425380 to compose that
    /// same leading `+`/`-` `IdentifierReference` `UnaryExpression` with an
    /// optional exactly-one authored binary `+`/`-` plain `IdentifierReference`
    /// continuation -- and widened by Issue #787 per #688 comment 5759154638
    /// to additionally admit an optional right-unary `+`/`-` wrapper on that
    /// same continuation's second operand, reusing the candidate-independent
    /// binary/right-unary token-boundary theorem accepted by #777/#778,
    /// exactly mirroring the bare-reference-first widening Issue #781 already
    /// applied below):
    ///
    /// ```text
    /// SelectedIdentifierReferenceExpressionStatementUseSiteBody ::=
    ///     SelectedLeadingPlusMinusIdentifierReferenceLeftAdditiveUseSiteBody
    ///   | SelectedLeadingPlusMinusIdentifierReferenceFreeStandingUseSiteBody
    ///   | SelectedAcceptedIdentifierReference
    ///   | SelectedTwoIdentifierReferenceAdditiveExpressionStatementBody
    ///
    /// SelectedLeadingPlusMinusIdentifierReferenceLeftAdditiveUseSiteBody ::=
    ///     SelectedLeadingPlusMinusIdentifierReferenceUnaryExpression
    ///     SelectedAdditiveTrivia
    ///     SelectedBinaryPlusMinus
    ///     SelectedAdditiveTrivia
    ///     (
    ///         SelectedAcceptedIdentifierReference
    ///       |
    ///         SelectedBinaryUnaryBoundary
    ///         SelectedUnaryPlusMinus
    ///         SelectedUnaryOperandTrivia
    ///         SelectedAcceptedIdentifierReference
    ///     )
    ///
    /// SelectedLeadingPlusMinusIdentifierReferenceFreeStandingUseSiteBody ::=
    ///     SelectedLeadingPlusMinusIdentifierReferenceUnaryExpression
    ///
    /// SelectedLeadingPlusMinusIdentifierReferenceUnaryExpression ::=
    ///     SelectedUnaryPlusMinus
    ///     SelectedUnaryOperandTrivia
    ///     SelectedAcceptedIdentifierReference
    ///
    /// SelectedTwoIdentifierReferenceAdditiveExpressionStatementBody ::=
    ///     SelectedAcceptedIdentifierReference
    ///     SelectedAdditiveTrivia
    ///     ("+" | "-")
    ///     SelectedAdditiveTrivia
    ///     SelectedAcceptedIdentifierReference
    /// ```
    ///
    /// Each operand is recognized exactly once by the unmodified shared
    /// `consume_selected_identifier_reference` recognizer -- no second
    /// scanner or decoder. After the first operand, selected trivia is
    /// skipped and, absent an authored `+` or `-` at that position, the
    /// probe commits the unchanged single-reference `One` occurrence
    /// immediately -- this body helper claims no terminator, so the caller
    /// decides whether what follows is a valid use-site ending. Otherwise
    /// exactly one authored `+` or `-` is consumed and discarded (no
    /// operator kind, `SourceAnchor`, or whole-expression anchor is
    /// retained), selected trivia is skipped, and a second operand is
    /// recognized by the same unmodified shared recognizer; once accepted,
    /// selected trivia is skipped once more and the probe commits
    /// `Two { first, second }`.
    ///
    /// Issue #781 widens the plain bare-reference-first continuation with one
    /// optional right-unary `+`/`-`. Opposite binary and unary signs may be
    /// adjacent, while equal signs require non-empty selected trivia, so
    /// `a+-b`, `a-+b`, `a+ +b`, and `a- -b` select without splitting `++` or
    /// `--`. Issue #787 applies this identical boundary theorem to the
    /// leading-unary-first route above (`+a+-b`, `+a-+b`, `+a+ +b`,
    /// `+a- -b`), so both continuations now admit the same bounded
    /// right-unary-wrapped second operand -- closing the final both-unary
    /// cell of the bounded exactly-two matrix. In either continuation the
    /// signs and inter-operator trivia cardinality are transient recognition
    /// state only, compared locally and then dropped; the retained result
    /// remains the unchanged `Two { first, second }` carrier, and at most one
    /// right-unary wrapper is admitted -- no recursion -- so `+a+-+b` and
    /// `+a-+-b` still fail the second operand and decline.
    ///
    /// Only a failing continuation declines (`NotSelected`), restoring the
    /// cursor to exactly where it stood before this probe began: no
    /// candidate reference; an escaped-ReservedWord first or second
    /// candidate; or a `+`/`-` continuation (plain or right-unary-wrapped)
    /// whose second operand does not complete. This is a whole-body
    /// transaction, not a first-operand-then-optional-continuation
    /// transaction, and it deliberately does not reuse the initializer's
    /// degrade-to-`One`-on-failure recovery model (Issue #785/#786): a
    /// locally recognized `IdentifierReference "+" IdentifierReference`
    /// prefix, with or without a right-unary wrapper, never authorizes a
    /// richer or longer source (`a+b+c`, `+a+b+c`, `+a+-b+c`), and an
    /// escaped-ReservedWord operand never gains a new Statement-local
    /// EE-04-R08 route. A matched body that the placement-owned caller
    /// cannot terminate validly (e.g. `a.b`, `+a+b+c`) is rolled back by that
    /// caller to the same pre-probe snapshot, reproducing this leaf's
    /// pre-#768 all-or-nothing decline for such input exactly.
    ///
    /// This deliberately does not call
    /// `consume_selected_identifier_reference_initializer`: that helper has
    /// initializer-owned staged partial-recovery semantics, intentionally
    /// degrading a failed continuation to a completed `One`/`Two` result so
    /// the enclosing binding/comma transaction can recover the remainder,
    /// and, for a plain three-operand chain, now fully composes a complete
    /// `Three(first, second, third)` (Issue #797 per #688 comment
    /// 5771773635), which would incorrectly authorize `a+b+c` as a complete
    /// free-standing Statement body. Free-standing use-sites use whole-body
    /// rollback instead and cannot reuse that owner contract; only the
    /// lexical primitive `consume_selected_identifier_reference` is shared,
    /// never the owner transaction.
    ///
    /// `ResourceLimited`/`InternalFailure` from either operand's recognition
    /// are propagated exactly, never downgraded to `NotSelected` or to a
    /// completed `One`. This helper retains no placement ownership of its
    /// own: TopLevel vs. Block placement belongs entirely to the caller.
    ///
    /// Issue #771 (widened by Issue #773, then by Issue #787): the unmodified
    /// `consume_selected_leading_plus_minus_identifier_reference_unary_expression`
    /// helper is probed first, before the bare/additive logic below. A
    /// `Matched(first)` result no longer commits `One(first)` unconditionally:
    /// selected trivia is skipped and, absent an authored `+` or `-` at that
    /// position, `One(first)` still commits immediately exactly as before
    /// #773. Otherwise exactly one authored binary `+` or `-` is consumed and
    /// discarded (no operator kind or `SourceAnchor` retained), selected
    /// trivia is skipped, and the same #777/#778 binary/right-unary boundary
    /// applied below governs an optional right-unary sign at that position
    /// (Issue #787): opposite binary and unary signs may be adjacent with no
    /// inter-operator trivia (`+a+-b`, `+a-+b`), while equal signs require
    /// non-empty inter-operator selected trivia (`+a+ +b`, `+a- -b`), so
    /// authored `++`/`--` is never split into a binary sign plus a
    /// right-unary sign (`+a++b`, `+a--b` remain outside); at most one
    /// right-unary wrapper is admitted, so `+a+-+b` still fails the second
    /// operand. When present, the right-unary sign is consumed and discarded
    /// before the shared recognizer runs, so it is never part of the second
    /// operand's authored `SourceAnchor`. A second operand is then recognized
    /// by the same unmodified shared `consume_selected_identifier_reference`
    /// recognizer; once accepted, selected trivia is skipped once more and
    /// the probe commits `Two { first, second }` -- the exact same carrier
    /// and left-to-right ordering the plain bare/additive route below
    /// produces, never a new item or body variant. A declining or
    /// escaped-ReservedWord second operand restores the cursor to this whole
    /// body probe's original pre-unary snapshot and declines (`NotSelected`),
    /// never degrading to `One(first)`: `+a+`, `+a+\u0069f`, and `+a+-+b` are
    /// not valid prefixes of a richer unselected syntax. No operator, trivia,
    /// or whole-unary/whole-additive `SourceAnchor` is retained; the authored
    /// `+`/`-` spellings are construction syntax only. Selected trivia is
    /// skipped once more after the final committed operand in either case,
    /// mirroring the existing bare-reference route below (which already
    /// skips trailing trivia before its own `One(first)`/`Two` returns): the
    /// placement-owned caller's terminator check (an authored `;`,
    /// `self.is_eof()`, or a peeked `}`) must observe the cursor positioned
    /// exactly after any trailing selected trivia, not merely after the last
    /// authored reference character. A `ResourceLimited`/`InternalFailure`
    /// classification from the unary helper, or from the second-operand
    /// recognizer reached through it, is propagated immediately, without
    /// falling through to the bare/additive route below. `NotSelected` from
    /// the unary helper itself falls through unchanged: the unary helper
    /// already restores `self.offset` to this probe's own snapshot on
    /// decline, so the existing bare/additive logic observes exactly the
    /// same starting position it always has. The unary helper's own local
    /// match (e.g. `+a` inside `+a+b`) never commits this whole body probe by
    /// itself -- the placement-owned caller's termination probe remains
    /// solely responsible for rejecting and rolling back a locally
    /// recognized unary atom, or unary-plus-binary pair, that is not
    /// followed by a valid use-site terminator. Unary-operator recursion
    /// (`++a+b`, `!a+b`), recursive right-unary wrapping (`+a+-+b`), and a
    /// third or later additive operand (`+a+b+c`) remain outside this leaf.
    ///
    /// Issue #793 (per #688 comment 5764090454) widens this leaf with the
    /// bounded one-reference / one-plain-decimal heterogeneous additive
    /// theorem already accepted for the initializer position by Issue #791,
    /// composing the candidate-independent theorem accepted by #789/#790.
    /// Two intentionally asymmetric routes compose the same retained
    /// `One(reference)` result:
    ///
    /// Reference-left (`a + 1;`): only the plain bare-reference-first route
    /// below is widened -- never the leading-unary-first route above, which
    /// stays hard-zero for this leaf, so `+a + 1;`/`-a + 1;` remain outside.
    /// When the plain route's second-operand `IdentifierReference` probe
    /// declines *and* no right-unary `+`/`-` wrapper was consumed for this
    /// continuation, one accepted plain Decimal atom is tried, via the
    /// existing unmodified `consume_selected_plain_exponent_decimal_literal`
    /// / `consume_selected_plain_fractional_decimal_literal` /
    /// `consume_selected_decimal_integer` helpers in that order, at the
    /// cursor position immediately after the binary operator and its
    /// trivia. A right-unary wrapper having been consumed forecloses the
    /// Decimal fallback unconditionally, so `a+-1;`/`a-+1;`/`a+ +1;`/`a- -1;`
    /// stay outside exactly like their initializer counterparts. A matched
    /// Decimal atom commits `One(first)` with the cursor left after its own
    /// trailing selected trivia; a declining Decimal atom (or a
    /// right-unary-guarded decline) restores the cursor to this whole body
    /// probe's original snapshot and returns `NotSelected` -- never `Two`,
    /// and never a degrade-to-`One` partial rollback.
    ///
    /// Decimal-left (`1 + a;`): when the bare first-operand
    /// `IdentifierReference` probe declines outright (not an
    /// escaped-ReservedWord decline), a distinct free-standing-local route,
    /// `consume_selected_plain_decimal_atom_identifier_reference_free_standing_use_site_body`,
    /// is tried from this same snapshot.
    ///
    /// Both routes deliberately do not call the initializer-owned
    /// `consume_selected_plain_decimal_atom_initializer` (Issue #791): that
    /// helper may commit a locally successful `DecimalOnly` result when its
    /// continuation declines, which would incorrectly let a bare Decimal
    /// atom (`1;`) or a truncated richer chain (`1 + a + 2;` up through
    /// `1`) become a complete free-standing use-site. Only the lower-level
    /// Decimal atom recognizers are shared; the transaction owner is not.
    /// Either route's Decimal atom is consumed and discarded -- never
    /// retained as a fact, operator, or orientation -- and a
    /// `ResourceLimited`/`InternalFailure` classification from either
    /// route's `IdentifierReference` operand is propagated immediately,
    /// never downgraded to a completed `One` or to `NotSelected`.
    fn consume_selected_identifier_reference_expression_statement_use_site_body(
        &mut self,
    ) -> SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition {
        let snapshot = self.offset;

        match self.consume_selected_leading_plus_minus_identifier_reference_unary_expression() {
            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::Matched(
                first,
            ) => {
                self.skip_selected_trivia();

                let binary_sign = if self.consume_ascii('+') {
                    '+'
                } else if self.consume_ascii('-') {
                    '-'
                } else {
                    return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::Matched(
                        SelectedFreeStandingIdentifierReferenceUseSite::One(first),
                    );
                };

                let before_inter_operator_trivia = self.offset;
                self.skip_selected_trivia();

                if let Some(unary_sign @ ('+' | '-')) = self.peek_char() {
                    if unary_sign == binary_sign && self.offset == before_inter_operator_trivia {
                        self.offset = snapshot;
                        return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::NotSelected;
                    }

                    let _ = self.advance_char();
                    self.skip_selected_trivia();
                }

                let second = match self.consume_selected_identifier_reference() {
                    SelectedIdentifierReferenceRecognition::Matched(fact) => fact,
                    SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName { .. }
                    | SelectedIdentifierReferenceRecognition::NotSelected => {
                        self.offset = snapshot;
                        return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::NotSelected;
                    }
                    SelectedIdentifierReferenceRecognition::ResourceLimited => {
                        return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::ResourceLimited;
                    }
                    SelectedIdentifierReferenceRecognition::InternalFailure => {
                        return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::InternalFailure;
                    }
                };

                self.skip_selected_trivia();

                return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::Matched(
                    SelectedFreeStandingIdentifierReferenceUseSite::Two { first, second },
                );
            }
            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::ResourceLimited => {
                return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::ResourceLimited;
            }
            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::InternalFailure => {
                return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::InternalFailure;
            }
            SelectedLeadingPlusMinusIdentifierReferenceUnaryExpressionRecognition::NotSelected => {}
        }

        let first = match self.consume_selected_identifier_reference() {
            SelectedIdentifierReferenceRecognition::Matched(fact) => fact,
            SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName { .. } => {
                self.offset = snapshot;
                return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::NotSelected;
            }
            SelectedIdentifierReferenceRecognition::NotSelected => {
                return self
                    .consume_selected_plain_decimal_atom_identifier_reference_free_standing_use_site_body(
                        snapshot,
                    );
            }
            SelectedIdentifierReferenceRecognition::ResourceLimited => {
                return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::ResourceLimited;
            }
            SelectedIdentifierReferenceRecognition::InternalFailure => {
                return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::InternalFailure;
            }
        };

        self.skip_selected_trivia();

        let binary_sign = if self.consume_ascii('+') {
            '+'
        } else if self.consume_ascii('-') {
            '-'
        } else {
            return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::Matched(
                SelectedFreeStandingIdentifierReferenceUseSite::One(first),
            );
        };

        let before_inter_operator_trivia = self.offset;
        self.skip_selected_trivia();

        let mut right_unary_consumed = false;
        if let Some(unary_sign @ ('+' | '-')) = self.peek_char() {
            if unary_sign == binary_sign && self.offset == before_inter_operator_trivia {
                self.offset = snapshot;
                return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::NotSelected;
            }

            let _ = self.advance_char();
            self.skip_selected_trivia();
            right_unary_consumed = true;
        }

        let after_operator = self.offset;
        match self.consume_selected_identifier_reference() {
            SelectedIdentifierReferenceRecognition::Matched(second) => {
                self.skip_selected_trivia();
                SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::Matched(
                    SelectedFreeStandingIdentifierReferenceUseSite::Two { first, second },
                )
            }
            SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName { .. }
            | SelectedIdentifierReferenceRecognition::NotSelected => {
                self.offset = after_operator;
                if !right_unary_consumed
                    && (self.consume_selected_plain_exponent_decimal_literal()
                        || self.consume_selected_plain_fractional_decimal_literal()
                        || self.consume_selected_decimal_integer())
                {
                    self.skip_selected_trivia();
                    return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::Matched(
                        SelectedFreeStandingIdentifierReferenceUseSite::One(first),
                    );
                }
                self.offset = snapshot;
                SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::NotSelected
            }
            SelectedIdentifierReferenceRecognition::ResourceLimited => {
                SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::ResourceLimited
            }
            SelectedIdentifierReferenceRecognition::InternalFailure => {
                SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::InternalFailure
            }
        }
    }

    /// Composes the Decimal-left orientation of the bounded one-reference /
    /// one-plain-decimal heterogeneous additive free-standing use-site
    /// theorem (Issue #793, per #688 comment 5764090454), reusing the
    /// candidate-independent theorem accepted by #789/#790:
    ///
    /// ```text
    /// SelectedPlainDecimalAtomIdentifierReferenceFreeStandingUseSiteBody ::=
    ///     SelectedAcceptedPlainDecimalAtom
    ///     SelectedAdditiveTrivia
    ///     ("+" | "-")
    ///     SelectedAdditiveTrivia
    ///     SelectedAcceptedIdentifierReference
    /// ```
    ///
    /// Called only from
    /// `consume_selected_identifier_reference_expression_statement_use_site_body`,
    /// with `body_snapshot` the exact offset that whole body probe started
    /// from (never merely this route's own local starting offset, since none
    /// exists separately) and the cursor already positioned there. The
    /// Decimal atom is recognized exactly once, via the existing unmodified
    /// `consume_selected_plain_exponent_decimal_literal` /
    /// `consume_selected_plain_fractional_decimal_literal` /
    /// `consume_selected_decimal_integer` helpers in that exact order, never
    /// rescanned; no accepted atom at all restores `body_snapshot` and
    /// declines (`NotSelected`) immediately, leaving the caller's other
    /// routes free to recognize the same source.
    ///
    /// There is no Decimal-only free-standing predecessor. This deliberately
    /// does not call the initializer-owned
    /// `consume_selected_plain_decimal_atom_initializer` (Issue #791): that
    /// helper's continuation may degrade to a locally successful
    /// `DecimalOnly` result when the reference continuation declines, which
    /// would incorrectly authorize a bare Decimal atom (`1;`) or a truncated
    /// richer chain (`1 + a + 2;` up through `1`) as a complete free-standing
    /// use-site. Physical lexical similarity does not establish
    /// transaction-owner equivalence, so only the lower-level Decimal atom
    /// recognizers are shared here; every decline in this route -- an absent
    /// binary `+`/`-`, or a declining/escaped-ReservedWord/absent
    /// `IdentifierReference` operand -- restores the cursor to exactly
    /// `body_snapshot` and returns `NotSelected`, never a partial commit.
    ///
    /// A matched `IdentifierReference` operand (via the same unmodified
    /// shared `consume_selected_identifier_reference` recognizer, never a
    /// second scanner/decoder) commits
    /// `SelectedFreeStandingIdentifierReferenceUseSite::One` carrying only
    /// that operand's existing `SelectedIdentifierReferenceFact`, with the
    /// cursor left after its own trailing selected trivia; the Decimal atom,
    /// the operator, and the orientation are never retained. A
    /// `ResourceLimited`/`InternalFailure` classification from that
    /// operand's recognition is propagated immediately and never downgraded
    /// to a completed `One` or to `NotSelected`.
    fn consume_selected_plain_decimal_atom_identifier_reference_free_standing_use_site_body(
        &mut self,
        body_snapshot: usize,
    ) -> SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition {
        if !(self.consume_selected_plain_exponent_decimal_literal()
            || self.consume_selected_plain_fractional_decimal_literal()
            || self.consume_selected_decimal_integer())
        {
            self.offset = body_snapshot;
            return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::NotSelected;
        }

        self.skip_selected_trivia();

        if !(self.consume_ascii('+') || self.consume_ascii('-')) {
            self.offset = body_snapshot;
            return SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::NotSelected;
        }

        self.skip_selected_trivia();

        match self.consume_selected_identifier_reference() {
            SelectedIdentifierReferenceRecognition::Matched(reference) => {
                self.skip_selected_trivia();
                SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::Matched(
                    SelectedFreeStandingIdentifierReferenceUseSite::One(reference),
                )
            }
            SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName { .. }
            | SelectedIdentifierReferenceRecognition::NotSelected => {
                self.offset = body_snapshot;
                SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::NotSelected
            }
            SelectedIdentifierReferenceRecognition::ResourceLimited => {
                SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::ResourceLimited
            }
            SelectedIdentifierReferenceRecognition::InternalFailure => {
                SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::InternalFailure
            }
        }
    }

    /// Placement-owned TopLevel free-standing `IdentifierReference`
    /// `ExpressionStatement` use-site probe (Issue #768 per #688 comment
    /// 5741848600, composing the EOF-only ASI / synthetic-source-provenance
    /// theorem accepted by #233/#234 and #318/#319 with the body probe
    /// above): recognizes the body transactionally, then decides this
    /// placement's own termination:
    ///
    /// ```text
    /// SelectedTopLevelIdentifierReferenceExpressionStatementUseSite ::=
    ///     SelectedIdentifierReferenceExpressionStatementUseSiteBody
    ///     SelectedStatementTrailingTrivia
    ///     ( AuthoredSemicolon | AutomaticAtEof )
    /// ```
    ///
    /// A declining body probe (or a `ResourceLimited`/`InternalFailure`
    /// classification from it) is propagated exactly. Given a matched body,
    /// selected trivia is skipped and an authored `;` is preferred when
    /// present (`AuthoredSemicolon`); otherwise actual source end of input
    /// (`self.is_eof()`, never merely a `LineTerminator` or any other
    /// position) commits `AutomaticAtEof`. Neither condition holding rolls
    /// the cursor back to exactly where it stood before this whole probe
    /// began and declines (`NotSelected`), so a locally recognized body
    /// never authorizes a richer, longer, or `LineTerminator`-terminated
    /// neighbor (`a.b`, `a+b+c`, `a\nb`). No `SourceAnchor` -- authored,
    /// synthetic, or zero-width -- is retained for the automatic semicolon,
    /// and no EOF decision offset is retained: `AutomaticAtEof` is
    /// categorical provenance only.
    fn consume_selected_top_level_identifier_reference_expression_statement_use_site(
        &mut self,
    ) -> SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition {
        let snapshot = self.offset;

        let body = match self.consume_selected_identifier_reference_expression_statement_use_site_body() {
            SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::Matched(body) => body,
            SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::NotSelected => {
                return SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition::NotSelected;
            }
            SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::ResourceLimited => {
                return SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition::ResourceLimited;
            }
            SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::InternalFailure => {
                return SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition::InternalFailure;
            }
        };

        let terminator = if self.consume_ascii(';') {
            SelectedTopLevelFreeStandingIdentifierReferenceUseSiteTerminator::AuthoredSemicolon
        } else if self.is_eof() {
            SelectedTopLevelFreeStandingIdentifierReferenceUseSiteTerminator::AutomaticAtEof
        } else {
            self.offset = snapshot;
            return SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition::NotSelected;
        };

        SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition::Matched(
            SelectedTopLevelIdentifierReferenceUseSite { body, terminator },
        )
    }

    /// Placement-owned one-level Block free-standing `IdentifierReference`
    /// `ExpressionStatement` use-site probe (Issue #768 per #688 comment
    /// 5741848600, composing the before-`}` ASI / Block-close-ownership
    /// theorem accepted by #688 comment 5685046994 / #717 with the body
    /// probe above): recognizes the body transactionally, then decides this
    /// placement's own termination:
    ///
    /// ```text
    /// SelectedBlockIdentifierReferenceExpressionStatementUseSite ::=
    ///     SelectedIdentifierReferenceExpressionStatementUseSiteBody
    ///     SelectedStatementTrailingTrivia
    ///     ( AuthoredSemicolon | [lookahead == `}`] )
    /// ```
    ///
    /// A declining body probe (or a `ResourceLimited`/`InternalFailure`
    /// classification from it) is propagated exactly. Given a matched body,
    /// selected trivia is skipped and an authored `;` is preferred when
    /// present (`AuthoredSemicolon`); otherwise the next significant source
    /// position being the containing Block's closing `}` commits
    /// `AutomaticBeforeBlockClose` -- this probe only peeks at `}`, never
    /// consumes it, leaving sole ownership of the closing brace with the
    /// enclosing `Cursor::parse_selected_block` caller. Neither condition
    /// holding rolls the cursor back to exactly where it stood before this
    /// whole probe began and declines (`NotSelected`), so a locally
    /// recognized body never authorizes a richer, longer, or
    /// `LineTerminator`-terminated neighbor. No `SourceAnchor` -- authored,
    /// synthetic, or for `}` itself -- is retained for the automatic
    /// semicolon: `AutomaticBeforeBlockClose` is categorical provenance
    /// only.
    fn consume_selected_block_identifier_reference_expression_statement_use_site(
        &mut self,
    ) -> SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition {
        let snapshot = self.offset;

        let body = match self.consume_selected_identifier_reference_expression_statement_use_site_body() {
            SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::Matched(body) => body,
            SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::NotSelected => {
                return SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition::NotSelected;
            }
            SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::ResourceLimited => {
                return SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition::ResourceLimited;
            }
            SelectedIdentifierReferenceExpressionStatementUseSiteBodyRecognition::InternalFailure => {
                return SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition::InternalFailure;
            }
        };

        let terminator = if self.consume_ascii(';') {
            SelectedBlockFreeStandingIdentifierReferenceUseSiteTerminator::AuthoredSemicolon
        } else if self.peek_char() == Some('}') {
            SelectedBlockFreeStandingIdentifierReferenceUseSiteTerminator::AutomaticBeforeBlockClose
        } else {
            self.offset = snapshot;
            return SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition::NotSelected;
        };

        SelectedBlockIdentifierReferenceExpressionStatementUseSiteRecognition::Matched(
            SelectedBlockIdentifierReferenceUseSite { body, terminator },
        )
    }

    /// Recognizes the bounded selected `IdentifierReference` initializer
    /// family fixed by Issue #754, widened by Issue #797 (per #688 comment
    /// 5771773635) to an optional bounded third operand -- see
    /// `consume_selected_identifier_reference_initializer_third_operand`,
    /// called from the `Two`-operand match arm below whenever the second
    /// operand was plain (never right-unary-wrapped):
    ///
    /// ```text
    /// SelectedTwoIdentifierReferenceAdditiveInitializer ::=
    ///     SelectedAcceptedIdentifierReference
    ///     SelectedAdditiveTrivia
    ///     ("+" | "-")
    ///     SelectedAdditiveTrivia
    ///     SelectedAcceptedIdentifierReference
    /// ```
    ///
    /// absorbing the plain single-reference route accepted by #334/#338/#750.
    /// The first operand is recognized exactly once by the unmodified shared
    /// `consume_selected_identifier_reference()` recognizer. An escaped
    /// ReservedWord first operand is never an accepted operand, so it is
    /// returned immediately as the existing classification-only
    /// `EscapedReservedIdentifierName` route without attempting any additive
    /// continuation, exactly matching prior behavior for that spelling.
    ///
    /// Issue #779 widens that continuation's right operand by exactly one
    /// bounded alternative -- an optional right-unary `+`/`-` wrapper -- per
    /// the candidate-independent token-boundary theorem accepted by
    /// #777/#778 and the production representation / placement authority
    /// accepted by #688 comment 5748542106:
    ///
    /// ```text
    /// SelectedIdentifierReferenceRightUnaryAdditiveInitializer ::=
    ///     SelectedAcceptedIdentifierReference
    ///     SelectedAdditiveTriviaBeforeBinary
    ///     SelectedBinaryPlusMinus
    ///     SelectedBinaryUnaryBoundary
    ///     SelectedUnaryPlusMinus
    ///     SelectedUnaryOperandTrivia
    ///     SelectedAcceptedIdentifierReference
    /// ```
    ///
    /// `SelectedBinaryUnaryBoundary` is the sole new load-bearing
    /// recognition capability, and it is discriminated by selected-trivia
    /// cardinality rather than by code-point identity alone, because
    /// ECMAScript's longest-input-element rule makes `++` and `--` single
    /// punctuators rather than two adjacent `+`/`-` code points:
    ///
    /// ```text
    /// binary != unary:
    ///     inter-operator selected trivia MAY be empty
    /// binary == unary:
    ///     inter-operator selected trivia MUST be non-empty
    /// ```
    ///
    /// so `a+-b` / `a-+b` are recognized with zero inter-operator trivia and
    /// `a+ +b` / `a- -b` only because non-empty selected trivia separates the
    /// two punctuators, while `a++b` / `a--b` decline this continuation
    /// entirely and are never split into a binary plus a unary sign. The
    /// binary sign, whether the inter-operator selected trivia was non-empty,
    /// and the right unary sign are transient recognition-time state only:
    /// they are compared locally on the same owned cursor and then dropped,
    /// so no tokenizer, token stream, `Punctuator` enum, operator kind,
    /// trivia `SourceAnchor`, `UnaryExpression`/`AdditiveExpression`
    /// `SourceAnchor`, or generic expression representation is introduced,
    /// and the retained carrier stays the unchanged `One`/`Two` pair. At most
    /// one right-unary wrapper is admitted -- no recursion -- so `a+-+b` and
    /// `a-++b` still fail the second operand and decline. The right unary
    /// sign is consumed and discarded before the shared recognizer runs, so
    /// it is never part of the second operand's authored `SourceAnchor`. The
    /// left-unary helper
    /// `consume_selected_leading_plus_minus_identifier_reference_left_additive_initializer`
    /// is a distinct owner for the leading-unary-first route and is not
    /// called from here; Issue #785 (per #688 comment 5757139580) widens
    /// that helper directly with the identical boundary logic to compose the
    /// both-unary family (`+a+-b`), reusing this same #777/#778 theorem
    /// rather than duplicating it in a new shared function. Free-standing
    /// right-unary use-sites (`a+-b;`) likewise remain outside this leaf,
    /// since
    /// `consume_selected_identifier_reference_expression_statement_use_site_body`
    /// owns a distinct carrier and whole-body rollback contract.
    ///
    /// Otherwise, this helper optimistically probes for a continuation:
    /// selected trivia, exactly one authored binary `+` or `-`, selected
    /// trivia, an optional single right-unary `+`/`-` admitted only by the
    /// boundary above together with its own selected operand trivia, and a
    /// second `IdentifierReference` operand recognized by the same unmodified
    /// shared recognizer (never a second scanner/decoder).
    /// If the whole continuation completes with an accepted (direct or
    /// escaped non-ReservedWord) second operand, `Two { first, second }` is
    /// returned with the cursor positioned immediately after the second
    /// operand. If the continuation does not complete for any reason
    /// (absent operator; an escaped-ReservedWord, malformed, or entirely
    /// absent second operand), the cursor is restored to exactly where it
    /// stood right after the first operand and `One(first)` is returned: no
    /// probed trivia, operator, or partial second-operand state is left
    /// committed. This is what naturally excludes longer additive chains
    /// (`a + b + c`), unary operands, non-`IdentifierReference` operands,
    /// grouping/member/call forms, and `UpdateExpression`/assignment tokens:
    /// each one fails to complete the continuation, degrades to `One(first)`
    /// with the cursor restored to immediately after the first operand, and
    /// the unconsumed remainder is then rejected by the existing enclosing
    /// owner terminator/comma transaction exactly as an unrecognized
    /// initializer suffix always has been — no dedicated firewall logic or
    /// generic expression parser is introduced here.
    ///
    /// A `ResourceLimited` or `InternalFailure` processing failure from
    /// either operand's recognition is propagated immediately and is never
    /// downgraded to a completed `One`/`Two` result or to `NotSelected`; no
    /// first-operand fact is ever returned to the caller when the second
    /// operand's recognition reports a processing failure.
    ///
    /// Widened by Issue #791 (per #688 comment 5762579228) to additionally
    /// admit a plain accepted Decimal atom (separator-free exponent,
    /// fractional, or decimal-integer, tried via the existing unmodified
    /// `consume_selected_plain_exponent_decimal_literal`,
    /// `consume_selected_plain_fractional_decimal_literal`, and
    /// `consume_selected_decimal_integer` helpers in that order) as an
    /// alternative second operand, but *only* when no right-unary `+`/`-`
    /// wrapper was consumed for this continuation: the existing accepted
    /// right-unary forms (`a+-b`, `a-+b`, `a+ +b`, `a- -b`) remain
    /// `IdentifierReference`-only, so `a+-1`, `a-+1`, `a+ +1`, and `a- -1`
    /// stay outside this widening. A successful Decimal alternative retains
    /// only `One(first)`; the Decimal atom is consumed and discarded, never
    /// retained as a fact, operator, or orientation. A declining Decimal
    /// alternative (or a right-unary-guarded decline) restores the cursor to
    /// exactly where it stood right after the first operand and returns
    /// `One(first)`, identically to every other continuation decline above.
    fn consume_selected_identifier_reference_initializer(
        &mut self,
    ) -> SelectedIdentifierReferenceInitializerRecognition {
        let first = match self.consume_selected_identifier_reference() {
            SelectedIdentifierReferenceRecognition::Matched(fact) => fact,
            SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName {
                identifier,
            } => {
                return SelectedIdentifierReferenceInitializerRecognition::EscapedReservedIdentifierName {
                    identifier,
                };
            }
            SelectedIdentifierReferenceRecognition::NotSelected => {
                return SelectedIdentifierReferenceInitializerRecognition::NotSelected;
            }
            SelectedIdentifierReferenceRecognition::ResourceLimited => {
                return SelectedIdentifierReferenceInitializerRecognition::ResourceLimited;
            }
            SelectedIdentifierReferenceRecognition::InternalFailure => {
                return SelectedIdentifierReferenceInitializerRecognition::InternalFailure;
            }
        };

        let after_first = self.offset;
        self.skip_selected_trivia();

        let binary_sign = if self.consume_ascii('+') {
            '+'
        } else if self.consume_ascii('-') {
            '-'
        } else {
            self.offset = after_first;
            return SelectedIdentifierReferenceInitializerRecognition::One(first);
        };

        let before_inter_operator_trivia = self.offset;
        self.skip_selected_trivia();

        let mut right_unary_consumed = false;
        if let Some(unary_sign @ ('+' | '-')) = self.peek_char() {
            if unary_sign == binary_sign && self.offset == before_inter_operator_trivia {
                self.offset = after_first;
                return SelectedIdentifierReferenceInitializerRecognition::One(first);
            }

            let _ = self.advance_char();
            self.skip_selected_trivia();
            right_unary_consumed = true;
        }

        let after_operator = self.offset;
        match self.consume_selected_identifier_reference() {
            SelectedIdentifierReferenceRecognition::Matched(second) => {
                if right_unary_consumed {
                    return SelectedIdentifierReferenceInitializerRecognition::Two {
                        first,
                        second,
                    };
                }
                self.consume_selected_identifier_reference_initializer_third_operand(first, second)
            }
            SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName { .. }
            | SelectedIdentifierReferenceRecognition::NotSelected => {
                self.offset = after_operator;
                if !right_unary_consumed
                    && (self.consume_selected_plain_exponent_decimal_literal()
                        || self.consume_selected_plain_fractional_decimal_literal()
                        || self.consume_selected_decimal_integer())
                {
                    return SelectedIdentifierReferenceInitializerRecognition::One(first);
                }
                self.offset = after_first;
                SelectedIdentifierReferenceInitializerRecognition::One(first)
            }
            SelectedIdentifierReferenceRecognition::ResourceLimited => {
                SelectedIdentifierReferenceInitializerRecognition::ResourceLimited
            }
            SelectedIdentifierReferenceRecognition::InternalFailure => {
                SelectedIdentifierReferenceInitializerRecognition::InternalFailure
            }
        }
    }

    /// Composes the bounded optional third-operand continuation of the
    /// exactly-three `IdentifierReference` additive initializer theorem
    /// (Issue #797 per #688 comment 5771773635), reusing the accepted
    /// candidate-independent theorem proven by #795/PR #796. Called only
    /// from `consume_selected_identifier_reference_initializer` immediately
    /// after a plain (never right-unary-wrapped) second operand has matched,
    /// with the cursor positioned immediately after that second operand
    /// (`after_second` -- before any trivia probed for the second binary
    /// operator).
    ///
    /// `after_second` is remembered first. Selected trivia is skipped; absent
    /// an authored binary `+`/`-` at that position, the cursor is restored to
    /// exactly `after_second` and `Two { first, second }` is returned --
    /// identical to the pre-#797 decline behavior, since `after_second` is
    /// precisely the cursor position `consume_selected_identifier_reference_initializer`
    /// itself would have left after matching a plain second operand. This
    /// keeps the enclosing declaration/statement owner's significant
    /// initializer end exact for `a+b` sources.
    ///
    /// Otherwise exactly one authored binary `+`/`-` is consumed (never a
    /// right-unary wrapper -- the #797 theorem is plain/plain/plain only, so
    /// no `SelectedBinaryUnaryBoundary` probing applies to this operator),
    /// selected trivia is skipped, and a third operand is recognized by the
    /// same unmodified shared `consume_selected_identifier_reference`
    /// recognizer. A matched (direct or escaped non-ReservedWord) third
    /// operand commits `Three { first, second, third }` with the cursor
    /// positioned immediately after it -- this is what naturally excludes a
    /// fourth operand (`a+b+c+d`): the local helper still returns a complete
    /// `Three`, and the enclosing owner rejects the untouched `+d` remainder,
    /// since no complete selected source may publish a truncated `Three`
    /// prefix. An escaped-ReservedWord, malformed, or entirely absent third
    /// operand restores the cursor to exactly `after_second` and returns
    /// `Two { first, second }`, leaving the untouched tail (e.g. an escaped
    /// spelling decoding to a ReservedWord, `++c`, `+ +c`, `+c.d`) for the
    /// enclosing owner to judge exactly as an
    /// unrecognized initializer suffix always has been -- no dedicated
    /// firewall logic, tokenizer, or generic expression parser is
    /// introduced. A `ResourceLimited`/`InternalFailure` processing failure
    /// from the third operand's recognition is propagated immediately and is
    /// never downgraded to a completed `Two` result.
    fn consume_selected_identifier_reference_initializer_third_operand(
        &mut self,
        first: SelectedIdentifierReferenceFact,
        second: SelectedIdentifierReferenceFact,
    ) -> SelectedIdentifierReferenceInitializerRecognition {
        let after_second = self.offset;
        self.skip_selected_trivia();

        if !self.consume_ascii('+') && !self.consume_ascii('-') {
            self.offset = after_second;
            return SelectedIdentifierReferenceInitializerRecognition::Two { first, second };
        }

        self.skip_selected_trivia();

        match self.consume_selected_identifier_reference() {
            SelectedIdentifierReferenceRecognition::Matched(third) => {
                SelectedIdentifierReferenceInitializerRecognition::Three {
                    first,
                    second,
                    third,
                }
            }
            SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName { .. }
            | SelectedIdentifierReferenceRecognition::NotSelected => {
                self.offset = after_second;
                SelectedIdentifierReferenceInitializerRecognition::Two { first, second }
            }
            SelectedIdentifierReferenceRecognition::ResourceLimited => {
                SelectedIdentifierReferenceInitializerRecognition::ResourceLimited
            }
            SelectedIdentifierReferenceRecognition::InternalFailure => {
                SelectedIdentifierReferenceInitializerRecognition::InternalFailure
            }
        }
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

    /// Composes an already-matched leading `+`/`-` `IdentifierReference`
    /// `UnaryExpression` fact (from
    /// `consume_selected_leading_plus_minus_identifier_reference_unary_expression`,
    /// unchanged) as the left operand of the initializer-owned exactly-two
    /// `IdentifierReference` additive continuation theorem (Issue #775 per
    /// #688 comment 5744972977, reusing the candidate-independent additive
    /// theorem accepted by #752/#753), widened by Issue #785 (per #688
    /// comment 5757139580) to additionally admit an optional right-unary
    /// `+`/`-` wrapper on the second operand, reusing the
    /// candidate-independent binary/right-unary token-boundary theorem
    /// accepted by #777/#778 and mirroring the continuation logic already
    /// used by `consume_selected_identifier_reference_initializer` (Issue
    /// #779) exactly, starting from the already-matched left-unary `first`
    /// instead of a plain first operand:
    ///
    /// ```text
    /// SelectedLeadingPlusMinusIdentifierReferenceLeftAdditiveInitializer ::=
    ///     SelectedLeadingPlusMinusIdentifierReferenceUnaryExpression
    ///     SelectedAdditiveTrivia
    ///     SelectedBinaryPlusMinus
    ///     SelectedAdditiveTrivia
    ///     (
    ///         SelectedAcceptedIdentifierReference
    ///       |
    ///         SelectedBinaryUnaryBoundary
    ///         SelectedUnaryPlusMinus
    ///         SelectedUnaryOperandTrivia
    ///         SelectedAcceptedIdentifierReference
    ///     )
    /// ```
    ///
    /// The plain-right alternative (`+a+b`, `-a-b`) remains accepted exactly
    /// as before Issue #785; the right-unary-wrapped alternative (`+a+-b`,
    /// `+a-+b`, `+a+ +b`, `+a- -b`) is the new bounded addition. The
    /// `SelectedAdditiveTrivia` shown once, before the alternation, is the
    /// single trivia position both alternatives share: for the plain-right
    /// alternative it is simply the trivia preceding the second operand; for
    /// the right-unary alternative that same position doubles as the
    /// inter-operator trivia the `SelectedBinaryUnaryBoundary` theorem
    /// inspects, so trivia is never modeled as owned by both branches at
    /// once.
    ///
    /// The caller passes the already-recognized `first` fact and the cursor
    /// positioned immediately after it. Selected trivia is skipped; absent an
    /// authored binary `+`/`-` at that position, the cursor is restored to
    /// immediately after `first` and `One(first)` is returned unchanged --
    /// this is the existing accepted `+a` initializer behavior, exactly
    /// preserved. Otherwise exactly one authored binary `+`/`-` is consumed,
    /// selected trivia is skipped, and the same #777/#778 boundary applies to
    /// an optional right-unary sign at that position: opposite binary and
    /// unary signs may be adjacent with no inter-operator trivia (`+a+-b`,
    /// `+a-+b`), while equal signs require non-empty inter-operator selected
    /// trivia (`+a+ +b`, `+a- -b`), so authored `++`/`--` is never split into
    /// a binary sign plus a right-unary sign (`+a++b`, `+a--b` remain
    /// outside). At most one right-unary wrapper is admitted -- no recursion
    /// -- so `+a+-+b` still fails the second operand. When present, the
    /// right-unary sign is consumed and discarded before the shared
    /// recognizer runs, so it is never part of the second operand's authored
    /// `SourceAnchor`. A second operand is then recognized by the same
    /// unmodified shared `consume_selected_identifier_reference` recognizer.
    /// An accepted second operand commits `Two { first, second }`; neither
    /// the left unary sign, the binary sign, the right unary sign, nor any
    /// inter-operator trivia is retained.
    ///
    /// This deliberately mirrors the initializer-owned local-continuation
    /// theorem already used by `consume_selected_identifier_reference_initializer`,
    /// not the free-standing whole-body rollback theorem used by
    /// `consume_selected_identifier_reference_expression_statement_use_site_body`
    /// (Issue #774): a declining, escaped-ReservedWord, or absent second
    /// operand restores the cursor to immediately after `first` and degrades
    /// to a completed `One(first)`, never to `NotSelected`. This is safe only
    /// because the enclosing declaration/statement owner remains
    /// authoritative over the complete unconsumed source: `+a+`, `+a+1`,
    /// `+a+\u{69}f`, and `+a+-+b` still fail as complete declarations, since
    /// the owner cannot validly terminate on the leftover `+`/operand source,
    /// and `+a+-b+c` still cannot truncate to a successful `Two(a,b)`
    /// declaration for the same reason. A `ResourceLimited`/`InternalFailure`
    /// classification from the second operand's recognition is propagated
    /// immediately and never degrades to a completed `One`/`Two` result.
    fn consume_selected_leading_plus_minus_identifier_reference_left_additive_initializer(
        &mut self,
        first: SelectedIdentifierReferenceFact,
    ) -> Result<SelectedIdentifierReferenceInitializer, ParseFailure> {
        let after_first = self.offset;
        self.skip_selected_trivia();

        let binary_sign = if self.consume_ascii('+') {
            '+'
        } else if self.consume_ascii('-') {
            '-'
        } else {
            self.offset = after_first;
            return Ok(SelectedIdentifierReferenceInitializer::One(first));
        };

        let before_inter_operator_trivia = self.offset;
        self.skip_selected_trivia();

        if let Some(unary_sign @ ('+' | '-')) = self.peek_char() {
            if unary_sign == binary_sign && self.offset == before_inter_operator_trivia {
                self.offset = after_first;
                return Ok(SelectedIdentifierReferenceInitializer::One(first));
            }

            let _ = self.advance_char();
            self.skip_selected_trivia();
        }

        match self.consume_selected_identifier_reference() {
            SelectedIdentifierReferenceRecognition::Matched(second) => {
                Ok(SelectedIdentifierReferenceInitializer::Two { first, second })
            }
            SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName { .. }
            | SelectedIdentifierReferenceRecognition::NotSelected => {
                self.offset = after_first;
                Ok(SelectedIdentifierReferenceInitializer::One(first))
            }
            SelectedIdentifierReferenceRecognition::ResourceLimited => {
                Err(ParseFailure::ResourceLimited)
            }
            SelectedIdentifierReferenceRecognition::InternalFailure => {
                Err(ParseFailure::InternalFailure)
            }
        }
    }

    /// Recognizes one accepted separator-free plain Decimal atom (via the
    /// existing unmodified `consume_selected_plain_exponent_decimal_literal`,
    /// `consume_selected_plain_fractional_decimal_literal`, and
    /// `consume_selected_decimal_integer` helpers, tried in that exact order
    /// exactly once, never rescanned) and then probes a bounded post-atom
    /// additive `IdentifierReference` continuation, composing the
    /// Decimal-left orientation of the one-reference / one-plain-decimal
    /// additive initializer theorem (Issue #791, per #688 comment
    /// 5762579228):
    ///
    /// ```text
    /// SelectedPlainDecimalAtomInitializer ::=
    ///     SelectedAcceptedPlainDecimalAtom
    ///     (
    ///         SelectedAdditiveTrivia
    ///         ("+" | "-")
    ///         SelectedAdditiveTrivia
    ///         SelectedAcceptedIdentifierReference
    ///     )?
    /// ```
    ///
    /// On no accepted Decimal atom at all, this returns `NotSelected` with
    /// the cursor left entirely unchanged, leaving the unmodified
    /// Boolean/null/`this`/`String`/`IdentifierReference` predecessors free
    /// to recognize their own initializer syntax. Once an accepted Decimal
    /// atom is recognized, its post-atom cursor position is remembered and
    /// selected trivia is skipped; absent an authored binary `+`/`-` at that
    /// position, the cursor is restored to exactly that remembered position
    /// and `DecimalOnly` is returned -- the exact existing presence-only
    /// Decimal initializer, unchanged. Otherwise exactly one authored binary
    /// `+`/`-` is consumed, selected trivia is skipped, and one plain
    /// `IdentifierReference` operand is recognized by the same unmodified
    /// shared `consume_selected_identifier_reference` recognizer -- never a
    /// leading- or right-unary-wrapped operand, since the #791 theorem is
    /// plain Decimal atom plus plain `IdentifierReference` only. A matched
    /// operand yields `DecimalWithReference`, carrying only that operand's
    /// `SelectedIdentifierReferenceFact`; the Decimal atom, the operator,
    /// and the orientation are never retained. A declining, escaped
    /// `ReservedWord`, or entirely absent operand restores the cursor to
    /// exactly the remembered post-atom position and returns `DecimalOnly`,
    /// preserving the existing Decimal-only initializer unchanged; the
    /// enclosing declaration/statement owner then judges any leftover
    /// source exactly as an unrecognized initializer suffix always has been.
    /// This is what naturally excludes longer additive chains (`1 + a + 2`),
    /// unary operands on either side (`1 + +a`, `+1 + a`), and richer
    /// operand families, without any dedicated firewall logic beyond the
    /// existing enclosing transaction. A `ResourceLimited` or
    /// `InternalFailure` processing failure from the `IdentifierReference`
    /// continuation is propagated immediately and is never downgraded to a
    /// completed `DecimalOnly` result.
    fn consume_selected_plain_decimal_atom_initializer(
        &mut self,
    ) -> SelectedPlainDecimalAtomInitializerRecognition {
        if !(self.consume_selected_plain_exponent_decimal_literal()
            || self.consume_selected_plain_fractional_decimal_literal()
            || self.consume_selected_decimal_integer())
        {
            return SelectedPlainDecimalAtomInitializerRecognition::NotSelected;
        }

        let after_decimal = self.offset;
        self.skip_selected_trivia();

        if !(self.consume_ascii('+') || self.consume_ascii('-')) {
            self.offset = after_decimal;
            return SelectedPlainDecimalAtomInitializerRecognition::DecimalOnly;
        }

        self.skip_selected_trivia();

        match self.consume_selected_identifier_reference() {
            SelectedIdentifierReferenceRecognition::Matched(reference) => {
                SelectedPlainDecimalAtomInitializerRecognition::DecimalWithReference(reference)
            }
            SelectedIdentifierReferenceRecognition::EscapedReservedIdentifierName { .. }
            | SelectedIdentifierReferenceRecognition::NotSelected => {
                self.offset = after_decimal;
                SelectedPlainDecimalAtomInitializerRecognition::DecimalOnly
            }
            SelectedIdentifierReferenceRecognition::ResourceLimited => {
                SelectedPlainDecimalAtomInitializerRecognition::ResourceLimited
            }
            SelectedIdentifierReferenceRecognition::InternalFailure => {
                SelectedPlainDecimalAtomInitializerRecognition::InternalFailure
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
        // Issue #758: the bounded free-standing top-level use-site probe is
        // transactional and runs before the existing raw top-level `var` /
        // Block / lexical-declaration dispatch. This ordering is
        // load-bearing: accepted `IdentifierReference` spellings include
        // contextual names such as `let`, and a direct name may begin with
        // the literal characters `var` without being the `var` keyword
        // (`varfoo;`). A declining probe leaves the cursor unperturbed, so
        // every existing dispatch decision below is unaffected.
        match cursor.consume_selected_top_level_identifier_reference_expression_statement_use_site() {
            SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition::Matched(use_site) => {
                if let Err(failure) = builder.push_use_site(use_site) {
                    return parse_failure_to_outcome(failure);
                }
            }
            SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition::ResourceLimited => {
                return SelectedLexicalSliceOutcome::ResourceLimited;
            }
            SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition::InternalFailure => {
                return SelectedLexicalSliceOutcome::InternalFailure;
            }
            SelectedTopLevelIdentifierReferenceExpressionStatementUseSiteRecognition::NotSelected => {
                if cursor.remaining().starts_with("var") {
                    let statement = match cursor.parse_variable_statement() {
                        Ok(statement) => statement,
                        Err(failure) => return parse_failure_to_outcome(failure),
                    };
                    if let Err(failure) = builder.push_variable_statement(statement) {
                        return parse_failure_to_outcome(failure);
                    }
                } else if cursor.peek_char() == Some('{') {
                    match cursor.parse_selected_block() {
                        Ok(SelectedBlockParseOutcome::Legacy(block)) => {
                            if let Err(failure) =
                                builder.push_item(SelectedTopLevelItem::Block(block))
                            {
                                return parse_failure_to_outcome(failure);
                            }
                        }
                        Ok(SelectedBlockParseOutcome::UseSiteEnabled(block)) => {
                            if let Err(failure) = builder.push_use_site_enabled_block(block) {
                                return parse_failure_to_outcome(failure);
                            }
                        }
                        Err(failure) => return parse_failure_to_outcome(failure),
                    }
                } else {
                    let item = match cursor.parse_declaration() {
                        Ok(declaration) => SelectedTopLevelItem::LexicalDeclaration(declaration),
                        Err(failure) => return parse_failure_to_outcome(failure),
                    };

                    if let Err(failure) = builder.push_item(item) {
                        return parse_failure_to_outcome(failure);
                    }
                }
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
        SelectedScriptBuilder::ReferenceUseEnabled(items) => {
            SelectedLexicalSliceOutcome::RecognizedIdentifierReferenceExpressionStatementSlice(
                SelectedIdentifierReferenceExpressionStatementScript { items },
            )
        }
        SelectedScriptBuilder::BlockReferenceUseEnabled(items) => {
            SelectedLexicalSliceOutcome::RecognizedBlockReferenceUseEnabledSlice(
                SelectedBlockReferenceUseEnabledScript { items },
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
