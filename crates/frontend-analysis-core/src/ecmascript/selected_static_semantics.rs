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
    SelectedLexicalDeclarationKind, SelectedLexicalScript,
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
            } => duplicate_binding,
        }
    }
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
    let declarations = script.declarations();

    for declaration in declarations {
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
                    return SelectedStaticSemanticsOutcome::Rejected(rejection);
                }
                SelectedBindingNameState::EscapedValid { decoded } => {
                    if is_unconditionally_reserved_word(decoded) {
                        return SelectedStaticSemanticsOutcome::Rejected(
                            SelectedStaticSemanticsRejection::EscapedReservedWord {
                                binding: binding.binding().clone(),
                            },
                        );
                    }
                    decoded.as_str()
                }
                SelectedBindingNameState::Unescaped => binding.binding().fragment(),
            };

            if semantic_name == "let" {
                return SelectedStaticSemanticsOutcome::Rejected(
                    SelectedStaticSemanticsRejection::BindingNamedLet {
                        binding: binding.binding().clone(),
                    },
                );
            }

            if let Some(identifier) = binding.escaped_reserved_initializer_identifier() {
                return SelectedStaticSemanticsOutcome::Rejected(
                    SelectedStaticSemanticsRejection::EscapedReservedWordInitializer {
                        identifier: identifier.clone(),
                    },
                );
            }
        }

        // EE-15-R02: declaration-local BoundNames duplicates.
        let mut first_by_name: HashMap<&str, usize> = HashMap::new();
        for (binding_index, binding) in declaration.bindings().iter().enumerate() {
            let Some(name) = binding.semantic_name() else {
                return SelectedStaticSemanticsOutcome::InternalFailure;
            };

            if let Some(&first_index) = first_by_name.get(name) {
                return SelectedStaticSemanticsOutcome::Rejected(
                    SelectedStaticSemanticsRejection::DuplicateDeclarationBinding {
                        first_binding: declaration.bindings()[first_index].binding().clone(),
                        duplicate_binding: binding.binding().clone(),
                    },
                );
            }

            if first_by_name.try_reserve(1).is_err() {
                return SelectedStaticSemanticsOutcome::ResourceLimited;
            }

            let previous = first_by_name.insert(name, binding_index);
            debug_assert!(previous.is_none());
        }

        // EE-15-R03: const bindings require an initializer. Missing syntax has
        // no fabricated source anchor; the affected authored binding is primary.
        if declaration.kind() == SelectedLexicalDeclarationKind::Const {
            for binding in declaration.bindings() {
                if binding.initializer() == SelectedInitializerState::Absent {
                    return SelectedStaticSemanticsOutcome::Rejected(
                        SelectedStaticSemanticsRejection::ConstBindingMissingInitializer {
                            binding: binding.binding().clone(),
                        },
                    );
                }
            }
        }
    }

    // EE-36-R01: only after all selected declaration-local checks pass.
    let mut first_by_name: HashMap<&str, (usize, usize)> = HashMap::new();
    for (declaration_index, declaration) in declarations.iter().enumerate() {
        for (binding_index, binding) in declaration.bindings().iter().enumerate() {
            let Some(name) = binding.semantic_name() else {
                return SelectedStaticSemanticsOutcome::InternalFailure;
            };

            if let Some(&(first_declaration, first_binding)) = first_by_name.get(name) {
                return SelectedStaticSemanticsOutcome::Rejected(
                    SelectedStaticSemanticsRejection::DuplicateLexicalName {
                        first_binding: declarations[first_declaration].bindings()[first_binding]
                            .binding()
                            .clone(),
                        duplicate_binding: binding.binding().clone(),
                    },
                );
            }

            if first_by_name.try_reserve(1).is_err() {
                return SelectedStaticSemanticsOutcome::ResourceLimited;
            }

            let previous = first_by_name.insert(name, (declaration_index, binding_index));
            debug_assert!(previous.is_none());
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
