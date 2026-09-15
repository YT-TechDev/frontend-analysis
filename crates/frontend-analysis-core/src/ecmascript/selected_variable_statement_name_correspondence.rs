//! Private selected var-enabled source-name correspondence for Issue #316.
//!
//! This capability consumes only the exact var-enabled selected-static
//! acceptance witness. It derives source-backed correspondence over the current
//! selected top-level / one-level-Block region path and preserves authored
//! relation traversal order plus every authored same-name `var` contributor in
//! authored order. #324 widens the top-level contributor domain so several
//! declarators of one `VariableStatement` each contribute an anchor. #334
//! makes each selected direct-authored var initializer `IdentifierReference`
//! an independently ordered top-level correspondence input, and #338 composes
//! the selected escaped non-ReservedWord spelling through the same retained
//! fact. #703 widens the contributor domain itself from selected top-level
//! `VariableStatement` declarators only to every selected authored `var`
//! declarator in the Script, adding selected one-level Block-contained `var`
//! declarators as same-source contributors, uniformly for every existing
//! correspondence query input and in exact global authored source order.
//! #705/#706 freeze the accepted-witness lifecycle theorem for the distinct
//! `SelectedOneLevelBlockStaticSemanticsAccepted` witness. The production
//! successor adds a second `pub(super)` entrypoint that consumes that witness
//! directly. It shares this one correspondence semantic owner and every
//! existing meaning and precedence rule; it does not introduce a fourth
//! correspondence meaning or a parallel type hierarchy.
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
    SelectedBlock, SelectedBlockItem, SelectedBlockVarBinding, SelectedLexicalBinding,
    SelectedLexicalDeclaration, SelectedOneLevelBlockScript, SelectedTopLevelItem,
    SelectedVariableBinding, SelectedVariableStatementScript, SelectedVariableTopLevelItem,
};
use super::selected_static_semantics::{
    SelectedOneLevelBlockStaticSemanticsAccepted, SelectedVariableStatementStaticSemanticsAccepted,
};

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

// Appends one authored contributor anchor for `semantic_name`, composing with
// any contributor already collected for that name in earlier authored order.
// Repeated names are never deduplicated into a singular logical target; this
// is the sole insertion mechanics shared by top-level `VariableStatement` and
// one-level Block `var` contributors (Issue #703).
fn append_var_contributor<'script>(
    contributors_by_name: &mut VarContributorsByName<'script>,
    semantic_name: &'script str,
    binding: &'script SourceAnchor,
) -> Result<(), AnalysisFailure> {
    if let Some(contributors) = contributors_by_name.get_mut(semantic_name) {
        contributors
            .try_reserve(1)
            .map_err(|_| AnalysisFailure::ResourceLimited)?;
        contributors.push(binding);
        return Ok(());
    }

    contributors_by_name
        .try_reserve(1)
        .map_err(|_| AnalysisFailure::ResourceLimited)?;
    let mut contributors = Vec::new();
    contributors
        .try_reserve(1)
        .map_err(|_| AnalysisFailure::ResourceLimited)?;
    contributors.push(binding);
    let previous = contributors_by_name.insert(semantic_name, contributors);
    debug_assert!(previous.is_none());
    Ok(())
}

fn var_contributors(
    script: &SelectedVariableStatementScript,
) -> Result<VarContributorsByName<'_>, AnalysisFailure> {
    let mut contributors_by_name: VarContributorsByName<'_> = HashMap::new();

    // Every selected authored `var` declarator in the Script contributes, in
    // exact global authored order: `LexicalDeclaration` items contribute
    // nothing; each `VariableStatement` declarator contributes in authored
    // `VariableDeclarationList` order; each one-level Block's `var`
    // declarators contribute in authored Block-item / declarator order
    // (Issue #703). Placement never groups contributors by region.
    for item in script.items() {
        match item {
            SelectedVariableTopLevelItem::LexicalDeclaration(_) => {}
            SelectedVariableTopLevelItem::VariableStatement(statement) => {
                for binding in statement.bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(AnalysisFailure::InternalFailure);
                    };
                    append_var_contributor(&mut contributors_by_name, name, binding.binding())?;
                }
            }
            SelectedVariableTopLevelItem::Block(block) => {
                for binding in block.block_var_bindings() {
                    let Some(name) = binding.semantic_name() else {
                        return Err(AnalysisFailure::InternalFailure);
                    };
                    append_var_contributor(&mut contributors_by_name, name, binding.binding())?;
                }
            }
        }
    }

    Ok(contributors_by_name)
}

