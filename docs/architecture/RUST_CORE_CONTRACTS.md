# Rust Core Contracts

## Purpose and Authority

This document is the normative Rust-design contract for the browser-independent
Frontend Analysis Core and Core-owned domain and Analysis Result contracts. It
specializes the durable [Architecture Principles](PRINCIPLES.md) within the
responsibilities and dependency direction owned by
[Architecture Layers](LAYERS.md). The [Documentation Index](../README.md)
classifies these documents and resolves source-of-truth questions.

This contract governs Rust-specific Core constraints only after maintainer
review and merge. It does not redefine layer boundaries or browser
independence, grant approval, or change security or release policy.
[Maintainership](../governance/MAINTAINERSHIP.md) determines valid approval;
[Secure Development](../development/SECURE_DEVELOPMENT.md) owns security
approval and security exceptions. The [Security Policy](../../SECURITY.md)
governs vulnerability reporting, and [Contributing](../../.github/CONTRIBUTING.md)
governs contribution workflow.

## Normative Language

- **MUST** and **MUST NOT** state a mandatory contract.
- **SHOULD** and **SHOULD NOT** state the default expectation; deviation
  requires documented evidence and review.
- **MAY** permits a choice when ownership and invariants remain satisfied.

## Scope and Non-Goals

This contract applies to Core-owned Rust boundaries. It does not automatically
govern Browser Adapter or Product internals, although those layers must preserve
the approved architecture and their crossings into Core must satisfy Core's
boundary contracts.

This document does not define a workspace, crates, modules, production structs,
enums, traits, functions, methods, or public API. It selects no parser,
protocol, serialization, lock, channel, or runtime library. It makes no choice
of thread pools, storage, FFI, WebAssembly guarantees, `no_std` support, or
performance strategy, and creates no stable API, ABI, serialization, SemVer, or
release promise. Analysis Results remain a domain contract boundary, not
necessarily a separate crate or process.

## Ownership and Lifecycle

- Every domain concept MUST have an identifiable owner for its data, lifecycle,
  mutation, invariants, and invalidation.
- Durable Core boundary values SHOULD be owned unless an explicit borrowing
  contract has demonstrated benefit and manageable lifetime semantics.
- Core values MUST NOT borrow from adapter transport buffers, browser sessions,
  UI state, or product framework contexts across durable boundaries.
- Boundary ownership MUST NOT depend on a browser protocol object's lifetime.
- Ownership transfer MUST be explicit.
- Shared ownership MUST represent a real ownership graph, not uncertainty about
  ownership.
- Invalidation and stale-state behavior MUST have an owner.
- Identifier validity scope and lifetime MUST be documented when identifiers
  can outlive their source observations.
- Holding data does not grant authority to redefine its semantic meaning.

This contract does not prescribe Rust lifetimes, arenas, smart pointers,
generational identifiers, or storage layouts.

## Borrowing and Cloning

- Borrowing SHOULD be used inside local algorithms when it improves clarity or
  avoids unnecessary allocation.
- Complex lifetimes SHOULD NOT leak across public or layer boundaries without
  demonstrated value.
- Owned boundary values are the default; this is not a zero-copy mandate.
- Cloning MUST be an intentional semantic and cost decision.
- Cloning MUST NOT be the default escape hatch for unresolved ownership.
- Cloning a large evidence structure, graph, syntax representation, or result
  set requires a documented reason and cost assessment.
- Clone behavior MUST NOT create two apparent owners of mutable semantic state.
- Cheap immutable cloning MAY be acceptable when cost and meaning are clear.

Copy-on-write, arenas, interning, and reference counting remain deferred. This
contract does not prohibit all cloning.

## Mutation and Shared State

- Immutable values are the default.
- Mutation MUST have one named owner and a bounded lifecycle.
- Mutation MUST preserve invariants at every externally observable boundary.
- APIs MUST NOT expose invalid intermediate state as valid domain state.
- Global mutable state and hidden process-wide mutable registries are
  prohibited.
