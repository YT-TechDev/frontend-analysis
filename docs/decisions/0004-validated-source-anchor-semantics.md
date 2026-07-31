# ADR 0004: Define Validated Source Anchor Semantics

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-07-31 |
| Decision owner / approver | YT-TechDev |
| Linked Issue | [#54](https://github.com/YT-TechDev/frontend-analysis/issues/54) |
| Related Pull Request | [#60](https://github.com/YT-TechDev/frontend-analysis/pull/60) |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | None — this record specializes the source-anchor domain selected by ADR 0003 within the existing architecture, Rust Core, security, and validation contracts; it changes none of those contracts. |

## Context

Accepted [ADR 0003](0003-validated-source-anchors-first-rust-core-domain.md)
selects Validated Source Anchors as the first Rust Core domain and assigns it
browser-independent source identity, immutable source ownership, validated byte
ranges, owned anchors, deterministic validation, and domain errors. It
deliberately leaves their exact semantics to this decision. Issue #55 cannot
implement the domain without resolving those choices first.

The input may originate outside Core and must be treated as potentially
untrusted. At the same time, an anchor may outlive the caller's temporary input,
and multiple anchors may refer to one source without each retaining a complete
copy. The milestone excludes dependencies, global state, concurrency, async,
serialization, parsers, browser protocols, source maps, and product behavior.

This decision is one tightly coupled semantic package: the ownership choice
determines lifetime and auto-trait behavior; byte-range invariants determine
safe fragment access; identity determines provenance; and deterministic typed
errors make invalid external input ordinary domain behavior. It constrains the
future Issue #55 implementation without prescribing complete Rust source or
final method signatures.

No normative contract update is required. The package applies the existing
[Architecture Principles](../architecture/PRINCIPLES.md),
[Architecture Layers](../architecture/LAYERS.md),
[Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md),
[Secure Development](../development/SECURE_DEVELOPMENT.md), and
[Validation](../development/VALIDATION.md) requirements to the first domain. It
does not change a layer, dependency direction, security exception, validation
rule, or general Rust policy. If maintainers revise this decision so that it
conflicts with one of those contracts, review must stop and the owning
normative contract must be addressed explicitly.

## Decision

ADR 0004 is accepted and the following package is the complete semantic
authority for Issue #55. Implementation may begin only after the Pull Request
accepting this ADR is merged, and remains constrained by ADR 0003, ADR 0004,
and Issue #55. Acceptance authorizes no work outside the approved milestone.

### Domain model and source input

The minimal coherent domain consists conceptually of:

- `SourceId`, an opaque caller-supplied source-instance identifier;
- one Core-owned immutable source containing exact Rust UTF-8 string data;
- a validated half-open source byte range;
- `SourceAnchor`, retaining the source and its validated range; and
- a typed source-range validation error.

The input domain is valid Rust UTF-8 text, not raw unvalidated bytes. Invalid
UTF-8 is therefore outside this API's domain. Once Core establishes ownership,
the source is immutable and its original UTF-8 bytes are authoritative. Core
performs no Unicode or newline normalization, BOM removal, trimming, encoding
conversion, rewriting, or canonicalization. Every offset and fragment refers
to that exact byte sequence.

### Source identity and provenance

`SourceId` is a browser-independent opaque value supplied explicitly by the
caller. A minimal standard-library integer newtype is the intended conceptual
representation, but its field and concrete integer representation remain
private or no wider than current workspace consumers require. Exact value
equality defines identifier equality.

Core does not derive identity from global state, a counter, randomness,
wall-clock time, source content, or a content hash. `SourceId` has no required
file-system, URL, browser, frame, realm, protocol, or execution-context meaning.
It is neither globally unique nor promised to persist across processes or
runs. Callers must supply distinct values whenever distinct source instances
must remain distinguishable in their analysis scope. No UUID, URL, path,
hashing, parser, or browser dependency and no serialized representation is
introduced.

Source-instance identity is the `SourceId` value. Source-content equality asks
whether complete UTF-8 byte sequences match and does not establish instance
identity. Range equality asks whether validated start and end offsets match.
Semantic anchor identity is source-instance identity plus validated range.
Fragment-text equality only compares selected bytes: identical fragments at
different ranges are not the same anchor, and identical complete text under
different identifiers is not the same source instance.

### Source ownership and shared storage

One immutable Core-owned source value owns the exact text. Each anchor retains
shared ownership of that same backing value, so it remains usable after the
caller's temporary input is dropped. The selected implementation posture is a
private standard-library, single-threaded shared-ownership mechanism. This is
a demonstrated ownership graph—one immutable source retained by independently
lived anchors—not shared mutation or preparation for speculative concurrency.

Creating or cloning an anchor must not duplicate the complete source text.
Cloning an anchor shares the immutable backing source and copies only its small
metadata. The storage type, allocation, reference count, and pointer identity
remain private; mutation is impossible through the workspace-facing contract.
The shared mechanism must not appear in public signatures unless implementation
review proves that unavoidable and maintainers explicitly approve it.

This selection deliberately creates no `Send` or `Sync` guarantee. Issue #55
must review actual auto traits and must not add an atomic primitive, an unsafe
implementation, or wrappers that imply thread safety. A future concurrency
requirement needs focused ownership and compatibility review; changing the
backing mechanism may change auto traits even if domain values otherwise look
unchanged.

Global registries, global mutable state, atomics, locks, channels, threads,
tasks, async runtimes, and background ownership services are not authorized.

### Byte range and empty range semantics

A source range is `[start, end)`, where `start` and `end` are UTF-8 byte
offsets, `start` is inclusive, and `end` is exclusive. Byte offsets are
authoritative for this milestone. A valid range satisfies all of:

1. `start <= end`;
2. `end <= source.len()` measured in bytes; and
3. both offsets are UTF-8 character boundaries.

Once the first two conditions hold, `start` is necessarily within the source.
No line or column is stored or calculated. Unicode scalar, grapheme-cluster,
UTF-16, parser-token, and browser-protocol offsets are not implicitly accepted
or converted. Validation never clamps, reorders, expands, shrinks, or
normalizes a request.

Empty ranges are valid when `start == end`, the shared offset is in
`0..=source.len()`, and it is a UTF-8 character boundary. Consequently, empty
ranges at zero and at the byte length are valid; an empty range inside a
multi-byte character is invalid. Its fragment is the empty string, not missing
data or an error. This represents insertion points and other zero-width source
locations without assigning parser meaning.

### Deterministic validation and errors

Validation uses this stable precedence; the first failing step is the only
returned category:

1. if `start > end`, return **reversed range**;
2. otherwise, if `end > source.len()`, return **outside source bounds**;
3. otherwise, if `start` is not a UTF-8 character boundary, return
   **invalid start boundary**;
4. otherwise, if `end` is not a UTF-8 character boundary, return
   **invalid end boundary**;
5. otherwise, accept.

Thus reversal wins even if an offset is also out of bounds; bounds wins over
either boundary defect; and an invalid start wins over an invalid end. An
ordered, bounded request needs no separate start-bounds test because
`start <= end <= source.len()`.

The domain error has four stable semantic categories corresponding to those
steps. Every category retains the requested `start` and `end`. The bounds
category also retains the source byte length. Boundary categories retain the
offending offset through the requested offsets; they need not embed source
text. Equality means the same category and exact equality of every retained
numeric field. Display text is diagnostic but is not itself the equality or a
stable machine-readable contract.

Errors are returned, never used to silently normalize a request, and never
embed the complete source. The error requires `Debug`, `Display`,
`std::error::Error`, `Clone`, `PartialEq`, and `Eq`. `Copy` is permitted only if
Issue #55 selects exclusively small, owned numeric fields and review confirms
that copying is semantically honest. No third-party error library is allowed.

### Source fragment semantics

A valid anchor identifies exactly the substring at its retained `[start, end)`.
Fragment access borrows that slice from the anchor's retained immutable source
without an additional allocation where the selected storage permits. It never
uses caller-supplied replacement text and performs no normalization or
rewriting. Repeated access returns the same bytes, and an empty range returns
an empty string.

Line extraction, context lines, highlighting, parser nodes, and source-map
semantics are outside this decision.

### Worked semantic examples

These examples are declarative outcomes, not Rust signatures.

For ASCII source `abc`, the byte length is 3 and every offset from 0 through 3
is a UTF-8 boundary:

| Request | Outcome |
| --- | --- |
| `[0, 3)` | valid; fragment `abc` |
| `[1, 2)` | valid; fragment `b` |
| `[0, 0)` | valid empty fragment at the beginning |
| `[3, 3)` | valid empty fragment at the end |
| `[2, 1)` | reversed-range error, retaining 2 and 1 |
| `[0, 4)` | outside-bounds error, retaining 0, 4, and source length 3 |

For source `aéz`, the UTF-8 bytes are `61 C3 A9 7A`, the byte length is 4,
and character boundaries are 0, 1, 3, and 4:

| Request | Outcome |
| --- | --- |
| `[1, 3)` | valid; fragment `é`, using boundaries before and after it |
| `[3, 4)` | valid; fragment `z` |
| `[2, 3)` | invalid-start-boundary error because byte 2 is inside `é` |
| `[1, 2)` | invalid-end-boundary error because byte 2 is inside `é` |
| `[3, 3)` | valid empty fragment on a boundary |
| `[2, 2)` | invalid-start-boundary error; the empty offset is inside `é` |

The last example follows precedence: for the same invalid offset, start
boundary validation occurs before end boundary validation.

### Visibility policy

Issue #55 may expose to current workspace consumers only the concepts needed to
construct or receive a Core-owned source, identify it with `SourceId`, request
validation, inspect a validated range, retain or clone a `SourceAnchor`, obtain
its source identity and range, borrow its fragment, and distinguish typed
validation failures. These are workspace-facing domain concepts and read-only
operations, not an external SDK commitment.

Fields, the shared-storage representation, source allocation form, validation
helpers, reference counts, pointer identity, and unchecked construction paths
remain private. Tests must use approved behavior rather than wider visibility.
The implementation review must choose the narrowest useful item visibility and
need not make every conceptual value a separately public Rust type. This ADR
does not promise crates.io publication or a stable external Rust API.

### Trait policy

The smallest accepted trait surface is:

| Concept | Required | Deliberately absent or deferred |
| --- | --- | --- |
| `SourceId` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` | `Hash`, `PartialOrd`, `Ord`, `Default`, `Display`, `Error`; `Send`/`Sync` are not promised |
| validated range | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` | `Hash`, `PartialOrd`, `Ord`, `Default`, `Display`, `Error`; `Send`/`Sync` are not promised |
| immutable owned source value, if separately visible | `Debug`, `Clone` | `Copy`, equality, hashing, ordering, `Default`, `Display`, `Error`; `Send`/`Sync` are not promised |
| `SourceAnchor` | `Debug`, `Clone` | `Copy`, `PartialEq`, `Eq`, `Hash`, ordering, `Default`, `Display`, `Error`, `Send`, `Sync` |
| validation error | `Debug`, `Display`, `std::error::Error`, `Clone`, `PartialEq`, `Eq`; `Copy` only under the field condition above | `Hash`, ordering, `Default`; `Send`/`Sync` are not promised |

Hashing and ordering are absent because no current collection use or semantic
order of source instances or ranges is required. Defaults are absent because
fabricating an identifier, source, range, anchor, or error would hide required
input or validation. Full-anchor equality remains absent even though its
semantic identity is defined: no current consumer requires the trait, and
deriving it could accidentally compare backing content, allocation identity,
or representation rather than the stated identity. A future requirement can
add equality deliberately against that definition.

No serialization traits are introduced. No stable layout or ABI, stable
external Rust API, MSRV, `no_std`, WASM, or FFI contract is promised. Under the
single-threaded shared-storage model, no `Send` or `Sync` guarantee is promised.
Auto traits are compatibility-relevant and must be inspected before Issue #55
is approved; accidental auto-trait exposure must not be treated as a durable
promise.

### Determinism, panic, and allocation posture

For identical `SourceId`, exact source UTF-8 bytes, requested `start`, and
requested `end`, validation produces the same successful semantic result or
the same error category and fields. Results do not depend on global counters,
randomness, wall-clock time, scheduling, hash-map iteration, pointer or
allocation addresses, browser state, file-system state, locale, or platform
newline conventions.

Externally influenced invalid ranges are domain errors and must not cause
ordinary Core panics. Panic remains reserved for a programmer error or an
impossible internal invariant violation under the Rust Core contract; this ADR
creates no broader panic guarantee. Allocation failure retains Rust and the
operating environment's behavior and is not converted into a range-validation
category.

No premature performance SLA is established. Establishing ownership performs
one deliberate source allocation or ownership transfer. Anchor creation does
not duplicate the complete source. Validation uses bounded standard-library
length and boundary operations, and fragment access should borrow from retained
storage. No cache, index, line table, source map, registry, or benchmark
framework is introduced.

## Alternatives Considered

### Borrowed anchors tied to caller lifetimes

Borrowing avoids shared allocation and reference-count metadata and may expose
useful lifetime relationships. It is deterministic and cheap, but makes every
anchor and downstream result depend on caller storage, prevents the required
temporary-input independence, and spreads lifetime compatibility through
consumers. Moving later to ownership would change APIs. It is not selected;
explicit immutable ownership is more reversible for downstream lifetimes.

### Complete source copy per anchor

Per-anchor ownership is simple, independent, and can naturally inherit
thread-safe auto traits. It is deterministic and avoids reference counts, but
memory grows by the entire source for every anchor and cloning repeats that
cost. It violates the milestone requirement. Shared immutable backing is
selected; reverting to copies would be mechanically possible but would change
documented allocation and clone behavior.

### Copy only the selected fragment per anchor

Fragment-only ownership reduces each anchor relative to a full-source copy and
has simple lifetimes. It loses authoritative source context, makes equal text
at different positions easier to conflate, cannot derive the fragment from the
retained original, and allocates on anchor creation. Preserving provenance
would require extra storage or a registry. It is deterministic but semantically
lossy and is not selected. Adding an explicit detached-fragment domain later
would be a separate reversible capability rather than changing anchors.

### Shared immutable source backing

The selected model satisfies lifetime independence, shares one source
allocation among anchors, makes clones small, and preserves exact fragment and
provenance semantics. Its costs are reference-count metadata, nontrivial
lifecycle review, and auto-trait compatibility. Immutability avoids locking and
invalidation. The private representation allows a future approved storage
change, but consumers could still observe auto-trait changes, so reversal needs
compatibility review.

### Single-threaded versus atomic shared ownership

Single-threaded shared ownership has non-atomic reference-count operations,
matches the milestone's explicit lack of concurrency, and avoids creating a
`Send`/`Sync` expectation. It uses less synchronization machinery but prevents
moving anchors across threads under the selected representation. Atomic shared
ownership could enable cross-thread retention and may simplify a future
concurrent consumer, at the cost of atomic operations and an immediate
compatibility commitment without a current owner, scheduling model, or need.
Both are deterministic for immutable content and have similar shared-source
memory shape. The single-threaded option is selected; moving to atomic sharing
is feasible but requires a focused concurrency and auto-trait decision.

### Global source registry

A registry could centralize deduplication and let anchors store keys. It adds
global lifecycle, cleanup, collision, synchronization, memory-retention,
testing, and determinism problems; a single-threaded registry would still add
global mutable state, while a concurrent registry adds locks or atomics.
Pointer or allocation identity would be especially unsuitable. It is rejected.
An explicit analysis-session owner could be considered later only if a real
multi-source lifecycle requires it.

### Caller-supplied versus globally generated identity

Caller-supplied identity is deterministic, carries no hidden state, and lets
the boundary that knows source instances distinguish them. Its cost is caller
discipline and lack of global uniqueness. Global counters, randomness, or time
could reduce caller bookkeeping but add process-wide lifecycle, nondeterminism,
or persistence implications and memory/state machinery. Caller supply is
selected. A future scoped identity allocator can remain outside this domain.

### Caller-supplied versus content-hash identity

A content hash can deduplicate identical bytes and be reproducible if its
algorithm is fixed. It costs hashing work, collision and algorithm/version
policy, potentially a dependency, and conflates distinct source instances with
identical content. It also creates a serialized-looking compatibility surface.
Caller-supplied identity is selected. Content hashes may later exist as
separate content metadata but must not silently replace instance identity.

### Half-open versus inclusive ranges

Half-open ranges compose with Rust string slicing, represent empty locations,
use source length as the natural final boundary, and compute byte length as
`end - start` without overflow adjustment. Inclusive ranges can feel natural
for highlighting but require special empty representation and careful final
index handling; conversion risks overflow and off-by-one errors. Both can be
deterministic and store two offsets. Half-open ranges are selected. A future
adapter must convert other units explicitly at its boundary.

### Bytes versus other offset units

UTF-8 bytes match the authoritative Rust string representation and permit
allocation-free slicing after boundary validation. Unicode scalar indices
require traversal; grapheme indices require a segmentation policy and likely a
dependency; line/column needs newline and indexing policy; UTF-16 matches some
protocols but not Rust storage. All alternatives add conversion cost, possible
indexes, and compatibility semantics. Bytes are deterministic without locale
or normalization and are selected. Explicit adapters may later convert a
known external unit without changing Core's authority.

### Allowing versus rejecting valid empty ranges

Allowing empty ranges naturally models zero-width locations, costs no extra
storage, and follows half-open range conventions. Rejecting them could simplify
consumers that assume visible text but would discard legitimate insertion
points and require a separate location type. Both policies are deterministic;
allowing them is more general and reversible because consumers can reject an
empty fragment for their own narrower domain. Valid empty ranges are selected.

### Typed errors versus panic, clamping, or normalization

Typed errors preserve failed values, permit exact deterministic assertions,
and keep untrusted invalid input out of panic control flow. They add an enum and
matching surface but no source-text allocation. Panic violates the Core
boundary; clamping, reordering, or normalization silently changes evidence and
can make malformed requests appear valid. String-only errors lose categories.
Typed errors are selected. Categories can be extended only with compatibility
review; error prose may evolve without redefining equality.

### Broad derives versus a minimal trait surface

Broad derivation is convenient for tests and collections but can accidentally
promise ordering, defaults, hashing, equality over private storage, thread
safety, or serialization. It adds little runtime memory, yet creates durable
source-compatibility constraints and can expose implementation details.
Minimal explicit traits require consumers to justify future additions and keep
storage changes more reversible. The minimal table above is selected.

## Consequences

### Positive

- Anchors retain exact, immutable source bytes independently of caller
  lifetimes without one full-source copy per anchor.
- Source-instance provenance, range identity, empty locations, fragments, and
  invalid requests have explicit browser-independent meaning.
- Stable validation precedence makes malformed input deterministic and
  testable without panics or normalization.
- Private single-threaded storage and a narrow trait surface avoid speculative
  concurrency, dependencies, serialization, and external API promises.

### Negative

- Anchors are not promised transferable or shareable across threads.
- Callers must allocate and manage `SourceId` values in their own analysis
  scope and prevent unintended identity reuse.
- Byte-offset callers must explicitly convert UTF-16, line/column, grapheme, or
  protocol offsets and handle conversion failures outside this contract.
- Shared ownership adds reference-count and lifecycle machinery even though
  source data is immutable.

### Risks

Callers could reuse an identifier for semantically distinct sources, so the
contract makes their scope responsibility explicit. Consumers could mistake
byte offsets for character or UTF-16 offsets; named domain concepts,
boundary validation, and tests mitigate that risk. Private storage could still
leak accidental auto traits or overly broad visibility during implementation;
Issue #55 must inspect the exposed surface. Display output could disclose
source text if implemented carelessly, so errors retain numeric diagnostics
only.

### Reversibility

Before Issue #55, revision has documentation cost only. After consumers exist,
range units, identity, empty-range policy, validation precedence, and error
categories are semantic compatibility boundaries and require a new approved
decision plus migration analysis. Private storage makes non-semantic ownership
changes more reversible, but `Send`/`Sync` changes remain compatibility-relevant.
New optional conversion or indexing layers can be added without changing byte
authority if they remain explicit.

## Compatibility and Migration

No production crate, Rust API, serialized data, protocol, product, or adapter
consumer exists to migrate in this decision. Issue #55 establishes
the initial workspace-facing semantics. There is no implicit conversion from a
browser protocol offset; adapters must identify units and normalize them before
Core validation.

The exact validation order and numeric error fields are deterministic semantic
contracts. Error `Display` wording is not a serialized or equality contract.
No serialized representation, stable ABI or layout, stable external Rust API,
external SemVer promise, MSRV, `no_std`, WASM, FFI, or cross-thread contract is
created. Auto traits, public fields and variants, constructors, re-exports, and
generic bounds must be reviewed as compatibility surfaces in Issue #55.

## Security and License Impact

Source text and offsets may be untrusted. Ordered bounds and UTF-8-boundary
validation must precede slicing so invalid external requests return typed
errors rather than panicking. Errors retain numeric offsets and source length
where needed but not the complete source, reducing accidental disclosure; this
does not establish a general logging policy. Immutable storage exposes no
mutation path and uses no unsafe code, global state, thread, lock, network, file
system, or background service.

This documentation-only decision introduces no dependency and therefore no new
supply-chain or third-party license impact. Future repository-authored code
remains under the MIT License. [Secure Development](../development/SECURE_DEVELOPMENT.md)
continues to govern untrusted input and sensitive output.

## Validation

This decision must be validated by:

- conformance with the current ADR template, Accepted status, index entry, and
  approval lifecycle language;
- resolution of every relative documentation link;
- independent byte-length and UTF-8 boundary checks for the ASCII and
  multi-byte examples;
- Markdown heading, table, whitespace, and final-newline inspection;
- `git diff --check`, repository-applicable documentation checks, and
  metadata-only workspace validation without generated-file modification;
- final changed-file and diff review proving that only this ADR and its index
  change; and
- explicit confirmation that no Rust, crate, Cargo manifest, lockfile,
  dependency, workspace, lint, toolchain, CI, or repository-setting change is
  included.

Issue #55 must add behavioral tests for every valid and invalid
example, error field and precedence combination, exact fragments, source
lifetime independence, shared clone behavior without complete-source
duplication, identity distinctions, deterministic repetition, and the reviewed
trait and visibility surface. It must run the Rust baseline required by
[Validation](../development/VALIDATION.md). Tests and implementation do not
authorize broader semantics.

## Follow-Up

- ADR 0004 is accepted. Issue #55 may begin after the Pull Request accepting
  this ADR is merged and may implement only the accepted domain.
- Issue #55 must review actual visibility and auto-trait consequences before
  exposing them.
- Concurrency, serialization, alternate offsets, parser work, adapters, broader
  public APIs, and other deferred capabilities require separate approval.
- Final milestone completion remains subject to independent audit
  [Issue #58](https://github.com/YT-TechDev/frontend-analysis/issues/58).

## Approval

Approved by `YT-TechDev`, the current maintainer of record, on 2026-07-31.

Durable approval:

- [Issue #54 maintainer architecture decision](https://github.com/YT-TechDev/frontend-analysis/issues/54#issuecomment-5138469533)

The approval accepts the complete semantic package recorded by ADR 0004,
including source provenance, immutable shared ownership, UTF-8 byte-range
semantics, empty ranges, deterministic validation precedence, typed errors,
fragment semantics, identity, visibility, traits, compatibility boundaries,
and explicit non-goals.

The approval does not authorize work beyond Issue #55 or Milestone #3. Any
substantive change to these semantics requires renewed architecture review and,
where applicable, a new ADR.

## References

- [Issue #54: define exact semantics for Validated Source Anchors](https://github.com/YT-TechDev/frontend-analysis/issues/54)
- [Parent Issue #52](https://github.com/YT-TechDev/frontend-analysis/issues/52)
- [Prerequisite Issue #53](https://github.com/YT-TechDev/frontend-analysis/issues/53)
- [Implementation Issue #55](https://github.com/YT-TechDev/frontend-analysis/issues/55)
- [Proposal Pull Request #60](https://github.com/YT-TechDev/frontend-analysis/pull/60)
- [Issue #54 maintainer architecture decision](https://github.com/YT-TechDev/frontend-analysis/issues/54#issuecomment-5138469533)
- [Merged prerequisite Pull Request #59](https://github.com/YT-TechDev/frontend-analysis/pull/59)
- [Milestone 3](https://github.com/YT-TechDev/frontend-analysis/milestone/3)
- [ADR 0001](0001-repository-topology-and-workspace-ownership.md)
- [ADR 0002](0002-rust-bootstrap-toolchain-and-validation-policy.md)
- [ADR 0003](0003-validated-source-anchors-first-rust-core-domain.md)
- [Documentation Index](../README.md)
- [Maintainership](../governance/MAINTAINERSHIP.md)
- [Architecture Principles](../architecture/PRINCIPLES.md)
- [Architecture Layers and Boundaries](../architecture/LAYERS.md)
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md)
- [Secure Development](../development/SECURE_DEVELOPMENT.md)
- [Validation and Completion Evidence](../development/VALIDATION.md)
- [ADR Process](README.md)
