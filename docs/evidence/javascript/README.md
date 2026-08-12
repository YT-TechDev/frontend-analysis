# JavaScript / ECMAScript Research Evidence

Status date: 2026-08-12

Classification: task and evidence record; non-normative.

## Current Status

The current JavaScript / ECMAScript architecture research phase has completed a
deep adversarial evidence audit covering state, control, effects,
interprocedural analysis, async execution, modules, Realms/Agents/shared memory,
meta-object semantics, WeakRef/finalization, and iterator cleanup.

Final audit recommendation:

```text
Evidence sufficiently stable — proceed to Architecture Model consolidation
```

The audit evaluated 17 cross-cutting candidate principles:

- **6** survived without further qualification;
- **11** survived with qualification;
- **0** required complete candidate-level abandonment.

This result authorizes architecture consolidation only. It does **not** authorize
ECMAScript production implementation, public APIs, Rust representation choices,
parser algorithms, abstract domains, or browser/runtime integration.

The final audit found and corrected an edition-discipline defect: evidence from
the current post-2026 draft must not be represented as ECMA-262 2026 normative
evidence. Explicit Resource Management is therefore isolated from the ES2026
normative set and may be researched separately as post-2026/current-draft
evidence.

## Evidence Baseline and Authority

Primary normative baseline:

```text
ECMA-262, 17th edition, June 2026
```

Relevant durable repository program records:

