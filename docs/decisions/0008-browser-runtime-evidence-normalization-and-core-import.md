# ADR 0008: Define Browser Runtime Evidence Normalization and Core Import Ownership

| Field | Value |
| --- | --- |
| Status | Proposed |
| Date | 2026-08-09 |
| Decision owner / approver | YT-TechDev — approval pending while Proposed |
| Linked Issue | #155 |
| Related Pull Request | #157 |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | `docs/architecture/LAYERS.md` on acceptance; ADR 0004 and existing source-anchor/raw-coordinate contracts remain unchanged |

## Context

Frontend Analysis needs browser-runtime evidence from multiple JavaScript engines while keeping the Rust Core browser independent.

Runtime research established several boundaries that cannot safely be collapsed:

- JavaScript runtime source may contain lone surrogate code units and therefore cannot always be represented losslessly by a Rust UTF-8 `String`.
- Browser source coordinates vary by engine and may also vary by evidence channel within one engine.
- Page, Worker, service-worker, frame, and future targets can have independent lifetimes.
- Protocol-native script, source, frame, session, and actor identifiers are not durable browser-independent identities.
- The existing Core `SourceId` contract from ADR 0004 is caller supplied, source-instance scoped, not globally unique, not persistent across runs, not content-derived, and not Core-generated.
- Runtime source identity and Core source identity answer different questions; some runtime sources may legitimately have no Core `SourceId` because they are unavailable, unverified, or not losslessly representable as Core UTF-8.
- SourceId allocation must not depend on protocol arrival order, Worker scheduling, async completion, hash-map iteration, randomness, global mutable state, or target-local reset.

The current architecture already requires Browser Adapters to isolate browser-specific protocols and normalize observations before Core. It intentionally leaves the concrete normalized-input and Application Orchestration contracts deferred. This ADR resolves that long-lived ownership ambiguity without freezing unqualified engine-specific values or production API shapes.

## Decision

Frontend Analysis adopts the following runtime-evidence ownership model:

```text
Browser Runtime
      ↓
Browser Adapter
      ↓
immutable browser-independent target-lifetime evidence
      ↓
Application Orchestration
      ↓
Core-facing import validation
      ↓
Frontend Analysis Core
```

### Browser Adapter capture domain

Each Browser Adapter owns browser-specific live capture state, including:

- protocol transport, session, and precise target lifecycle;
- protocol-native identifiers and ephemeral handles;
- exact lossless runtime source snapshots;
- engine-native and evidence-channel-specific coordinate spaces;
- raw engine reason vocabularies and capability evidence;
- native-coordinate-to-runtime-snapshot normalization proof; and
- material raw provenance required to audit normalization.

Lossless runtime source that cannot be represented as strict UTF-8 remains adapter-owned evidence. Core does not adopt a UTF-16 runtime-source domain. Raw CDP, WebKit Inspector Protocol, Firefox RDP, or future protocol DTOs do not become Core domain values.

### Target-lifetime normalized handoff

The Browser Adapter produces owned browser-independent evidence scoped to one precise target lifetime. The handoff is immutable after transfer and contains no live adapter reference, protocol-native handle, raw protocol DTO, or untranslated native coordinate.

Target lifetimes remain independently invalidatable. One run-global invalidation epoch is not sufficient.

`TargetEvidenceSnapshot` is a conceptual role name only. This ADR does not freeze a Rust type name, module, public API, serialized shape, container ordering, or streaming contract.

### Runtime source identity

A project-owned opaque runtime source identity identifies one exact runtime source snapshot instance under one target lifetime. This identity is conceptually a `RuntimeSourceKey`.

It is not Core `SourceId`, a protocol identifier, URL/path, content hash, pointer, timestamp, or persisted cross-run identifier. Target replacement, generated source creation, or exact runtime source revision creates a distinct runtime source instance.

### Runtime-to-Core source binding

`RuntimeSourceKey` and `SourceId` remain separate identity domains.