- State MUST NOT cross layers through framework context, service locators,
  generic utilities, or hidden callbacks.
- Interior mutability requires a named owner, the protected invariant, why
  ordinary exclusive mutation is insufficient, thread and reentrancy
  assumptions, failure behavior, and validation.
- `Rc`, `Arc`, `Cell`, `RefCell`, locks, atomics, and channels require
  demonstrated need. `Arc<Mutex<_>>` is not a default architecture.
- Shared mutation MUST define who mutates, when mutation occurs, and what
  readers observe.
- Lock scope and ordering MUST be documented where applicable.
- Locks SHOULD NOT be held across external callbacks or async suspension
  without explicit safety and liveness evidence.
- Cache ownership and invalidation MUST be explicit.

This contract selects no synchronization or cache mechanism.

## Domain Type Modeling

- Meaningful types SHOULD protect semantic distinctions where primitive
  interchange could cause errors.
- Identifier newtypes SHOULD be used when they protect identity scope, units,
  provenance, or cross-domain substitution. A wrapper MUST NOT be introduced
  only for style.
- Mutually exclusive states SHOULD use an enum or equivalent
  invalid-state-preventing model. Independent dimensions SHOULD NOT be forced
  into one combinatorial enum.
- Boolean parameters SHOULD be avoided when call-site meaning is unclear.
  Multiple booleans MUST NOT represent mutually exclusive states when a
  meaningful state model can protect the invariant.
- Free-form strings MUST NOT replace a finite owned vocabulary when the domain
  is stable enough to model. A closed enum MUST NOT represent an intentionally
  open domain without compatibility analysis.
- Construction SHOULD validate invariants before general use. Fields that
  protect invariants SHOULD NOT be publicly mutable.
- Unvalidated parser or transport data MUST NOT be treated as valid Core domain
  state.
- Traits and generics require a demonstrated variation point or invariant.
- Units, coordinate systems, source identity, provenance, and lifecycle scope
  MUST be distinguishable when confusion changes meaning.
- Ordering MUST be explicit when order is semantic. Hash iteration, allocation,
  address, task-completion, or incidental parser order MUST NOT silently become
  semantic order.

This document defines no production type.

## Optional, Partial, and Conflicting Evidence

- `Option` MUST NOT ambiguously combine absent, unknown, unsupported,
  unobserved, redacted, invalid, or conflicting states when those distinctions
  affect analysis.
- Materially different absence states SHOULD be modeled explicitly.
- Partial analysis MUST distinguish available results from failed or
  unavailable portions.
- Unsupported browser observations MUST NOT be rewritten as absence.
- Conflicting evidence MUST NOT be collapsed into one arbitrary value.
- Uncertainty MUST remain visible until an owned rule resolves it.
- Defaults MUST NOT replace missing evidence when doing so changes analysis
  meaning.
- Sentinel strings, magic values, and empty collections MUST NOT encode
  multiple states without a contract.
- Evidence completeness and certainty remain domain semantics, not presentation
  decisions.

This contract defines neither an evidence enum nor a confidence scale.

## Protocol and Serialization Separation

- Browser protocol types terminate at the Browser Adapter boundary.
- Parser, transport, and serialization data-transfer objects MUST NOT become
  Core public types by convenience.
- Conversion into Core MUST validate invariants and represent loss or
  unsupported states honestly.
- Serialization is a separate compatibility contract. Adding serialization
  derives or a wire representation is not a harmless internal detail.
- Core layout MUST NOT be shaped around one protocol or serialization library.
- Third-party types, errors, traits, feature flags, and versioning MUST NOT leak
  into durable Core contracts without approval.
- Debug or display formatting is not automatically a stable serialized format.

No serialization library or format is selected here.

## Error Modeling

- Recoverable failure at an owned boundary MUST be explicit.
- Future public and cross-layer errors MUST be owned by that boundary.
- Adapter, protocol, parse, domain-validation, analysis, cancellation, and
  infrastructure failures remain separate when their handling differs.
