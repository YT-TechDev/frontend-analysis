# Repository Agent Contract

## Repository Purpose

Frontend Analysis is an architecture-first, evidence-driven, browser-independent platform for understanding, visualizing, and diagnosing frontend behavior. It is not a browser or browser-engine replacement. Browser-specific protocols and lifecycle details belong in adapters; reusable analysis semantics belong in the browser-independent Core.

## Role and Authority

An agent may inspect relevant context, implement approved scope, provide evidence and alternatives, validate, and report blockers.

An agent must not approve architecture, public API, dependency, security, concurrency, async, compatibility, unsafe, release, or other contract decisions; invent missing architecture; resolve conflicting normative documents; claim maintainer authority; or treat private AI conversations as repository truth. Issue creation, assignment, milestone placement, existing code, silence, Pull Request creation, or merge does not establish approval. Authority remains with maintainers under [Maintainership and Decision Authority](docs/governance/MAINTAINERSHIP.md).

## Source of Truth

The active Issue or task defines the desired result and scope. Specialized normative documents govern their topics; this file governs shared agent execution only. Task instructions cannot silently weaken architecture, security, compatibility, governance, or contribution contracts. Use live repository state and durable records. If authority conflicts or is unclear, stop at the affected boundary for maintainer resolution. [The Documentation Index](docs/README.md) governs classification, ownership, and conflict handling.

## Required Context

Always read the active task, this contract, and directly relevant files. Select more by impact:

| Task impact | Read |
| --- | --- |
| Documentation classification, source conflict, or supersession | [`docs/README.md`](docs/README.md) |
| Approval, authority, or unresolved decision | [`docs/governance/MAINTAINERSHIP.md`](docs/governance/MAINTAINERSHIP.md) |
| Architecture principles or durable boundaries | [`docs/architecture/PRINCIPLES.md`](docs/architecture/PRINCIPLES.md) |
| Layer ownership, dependency direction, adapters, Core, results, or products | [`docs/architecture/LAYERS.md`](docs/architecture/LAYERS.md) |
| Rust Core ownership, types, errors, concurrency, async, visibility, compatibility, or unsafe | [`docs/architecture/RUST_CORE_CONTRACTS.md`](docs/architecture/RUST_CORE_CONTRACTS.md) |
| Dependencies, untrusted input, secrets, network trust, sensitive logging, or security exceptions | [`docs/development/SECURE_DEVELOPMENT.md`](docs/development/SECURE_DEVELOPMENT.md) |
| Significant architecture decision or supersession | [`docs/decisions/README.md`](docs/decisions/README.md) |
| Contribution, branch, Pull Request, or validation reporting | [`.github/CONTRIBUTING.md`](.github/CONTRIBUTING.md) |
| Vulnerability reporting | [`SECURITY.md`](SECURITY.md) |

Read only documents relevant to actual or discovered impacts; do not explore broadly merely to appear thorough. A small diff does not excuse omitting a relevant contract. The Code of Conduct governs conduct concerns but is not routine technical-task context.

## Execution Rules

1. Resolve the repository, active Issue or task, target branch, and desired result.
2. Confirm scope, exclusions, invariants, compatibility, and required approvals.
3. Read only scope-relevant authoritative documents.
4. Inspect the minimum repository context needed.
5. Implement the smallest coherent change satisfying the approved contract.
6. Run proportionate validation.
7. Inspect the final changed-file list and diff.
8. Report results, limitations, deviations, risks, and unresolved work.

For detailed execution stages, escalation classification, validation duties,
and completion reporting, see the
[Implementation Agent Workflow](docs/development/AGENT_WORKFLOW.md).

Architecture precedes implementation. Missing design is not permission to invent it; stop at unresolved architecture or ownership boundaries. Review generated changes like human-written changes. Report out-of-scope issues instead of fixing them; broad refactoring requires approved scope.

Keep the diff focused. Prohibit unrelated formatting, opportunistic refactoring, hidden behavior changes, unrequested migrations or breaks, and speculative helpers, frameworks, plugin systems, abstraction hierarchies, or future infrastructure. Do not expand dependencies, CI, workflows, settings, releases, or documentation unless in scope. Never weaken a contract for convenience or bypass boundaries through generic `common`, `shared`, or `utils` modules.

A necessary adjacent change must be required for correctness, remain approved, cross no unresolved boundary, and be reported. Escalate expansions to architecture, API, dependency, security, compatibility, concurrency, async, or unsafe scope.

## Architecture and Rust Boundaries