A runtime source may bind to a Core `SourceId` only when its exact snapshot is available, the exact snapshot is losslessly representable as strict UTF-8, that exact UTF-8 projection is the text Core will own, and the proposed binding is valid in the active analysis/import scope.

Within one managed analysis/import scope:

- one runtime source instance binds to at most one `SourceId`;
- distinct imported runtime source instances use distinct `SourceId` values even when their bytes are equal;
- unrepresentable, unavailable, unverified, or intentionally unimported runtime source receives no fabricated `SourceId`; and
- the binding is immutable after Core-owned source construction.

### SourceId authority

ADR 0004 remains authoritative: `SourceId` is caller supplied at the Core boundary.

For managed runtime-enabled analysis, the caller-side semantic responsibility for SourceId authority belongs to Application Orchestration. Application Orchestration owns one explicit single-owner SourceId authority per Core analysis/import scope.

The authority:

- is broader than any individual target lifetime;
- does not reset on navigation, reload, target destruction, Worker termination, generated-source completion, or source revision;
- never reuses a claimed value while the analysis scope remains alive;
- may reuse numeric values in another independent analysis scope;
- may explicitly reserve existing caller-owned SourceId values; and
- rejects collisions rather than silently remapping them.

The default managed allocation policy is checked scope-local sequential allocation from `SourceId(0)` upward, skipping explicit reservations. Numeric value and allocation order are operational only and have no semantic meaning.

Allocation occurs from deliberate source-instance import registration owned by Application Orchestration. Protocol event arrival, task completion, Worker scheduling, hash-map iteration, filesystem enumeration, randomness, or wall-clock time do not directly allocate SourceId. Exhaustion is an explicit recoverable failure; allocation never wraps.

Application Orchestration is a responsibility already present in the architecture. This ADR does not require a new crate, process, service, or repository.

### Coordinate normalization and SourceAnchor

Browser-native coordinates terminate before Core.

```text
native observation coordinate
      ↓
qualified evidence-channel NativeCoordinateSpace
      ↓
verified runtime-snapshot-local coordinate
      ↓
strict UTF-8 representability and byte conversion
      ↓
Core source text under an explicit SourceId binding
      ↓
SourceAnchor
```

Source binding and location binding are separate proofs. A valid `RuntimeSourceKey -> SourceId` binding does not by itself prove a runtime location. One runtime source may be referenced by multiple native coordinate spaces, so coordinate-space ownership is evidence-channel specific rather than one descriptor per runtime source.

### Provenance domains

Resource source and runtime source remain distinct source-instance provenance domains. If both are imported as Core-owned UTF-8 source instances, they use distinct `SourceId` values even when their complete bytes are equal.

Byte identity, decoding, newline preprocessing, BOM handling, generation, or introduction are provenance evidence and do not collapse identity. This ADR does not freeze a closed provenance-relation enum.

### Runtime observation families

The browser-independent normalized boundary may carry runtime source/script observation, paused execution, call-frame value snapshots, exception evidence with independently represented location roles, and optional qualified coverage evidence.

Ephemeral protocol call-frame or actor handles remain adapter-owned. A common pause cause, when available, is a coarse normalized view and does not erase raw engine evidence. Coverage remains optional and measurement-qualified; native granularity remains explicit.

### Held empirical dimensions

This ADR freezes ownership and invariant boundaries, not unobserved engine values. WebKit/Firefox coordinate units and transforms, exact cross-engine pause-reason mapping, a closed coverage taxonomy, product-browser resource/runtime transformations, and Safari external raw-WIP transport remain empirical or capability concerns. No Chromium-specific value becomes a universal browser-independent fact.

### API and implementation boundary

This decision does not authorize a new crate/workspace member, generic `common`/`shared`/`utils` crate, stable public Runtime Evidence API, serialization, persistence, wire protocol, async/concurrency model, atomics/locks/channels/streams, `Send`/`Sync` commitment, Browser Adapter implementation, browser automation, product implementation, publication, or release work.

