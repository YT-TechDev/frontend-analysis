//! Selected production static semantics for the bounded ECMAScript Script slice.
//!
//! This module consumes only retained source-backed selected lexical facts. It
//! does not reparse authoritative source, broaden grammar coverage, or construct
//! aggregate `QualificationOutcome::Qualified`.

use std::collections::HashMap;

use crate::{SourceAnchor, SourceText};

use super::qualification::{EvidenceSubject, QualificationOutcome};
use super::selected_binding_identifier::is_unconditionally_reserved_word;
use super::selected_lexical_slice::{
    SelectedBindingNameState, SelectedBlock, SelectedBlockItem,
    SelectedBlockReferenceUseEnabledScript, SelectedBlockReferenceUseEnabledTopLevelItem,
    SelectedBlockVarBinding, SelectedIdentifierReferenceExpressionStatementScript,
    SelectedInitializerState, SelectedInvalidEscapePosition, SelectedLexicalDeclaration,
    SelectedLexicalDeclarationKind, SelectedLexicalScript, SelectedOneLevelBlockScript,
    SelectedReferenceUseEnabledTopLevelItem, SelectedTopLevelItem, SelectedUseSiteEnabledBlock,
    SelectedUseSiteEnabledBlockItem, SelectedVariableBinding, SelectedVariableStatementScript,
    SelectedVariableTopLevelItem,
};

#[derive(Debug)]
pub(super) struct SelectedStaticSemanticsAccepted<'script> {
    script: &'script SelectedLexicalScript,
}

impl<'script> SelectedStaticSemanticsAccepted<'script> {
    pub(super) fn script(&self) -> &'script SelectedLexicalScript {
        self.script
    }
}

#[derive(Debug)]
pub(super) struct SelectedOneLevelBlockStaticSemanticsAccepted<'script> {
    script: &'script SelectedOneLevelBlockScript,
}

impl<'script> SelectedOneLevelBlockStaticSemanticsAccepted<'script> {
    pub(super) fn script(&self) -> &'script SelectedOneLevelBlockScript {
        self.script
    }
}

#[derive(Debug)]
pub(super) struct SelectedVariableStatementStaticSemanticsAccepted<'script> {
    script: &'script SelectedVariableStatementScript,
}

impl<'script> SelectedVariableStatementStaticSemanticsAccepted<'script> {
    pub(super) fn script(&self) -> &'script SelectedVariableStatementScript {
        self.script
    }
}

/// Distinct static accepted witness for the new fourth / broadest selected
/// Script carrier (Issue #758). Historical accepted witness types remain
/// unchanged; this is a new, separate witness, never a widening of
/// `SelectedVariableStatementStaticSemanticsAccepted`.
#[derive(Debug)]
pub(super) struct SelectedIdentifierReferenceExpressionStatementStaticSemanticsAccepted<'script> {
    script: &'script SelectedIdentifierReferenceExpressionStatementScript,
}

impl<'script> SelectedIdentifierReferenceExpressionStatementStaticSemanticsAccepted<'script> {
    pub(super) fn script(&self) -> &'script SelectedIdentifierReferenceExpressionStatementScript {
        self.script
    }
}

/// Distinct static accepted witness for the new fifth / broadest selected
/// Script carrier (Issue #762). Historical accepted witness types --
/// including the #759 `SelectedIdentifierReferenceExpressionStatementStaticSemanticsAccepted`
/// witness -- remain unchanged; this is a new, separate witness, never a
/// widening of any of them.
#[derive(Debug)]
pub(super) struct SelectedBlockReferenceUseEnabledStaticSemanticsAccepted<'script> {
    script: &'script SelectedBlockReferenceUseEnabledScript,
}

impl<'script> SelectedBlockReferenceUseEnabledStaticSemanticsAccepted<'script> {
    pub(super) fn script(&self) -> &'script SelectedBlockReferenceUseEnabledScript {
        self.script
    }
}

#[derive(Debug)]
pub(super) enum SelectedStaticSemanticsOutcome<'script> {
    Accepted(SelectedStaticSemanticsAccepted<'script>),
    Rejected(SelectedStaticSemanticsRejection),
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug)]
pub(super) enum SelectedOneLevelBlockStaticSemanticsOutcome<'script> {
    Accepted(SelectedOneLevelBlockStaticSemanticsAccepted<'script>),
    Rejected(SelectedStaticSemanticsRejection),
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug)]
pub(super) enum SelectedVariableStatementStaticSemanticsOutcome<'script> {
    Accepted(SelectedVariableStatementStaticSemanticsAccepted<'script>),
    Rejected(SelectedStaticSemanticsRejection),
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug)]
pub(super) enum SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome<'script> {
    Accepted(SelectedIdentifierReferenceExpressionStatementStaticSemanticsAccepted<'script>),
    Rejected(SelectedStaticSemanticsRejection),
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug)]
pub(super) enum SelectedBlockReferenceUseEnabledStaticSemanticsOutcome<'script> {
    Accepted(SelectedBlockReferenceUseEnabledStaticSemanticsAccepted<'script>),
    Rejected(SelectedStaticSemanticsRejection),
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug)]
pub(super) enum SelectedStaticSemanticsRejection {
    InvalidEscapedIdentifierStart {
        escape: SourceAnchor,
    },
    InvalidEscapedIdentifierPart {
        escape: SourceAnchor,
    },
    EscapedReservedWord {
        binding: SourceAnchor,
    },
    EscapedReservedWordInitializer {
        identifier: SourceAnchor,
    },
    BindingNamedLet {
        binding: SourceAnchor,
    },
    DuplicateDeclarationBinding {
        first_binding: SourceAnchor,
        duplicate_binding: SourceAnchor,
    },
    ConstBindingMissingInitializer {
        binding: SourceAnchor,
    },
    DuplicateBlockLexicalName {
        first_binding: SourceAnchor,
        duplicate_binding: SourceAnchor,
    },
    DuplicateLexicalName {
        first_binding: SourceAnchor,
        duplicate_binding: SourceAnchor,
    },
    LexicalVarNameCollision {
        lexical_binding: SourceAnchor,
        var_binding: SourceAnchor,
        primary_binding: SourceAnchor,
    },
    BlockLexicalVarNameCollision {
        lexical_binding: SourceAnchor,
        var_binding: SourceAnchor,
        primary_binding: SourceAnchor,
    },
}

