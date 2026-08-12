# ADR 0009: Define JavaScript Semantic Analysis Architecture

| Field | Value |
| --- | --- |
| Status | Proposed |
| Date | 2026-08-12 |
| Decision owner / approver | YT-TechDev — approval pending |
| Linked Issue | #187 |
| Related Pull Request | None (proposal PR pending) |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | Proposed new `docs/architecture/JAVASCRIPT_ARCHITECTURE.md`; `docs/README.md` authority map on acceptance. Existing `PRINCIPLES.md`, `LAYERS.md`, `RUST_CORE_CONTRACTS.md`, `SOURCE_PARSER_OWNERSHIP.md`, ADR 0007, and ADR 0008 remain unsuperseded. |

## Context

Frontend Analysis needs a durable JavaScript / ECMAScript semantic architecture before production ECMAScript implementation begins. The repository already owns the language frontend direction under ADR 0007, project-owned parser/static-semantics coordination under #108, the complete-analysis guarantee research under #142, the language-profile research under #144, and browser-runtime evidence normalization under ADR 0008.

The JavaScript research phase then expanded across control/completion, value and abstract interpretation, effects and hidden invocation, interprocedural analysis, dynamic code, async/Promise/Job semantics, modules, Realms/Agents/shared memory, Proxy/exotic objects, WeakRef/finalization, and iterator cleanup. The consolidated evidence is recorded in `docs/evidence/javascript/README.md`.

The final adversarial evidence audit and cross-contract audit established that the architecture must preserve semantic distinctions without prematurely requiring separate Rust representations. It also established two material qualifications before this proposal:

1. source qualification may require an explicit standardized host / Normative Optional qualification profile, including applicable Annex B source/static-semantics policy; and
2. useful analysis of invalid, recovered, incomplete, or partial source does not authorize normative execution claims whose required validity prerequisites are unsatisfied.

Further ES2026 host-dependent Early Error, Annex B, and qualification-profile detail research is intentionally deferred. That research may refine the profile model but must not silently reopen this architecture unless evidence falsifies an invariant.

This decision is required because leaving Standard Qualification, semantic-capability ownership, runtime-evidence consumption, result provenance, and lifecycle semantics implicit would cause future JavaScript work to repeatedly reopen the same boundaries and could couple Core to one parser, browser, analysis framework, graph model, or abstract domain.

## Decision

Frontend Analysis adopts JavaScript Architecture Model v1.1 as the proposed durable semantic architecture, subject to explicit maintainer approval and the acceptance update that creates the active specialized normative contract.

The architecture has four conceptual responsibility boundaries:

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

This is a semantic responsibility and information-flow model. It is not a crate diagram, mandatory execution pipeline, public API, AST/IR design, graph framework, solver choice, or serialization contract.

The decision freezes twelve durable invariants:

1. **JA-1 — Evidence and qualification are explicitly profiled.** Material claims preserve enough edition/profile/context/evidence information to recover their applicable semantic envelope.
2. **JA-2 — Standard Qualification is bounded and context-qualified.** It owns source-standard qualification under explicit edition/profile/goal/context, including standardized host/Normative Optional qualification policy where required, but does not use runtime success as source-standard authority.
3. **JA-3 — Downstream analysis is claim-prerequisite-driven.** Whole-source qualification success is not a universal downstream gate, but execution claims cannot outlive validity prerequisites required for that execution.
4. **JA-4 — Semantic authority is explicit.** Semantic authority, evidence origin, computation, storage, and result transport remain distinct. Circular definition of semantic authority is prohibited.
5. **JA-5 — Capability decomposition is semantic, open, and representation-neutral.** Binding, Control, Effect, Value, Interprocedural, Module, Async, Concurrency, and future capability labels are responsibilities, not required physical modules or types.
6. **JA-6 — JavaScript control and effects admit hidden and enclosing semantics.** Architecture must support accessors, Proxy/exotic behavior, coercion, iterators, unknown calls, cleanup, pending completion, suspension/resumption, and related hidden semantics without using surface call syntax as a universal effect oracle.
7. **JA-7 — Host/runtime evidence crosses a qualified browser-independent boundary.** ECMAScript semantics, host contracts, host-standard realization, browser realization, runtime observation, and Core-derived interpretation remain distinguishable. ADR 0008 remains authoritative for runtime-evidence normalization/import ownership.
8. **JA-8 — Runtime evidence is scoped and observationally qualified.** Origin, applicability lifetime, completeness/lossiness, correlation basis, and relevant ordering meaning remain recoverable. `not observed` does not automatically mean `did not happen`.
9. **JA-9 — Claims carry a semantic applicability envelope.** Material analysis claims preserve enough objective, feature, dynamic-code, host, precision, resource, dependency, and evidence-origin context to understand what is actually justified.
10. **JA-10 — Lifecycle and claim modality remain distinct.** Capability/run completeness does not collapse into claim certainty, and partial/unsupported/resource-limited states do not become successful zero-result or ambiguous global `Complete`.
11. **JA-11 — Provenance and conflicts survive derivation.** Derived claims remain traceable to sufficient supporting evidence; authored ranges are not fabricated for derived semantics; genuine evidence conflicts remain representable rather than silently overwritten.
12. **JA-12 — Semantic distinctions constrain expressiveness, not physical representation.** AST/IR, abstract domains, heap models, CFG/call/module/evidence graphs, solvers, result types, crates, traits, public APIs, and serialization remain open until separately justified.

