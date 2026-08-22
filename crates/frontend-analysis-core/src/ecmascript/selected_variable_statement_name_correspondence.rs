//! Private selected var-enabled source-name correspondence for Issue #316.
//!
//! This capability consumes only the exact var-enabled selected-static
//! acceptance witness. It derives source-backed correspondence over the current
//! selected top-level / one-level-Block region path and preserves authored
//! relation traversal order plus every authored same-name top-level `var`
//! contributor in authored order. #324 widens only that contributor domain so
//! several declarators of one `VariableStatement` each contribute an anchor.
//!
//! This is not runtime binding resolution. An authored `VariableDeclaration`
//! contributor is not a unique runtime binding identity or a `ResolveBinding`
//! target. No-match is only absence of a selected same-source contributor on the
//! covered path; it is not runtime unresolvability or `ReferenceError`.
//!
//! Contributor order is declaration provenance only. This module does not model
//! `Before` / `Same` / `After`, TDZ, initialization, hoisting, runtime binding
//! creation or lookup order, execution reachability, Environment Records, or
//! value flow.

use std::collections::HashMap;

use crate::SourceAnchor;

use super::selected_lexical_slice::{
    SelectedBlock, SelectedLexicalBinding, SelectedLexicalDeclaration,
    SelectedVariableStatementScript, SelectedVariableTopLevelItem,
};
use super::selected_static_semantics::SelectedVariableStatementStaticSemanticsAccepted;

#[derive(Debug, Clone, Copy)]
pub(super) enum SelectedVariableStatementNameCorrespondenceRegion<'script> {
    TopLevel,
    Block(&'script SourceAnchor),
}

#[derive(Debug)]
pub(super) enum SelectedVariableStatementNameCorrespondence<'script> {
    VisibleSelectedLexicalBinding {
        binding: &'script SourceAnchor,
        region: SelectedVariableStatementNameCorrespondenceRegion<'script>,
    },
    SameSourceSelectedVarNameContributors {
        contributors: Vec<&'script SourceAnchor>,
    },
    NoSelectedSameSourceContributor,
}

impl<'script> SelectedVariableStatementNameCorrespondence<'script> {
    pub(super) fn selected_lexical_binding(
        &self,
    ) -> Option<(
        &'script SourceAnchor,
        SelectedVariableStatementNameCorrespondenceRegion<'script>,
    )> {
        match self {
            Self::VisibleSelectedLexicalBinding { binding, region } => Some((binding, *region)),
            Self::SameSourceSelectedVarNameContributors { .. }
            | Self::NoSelectedSameSourceContributor => None,
        }
    }

    pub(super) fn var_contributors(&self) -> Option<&[&'script SourceAnchor]> {
        match self {
            Self::SameSourceSelectedVarNameContributors { contributors } => Some(contributors),
            Self::VisibleSelectedLexicalBinding { .. } | Self::NoSelectedSameSourceContributor => {
                None
            }
        }
    }

    pub(super) fn is_no_selected_same_source_contributor(&self) -> bool {
        matches!(self, Self::NoSelectedSameSourceContributor)
    }
}

#[derive(Debug)]
pub(super) struct SelectedVariableStatementNameCorrespondenceRelation<'script> {
    containing_binding: &'script SourceAnchor,
    current_region: SelectedVariableStatementNameCorrespondenceRegion<'script>,
    reference: &'script SourceAnchor,
    semantic_name: &'script str,
    correspondence: SelectedVariableStatementNameCorrespondence<'script>,
}

impl<'script> SelectedVariableStatementNameCorrespondenceRelation<'script> {
    pub(super) fn containing_binding(&self) -> &'script SourceAnchor {
        self.containing_binding
    }

    pub(super) fn current_region(
        &self,
    ) -> SelectedVariableStatementNameCorrespondenceRegion<'script> {
        self.current_region
    }

    pub(super) fn reference(&self) -> &'script SourceAnchor {
        self.reference
    }

    pub(super) fn semantic_name(&self) -> &'script str {
        self.semantic_name
    }

    pub(super) fn correspondence(&self) -> &SelectedVariableStatementNameCorrespondence<'script> {
        &self.correspondence
    }
}

#[derive(Debug)]
pub(super) struct SelectedVariableStatementNameCorrespondenceAnalysis<'script> {
    relations: Vec<SelectedVariableStatementNameCorrespondenceRelation<'script>>,
}

