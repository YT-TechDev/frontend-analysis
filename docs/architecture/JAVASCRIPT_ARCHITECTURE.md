# JavaScript / ECMAScript Architecture

## Purpose and Authority

This document is the specialized normative architecture contract for JavaScript /
ECMAScript semantic analysis in Frontend Analysis.

It defines the durable semantic responsibilities, ownership boundaries,
information-flow constraints, lifecycle rules, provenance requirements, and
representation-neutrality requirements that future JavaScript work MUST preserve.

It specializes, but does not supersede:

- [Architecture Principles](PRINCIPLES.md);
- [Architecture Layers and Boundaries](LAYERS.md);
- [Rust Core Contracts](RUST_CORE_CONTRACTS.md);
- [Source Parser Ownership](SOURCE_PARSER_OWNERSHIP.md);
- [ADR 0007 — Own Lossless Source Parsers](../decisions/0007-own-lossless-source-parsers.md); and
- [ADR 0008 — Define Browser Runtime Evidence Normalization and Core Import Ownership](../decisions/0008-browser-runtime-evidence-normalization-and-core-import.md).

[ADR 0009 — Define JavaScript Semantic Analysis Architecture](../decisions/0009-javascript-semantic-analysis-architecture.md)
records the approved rationale for this contract. The durable JavaScript research
record in [`docs/evidence/javascript/`](../evidence/javascript/README.md) is
supporting evidence and research history; it does not override this normative
contract.

This document does not authorize production implementation by itself. Production
work still requires focused approved Issues and all applicable repository
validation and compatibility review.

## Problem Definition

Frontend Analysis needs to determine what exact ECMAScript source means under an
explicit edition and qualification context, derive optional browser-independent
semantic claims from that source, and incorporate qualified host/runtime evidence
without conflating:

- source-standard semantics;
- analysis inference;
- host semantics;
- browser-specific realization;
- runtime observation; or
- physical implementation representation.

The Core is an analysis platform, not a JavaScript engine. It MUST remain
browser-independent and MUST NOT make one browser engine, one parser, one AST/IR,
one abstract interpreter, one graph model, or one runtime protocol the semantic
authority for JavaScript.

## Non-Goals

This contract does not make Frontend Analysis:

- a JavaScript VM or interpreter;
- a JIT or optimizing compiler backend;
- a browser event-loop implementation;
- a browser module loader;
- a garbage collector;
- a universal whole-program analyzer;
- a mandatory abstract interpreter;
- a mandatory CFG or graph framework; or
- a one-type-per-ECMAScript-concept Rust architecture.

## Conceptual Responsibility Model

The JavaScript architecture has four conceptual responsibility boundaries:

```text
Source + Provenance + Explicit Qualification Context
        ↓
Standard Qualification
        ↓
source-backed qualification evidence + scoped lifecycle
        ↓
optional / open / composable Semantic Analysis Capabilities
        ↑
qualified Host / Runtime Evidence
        ↑
Browser Adapter / Browser Runtime

Qualification + Semantic Claims + Runtime Evidence
        ↓
Qualified Analysis Results / Evidence Provenance
```

This diagram expresses semantic responsibility and information flow. It is not a
mandatory execution pipeline, crate diagram, module topology, public API, AST/IR
shape, solver order, or serialization schema.

## Architecture Invariants

### JA-1 — Evidence and qualification are explicitly profiled

Every material JavaScript claim MUST preserve enough information to recover the
semantic envelope under which the claim is justified.

Depending on the claim, that envelope may include:

- ECMAScript edition;
- standards profile;
- Normative Optional or standardized host-qualification policy;
- grammar goal or source kind;
- qualification context;
- analysis objective and capability coverage;
- dynamic-code policy;
- host assumptions;
- precision/resource policy;
- runtime observation scope; and
- evidence origin.

A yearly ECMAScript edition and a later current draft MUST NOT be silently treated
as one qualification profile.

### JA-2 — Standard Qualification is bounded and context-qualified

Standard Qualification owns source-standard qualification under exact retained
source, explicit ECMAScript edition/profile, grammar goal/source kind, and the
required qualification context.