### Standard Qualification boundary

Standard Qualification answers which source-standard facts are justified for exact retained source under an explicit ECMAScript edition/profile, grammar goal/source kind, and required qualification context without executing the program or resolving its external runtime environment.

It may own lexical/grammar qualification, required source-qualification Static Semantics and Early Errors, source-established declarations/relations, source-level ModuleRequest/import/export facts within supported scope, diagnostics, and qualification lifecycle.

Parser or syntax-tree production alone does not establish complete ECMAScript validity when required static semantics or profile checks remain unverified. External parsers and browser engines remain differential/research evidence, not Frontend Analysis semantic authority.

### Semantic Analysis Capability boundary

A semantic capability owns a bounded semantic question, the prerequisites required for its claims, its claim semantics, analysis assumptions, and uncertainty/completeness meaning. Capabilities consume declared semantic facts rather than unrestricted internals of unrelated capabilities.

Cyclic computational refinement is permitted when explicitly bounded and when participants, initial approximation, refinement semantics, termination/resource policy, and incomplete-result semantics are defined. This decision does not select fixed-point, worklist, widening, context, summary, or other solver machinery.

### Host / Runtime Evidence boundary

ADR 0008 remains authoritative for Browser Adapter capture ownership, target-lifetime evidence, runtime-source identity, `SourceId` authority, native-coordinate termination, Application Orchestration, and Core-facing import validation.

This decision specializes only the JavaScript semantic consumption rule: normalized runtime evidence may refine or correlate JavaScript analysis while retaining host/runtime provenance, but browser observations cannot silently become intrinsic ECMAScript truth and browser brand does not become Core semantic branching.

Runtime evidence remains optional. Source-only Standard Qualification and static analysis remain possible without a live browser when the selected capability permits.

### Qualified Analysis Result boundary

Analysis Results carry qualified claims rather than unqualified value bags. Material roles such as observation, normative claim, inference, finding, diagnostic, modality, applicability, lifecycle, supporting evidence, derivation provenance, and conflict relation remain semantically distinguishable where meaning changes.

A negative claim requires completion of the relevant negative query under sufficient prerequisites. Derived semantics may cite authored source evidence but cannot fabricate authored source ranges for non-authored semantic events. Invalidated required evidence triggers re-evaluation of dependent claims; independently sufficient retained support may preserve a claim.

### Representation boundary

This decision intentionally leaves open all physical representation choices listed in the JavaScript evidence record, including AST/CST/IR shape, state/lattice decomposition, heap/location abstraction, Unknown taxonomy, strong/weak update representation, CFG/Completion/effect representation, call/module/async/concurrency/evidence graph storage, WeakRef/liveness representation, evidence invalidation storage, result structs, crate/module topology, traits, public APIs, serialization, and async/concurrency implementation.

## Alternatives Considered

### Make one parser/AST the JavaScript architecture