- Core MUST NOT depend on Browser Adapter error types.
- Third-party errors MAY be preserved internally as sources but MUST NOT become
  durable public error types by convenience.
- Error conversion MUST retain useful context without exposing secrets or
  unstable implementation details.
- Failure scope MUST be explicit: whole operation, one input, partial result,
  unsupported observation, or cancellation.
- A partial result MUST NOT be represented as complete success.
- One catch-all string error MUST NOT replace owned error categories. One
  global mega-error SHOULD NOT span unrelated ownership domains.
- Localized user-facing prose belongs outside Core error semantics.

This contract defines no error enum, code, retry policy, or API signature.

## Panic, `unwrap`, and `expect`

- Malformed or untrusted input, unsupported browser behavior, external failure,
  configuration, cancellation, resource failure, and ordinary runtime
  conditions MUST NOT panic at a Core boundary.
- Panic is reserved for programmer error or an internal invariant violation
  that is not recoverable at that location. Panic MUST NOT be control flow.
- Production `unwrap` and `expect` MUST NOT be used for externally influenced
  data, environment, timing, concurrency, I/O, or user behavior.
- Tests MAY use `unwrap` and `expect`.
- Production `expect` MAY be used only when the proof is immediate, reviewable,
  unaffected by external state, and its message states the invariant. A clearer
  type or control-flow design should be preferred.
- Debug assertions MUST NOT be the only enforcement of required release
  invariants.

This document selects no global panic strategy.

## Concurrency

Core SHOULD remain single-owner and deterministic unless an approved
requirement demonstrates concurrency value. A concurrency proposal MUST define:

- work ownership;
- shared or transferred values;
- ordering;
- cancellation;
- boundedness and backpressure;
- error aggregation;
- shutdown and lifecycle;
- determinism; and
- validation.

Shared ownership MUST NOT be introduced only because parallelism may be useful
later. Intentional `Send` or `Sync` promises require compatibility review, and
accidental public auto-trait behavior may itself become a compatibility concern.
An `unsafe impl Send` or `unsafe impl Sync` requires the full unsafe exception
process below.

Parallel execution MUST NOT silently change analysis meaning. Completion order
MUST NOT become semantic order without approval. Races, deadlocks, starvation,
and unbounded queues require explicit review. Concurrency mechanisms stay
behind owned contracts. This document selects no threads, pools, channels,
locks, or libraries.

## Asynchronous Boundaries

- Async is introduced only for an owned asynchronous requirement.
- Adapter async I/O does not automatically make Core domain contracts async.
- Async outer layers MAY invoke synchronous Core analysis.
- Executor, reactor, task, runtime, and framework types MUST NOT leak into Core
  domain models.
- A concrete async runtime requires explicit approval.
- Async boundaries MUST define owner, cancellation, completion, errors,
  ordering, resources, and backpressure.
- Cancellation MUST NOT leave observable invalid Core state.
- Hidden blocking work MUST NOT be placed behind an async-looking boundary
  without an explicit contract.
- Locks SHOULD NOT cross `.await` or equivalent suspension without focused
  safety and liveness proof.

Async traits, futures, streams, channels, and callbacks remain unselected. This
document defines neither synchronous nor asynchronous APIs.

## Determinism Under Concurrency

- Equal normalized inputs and approved configuration SHOULD preserve equivalent
  analysis meaning regardless of scheduling.
- Parallelism MAY change timing but MUST NOT silently change evidence meaning,
  certainty, or conflict resolution.
- Result order MUST be stable when order is part of the contract.
- When order is non-semantic, consumers MUST NOT infer meaning from scheduling
  order.
- Generated identifiers, timestamps, and completion order MUST NOT become
  hidden semantic inputs.
- Deterministic or single-threaded validation SHOULD remain possible where the
  domain permits.

## Visibility and Public API

- Narrowest useful visibility is the default. Items remain private unless
  another owner demonstrably needs access.
- `pub` is an architecture and compatibility decision, not convenience.
- Public re-exports, fields, traits, bounds, auto traits, errors, constructors,
  and macros may create commitments and require review.