Where the standard makes source/static-semantic validity depend on standardized
Normative Optional or host-qualification policy, that policy MAY be an explicit
qualification input.

Standard Qualification MUST NOT use successful execution in Chrome, Firefox,
WebKit, or another runtime as authority for ECMAScript source-standard validity.
Browser/runtime evidence belongs to the Host / Runtime Evidence boundary.

Source-standard qualification does not by itself establish any of the following:

```text
loadability
link validity
evaluation safety
termination
effect-freedom
host support
```

### JA-3 — Downstream analysis is claim-prerequisite-driven

Whole-source qualification success is not a universal gate for every downstream
analysis.

A semantic capability MUST declare the minimum upstream facts and lifecycle
conditions required for the claim it produces. Source-structural, diagnostic,
recovery-based, or hypothetical analysis MAY remain useful when a source is
invalid, incomplete, or partially qualified.

However, a claim about normative ECMAScript execution MUST NOT outlive a validity
prerequisite required for that execution. Invalid or incomplete source MUST NOT
silently acquire executable-program semantics through downstream analysis.

### JA-4 — Semantic authority is explicit

Semantic authority, evidence origin, computation, storage, and result transport
are distinct ownership dimensions.

A material semantic proposition MUST have an unambiguous owning contract for its
meaning. Multiple analyses or evidence sources MAY produce or support a claim;
that does not transfer semantic authority.

Circular definition of semantic authority is prohibited. Cyclic computational
refinement is permitted only through an explicit bounded solver contract.

### JA-5 — Capability decomposition is semantic, open, and representation-neutral

Current semantic capability families may include:

- Binding / Scope;
- Control / Completion;
- Effect;
- Value / Abstract Interpretation;
- Interprocedural;
- Module;
- Async;
- Concurrency / Memory; and
- future capabilities justified by evidence and product needs.

These names describe semantic responsibilities. They do not mandate one crate,
module, trait, pass, service, graph, or public type per capability.

One physical component MAY implement multiple semantic capabilities. One semantic
capability MAY use multiple internal algorithms or representations. The taxonomy
remains open.

### JA-6 — JavaScript control and effects admit hidden and enclosing semantics

The architecture MUST be able to represent semantic consequences that are not
visible from surface `CallExpression` syntax alone.

Relevant examples include:

- accessors;
- Proxy and exotic-object internal methods;
- coercion;
- iterator protocol operations;
- unknown calls;
- cleanup and closing operations;
- pending abrupt completion;
- `finally` mediation;
- suspension and resumption; and
- async cleanup.

The architecture MUST NOT equate:

```text
no obvious call syntax
= no user-code invocation
= pure evaluation
= no abrupt completion
= safe elimination / duplication / reordering
```

Pending control transfer and finalized control transfer MUST remain distinguishable
where enclosing semantics can mediate the transfer.

### JA-7 — Host/runtime evidence crosses a qualified browser-independent boundary

The following concepts MUST remain distinguishable:

```text
ECMAScript normative semantics
ECMAScript host contracts
host-standard semantics
browser-specific realization
runtime observation
Core-derived interpretation
```

ADR 0008 remains authoritative for Browser Adapter capture ownership,
target-lifetime evidence, runtime-source identity, `SourceId` authority,
native-coordinate termination, Application Orchestration, and Core-facing import
validation.

This contract specializes only JavaScript semantic consumption: qualified
browser-independent host/runtime evidence MAY refine or correlate JavaScript
analysis, but runtime observation MUST NOT silently become intrinsic ECMAScript
truth and browser brands MUST NOT become Core semantic branches.

Runtime evidence is optional. Source-only qualification and static analysis MUST
remain possible when the selected capability permits.

### JA-8 — Runtime evidence is scoped and observationally qualified

Material runtime evidence MUST preserve enough context to understand its origin,
applicability, and limitations. Relevant dimensions may include:

- browser/runtime origin;
- target or observation lifetime;
- subject identity or correlation basis;
- completeness;
- lossiness;
- unavailable or unsupported evidence;
- observation interval; and
- ordering semantics.

