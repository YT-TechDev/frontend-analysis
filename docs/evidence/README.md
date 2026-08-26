# Language Research Evidence

## Purpose

This directory preserves durable, reviewable research evidence for the HTML, CSS,
and JavaScript / ECMAScript tracks of Frontend Analysis.

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

## Current Language Records

| Domain | Current evidence state | Record |
| --- | --- | --- |
| HTML | First source-analysis slice remains established; #348 research and Candidate C / ADR 0010 remain authoritative; bounded production tree construction is merged through TC-S6, while TC-S7 candidate-independent validation is accepted and merged and its production placement remains undecided. | [HTML evidence](html/README.md) · [2026-08 tree checkpoint](html/2026-08-tree-construction-frontier-checkpoint.md) · [ADR 0010](../decisions/0010-html-tree-construction-architecture.md) · [HTML tree-construction contract](../architecture/HTML_TREE_CONSTRUCTION.md) |
| CSS | Semantic Foundation Freeze #184/#185 remains the latest identified CSS semantic production authority; source, tokenizer, structural parser/context/declaration, Core reconciliation, and bounded CoreV1 selector-qualification foundations remain frozen, with no later CSS semantic production PR identified at the 2026-08-26 status check. | [CSS evidence](css/README.md) · [2026-08 CSS status](css/2026-08-semantic-foundation-status-checkpoint.md) |
| JavaScript / ECMAScript | Architecture Model v1.1 is accepted; the subsequent adversarial research wave is closed/frozen with no architecture-breaking contradiction found, while production representation and implementation remain intentionally open. | [JavaScript evidence](javascript/README.md) · [Post-v1.1 closure](javascript/2026-08-post-v1.1-research-wave-closure.md) |

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
