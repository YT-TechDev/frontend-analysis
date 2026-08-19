//! First private Binding / Scope analysis for the selected ECMAScript slice.
//!
//! This capability consumes an exact-script selected-static acceptance witness.
//! It derives same-source declaration/reference relations only. It does not
//! model runtime binding resolution, initialization state, or value flow.

use std::collections::HashMap;

use crate::SourceAnchor;

use super::selected_static_semantics::SelectedStaticSemanticsAccepted;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedBindingScopeTarget<'script> {
    SameSourceSelectedLexicalBinding(&'script SourceAnchor),
    NoSameSourceSelectedLexicalBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SelectedBindingScopeRelation<'script> {
    reference: &'script SourceAnchor,
    semantic_name: &'script str,
    target: SelectedBindingScopeTarget<'script>,
}

impl<'script> SelectedBindingScopeRelation<'script> {
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
    let mut binding_by_name: HashMap<&'script str, &'script SourceAnchor> = HashMap::new();

    for declaration in script.declarations() {
        for binding in declaration.bindings() {
            let Some(name) = binding.semantic_name() else {
                return SelectedBindingScopeOutcome::InternalFailure;
            };

            if binding_by_name.try_reserve(1).is_err() {
                return SelectedBindingScopeOutcome::ResourceLimited;
            }
            if binding_by_name.insert(name, binding.binding()).is_some() {
                return SelectedBindingScopeOutcome::InternalFailure;
            }
        }
    }

    let mut relations = Vec::new();
    for declaration in script.declarations() {
        for binding in declaration.bindings() {
            let Some(reference) = binding.identifier_reference_initializer() else {
                continue;
            };

            if relations.try_reserve(1).is_err() {
                return SelectedBindingScopeOutcome::ResourceLimited;
            }

            let semantic_name = reference.semantic_name();
            let target = match binding_by_name.get(semantic_name).copied() {
                Some(target) => {
                    SelectedBindingScopeTarget::SameSourceSelectedLexicalBinding(target)
                }
                None => SelectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding,
            };
            relations.push(SelectedBindingScopeRelation {
                reference: reference.reference(),
                semantic_name,
                target,
            });
        }
    }

    SelectedBindingScopeOutcome::Complete(SelectedBindingScopeAnalysis { relations })
}