Browser protocols, transport, and engine-specific lifecycles stay in Browser Adapters. Core receives approved browser-independent inputs and owns browser-independent analysis semantics. It must not depend on CDP, WebKit Inspector Protocol, Firefox protocol, browser implementations or Adapters, React, Electron, Tauri, VS Code APIs, UI frameworks, or products. Core and result contracts own Analysis Result meaning; presentation may transform representation but not redefine evidence, certainty, findings, or diagnostics. Runtime direction does not reverse source dependency ownership. Browser data crosses only through an approved browser-independent contract; unsupported or lossy normalization stays explicit. See [Architecture Principles](docs/architecture/PRINCIPLES.md) and [Architecture Layers and Boundaries](docs/architecture/LAYERS.md).

For Rust Core, ownership, lifecycle, mutation, invalidation, and error responsibility must be explicit. Immutable, single-owner designs are default; cloning is not an ownership escape hatch. `Rc`, `Arc`, `Cell`, `RefCell`, locks, atomics, channels, concurrency, and async require demonstrated need; `Arc<Mutex<_>>` is not default architecture. Protocol, parser, serialization, runtime, and third-party errors must not become Core contracts by convenience. External or untrusted conditions must not cause ordinary Core panics. Use narrowest useful visibility; create no accidental public API, serialized contract, `Send`, or `Sync` promise. `unsafe Rust` is prohibited without an explicitly approved focused Issue. See [Rust Core Contracts](docs/architecture/RUST_CORE_CONTRACTS.md).

## Escalate and Stop

Stop at the affected boundary and report when any of these appears:

- unresolved ownership, lifecycle, mutation, or invalidation;
- a layer, dependency-direction, or browser-independent contract change;
- browser-specific behavior or a protocol type entering Core;
- a public API, exported type, serialized representation, protocol-neutral contract, or compatibility change;
- a dependency addition, removal, foundational change, or major update;
- security-sensitive behavior, untrusted-input contracts, secrets, network trust, sensitive logging, or a security exception;
- proposed `unsafe Rust`;
- new concurrency, shared mutation, async, cancellation, streaming, process, IPC, or runtime boundaries;
- required scope expansion, conflicting or missing authority, or an unauthorized repository side effect;
- validation failure suggesting the approved design is wrong, or a need to weaken an invariant to pass; or
- vulnerability details requiring private reporting.

Preserve evidence, explain conflicts, identify affected documents, present realistic alternatives, and recommend a focused follow-up Issue when useful. Never choose unresolved architecture, guess a contract, mark a proposal approved, or hide escalation. Follow [Maintainership](docs/governance/MAINTAINERSHIP.md), [Secure Development](docs/development/SECURE_DEVELOPMENT.md), and, for significant architecture decisions, the [ADR Process](docs/decisions/README.md).

## Validation

Run validation required by the task and affected area, preferring existing commands. Validate behavior, not compilation alone. Inspect the final file list and diff; run relevant available formatting, linting, tests, links, or contract checks. Do not add a tool solely to validate documentation unless required.

Report each check run, its result, failures, and omitted checks with reasons. Never claim validation that did not occur. State partial completion; passing tests do not authorize contract changes. Proportionate documentation checks include links, file scope, placeholders and private links, whitespace and final newline, and Markdown structure or rendering.

## Repository Side Effects

Unless explicitly requested and permitted by repository authority, do not create or merge Pull Requests; push or rewrite shared history; create tags or releases; publish packages; change settings, branch protection, labels, milestones, secrets, environments, or permissions; add CI, automation, bots, or schedules; contact external parties; or expose private information. Report unavailable authorized side effects.

## Completion Report

Provide a concise structured report with: completed, partial, or blocked result; changed files and contracts or behavior; validation performed and results; validation omitted and why; scope deviations and necessary adjacent changes; compatibility, security, dependency, concurrency, async, public API, and unsafe impact; remaining risks; unresolved decisions or blockers; follow-up work; and commit or Pull Request details only when created or requested.

Do not claim work not performed, omit failed validation, or present proposed architecture as approved. Durable records must stand on repository evidence and must not require private AI links.

## Authoritative References

- [Documentation Index](docs/README.md)
- [Maintainership and Decision Authority](docs/governance/MAINTAINERSHIP.md)
- [Architecture Principles](docs/architecture/PRINCIPLES.md)
- [Architecture Layers and Boundaries](docs/architecture/LAYERS.md)
- [Rust Core Contracts](docs/architecture/RUST_CORE_CONTRACTS.md)
- [Secure Development](docs/development/SECURE_DEVELOPMENT.md)
- [ADR Process](docs/decisions/README.md)
- [Contributing](.github/CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [Implementation Agent Workflow](docs/development/AGENT_WORKFLOW.md)
