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
    SelectedBindingNameState, SelectedInitializerState, SelectedInvalidEscapePosition,
    SelectedLexicalDeclaration, SelectedLexicalDeclarationKind, SelectedLexicalScript,
    SelectedOneLevelBlockScript, SelectedTopLevelItem,
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

fn evaluate_selected_declaration_local_static_semantics(
    declaration: &SelectedLexicalDeclaration,
) -> Result<(), SelectedDeclarationCheckFailure> {
    // Tier A: binding-attributed classification in binding source order.
    for binding in declaration.bindings() {
        let semantic_name = match binding.name_state() {
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
                return Err(SelectedDeclarationCheckFailure::Rejected(rejection));
            }
            SelectedBindingNameState::EscapedValid { decoded } => {
                if is_unconditionally_reserved_word(decoded) {
                    return Err(SelectedDeclarationCheckFailure::Rejected(
                        SelectedStaticSemanticsRejection::EscapedReservedWord {
                            binding: binding.binding().clone(),
                        },
                    ));
                }
                decoded.as_str()
            }
            SelectedBindingNameState::Unescaped => binding.binding().fragment(),
        };

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
                return Ok(Some((
                    (*first_binding).clone(),
                    binding.binding().clone(),
                )));
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
        Ok(Some((first_binding, duplicate_binding))) => {
            SelectedStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::DuplicateLexicalName {
                    first_binding,
                    duplicate_binding,
                },
            )
        }
        Ok(None) => SelectedStaticSemanticsOutcome::Accepted(SelectedStaticSemanticsAccepted {
            script,
        }),
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
                for declaration in block.declarations() {
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
            }
        }
    }

    // Tier 2 / EE-14-R01: each selected Block is an independent lexical region.
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

    // Tier 3 / EE-36-R01: only top-level LexicalDeclaration items participate.
    let top_level_declarations = script.items().iter().filter_map(|item| match item {
        SelectedTopLevelItem::LexicalDeclaration(declaration) => Some(declaration),
        SelectedTopLevelItem::Block(_) => None,
    });

    match first_duplicate_lexical_name(top_level_declarations) {
        Ok(Some((first_binding, duplicate_binding))) => {
            SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::DuplicateLexicalName {
                    first_binding,
                    duplicate_binding,
                },
            )
        }
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