Not selected. Grammar acceptance alone does not establish required static-semantic validity, language-profile membership, host/runtime meaning, effect/value semantics, or qualified result provenance. It would also create premature representation coupling.

### Define a fixed sequential analysis pipeline

Not selected. Binding, Control, Effect, Value, Interprocedural, Module, Async, and other analyses have capability-specific prerequisites and may have cyclic computational refinement. A mandatory `parse -> bind -> CFG -> effects -> values -> calls` pipeline would encode unsupported dependency assumptions.

### Build one universal abstract interpreter first

Not selected. Abstract interpretation is an optional semantic capability/strategy, not ECMAScript concrete semantics. Different product objectives may require different analyses, abstractions, precision strategies, or no abstract interpretation at all.

### Let browser runtime behavior define JavaScript semantics

Rejected. Browser engines remain authoritative for their observed behavior, not for intrinsic ECMAScript source-standard qualification. This would violate Core browser independence and collapse normative, host, implementation, and observation evidence.

### Freeze separate crates/types/graphs for every semantic distinction

Rejected. The evidence requires semantic distinctions to remain representable but does not justify one physical decomposition. This would violate the representation-neutral result of the final audit.

### Defer all JavaScript architecture until after implementation

Not selected. The current evidence is sufficient to freeze long-lived responsibility, authority, lifecycle, provenance, and host-integration boundaries. Deferring them would make implementation choices accidentally define architecture and force repeated redesign.

### Continue qualification-detail research before freezing any architecture

Not selected. ES2026 Annex B / host-dependent Early Error / Normative Optional details remain important, but v1.1 now contains the correct qualification-profile extension point. Continuing that research as a blocker would mix architecture consolidation with an edition-specific qualification inventory and risk reopening already stable boundaries without contradictory evidence.

## Consequences

### Positive

- JavaScript Core semantics remain browser independent.
- Source qualification, static semantic analysis, runtime evidence, and qualified result semantics gain explicit ownership boundaries.
- Parser acceptance cannot silently become complete ECMAScript validity.
- Invalid/partial source can still support useful bounded analysis without inventing normative execution claims.
- Hidden invocation, pending completion, async cleanup, module/host boundaries, concurrency relations, and WeakRef/liveness uncertainty remain architecturally expressible.
- Future analysis capabilities can be added incrementally without forcing all consumers through one universal analyzer.
- Runtime evidence can enrich static analysis without contaminating source-standard authority.
- Evidence provenance and lifecycle remain first-class semantic concerns.
- Major representation choices remain replaceable and can be justified by actual consumers later.

### Negative

- Future implementations must carry more explicit lifecycle, qualification, provenance, and uncertainty information than a simple parser/AST/result list design.
- Capability boundaries require deliberate semantic contracts rather than informal sharing of internal state.
- Hybrid static/runtime analysis requires evidence qualification and correlation discipline.
- Some analyses will need explicit partial/unsupported/resource-limited behavior instead of convenient empty results.
- Representation choices are intentionally deferred, so additional focused architecture work is still required before public APIs or large implementation slices.

### Risks

The architecture could be over-interpreted as requiring one physical component per capability. Mitigation: JA-5 and JA-12 explicitly prohibit that inference.

The qualification profile could become a browser-brand switch. Mitigation: standardized qualification policy remains declarative and distinct from live runtime evidence; browser-specific observations enter through the runtime evidence boundary.

The result model could become a universal evidence mega-structure. Mitigation: only semantic distinctions and traceability are frozen; schema and storage remain open.

Deferred ES2026 qualification research may discover more context dimensions. Mitigation: JA-1 through JA-3 require explicit qualification context and claim prerequisites without freezing a closed context enum. A real contradiction requires reopening the architecture through the normal decision process.

### Reversibility

Physical implementation remains highly reversible because this ADR selects no crates, public APIs, AST/IR, graph schema, abstract domain, solver, serialization, or runtime library.

Changing JA-1 through JA-12, the four responsibility boundaries, semantic authority rules, browser-independent host/runtime boundary, or scoped lifecycle/provenance semantics would be a material architecture change and require a new ADR or explicit supersession.

## Compatibility and Migration

No current production Rust API, workspace member, dependency, serialized format, browser protocol, ABI, or product contract changes merely by proposing or accepting this decision.

