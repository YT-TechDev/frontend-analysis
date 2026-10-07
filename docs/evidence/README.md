# Evidence Records

## Purpose

This directory preserves durable, reviewable evidence for language research and
cross-cutting repository qualification.

These records exist to answer:

- what has actually been established;
- which claims were falsified or deliberately weakened;
- which architecture boundaries are supported by evidence;
- which implementation and representation decisions remain open; and
- which durable Issues, Pull Requests, specifications, and validation records
  support the current position.

Evidence documents are **task and evidence records**, not normative architecture
contracts. They do not silently override documents under `docs/architecture/`,
accepted ADRs, or maintainer decisions. When evidence supports a normative
architecture change, that change must follow the repository's normal decision
and documentation process.

## Evidence Discipline

Language research follows these rules:

1. Raw retained source and existing source-provenance contracts remain the
   project authority for exact source-backed observations.
2. Normative specifications outrank candidate parser or browser behavior for
   language semantics.
3. Candidate-independent gold and project-owned validation outrank agreement
   with one implementation.
4. External parsers and browser engines are differential and interoperability
   evidence, not semantic authority for Core contracts.
5. Negative evidence is preserved. A rejected hypothesis must not disappear
   merely because a later architecture no longer considers it.
6. Evidence edition, specification snapshot, browser/engine version, profile,
   capability, and validation envelope must remain explicit when they affect a
   conclusion.
7. Semantic distinctions do not automatically prescribe separate Rust types,
   modules, stores, lattices, graphs, crates, or public APIs.
8. Research status and production status remain distinct. A stable evidence set
   may authorize architecture consolidation without authorizing implementation.

## Repository Qualification Records

`repository/` contains durable task/evidence records for cross-cutting
repository, toolchain, and CI qualification. It is not normative architecture,
toolchain-policy authority, or language-semantic authority.

- [Repository qualification evidence](repository/README.md)
- [2026-09 Rust 1.98.1 toolchain migration](repository/2026-09-rust-1.98.1-toolchain-migration.md)

## Current Language Records

The [2026-10-08 whole-project PAUSE adoption](https://github.com/YT-TechDev/frontend-analysis/issues/104#issuecomment-6044454270) governs the accepted four-command [v0.2.0 GitHub/source release](https://github.com/YT-TechDev/frontend-analysis/releases/tag/v0.2.0): maintain accepted capabilities, correct reproducible defects, use the Product, and observe concrete demand. No new implementation or broad research frontier is selected.

| Domain | Current evidence state | Record |
| --- | --- | --- |
| HTML | **STRONG BOUNDED BASELINE / PAUSED.** Candidate C / ADR 0010; selected constructed tree, ordinary authored attributes, selected Data/AttributeValue character references, Paragraph close/recovery causal evidence and corrected PI entry refusal (#912/#913). Public answer: `fa html-tree`. Not complete HTML or runtime DOM semantics. | [Current HTML evidence](html/README.md) · [#348 adopted PAUSE](https://github.com/YT-TechDev/frontend-analysis/issues/348#issuecomment-6044419612) · [Historical TC-S10 checkpoint](html/2026-09-post-tc-s10-accepted-baseline-checkpoint.md) |
| CSS | **STRONG BOUNDED BASELINE / PAUSED.** Bounded selector and authored transform qualification; 21/21 current-normative selected transform Functions with unknown/future containment. Public answers: `fa css-selectors` and `fa css-transforms` (PR #915). Zero selected transforms cannot prove whole-source absence; no matching/cascade/browser claim. | [Current CSS evidence](css/README.md) · [#418 adopted PAUSE](https://github.com/YT-TechDev/frontend-analysis/issues/418#issuecomment-6041412391) · [Historical value checkpoint](css/2026-09-authored-value-qualification-and-transform-coverage-checkpoint.md) |
| ECMAScript | **STRONG BOUNDED BASELINE / FRONTIER EXPANSION PAUSED.** Architecture Model v1.1, bounded selected grammar/static semantics and `fa es-binding-refs` Product (PR #863). Historical post-#343 `9 / 184` is not current selected reachability. Aggregate selected completion **NOT ESTABLISHED**; non-blocking for this public question. No general scope, TDZ, runtime or module semantics. | [Current ECMAScript evidence](javascript/README.md) · [#688 adopted PAUSE](https://github.com/YT-TechDev/frontend-analysis/issues/688#issuecomment-6043550550) · [Historical post-#343 checkpoint](javascript/2026-08-post-343-selected-ecmascript-research-checkpoint.md) |

## Shared Cross-Language Evidence

The three language tracks independently reinforce several reusable constraints:

- exact source-backed evidence must remain traceable to retained source rather
  than reconstructed from normalized meaning;
- parser success alone does not imply complete semantic validity;
- higher layers must not silently upgrade lower-layer incompleteness or erase
  diagnostics, recovery, unsupported, resource, or invariant-failure meaning;
- capability-specific analysis is preferred over premature universal AST,
  event, graph, result, or analyzer abstractions;
- authored syntax and synthesized or runtime-derived structure remain distinct;
- browser/runtime evidence is a separate qualified evidence path and must not
  redefine source-parser authority;
- deterministic, bounded, candidate-independent validation is a first-class
  architecture input; and
- semantic ownership must be explicit before implementation placement is
  selected.

These are shared evidence constraints, not proof that HTML, CSS, and ECMAScript
should share one parser architecture or one internal representation.

## Repository Authority

Relevant durable repository sources include:

- [ADR 0007 — own lossless source parsers](../decisions/0007-own-lossless-source-parsers.md)
- [ADR 0010 — define HTML tree-construction architecture](../decisions/0010-html-tree-construction-architecture.md)
- [Source Parser Ownership](../architecture/SOURCE_PARSER_OWNERSHIP.md)
- [HTML Tree-Construction Architecture](../architecture/HTML_TREE_CONSTRUCTION.md)
- [Architecture Principles](../architecture/PRINCIPLES.md)
- [Architecture Layers and Boundaries](../architecture/LAYERS.md)
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md)
- [Validated Source Anchors](../architecture/VALIDATED_SOURCE_ANCHORS.md)
- [Raw Source Coordinates](../architecture/RAW_SOURCE_COORDINATES.md)

The language-specific records link their own Issues, Pull Requests, standards,
and validation evidence.

## Update Policy

Update a language evidence record when a new result materially changes one of:

- a supported or falsified hypothesis;
- a capability boundary;
- a normative edition/profile assumption;
- a provenance or failure invariant;
- a browser/interoperability conclusion;
- an architecture inference;
- an OPEN representation decision; or
- the research-to-architecture readiness status.

Do not rewrite historical failures into success. Prefer a new dated correction,
supersession note, or qualified statement that preserves why the earlier claim
was rejected.