impl SelectedStaticSemanticsRejection {
    fn primary_anchor(&self) -> &SourceAnchor {
        match self {
            Self::InvalidEscapedIdentifierStart { escape }
            | Self::InvalidEscapedIdentifierPart { escape } => escape,
            Self::EscapedReservedWord { binding } | Self::BindingNamedLet { binding } => binding,
            Self::EscapedReservedWordInitializer { identifier } => identifier,
            Self::DuplicateDeclarationBinding {
                duplicate_binding, ..
            }
            | Self::DuplicateBlockLexicalName {
                duplicate_binding, ..
            }
            | Self::DuplicateLexicalName {
                duplicate_binding, ..
            } => duplicate_binding,
            Self::ConstBindingMissingInitializer { binding } => binding,
            Self::LexicalVarNameCollision {
                primary_binding, ..
            }
            | Self::BlockLexicalVarNameCollision {
                primary_binding, ..
            } => primary_binding,
        }
    }
}

#[derive(Debug)]
enum SelectedDeclarationCheckFailure {
    Rejected(SelectedStaticSemanticsRejection),
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedDuplicateCheckFailure {
    ResourceLimited,
    InternalFailure,
}

fn evaluate_selected_binding_name_static_semantics<'binding>(
    binding: &'binding SourceAnchor,
    name_state: &'binding SelectedBindingNameState,
) -> Result<&'binding str, SelectedDeclarationCheckFailure> {
    match name_state {
        SelectedBindingNameState::InvalidEscapedPosition { position, escape } => {
            let rejection = match position {
                SelectedInvalidEscapePosition::Start => {
                    SelectedStaticSemanticsRejection::InvalidEscapedIdentifierStart {
                        escape: escape.clone(),
                    }
                }
                SelectedInvalidEscapePosition::Part => {
                    SelectedStaticSemanticsRejection::InvalidEscapedIdentifierPart {
                        escape: escape.clone(),
                    }
                }
            };
            Err(SelectedDeclarationCheckFailure::Rejected(rejection))
        }
        SelectedBindingNameState::EscapedValid { decoded } => {
            if is_unconditionally_reserved_word(decoded) {
                return Err(SelectedDeclarationCheckFailure::Rejected(
                    SelectedStaticSemanticsRejection::EscapedReservedWord {
                        binding: binding.clone(),
                    },
                ));
            }
            Ok(decoded.as_str())
        }
        SelectedBindingNameState::Unescaped => Ok(binding.fragment()),
    }
}

fn evaluate_selected_declaration_local_static_semantics(
    declaration: &SelectedLexicalDeclaration,
) -> Result<(), SelectedDeclarationCheckFailure> {
    // Tier A: binding-attributed classification in binding source order.
    for binding in declaration.bindings() {
        let semantic_name = evaluate_selected_binding_name_static_semantics(
            binding.binding(),
            binding.name_state(),
        )?;

        if semantic_name == "let" {
            return Err(SelectedDeclarationCheckFailure::Rejected(
                SelectedStaticSemanticsRejection::BindingNamedLet {
                    binding: binding.binding().clone(),
                },
            ));
        }

        if let Some(identifier) = binding.escaped_reserved_initializer_identifier() {
            return Err(SelectedDeclarationCheckFailure::Rejected(
                SelectedStaticSemanticsRejection::EscapedReservedWordInitializer {
                    identifier: identifier.clone(),
                },
            ));
        }
    }

    // EE-15-R02: declaration-local BoundNames duplicates.
    let mut first_by_name: HashMap<&str, usize> = HashMap::new();
    for (binding_index, binding) in declaration.bindings().iter().enumerate() {
        let Some(name) = binding.semantic_name() else {
            return Err(SelectedDeclarationCheckFailure::InternalFailure);
        };

        if let Some(&first_index) = first_by_name.get(name) {
            return Err(SelectedDeclarationCheckFailure::Rejected(
                SelectedStaticSemanticsRejection::DuplicateDeclarationBinding {
                    first_binding: declaration.bindings()[first_index].binding().clone(),
                    duplicate_binding: binding.binding().clone(),
                },
            ));
        }

        if first_by_name.try_reserve(1).is_err() {
            return Err(SelectedDeclarationCheckFailure::ResourceLimited);
        }

        let previous = first_by_name.insert(name, binding_index);
        debug_assert!(previous.is_none());
    }

    // EE-15-R03: const bindings require an initializer. Missing syntax has
    // no fabricated source anchor; the affected authored binding is primary.
    if declaration.kind() == SelectedLexicalDeclarationKind::Const {
        for binding in declaration.bindings() {
            if binding.initializer() == SelectedInitializerState::Absent {
                return Err(SelectedDeclarationCheckFailure::Rejected(
                    SelectedStaticSemanticsRejection::ConstBindingMissingInitializer {
                        binding: binding.binding().clone(),
                    },
                ));
            }
        }
    }

    Ok(())
}

fn evaluate_selected_variable_binding_local_static_semantics(
    binding: &SelectedVariableBinding,
) -> Result<(), SelectedDeclarationCheckFailure> {
    let _ =
        evaluate_selected_binding_name_static_semantics(binding.binding(), binding.name_state())?;

    if let Some(identifier) = binding.escaped_reserved_initializer_identifier() {
        return Err(SelectedDeclarationCheckFailure::Rejected(
            SelectedStaticSemanticsRejection::EscapedReservedWordInitializer {
                identifier: identifier.clone(),
            },
        ));
    }

    Ok(())
}

/// Tier-1 binding-local obligations for one declarator of a Block-contained
/// `var` statement (Issue #688/#691, widened to `1..N` declarators by #695,
/// widened to an optional selected decimal-integer initializer per
/// declarator by #699, widened to the same-declarator escaped-ReservedWord
/// initializer `EE-04-R08` check by #715, reusing the existing top-level var
/// `SelectedStaticSemanticsRejection::EscapedReservedWordInitializer`
/// rejection identity and same-declarator LHS-before-RHS ordering). The
/// lexical-only `BindingNamedLet` restriction does not extend to
/// `VariableStatement` bindings, matching the existing top-level var
/// treatment.
fn evaluate_selected_block_var_binding_local_static_semantics(
    binding: &SelectedBlockVarBinding,
) -> Result<(), SelectedDeclarationCheckFailure> {
    let _ =
        evaluate_selected_binding_name_static_semantics(binding.binding(), binding.name_state())?;

    if let Some(identifier) = binding.escaped_reserved_initializer_identifier() {
        return Err(SelectedDeclarationCheckFailure::Rejected(
            SelectedStaticSemanticsRejection::EscapedReservedWordInitializer {
                identifier: identifier.clone(),
            },
        ));
    }

    Ok(())
}

