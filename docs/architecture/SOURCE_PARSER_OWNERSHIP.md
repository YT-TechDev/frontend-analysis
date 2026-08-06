# Source Parser Ownership

## Purpose and Authority

This document is the normative source for ownership, dependency, provenance,
and capability rules for project-owned source parsers in Frontend Analysis. It
specializes the project-wide [Architecture Principles](PRINCIPLES.md),
[Architecture Layers](LAYERS.md), and [Rust Core Contracts](RUST_CORE_CONTRACTS.md)
under [ADR 0007](../decisions/0007-own-lossless-source-parsers.md).

It does not define concrete crates, modules, structs, enums, algorithms,
serialization, JavaScript bindings, package publication, or complete language
support. Focused Issues and approved implementation contracts own those details.

## Owned Language Frontends

Frontend Analysis owns purpose-built lossless Rust source parsers in this order:

```text
HTML → CSS → ECMAScript
```

HTML is the first active language domain. CSS begins after the first HTML
parser-to-Core vertical slice records reusable lessons. ECMAScript follows the
HTML and CSS foundations and includes explicitly owned static semantics where
required for validity.

The sequence is an implementation order, not a claim that one language model or
parser framework is shared by all three languages.

## Semantic Authority

Within an explicitly supported capability, a project-owned parser is the
semantic authority for Frontend Analysis source-backed observations it produces.
Authority is bounded by the declared language, standards snapshot, parse mode,
configuration, implemented states, grammar, static semantics, recovery behavior,
and termination contract.

A parser MUST NOT claim support beyond its validated capability. Partial
specification coverage MUST NOT be represented as universal language
conformance.

Browser engines remain authoritative for their own runtime behavior. Runtime
DOM, CSSOM, layout, paint, execution, and protocol observations remain separate
evidence and MUST NOT be relabeled as project-owned source-parser output.

## Source and Provenance Invariants

- Retained raw UTF-8 bytes are authoritative.
- Exact source ranges are zero-based and half-open.
- Every exact source-backed observation is bound to one retained source identity.
- Exact endpoints originate from the owned parser lifecycle and MUST NOT be
  recovered later through source search, delimiter scanning, decoded-length
  inference, parser replay, fragment reparsing, or a second tokenizer.
- Raw authored spelling remains available independently of decoded, escaped,
  normalized, canonicalized, or interpreted values.
- Authored syntax, recovered syntax, discarded input, and synthesized structure
  remain distinguishable.
- Synthesized or implied structure MUST NOT claim an authored source range that
  does not exist.
- Source identity and range relationships are validated through Core-owned
  source contracts before becoming trusted analysis evidence.

## Capability and Result Integrity

Parser results MUST distinguish materially different states, including where
applicable:

- complete supported processing;
- complete processing with diagnostics;
- recovered processing;
- partial processing;
- unsupported or deferred capability;
- invalid boundary input;
- resource-limit termination;
- cancellation or abort when approved;
- internal invariant failure; and
- normal end of input.

Unsupported, partial, recovered, aborted, resource-limited, or failed processing
MUST NOT become clean success, clean absence, or successful zero observations.
Parser-result completeness cannot exceed the completeness of its tokenizer,
input, required grammar, or required static-semantics stages.

Equal retained source, approved configuration, parser implementation version,
and capability identity SHOULD produce equivalent analysis meaning and stable
semantic ordering.

## Layer and Dependency Boundary

Project-owned source parsers are Core-owned language-analysis responsibilities.
They may depend only on approved browser-independent contracts and lower-level
facilities permitted by the architecture.

Parser implementation details MUST NOT leak into unrelated Core domains,
Analysis Results, browser adapters, presentation adapters, or products. Narrow
visibility is the default. A public parser operation, type, error, trait, or
export requires a named consumer and focused compatibility approval.

Core parser code MUST NOT depend on:

- browser protocols or browser-engine object models;
- React, Electron, Tauri, VS Code, or product frameworks;
- filesystem, network, or process acquisition without a separately approved
  Core-owned boundary;
- external parser-native types or errors;
- serialization or transport formats selected for an outer consumer; or
- one native-only execution assumption that prevents equivalent approved WASM
  analysis meaning.

## Third-Party Parser and Browser Policy

Third-party parsers and browser engines MAY be used for:

- differential testing;
- interoperability comparison;
- fixture and regression discovery;
- standards-behavior cross-checks;
- bounded performance comparison; and
- optional adapters.

They MUST NOT:

- define authoritative Core source coordinates;
- force native types or recovery semantics into Core contracts;
- weaken required evidence because an API does not expose it;
- become mandatory production dependencies without focused approval;
- place an upstream feature, contribution, maintainer decision, or release on
  the Core parser critical path; or
- become the sole correctness oracle for project-owned parser validation.

Historical qualification and upstream records remain valid evidence of the
candidates and boundaries they investigated. The parser-ownership decision does
not rewrite those records.

## Implementation and Evolution

Each language proceeds through focused capability slices. The default sequence
within a language is:

```text
architecture and ownership
    ↓
source-backed token contracts
    ↓
diagnostics, recovery, completeness, and termination contracts
    ↓
candidate-independent validation foundation
    ↓
bounded tokenizer implementation
    ↓
analysis-parser model and implementation
    ↓
Core integration
    ↓
later tree or semantic expansion
```

A shared abstraction is permitted only when it protects a demonstrated
cross-language invariant. Similar names, syntax-tree shapes, or implementation
mechanics alone do not justify a generic parser framework or universal AST.

Foundational changes to parser ownership, layer placement, public compatibility,
serialization, concurrency, async, unsafe Rust, or workspace boundaries require
the applicable focused approval and ADR process.

## Validation Obligations

Each implemented capability requires proportionate evidence, including:

- specification-derived candidate-independent fixtures;
- exact retained UTF-8 source and range expectations;
- source identity, containment, ordering, and raw-spelling invariants;
- malformed, recovered, partial, unsupported, resource, abort, and termination
  cases;
- deterministic repeated-run validation;
- property tests and fuzzing with durable regression fixtures;
- panic and non-termination containment;
- bounded resource observations;
- non-authoritative differential comparison;
- dependency, feature, license, workspace, and public-export inventories; and
- native execution plus relevant WASM-target validation.

Validation evidence MUST report unavailable work honestly as `Not run` or
`Blocked`. It MUST NOT infer executable proof from design reasoning alone.

## Security and Unsafe Rust

Source input is untrusted. Malformed or adversarial input MUST NOT cause an
uncontrolled panic across a Core boundary. Resource exhaustion and termination
behavior require explicit ownership and validation.

Repository-authored `unsafe Rust` is not authorized by ADR 0007 or this
contract. Any proposal follows the focused exception process in
[Rust Core Contracts](RUST_CORE_CONTRACTS.md) and
[Secure Development](../development/SECURE_DEVELOPMENT.md).

## Current Coordination

- [#104](https://github.com/YT-TechDev/frontend-analysis/issues/104) owns the
  parser-program architecture and sequencing.
- [#106](https://github.com/YT-TechDev/frontend-analysis/issues/106) owns the
  first HTML parser and Core-integration workstream.
- [#107](https://github.com/YT-TechDev/frontend-analysis/issues/107) owns the
  later CSS workstream.
- [#108](https://github.com/YT-TechDev/frontend-analysis/issues/108) owns the
  later ECMAScript frontend and required static-semantics workstream.
