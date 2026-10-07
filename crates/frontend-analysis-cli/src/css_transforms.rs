//! Human-readable presentation of the `fa css-transforms` report (#914).
//!
//! Analysis meaning remains owned by the CSS transform facade. This module only
//! renders the retained report and source evidence deterministically. It
//! searches no source text, decodes no CSS, and never infers that a report
//! with zero observations means the source contains no transform: the scope
//! line states the selected capability on every report.

use std::collections::HashMap;
use std::fmt::Write as _;

use frontend_analysis_core::css::transform::{
    CssTransformOutcome, CssTransformProfile, CssTransformReport, CssTransformUnsupportedReason,
};

use crate::css_selectors::render_upstream_stages;
use crate::{render_evidence, render_location};

pub(crate) fn render_report(source_bytes: usize, report: &CssTransformReport) -> String {
    let mut out = String::new();
    let profile = match report.profile() {
        CssTransformProfile::SelectedDirectAuthoredTransform => {
            "selected direct-authored transform qualification"
        }
    };
    let _ = writeln!(out, "capability: css-transforms");
    let _ = writeln!(out, "profile: {profile}");
    let _ = writeln!(
        out,
        "source: id {}; {source_bytes} bytes",
        report.source_id().value()
    );

    render_upstream_stages(&mut out, report.tokenizer(), report.parser());

    let _ = writeln!(
        out,
        "scope: retained ordinary declarations recognized as transform; not a source-wide \
         transform detector"
    );
    let _ = writeln!(
        out,
        "observations: {} selected",
        report.observations().len()
    );

    // A context header can be long and is shared by every declaration it
    // directly owns: its fragment is rendered at its first reference and later
    // references name only the same already-owned location.
    let mut first_reference: HashMap<(usize, usize), usize> = HashMap::new();
    for (index, observation) in report.observations().iter().enumerate() {
        let number = index + 1;
        let outcome = match observation.outcome() {
            CssTransformOutcome::Qualified => {
                "qualified by selected direct-authored profile".to_owned()
            }
            CssTransformOutcome::InvalidForSelectedValueGrammar => {
                "invalid for selected value grammar".to_owned()
            }
            CssTransformOutcome::UnsupportedBySelectedValueProfile(reason) => format!(
                "unsupported by selected value profile ({})",
                unsupported_reason(reason)
            ),
        };
        let _ = writeln!(out, "observation {number}: {outcome}");
        let _ = writeln!(
            out,
            "  declaration: {}",
            render_evidence(observation.declaration())
        );
        let _ = writeln!(
            out,
            "  property: {}",
            render_evidence(observation.property())
        );
        let _ = writeln!(out, "  value: {}", render_evidence(observation.value()));
        match observation.priority() {
            Some(priority) => {
                let _ = writeln!(out, "  priority: {}", render_evidence(priority));
            }
            None => {
                let _ = writeln!(out, "  priority: none");
            }
        }

        let context = observation.context();
        let key = (context.range().start(), context.range().end());
        match first_reference.get(&key) {
            Some(first) => {
                let _ = writeln!(
                    out,
                    "  context: {} (same context as observation {first})",
                    render_location(context)
                );
            }
            None => {
                first_reference.insert(key, number);
                let _ = writeln!(out, "  context: {}", render_evidence(context));
            }
        }
    }
    out
}

fn unsupported_reason(reason: CssTransformUnsupportedReason) -> &'static str {
    match reason {
        CssTransformUnsupportedReason::CssWideKeyword => "css-wide keyword",
        CssTransformUnsupportedReason::DeferredSubstitutionFunction => {
            "deferred substitution function"
        }
        CssTransformUnsupportedReason::WholeValueFunction => "whole-value function",
        CssTransformUnsupportedReason::UnselectedTransformFunction => {
            "unselected transform function"
        }
        CssTransformUnsupportedReason::FunctionValuedTransformArgument => {
            "function-valued transform argument"
        }
    }
}
