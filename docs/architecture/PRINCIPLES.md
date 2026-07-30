# Architecture Principles

## Purpose and Authority

This document is the normative source for the durable architecture principles
of Frontend Analysis. It governs project-wide invariants; the normative layer
model and dependency boundaries are defined by
[Architecture Layers and Boundaries](LAYERS.md).

This document does not define crate or module layout, production domain types,
Rust traits, public APIs, serialized formats, protocol or parser libraries,
async runtimes, concurrency primitives, UI frameworks, storage systems, or
release architecture.

Architecture approval remains governed by
[Maintainership and Decision Authority](../governance/MAINTAINERSHIP.md).
Documentation classification, topic ownership, conflict handling, and
supersession remain governed by the [Documentation Index](../README.md).
Security-sensitive implementation requirements remain governed by
[Secure Development](../development/SECURE_DEVELOPMENT.md). These principles
become authoritative only after maintainer review and merge; neither an open
Issue nor an implementation agent approves architecture.

## Project Architecture Goals

Frontend Analysis is a browser-independent analysis platform, not a browser or
a replacement for a browser engine. Its architecture must support:

- browser-independent analysis semantics and reusable Core behavior;
- explicit layer ownership and replaceable browser adapters;
- multiple presentation consumers without coupling Core to any product;
- stable, testable contracts and evidence-driven diagnostics;
- deterministic behavior where practical;
- maintainability over implementation speed;
- incremental, reviewable architecture evolution; and
- deliberate contracts without accidental public API commitments.

These goals do not promise support for a particular browser, engine, product,
or version.

## Browser Independence

Core concepts must be expressed without Chrome DevTools Protocol (CDP), WebKit
Inspector Protocol, Firefox protocol, browser-engine types, or protocol
serialization models. Engine-specific lifecycle, identifiers, event ordering,
and transport details remain adapter-owned unless an explicitly approved,
browser-independent concept normalizes them.

Core behavior must not be selected by browser brand or branch on protocol
names. Interoperability differences may be preserved as evidence, but protocol
models must not become Core models. Adapters may preserve source provenance and
engine-specific evidence when needed only through an explicitly approved,
browser-independent boundary representation. This contract does not design
that representation and does not claim every browser difference can be fully
normalized.

Adding another browser adapter must not require redefining existing Core
semantics without an explicit architecture change.

## Explicit Ownership and Semantic Authority

Every responsibility must have an identified owner for its data, lifecycle,
mutation, invariants, errors, normalization, evidence meaning, diagnostic
meaning, and compatibility impact.

- Browser adapters own protocol interpretation and normalization.
- Core owns browser-independent analysis semantics.
- Analysis Result contracts own the meaning of evidence and diagnostics; Core
  owns their production.
- Presentation adapters own consumer-specific representation.
- Products own user interaction and product lifecycle.

No layer may silently redefine semantics owned by another layer. Ownership
must not be resolved through global mutable state, arbitrary cloning, shared
mutable utility modules, untyped maps, hidden callbacks, framework context, or
transport objects leaking across boundaries. This rule does not prescribe
concrete Rust ownership types; Rust-specific ownership and type constraints are
governed by [Rust Core Contracts](RUST_CORE_CONTRACTS.md).

## Stable Dependency Direction

Source-code dependencies must point toward stable, browser-independent
contracts. Technology-specific and outer layers may depend on the stable
contracts they consume; stable domain layers must not depend on outer
implementations.

Core must not import adapter, browser protocol, browser implementation, UI,
product, presentation, or platform types. In particular, it must not depend on
React, Electron, Tauri, VS Code APIs, DOM UI code, a browser UI, or a concrete
CLI, desktop, web, or editor product. Presentation may consume Analysis Result
contracts but not Core internals. Products may orchestrate approved components
but cannot redefine their semantic contracts. Test infrastructure follows the
same dependency boundaries unless the relevant stable layer owns an explicit
testing seam. Examples never justify a reverse dependency.

Runtime invocation and compile-time dependency are different. A UI request for
re-analysis does not make Core depend on that UI, and displaying a Core result
with React does not make Core depend on React. Runtime calls may travel in both
directions only through explicit application boundaries or stable-side-owned
contracts; runtime control flow does not reverse compile-time ownership.

## Evidence and Diagnostic Integrity

Observations, normalized inputs, evidence, findings, diagnostics, and
presentation are distinct concepts. Source provenance must not be silently
discarded, and uncertainty, loss, or unsupported observations must not be
rewritten as certainty.