- Internal types MUST NOT become public integration points.
- Technical reachability does not equal approved public API.
- Tests SHOULD use narrow testing seams rather than widened production
  visibility.
- Fields protecting invariants SHOULD remain non-public.
- Exposure to another crate, process, language, or product requires focused
  boundary approval.

This contract creates no stable Rust API.

## Compatibility Dimensions

Rust source API, ABI, serialization, browser protocol, adapter, Analysis Result
semantics, diagnostic meaning, ordering and determinism, `Send` and `Sync`,
performance, and presentation are separate compatibility dimensions.

- Compatibility in one dimension does not imply compatibility in another.
- Unchanged serialization does not guarantee unchanged semantics.
- Fields, variants, bounds, auto traits, errors, ordering, and defaults may be
  breaking depending on their contract.
- Dependency versioning does not define project compatibility.
- Breaking impact MUST be explicit.
- Migration and deprecation require focused decisions where applicable.

This contract creates no SemVer, stable ABI, or serialization promise.

## Unsafe Rust

```text
unsafe Rust is not permitted without an explicitly approved Issue.
```

[Secure Development](../development/SECURE_DEVELOPMENT.md) owns security
approval, and [Maintainership](../governance/MAINTAINERSHIP.md) determines valid
approval. This document owns the Rust-specific evidence and containment
requirements. The rule applies to repository-authored Rust; it does not claim
that every transitive dependency is unsafe-free.

Before implementation, an unsafe proposal requires:

1. the exact unsafe operation;
2. why safe Rust is insufficient;
3. safe alternatives considered;
4. evidence of necessity;
5. benchmark or profiling evidence for performance claims;
6. the smallest unsafe boundary;
7. exact safety invariants;
8. ownership and lifetime invariants;
9. aliasing and mutation invariants;
10. initialization and validity invariants;
11. thread-safety and `Send` and `Sync` implications;
12. panic and unwinding implications where relevant;
13. FFI and platform assumptions where relevant;
14. a safe containment interface;
15. edge-case tests;
16. appropriate static, dynamic, fuzz, sanitizer, interpreter, and platform
    validation where applicable;
17. maintenance and audit risk;
18. documentation;
19. a removal or further-containment strategy; and
20. explicit maintainer approval in a focused Issue.

Unsafe MUST NOT be introduced to silence the borrow checker, avoid ownership
design, pursue speculative performance, support a performance claim without
benchmark evidence, bypass layer boundaries, rely on unchecked protocol
assumptions, assert unproven `Send` or `Sync`, follow an example or agent's
recommendation, or hide inside an unrelated Pull Request.

Each unsafe block requires a concise safety rationale tied to approved
invariants. Unsafe must be minimal, contained, reviewable, and re-reviewed when
surrounding assumptions change. This document does not approve an unsafe
exception.

## Review Checklist

Reviewers must examine the following; completing the checklist does not approve
a design:

- [ ] 1. Is the layer owner identified?
- [ ] 2. Are the data, lifecycle, and mutation owners identified?
- [ ] 3. Is the boundary intentionally owned or borrowed?
- [ ] 4. Are clone semantics and cost justified?
- [ ] 5. Is shared ownership necessary?
- [ ] 6. Is every interior-mutability invariant documented?
- [ ] 7. Does the model prevent invalid states without stylistic wrappers?
- [ ] 8. Are unknown, unsupported, partial, and conflicting states preserved?
- [ ] 9. Are protocol and serialization details contained?
- [ ] 10. Does the boundary own its errors?
- [ ] 11. Can external conditions expose a panic?
- [ ] 12. Is concurrency necessary?
- [ ] 13. Are ordering, cancellation, backpressure, and determinism defined?
- [ ] 14. Does async originate in an owned asynchronous requirement?
- [ ] 15. Is visibility narrow, and is every public commitment intentional?
- [ ] 16. Are affected compatibility dimensions separate and explicit?
- [ ] 17. Are there any unsafe assumptions?
- [ ] 18. Are required documentation and approval identified?
- [ ] 19. Is validation evidence proportionate?
- [ ] 20. Would a simpler design satisfy the contract?