The Core MUST NOT infer semantic absence merely from an absent runtime event unless
the observation channel is known to be complete for the relevant negative query.

```text
not observed != did not happen
```

Protocol receive order, timestamps, host scheduling order, ECMAScript execution
order, and shared-memory relations MUST NOT be silently collapsed into one
ordering concept.

### JA-9 — Claims carry a semantic applicability envelope

A material analysis claim MUST remain interpretable under the assumptions and
coverage that justify it.

The architecture MUST preserve enough context to distinguish, where relevant:

- must / necessary;
- may / possible;
- excluded / cannot;
- observed;
- unsupported;
- indeterminate; and
- resource-limited or incomplete analysis.

A generic scalar confidence score MUST NOT replace semantic modality or analysis
lifecycle.

### JA-10 — Lifecycle and claim modality remain distinct

Run/capability lifecycle is scoped to the operation that produced it.

A composite JavaScript result MAY contain, for example:

```text
Standard Qualification: Complete
Binding Analysis: Complete
Value Analysis: Resource Limited
Runtime Capture: Partial
```

These states MUST NOT be collapsed into an ambiguous global `Complete` unless the
scope of that status is explicitly defined.

A complete run may still produce an indeterminate claim. A resource-limited run
may still contain claims already justified before the limit was reached.

Absence of an emitted finding is not evidence of semantic absence unless the
relevant negative query completed under sufficient prerequisites and coverage.

### JA-11 — Provenance and conflicts survive derivation

Material derived claims MUST remain traceable to sufficient supporting evidence
and derivation context.

Authored source evidence MUST use exact retained source provenance and the
project-owned source-anchor contracts. Derived semantic events that were not
authored MUST NOT receive fabricated authored ranges merely because authored
syntax caused or supported the derivation.

Evidence conflicts MUST remain representable. Normative evidence, host-standard
evidence, browser observations, and analysis inferences MUST NOT be collapsed by
a global priority rule without first checking applicability envelopes.

When a required premise loses validity, dependent claims MUST be re-evaluated.
A claim MAY remain justified only when retained provenance establishes an
independent sufficient support path.

### JA-12 — Semantic distinctions constrain expressiveness, not physical representation

This architecture freezes the semantic distinctions that future implementation
must preserve. It intentionally does not freeze the physical representation used
to preserve them.

The following remain open until separately justified:

- JavaScript AST/CST/IR and lowering strategy;
- abstract-value domain or lattice shape;
- state/store decomposition;
- heap/location/alias/identity abstraction;
- Unknown/Top/coarse-state taxonomy;
- strong/weak update representation;
- path/context/heap sensitivity policy;
- CFG or Completion representation;
- effect taxonomy or lattice;
- call graph, summaries, or interprocedural solver;
- module identity or relation storage;
- async/job/scheduling representation;
- shared-memory/concurrency representation;
- Proxy-specific abstract domain;
- WeakRef/GC/liveness representation;
- evidence graph or invalidation storage;
- result Rust types or schemas;
- crate/module topology;
- traits or public APIs;
- severity/confidence presentation policy;
- serialization; and
- async/concurrency implementation machinery.

## Responsibility Boundaries

### Standard Qualification

Standard Qualification answers:

> Given this exact source, selected ECMAScript edition/profile, grammar goal, and
> required qualification context, which source-standard facts can Frontend
> Analysis justify without executing the program or resolving its external
> environment?

It may own, within implemented scope:

- lexical and grammar qualification;
- applicable source-level Static Semantics;
- Early Errors;
- source-established declaration and relation facts;
- source-level module facts such as `ModuleRequest` where external resolution is
  unnecessary;
- source-standard diagnostics; and
- qualification lifecycle/completeness.

Parser acceptance is syntax evidence, not complete semantic authority. A parser
may accept source that still fails required Early Errors or profile checks.

Standard Qualification does not own:

- complete binding/call/effect/value analysis;
- host module resolution or loading;
- linking/evaluation success;
- browser scheduling;
- runtime observation; or
- whole-program guarantees.

### Semantic Analysis Capabilities

A Semantic Analysis Capability owns a bounded semantic question, the prerequisites
required for its claims, claim meaning, analysis assumptions, uncertainty, and
completion semantics.

