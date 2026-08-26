//! TC-S1 — Disabled-Scripting Document Shell Construction, plus the accepted
//! TC-S2 — Selected After-Body Uniform Character-Run Handling, TC-S3 —
//! Selected In-Body No-Attribute `div` Construction, TC-S4 — Selected
//! In-Body Heterogeneous `div`/`section` Block Closure Recovery, and TC-S5 —
//! Selected In-Body `p` Lifecycle with Bounded Implicit Closure and
//! Unmatched-End Synthesis, and TC-S6 — Selected End Tags over Current P with
//! Bounded Non-Noop Implied-End Handling successors.
//!
//! The first Core-private HTML tree-construction capability, implemented on
//! the architecture approved under Issue #117 and recorded by ADR 0010
//! (`docs/decisions/0010-html-tree-construction-architecture.md`) and the
//! specialized normative contract
//! `docs/architecture/HTML_TREE_CONSTRUCTION.md`. The implementation guide is
//! `docs/development/HTML_DOCUMENT_SHELL_CONSTRUCTION.md`. TC-S2's accepted
//! `AfterBody -> InBody` recovery back-edge (Issue #355) is layered onto the
//! same driver/session/result split without moving any of its boundaries.
//! TC-S3's accepted selected `div` construction (Issue #359) is layered on the
//! same way: it adds a closed private selected ordinary HTML-element domain
//! inside the existing session/result ownership, leaving the tokenizer
//! tree-unaware and the driver tag-name agnostic. TC-S4's accepted
//! heterogeneous closure recovery (Issue #363) is layered on that in turn: it
//! grows the same closed domain from `div` to exactly `div | section` and adds
//! one narrow recovery relation, so a selected end tag whose nearest same-name
//! target is not the current node pops the intervening selected elements
//! before closing that target. TC-S5 (Issue #367) keeps that authored-only
//! `div | section` domain closed and adds a separate private Paragraph domain
//! capable of representing both an authored `<p>` and the source-less P
//! synthesized by the unmatched-`</p>` rule. It adds no driver semantics and
//! no tokenizer feedback.
//!
//! ```text
//! &SourceText + HtmlTokenizerLimits
//!         ↓
//! driver — Core-owned coordination, same-token redispatch, and effective
//!          completion
//!         ↓
//! existing batch tokenizer (unchanged)
//!         ↓
//! validated HtmlTokenizerRunResult
//!         ↓
//! session — private, exclusively mutable single-run construction state;
//!           one insertion-mode dispatch per driver call
//!         ↓
//! validated freeze
//!         ↓
//! result — immutable tree / provenance / action / diagnostic / completion
//! ```
//!
//! Ownership is split so that no module can quietly take another's
//! responsibility:
//!
//! - [`driver`] coordinates. It is the only module that calls the tokenizer.
//! - [`session`] owns all mutable construction state for one run. It never
//!   calls the tokenizer and never escapes to a consumer.
//! - [`result`] owns immutable, validated durable meaning. It owns no mutable
//!   state and never observes the tokenizer.
//!
//! This subsystem is crate-private. It creates no public Rust API,
//! serialization, ABI, browser-protocol, DOM, or product compatibility
//! promise, adds no dependency, feature, workspace target, async, concurrency,
//! shared mutation, or `unsafe Rust`, and adds no tree resource dimension,
//! limit, or numeric constant.
//!
//! It is a bounded slice, not HTML parsing. The existing explicit-start-tag
//! analysis capability remains an unchanged sibling.

pub(crate) mod driver;
pub(crate) mod result;
pub(crate) mod session;

#[cfg(test)]
mod after_body_successor_production;
#[cfg(test)]
mod after_body_successor_validation;
#[cfg(test)]
mod in_body_body_end_open_stack_successor_production;
#[cfg(test)]
mod in_body_body_end_open_stack_successor_validation;
#[cfg(test)]
mod in_body_div_section_successor_production;
#[cfg(test)]
mod in_body_div_section_successor_validation;
#[cfg(test)]
mod in_body_div_successor_production;
#[cfg(test)]
mod in_body_div_successor_validation;
#[cfg(test)]
mod in_body_html_end_open_stack_successor_validation;
#[cfg(test)]
mod in_body_p_implied_end_successor_production;
#[cfg(test)]
mod in_body_p_implied_end_successor_validation;
#[cfg(test)]
mod in_body_p_successor_production;
#[cfg(test)]
mod in_body_p_successor_validation;
#[cfg(test)]
mod validation;