// One-level Block accepted-witness traversal, implementing the lifecycle
// theorem frozen by Issue #705 / PR #706. There is no top-level
// `VariableStatement` item in `SelectedOneLevelBlockScript`, so these
// functions traverse the distinct two-variant `SelectedTopLevelItem`
// enum, but share every insertion/precedence helper above with the
// `SelectedVariableStatementScript` traversal rather than introducing a
// parallel correspondence semantic owner.

fn one_level_block_top_level_bindings(
    script: &SelectedOneLevelBlockScript,
) -> Result<LexicalBindingsByName<'_>, AnalysisFailure> {
    let mut bindings_by_name = HashMap::new();

    for item in script.items() {
        let SelectedTopLevelItem::LexicalDeclaration(declaration) = item else {
            continue;
        };
        insert_declaration_bindings(declaration, &mut bindings_by_name)?;
    }

    Ok(bindings_by_name)
}

fn one_level_block_var_contributors(
    script: &SelectedOneLevelBlockScript,
) -> Result<VarContributorsByName<'_>, AnalysisFailure> {
    let mut contributors_by_name: VarContributorsByName<'_> = HashMap::new();

    // Every selected authored Block `var` declarator contributes, in exact
    // global authored order across every selected Block.
    for item in script.items() {
        let SelectedTopLevelItem::Block(block) = item else {
            continue;
        };
        for binding in block.block_var_bindings() {
            let Some(name) = binding.semantic_name() else {
                return Err(AnalysisFailure::InternalFailure);
            };
            append_var_contributor(&mut contributors_by_name, name, binding.binding())?;
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

// Block-var analog of `append_binding_relation` for the exact containing
// `SelectedBlockVarBinding` (Issue #710, widened to an escaped
// non-ReservedWord RHS by Issue #713). A direct or escaped RHS relation is
// owned by its exact containing Block-var declarator, using the same
// current-region, current-Block-lexical-precedence,
// top-level-lexical-fallback, and all-selected-var-contributor semantics as
// every other correspondence input; no fourth correspondence meaning is
// introduced. Decimal and absent initializers carry no
// `identifier_reference_initializer` fact and contribute no relation.
fn append_block_var_binding_relation<'script>(
    binding: &'script SelectedBlockVarBinding,
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

fn append_variable_binding_relation<'script>(
    binding: &'script SelectedVariableBinding,
    top_level_bindings: &LexicalBindingsByName<'script>,
    var_contributors: &VarContributorsByName<'script>,
    relations: &mut Vec<SelectedVariableStatementNameCorrespondenceRelation<'script>>,
) -> Result<(), AnalysisFailure> {
    let Some(reference) = binding.identifier_reference_initializer() else {
        return Ok(());
    };
    let current_region = SelectedVariableStatementNameCorrespondenceRegion::TopLevel;
    let correspondence = correspondence_for_name(
        reference.semantic_name(),
        current_region,
        top_level_bindings,
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

// Shared mixed Block-item relation traversal for both accepted-witness
// routes (Issue #710). Once Block `var` bindings can own RHS
// `IdentifierReference` queries, `block.declarations()` (lexical-only) is
// no longer sufficient to preserve authored relation order across a Block
// that mixes lexical declarations and `var` statements: relations must
// follow exact authored `block.items()` order, not a grouping by
// declaration kind. `block.declarations()` and `block_var_bindings()`
// remain the distinct lexical-visibility and var-contributor accessors used
// elsewhere; this traversal is the correspondence-query consumer only.
fn append_block_item_relations<'script>(
    block: &'script SelectedBlock,
    current_region: SelectedVariableStatementNameCorrespondenceRegion<'script>,
    current_bindings: &LexicalBindingsByName<'script>,
    top_level_bindings: &LexicalBindingsByName<'script>,
    var_contributors: &VarContributorsByName<'script>,
    relations: &mut Vec<SelectedVariableStatementNameCorrespondenceRelation<'script>>,
) -> Result<(), AnalysisFailure> {
    for item in block.items() {
        match item {
            SelectedBlockItem::LexicalDeclaration(declaration) => {
                append_declaration_relations(
                    declaration,
                    current_region,
                    current_bindings,
                    top_level_bindings,
                    var_contributors,
                    relations,
                )?;
            }
            SelectedBlockItem::Var(statement) => {
                for binding in statement.bindings() {
                    append_block_var_binding_relation(
                        binding,
                        current_region,
                        current_bindings,
                        top_level_bindings,
                        var_contributors,
                        relations,
                    )?;
                }
            }
        }
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
                append_block_item_relations(
                    block,
                    current_region,
                    &current_bindings,
                    &top_level_bindings,
                    &var_contributors,
                    &mut relations,
                )?;
            }
            SelectedVariableTopLevelItem::VariableStatement(statement) => {
                for binding in statement.bindings() {
                    append_variable_binding_relation(
                        binding,
                        &top_level_bindings,
                        &var_contributors,
                        &mut relations,
                    )?;
                }
            }
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

fn analyze_one_level_block<'script>(
    script: &'script SelectedOneLevelBlockScript,
) -> Result<SelectedVariableStatementNameCorrespondenceAnalysis<'script>, AnalysisFailure> {
    let top_level_bindings = one_level_block_top_level_bindings(script)?;
    let var_contributors = one_level_block_var_contributors(script)?;
    let mut relations = Vec::new();

    for item in script.items() {
        match item {
            SelectedTopLevelItem::LexicalDeclaration(declaration) => {
                append_declaration_relations(
                    declaration,
                    SelectedVariableStatementNameCorrespondenceRegion::TopLevel,
                    &top_level_bindings,
                    &top_level_bindings,
                    &var_contributors,
                    &mut relations,
                )?;
            }
            SelectedTopLevelItem::Block(block) => {
                let current_bindings = block_bindings(block)?;
                let current_region =
                    SelectedVariableStatementNameCorrespondenceRegion::Block(block.block());
                append_block_item_relations(
                    block,
                    current_region,
                    &current_bindings,
                    &top_level_bindings,
                    &var_contributors,
                    &mut relations,
                )?;
            }
        }
    }

    Ok(SelectedVariableStatementNameCorrespondenceAnalysis { relations })
}

/// Second accepted-witness production entrypoint for the distinct
/// `SelectedOneLevelBlockStaticSemanticsAccepted` witness, implementing the
/// lifecycle theorem frozen by Issue #705 / PR #706. It shares this module's
/// single correspondence semantic owner, every existing precedence and
/// insertion helper, and the existing
/// `SelectedVariableStatementNameCorrespondenceOutcome` result type; it
/// introduces no fourth correspondence meaning and no parallel type
/// hierarchy.
pub(super) fn analyze_selected_one_level_block_name_correspondence<'script>(
    accepted: &SelectedOneLevelBlockStaticSemanticsAccepted<'script>,
) -> SelectedVariableStatementNameCorrespondenceOutcome<'script> {
    match analyze_one_level_block(accepted.script()) {
        Ok(analysis) => SelectedVariableStatementNameCorrespondenceOutcome::Complete(analysis),
        Err(AnalysisFailure::ResourceLimited) => {
            SelectedVariableStatementNameCorrespondenceOutcome::ResourceLimited
        }
        Err(AnalysisFailure::InternalFailure) => {
            SelectedVariableStatementNameCorrespondenceOutcome::InternalFailure
        }
    }
}