Capabilities SHOULD consume declared semantic facts/contracts instead of
unrestricted internal state owned by another capability.

Current responsibility examples include:

- Binding / Scope: declarations, lexical relationships, identifier-reference
  qualification, and dynamic-resolution uncertainty;
- Control / Completion: normal/abrupt continuation, pending transfer, cleanup
  mediation, suspension, and resumption relations;
- Effect: observable/change/hidden-invocation consequences without assuming one
  universal effect lattice;
- Value / Abstract Interpretation: optional abstract-value or abstract-state
  properties under an explicitly declared domain and analysis envelope;
- Interprocedural: possible callable targets, cross-function propagation,
  recursion/cycles, and interprocedural relations without requiring one call-graph
  representation;
- Module: source module relations and downstream analysis of qualified host-resolved
  module evidence without owning host resolution itself;
- Async: ECMAScript-side suspension/resumption, Promise/Job dependencies, async
  completion mediation, async iterator cleanup, and top-level-await dependencies;
  and
- Concurrency / Memory: agent/shared-memory semantic relations such as
  reads-from, synchronizes-with, happens-before, and related findings when that
  capability is implemented.

A future WeakRef/liveness capability remains possible, but this contract does not
freeze its ownership decomposition or representation.

#### Cyclic solver composition

Analysis solving may be cyclic, for example:

```text
Value
  ↕
Possible Callee
  ↕
Effects
```

Such composition MUST NOT become ad-hoc recursive internal coupling. A cyclic
solver contract MUST declare, conceptually:

- participating capabilities;
- the shared analysis objective;
- initial approximation;
- refinement direction/meaning;
- convergence or termination policy;
- resource bounds; and
- incomplete-result semantics.

The concrete fixpoint, worklist, widening, or other solver remains open.

### Host / Runtime Evidence

The Browser Adapter owns browser-specific protocol transport, session/target
lifecycle, raw protocol identifiers, browser-native coordinates, raw observations,
and browser-specific decoding/validation as defined by the broader architecture
and ADR 0008.

The stable browser-independent side owns the normalized evidence contract meaning.
Adapters produce evidence that conforms to that contract; they do not gain
semantic authority to redefine JavaScript meaning.

Material runtime-derived claims SHOULD remain auditable to their originating
observation. This does not require Core to retain, borrow, or publicly expose raw
protocol payloads or live adapter state.

Source qualification and runtime observation remain independent evidence channels.
For example:

```text
ES2026-qualified source
+ observed host module-load failure
```

means that a standard-qualified source failed under that observed host/load
environment. It does not retroactively make the source invalid ECMAScript.

Likewise, browser acceptance of implementation-extension syntax does not make that
syntax qualified under the selected ECMAScript profile.

### Qualified Analysis Results / Evidence Provenance

Analysis Results carry qualified claims rather than unqualified value bags.
Where meaning differs, results MUST be able to preserve distinctions among:

- source/runtime observation;
- normative or standard-backed claim;
- analysis inference;
- finding;
- diagnostic;
- claim modality;
- applicability envelope;
- lifecycle/completeness;
- supporting evidence;
- derivation provenance; and
- conflict relation.

This does not require one giant result structure or one evidence graph. The
physical representation remains open.

## Partial, Invalid, and Incomplete Source

Frontend Analysis SHOULD preserve useful source-backed evidence even when a
source is invalid, recovered, unsupported, context-insufficient, or
resource-limited, when doing so is semantically justified.

However, downstream analysis MUST preserve the meaning of the upstream state.
Examples:

```text
invalid source
→ source syntax/declaration evidence may still exist
→ diagnostics/recovery analysis may still exist
→ normative execution claim is not automatically authorized
```

A later capability may refine or derive from earlier facts, but MUST NOT silently
rewrite the historical meaning or lifecycle of those facts.

## Host Qualification Profile vs Runtime Profile

The architecture distinguishes:

```text
Host / Normative-Optional Qualification Profile
!=
Host / Runtime Observation Profile
```

