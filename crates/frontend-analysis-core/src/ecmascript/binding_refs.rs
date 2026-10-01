//! Public selected flat lexical binding reference correspondence facade
//! (Issue #862, ADR 0011).
//!
//! This is the single intentional public ECMAScript analysis boundary over the
//! existing crate-private selected pipeline:
//!
//! ```text
//! &SourceText
//!   -> recognize_selected_lexical_slice
//!   -> RecognizedSelectedSlice only
//!   -> evaluate_selected_static_semantics
//!   -> Accepted witness
//!   -> analyze_selected_binding_scope
//!   -> owned EsBindingRefReport
//! ```
//!
//! The report states selected same-source name correspondence between
//! identifier-reference initializer facts and selected top-level lexical
//! bindings. It is not runtime `ResolveBinding`, TDZ analysis, value flow, or
//! a complete ECMAScript scope model. Every anchor is cloned from evidence
//! retained by the existing producers; no source text is searched, rescanned,
//! retokenized, or reparsed here. Recognized carriers other than the flat
//! selected lexical script are outside this question and are reported as
//! [`EsBindingRefOutcome::UnsupportedCoverage`].
//!
//! The report owns its source evidence and remains usable after the caller
//! drops the originating [`SourceText`]. Its anchors share single-threaded
//! ownership and create no `Send` or `Sync` promise.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

use super::selected_binding_scope::{
    SelectedBindingScopeOutcome, SelectedBindingScopeTarget, SelectedLexicalBindingOrder,
    analyze_selected_binding_scope,
};
use super::selected_lexical_slice::{
    SelectedLexicalSliceOutcome, recognize_selected_lexical_slice,
};
use super::selected_static_semantics::{
    SelectedStaticSemanticsOutcome, SelectedStaticSemanticsRejection,
    evaluate_selected_static_semantics,
};

#[cfg(test)]
mod tests;

/// Analyzes `source` as a Script for selected flat top-level lexical binding
/// initializer reference correspondence.
///
/// Ordinary analysis outcomes, including unsupported coverage, selected
/// rejection, and resource refusal, are returned as `Ok`. `Err` is reserved
/// for a returned Core boundary or internal contract failure.
pub fn analyze_selected_flat_lexical_binding_refs(
    source: &SourceText,
) -> Result<EsBindingRefReport, EsBindingRefCoreFailure> {
    let outcome = analyze_outcome(source)?;
    Ok(EsBindingRefReport {
        source_id: source.id(),
        outcome,
    })
}

fn analyze_outcome(source: &SourceText) -> Result<EsBindingRefOutcome, EsBindingRefCoreFailure> {
    let script = match recognize_selected_lexical_slice(source) {
        SelectedLexicalSliceOutcome::RecognizedSelectedSlice(script) => script,
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(_)
        | SelectedLexicalSliceOutcome::RecognizedVariableStatementSlice(_)
        | SelectedLexicalSliceOutcome::RecognizedIdentifierReferenceExpressionStatementSlice(_)
        | SelectedLexicalSliceOutcome::RecognizedBlockReferenceUseEnabledSlice(_)
        | SelectedLexicalSliceOutcome::UnsupportedCoverage => {
            return Ok(EsBindingRefOutcome::UnsupportedCoverage);
        }
        SelectedLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence { subject } => {
            return Ok(EsBindingRefOutcome::SelectedGrammarRejection { subject });
        }
        SelectedLexicalSliceOutcome::ResourceLimited => {
            return Ok(EsBindingRefOutcome::ResourceLimited);
        }
        SelectedLexicalSliceOutcome::InternalFailure => {
            return Err(EsBindingRefCoreFailure::InternalFailure);
        }
    };

    let accepted = match evaluate_selected_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Accepted(accepted) => accepted,
        SelectedStaticSemanticsOutcome::Rejected(rejection) => {
            return match project_static_rejection(rejection) {
                Some(rejection) => Ok(EsBindingRefOutcome::SelectedStaticRejection(rejection)),
                None => Err(EsBindingRefCoreFailure::InternalFailure),
            };
        }
        SelectedStaticSemanticsOutcome::ResourceLimited => {
            return Ok(EsBindingRefOutcome::ResourceLimited);
        }
        SelectedStaticSemanticsOutcome::InternalFailure => {
            return Err(EsBindingRefCoreFailure::InternalFailure);
        }
    };

    let analysis = match analyze_selected_binding_scope(&accepted) {
        SelectedBindingScopeOutcome::Complete(analysis) => analysis,
        SelectedBindingScopeOutcome::ResourceLimited => {
            return Ok(EsBindingRefOutcome::ResourceLimited);
        }
        SelectedBindingScopeOutcome::InternalFailure => {
            return Err(EsBindingRefCoreFailure::InternalFailure);
        }
    };

    let mut relations = Vec::new();
    for relation in analysis.relations() {
        if relations.try_reserve(1).is_err() {
            return Ok(EsBindingRefOutcome::ResourceLimited);
        }
        let mut semantic_name = String::new();
        if semantic_name
            .try_reserve_exact(relation.semantic_name().len())
            .is_err()
        {
            return Ok(EsBindingRefOutcome::ResourceLimited);
        }
        semantic_name.push_str(relation.semantic_name());

        let target = match relation.target() {
            SelectedBindingScopeTarget::SameSourceSelectedLexicalBinding { binding, order } => {
                EsBindingRefTarget::SameSourceSelectedLexicalBinding {
                    binding: binding.clone(),
                    order: project_order(order),
                }
            }
            SelectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding => {
                EsBindingRefTarget::NoSameSourceSelectedLexicalBinding
            }
        };
        relations.push(EsBindingRefRelation {
            containing_binding: relation.containing_binding().clone(),
            reference: relation.reference().clone(),
            semantic_name,
            target,
        });
    }

    Ok(EsBindingRefOutcome::Complete { relations })
}