Concrete Rust names and physical placement remain later focused implementation decisions.

## Alternatives Considered

### Raw browser protocol DTOs cross directly into Core

Rejected. It makes Core protocol-dependent, transfers browser lifecycle and compatibility semantics into the stable domain, permits native coordinates to masquerade as Core positions, and lets one engine vocabulary shape browser-independent analysis.

### One universal runtime-source and coordinate model in Core

Rejected. Runtime JavaScript may contain code units strict Core UTF-8 cannot represent, coordinate units vary, and evidence channels may use different spaces for one runtime source.

### Reuse protocol-native IDs or RuntimeSourceKey as SourceId

Rejected. Protocol IDs are session/target/lifecycle specific. `RuntimeSourceKey` identifies a runtime source snapshot under a target lifetime, while `SourceId` identifies a Core-owned strict UTF-8 source instance in a caller-defined analysis scope. Some runtime sources cannot be imported into Core.

### Let each Browser Adapter allocate SourceId

Rejected. Browser Adapters do not own Core source identity, and protocol/event timing would become an identity input.

### Use a Core-global or process-global SourceId allocator

Rejected. ADR 0004 defines SourceId as caller supplied; global mutable registries are prohibited and process-wide allocation would create unnecessary concurrency and cross-analysis coupling.

### Reset SourceId allocation per TargetLifetime

Rejected. Historical page evidence and independently live Worker evidence may coexist in one Core analysis scope, so target replacement cannot make an old SourceId safe to recycle.

### Random, hash-, URL-, or content-derived SourceId values

Rejected. They create collision, persistence, content-identity, or browser/resource semantics that ADR 0004 intentionally does not assign to SourceId.

### Introduce a dedicated runtime-contract or generic common crate now

Not selected. There is no demonstrated independent release cadence, consumer, dependency cycle, or compatibility boundary that justifies a new foundational crate. A future focused decision may select one when evidence requires it.

### Defer the architecture decision

Not selected. Ownership, identity, source, coordinate, and allocation boundaries now have enough evidence to prevent repeated redesign while engine-specific empirical values remain held separately.

## Consequences

### Positive

- Core remains browser- and protocol-independent.
- Existing UTF-8 source-anchor semantics remain authoritative.
- Lossless JavaScript runtime source can be preserved without weakening Core's source domain.
- Page and Worker lifecycle can evolve independently.
- Native coordinates and protocol IDs cannot silently become Core positions or durable identity.
- Runtime and authored/resource source provenance remains expressible.
- SourceId retains accepted caller-supplied semantics.
- SourceId allocation is bounded to one explicit owner and analysis scope.
- Capability gaps remain representable without false normalization.

### Negative

- Runtime ingestion requires explicit identity, source-binding, and unavailable-state handling.
- Some captured runtime sources remain intentionally unanchorable in Core.
- Application Orchestration must own bounded mutable SourceId allocation state.
- Browser Adapters must retain raw evidence sufficient to audit normalization.
- A later external adapter may require a separate public browser-independent contract decision.

### Risks

The normalized handoff could become an oversized universal DTO. Mitigation is to preserve capability-specific observation families and avoid closed taxonomies before evidence supports them. Sequential SourceId allocation could be mistaken for chronology; numeric ordering is explicitly non-semantic. Future persistence pressure must use a separate identity/compatibility decision rather than silently expanding ADR 0004 semantics.

### Reversibility

Private adapter capture representation and normalized input Rust layout remain replaceable because this ADR freezes no public type names, serialization, ABI, or protocol transport. Changing ownership boundaries, SourceId authority, target-lifetime scope, or native-coordinate termination rules is a material architecture change requiring a new ADR.

## Compatibility and Migration

Existing `SourceId`, `SourceText`, `SourceRange`, `SourceAnchor`, and `RawSourceCoordinate` semantics are preserved. ADR 0004 is not superseded.

No existing public Rust API, serialized format, Cargo workspace topology, dependency, feature, browser protocol, or product contract changes merely by accepting this ADR.

