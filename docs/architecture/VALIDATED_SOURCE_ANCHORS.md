# Validated Source Anchors Guide

## Purpose and authority

This non-normative guide helps contributors work within the approved Validated
Source Anchors contracts. [ADR 0003](../decisions/0003-validated-source-anchors-first-rust-core-domain.md)
owns the selected domain and crate boundary, [ADR 0004](../decisions/0004-validated-source-anchor-semantics.md)
owns source-anchor semantics, [Architecture Layers](LAYERS.md) owns layer
responsibilities, and [Rust Core Contracts](RUST_CORE_CONTRACTS.md) owns general
Rust design constraints. This guide explains those decisions; it does not
create a stable external API, serialization format, release promise, MSRV,
WASM, FFI, or browser compatibility guarantee.

## Current domain responsibility

`frontend-analysis-core` currently owns only caller-supplied,
browser-independent `SourceId` values; immutable ownership of exact UTF-8
source; validated half-open UTF-8 byte ranges; owned `SourceAnchor` values;
exact fragment borrowing; and deterministic typed range errors. This is the
first Core domain, not a generic utility layer.

## Core invariants

- Exact source bytes are preserved without trimming, newline conversion,
  normalization, BOM removal, or replacement.
- Ranges are half-open `[start, end)` UTF-8 byte offsets, and both offsets must
  be character boundaries.
- Empty ranges are valid at character boundaries.
- Validation order is deterministic: reversal, bounds, start boundary, then
  end boundary.
- An anchor retains its source independently of the caller's lifetime, without
  copying the complete source for every anchor.
- Storage representation remains private.
- Source identity remains caller-supplied; Core implies no global uniqueness.
- No `Send`, `Sync`, or serialization guarantee exists.

## Ownership by layer

The entries for future layers describe boundaries, not claims that their
implementations currently exist.

| Layer or owner | Responsibility |
| --- | --- |
| Caller or source acquisition boundary | Own exact input acquisition and assign a scoped `SourceId`. |
| Parser | Compute parser-owned tokens or syntax and map accepted positions to authoritative UTF-8 byte offsets. |
| Browser Adapter | Convert protocol-specific identities and offset units before crossing into Core. |
| Frontend Analysis Core | Validate and retain browser-independent source anchors. |
| Analysis Results | Retain approved anchors as provenance when later contracts authorize result models. |
| Presentation | Render or explain approved result data without redefining source-anchor semantics. |

## Accepted responsibilities

- Retain exact source text and validate byte ranges.
- Return a typed boundary error.
- Attach an anchor to a future approved analysis result.
- Convert adapter-specific UTF-16 or protocol offsets before calling Core.
- Use `fragment()` for exact retained source context.

## Rejected responsibilities

- Adding CDP, WebKit, Firefox, frame, realm, target, or browser-session types to
  Core, or passing browser-protocol offset units directly to `SourceText::anchor`.
- Storing parser tokens or AST nodes in this domain.
- Adding line/column indexing, source maps, file or URL loading, or a source
  registry.
- Generating global IDs in Core or treating content hashes as implicit
  source-instance identity.
- Adding serialization derives, dependencies for convenience, or speculative
  shared-thread ownership, locks, async, threads, or channels.
- Exposing private storage or reference counts.

## Consumer flow

```text
Exact UTF-8 source acquisition
    ↓
Caller assigns SourceId
    ↓
SourceText owns exact String
    ↓
Caller or approved boundary requests [start, end)
    ↓
Core validates ordering, bounds, and UTF-8 boundaries
    ↓
SourceAnchor retains source identity, range, and exact fragment access
```

A Browser Adapter or parser must normalize its own offset system before
requesting a Core anchor. Core performs no hidden offset conversion.

## Minimal Rust example

This workspace example demonstrates the current public API; it does not imply
crates.io availability or a stable external SDK.

```rust
use frontend_analysis_core::{SourceId, SourceText};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_id = SourceId::new(7);
    let source = SourceText::new(source_id, String::from("aéz"));
    let anchor = source.anchor(1, 3)?;

    assert_eq!(anchor.source_id(), source_id);
    assert_eq!(anchor.range().start(), 1);
    assert_eq!(anchor.range().end(), 3);
    assert_eq!(anchor.fragment(), "é");
    Ok(())
}
```

## Error example

Match typed variants and fields, not `Display` wording:

```rust
use frontend_analysis_core::{SourceId, SourceRangeError, SourceText};

let source = SourceText::new(SourceId::new(7), String::from("aéz"));
match source.anchor(2, 2) {
    Err(SourceRangeError::InvalidStartBoundary { start: 2, end: 2 }) => {}
    other => panic!("unexpected result: {other:?}"),
}
```

## Change-review triggers

A focused Issue and architecture review are required before adding new
Core-owned source concepts, another crate, another public type or method,
offset-unit conversion, line indexes, serialization, dependencies,
concurrency, `Send` or `Sync` promises, parsers, Browser Adapter integration,
source registries, or file or URL loading.

## Validation

Follow [Contributing](../../.github/CONTRIBUTING.md) and [Validation and
Completion Evidence](../development/VALIDATION.md). Current production checks
are:

```bash
python3 .github/scripts/validate-rust-workspace-state.py .
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo metadata --offline --format-version 1 --locked
```
