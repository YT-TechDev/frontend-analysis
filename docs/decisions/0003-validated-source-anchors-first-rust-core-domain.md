# ADR 0003: Establish Validated Source Anchors as the first Rust Core domain

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-07-31 |
| Decision owner / approver | YT-TechDev |
| Linked Issue | [#53](https://github.com/YT-TechDev/frontend-analysis/issues/53) |
| Related Pull Request | None at decision-record creation |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | None at this decision stage — the existing architecture, Rust Core, security, and validation contracts already govern the selected boundary. This ADR resolves choices those contracts deliberately deferred and constrains Issues #54 and #55 without changing the contracts. |

## Context

The minimal Rust workspace bootstrap established by [ADR 0001](0001-repository-topology-and-workspace-ownership.md) and [ADR 0002](0002-rust-bootstrap-toolchain-and-validation-policy.md) has been completed and independently audited. The repository now has an accepted virtual-workspace, toolchain, validation, and CI baseline with no production package. It is ready for a separately planned, production-quality Core domain; the bootstrap itself did not authorize one.

The first domain must be small, coherent, browser-independent, and substantive enough to establish a real Core responsibility. It must not prematurely select an HTML, CSS, or JavaScript parser, a browser protocol, a browser-runtime model, or a presentation architecture. The repository owner explicitly selected **Option 1 — Validated Source Anchors** in [Issue #53](https://github.com/YT-TechDev/frontend-analysis/issues/53); this ADR records and justifies that accepted direction rather than reopening the option selection.

The capability will eventually accept immutable UTF-8 source text and a requested half-open byte range, validate the request deterministically, and produce an owned source anchor. This statement identifies the capability boundary, not its complete data model. Source identity, range invariants, ownership mechanics, error cases, and the exact Rust API remain decisions for ADR 0004 under Issue #54.

The required dependency direction remains:

```text
Browser Runtime
    ↓
Browser Adapter
    ↓
Frontend Analysis Core
    ↓
Analysis Results
    ↓
Presentation Layer
    ↓
Desktop / CLI / VS Code / Web
```

## Decision

### First production domain and crate boundary

**Validated Source Anchors is the first production-owned Rust Core domain.** Exactly one production crate is authorized for this milestone:

- package name: `frontend-analysis-core`;
- repository path: `crates/frontend-analysis-core`;
- owning layer: Frontend Analysis Core.

The crate is a domain owner, not a miscellaneous utility crate. It must remain independent of browser protocols and implementations, GUI frameworks, desktop shells, CLI presentation, VS Code, React, Electron, Tauri, and every other product surface.

This ADR authorizes the boundary and policies for later implementation. It does not authorize creating the crate or writing Rust in this Pull Request.

### Ownership boundary

At this stage, Frontend Analysis Core owns:

- browser-independent source identity or provenance concepts;
- immutable source ownership suitable for retained analysis results;
- validated source byte ranges;
- owned source anchors;
- deterministic domain validation; and
- domain-level validation errors.

It does not own:

- HTML, CSS, or JavaScript parser implementations, parser tokens, or syntax trees;
- browser protocol payloads or CDP, WebKit Inspector Protocol, or Firefox protocol concepts;
- runtime DOM, CSSOM, layout, paint, or event tracing;
- browser sessions, targets, frames, realms, or execution contexts;
- diagnostics, findings, severity, or evidence graphs; or
- desktop, CLI, VS Code, web, or other presentation behavior.

Future Browser Adapters must normalize browser-specific observations into an approved browser-independent representation before the data crosses into Core-owned models. Protocol data must not enter Core unnormalized, and Core must not depend on an adapter.

### Why exactly one production crate

One crate is justified because this domain establishes the first durable production ownership boundary. An empty production crate would claim ownership without behavior and is not acceptable. Splitting this still-small domain among multiple crates would create boundaries before their independent owners and dependency directions are known. Parser, protocol, adapter, diagnostics, and presentation crates are not justified by this decision.

The single-crate choice does not make `frontend-analysis-core` an unrestricted dumping ground. Code belongs there only when the approved Core domain owns its semantics. Each additional domain or crate requires a separately approved ownership decision; proximity, reuse, or convenience is insufficient.

### Crate policy

The milestone may introduce exactly one production crate. `frontend-analysis-core` at `crates/frontend-analysis-core` must:

- set `publish = false`;
- inherit applicable workspace package metadata and lint configuration;
- contain safe Rust only and use no `unsafe` code;
- introduce neither async nor concurrency primitives; and
- introduce no browser-specific or parser-specific dependency.

The implementation change must establish the applicable workspace metadata and lint inheritance consistently with ADR 0002; this ADR does not modify workspace configuration.

### Dependency policy

The first production crate must have zero third-party Rust dependencies. The Rust standard library is sufficient for the approved milestone. Dependencies must not be added for convenience, speculative extensibility, serialization, error formatting, identifiers, synchronization, testing, or future parser work. Any future dependency requires focused architectural, security, supply-chain, license, compatibility, and maintenance justification under [Secure Development](../development/SECURE_DEVELOPMENT.md).

### Visibility policy

Only the minimum API demonstrably required by workspace consumers may be public. Representation details remain private, and `pub` must not be used speculatively or merely to simplify tests. Public visibility does not imply crates.io publication, and this milestone creates no stable external SDK promise. ADR 0004 and implementation review must further constrain exact type and method visibility; this ADR prescribes no final Rust signature.

### Compatibility policy

Current guarantees are limited but deliberate: domain behavior must be deterministic, Core must remain browser-independent, and source-anchor semantics must be explicit before implementation. Review must prevent accidental compatibility commitments.

This milestone does not promise crates.io publication, a stable external Rust API, stable serialization, a wire format, SemVer compatibility for external consumers, an MSRV, `no_std`, WASM ABI stability, FFI compatibility, or browser protocol compatibility. These deferrals do not relax the approved deterministic behavior or browser-independent semantic boundary.

### Cargo.lock policy

ADR 0002 correctly omitted `Cargo.lock` while the workspace had no package or dependency resolution and deferred ownership to the first-package decision. This repository is an application/workspace-style repository rather than a published-library artifact. When the first production package is introduced, the workspace-generated `Cargo.lock` must therefore be committed and intentionally maintained, including when the initial graph has no third-party dependency. Resolution changes must remain visible and reviewable, and future implementation validation that can update or resolve the graph must use Cargo's `--locked` mode after the committed lockfile is established.

This policy creates no permission to add a dependency. This ADR Pull Request does not create or modify `Cargo.lock`.

### Lint and unsafe policy

The production crate must inherit the applicable workspace lint policy. Consistent with ADR 0002, the implementation milestone must use the workspace-inherited `unsafe_code = "deny"` policy and opt the crate in through `[lints] workspace = true`. Warnings must not be silently suppressed to bypass repository policy. Unsafe Rust is prohibited for this milestone. Lint exceptions require explicit, focused justification and must not be added speculatively.

## Alternatives Considered

### HTML parsing vertical slice

A parsing slice could create visible source analysis and exercise parser integration. It was not selected because the repository has not approved parser ownership, parser semantics, or a parser dependency, and parser tokens and syntax trees are outside the presently approved Core boundary. Selecting parsing first would combine the initial Core responsibility with unresolved parser and dependency decisions. Validated Source Anchors instead establishes a browser-independent location primitive that future approved parsers can consume without selecting one now.

### Browser protocol or Browser Adapter work

Adapter work could begin collecting runtime observations. It was not selected because Browser Adapters own protocol and engine lifecycle concerns, while this milestone is specifically establishing the first production Core domain. Starting with a protocol would risk shaping Core around one browser and would not establish the selected browser-independent responsibility. Adapter work remains future, and must normalize observations before crossing the Core boundary.

### Diagnostics or evidence graph modeling

Diagnostics and evidence graphs would establish analysis-result semantics, but they require decisions about findings, certainty, evidence, lifecycle, and presentation-independent result meaning that this milestone does not resolve. Selecting them first would broaden the initial domain beyond the small source-location responsibility. They remain outside this crate boundary until focused ownership decisions exist.

### Empty or generic Core crate

An empty or generic crate would provide a directory for later work and avoid choosing initial behavior. It was rejected because a package name alone is not a durable production responsibility and would invite unrelated code to accumulate without an owned domain. The first production crate must ship only with concrete, validated behavior approved by ADR 0004 and Issue #55.

### Multiple specialized production crates immediately

Multiple crates could make prospective separation visible early. They were not selected because the approved domain is still small and no independently justified owners or dependency directions exist for those splits. Premature fragmentation would create maintenance and compatibility boundaries without corresponding responsibilities. Further crates require focused approval when real domains justify them.

## Consequences

### Positive

- The repository gains a concrete, testable Core-owned behavior rather than a placeholder.
- Browser independence and the established dependency direction remain intact.
- Future approved parsers and adapters can refer to stable, owned source locations after the semantic contract is accepted.
- Zero third-party dependencies keep the first production dependency graph minimal.
- Crate ownership, validation behavior, and excluded responsibilities are reviewable and testable.

### Negative

- No parser capability is delivered.
- No browser data is collected.
- No diagnostic or analysis result is produced.
- Implementation remains blocked until ADR 0004 defines the source-anchor semantics.
- Future domains, dependencies, consumers, or compatibility requirements may require additional Issues and ADRs.

### Risks

The broad name `frontend-analysis-core` could attract unrelated utilities or prematurely public types. The ownership list, exclusions, narrow-visibility rule, zero-dependency policy, and focused review gate mitigate that risk. Another risk is treating the capability description as a complete data model; ADR 0004 must resolve the semantic details before implementation.

### Reversibility

Before implementation, replacement requires a new approved ADR and has only documentation cost. After workspace consumers depend on the crate or anchors, reversal must assess Rust API, semantic, ownership, and migration impact. Additional crates must be introduced through focused ownership decisions rather than silently splitting or broadening this crate.

## Compatibility and Migration

There is no production crate, Rust API, serialized representation, protocol contract, or existing consumer to migrate in this ADR Pull Request. The decision establishes deterministic, browser-independent domain expectations while deliberately deferring exact source-anchor semantics and API visibility to ADR 0004.

No stable API, ABI, wire, serialization, external SemVer, MSRV, `no_std`, WASM, FFI, or browser-protocol guarantee is created. Future implementation must review exposed types, errors, ordering, auto traits, and other accidental commitments independently. The committed-lockfile policy affects repository reproducibility, not external API compatibility.

## Security and License Impact

No dependency, parser, protocol input, executable behavior, unsafe code, or license is added by this documentation-only decision. Future source text and requested ranges are potentially untrusted input under [Secure Development](../development/SECURE_DEVELOPMENT.md); ADR 0004 must make validation and failure semantics explicit, and implementation must not turn externally influenced invalid input into ordinary Core panics.

The zero-third-party-dependency policy creates no new third-party license or supply-chain impact. Repository-authored code remains under the existing MIT License. Any future dependency or unsafe proposal requires its separate approved review and cannot use this ADR as authorization.

## Validation

This decision record must be validated by:

- conformance with the ADR template, numbering, status, and index conventions;
- resolution of every relative documentation link;
- Markdown structure, whitespace, and final-newline inspection;
- final changed-file and diff review; and
- explicit confirmation that no Rust source, production crate, Cargo manifest, `Cargo.lock`, dependency, workspace membership, lint configuration, or CI file changed.

Future implementation validation is owned by ADR 0004, Issue #55, and [Validation and Completion Evidence](../development/VALIDATION.md). It must test deterministic valid and invalid domain behavior and run the applicable Rust baseline with the committed lockfile enforced. Passing validation cannot widen this ADR's ownership or compatibility boundary.

## Follow-Up

- ADR 0004 under [Issue #54](https://github.com/YT-TechDev/frontend-analysis/issues/54) must be accepted before domain implementation begins. It owns the complete source-anchor semantics and further API constraints.
- [Issue #55](https://github.com/YT-TechDev/frontend-analysis/issues/55) remains blocked until ADR 0003 and ADR 0004 are accepted.
- Every future expansion requires a focused Issue or ADR appropriate to its ownership and compatibility impact.
- Milestone completion requires independent audit [Issue #58](https://github.com/YT-TechDev/frontend-analysis/issues/58) to record exactly `PASS` or `NO-GO`.

### Explicit non-goals

This ADR does not authorize:

- Rust implementation or production crate creation in this Pull Request;
- HTML, CSS, or JavaScript parsing;
- Browser Adapter implementation or browser protocol dependencies;
- serialization, line and column indexing, or source maps;
- diagnostics or evidence graphs;
- async, concurrency, WASM, FFI, `no_std`, or unsafe Rust;
- crates.io release work; or
- additional production crates.

## Approval

Approved by `YT-TechDev`, the current maintainer of record, on 2026-07-31.

Durable approval: [Issue #53 — establish the first production Rust Core domain and crate boundary](https://github.com/YT-TechDev/frontend-analysis/issues/53).

The decision-specific approval selects **Option 1 — Validated Source Anchors** as the first production Core capability and accepts the crate ownership and milestone policy recorded here. It does not approve the data model or final Rust API deferred to ADR 0004, nor implementation deferred to Issue #55.

Any substantive change requires renewed maintainer review.

## References

- [Issue #53: establish the first production Rust Core domain and crate boundary](https://github.com/YT-TechDev/frontend-analysis/issues/53)
- [Parent Issue #52](https://github.com/YT-TechDev/frontend-analysis/issues/52)
- [Milestone 3](https://github.com/YT-TechDev/frontend-analysis/milestone/3)
- [Issue #54](https://github.com/YT-TechDev/frontend-analysis/issues/54)
- [Issue #55](https://github.com/YT-TechDev/frontend-analysis/issues/55)
- [Independent audit Issue #58](https://github.com/YT-TechDev/frontend-analysis/issues/58)
- [ADR 0001](0001-repository-topology-and-workspace-ownership.md)
- [ADR 0002](0002-rust-bootstrap-toolchain-and-validation-policy.md)
- [Architecture Principles](../architecture/PRINCIPLES.md)
- [Architecture Layers and Boundaries](../architecture/LAYERS.md)
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md)
- [Secure Development](../development/SECURE_DEVELOPMENT.md)
- [Validation and Completion Evidence](../development/VALIDATION.md)
- [ADR Process](README.md)
