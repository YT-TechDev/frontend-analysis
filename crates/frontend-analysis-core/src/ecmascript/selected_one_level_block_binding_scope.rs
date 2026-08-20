//! Private one-level Block hierarchical Binding / Scope analysis for Issue #304.
//!
//! This capability consumes only the exact Block-enabled selected-static
//! acceptance witness. It derives nearest selected lexical targets over the
//! accepted one-level region path and preserves authored relation traversal
//! order. It does not model runtime binding resolution, TDZ state, runtime
//! initialization, structural Before/Same/After order, or value flow.

use std::collections::HashMap;

use crate::SourceAnchor;

use super::selected_lexical_slice::{
    SelectedBlock, SelectedLexicalBinding, SelectedLexicalDeclaration, SelectedOneLevelBlockScript,
    SelectedTopLevelItem,
};
use super::selected_static_semantics::SelectedOneLevelBlockStaticSemanticsAccepted;

#[derive(Debug, Clone, Copy)]
pub(super) enum SelectedOneLevelBlockBindingScopeRegion<'script> {
    TopLevel,
    Block(&'script SourceAnchor),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum SelectedOneLevelBlockBindingScopeTarget<'script> {
    SelectedLexicalBinding {
        binding: &'script SourceAnchor,
        region: SelectedOneLevelBlockBindingScopeRegion<'script>,
    },
    NoSelectedLexicalBindingTargetInCoveredRegions,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SelectedOneLevelBlockBindingScopeRelation<'script> {
    containing_binding: &'script SourceAnchor,
    current_region: SelectedOneLevelBlockBindingScopeRegion<'script>,
    reference: &'script SourceAnchor,
    semantic_name: &'script str,
    target: SelectedOneLevelBlockBindingScopeTarget<'script>,
}

impl<'script> SelectedOneLevelBlockBindingScopeRelation<'script> {
    pub(super) fn containing_binding(&self) -> &'script SourceAnchor {
        self.containing_binding
    }

    pub(super) fn current_region(&self) -> SelectedOneLevelBlockBindingScopeRegion<'script> {
        self.current_region
    }

    pub(super) fn reference(&self) -> &'script SourceAnchor {
        self.reference
    }

    pub(super) fn semantic_name(&self) -> &'script str {
        self.semantic_name
    }

    pub(super) fn target(&self) -> SelectedOneLevelBlockBindingScopeTarget<'script> {
        self.target
    }
}

#[derive(Debug)]
pub(super) struct SelectedOneLevelBlockBindingScopeAnalysis<'script> {
    relations: Vec<SelectedOneLevelBlockBindingScopeRelation<'script>>,
}

impl<'script> SelectedOneLevelBlockBindingScopeAnalysis<'script> {
    pub(super) fn relations(&self) -> &[SelectedOneLevelBlockBindingScopeRelation<'script>] {
        &self.relations
    }
}

#[derive(Debug)]
pub(super) enum SelectedOneLevelBlockBindingScopeOutcome<'script> {
    Complete(SelectedOneLevelBlockBindingScopeAnalysis<'script>),
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalysisFailure {
    ResourceLimited,
    InternalFailure,
}

fn insert_declaration_bindings<'script>(
    declaration: &'script SelectedLexicalDeclaration,
    bindings_by_name: &mut HashMap<&'script str, &'script SourceAnchor>,
) -> Result<(), AnalysisFailure> {
    for binding in declaration.bindings() {
        let Some(name) = binding.semantic_name() else {
            return Err(AnalysisFailure::InternalFailure);
        };

        bindings_by_name
            .try_reserve(1)
            .map_err(|_| AnalysisFailure::ResourceLimited)?;
        if bindings_by_name.insert(name, binding.binding()).is_some() {
            return Err(AnalysisFailure::InternalFailure);
        }
    }

    Ok(())
}

fn top_level_bindings(
    script: &SelectedOneLevelBlockScript,
) -> Result<HashMap<&str, &SourceAnchor>, AnalysisFailure> {
    let mut bindings_by_name = HashMap::new();

    for item in script.items() {
        let SelectedTopLevelItem::LexicalDeclaration(declaration) = item else {
            continue;
        };
        insert_declaration_bindings(declaration, &mut bindings_by_name)?;
    }

    Ok(bindings_by_name)
}

fn block_bindings(
    block: &SelectedBlock,
) -> Result<HashMap<&str, &SourceAnchor>, AnalysisFailure> {
    let mut bindings_by_name = HashMap::new();
    for declaration in block.declarations() {
        insert_declaration_bindings(declaration, &mut bindings_by_name)?;
    }
    Ok(bindings_by_name)
}

