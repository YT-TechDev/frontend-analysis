//! First private Binding / Scope analysis for the selected ECMAScript slice.
//!
//! This capability consumes an exact-script selected-static acceptance witness.
//! It derives same-source declaration/reference relations and selected lexical
//! binding evaluation order only. It does not model runtime binding resolution,
//! runtime initialization state, or value flow.

use std::collections::HashMap;

use crate::SourceAnchor;

use super::selected_static_semantics::SelectedStaticSemanticsAccepted;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedLexicalBindingOrder {
    Before,
    Same,
    After,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum SelectedBindingScopeTarget<'script> {
    SameSourceSelectedLexicalBinding {
        binding: &'script SourceAnchor,
        order: SelectedLexicalBindingOrder,
    },
    NoSameSourceSelectedLexicalBinding,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SelectedBindingScopeRelation<'script> {
    containing_binding: &'script SourceAnchor,
    reference: &'script SourceAnchor,
    semantic_name: &'script str,
    target: SelectedBindingScopeTarget<'script>,
}

impl<'script> SelectedBindingScopeRelation<'script> {
    pub(super) fn containing_binding(&self) -> &'script SourceAnchor {
        self.containing_binding
    }

    pub(super) fn reference(&self) -> &'script SourceAnchor {
        self.reference
    }

    pub(super) fn semantic_name(&self) -> &'script str {
        self.semantic_name
    }

    pub(super) fn target(&self) -> SelectedBindingScopeTarget<'script> {
        self.target
    }
}

#[derive(Debug)]
pub(super) struct SelectedBindingScopeAnalysis<'script> {
    relations: Vec<SelectedBindingScopeRelation<'script>>,
}

impl<'script> SelectedBindingScopeAnalysis<'script> {
    pub(super) fn relations(&self) -> &[SelectedBindingScopeRelation<'script>] {
        &self.relations
    }
}

#[derive(Debug)]
pub(super) enum SelectedBindingScopeOutcome<'script> {
    Complete(SelectedBindingScopeAnalysis<'script>),
    ResourceLimited,
    InternalFailure,
}

pub(super) fn analyze_selected_binding_scope<'script>(
    accepted: &SelectedStaticSemanticsAccepted<'script>,
) -> SelectedBindingScopeOutcome<'script> {
    let script = accepted.script();
    let mut binding_by_name: HashMap<&'script str, (&'script SourceAnchor, usize)> = HashMap::new();
    let mut binding_position = 0usize;

    for declaration in script.declarations() {
        for binding in declaration.bindings() {
            let Some(name) = binding.semantic_name() else {
                return SelectedBindingScopeOutcome::InternalFailure;
            };

            if binding_by_name.try_reserve(1).is_err() {
                return SelectedBindingScopeOutcome::ResourceLimited;
            }
            if binding_by_name
                .insert(name, (binding.binding(), binding_position))
                .is_some()
            {
                return SelectedBindingScopeOutcome::InternalFailure;
            }

            let Some(next_position) = binding_position.checked_add(1) else {
                return SelectedBindingScopeOutcome::InternalFailure;
            };
            binding_position = next_position;
        }
    }

    let mut relations = Vec::new();
    let mut containing_position = 0usize;
    for declaration in script.declarations() {
        for binding in declaration.bindings() {
            if let Some(reference) = binding.identifier_reference_initializer() {
                if relations.try_reserve(1).is_err() {
                    return SelectedBindingScopeOutcome::ResourceLimited;
                }

                let semantic_name = reference.semantic_name();
                let target = match binding_by_name.get(semantic_name).copied() {
                    Some((target, target_position)) => {
                        let order = if target_position < containing_position {
                            SelectedLexicalBindingOrder::Before
                        } else if target_position == containing_position {
                            SelectedLexicalBindingOrder::Same
                        } else {
                            SelectedLexicalBindingOrder::After
                        };
                        SelectedBindingScopeTarget::SameSourceSelectedLexicalBinding {
                            binding: target,
                            order,
                        }
                    }
                    None => SelectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding,
                };
                relations.push(SelectedBindingScopeRelation {
                    containing_binding: binding.binding(),
                    reference: reference.reference(),
                    semantic_name,
                    target,
                });
            }

            let Some(next_position) = containing_position.checked_add(1) else {
                return SelectedBindingScopeOutcome::InternalFailure;
            };
            containing_position = next_position;
        }
    }

    SelectedBindingScopeOutcome::Complete(SelectedBindingScopeAnalysis { relations })
}