A standardized qualification policy may affect source qualification when the
selected ECMAScript edition explicitly makes source/static-semantic rules depend
on that policy.

A browser/runtime profile describes implementation or observation context. It
MUST NOT become a hidden switch inside Standard Qualification merely because a
browser exhibited a behavior.

Future work may define concrete profile representation and selection rules. This
contract freezes the semantic distinction, not the data model.

## Compatibility and Evolution

### New ECMAScript editions

Adding a newer ECMAScript edition MUST NOT silently mutate the semantics of an
existing edition profile. Edition selection remains explicit.

### New Browser Adapters

Adding a browser adapter MUST NOT change the meaning of Standard Qualification.
Browser-specific differences enter through qualified evidence and capability
contracts.

### New analysis capabilities

Adding a capability MAY introduce new derived claims or refinements but MUST NOT
silently redefine existing claim semantics owned by another capability.

### Better precision

A more precise solver or analysis strategy MAY produce stronger results under a
new or more precise analysis envelope. It MUST NOT silently mutate the meaning of
historical claims.

### Representation replacement

Internal AST/IR, abstract domains, graphs, stores, algorithms, or physical crate
placement MAY be replaced if the replacement preserves the accepted semantic
contracts and compatibility obligations that apply at that time.

## Deferred Research

The following remain focused follow-up research rather than blockers to this
architecture baseline:

- ES2026 host-dependent Early Error inventory;
- Annex B applicability and Normative Optional groups;
- Script/Module and other grammar-goal interactions;
- direct-eval and function-construction qualification context;
- implementation-extension boundaries;
- later-draft / later-edition deltas;
- Test262 filtering/profile alignment;
- concrete qualification-profile selection;
- exact module identity / host-resolution integration;
- runtime observation-channel completeness and source/runtime correlation;
- shared-memory analysis objectives; and
- WeakRef/GC/liveness analysis ownership and abstraction.

A follow-up may refine these open details. It MUST NOT silently reopen JA-1 through
JA-12 unless new evidence actually contradicts an invariant. A material change to
these invariants or the four responsibility boundaries requires the normal
architecture decision process and, when applicable, a new or superseding ADR.

## Implementation Gate

Acceptance of this architecture establishes a durable semantic baseline. It does
not approve:

- production ECMAScript implementation;
- Rust type or API design;
- crate/module decomposition;
- parser/solver dependencies;
- Browser Adapter implementation;
- serialization;
- async/concurrency runtime selection; or
- release/public compatibility commitments.

Each such change requires focused approved scope and all applicable repository
contracts.

## Validation Expectations

Future JavaScript implementation or architecture changes MUST be reviewed against,
at minimum:

1. browser independence;
2. exact source/provenance authority;
3. explicit qualification profile/context;
4. claim-specific prerequisites;
5. semantic authority ownership;
6. hidden/enclosing control and effect semantics;
7. qualified host/runtime evidence boundaries;
8. scoped lifecycle/completeness;
9. negative-query correctness;
10. claim modality/applicability;
11. provenance/conflict preservation; and
12. representation neutrality unless a separate approved decision intentionally
    narrows it.

## References

- [ADR 0009 — Define JavaScript Semantic Analysis Architecture](../decisions/0009-javascript-semantic-analysis-architecture.md)
- [JavaScript research evidence](../evidence/javascript/README.md)
- [Architecture Principles](PRINCIPLES.md)
- [Architecture Layers and Boundaries](LAYERS.md)
- [Rust Core Contracts](RUST_CORE_CONTRACTS.md)
- [Source Parser Ownership](SOURCE_PARSER_OWNERSHIP.md)
- [ADR 0007 — Own Lossless Source Parsers](../decisions/0007-own-lossless-source-parsers.md)
- [ADR 0008 — Define Browser Runtime Evidence Normalization and Core Import Ownership](../decisions/0008-browser-runtime-evidence-normalization-and-core-import.md)
- Issue #108 — ECMAScript parser/static-semantics program
- Issue #142 — ECMAScript complete-analysis guarantee research
- Issue #144 — ECMAScript language-profile research
- Issue #187 — JavaScript Architecture Model v1.1 freeze and approval