On acceptance, `docs/architecture/LAYERS.md` must be updated so the active normative architecture records Browser Adapter ownership of native/lossless runtime capture and normalization, target-lifetime-scoped normalized handoff, Application Orchestration ownership of managed analysis-scope SourceId authority, Core-facing validation of browser-independent normalized evidence, and the prohibition on direct protocol/native-coordinate authority in Core.

`docs/architecture/RUST_CORE_CONTRACTS.md`, `docs/architecture/VALIDATED_SOURCE_ANCHORS.md`, and `docs/architecture/RAW_SOURCE_COORDINATES.md` require no semantic change if implementation remains within their existing rules. If review discovers a real conflict, acceptance stops until the owning normative contract is addressed explicitly.

## Security and License Impact

No dependency, parser, browser protocol library, unsafe Rust, FFI, process boundary, or license change is selected.

Browser runtime data, protocol payloads, coordinates, and source text are externally influenced data. Future implementation must validate boundary data, avoid uncontrolled panic, preserve source provenance, avoid logging arbitrary source content, enforce bounded resource behavior, and keep active vulnerability details on the private security route.

The repository remains MIT licensed. Third-party protocol/library licensing is a future adapter-specific dependency decision.

## Validation

Before this ADR may become Accepted, review must demonstrate:

1. the live ADR number remains valid;
2. all required ADR fields and sections are complete;
3. ADR 0004 SourceId semantics are preserved;
4. Browser Adapter, Application Orchestration, and Core responsibilities remain non-overlapping;
5. page reload and independently live Worker evidence cannot cause SourceId reuse inside one analysis scope;
6. unrepresentable runtime source remains valid adapter evidence without fabricated Core text;
7. native debugger, coverage, or generated-source coordinates cannot enter Core without explicit coordinate-space translation;
8. distinct equal-byte runtime source instances remain distinct identities;
9. ResourceSource and RuntimeSource remain distinct when equal;
10. SourceId allocation does not depend on protocol event or task-completion order and cannot wrap;
11. no global allocator/registry is introduced;
12. no new crate, public API, serialization, async, concurrency, or implementation is authorized;
13. held WebKit/Firefox empirical values remain held;
14. explicit maintainer approval is recorded; and
15. `LAYERS.md` is updated consistently in the acceptance change.

Use only repository-approved validation status vocabulary.

## Follow-Up

After acceptance, separately plan the first private/internal browser-independent runtime-evidence domain implementation, physical placement of Application Orchestration SourceId authority when a real caller requires it, one bounded CDP adapter slice, completion of WebKit/Firefox empirical evidence, and any dedicated neutral contract crate only when a real dependency/release boundary justifies it.

Persistence, serialization, async, concurrency, and cross-process identity remain independent future decisions. No follow-up begins automatically.

## Approval

Pending.

While this ADR is `Proposed`, it does not authorize implementation.

Acceptance requires one explicit, attributable, decision-specific durable approval by `YT-TechDev` on Issue #155, including confirmation that approval authorizes the architecture and normative documentation update but not production runtime implementation.

After approval, ADR status, approval evidence, ADR index, and `docs/architecture/LAYERS.md` are updated through a focused acceptance Pull Request.

## References

- Issue #155
- Pull Request #157
- ADR process: `docs/decisions/README.md`
- ADR 0004: `docs/decisions/0004-validated-source-anchor-semantics.md`
- Architecture Layers: `docs/architecture/LAYERS.md`
- Rust Core Contracts: `docs/architecture/RUST_CORE_CONTRACTS.md`
- Validated Source Anchors: `docs/architecture/VALIDATED_SOURCE_ANCHORS.md`
- Raw Source Coordinates: `docs/architecture/RAW_SOURCE_COORDINATES.md`
- Issue Model: `docs/development/ISSUE_MODEL.md`
- Validation: `docs/development/VALIDATION.md`
- Secure Development: `docs/development/SECURE_DEVELOPMENT.md`