## Representative Proposals

Each outcome applies the contracts above; only the named authority can approve
an escalation.

### 1. Clone a Large Evidence Graph to Solve a Borrow Error

- **Verdict:** Prohibited as an unreviewed ownership escape hatch; redesign
  ownership or borrowing first. A clone is conditional only with a semantic
  reason and cost evidence.
- **Owner:** The Core owner of the evidence graph and consuming analysis.
- **Invariant:** Cloning cannot obscure ownership or create apparent owners of
  mutable semantic state.
- **Required evidence:** Semantic need, alternatives, graph size, allocation and
  runtime cost, lifecycle, and validation.
- **Escalation:** Focused ownership and compatibility review by the maintainer.
- **May implementation proceed?** No for the escape hatch; only after the
  conditional clone is approved.

### 2. Store Shared State in `Arc<Mutex<_>>`

- **Verdict:** Prohibited as a default; conditional only for demonstrated
  concurrent shared mutation.
- **Owner:** The component that owns the state, mutation, and lifecycle.
- **Invariant:** Readers observe defined valid states and mutation preserves
  domain invariants.
- **Required evidence:** Need, ownership, protected invariants, lock scope and
  order, contention, deadlock, poisoning, lifecycle, determinism, and validation.
- **Escalation:** Focused concurrency and architecture review.
- **May implementation proceed?** No until the evidence and approval exist.

### 3. Expose a CDP Enum from Core

- **Verdict:** Prohibited.
- **Owner:** The Browser Adapter owns Chrome DevTools Protocol (CDP) concepts.
- **Invariant:** Core contracts remain browser-independent.
- **Required evidence:** An approved browser-independent concept and a validated
  boundary conversion preserving unknown and unsupported evidence.
- **Escalation:** Architecture boundary review if no adequate Core concept
  exists.
- **May implementation proceed?** No; only the adapter conversion may proceed
  within an approved contract.

### 4. Represent Mutually Exclusive States with Booleans

- **Verdict:** Prohibited when invalid combinations or unclear meaning are
  possible.
- **Owner:** The Core owner of the domain state.
- **Invariant:** Invalid mutually exclusive combinations are not representable.
- **Required evidence:** State dimensions, vocabulary openness, compatibility,
  and proof that the proposed model preserves invariants.
- **Escalation:** Focused domain-model review; use a meaningful state model when
  the domain is closed enough.
- **May implementation proceed?** No with unsafe boolean combinations; this
  document does not invent the replacement production type.

### 5. Return a Third-Party Parser Error Publicly

- **Verdict:** Prohibited.
- **Owner:** Parser integration owns the third-party error; the exposed boundary
  owns its error contract.
- **Invariant:** Durable errors do not leak unstable third-party details.
- **Required evidence:** Boundary-owned categories, context and secret handling,
  source preservation needs, and compatibility impact.
- **Escalation:** Public-boundary and compatibility review.
- **May implementation proceed?** No; map the failure to a boundary-owned error
  and preserve the parser error internally as a source if useful.

### 6. Make a Core Trait Async Because an Adapter Uses Async I/O

- **Verdict:** Prohibited without a Core-owned asynchronous requirement.
- **Owner:** The Browser Adapter owns its async I/O; Core owns its analysis
  contract.
- **Invariant:** Outer runtime control flow does not reverse source dependency
  ownership or leak runtime concerns into Core.
- **Required evidence:** A Core-owned async need, cancellation, completion,
  errors, ordering, resources, backpressure, compatibility, and alternatives.
- **Escalation:** Focused async-boundary and public-contract review.
- **May implementation proceed?** No; an async outer layer may call synchronous
  Core analysis.

### 7. Add Unsafe for Performance Without Benchmark Evidence

- **Verdict:** Prohibited.
- **Owner:** The proposed unsafe boundary's owner, subject to security and
  maintainer authority.
- **Invariant:** Safe Rust remains the default and every unsafe assumption is
  proven and contained.
