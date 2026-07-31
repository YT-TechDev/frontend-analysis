# ADR 0005: Raw Source Coordinate Semantics

| Field | Value |
| --- | --- |
| Status | Proposed |
| Date | 2026-07-31 |
| Decision owner / approver | Awaiting explicit review by `YT-TechDev`; no maintainer approval is claimed |
| Linked Issue | [#70](https://github.com/YT-TechDev/frontend-analysis/issues/70) |
| Related Pull Request | None at proposal creation |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | None proposed — this record specializes the existing Validated Source Anchors domain within the current architecture, Rust Core, security, and validation contracts; it does not change those contracts. If review identifies a conflict, implementation must stop and the owning normative contract must be addressed separately. |

## Context

Accepted [ADR 0003](0003-validated-source-anchors-first-rust-core-domain.md)
and [ADR 0004](0004-validated-source-anchor-semantics.md) establish exact,
immutable UTF-8 source retention and validated half-open byte ranges as the
first browser-independent Core domain. They intentionally do not define line
and column coordinates. Consumers nevertheless need a grammar-neutral way to
describe either validated endpoint without importing parser, browser,
protocol, editor, locale, or presentation semantics into Core.

The decision problem is: should Frontend Analysis Core derive a grammar-neutral
raw source coordinate from validated `SourceAnchor` endpoints, and, if
accepted, what exact ownership, newline, unit, lifecycle, public API,
compatibility, and security semantics govern it?

The input evidence already belongs to `SourceAnchor`: the exact immutable UTF-8
source, a validated endpoint, the caller-supplied `SourceId`, and the
endpoint's authoritative UTF-8 byte offset. Derivation needs no parser,
browser, protocol, filesystem, URL, runtime, product, locale, or presentation
state. That makes raw derivation Core-owned under the existing
[Architecture Principles](../architecture/PRINCIPLES.md) and
[layer boundaries](../architecture/LAYERS.md). Parser-specific line
terminators, protocol-specific position encodings, transformed-source and
source-map positions, and presentation columns remain explicit conversions
owned outside Core.

This proposal must preserve exact source evidence, validation precedence,
browser independence, deterministic behavior, source-content safety, the
single-crate dependency boundary, and the absence of an external compatibility
promise. While this ADR remains Proposed, it authorizes no Rust
implementation. Issue [#71](https://github.com/YT-TechDev/frontend-analysis/issues/71)
remains blocked until explicit maintainer approval is recorded and a separate
focused Pull Request changes ADR 0005 and the ADR index to Accepted.

## Decision

If accepted, Core will expose a detached projection named
`RawSourceCoordinate`. The broader name `SourceCoordinate` is neither proposed
nor reserved: occupying it now would prematurely constrain future parser,
protocol, transformed-source, source-map, or presentation coordinate domains.

### Authority, units, and provenance

`RawSourceCoordinate` will be derived exclusively from:

- the exact immutable UTF-8 source retained by a `SourceAnchor`;
- one validated anchor endpoint;
- the source's caller-supplied `SourceId`; and
- that endpoint's authoritative UTF-8 byte offset.

`byte_offset` remains the authoritative source location. `line_index` is a
derived, zero-based raw line index. `byte_column` is a derived, zero-based UTF-8
byte distance from the current raw line start. The coordinate is a projection,
not a replacement for `SourceAnchor`, and line and column are not Unicode
scalar, grapheme, UTF-16, display-cell, parser, editor, or browser-protocol
units.

The value contains only approved copied identity and numeric evidence:
`SourceId`, the exact endpoint `byte_offset`, derived `line_index`, and derived
`byte_column`. It retains neither source text nor a borrow, reference, or
`SourceAnchor`. It can outlive the source handle as a detached numeric
projection, but it does not replace `SourceAnchor` as authoritative retained
source evidence. Future Analysis Results should continue retaining
`SourceAnchor` whenever exact source evidence is required unless a later
accepted contract decides otherwise.

### Proposed public surface

The complete proposed public surface is exactly:

- `RawSourceCoordinate`, because consumers need a named, grammar-neutral value
  that can be returned from an anchor and retained independently;
- `RawSourceCoordinate::source_id`, because a detached coordinate must identify
  the caller-defined source to which its numbers apply;
- `RawSourceCoordinate::byte_offset`, because consumers must access the
  authoritative validated location rather than reconstruct it from derived
  values;
- `RawSourceCoordinate::line_index`, because the proposal's Core-owned raw line
  projection is otherwise inaccessible;
- `RawSourceCoordinate::byte_column`, because the proposal's explicitly
  byte-based raw column is otherwise inaccessible;
- `SourceAnchor::start_coordinate`, because only the anchor can pair retained
  source evidence with its already validated start endpoint without accepting
  a new unvalidated offset; and
- `SourceAnchor::end_coordinate`, for the same reason at the validated end
  endpoint, including valid end-of-source and empty-range positions.

`RawSourceCoordinate` will implement exactly `Debug`, `Clone`, `Copy`,
`PartialEq`, and `Eq`. `Clone` and `Copy` fit a small detached value containing
only copied identity and numeric evidence. `PartialEq` and `Eq` support exact
deterministic comparison of that evidence. `Debug` supports structural
inspection under the source-content restrictions below.

The following are deliberately excluded:

- a public constructor and public fields, because construction must remain tied
  to validated anchor endpoints and private invariants;
- `SourceText::coordinate`, because accepting an arbitrary offset would create
  a second validation entry point and bypass the anchor-owned provenance;
- a `SourceCoordinate` alias, because it would imply or reserve broader
  semantics this proposal does not define;
- a new error type, because projection after validation is infallible;
- unchecked construction, because externally influenced invalid offsets must
  not bypass typed validation;
- `Display`, because there is no approved user-facing or protocol formatting;
- `Hash`, `PartialOrd`, and `Ord`, because no hashing or ordering contract is
  needed and numeric ordering across different `SourceId` values could be
  misleading;
- `Default`, because no source-independent coordinate is meaningful;
- serialization traits, because there is no approved wire or storage format;
- builder APIs, because the value has no independently configurable parts;
- generic conversion traits, because conversions need domain-specific units,
  validation, and ownership; and
- reverse conversion, because a detached projection does not retain source
  evidence and cannot reconstruct or validate a `SourceAnchor`.

No other public item or trait is part of this proposal.

### Raw newline semantics

The raw Core model recognizes exactly LF (`\n`), lone CR (`\r` not followed by
LF), and CRLF (`\r\n`):

- LF is one complete raw line break;
- CR not followed by LF is one complete raw line break;
- CRLF is one complete raw line-break sequence and increments `line_index`
  exactly once;
- the valid UTF-8 byte boundary between CR and LF remains on the preceding raw
  line; and
- transition to the next raw line occurs only after the complete CRLF sequence.

FF, VT, NEL, U+2028, and U+2029 are not raw Core line transitions. Parser or
protocol layers may interpret any of these values differently, but must perform
an explicit conversion under their own contracts.

The following tables exhaust every valid byte boundary in each source and are
normative examples of the proposed transition rules.

#### Empty source

| Source | Byte offset | Line index | Byte column |
| --- | ---: | ---: | ---: |
| `""` | 0 | 0 | 0 |

The sole boundary is the start and end of raw line zero.

#### ASCII without terminator

| Source | Byte offset | Line index | Byte column |
| --- | ---: | ---: | ---: |
| `"a"` | 0 | 0 | 0 |
| `"a"` | 1 | 0 | 1 |

Crossing `a` advances the byte column by one without a line transition.

#### LF

| Source | Byte offset | Line index | Byte column |
| --- | ---: | ---: | ---: |
| `"\n"` | 0 | 0 | 0 |
| `"\n"` | 1 | 1 | 0 |

The boundary after LF has completed one line break.

#### Lone CR

| Source | Byte offset | Line index | Byte column |
| --- | ---: | ---: | ---: |
| `"\r"` | 0 | 0 | 0 |
| `"\r"` | 1 | 1 | 0 |

At end of source the CR is known to be lone, so the final boundary is on the
next line.

#### CRLF

| Source | Byte offset | Line index | Byte column |
| --- | ---: | ---: | ---: |
| `"\r\n"` | 0 | 0 | 0 |
| `"\r\n"` | 1 | 0 | 1 |
| `"\r\n"` | 2 | 1 | 0 |

At offset 1 the complete CRLF sequence has not yet been crossed, so the
boundary between CR and LF remains on line zero at byte column one. Crossing LF
completes the pair and moves offset 2 to line one, column zero.

#### Text around CRLF

| Source | Byte offset | Line index | Byte column |
| --- | ---: | ---: | ---: |
| `"a\r\nb"` | 0 | 0 | 0 |
| `"a\r\nb"` | 1 | 0 | 1 |
| `"a\r\nb"` | 2 | 0 | 2 |
| `"a\r\nb"` | 3 | 1 | 0 |
| `"a\r\nb"` | 4 | 1 | 1 |

Offsets 1 and 2 are respectively before CR and inside the CRLF pair. Neither
has crossed the complete pair. Offset 3 is after the complete pair and begins
line one; crossing `b` advances its byte column to one.

#### Mixed sequence

| Source | Byte offset | Line index | Byte column |
| --- | ---: | ---: | ---: |
| `"\r\r\n\n"` | 0 | 0 | 0 |
| `"\r\r\n\n"` | 1 | 1 | 0 |
| `"\r\r\n\n"` | 2 | 1 | 1 |
| `"\r\r\n\n"` | 3 | 2 | 0 |
| `"\r\r\n\n"` | 4 | 3 | 0 |

The first CR is lone because the next byte is CR, so offset 1 begins line one.
The second CR starts a CRLF pair; offset 2, between that CR and LF, remains on
line one and advances to byte column one. Offset 3 follows the complete CRLF
and begins line two. The final LF independently moves offset 4 to line three.

### UTF-8 semantics

Coordinates are constructed only from already validated UTF-8 character
boundaries. The coordinate domain does not duplicate validation. Multi-byte
characters advance `byte_column` by their UTF-8 byte length: `é` is two UTF-8
bytes, `あ` is three, and `😀` is four. For example, the valid successive
boundaries in `"éあ😀"` have byte columns 0, 2, 5, and 9 on raw line zero.
These examples define byte-column meaning; they do not create a new validation
API.

Derivation performs no Unicode normalization, newline normalization, BOM
removal, trimming, encoding conversion, grapheme calculation, or display-width
calculation. The exact retained source remains unchanged.

### Validation, errors, and panic behavior

Projection from a validated anchor endpoint is infallible and introduces no new
error type. Invalid offsets continue to be rejected before projection through
the existing `SourceText::anchor` and `SourceRangeError` contract, with the
unchanged precedence from ADR 0004:

1. reversed range;
2. out of bounds;
3. invalid start UTF-8 boundary;
4. invalid end UTF-8 boundary; and
5. success.

Externally influenced invalid input must not become an ordinary panic path.
This proposal neither changes nor duplicates ADR 0004 validation semantics.

### Initial implementation posture and determinism

If accepted, the initial implementation will derive coordinates on demand,
with no retained public line index, observable cache, public cache
configuration, or source duplication. It will remain standard-library-only,
synchronous, deterministic, and safe Rust.

The initial implementation should derive coordinates on demand without a
retained index. Allocation strategy, algorithmic complexity, scanning structure,
and future private caching are not public compatibility guarantees.

This does not promise permanent allocation-free behavior and does not prescribe
a loop, iterator, module filename, internal field layout, or future private
cache implementation.

For identical `SourceId`, exact source UTF-8 bytes, and validated endpoint byte
offset, the derived coordinate must be identical. It must not depend on global
state, random values, wall-clock time, locale, platform newline conventions,
filesystem state, network state, browser state, parser state, pointer identity,
allocation addresses, or hash iteration order.

### Debug and source-content safety

`Debug` may expose only approved structural numeric evidence: `SourceId`,
`byte_offset`, `line_index`, and `byte_column`. It must not expose complete
source text, fragments, private pointers, reference counts, backing storage
type, or cache state. No new error or log output may expose source content.

### Crate, dependency, and package-version boundary

If accepted, implementation will extend the existing
`frontend-analysis-core` crate. Exactly one production crate, one workspace
member, and one library target remain; zero third-party dependencies remain.
There will be no new feature, build script, generated code, target, crate, or CI
transition. A private module is sufficient to organize one Core-owned domain;
another crate would create an unjustified ownership and dependency boundary for
a projection inseparable from `SourceAnchor` evidence.

The later implementation Issue
[#71](https://github.com/YT-TechDev/frontend-analysis/issues/71) is planned to
update the private package version from `0.3.0` to `0.4.0` and synchronize the
corresponding root `Cargo.lock` entry. This proposal Pull Request makes neither
change. That future synchronization will not imply crates.io publication,
external release, stable external SemVer, MSRV, serialization compatibility,
ABI stability, or release automation.

## Alternatives Considered

### Selected LF, lone-CR, and CRLF raw-source semantics

**Benefits:** preserves exact source bytes, gives all three common raw newline
forms deterministic treatment, defines the otherwise ambiguous CRLF-interior
boundary, and remains grammar- and protocol-neutral. **Costs:** consumers whose
grammar normalizes source or recognizes more line terminators must convert
explicitly. **Selection rationale:** this is the smallest complete raw-source
model consistent with exact retained evidence and Core ownership.

### Rust `str::lines()` or LF-oriented semantics

Rust [`str::lines()`](https://doc.rust-lang.org/std/primitive.str.html#method.lines)
splits on LF and strips an optional preceding CR. **Benefits:** familiar
standard-library behavior and little custom policy. **Costs:** its yielded-line
abstraction does not define every byte-boundary coordinate or the required
CRLF-interior position, and LF-only scanning would leave lone CR inconsistent.
**Non-selection rationale:** the coordinate contract needs exhaustive boundary
semantics, not an iterator's line-yielding behavior.

### HTML-specific preprocessing semantics

The WHATWG [HTML syntax](https://html.spec.whatwg.org/multipage/syntax.html) and
[parsing](https://html.spec.whatwg.org/multipage/parsing.html) specifications
define HTML input and preprocessing rules. **Benefits:** direct correspondence
for an HTML parser. **Costs:** normalization and parser lifecycle would change
the meaning of raw retained offsets and import grammar ownership into Core.
**Non-selection rationale:** HTML conversion belongs to an HTML parser or
adapter operating under its own contract.

### CSS-specific preprocessing semantics

[CSS Syntax Level 3](https://www.w3.org/TR/css-syntax-3/) defines CSS input
preprocessing and newline concepts. **Benefits:** coordinates could align
directly with CSS tokenization. **Costs:** CSS-specific replacement and newline
rules are not neutral across JavaScript, HTML, or arbitrary source. **Non-selection
rationale:** CSS positions require an explicit CSS-owned conversion.

### ECMAScript `LineTerminator` semantics

The ECMAScript [lexical grammar](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html)
defines grammar-specific `LineTerminator` values. **Benefits:** natural
alignment for JavaScript parsing and diagnostics. **Costs:** U+2028 and U+2029
would become line transitions only because of one grammar, changing raw Core
meaning for other source. **Non-selection rationale:** ECMAScript conversion
belongs to an ECMAScript parser contract.

### LSP or UTF-16 protocol coordinates

LSP 3.17 [`Position`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#position)
defines protocol positions and negotiated character encodings, with UTF-16 as
the historical default. **Benefits:** direct editor-protocol interoperability.
**Costs:** the representation would depend on a protocol contract and position
encoding rather than authoritative UTF-8 bytes. **Non-selection rationale:** an
LSP adapter must convert explicitly and handle its negotiated encoding.

### Unicode scalar, grapheme, or display-cell columns

Unicode [UAX #29](https://www.unicode.org/reports/tr29/) defines text boundary
algorithms relevant to grapheme segmentation. **Benefits:** scalar or grapheme
counts can be more intuitive to users, and display cells can suit terminal
rendering. **Costs:** these units differ from each other, grapheme rules evolve,
display width depends on presentation context, and none preserves direct byte
authority. **Non-selection rationale:** these are explicit presentation or
consumer projections, not one universal Core column.

### Normalized-source coordinates

**Benefits:** downstream parsers could operate on canonical newlines, Unicode,
or stripped prefixes. **Costs:** offsets would refer to transformed bytes,
requiring provenance and reverse mapping and potentially losing exact evidence.
**Non-selection rationale:** transformed source and source maps need a later
separate contract rather than silently redefining raw anchors.

### Retained `SourceLineIndex` or cache

**Benefits:** repeated queries could avoid repeated scans and improve
high-volume performance. **Costs:** retained memory, invalidation and ownership
questions, observable performance expectations, and premature complexity
without workload evidence. **Non-selection rationale:** on-demand derivation is
the smallest initial posture; evidence may justify a later private optimization
or separately approved index contract.

### A new production crate

**Benefits:** a separate package could advertise an isolated coordinate API.
**Costs:** it would add workspace, versioning, dependency-direction, and
maintenance boundaries for behavior that depends directly on the existing
anchor domain. **Non-selection rationale:** a private module in the sole Core
crate provides organization without architecture fragmentation.

### `SourceText::coordinate(offset)` as a direct API

**Benefits:** callers could query any position without first creating an
anchor. **Costs:** it requires duplicate boundary validation, an error contract,
and provenance rules, and permits bypassing endpoint ownership. **Non-selection
rationale:** `SourceAnchor` endpoints already provide the approved validation
and evidence boundary.

### No Core-owned coordinate domain

**Benefits:** Core would expose no additional public API and every consumer
could adopt native parser or protocol units. **Costs:** raw newline and byte
semantics would be duplicated, conversions could silently disagree, and
Analysis Results would lack one grammar-neutral projection of retained
evidence. **Non-selection rationale:** the derivation depends only on existing
Core evidence, so duplicating it outside Core would obscure ownership rather
than preserve a boundary.

## Consequences

### Positive

- One deterministic raw coordinate meaning is tied to validated exact source
  evidence without importing parser or protocol types.
- Authoritative byte offsets and exhaustive CRLF-interior behavior prevent
  silent line/column ambiguity.
- The minimal endpoint-only API reuses existing validation and creates no new
  failure domain.
- Detached copied values avoid source duplication and can be retained without
  extending a source handle's lifetime.
- The existing crate, dependency, synchronous, and safe-Rust boundaries remain
  intact.

### Negative

- Consumers needing editor, parser, Unicode-display, normalized-source, or
  browser positions must implement explicit conversions.
- On-demand derivation may repeatedly scan source prefixes for repeated
  endpoint queries.
- A byte column may not match what a user perceives as a character or display
  column.
- The public workspace-facing type and accessors create future change cost even
  without a stable external SDK promise.

### Risks

- Consumers may mislabel `byte_column` as a Unicode, editor, or protocol column;
  explicit naming, documentation, and contract tests mitigate that risk.
- Incorrect CRLF scanning could transition at the interior boundary or count a
  pair twice; exhaustive boundary tables and implementation tests mitigate it.
- Detached coordinates may be treated as sufficient evidence after source text
  is gone; documentation must preserve `SourceAnchor` as the authoritative
  retained evidence type.
- Repeated on-demand prefix scans are a residual CPU-amplification risk for
  future high-volume consumers. Measurement, rather than this proposal, must
  justify any optimization.
- Debug or future logging could leak sensitive source content; the structural
  numeric-only rule and security validation mitigate that risk.

### Reversibility

While Proposed, the record can be revised or rejected without Rust API or data
migration. After acceptance and implementation, private scanning mechanics can
change while semantics remain stable. Changing units, newline transitions,
provenance, public names, or endpoint ownership would require a new focused
Issue, explicit approval, compatibility analysis, and normally a superseding
ADR. Removing the workspace-facing API would require migration of workspace
callers. Parser/protocol conversion and private caching can be added separately
without replacing this record if they preserve its public semantics and receive
their own required approval.

## Compatibility and Migration

Existing source-anchor API, validation precedence, retained-source behavior,
and fragment behavior remain unchanged. If accepted, the proposed API is
additive and workspace-facing, not a stable external SDK. Existing callers need
no migration, and parser, adapter, product, and presentation consumers are not
silently converted.

For identical approved inputs, equality and accessor results are deterministic.
There is no serialized representation, stable wire format, stable ABI,
crates.io support, external SemVer guarantee, MSRV, WASM guarantee, FFI
guarantee, `no_std` guarantee, `Send` or `Sync` guarantee, parser
interoperability guarantee, browser-protocol compatibility guarantee,
performance SLA, allocation ABI, or reverse conversion. No ordering contract
is created. Any future conversion or compatibility promise requires its own
contract and approval.

## Security and License Impact

Source text may contain secrets. Complete source, fragments, and storage details
must not appear in `Debug`, error, log, or validation output; only approved
identity and numeric structural evidence may be exposed. No new error or log
path is proposed. Malformed or externally influenced ranges remain handled by
the existing typed validation and must not become ordinary panics.

The proposal introduces no `unsafe` Rust, filesystem or network access,
external dependency, or supply-chain expansion. It authorizes no performance
optimization without evidence. Repeated on-demand scans remain the identified
residual CPU-amplification risk for future high-volume consumers. The
[Secure Development](../development/SECURE_DEVELOPMENT.md) and
[Security Policy](../../SECURITY.md) continue to govern security work; active
vulnerability details do not belong in this ADR.

The [MIT License](../../LICENSE) remains unchanged. No dependency-license or
redistribution obligation is added.

## Validation

If accepted, Issue #71 must validate the implementation with focused tests that
cover every table row in this ADR, including empty source, end-of-source,
LF, lone CR, CRLF interior and completion boundaries, text around CRLF, and the
mixed sequence. Tests must also cover multi-byte UTF-8 byte columns for `é`,
`あ`, and `😀`; both start and end endpoints; empty anchors; exact `SourceId`
and byte-offset retention; equality, copying, and deterministic repeated
projection; and the absence of source content in `Debug`.

Tests must demonstrate that FF, VT, NEL, U+2028, and U+2029 do not cause raw
line transitions, projection introduces no error or panic path, and existing
`SourceText::anchor` validation precedence is unchanged. API review must verify
that only the proposed public items and traits exist and excluded constructors,
aliases, conversions, traits, cache configuration, and serialization remain
absent.

Repository validation must run the existing workspace-policy validator,
formatting, Clippy with warnings denied, all workspace tests, locked offline
metadata, rustdoc with warnings denied, and diff checks. Metadata and validator
evidence must continue to show `production`, one package, one workspace member,
zero dependencies, and one library target. Documentation review must verify
links, tables, Proposed status, approval evidence, and unchanged normative
contracts. Passing validation does not accept this ADR or authorize work before
the required approval record.

## Follow-Up

- [#71](https://github.com/YT-TechDev/frontend-analysis/issues/71) is the
  blocked implementation work and may begin only after explicit maintainer
  approval and a separate documentation-only Accepted-status Pull Request.
- [#72](https://github.com/YT-TechDev/frontend-analysis/issues/72) is later
  contract-test and contributor-documentation work.
- [#73](https://github.com/YT-TechDev/frontend-analysis/issues/73) is the
  independent final audit.
- Reverse mapping, parser/protocol conversion, retained indexing, source maps,
  diagnostics, serialization, and performance optimization remain separately
  planned future work.

This proposal authorizes none of these follow-ups beyond the current milestone
contract.

## Approval

Pending.

No maintainer approval is claimed by this Proposed ADR or by merging its
proposal Pull Request. `YT-TechDev` must record an explicit, attributable,
decision-specific approval or rejection on Issue #70. If approved, a separate
documentation-only Pull Request must update this ADR and the ADR index to
Accepted and link the durable approval record before Issue #71 may begin.

## References

- [Issue #69: parent coordination](https://github.com/YT-TechDev/frontend-analysis/issues/69)
- [Issue #70: raw source-coordinate semantics proposal](https://github.com/YT-TechDev/frontend-analysis/issues/70)
- [Issue #71: blocked implementation](https://github.com/YT-TechDev/frontend-analysis/issues/71)
- [Issue #72: later contract tests and contributor documentation](https://github.com/YT-TechDev/frontend-analysis/issues/72)
- [Issue #73: independent final audit](https://github.com/YT-TechDev/frontend-analysis/issues/73)
- [ADR 0003: Validated Source Anchors as the first Rust Core domain](0003-validated-source-anchors-first-rust-core-domain.md)
- [ADR 0004: Validated Source Anchor Semantics](0004-validated-source-anchor-semantics.md)
- [Architecture Principles](../architecture/PRINCIPLES.md)
- [Architecture Layers and Boundaries](../architecture/LAYERS.md)
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md)
- [Validated Source Anchors Guide](../architecture/VALIDATED_SOURCE_ANCHORS.md)
- [Validation and Completion Evidence](../development/VALIDATION.md)
- [Secure Development](../development/SECURE_DEVELOPMENT.md)
- [Maintainership and Decision Authority](../governance/MAINTAINERSHIP.md)
- [Rust `str::lines()`](https://doc.rust-lang.org/std/primitive.str.html#method.lines)
- [WHATWG HTML syntax](https://html.spec.whatwg.org/multipage/syntax.html)
- [WHATWG HTML parsing](https://html.spec.whatwg.org/multipage/parsing.html)
- [CSS Syntax Level 3](https://www.w3.org/TR/css-syntax-3/)
- [ECMAScript lexical grammar](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html)
- [LSP 3.17 `Position`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#position)
- [Unicode Standard Annex #29](https://www.unicode.org/reports/tr29/)
