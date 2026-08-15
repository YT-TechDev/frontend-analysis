//! First production static-semantics slice for the selected ECMAScript Script grammar.
//!
//! This module consumes only the source-backed declaration facts retained by the
//! first lexical slice. It does not reparse authoritative source, broaden grammar
//! coverage, or construct aggregate `QualificationOutcome::Qualified`.

use std::collections::HashMap;

use crate::{SourceAnchor, SourceText};

use super::first_lexical_slice::SelectedLexicalScript;
use super::qualification::{EvidenceSubject, QualificationOutcome};

/// Slice-local result after every static-semantic obligation owned by this Leaf
/// has been checked for a #204-recognized selected Script.
#[derive(Debug)]
pub(super) enum SelectedStaticSemanticsOutcome {
    Accepted,
    Rejected(SelectedStaticSemanticsRejection),
    ResourceLimited,
}

/// Private supporting evidence for the two selected Early Error families that
/// can trigger for the current grammar subset.
#[derive(Debug)]
pub(super) enum SelectedStaticSemanticsRejection {
    BindingNamedLet {
        binding: SourceAnchor,
    },
    DuplicateLexicalName {
        first_binding: SourceAnchor,
        duplicate_binding: SourceAnchor,
    },
}

impl SelectedStaticSemanticsRejection {
    fn primary_binding(&self) -> &SourceAnchor {
        match self {
            Self::BindingNamedLet { binding } => binding,
            Self::DuplicateLexicalName {
                duplicate_binding, ..
            } => duplicate_binding,
        }
    }
}

/// Evaluates the first bounded selected static-semantics capability.
///
/// The current selected identifiers are unescaped, so exact binding-fragment
/// equality is equivalent to `StringValue` equality only for this grammar
/// subset. No Unicode normalization is performed.
///
/// Declaration-local `let` rejection is intentionally checked for the entire
/// selected Script before Script-level duplicate-name rejection. This is a
/// deterministic project evidence-selection policy, not an ECMAScript-specified
/// diagnostic order.
pub(super) fn evaluate_first_static_semantics(
    script: &SelectedLexicalScript,
) -> SelectedStaticSemanticsOutcome {
    let declarations = script.declarations();

    for declaration in declarations {
        if declaration.binding().fragment() == "let" {
            return SelectedStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::BindingNamedLet {
                    binding: declaration.binding().clone(),
                },
            );
        }
    }

    let mut first_by_name: HashMap<&str, usize> = HashMap::new();

    for (index, declaration) in declarations.iter().enumerate() {
        let name = declaration.binding().fragment();

        if let Some(&first_index) = first_by_name.get(name) {
            return SelectedStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::DuplicateLexicalName {
                    first_binding: declarations[first_index].binding().clone(),
                    duplicate_binding: declaration.binding().clone(),
                },
            );
        }

        if first_by_name.try_reserve(1).is_err() {
            return SelectedStaticSemanticsOutcome::ResourceLimited;
        }

        let previous = first_by_name.insert(name, index);
        debug_assert!(previous.is_none());
    }

    SelectedStaticSemanticsOutcome::Accepted
}

/// Maps only a proven selected rejection into the existing whole-source #199
/// rejection contract.
///
/// `source` participates solely in source-ownership reconciliation for the
/// already-retained primary authored anchor. A mismatch fails closed as
/// incomplete/internal processing and never fabricates a semantic verdict.
pub(super) fn selected_rejection_to_qualification(
    source: &SourceText,
    rejection: &SelectedStaticSemanticsRejection,
) -> QualificationOutcome {
    let subject = match EvidenceSubject::authored(source, rejection.primary_binding().clone()) {
        Ok(subject) => subject,
        Err(_) => return QualificationOutcome::internal_failure(),
    };

    QualificationOutcome::static_semantics_rejected(subject)
}