- **Required evidence:** All twenty unsafe proposal requirements, including
  benchmark evidence and safe alternatives.
- **Escalation:** A focused Issue under Secure Development with explicit
  maintainer approval.
- **May implementation proceed?** No before the complete exception process.

### 8. Use `RefCell` to Mutate Through a Shared Reference

- **Verdict:** Conditional only for documented single-threaded ownership and a
  protected invariant; prohibited merely to bypass ownership design.
- **Owner:** The component owning the value and interior mutation.
- **Invariant:** Runtime borrow rules, reentrancy, and externally observable
  domain validity are preserved.
- **Required evidence:** Why exclusive mutation is insufficient, ownership,
  lifecycle, runtime borrow-failure behavior, reentrancy, and validation.
- **Escalation:** Focused ownership and interior-mutability review.
- **May implementation proceed?** Only after demonstrated need and approval.

### 9. Derive Serialization on a Core Domain Type for Convenience

- **Verdict:** Conditional or prohibited until a serialization contract is
  approved.
- **Owner:** Core owns domain meaning; the relevant boundary owns its wire
  contract.
- **Invariant:** Domain and wire compatibility remain separate.
- **Required evidence:** Consumers, format, versioning, unknown-value behavior,
  compatibility dimensions, invariant validation, and alternatives.
- **Escalation:** Focused serialization and compatibility review.
- **May implementation proceed?** No as a convenience-only derive.

### 10. Make All Core Items Public for Easier Integration

- **Verdict:** Prohibited.
- **Owner:** Core owns its internals; each approved integration boundary owns
  its exposed contract.
- **Invariant:** Products consume approved boundaries rather than Core internals.
- **Required evidence:** A consumer and owner for each exposure, invariants,
  compatibility cost, and a narrower alternative analysis.
- **Escalation:** Focused public-API and architecture review.
- **May implementation proceed?** No; visibility must remain as narrow as useful.

### 11. Return a Borrowed Core Value Tied to an Adapter Parser Buffer

- **Verdict:** Prohibited across a durable Core boundary.
- **Owner:** The adapter owns parser buffers; Core owns its durable boundary
  value.
- **Invariant:** Core lifecycle and validity do not depend on adapter or parser
  storage.
- **Required evidence:** An alternative ownership transfer, conversion and
  validation plan, lifetime scope, and cost assessment.
- **Escalation:** Architecture and ownership-boundary review if an owned
  conversion cannot satisfy the requirement.
- **May implementation proceed?** No with the cross-boundary borrow.

### 12. Parallelize Analysis and Return Task-Completion Order

- **Verdict:** Conditional only when order is explicitly non-semantic;
  prohibited when callers observe changed semantic order.
- **Owner:** Core owns analysis meaning and result ordering; the execution owner
  owns parallel work.
- **Invariant:** Scheduling does not change evidence meaning, certainty,
  conflict resolution, or contractual order.
- **Required evidence:** Concurrency value, work ownership, ordering contract,
  cancellation, boundedness, backpressure, error aggregation, lifecycle,
  deterministic validation, and consumer behavior.
- **Escalation:** Focused concurrency, semantics, and compatibility review.
- **May implementation proceed?** Only after the evidence proves order is
  non-semantic and the full concurrency contract is approved.

## Deferred Implementation Decisions

The following remain deferred:

- workspace, crate, and module structure;
- concrete ownership types;
- arenas, interning, and copy-on-write;
- identifiers and production domain types;
- error enums and codes;
- public traits and functions;
- parser, protocol, and serialization libraries;
- synchronous or asynchronous public APIs;
- runtime and executor;
- locks, atomics, channels, threads, and pools;
- caches and storage;
- cancellation and streaming APIs;
- FFI, WebAssembly, and `no_std` support;
- performance optimization;
- stable API, ABI, and SemVer policy; and
- every unsafe exception.

Deferred does not mean unrestricted. A future proposal must still satisfy the
applicable architecture, security, governance, and Rust-specific contracts.