Presentation may filter, sort, group, localize, or visually encode results
without changing their domain meaning. Presentation-specific color, icon,
layout, and copy are not Core evidence. Adapters may attach source metadata
through approved neutral contracts but may not alter analysis meaning.
Products must not mutate a result and present it as the original Core result;
derived presentation data must remain distinguishable from source analysis
results. This document defines no production evidence type or diagnostic
severity enumeration.

## Determinism and Reproducibility

Equal normalized inputs and equal approved configuration should produce
equivalent analysis meaning where practical. Nondeterministic behavior must
have explicit ownership and supporting evidence. After normalization, semantic
ordering must not accidentally depend on hash iteration, event-arrival races,
UI timing, or browser transport timing. Timestamps, generated identifiers,
environment data, and runtime scheduling must not silently become semantic
inputs.

Architecture should protect deterministic replay and testability where the
domain permits. It does not require a replay format, clock abstraction, random
number generator, or event store.

## Boundary-Driven Abstraction

An abstraction is justified when it protects a demonstrated variation point,
a stable domain invariant, a replaceable external dependency, a testable
boundary, an ownership boundary, or a compatibility boundary.

Abstraction is not justified solely because another browser may someday exist,
a framework pattern is fashionable, a code generator prefers it, an
implementation agent can generalize it, a future feature might use it, or two
functions look superficially similar.

The conceptual Browser Adapter boundary is required. That requirement does not
authorize a speculative generic adapter hierarchy, plugin system, service
container, or dynamic-dispatch design.

## Evolution and Compatibility

Architecture may evolve through the approved decision process. A proposed
change must consider existing contracts, browser independence, ownership,
dependency direction, applicable public and serialized compatibility,
migration impact, evidence meaning, testability, and maintenance cost.

This document declares neither a stable public API nor a semantic-versioning
promise. It applies only to this repository; no future repository inherits it
unless that repository explicitly adopts it.

## Cross-Cutting Concerns

Logging, tracing, configuration, metrics, clocks and time, identifiers,
filesystem and network access, caching, error reporting, feature flags, and
test helpers receive no permission to bypass layer boundaries. A cross-cutting
capability must either:

1. be a genuinely domain-neutral lower-level facility with no dependency on a
   higher layer; or
2. cross a boundary through an explicit contract owned by the stable consuming
   layer and implemented by an outer layer.

Which approach applies is a focused decision, not selected here. A global
`common`, `shared`, or `utils` module must not accumulate mixed ownership or
create reverse dependencies. Feature flags must not import browser concepts
into Core, invisibly change evidence meaning, create product-specific branches
inside Core, or bypass architecture approval.

## Architecture Decision Tests

A proposed architecture change must answer:

1. What problem and demonstrated variation require the change?
2. Which layer owns the responsibility?
3. Which layer owns the data and lifecycle?
4. Does the change preserve browser independence?
5. Does dependency direction remain toward stable contracts?
6. Does any protocol or presentation type cross into Core?
7. Does the change alter evidence or diagnostic meaning?
8. Does the abstraction protect a real invariant or only anticipated reuse?
9. What compatibility and migration impact exists?
10. What validation demonstrates the boundary?
11. Which authoritative document must change?
12. Has explicit maintainer approval been recorded?

This checklist collects decision evidence; completing it does not approve a
change. Significant architecture decisions matching the triggers in the
[Architecture Decision Record Process](../decisions/README.md) must be recorded
through that process. Neither this checklist nor an ADR approves a decision by
itself.

## Prohibited Shortcuts

The architecture prohibits:

- importing browser protocol, UI, or product types into Core;
- defining Core behavior by browser brand or treating protocol payloads as
  Core domain models;
- allowing presentation to rewrite evidence or diagnostic semantics;
- using shared mutable global state to cross layers;
- using callbacks with undocumented ownership to reverse dependencies;
- placing mixed responsibilities in a generic utility layer;
- adding speculative abstraction hierarchies or framework-driven architecture;
- using feature flags as hidden dependency inversion;
- treating a merged implementation as retroactive architecture approval; and
- treating private AI output as the only architecture rationale.

## Deferred Implementation Decisions

The following remain deliberately undecided: workspace and crate structure,
module structure, Rust traits and ownership types, parser and browser protocol
libraries, async runtime, concurrency model, serialization format, transport
protocol, storage, caching implementation, logging implementation, UI and
desktop frameworks, VS Code integration, WebAssembly compatibility guarantees,
`no_std`, stable public API, and release packaging.

Deferred does not mean unrestricted. Each decision requires focused scope and
explicit approval under the applicable repository contracts before it is
introduced.