fn project_order(order: SelectedLexicalBindingOrder) -> EsBindingRefOrder {
    match order {
        SelectedLexicalBindingOrder::Before => EsBindingRefOrder::Before,
        SelectedLexicalBindingOrder::Same => EsBindingRefOrder::Same,
        SelectedLexicalBindingOrder::After => EsBindingRefOrder::After,
    }
}

/// Projects only the rejections reachable for the accepted flat selected
/// lexical carrier. Any other variant is an internal contract violation.
fn project_static_rejection(
    rejection: SelectedStaticSemanticsRejection,
) -> Option<EsBindingRefStaticRejection> {
    match rejection {
        SelectedStaticSemanticsRejection::InvalidEscapedIdentifierStart { escape } => {
            Some(EsBindingRefStaticRejection::InvalidEscapedIdentifierStart { escape })
        }
        SelectedStaticSemanticsRejection::InvalidEscapedIdentifierPart { escape } => {
            Some(EsBindingRefStaticRejection::InvalidEscapedIdentifierPart { escape })
        }
        SelectedStaticSemanticsRejection::EscapedReservedWord { binding } => {
            Some(EsBindingRefStaticRejection::EscapedReservedWordBinding { binding })
        }
        SelectedStaticSemanticsRejection::EscapedReservedWordInitializer { identifier } => {
            Some(EsBindingRefStaticRejection::EscapedReservedWordInitializer { identifier })
        }
        SelectedStaticSemanticsRejection::BindingNamedLet { binding } => {
            Some(EsBindingRefStaticRejection::BindingNamedLet { binding })
        }
        SelectedStaticSemanticsRejection::DuplicateDeclarationBinding {
            first_binding,
            duplicate_binding,
        } => Some(EsBindingRefStaticRejection::DuplicateDeclarationBinding {
            first_binding,
            duplicate_binding,
        }),
        SelectedStaticSemanticsRejection::ConstBindingMissingInitializer { binding } => {
            Some(EsBindingRefStaticRejection::ConstBindingMissingInitializer { binding })
        }
        SelectedStaticSemanticsRejection::DuplicateLexicalName {
            first_binding,
            duplicate_binding,
        } => Some(EsBindingRefStaticRejection::DuplicateLexicalName {
            first_binding,
            duplicate_binding,
        }),
        _ => None,
    }
}

/// Owned result of one selected flat lexical binding reference analysis.
#[derive(Debug, Clone)]
pub struct EsBindingRefReport {
    source_id: SourceId,
    outcome: EsBindingRefOutcome,
}

impl EsBindingRefReport {
    /// Returns the identity of the analyzed source.
    pub fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// Returns the analysis outcome.
    pub fn outcome(&self) -> &EsBindingRefOutcome {
        &self.outcome
    }
}

/// Distinct analysis outcomes. Zero relations in `Complete` is a complete
/// analysis, not unsupported coverage.
#[derive(Debug, Clone)]
pub enum EsBindingRefOutcome {
    /// The source is inside the selected scope and every retained fact was
    /// classified, in producer order.
    Complete {
        relations: Vec<EsBindingRefRelation>,
    },
    /// The source is outside the selected flat lexical binding-initializer
    /// scope. This is not a validity judgment.
    UnsupportedCoverage,
    /// Existing selected grammar evidence rejected the source.
    SelectedGrammarRejection { subject: SourceAnchor },
    /// Existing selected static-semantics evidence rejected the source.
    SelectedStaticRejection(EsBindingRefStaticRejection),
    /// An existing bounded allocation or execution refusal occurred.
    ResourceLimited,
}

/// One retained identifier-reference fact and its selected correspondence.
#[derive(Debug, Clone)]
pub struct EsBindingRefRelation {
    containing_binding: SourceAnchor,
    reference: SourceAnchor,
    semantic_name: String,
    target: EsBindingRefTarget,
}

impl EsBindingRefRelation {
    /// Authored anchor of the binding whose initializer retains the reference.
    pub fn containing_binding(&self) -> &SourceAnchor {
        &self.containing_binding
    }

    /// Authored anchor of the identifier reference.
    pub fn reference(&self) -> &SourceAnchor {
        &self.reference
    }

    /// Semantic name of the reference; may differ from its authored spelling.
    pub fn semantic_name(&self) -> &str {
        &self.semantic_name
    }

    /// Selected same-source target classification.
    pub fn target(&self) -> &EsBindingRefTarget {
        &self.target
    }
}

/// Selected same-source name correspondence target.
#[derive(Debug, Clone)]
pub enum EsBindingRefTarget {
    SameSourceSelectedLexicalBinding {
        binding: SourceAnchor,
        order: EsBindingRefOrder,
    },
    NoSameSourceSelectedLexicalBinding,
}

/// Declaration order of the target relative to the containing binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EsBindingRefOrder {
    Before,
    Same,
    After,
}

/// Selected static rejections reachable for the flat selected carrier.
#[derive(Debug, Clone)]
pub enum EsBindingRefStaticRejection {
    InvalidEscapedIdentifierStart {
        escape: SourceAnchor,
    },
    InvalidEscapedIdentifierPart {
        escape: SourceAnchor,
    },
    EscapedReservedWordBinding {
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

/// Returned Core boundary or internal contract failure. `Display` never
/// includes authored source content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EsBindingRefCoreFailure {
    InternalFailure,
}

impl fmt::Display for EsBindingRefCoreFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InternalFailure => {
                f.write_str("ECMAScript binding reference analysis internal failure")
            }
        }
    }
}

impl Error for EsBindingRefCoreFailure {}
