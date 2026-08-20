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
    SelectedLexicalDeclaration, SelectedLexicalDeclarationKind, SelectedLexicalItem,
    SelectedLexicalScript,
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
pub(super) enum SelectedStaticSemanticsOutcome<'script> {
    Accepted(SelectedStaticSemanticsAccepted<'script>),
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
    DuplicateLexicalName {
        first_binding: SourceAnchor,
        duplicate_binding: SourceAnchor,
    },
    DuplicateBlockLexicalName {
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
            } => duplicate_binding,
            Self::ConstBindingMissingInitializer { binding } => binding,
            Self::DuplicateLexicalName {
                duplicate_binding, ..
            }
            | Self::DuplicateBlockLexicalName {
                duplicate_binding, ..
            } => duplicate_binding,
        }
    }
}

#[derive(Debug)]
enum DeclarationCheckFailure {
    Rejected(SelectedStaticSemanticsRejection),
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug, Clone, Copy)]
enum DuplicateRegionOwner {
    Script,
    Block,
}

fn evaluate_declaration_local(
    declaration: &SelectedLexicalDeclaration,
) -> Result<(), DeclarationCheckFailure> {
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
                return Err(DeclarationCheckFailure::Rejected(rejection));
            }
            SelectedBindingNameState::EscapedValid { decoded } => {
                if is_unconditionally_reserved_word(decoded.as_str()) {
                    return Err(DeclarationCheckFailure::Rejected(
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
            return Err(DeclarationCheckFailure::Rejected(
                SelectedStaticSemanticsRejection::BindingNamedLet {
                    binding: binding.binding().clone(),
                },
            ));
        }

        if let Some(identifier) = binding.escaped_reserved_initializer_identifier() {
            return Err(DeclarationCheckFailure::Rejected(
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
            return Err(DeclarationCheckFailure::InternalFailure);
        };

        if let Some(&first_index) = first_by_name.get(name) {
            return Err(DeclarationCheckFailure::Rejected(
                SelectedStaticSemanticsRejection::DuplicateDeclarationBinding {
                    first_binding: declaration.bindings()[first_index].binding().clone(),
                    duplicate_binding: binding.binding().clone(),
                },
            ));
        }

        if first_by_name.try_reserve(1).is_err() {
            return Err(DeclarationCheckFailure::ResourceLimited);
        }

        let previous = first_by_name.insert(name, binding_index);
        debug_assert!(previous.is_none());
    }

    // EE-15-R03: const bindings require an initializer. Missing syntax has
    // no fabricated source anchor; the affected authored binding is primary.
    if declaration.kind() == SelectedLexicalDeclarationKind::Const {
        for binding in declaration.bindings() {
            if binding.initializer() == SelectedInitializerState::Absent {
                return Err(DeclarationCheckFailure::Rejected(
                    SelectedStaticSemanticsRejection::ConstBindingMissingInitializer {
                        binding: binding.binding().clone(),
                    },
                ));
            }
        }
    }

    Ok(())
}

fn evaluate_duplicate_lexical_names<'declaration>(
    declarations: impl Iterator<Item = &'declaration SelectedLexicalDeclaration>,
    owner: DuplicateRegionOwner,
) -> Result<(), DeclarationCheckFailure> {
    let mut first_by_name: HashMap<&'declaration str, &'declaration SourceAnchor> = HashMap::new();

    for declaration in declarations {
        for binding in declaration.bindings() {
            let Some(name) = binding.semantic_name() else {
                return Err(DeclarationCheckFailure::InternalFailure);
            };

            if let Some(first_binding) = first_by_name.get(name) {
                let rejection = match owner {
                    DuplicateRegionOwner::Script => {
                        SelectedStaticSemanticsRejection::DuplicateLexicalName {
                            first_binding: (*first_binding).clone(),
                            duplicate_binding: binding.binding().clone(),
                        }
                    }
                    DuplicateRegionOwner::Block => {
                        SelectedStaticSemanticsRejection::DuplicateBlockLexicalName {
                            first_binding: (*first_binding).clone(),
                            duplicate_binding: binding.binding().clone(),
                        }
                    }
                };
                return Err(DeclarationCheckFailure::Rejected(rejection));
            }

            if first_by_name.try_reserve(1).is_err() {
                return Err(DeclarationCheckFailure::ResourceLimited);
            }

            let previous = first_by_name.insert(name, binding.binding());
            debug_assert!(previous.is_none());
        }
    }

    Ok(())
}

fn map_check_failure<'script>(
    failure: DeclarationCheckFailure,
) -> SelectedStaticSemanticsOutcome<'script> {
    match failure {
        DeclarationCheckFailure::Rejected(rejection) => {
            SelectedStaticSemanticsOutcome::Rejected(rejection)
        }
        DeclarationCheckFailure::ResourceLimited => SelectedStaticSemanticsOutcome::ResourceLimited,
        DeclarationCheckFailure::InternalFailure => SelectedStaticSemanticsOutcome::InternalFailure,
    }
}

/// Evaluates every selected declaration-local obligation in authored traversal
/// order, then evaluates duplicate lexical names independently per lexical
/// region. Script-direct declarations retain historical EE-36-R01 ownership;
/// each selected Block owns EE-14-R01.
///
/// This ordering is the project evidence-selection policy accepted by #215 and
/// refined for one-level lexical regions by #289. It is not an ECMAScript-
/// specified diagnostic ordering.
pub(super) fn evaluate_selected_static_semantics<'script>(
    script: &'script SelectedLexicalScript,
) -> SelectedStaticSemanticsOutcome<'script> {
    // Pass 1: preserve declaration-local evidence precedence in authored
    // traversal order across top-level declarations and selected Block bodies.
    for item in script.items() {
        match item {
            SelectedLexicalItem::Declaration(declaration) => {
                if let Err(failure) = evaluate_declaration_local(declaration) {
                    return map_check_failure(failure);
                }
            }
            SelectedLexicalItem::Block(block) => {
                for declaration in block.declarations() {
                    if let Err(failure) = evaluate_declaration_local(declaration) {
                        return map_check_failure(failure);
                    }
                }
            }
        }
    }

    // Pass 2a: historical Script duplicate domain includes direct top-level
    // declarations only. Inner Block declarations must never be flattened here.
    if let Err(failure) = evaluate_duplicate_lexical_names(
        script.items().iter().filter_map(|item| match item {
            SelectedLexicalItem::Declaration(declaration) => Some(declaration),
            SelectedLexicalItem::Block(_) => None,
        }),
        DuplicateRegionOwner::Script,
    ) {
        return map_check_failure(failure);
    }

    // Pass 2b: each selected Block is an independent lexical duplicate domain.
    for item in script.items() {
        let SelectedLexicalItem::Block(block) = item else {
            continue;
        };
        if let Err(failure) = evaluate_duplicate_lexical_names(
            block.declarations().iter(),
            DuplicateRegionOwner::Block,
        ) {
            return map_check_failure(failure);
        }
    }

    SelectedStaticSemanticsOutcome::Accepted(SelectedStaticSemanticsAccepted { script })
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