impl<'script> SelectedVariableStatementNameCorrespondenceAnalysis<'script> {
    pub(super) fn relations(
        &self,
    ) -> &[SelectedVariableStatementNameCorrespondenceRelation<'script>] {
        &self.relations
    }
}

#[derive(Debug)]
pub(super) enum SelectedVariableStatementNameCorrespondenceOutcome<'script> {
    Complete(SelectedVariableStatementNameCorrespondenceAnalysis<'script>),
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalysisFailure {
    ResourceLimited,
    InternalFailure,
}

type LexicalBindingsByName<'script> = HashMap<&'script str, &'script SourceAnchor>;
type VarContributorsByName<'script> = HashMap<&'script str, Vec<&'script SourceAnchor>>;

fn insert_declaration_bindings<'script>(
    declaration: &'script SelectedLexicalDeclaration,
    bindings_by_name: &mut LexicalBindingsByName<'script>,
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
    script: &SelectedVariableStatementScript,
) -> Result<LexicalBindingsByName<'_>, AnalysisFailure> {
    let mut bindings_by_name = HashMap::new();

    for item in script.items() {
        let SelectedVariableTopLevelItem::LexicalDeclaration(declaration) = item else {
            continue;
        };
        insert_declaration_bindings(declaration, &mut bindings_by_name)?;
    }

    Ok(bindings_by_name)
}

fn block_bindings(block: &SelectedBlock) -> Result<LexicalBindingsByName<'_>, AnalysisFailure> {
    let mut bindings_by_name = HashMap::new();
    for declaration in block.declarations() {
        insert_declaration_bindings(declaration, &mut bindings_by_name)?;
    }
    Ok(bindings_by_name)
}

fn var_contributors(
    script: &SelectedVariableStatementScript,
) -> Result<VarContributorsByName<'_>, AnalysisFailure> {
    let mut contributors_by_name: VarContributorsByName<'_> = HashMap::new();

    for item in script.items() {
        let SelectedVariableTopLevelItem::VariableStatement(statement) = item else {
            continue;
        };

        // Each authored declarator of one `VariableStatement` contributes its
        // own anchor, in authored `VariableDeclarationList` order, and composes
        // with contributors from earlier statements. Repeated names are never
        // deduplicated into a singular logical target.
        for binding in statement.bindings() {
            let Some(name) = binding.semantic_name() else {
                return Err(AnalysisFailure::InternalFailure);
            };

            if let Some(contributors) = contributors_by_name.get_mut(name) {
                contributors
                    .try_reserve(1)
                    .map_err(|_| AnalysisFailure::ResourceLimited)?;
                contributors.push(binding.binding());
                continue;
            }

            contributors_by_name
                .try_reserve(1)
                .map_err(|_| AnalysisFailure::ResourceLimited)?;
            let mut contributors = Vec::new();
            contributors
                .try_reserve(1)
                .map_err(|_| AnalysisFailure::ResourceLimited)?;
            contributors.push(binding.binding());
            let previous = contributors_by_name.insert(name, contributors);
            debug_assert!(previous.is_none());
        }
    }

    Ok(contributors_by_name)
}

fn copy_var_contributors<'script>(
    contributors: &[&'script SourceAnchor],
) -> Result<Vec<&'script SourceAnchor>, AnalysisFailure> {
    let mut copied = Vec::new();
    copied
        .try_reserve(contributors.len())
        .map_err(|_| AnalysisFailure::ResourceLimited)?;
    copied.extend_from_slice(contributors);
    Ok(copied)
}

fn correspondence_for_name<'script>(
    semantic_name: &str,
    current_region: SelectedVariableStatementNameCorrespondenceRegion<'script>,
    current_bindings: &LexicalBindingsByName<'script>,
    top_level_bindings: &LexicalBindingsByName<'script>,
    var_contributors: &VarContributorsByName<'script>,
) -> Result<SelectedVariableStatementNameCorrespondence<'script>, AnalysisFailure> {
    if let Some(binding) = current_bindings.get(semantic_name).copied() {
        return Ok(
            SelectedVariableStatementNameCorrespondence::VisibleSelectedLexicalBinding {
                binding,
                region: current_region,
            },
        );
    }

    if let SelectedVariableStatementNameCorrespondenceRegion::Block(_) = current_region
        && let Some(binding) = top_level_bindings.get(semantic_name).copied()
    {
        return Ok(
            SelectedVariableStatementNameCorrespondence::VisibleSelectedLexicalBinding {
                binding,
                region: SelectedVariableStatementNameCorrespondenceRegion::TopLevel,
            },
        );
    }

    if let Some(contributors) = var_contributors.get(semantic_name) {
        return Ok(
            SelectedVariableStatementNameCorrespondence::SameSourceSelectedVarNameContributors {
                contributors: copy_var_contributors(contributors)?,
            },
        );
    }

    Ok(SelectedVariableStatementNameCorrespondence::NoSelectedSameSourceContributor)
}