- [#104 — project-owned lossless parser architecture](https://github.com/YT-TechDev/frontend-analysis/issues/104)
- [#108 — ECMAScript parser/static-semantics program](https://github.com/YT-TechDev/frontend-analysis/issues/108)
- [#142 — project-owned ECMAScript analysis guarantee contract](https://github.com/YT-TechDev/frontend-analysis/issues/142)
- [#144 — project-owned ECMAScript language profile model](https://github.com/YT-TechDev/frontend-analysis/issues/144)

Important immutable research identities already recorded by the ECMAScript
program include:

```text
original ES2026 tag target:
0248456c758431e4bb8e5d26333ff1865123c9cd

ES2026 errata tag target:
d89c03f2db8a597bc915b363a6518d0cc8acdbc0

pinned Test262 evidence revision used by the earlier audit:
be13516fb6441b950ba8a3df97eb34062c186972
```

Evidence authority remains:

```text
selected immutable ECMA-262 / referenced normative standards
        ↓
pinned Test262 evidence within its scope
        ↓
project-owned gold / generated / adversarial evidence
        ↓
external parsers, engines, and research tools as differential observations
```

No external parser, engine, or Test262 metadata convention defines the project
language profile by itself.

## Evidence Classification

The final research audit uses these evidence classes:

- **N — Normative ECMAScript evidence**
- **H — Normative host/Web evidence**
- **T — Program-analysis / Abstract Interpretation theory**
- **J — JavaScript-analysis empirical or published-system evidence**
- **A — Frontend Analysis architecture inference**

An architecture inference must not be presented as though it were an ECMA-262
normative fact.

## Cross-Cutting Evidence Core

The following principles survived repeated adversarial revalidation. They are
research evidence candidates for architecture consolidation, not public type or
module commitments.

### E1 — Concrete semantics and static-analysis abstraction are distinct layers

Concrete ECMAScript semantics must remain distinguishable from
abstract-interpretation/static-analysis semantics.

Important terminology rule:

```text
ECMA-262 Abstract Operation
!=
Abstract Interpretation operation
```

A specification algorithm named an “Abstract Operation” is not evidence that
Frontend Analysis should expose or implement an abstract domain.

### E2 — State dimensions may be semantically distinct without being independent

Binding/environment semantics, object/property semantics, Realm state,
execution-context state, Agent state, agent-cluster shared state, and
analysis-side abstract state expose distinctions that can affect meaning.

However:

```text
semantically distinct
!= independent
!= orthogonal
!= separately stored
```

Distinct dimensions may compose, reference, or constrain one another.

### E3 — Control dimensions are distinct but interacting

Completion classification, pending control transfer, call/return,
suspension/resumption, Promise/Job behavior, and host scheduling cannot be
collapsed into one universal control concept.

Examples established by normative semantics include:

- generators suspend and later resume an existing evaluation;
- async `await` suspends/resumes rather than behaving as return + new call;
- generator resumption can carry a Completion Record;
- async function body `return`/`throw` is mediated through Promise settlement;
- `finally`, `IteratorClose`, and `AsyncIteratorClose` can process an already
  pending abrupt completion; and
- async iterator cleanup can itself suspend before final propagation.

Therefore:

```text
pending control transfer
!= finalized control transfer
```

### E4 — Surface syntax is not a universal effect oracle

Source syntax alone cannot universally determine effectfulness.

Counterexamples include:

- accessors;
- Proxy internal-method traps;
- coercion/conversion hooks;
- Reflect operations;
- iterator acquisition/stepping/result inspection/closing; and
- calls whose reachable state is not limited to explicit arguments.

This does **not** mean syntax can never justify purity for a bounded operation,
and it does not forbid a future lowered IR from establishing stronger purity
invariants after hidden effects have been made explicit.

### E5 — Semantic relations must not be conflated because they can share graph storage

CFG edges, possible-call relations, module dependencies, async dependencies,
memory-model relations, host scheduling relations, and analysis-side relations
may all be representable as graphs, but graph shape does not determine semantic
ownership.

Distinct relations may also overlap, derive from, include, or constrain one
another. For example, ECMAScript shared-memory `happens-before` is related to
agent-order and synchronization relations.

Therefore:

```text
semantic relation != separate physical graph
```

and also:

```text
one physical graph != one semantic relation
```

### E6 — Sensitivity is an analysis precision strategy, not ECMAScript semantics

Path, context, heap, and related partitioning/sensitivity strategies alter which
abstract states remain distinct and therefore affect precision and cost.

They are not themselves ECMAScript concrete semantics.

Any statement about “soundness” must state its envelope, including the relevant:

- concrete semantics;
- supported language/profile features;
- dynamic-code policy;
- host assumptions;
- analysis objective;
- abstraction/sensitivity policy; and
- resource/termination assumptions where material.

Unqualified project-wide claims such as “Frontend Analysis is sound” are not
currently justified.

### E7 — Loss of exact information does not universally require loss of all coarser facts

Research counterexamples show that unknown exact property keys, possible
callees, aliases, or Proxy behavior do not universally force an entire analysis
state to collapse to one Top value.

However this principle does not guarantee that useful coarse information always
survives. What can be retained depends on the analysis domain and operation.

### E8 — Identity, liveness, reachability approximation, and physical lifetime are distinct

Source-level existence, runtime object identity observability, normative WeakRef
liveness, implementation reachability approximation, and physical reclamation
must not be treated as one lifecycle state.

ECMAScript WeakRef liveness is not simply an implementation heap-reachability
predicate. Implementations may use reachability as a conservative approximation,
and the specification permits constrained implementation latitude around
collection/finalization.

### E9 — Host boundary is an integration/ownership boundary, not semantic isolation

ECMAScript defines host integration contracts for areas such as Promise Jobs,
module loading, finalization scheduling, Agent suspension capability, and
cross-agent synchronization.

The host supplies behavior/evidence under those contracts. Browser-specific
realization must not be relabeled as intrinsic ECMAScript behavior.

Therefore:

```text
ECMAScript normative contract
        ↕
host-defined realization / evidence
```

is a better model than a completely isolated wall.

### E10 — Semantic distinctions do not prescribe one physical representation

The evidence constrains what an implementation must be capable of representing.
It does not currently justify one canonical decomposition into:

- Rust structs/enums;
- separate stores;
- lattices/product domains;
- graphs;
- state machines;
- crates; or
- public APIs.

The reverse claim is also unsupported: representation neutrality does not mean
all concerns should be merged into one structure.

## Meta-Invariants Surviving Cross-Batch Audit

### M1 — Distinction does not imply independence

`A != B` does not imply that A cannot depend on or interact with B.

### M2 — Boundary does not imply isolation

An ownership boundary may still expose explicit semantic integration contracts.

### M3 — Relation does not imply separate graph

Semantic relation identity does not prescribe storage architecture.

### M4 — Composition does not imply conflation

Two semantic concerns may be represented in one composite structure while
remaining different concepts.

### M5 — Precision loss does not imply mandatory total knowledge loss

Exact knowledge may be lost while coarser knowledge remains possible, but this
is not a universal guarantee for every domain.

### M6 — Normative specification concept does not imply public architecture type

The specification's distinctions among concepts such as Realm, Agent,
Completion Record, Environment Record, or Module Record do not by themselves
justify public Rust types with matching names or decomposition.

### M7 — Pending control transfer does not imply finalized control transfer

`return`, `throw`, `break`, and `continue` may still be processed by surrounding
semantics such as `finally`, `IteratorClose`, or `AsyncIteratorClose` before the
ultimate propagation result is determined.

## Domain Evidence from the Z–AH Research Sequence

The final cross-batch audit reconstructed and challenged the surviving evidence
from the research sequence. The original batch-by-batch conversational transcripts
were not repository artifacts at audit time; this document therefore preserves
the independently consolidated conclusions rather than claiming a verbatim
archive of every intermediate statement.

### State / Abstract Interpretation

Surviving evidence includes:

- binding/environment semantics and object/property semantics are normatively
  distinguishable;
- closures must not generally be modeled as immutable capture-time value
  snapshots;
- runtime identifier resolution can depend on dynamic environment state without
  invalidating static source qualification;
- analysis join/merge is not an ECMAScript runtime transition;
- one CFG/control location need not correspond to exactly one abstract state;
- CFG convergence does not universally force abstract-state convergence;
- path/context/heap sensitivity are precision strategies rather than concrete
  semantics;
- exact-key uncertainty does not universally require loss of all property facts;
- update precision may depend on location/alias certainty rather than assignment
  syntax alone; and
- no canonical universal lattice, Unknown taxonomy, update representation, or
  abstract-state decomposition is justified yet.

### Calls / Interprocedural Analysis

Surviving evidence includes:

- unresolved call targets do not universally require whole-state/whole-heap
  Top/havoc;
- call effects are not bounded solely by explicit argument syntax;
- return-value knowledge and call-effect knowledge are different dimensions;
- interprocedural analysis does not require one canonical function-summary
  representation;
- recursion is repeated call reachability, not a special ECMAScript call kind;
- finite static analysis may merge/abstract recursive histories without one
  mandated algorithm such as widening or k-CFA; and
- escape/invalidation/call-summary representations remain OPEN.

### Dynamic Code

Surviving evidence includes:

- direct `eval` has specification-defined behavior distinct from an ordinary
  unknown call and from indirect eval;
- uncertainty over existing callable targets differs from uncertainty introduced
  by dynamically parsed executable source;
- dynamic `Function` construction must not be conflated with ordinary lexical
  closure creation; and
- dynamic-code support requires an explicit analysis policy/envelope, while full
  static reconstruction is not the only possible strategy.

### Async / Promise / Jobs / Suspension

Surviving evidence includes:

- completion, suspension, resumption, Promise settlement, Job scheduling, and
  host task/microtask realization are distinct concepts;
- `await` suspends/resumes an existing async evaluation;
- generator `yield` is suspension rather than function completion;
- Promise settlement does not synchronously invoke reaction handlers;
- an ECMAScript Promise Job is not universally identical to a browser
  microtask; and
- browser microtask queues/event-loop checkpoints belong to host semantics.

### Modules

Surviving evidence includes:

- a syntactic ModuleRequest and a host-resolved Module Record are distinct;
- raw specifier text alone is not a canonical universal module identity;
- loading, linking, and evaluation are distinct stages;
- circular module dependencies are supported graph structures, not inherently
  errors;
- a module without syntactic top-level `await` may participate in asynchronous
  evaluation because of dependencies;
- static import relations and runtime dynamic-import reachability are distinct;
  and
- browser module resolution/fetch policy belongs to host semantics under the
  ECMAScript integration boundary.

### Realms / Agents / Shared Memory

Surviving evidence includes:

- Realm, global object, global environment, execution context, Agent, and Agent
  Cluster are distinct concepts;
- Agent identity must not be equated canonically with a physical OS thread;
- per-Agent execution state and agent-cluster shared-memory state are distinct;
- communication reachability and shared-memory reachability are different
  relations;
- the shared-memory model adds axiomatic memory-event constraints beyond a
  universal single sequential state-transition model;
- atomic and non-atomic shared-memory accesses have different ordering
  guarantees;
- ECMAScript data races do not mean C/C++-style undefined behavior;
- data-race freedom does not imply unique deterministic execution; and
- host synchronization can participate in memory-model ordering.

### Proxy / Reflect / Exotic Objects

Surviving evidence includes:

- ordinary-object semantics are not universal object semantics;
- ordinary, exotic, and user-interposed behavior are distinct;
- surface object operations dispatch through essential internal methods;
- Proxy interception can invoke user code without explicit call syntax at the
  operation site;
- Proxy traps remain constrained by ECMAScript invariants rather than being
  arbitrary semantics;
- trap completion and enclosing internal-method completion are distinct;
- Reflect operations are not intrinsically pure metadata inspection;
- property target and receiver are distinct concepts; and
- property state, internal-slot state, and private-element state must not be
  assumed to share one semantic namespace.

### WeakRef / Finalization / GC

Surviving evidence includes:

- normative WeakRef liveness and implementation graph reachability are distinct;
- non-liveness does not imply immediate or guaranteed collection;
- non-liveness does not imply immediate mandatory WeakRef clearing;
- successful `deref()` creates bounded synchronous kept-alive guarantees;
- FinalizationRegistry registration does not guarantee an eventual callback;
- cleanup eligibility, host scheduling, and user callback execution are distinct;
- finalization callbacks do not arbitrarily preempt synchronous ECMAScript
  execution; and
- specification-permitted implementation latitude must not be conflated with
  analysis-induced imprecision.

### Iterator / Cleanup Semantics

Surviving evidence includes:

- iteration is protocol-driven dispatch rather than mere traversal of stored
  values;
- iterator acquisition, stepping, `done`/`value` inspection, and closing may
  invoke user code;
- consuming zero iterator elements does not imply zero iterator effects;
- iterator exhaustion and iterator closing are distinct events;
- abrupt control transfer may trigger cleanup before final propagation;
- cleanup may preserve or replace a pending Completion according to
  operation-specific precedence rules;
- `for...in`, `for...of`, and `for await...of` must not be assumed to share
  identical protocol/cleanup semantics;
- async iteration can adapt a synchronous iterator; and
- async iterator cleanup may introduce suspension while an abrupt completion is
  already pending.

## Strong Claims Explicitly Rejected or Downgraded

Future architecture or implementation work must not silently reintroduce these
claims as assumptions:

- `abstract state = variable -> abstract value` is universally sufficient;
- object heap alone is a complete ECMAScript execution-state model;
- closure capture means immutable capture-time value snapshot;
- every identifier has one runtime target permanently fixed solely by lexical
  syntax;
- analysis join is a runtime ECMAScript event;
- every CFG merge requires exactly one abstract state;
- path sensitivity is inherently required for soundness;
- maximal context/path sensitivity is always best;
- one undifferentiated canonical Unknown taxonomy is uniquely justified;
- unknown exact property key requires loss of all property knowledge;
- unknown call requires whole-heap Top;
- assignment syntax alone determines strong replacement;
- invalidation is canonically `AbstractValue -> Top`;
- one universal lattice/domain decomposition is already justified;
- worklist/fixed-point/widening is a mandatory architecture representation;
- call summaries are mandatory interprocedural architecture;
- recursive calls require a special ECMAScript call kind;
- `eval` is merely another unknown call;
- `await` is equivalent to function return;
- suspended execution is completed execution;
- Promise settlement synchronously invokes handlers;
- Promise Job is universally identical to microtask;
- circular module dependency is inherently invalid;
- a module without syntactic TLA is always synchronously evaluated;
- Realm equals global object;
- Agent equals execution context or physical OS thread;
- all shared-memory access is sequentially consistent;
- JavaScript data race means undefined behavior;
- property read/write is intrinsically a simple pure heap operation;
- Proxy supplies arbitrary unrestricted semantics;
- Reflect is intrinsically pure reflection;
- WeakRef/GC non-liveness implies immediate collection;
- FinalizationRegistry callback is guaranteed eventually;
- iterator cleanup always preserves the original Completion;
- iterator cleanup always replaces the original Completion; and
- normative specification concepts should automatically become matching public
  Rust types.

## Minimal Architecture Invariants Supported by the Final Audit

The audit reduced the project-specific architecture candidates to eight minimal
principles. They are evidence-backed consolidation inputs and still require the
normal maintainer process before becoming normative architecture contracts.

### FA-1 — Edition- and layer-qualified evidence

Every load-bearing claim identifies its specification edition/snapshot and
whether it is normative, host, theory/empirical, or project inference.

### FA-2 — Standard Qualification is a scoped source capability

A Standard Qualification capability may establish source/profile/grammar/static
-semantic facts within its explicit context without claiming external module
resolution or runtime success.

```text
source-standard-qualified
!= loadable
!= link-valid
!= evaluation-safe
```

### FA-3 — Preserve capability-relevant semantic distinctions

If two concepts can change an analysis result, the architecture must be capable
of representing that distinction. This does not require separate physical types
or stores.

### FA-4 — Host/browser facts enter Core through qualified contracts

Browser-specific realization or observation must retain host/profile/provenance
identity and cannot silently become intrinsic ECMAScript truth.

### FA-5 — Control/effect models admit hidden invocation and enclosing semantics

Architecture must leave room for accessors, Proxy, coercion, iterator protocols,
cleanup, suspension/resumption, and other semantics that source shape alone may
not expose.

### FA-6 — Relation meaning and derivation provenance survive storage choices

If CFG/call/module/async/memory/evidence relations share storage, their semantic
kind, direction, origin, derivation, assumptions, and query meaning must remain
recoverable.

### FA-7 — Analysis claims carry an uncertainty/soundness envelope

A claim must expose enough context to understand its objective, profile/edition,
feature coverage, dynamic-code and host assumptions, abstraction/precision
policy, resource limits, and inferred/observed status where material.

### FA-8 — Runtime identity/liveness/scheduling/concurrency remain observationally qualified

Normative possibility, host behavior, implementation approximation, and observed
runtime event are not interchangeable evidence categories.

## Representation OPEN Set

The final audit intentionally does **not** freeze a canonical representation for
areas including:

- abstract-state/domain decomposition;
- lattice/product structure;
- Unknown taxonomy;
- heap/object abstraction;
- strong/weak update representation;
- call graph/context representation;
- call summaries;
- invalidation and escape analysis;
- completion/control-flow representation;
- async state-machine or callback graph representation;
- effect taxonomy;
- relation/graph storage;
- module identity schema;
- host-profile/runtime-evidence interfaces;
- concurrency/memory-event representation;
- WeakRef/GC/lifetime representation;
- iterator cleanup representation;
- crate/module topology;
- public Rust types; and
- serialization/wire formats.

A later design may select one of these only after the owning capability and
compatibility boundary justify it.

## Current Architecture-Consolidation Direction

The evidence currently supports a responsibility model in which:

```text
Source + provenance + explicit qualification context
        ↓
ECMAScript Standard Qualification
        ↓
source-backed qualified facts
        ↓
optional analysis capabilities
(binding / control / effect / value / interprocedural / module / async / concurrency ...)
        ↓
qualified evidence / analysis results
        ↑
qualified host/runtime evidence
        ↑
browser or other host adapters
```

This is a semantic responsibility model, not a crate diagram or approved public
API.

Standard Qualification should answer what source-standard facts are justified
under an explicit edition/profile/grammar/source context. It should not silently
become a whole-program evaluator, module loader, abstract interpreter, browser
event loop, or runtime engine.

## Remaining Research / Architecture Risks

Material remaining questions include:

- exact Standard Qualification public/internal contract;
- immutable ECMAScript language-profile selection and evolution;
- complete static-semantics/Early Error coverage model;
- dynamic-code analysis policy by capability;
- host profile and runtime-evidence contract;
- module resolution identity and host integration;
- analysis-objective-specific soundness envelopes;
- async scheduling analysis scope;
- shared-memory/concurrency analysis scope;
- WeakRef/lifetime analysis scope;
- physical representation and crate placement; and
- public compatibility only after a named consumer exists.

These are architecture-consolidation questions or future capability research;
they are not evidence that the current foundation must be discarded.

## Final Evidence Boundary

The current JavaScript evidence is sufficiently stable to continue Architecture
Model consolidation. The next work must continue architecture-first and preserve
the OPEN set.

No code implementation, Issue decomposition for production, parser algorithm,
public API, or Rust type hierarchy is authorized merely by this evidence record.