/// Tier-2b (`EE-14-R02`): one Block's own `LexicallyDeclaredNames` intersects
/// its own `VarDeclaredNames`. Streams the Block's items in authored order,
/// so the primary evidence anchor is always whichever binding (lexical or
/// bare-var) completes the collision, matching the existing Script-level
/// `EE-36-R02` primary-selection policy in `first_lexical_var_name_collision`.
fn first_block_lexical_var_collision(
    block: &SelectedBlock,
) -> Result<Option<SelectedStaticSemanticsRejection>, SelectedDuplicateCheckFailure> {
    let mut lexical_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();
    let mut var_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();

    for item in block.items() {
        match item {
            SelectedBlockItem::LexicalDeclaration(declaration) => {
                for binding in declaration.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(var_binding) = var_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::BlockLexicalVarNameCollision {
                                lexical_binding: binding.binding().clone(),
                                var_binding: (*var_binding).clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !lexical_by_name.contains_key(name) {
                        if lexical_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = lexical_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedBlockItem::Var(statement) => {
                // Every declarator in the statement's authored
                // VariableDeclarationList order participates independently,
                // so a collision at the first, an interior, or the final
                // declarator position is detected exactly when reached.
                for binding in statement.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(lexical_binding) = lexical_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::BlockLexicalVarNameCollision {
                                lexical_binding: (*lexical_binding).clone(),
                                var_binding: binding.binding().clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !var_by_name.contains_key(name) {
                        if var_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = var_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
        }
    }

    Ok(None)
}

/// The `SelectedUseSiteEnabledBlock` counterpart of
/// `first_block_lexical_var_collision` (Issue #762). Streams the Block's
/// items in authored order exactly as the historical function does; the
/// free-standing use-site item contributes neither a lexical nor a `var`
/// name and is skipped.
fn first_use_site_enabled_block_lexical_var_collision(
    block: &SelectedUseSiteEnabledBlock,
) -> Result<Option<SelectedStaticSemanticsRejection>, SelectedDuplicateCheckFailure> {
    let mut lexical_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();
    let mut var_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();

    for item in block.items() {
        match item {
            SelectedUseSiteEnabledBlockItem::LexicalDeclaration(declaration) => {
                for binding in declaration.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(var_binding) = var_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::BlockLexicalVarNameCollision {
                                lexical_binding: binding.binding().clone(),
                                var_binding: (*var_binding).clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !lexical_by_name.contains_key(name) {
                        if lexical_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = lexical_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedUseSiteEnabledBlockItem::Var(statement) => {
                for binding in statement.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(lexical_binding) = lexical_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::BlockLexicalVarNameCollision {
                                lexical_binding: (*lexical_binding).clone(),
                                var_binding: binding.binding().clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !var_by_name.contains_key(name) {
                        if var_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = var_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(_) => {}
        }
    }

    Ok(None)
}

fn first_duplicate_lexical_name<'declaration, I>(
    declarations: I,
) -> Result<Option<(SourceAnchor, SourceAnchor)>, SelectedDuplicateCheckFailure>
where
    I: IntoIterator<Item = &'declaration SelectedLexicalDeclaration>,
{
    let mut first_by_name: HashMap<&'declaration str, &'declaration SourceAnchor> = HashMap::new();

    for declaration in declarations {
        for binding in declaration.bindings() {
            let Some(name) = binding.semantic_name() else {
                return Err(SelectedDuplicateCheckFailure::InternalFailure);
            };

            if let Some(first_binding) = first_by_name.get(name) {
                return Ok(Some(((*first_binding).clone(), binding.binding().clone())));
            }

            if first_by_name.try_reserve(1).is_err() {
                return Err(SelectedDuplicateCheckFailure::ResourceLimited);
            }

            let previous = first_by_name.insert(name, binding.binding());
            debug_assert!(previous.is_none());
        }
    }

    Ok(None)
}

fn first_lexical_var_name_collision(
    script: &SelectedVariableStatementScript,
) -> Result<Option<SelectedStaticSemanticsRejection>, SelectedDuplicateCheckFailure> {
    let mut first_lexical_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();
    let mut first_var_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();

    for item in script.items() {
        match item {
            SelectedVariableTopLevelItem::LexicalDeclaration(declaration) => {
                for binding in declaration.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(var_binding) = first_var_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: binding.binding().clone(),
                                var_binding: (*var_binding).clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_lexical_by_name.contains_key(name) {
                        if first_lexical_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_lexical_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedVariableTopLevelItem::Block(block) => {
                // Only the Block's own var contributors propagate into Script
                // `VarDeclaredNames` (Issue #691, widened to `1..N`
                // contributors per Block var statement by #695). The Block's
                // lexical names never enter this Script-level lexical domain:
                // that asymmetry is load-bearing (`var x; { let x; }` must
                // not become falsely symmetric with `let x; { var x; }`).
                for binding in block.block_var_bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(lexical_binding) = first_lexical_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: (*lexical_binding).clone(),
                                var_binding: binding.binding().clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_var_by_name.contains_key(name) {
                        if first_var_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_var_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedVariableTopLevelItem::VariableStatement(statement) => {
                // Every selected declarator participates, in authored
                // VariableDeclarationList order. Var provenance stays the
                // first authored occurrence of a name, as already fixed for
                // repeated VariableStatements.
                for binding in statement.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(lexical_binding) = first_lexical_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: (*lexical_binding).clone(),
                                var_binding: binding.binding().clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_var_by_name.contains_key(name) {
                        if first_var_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_var_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
        }
    }

    Ok(None)
}

/// The `SelectedIdentifierReferenceExpressionStatementScript` counterpart of
/// `first_lexical_var_name_collision` (Issue #758). The free-standing
/// `IdentifierReferenceExpressionStatement` use-site item contributes no
/// declaration name and is skipped; every `LexicalDeclaration`, `Block`,
/// and `VariableStatement` item participates exactly as it already does
/// for the var-enabled variant.
fn first_reference_use_enabled_lexical_var_name_collision(
    script: &SelectedIdentifierReferenceExpressionStatementScript,
) -> Result<Option<SelectedStaticSemanticsRejection>, SelectedDuplicateCheckFailure> {
    let mut first_lexical_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();
    let mut first_var_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();

    for item in script.items() {
        match item {
            SelectedReferenceUseEnabledTopLevelItem::LexicalDeclaration(declaration) => {
                for binding in declaration.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(var_binding) = first_var_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: binding.binding().clone(),
                                var_binding: (*var_binding).clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_lexical_by_name.contains_key(name) {
                        if first_lexical_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_lexical_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedReferenceUseEnabledTopLevelItem::Block(block) => {
                for binding in block.block_var_bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(lexical_binding) = first_lexical_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: (*lexical_binding).clone(),
                                var_binding: binding.binding().clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_var_by_name.contains_key(name) {
                        if first_var_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_var_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedReferenceUseEnabledTopLevelItem::VariableStatement(statement) => {
                for binding in statement.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(lexical_binding) = first_lexical_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: (*lexical_binding).clone(),
                                var_binding: binding.binding().clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_var_by_name.contains_key(name) {
                        if first_var_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_var_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(_) => {}
        }
    }

    Ok(None)
}

/// The `SelectedBlockReferenceUseEnabledScript` counterpart of
/// `first_lexical_var_name_collision` (Issue #762). Every `LexicalDeclaration`,
/// `Block`, `UseSiteEnabledBlock`, and `VariableStatement` item participates
/// exactly as it already does for the reference-use-enabled (#758) variant;
/// the free-standing `IdentifierReferenceExpressionStatement` use-site item
/// (TopLevel or Block-contained) contributes no declaration name and is
/// skipped.
fn first_block_reference_use_enabled_lexical_var_name_collision(
    script: &SelectedBlockReferenceUseEnabledScript,
) -> Result<Option<SelectedStaticSemanticsRejection>, SelectedDuplicateCheckFailure> {
    let mut first_lexical_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();
    let mut first_var_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();

    for item in script.items() {
        match item {
            SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(declaration) => {
                for binding in declaration.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(var_binding) = first_var_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: binding.binding().clone(),
                                var_binding: (*var_binding).clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_lexical_by_name.contains_key(name) {
                        if first_lexical_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_lexical_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::Block(block) => {
                for binding in block.block_var_bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(lexical_binding) = first_lexical_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: (*lexical_binding).clone(),
                                var_binding: binding.binding().clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_var_by_name.contains_key(name) {
                        if first_var_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_var_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block) => {
                for binding in block.block_var_bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(lexical_binding) = first_lexical_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: (*lexical_binding).clone(),
                                var_binding: binding.binding().clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_var_by_name.contains_key(name) {
                        if first_var_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_var_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::VariableStatement(statement) => {
                for binding in statement.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(lexical_binding) = first_lexical_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: (*lexical_binding).clone(),
                                var_binding: binding.binding().clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_var_by_name.contains_key(name) {
                        if first_var_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_var_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                _,
            ) => {}
        }
    }

    Ok(None)
}

/// The `SelectedOneLevelBlockScript` counterpart of
/// `first_lexical_var_name_collision` for sources with no top-level
/// `VariableStatement` (Issue #691, widened to `1..N` contributors per Block
/// var statement by #695). Script `VarDeclaredNames` here can only be
/// populated by Block-contained var contributors; Block lexical names never
/// enter the Script lexical domain, preserving the same asymmetry as the
/// var-enabled variant.
fn first_one_level_block_script_lexical_var_collision(
    script: &SelectedOneLevelBlockScript,
) -> Result<Option<SelectedStaticSemanticsRejection>, SelectedDuplicateCheckFailure> {
    let mut first_lexical_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();
    let mut first_var_by_name: HashMap<&str, &SourceAnchor> = HashMap::new();

    for item in script.items() {
        match item {
            SelectedTopLevelItem::LexicalDeclaration(declaration) => {
                for binding in declaration.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(var_binding) = first_var_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: binding.binding().clone(),
                                var_binding: (*var_binding).clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_lexical_by_name.contains_key(name) {
                        if first_lexical_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_lexical_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
            SelectedTopLevelItem::Block(block) => {
                for binding in block.block_var_bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(SelectedDuplicateCheckFailure::InternalFailure);
                    };

                    if let Some(lexical_binding) = first_lexical_by_name.get(name) {
                        return Ok(Some(
                            SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                                lexical_binding: (*lexical_binding).clone(),
                                var_binding: binding.binding().clone(),
                                primary_binding: binding.binding().clone(),
                            },
                        ));
                    }

                    if !first_var_by_name.contains_key(name) {
                        if first_var_by_name.try_reserve(1).is_err() {
                            return Err(SelectedDuplicateCheckFailure::ResourceLimited);
                        }
                        let previous = first_var_by_name.insert(name, binding.binding());
                        debug_assert!(previous.is_none());
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Evaluates every selected declaration-local obligation in declaration source
/// order, then the selected Script-level duplicate-name obligation.
///
/// This ordering is the project evidence-selection policy accepted by #215 and
/// independently challenged by #216/#219. It is not an ECMAScript-specified
/// diagnostic ordering.
pub(super) fn evaluate_selected_static_semantics<'script>(
    script: &'script SelectedLexicalScript,
) -> SelectedStaticSemanticsOutcome<'script> {
    for declaration in script.declarations() {
        match evaluate_selected_declaration_local_static_semantics(declaration) {
            Ok(()) => {}
            Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                return SelectedStaticSemanticsOutcome::Rejected(rejection);
            }
            Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                return SelectedStaticSemanticsOutcome::ResourceLimited;
            }
            Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                return SelectedStaticSemanticsOutcome::InternalFailure;
            }
        }
    }

    // EE-36-R01: only after all selected declaration-local checks pass.
    match first_duplicate_lexical_name(script.declarations()) {
        Ok(Some((first_binding, duplicate_binding))) => SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateLexicalName {
                first_binding,
                duplicate_binding,
            },
        ),
        Ok(None) => {
            SelectedStaticSemanticsOutcome::Accepted(SelectedStaticSemanticsAccepted { script })
        }
        Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
            SelectedStaticSemanticsOutcome::ResourceLimited
        }
        Err(SelectedDuplicateCheckFailure::InternalFailure) => {
            SelectedStaticSemanticsOutcome::InternalFailure
        }
    }
}

/// Evaluates the first one-level Block-enabled selected Script while preserving
/// the existing flat capability as a distinct static-acceptance prerequisite.
///
/// Evidence selection follows the private #291 policy:
/// declaration-local checks first in authored order, then Block-local EE-14-R01
/// duplicate checks in Block source order, then top-level Script EE-36-R01.
pub(super) fn evaluate_selected_one_level_block_static_semantics<'script>(
    script: &'script SelectedOneLevelBlockScript,
) -> SelectedOneLevelBlockStaticSemanticsOutcome<'script> {
    // Tier 1: every declaration-local selected check in authored declaration
    // order across top-level declarations and Block bodies.
    for item in script.items() {
        match item {
            SelectedTopLevelItem::LexicalDeclaration(declaration) => {
                match evaluate_selected_declaration_local_static_semantics(declaration) {
                    Ok(()) => {}
                    Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                        return SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(rejection);
                    }
                    Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                        return SelectedOneLevelBlockStaticSemanticsOutcome::ResourceLimited;
                    }
                    Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                        return SelectedOneLevelBlockStaticSemanticsOutcome::InternalFailure;
                    }
                }
            }
            SelectedTopLevelItem::Block(block) => {
                for item in block.items() {
                    match item {
                        SelectedBlockItem::LexicalDeclaration(declaration) => {
                            match evaluate_selected_declaration_local_static_semantics(declaration)
                            {
                                Ok(()) => {}
                                Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                                    return SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
                                        rejection,
                                    );
                                }
                                Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                                    return SelectedOneLevelBlockStaticSemanticsOutcome::ResourceLimited;
                                }
                                Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                                    return SelectedOneLevelBlockStaticSemanticsOutcome::InternalFailure;
                                }
                            }
                        }
                        SelectedBlockItem::Var(statement) => {
                            // Every declarator of the statement's authored
                            // VariableDeclarationList is checked in order.
                            for binding in statement.bindings() {
                                match evaluate_selected_block_var_binding_local_static_semantics(
                                    binding,
                                ) {
                                    Ok(()) => {}
                                    Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                                        return SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
                                            rejection,
                                        );
                                    }
                                    Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                                        return SelectedOneLevelBlockStaticSemanticsOutcome::ResourceLimited;
                                    }
                                    Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                                        return SelectedOneLevelBlockStaticSemanticsOutcome::InternalFailure;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Tier 2a / EE-14-R01: each selected Block is an independent lexical
    // region. This full pass across every Block completes before Tier 2b
    // begins, so a later Block's EE-14-R01 always outranks an earlier
    // Block's EE-14-R02.
    for item in script.items() {
        let SelectedTopLevelItem::Block(block) = item else {
            continue;
        };

        match first_duplicate_lexical_name(block.declarations()) {
            Ok(Some((first_binding, duplicate_binding))) => {
                return SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
                    SelectedStaticSemanticsRejection::DuplicateBlockLexicalName {
                        first_binding,
                        duplicate_binding,
                    },
                );
            }
            Ok(None) => {}
            Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
                return SelectedOneLevelBlockStaticSemanticsOutcome::ResourceLimited;
            }
            Err(SelectedDuplicateCheckFailure::InternalFailure) => {
                return SelectedOneLevelBlockStaticSemanticsOutcome::InternalFailure;
            }
        }
    }

    // Tier 2b / EE-14-R02: each selected Block's own LexicallyDeclaredNames
    // vs. its own VarDeclaredNames, in Block source order.
    for item in script.items() {
        let SelectedTopLevelItem::Block(block) = item else {
            continue;
        };

        match first_block_lexical_var_collision(block) {
            Ok(Some(rejection)) => {
                return SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(rejection);
            }
            Ok(None) => {}
            Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
                return SelectedOneLevelBlockStaticSemanticsOutcome::ResourceLimited;
            }
            Err(SelectedDuplicateCheckFailure::InternalFailure) => {
                return SelectedOneLevelBlockStaticSemanticsOutcome::InternalFailure;
            }
        }
    }

    // Tier 3 / EE-36-R01: only top-level LexicalDeclaration items participate.
    let top_level_declarations = script.items().iter().filter_map(|item| match item {
        SelectedTopLevelItem::LexicalDeclaration(declaration) => Some(declaration),
        SelectedTopLevelItem::Block(_) => None,
    });

    match first_duplicate_lexical_name(top_level_declarations) {
        Ok(Some((first_binding, duplicate_binding))) => {
            return SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::DuplicateLexicalName {
                    first_binding,
                    duplicate_binding,
                },
            );
        }
        Ok(None) => {}
        Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
            return SelectedOneLevelBlockStaticSemanticsOutcome::ResourceLimited;
        }
        Err(SelectedDuplicateCheckFailure::InternalFailure) => {
            return SelectedOneLevelBlockStaticSemanticsOutcome::InternalFailure;
        }
    }

    // Tier 4 / EE-36-R02: Script TopLevelLexicallyDeclaredNames vs. Script
    // VarDeclaredNames, where the var side may include any Block-var
    // declarator (Issue #691, widened to 1..N declarators per Block var
    // statement by #695) propagated from any position.
    match first_one_level_block_script_lexical_var_collision(script) {
        Ok(Some(rejection)) => SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(rejection),
        Ok(None) => SelectedOneLevelBlockStaticSemanticsOutcome::Accepted(
            SelectedOneLevelBlockStaticSemanticsAccepted { script },
        ),
        Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
            SelectedOneLevelBlockStaticSemanticsOutcome::ResourceLimited
        }
        Err(SelectedDuplicateCheckFailure::InternalFailure) => {
            SelectedOneLevelBlockStaticSemanticsOutcome::InternalFailure
        }
    }
}

/// Evaluates the distinct top-level `VariableStatement` capability fixed by
/// #310 without widening either historical selected acceptance witness.
pub(super) fn evaluate_selected_variable_statement_static_semantics<'script>(
    script: &'script SelectedVariableStatementScript,
) -> SelectedVariableStatementStaticSemanticsOutcome<'script> {
    // Tier 1: declaration/binding-local checks in authored order. Every
    // selected var declarator is evaluated in authored
    // VariableDeclarationList order and consumes only context-neutral
    // BindingIdentifier checks.
    for item in script.items() {
        match item {
            SelectedVariableTopLevelItem::LexicalDeclaration(declaration) => {
                match evaluate_selected_declaration_local_static_semantics(declaration) {
                    Ok(()) => {}
                    Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                        return SelectedVariableStatementStaticSemanticsOutcome::Rejected(
                            rejection,
                        );
                    }
                    Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                        return SelectedVariableStatementStaticSemanticsOutcome::ResourceLimited;
                    }
                    Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                        return SelectedVariableStatementStaticSemanticsOutcome::InternalFailure;
                    }
                }
            }
            SelectedVariableTopLevelItem::Block(block) => {
                for item in block.items() {
                    match item {
                        SelectedBlockItem::LexicalDeclaration(declaration) => {
                            match evaluate_selected_declaration_local_static_semantics(declaration)
                            {
                                Ok(()) => {}
                                Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                                    return SelectedVariableStatementStaticSemanticsOutcome::Rejected(
                                        rejection,
                                    );
                                }
                                Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                                    return SelectedVariableStatementStaticSemanticsOutcome::ResourceLimited;
                                }
                                Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                                    return SelectedVariableStatementStaticSemanticsOutcome::InternalFailure;
                                }
                            }
                        }
                        SelectedBlockItem::Var(statement) => {
                            // Every declarator of the statement's authored
                            // VariableDeclarationList is checked in order.
                            for binding in statement.bindings() {
                                match evaluate_selected_block_var_binding_local_static_semantics(
                                    binding,
                                ) {
                                    Ok(()) => {}
                                    Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                                        return SelectedVariableStatementStaticSemanticsOutcome::Rejected(
                                            rejection,
                                        );
                                    }
                                    Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                                        return SelectedVariableStatementStaticSemanticsOutcome::ResourceLimited;
                                    }
                                    Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                                        return SelectedVariableStatementStaticSemanticsOutcome::InternalFailure;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            SelectedVariableTopLevelItem::VariableStatement(statement) => {
                for binding in statement.bindings() {
                    match evaluate_selected_variable_binding_local_static_semantics(binding) {
                        Ok(()) => {}
                        Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                            return SelectedVariableStatementStaticSemanticsOutcome::Rejected(
                                rejection,
                            );
                        }
                        Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                            return SelectedVariableStatementStaticSemanticsOutcome::ResourceLimited;
                        }
                        Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                            return SelectedVariableStatementStaticSemanticsOutcome::InternalFailure;
                        }
                    }
                }
            }
        }
    }

    // Tier 2a / EE-14-R01: selected Blocks remain independent lexical
    // regions. This full pass across every Block completes before Tier 2b
    // begins, so a later Block's EE-14-R01 always outranks an earlier
    // Block's EE-14-R02.
    for item in script.items() {
        let SelectedVariableTopLevelItem::Block(block) = item else {
            continue;
        };

        match first_duplicate_lexical_name(block.declarations()) {
            Ok(Some((first_binding, duplicate_binding))) => {
                return SelectedVariableStatementStaticSemanticsOutcome::Rejected(
                    SelectedStaticSemanticsRejection::DuplicateBlockLexicalName {
                        first_binding,
                        duplicate_binding,
                    },
                );
            }
            Ok(None) => {}
            Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
                return SelectedVariableStatementStaticSemanticsOutcome::ResourceLimited;
            }
            Err(SelectedDuplicateCheckFailure::InternalFailure) => {
                return SelectedVariableStatementStaticSemanticsOutcome::InternalFailure;
            }
        }
    }

    // Tier 2b / EE-14-R02: each selected Block's own LexicallyDeclaredNames
    // vs. its own VarDeclaredNames, in Block source order.
    for item in script.items() {
        let SelectedVariableTopLevelItem::Block(block) = item else {
            continue;
        };

        match first_block_lexical_var_collision(block) {
            Ok(Some(rejection)) => {
                return SelectedVariableStatementStaticSemanticsOutcome::Rejected(rejection);
            }
            Ok(None) => {}
            Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
                return SelectedVariableStatementStaticSemanticsOutcome::ResourceLimited;
            }
            Err(SelectedDuplicateCheckFailure::InternalFailure) => {
                return SelectedVariableStatementStaticSemanticsOutcome::InternalFailure;
            }
        }
    }

    // Tier 3 / EE-36-R01: only top-level lexical declarations participate.
    let top_level_declarations = script.items().iter().filter_map(|item| match item {
        SelectedVariableTopLevelItem::LexicalDeclaration(declaration) => Some(declaration),
        SelectedVariableTopLevelItem::Block(_)
        | SelectedVariableTopLevelItem::VariableStatement(_) => None,
    });

    match first_duplicate_lexical_name(top_level_declarations) {
        Ok(Some((first_binding, duplicate_binding))) => {
            return SelectedVariableStatementStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::DuplicateLexicalName {
                    first_binding,
                    duplicate_binding,
                },
            );
        }
        Ok(None) => {}
        Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
            return SelectedVariableStatementStaticSemanticsOutcome::ResourceLimited;
        }
        Err(SelectedDuplicateCheckFailure::InternalFailure) => {
            return SelectedVariableStatementStaticSemanticsOutcome::InternalFailure;
        }
    }

    // Tier 4 / EE-36-R02: first collision completed in authored traversal order.
    match first_lexical_var_name_collision(script) {
        Ok(Some(rejection)) => SelectedVariableStatementStaticSemanticsOutcome::Rejected(rejection),
        Ok(None) => SelectedVariableStatementStaticSemanticsOutcome::Accepted(
            SelectedVariableStatementStaticSemanticsAccepted { script },
        ),
        Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
            SelectedVariableStatementStaticSemanticsOutcome::ResourceLimited
        }
        Err(SelectedDuplicateCheckFailure::InternalFailure) => {
            SelectedVariableStatementStaticSemanticsOutcome::InternalFailure
        }
    }
}

/// Evaluates the new fourth / broadest selected Script carrier (Issue
/// #758), projecting every already-accepted declaration-local, Block-local,
/// Script duplicate-lexical, and Script lexical/var-collision rule exactly
/// as the var-enabled variant already does. The free-standing
/// `IdentifierReferenceExpressionStatement` use-site item contributes no
/// `BoundNames`/`LexicallyDeclaredNames`/`VarDeclaredNames` and is skipped
/// at every tier; it never alters existing evidence-selection order, and
/// known static rejection is never downgraded to `UnsupportedCoverage`. No
/// new Early Error identity is introduced.
pub(super) fn evaluate_selected_identifier_reference_expression_statement_static_semantics<
    'script,
>(
    script: &'script SelectedIdentifierReferenceExpressionStatementScript,
) -> SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome<'script> {
    // Tier 1: declaration/binding-local checks in authored order.
    for item in script.items() {
        match item {
            SelectedReferenceUseEnabledTopLevelItem::LexicalDeclaration(declaration) => {
                match evaluate_selected_declaration_local_static_semantics(declaration) {
                    Ok(()) => {}
                    Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                        return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Rejected(
                            rejection,
                        );
                    }
                    Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                        return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::ResourceLimited;
                    }
                    Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                        return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::InternalFailure;
                    }
                }
            }
            SelectedReferenceUseEnabledTopLevelItem::Block(block) => {
                for item in block.items() {
                    match item {
                        SelectedBlockItem::LexicalDeclaration(declaration) => {
                            match evaluate_selected_declaration_local_static_semantics(declaration)
                            {
                                Ok(()) => {}
                                Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                                    return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Rejected(
                                        rejection,
                                    );
                                }
                                Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                                    return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::ResourceLimited;
                                }
                                Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                                    return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::InternalFailure;
                                }
                            }
                        }
                        SelectedBlockItem::Var(statement) => {
                            for binding in statement.bindings() {
                                match evaluate_selected_block_var_binding_local_static_semantics(
                                    binding,
                                ) {
                                    Ok(()) => {}
                                    Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                                        return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Rejected(
                                            rejection,
                                        );
                                    }
                                    Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                                        return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::ResourceLimited;
                                    }
                                    Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                                        return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::InternalFailure;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            SelectedReferenceUseEnabledTopLevelItem::VariableStatement(statement) => {
                for binding in statement.bindings() {
                    match evaluate_selected_variable_binding_local_static_semantics(binding) {
                        Ok(()) => {}
                        Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                            return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Rejected(
                                rejection,
                            );
                        }
                        Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                            return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::ResourceLimited;
                        }
                        Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                            return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::InternalFailure;
                        }
                    }
                }
            }
            SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(_) => {}
        }
    }

    // Tier 2a / EE-14-R01: selected Blocks remain independent lexical
    // regions.
    for item in script.items() {
        let SelectedReferenceUseEnabledTopLevelItem::Block(block) = item else {
            continue;
        };

        match first_duplicate_lexical_name(block.declarations()) {
            Ok(Some((first_binding, duplicate_binding))) => {
                return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Rejected(
                    SelectedStaticSemanticsRejection::DuplicateBlockLexicalName {
                        first_binding,
                        duplicate_binding,
                    },
                );
            }
            Ok(None) => {}
            Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
                return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::ResourceLimited;
            }
            Err(SelectedDuplicateCheckFailure::InternalFailure) => {
                return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::InternalFailure;
            }
        }
    }

    // Tier 2b / EE-14-R02: each selected Block's own LexicallyDeclaredNames
    // vs. its own VarDeclaredNames, in Block source order.
    for item in script.items() {
        let SelectedReferenceUseEnabledTopLevelItem::Block(block) = item else {
            continue;
        };

        match first_block_lexical_var_collision(block) {
            Ok(Some(rejection)) => {
                return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Rejected(rejection);
            }
            Ok(None) => {}
            Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
                return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::ResourceLimited;
            }
            Err(SelectedDuplicateCheckFailure::InternalFailure) => {
                return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::InternalFailure;
            }
        }
    }

    // Tier 3 / EE-36-R01: only top-level lexical declarations participate.
    let top_level_declarations = script.items().iter().filter_map(|item| match item {
        SelectedReferenceUseEnabledTopLevelItem::LexicalDeclaration(declaration) => {
            Some(declaration)
        }
        SelectedReferenceUseEnabledTopLevelItem::Block(_)
        | SelectedReferenceUseEnabledTopLevelItem::VariableStatement(_)
        | SelectedReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(_) => {
            None
        }
    });

    match first_duplicate_lexical_name(top_level_declarations) {
        Ok(Some((first_binding, duplicate_binding))) => {
            return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::DuplicateLexicalName {
                    first_binding,
                    duplicate_binding,
                },
            );
        }
        Ok(None) => {}
        Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
            return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::ResourceLimited;
        }
        Err(SelectedDuplicateCheckFailure::InternalFailure) => {
            return SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::InternalFailure;
        }
    }

    // Tier 4 / EE-36-R02: first collision completed in authored traversal order.
    match first_reference_use_enabled_lexical_var_name_collision(script) {
        Ok(Some(rejection)) => {
            SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Rejected(
                rejection,
            )
        }
        Ok(None) => SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Accepted(
            SelectedIdentifierReferenceExpressionStatementStaticSemanticsAccepted { script },
        ),
        Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
            SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::ResourceLimited
        }
        Err(SelectedDuplicateCheckFailure::InternalFailure) => {
            SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::InternalFailure
        }
    }
}