fn append_binding_relation<'script>(
    binding: &'script SelectedLexicalBinding,
    current_region: SelectedVariableStatementNameCorrespondenceRegion<'script>,
    current_bindings: &LexicalBindingsByName<'script>,
    top_level_bindings: &LexicalBindingsByName<'script>,
    var_contributors: &VarContributorsByName<'script>,
    relations: &mut Vec<SelectedVariableStatementNameCorrespondenceRelation<'script>>,
) -> Result<(), AnalysisFailure> {
    let Some(reference) = binding.identifier_reference_initializer() else {
        return Ok(());
    };

    let correspondence = correspondence_for_name(
        reference.semantic_name(),
        current_region,
        current_bindings,
        top_level_bindings,
        var_contributors,
    )?;

    relations
        .try_reserve(1)
        .map_err(|_| AnalysisFailure::ResourceLimited)?;
    relations.push(SelectedVariableStatementNameCorrespondenceRelation {
        containing_binding: binding.binding(),
        current_region,
        reference: reference.reference(),
        semantic_name: reference.semantic_name(),
        correspondence,
    });

    Ok(())
}

fn append_declaration_relations<'script>(
    declaration: &'script SelectedLexicalDeclaration,
    current_region: SelectedVariableStatementNameCorrespondenceRegion<'script>,
    current_bindings: &LexicalBindingsByName<'script>,
    top_level_bindings: &LexicalBindingsByName<'script>,
    var_contributors: &VarContributorsByName<'script>,
    relations: &mut Vec<SelectedVariableStatementNameCorrespondenceRelation<'script>>,
) -> Result<(), AnalysisFailure> {
    for binding in declaration.bindings() {
        append_binding_relation(
            binding,
            current_region,
            current_bindings,
            top_level_bindings,
            var_contributors,
            relations,
        )?;
    }
    Ok(())
}

fn analyze<'script>(
    script: &'script SelectedVariableStatementScript,
) -> Result<SelectedVariableStatementNameCorrespondenceAnalysis<'script>, AnalysisFailure> {
    let top_level_bindings = top_level_bindings(script)?;
    let var_contributors = var_contributors(script)?;
    let mut relations = Vec::new();

    for item in script.items() {
        match item {
            SelectedVariableTopLevelItem::LexicalDeclaration(declaration) => {
                append_declaration_relations(
                    declaration,
                    SelectedVariableStatementNameCorrespondenceRegion::TopLevel,
                    &top_level_bindings,
                    &top_level_bindings,
                    &var_contributors,
                    &mut relations,
                )?;
            }
            SelectedVariableTopLevelItem::Block(block) => {
                let current_bindings = block_bindings(block)?;
                let current_region =
                    SelectedVariableStatementNameCorrespondenceRegion::Block(block.block());
                for declaration in block.declarations() {
                    append_declaration_relations(
                        declaration,
                        current_region,
                        &current_bindings,
                        &top_level_bindings,
                        &var_contributors,
                        &mut relations,
                    )?;
                }
            }
            SelectedVariableTopLevelItem::VariableStatement(_) => {}
        }
    }

    Ok(SelectedVariableStatementNameCorrespondenceAnalysis { relations })
}

pub(super) fn analyze_selected_variable_statement_name_correspondence<'script>(
    accepted: &SelectedVariableStatementStaticSemanticsAccepted<'script>,
) -> SelectedVariableStatementNameCorrespondenceOutcome<'script> {
    match analyze(accepted.script()) {
        Ok(analysis) => SelectedVariableStatementNameCorrespondenceOutcome::Complete(analysis),
        Err(AnalysisFailure::ResourceLimited) => {
            SelectedVariableStatementNameCorrespondenceOutcome::ResourceLimited
        }
        Err(AnalysisFailure::InternalFailure) => {
            SelectedVariableStatementNameCorrespondenceOutcome::InternalFailure
        }
    }
}