fn target_for_name<'script>(
    semantic_name: &str,
    current_region: SelectedOneLevelBlockBindingScopeRegion<'script>,
    current_bindings: &HashMap<&'script str, &'script SourceAnchor>,
    top_level_bindings: &HashMap<&'script str, &'script SourceAnchor>,
) -> SelectedOneLevelBlockBindingScopeTarget<'script> {
    if let Some(binding) = current_bindings.get(semantic_name).copied() {
        return SelectedOneLevelBlockBindingScopeTarget::SelectedLexicalBinding {
            binding,
            region: current_region,
        };
    }

    if let SelectedOneLevelBlockBindingScopeRegion::Block(_) = current_region
        && let Some(binding) = top_level_bindings.get(semantic_name).copied()
    {
        return SelectedOneLevelBlockBindingScopeTarget::SelectedLexicalBinding {
            binding,
            region: SelectedOneLevelBlockBindingScopeRegion::TopLevel,
        };
    }

    SelectedOneLevelBlockBindingScopeTarget::NoSelectedLexicalBindingTargetInCoveredRegions
}

fn append_binding_relation<'script>(
    binding: &'script SelectedLexicalBinding,
    current_region: SelectedOneLevelBlockBindingScopeRegion<'script>,
    current_bindings: &HashMap<&'script str, &'script SourceAnchor>,
    top_level_bindings: &HashMap<&'script str, &'script SourceAnchor>,
    relations: &mut Vec<SelectedOneLevelBlockBindingScopeRelation<'script>>,
) -> Result<(), AnalysisFailure> {
    let Some(reference) = binding.identifier_reference_initializer() else {
        return Ok(());
    };

    relations
        .try_reserve(1)
        .map_err(|_| AnalysisFailure::ResourceLimited)?;

    let semantic_name = reference.semantic_name();
    let target = target_for_name(
        semantic_name,
        current_region,
        current_bindings,
        top_level_bindings,
    );

    relations.push(SelectedOneLevelBlockBindingScopeRelation {
        containing_binding: binding.binding(),
        current_region,
        reference: reference.reference(),
        semantic_name,
        target,
    });

    Ok(())
}

fn append_declaration_relations<'script>(
    declaration: &'script SelectedLexicalDeclaration,
    current_region: SelectedOneLevelBlockBindingScopeRegion<'script>,
    current_bindings: &HashMap<&'script str, &'script SourceAnchor>,
    top_level_bindings: &HashMap<&'script str, &'script SourceAnchor>,
    relations: &mut Vec<SelectedOneLevelBlockBindingScopeRelation<'script>>,
) -> Result<(), AnalysisFailure> {
    for binding in declaration.bindings() {
        append_binding_relation(
            binding,
            current_region,
            current_bindings,
            top_level_bindings,
            relations,
        )?;
    }
    Ok(())
}

fn analyze<'script>(
    script: &'script SelectedOneLevelBlockScript,
) -> Result<SelectedOneLevelBlockBindingScopeAnalysis<'script>, AnalysisFailure> {
    let top_level_bindings = top_level_bindings(script)?;
    let mut relations = Vec::new();

    for item in script.items() {
        match item {
            SelectedTopLevelItem::LexicalDeclaration(declaration) => {
                append_declaration_relations(
                    declaration,
                    SelectedOneLevelBlockBindingScopeRegion::TopLevel,
                    &top_level_bindings,
                    &top_level_bindings,
                    &mut relations,
                )?;
            }
            SelectedTopLevelItem::Block(block) => {
                let current_bindings = block_bindings(block)?;
                let current_region = SelectedOneLevelBlockBindingScopeRegion::Block(block.block());
                for declaration in block.declarations() {
                    append_declaration_relations(
                        declaration,
                        current_region,
                        &current_bindings,
                        &top_level_bindings,
                        &mut relations,
                    )?;
                }
            }
        }
    }

    Ok(SelectedOneLevelBlockBindingScopeAnalysis { relations })
}

pub(super) fn analyze_selected_one_level_block_binding_scope<'script>(
    accepted: &SelectedOneLevelBlockStaticSemanticsAccepted<'script>,
) -> SelectedOneLevelBlockBindingScopeOutcome<'script> {
    match analyze(accepted.script()) {
        Ok(analysis) => SelectedOneLevelBlockBindingScopeOutcome::Complete(analysis),
        Err(AnalysisFailure::ResourceLimited) => {
            SelectedOneLevelBlockBindingScopeOutcome::ResourceLimited
        }
        Err(AnalysisFailure::InternalFailure) => {
            SelectedOneLevelBlockBindingScopeOutcome::InternalFailure
        }
    }
}