/// Evaluates the new fifth / broadest selected Script carrier (Issue #762),
/// projecting every already-accepted declaration-local, Block-local, Script
/// duplicate-lexical, and Script lexical/var-collision rule exactly as the
/// #758 reference-use-enabled variant already does, for both the historical
/// `Block` and the new `UseSiteEnabledBlock` representation. Every
/// free-standing `IdentifierReferenceExpressionStatement` use-site item --
/// TopLevel or Block-contained -- contributes no
/// `BoundNames`/`LexicallyDeclaredNames`/`VarDeclaredNames` and is skipped
/// at every tier; it never alters existing evidence-selection order, and
/// known static rejection is never downgraded to `UnsupportedCoverage`. No
/// new Early Error identity is introduced.
pub(super) fn evaluate_selected_block_reference_use_enabled_static_semantics<'script>(
    script: &'script SelectedBlockReferenceUseEnabledScript,
) -> SelectedBlockReferenceUseEnabledStaticSemanticsOutcome<'script> {
    // Tier 1: declaration/binding-local checks in authored order.
    for item in script.items() {
        match item {
            SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(declaration) => {
                match evaluate_selected_declaration_local_static_semantics(declaration) {
                    Ok(()) => {}
                    Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
                            rejection,
                        );
                    }
                    Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited;
                    }
                    Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure;
                    }
                }
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::Block(block) => {
                for item in block.items() {
                    match item {
                        SelectedBlockItem::LexicalDeclaration(declaration) => {
                            match evaluate_selected_declaration_local_static_semantics(declaration)
                            {
                                Ok(()) => {}
                                Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                                    return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
                                        rejection,
                                    );
                                }
                                Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                                    return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited;
                                }
                                Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                                    return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure;
                                }
                            }
                        }
                        SelectedBlockItem::Var(statement) => {
                            for binding in statement.bindings() {
                                match evaluate_selected_block_var_binding_local_static_semantics(
                                    binding,
                                ) {
                                    Ok(()) => {}
                                    Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
                                            rejection,
                                        );
                                    }
                                    Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited;
                                    }
                                    Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block) => {
                for item in block.items() {
                    match item {
                        SelectedUseSiteEnabledBlockItem::LexicalDeclaration(declaration) => {
                            match evaluate_selected_declaration_local_static_semantics(declaration)
                            {
                                Ok(()) => {}
                                Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                                    return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
                                        rejection,
                                    );
                                }
                                Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                                    return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited;
                                }
                                Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                                    return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure;
                                }
                            }
                        }
                        SelectedUseSiteEnabledBlockItem::Var(statement) => {
                            for binding in statement.bindings() {
                                match evaluate_selected_block_var_binding_local_static_semantics(
                                    binding,
                                ) {
                                    Ok(()) => {}
                                    Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
                                            rejection,
                                        );
                                    }
                                    Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited;
                                    }
                                    Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure;
                                    }
                                }
                            }
                        }
                        SelectedUseSiteEnabledBlockItem::IdentifierReferenceExpressionStatement(
                            _,
                        ) => {}
                    }
                }
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::VariableStatement(statement) => {
                for binding in statement.bindings() {
                    match evaluate_selected_variable_binding_local_static_semantics(binding) {
                        Ok(()) => {}
                        Err(SelectedDeclarationCheckFailure::Rejected(rejection)) => {
                            return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
                                rejection,
                            );
                        }
                        Err(SelectedDeclarationCheckFailure::ResourceLimited) => {
                            return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited;
                        }
                        Err(SelectedDeclarationCheckFailure::InternalFailure) => {
                            return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure;
                        }
                    }
                }
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                _,
            ) => {}
        }
    }

    // Tier 2a / EE-14-R01: selected Blocks (historical and use-site-enabled)
    // remain independent lexical regions.
    for item in script.items() {
        let declarations_check = match item {
            SelectedBlockReferenceUseEnabledTopLevelItem::Block(block) => {
                first_duplicate_lexical_name(block.declarations())
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block) => {
                first_duplicate_lexical_name(block.declarations())
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(_)
            | SelectedBlockReferenceUseEnabledTopLevelItem::VariableStatement(_)
            | SelectedBlockReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                _,
            ) => continue,
        };

        match declarations_check {
            Ok(Some((first_binding, duplicate_binding))) => {
                return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
                    SelectedStaticSemanticsRejection::DuplicateBlockLexicalName {
                        first_binding,
                        duplicate_binding,
                    },
                );
            }
            Ok(None) => {}
            Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
                return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited;
            }
            Err(SelectedDuplicateCheckFailure::InternalFailure) => {
                return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure;
            }
        }
    }

    // Tier 2b / EE-14-R02: each selected Block's own LexicallyDeclaredNames
    // vs. its own VarDeclaredNames, in Block source order.
    for item in script.items() {
        match item {
            SelectedBlockReferenceUseEnabledTopLevelItem::Block(block) => {
                match first_block_lexical_var_collision(block) {
                    Ok(Some(rejection)) => {
                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
                            rejection,
                        );
                    }
                    Ok(None) => {}
                    Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited;
                    }
                    Err(SelectedDuplicateCheckFailure::InternalFailure) => {
                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure;
                    }
                }
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(block) => {
                match first_use_site_enabled_block_lexical_var_collision(block) {
                    Ok(Some(rejection)) => {
                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
                            rejection,
                        );
                    }
                    Ok(None) => {}
                    Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited;
                    }
                    Err(SelectedDuplicateCheckFailure::InternalFailure) => {
                        return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure;
                    }
                }
            }
            SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(_)
            | SelectedBlockReferenceUseEnabledTopLevelItem::VariableStatement(_)
            | SelectedBlockReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
                _,
            ) => {}
        }
    }

    // Tier 3 / EE-36-R01: only top-level lexical declarations participate.
    let top_level_declarations =
        script.items().iter().filter_map(|item| {
            match item {
        SelectedBlockReferenceUseEnabledTopLevelItem::LexicalDeclaration(declaration) => {
            Some(declaration)
        }
        SelectedBlockReferenceUseEnabledTopLevelItem::Block(_)
        | SelectedBlockReferenceUseEnabledTopLevelItem::UseSiteEnabledBlock(_)
        | SelectedBlockReferenceUseEnabledTopLevelItem::VariableStatement(_)
        | SelectedBlockReferenceUseEnabledTopLevelItem::IdentifierReferenceExpressionStatement(
            _,
        ) => None,
    }
        });

    match first_duplicate_lexical_name(top_level_declarations) {
        Ok(Some((first_binding, duplicate_binding))) => {
            return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::DuplicateLexicalName {
                    first_binding,
                    duplicate_binding,
                },
            );
        }
        Ok(None) => {}
        Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
            return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited;
        }
        Err(SelectedDuplicateCheckFailure::InternalFailure) => {
            return SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure;
        }
    }

    // Tier 4 / EE-36-R02: first collision completed in authored traversal order.
    match first_block_reference_use_enabled_lexical_var_name_collision(script) {
        Ok(Some(rejection)) => {
            SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(rejection)
        }
        Ok(None) => SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Accepted(
            SelectedBlockReferenceUseEnabledStaticSemanticsAccepted { script },
        ),
        Err(SelectedDuplicateCheckFailure::ResourceLimited) => {
            SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::ResourceLimited
        }
        Err(SelectedDuplicateCheckFailure::InternalFailure) => {
            SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::InternalFailure
        }
    }
}

pub(super) fn selected_rejection_to_qualification(
    source: &SourceText,
    rejection: &SelectedStaticSemanticsRejection,
) -> QualificationOutcome {
    let subject = match EvidenceSubject::authored(source, rejection.primary_anchor().clone()) {
        Ok(subject) => subject,
        Err(_) => return QualificationOutcome::internal_failure(),
    };

    QualificationOutcome::static_semantics_rejected(subject)
}