Accepted ADR 0007 remains authoritative for project-owned lossless source parser strategy and sequencing. Accepted ADR 0008 remains authoritative for browser-runtime evidence normalization/import ownership and is not superseded.

On acceptance, `docs/architecture/JAVASCRIPT_ARCHITECTURE.md` becomes the specialized normative JavaScript semantic architecture contract and `docs/README.md` is updated to register its authority. Existing architecture contracts remain authoritative for their broader or Rust-specific topics.

Future ECMAScript implementation work must conform to the accepted JavaScript contract but requires separately approved focused Issues. No migration of production code is required at acceptance time because production ECMAScript implementation has not begun.

## Security and License Impact

No dependency, browser protocol library, unsafe Rust, FFI, process boundary, serialization, or license change is selected.

JavaScript source and runtime evidence are externally influenced data. Future implementation remains subject to `docs/development/SECURE_DEVELOPMENT.md`, `docs/architecture/RUST_CORE_CONTRACTS.md`, and existing untrusted-input, panic, resource, provenance, and logging requirements.

The repository remains MIT licensed. Third-party parser, analysis-library, browser-protocol, or runtime-library licensing remains a future dependency decision.

## Validation

Before this ADR may become Accepted, review must demonstrate:

1. #187 contains explicit durable maintainer approval for this exact decision;
2. JA-1 through JA-12 match the final v1.1 architecture audit and include the host-qualification-profile and invalid-source claim-prerequisite corrections;
3. the four responsibility boundaries are semantic responsibilities, not physical module requirements;
4. `PRINCIPLES.md`, `LAYERS.md`, `RUST_CORE_CONTRACTS.md`, and `SOURCE_PARSER_OWNERSHIP.md` are not contradicted;
5. ADR 0007 and ADR 0008 remain accepted and unsuperseded;
6. browser-specific runtime/protocol details do not become Core semantic authority;
7. whole-source qualification success is not a universal analysis gate, while execution claims still require their validity prerequisites;
8. capability lifecycle and claim modality remain distinct;
9. negative findings cannot be inferred from missing/incomplete analysis;
10. evidence conflicts and provenance remain representable;
11. no AST/IR/lattice/graph/solver/crate/trait/public API/serialization/async/concurrency implementation is frozen;
12. the acceptance change creates or updates the specialized normative JavaScript architecture contract and documentation authority map consistently; and
13. no production Rust/dependency/workspace change is introduced by architecture acceptance.

Use only repository-approved validation status vocabulary.

## Follow-Up

After acceptance, separately plan and approve:

1. focused ES2026 qualification delta / host-dependent static-semantics research covering Annex B, Normative Optional policy, goal/context interactions, later-draft deltas, Test262 filtering, and concrete qualification-profile selection;
2. first bounded Standard Qualification implementation architecture under #108;
3. capability-specific implementation planning only after the owning semantic contract is sufficiently precise;
4. exact module identity / host-resolution integration when a real consumer requires it;
5. runtime observation-channel completeness and correlation work;
6. shared-memory and WeakRef/liveness capability research where product objectives justify it; and
7. public API, serialization, crate decomposition, or async/concurrency decisions only when named consumers and compatibility boundaries exist.

No follow-up implementation begins automatically from this ADR.

## Approval

Pending explicit maintainer approval on #187.

`Proposed` status authorizes no production implementation and is not an active normative JavaScript contract.

## References

- Issue #187 — JavaScript Architecture Model v1.1 freeze proposal
- Issue #108 — project-owned ECMAScript parser/static-semantics program
- Issue #142 — project-owned ECMAScript analysis guarantee research
- Issue #144 — project-owned ECMAScript language-profile research
- `docs/evidence/javascript/README.md`
- `docs/architecture/PRINCIPLES.md`
- `docs/architecture/LAYERS.md`
- `docs/architecture/RUST_CORE_CONTRACTS.md`
- `docs/architecture/SOURCE_PARSER_OWNERSHIP.md`
- ADR 0007 — Own Lossless Source Parsers
- ADR 0008 — Define Browser Runtime Evidence Normalization and Core Import Ownership
- ADR process — `docs/decisions/README.md`
