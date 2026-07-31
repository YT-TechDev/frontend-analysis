# Raw Source Coordinates Guide

## Purpose and authority

This guide is explanatory and non-normative. [ADR 0005](../decisions/0005-raw-source-coordinate-semantics.md)
is the normative authority for raw source-coordinate semantics; [ADR 0004](../decisions/0004-validated-source-anchor-semantics.md)
owns source-anchor validation and retention. This guide also follows the
[Validated Source Anchors Guide](VALIDATED_SOURCE_ANCHORS.md),
[Architecture Layers](LAYERS.md), and [Rust Core Contracts](RUST_CORE_CONTRACTS.md).

## Current domain responsibility

`frontend-analysis-core` owns Validated Source Anchors and Raw Source Line
Coordinates. The latter is a browser-independent, grammar-neutral projection
of an already validated endpoint, not a general position-conversion service.

## Relationship to Validated Source Anchors

`SourceAnchor` remains authoritative retained source evidence.
`RawSourceCoordinate` is a detached derived projection created only from a
validated anchor's start or end endpoint. Its `byte_offset` remains
authoritative; derived line and byte-column values neither retain nor replace
the exact source evidence.

## Public surface

The implemented surface is `RawSourceCoordinate` with
`RawSourceCoordinate::source_id`, `RawSourceCoordinate::byte_offset`,
`RawSourceCoordinate::line_index`, and `RawSourceCoordinate::byte_column`, plus
`SourceAnchor::start_coordinate` and `SourceAnchor::end_coordinate`. There is
no public coordinate constructor or arbitrary-offset convenience API.

## Authoritative units

Byte offsets are authoritative. Line indexes are zero-based. Byte columns are
zero-based UTF-8 byte distances from the current raw line start. They are not
Unicode-scalar, grapheme, UTF-16, display-cell, parser, browser-protocol,
editor, or UI positions.

## Raw newline behavior

Core recognizes LF, lone CR, and CRLF. CRLF is one complete sequence: the
boundary between CR and LF remains on the preceding raw line, and the
transition occurs after LF. FF, VT, NEL, U+2028, and U+2029 remain ordinary
raw content. See [ADR 0005](../decisions/0005-raw-source-coordinate-semantics.md)
for the complete normative tables.

## CRLF interior example

For the source `"\r\n"`:

| Byte offset | Line index | Byte column |
| ---: | ---: | ---: |
| 0 | 0 | 0 |
| 1 | 0 | 1 |
| 2 | 1 | 0 |

## UTF-8 byte-column example

For `"éあ😀"`, valid offsets `0`, `2`, `5`, and `9` have byte columns `0`,
`2`, `5`, and `9`. No Unicode normalization occurs, and these values do not
count characters, graphemes, UTF-16 code units, or display cells.

## Provenance and lifecycle

A coordinate copies its caller-supplied `SourceId` and numeric evidence. It may
outlive source and anchor handles, but it retains no source text and does not
establish globally unique identity. Equal text under different `SourceId`
values has different provenance.

## Ownership by layer

The future-layer entries describe boundaries, not claims that implementations
currently exist.

| Layer or owner | Responsibility |
| --- | --- |
| Source acquisition | Assign `SourceId` and provide exact UTF-8 text. |
| Frontend Analysis Core | Validate and retain anchors, then derive raw coordinates from validated endpoints. |
| Parser | Own grammar-specific positions and explicit conversion to authoritative offsets. |
| Browser Adapter | Own conversion from protocol-specific position units. |
| Analysis Results | Retain `SourceAnchor` when a future approved result requires exact evidence. |
| Presentation | Own one-based, display-oriented, or other presentation conversion. |

## Accepted responsibilities

- Project validated start and end endpoints deterministically.
- Preserve source identity and authoritative UTF-8 byte offsets.
- Derive the accepted raw line index and UTF-8 byte column.
- Return detached copied evidence while anchors retain exact source evidence.

## Rejected responsibilities

- `SourceText::coordinate`, public coordinate constructors, or arbitrary
  unvalidated offsets.
- Reverse line/column-to-offset conversion, a retained `SourceLineIndex`, or
  observable caches.
- Parser, browser-protocol, editor, or UI position semantics.
- UTF-16, grapheme, Unicode-scalar, or display-cell columns.
- Source maps, diagnostics, serialization, or performance guarantees.
- crates.io availability, release, or external compatibility claims.

## Minimal workspace example

This example uses the workspace package; it does not imply crates.io
publication or a stable external SDK.

```rust
use frontend_analysis_core::{SourceId, SourceText};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = SourceText::new(SourceId::new(7), "a\r\nb".to_owned());
    let anchor = source.anchor(1, 3)?;
    let start = anchor.start_coordinate();
    let end = anchor.end_coordinate();

    assert_eq!((start.byte_offset(), start.line_index(), start.byte_column()), (1, 0, 1));
    assert_eq!((end.byte_offset(), end.line_index(), end.byte_column()), (3, 1, 0));
    Ok(())
}
```

## Debug and source-content safety

Source text may contain secrets. Coordinate `Debug` provides structural
identity and numeric evidence only; contributors must not add source text or
selected fragments to `Debug`, errors, logs, or generated evidence.

## Compatibility and non-guarantees

The current private package creates no crates.io, stable external SDK, SemVer,
MSRV, ABI, serialization, `Send`, `Sync`, allocation, or performance promise.
Raw coordinates do not imply compatibility with parser, protocol, editor, or
presentation coordinate systems.

## Change-review triggers

A focused Issue and applicable approval are required before adding a public
constructor or API, arbitrary-offset projection, reverse mapping, retained
index, observable cache, another position unit, serialization, dependency,
performance contract, or layer conversion. Follow the owning ADR and
architecture contracts rather than extending this guide as authority.

## Validation

Follow [Contributing](../../.github/CONTRIBUTING.md) and [Validation and
Completion Evidence](../development/VALIDATION.md). Public-contract changes
require focused tests through crate-root re-exports and the applicable Rust
baseline, including rustdoc with warnings denied for public documentation work.
