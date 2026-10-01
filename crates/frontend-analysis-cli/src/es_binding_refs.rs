//! Human-readable presentation of the `fa es-binding-refs` report (#862).
//!
//! Every semantic classification, order, and source anchor is read from the
//! owned Core report; this module only chooses wording. Source evidence is
//! rendered from retained anchors and nothing is searched or reconstructed.

use std::fmt::Write as _;

use frontend_analysis_core::SourceAnchor;
use frontend_analysis_core::ecmascript::binding_refs::{
    EsBindingRefOrder, EsBindingRefOutcome, EsBindingRefReport, EsBindingRefStaticRejection,
    EsBindingRefTarget,
};

use crate::render_evidence;

pub(crate) fn render_report(source_bytes: usize, report: &EsBindingRefReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "capability: es-binding-refs");
    let _ = writeln!(
        out,
        "source: id {}; {source_bytes} bytes",
        report.source_id().value()
    );
    let _ = writeln!(out, "goal: Script");
    let _ = writeln!(out, "scope: selected flat lexical binding initializers");

    match report.outcome() {
        EsBindingRefOutcome::Complete { relations } => {
            let _ = writeln!(out, "analysis: complete");
            let _ = writeln!(out, "relations: {}", relations.len());
            for (index, relation) in relations.iter().enumerate() {
                let _ = writeln!(out, "relation {}:", index + 1);
                let _ = writeln!(
                    out,
                    "  name: \"{}\"",
                    crate::escape_fragment(relation.semantic_name())
                );
                let _ = writeln!(
                    out,
                    "  containing binding: {}",
                    render_evidence(relation.containing_binding())
                );
                let _ = writeln!(
                    out,
                    "  reference: {}",
                    render_evidence(relation.reference())
                );
                match relation.target() {
                    EsBindingRefTarget::SameSourceSelectedLexicalBinding { binding, order } => {
                        let order = match order {
                            EsBindingRefOrder::Before => "before",
                            EsBindingRefOrder::Same => "same",
                            EsBindingRefOrder::After => "after",
                        };
                        let _ = writeln!(
                            out,
                            "  target: same-source selected lexical binding ({order})"
                        );
                        let _ = writeln!(out, "    binding: {}", render_evidence(binding));
                    }
                    EsBindingRefTarget::NoSameSourceSelectedLexicalBinding => {
                        let _ = writeln!(out, "  target: no same-source selected lexical binding");
                    }
                }
            }
        }
        EsBindingRefOutcome::UnsupportedCoverage => {
            let _ = writeln!(out, "analysis: unsupported coverage for selected scope");
        }
        EsBindingRefOutcome::SelectedGrammarRejection { subject } => {
            let _ = writeln!(out, "analysis: selected grammar rejection");
            let _ = writeln!(out, "  subject: {}", render_evidence(subject));
        }
        EsBindingRefOutcome::SelectedStaticRejection(rejection) => {
            let (label, anchors) = static_rejection(rejection);
            let _ = writeln!(out, "analysis: selected static rejection ({label})");
            for (role, anchor) in anchors {
                let _ = writeln!(out, "  {role}: {}", render_evidence(anchor));
            }
        }
        EsBindingRefOutcome::ResourceLimited => {
            let _ = writeln!(out, "analysis: resource limited");
        }
    }
    out
}

fn static_rejection(
    rejection: &EsBindingRefStaticRejection,
) -> (&'static str, Vec<(&'static str, &SourceAnchor)>) {
    match rejection {
        EsBindingRefStaticRejection::InvalidEscapedIdentifierStart { escape } => {
            ("invalid escaped identifier start", vec![("escape", escape)])
        }
        EsBindingRefStaticRejection::InvalidEscapedIdentifierPart { escape } => {
            ("invalid escaped identifier part", vec![("escape", escape)])
        }
        EsBindingRefStaticRejection::EscapedReservedWordBinding { binding } => {
            ("escaped reserved word binding", vec![("binding", binding)])
        }
        EsBindingRefStaticRejection::EscapedReservedWordInitializer { identifier } => (
            "escaped reserved word initializer",
            vec![("identifier", identifier)],
        ),
        EsBindingRefStaticRejection::BindingNamedLet { binding } => {
            ("binding named let", vec![("binding", binding)])
        }
        EsBindingRefStaticRejection::DuplicateDeclarationBinding {
            first_binding,
            duplicate_binding,
        } => (
            "duplicate declaration binding",
            vec![
                ("first binding", first_binding),
                ("duplicate binding", duplicate_binding),
            ],
        ),
        EsBindingRefStaticRejection::ConstBindingMissingInitializer { binding } => (
            "const binding missing initializer",
            vec![("binding", binding)],
        ),
        EsBindingRefStaticRejection::DuplicateLexicalName {
            first_binding,
            duplicate_binding,
        } => (
            "duplicate lexical name",
            vec![
                ("first binding", first_binding),
                ("duplicate binding", duplicate_binding),
            ],
        ),
    }
}
