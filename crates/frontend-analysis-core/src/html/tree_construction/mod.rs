//! TC-S1 — Disabled-Scripting Document Shell Construction.
//!
//! The first Core-private HTML tree-construction capability, implemented on
//! the architecture approved under Issue #117 and recorded by ADR 0010
//! (`docs/decisions/0010-html-tree-construction-architecture.md`) and the
//! specialized normative contract
//! `docs/architecture/HTML_TREE_CONSTRUCTION.md`. The implementation guide is
//! `docs/development/HTML_DOCUMENT_SHELL_CONSTRUCTION.md`.
//!
//! ```text
//! &SourceText + HtmlTokenizerLimits
//!         ↓
//! driver — Core-owned TC-S1 coordination and effective completion
//!         ↓
//! existing batch tokenizer (unchanged)
//!         ↓
//! validated HtmlTokenizerRunResult
//!         ↓
//! session — private, exclusively mutable single-run construction state
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
mod after_body_successor_validation;
#[cfg(test)]
mod validation;
