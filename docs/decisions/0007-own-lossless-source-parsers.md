# ADR 0007: Own Lossless Source Parsers

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-06 |
| Decision owner / approver | YT-TechDev |
| Linked Issue | [#105](https://github.com/YT-TechDev/frontend-analysis/issues/105) |
| Related Pull Request | [#118](https://github.com/YT-TechDev/frontend-analysis/pull/118) |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | `docs/architecture/SOURCE_PARSER_OWNERSHIP.md`, `docs/README.md` |

## Context

Frontend Analysis needs exact source-backed evidence that browser-oriented and
compiler-oriented parsers commonly do not retain or expose through stable public
contracts. Required evidence includes retained raw UTF-8 identity, exact
zero-based half-open byte ranges, authored spelling, diagnostics, recovery,
capability completeness, and deterministic ordering.

Qualification work for ECMAScript, HTML, and CSS established that external
parsers can remain useful while still omitting one or more required boundaries.
Continuing to qualify candidates, design upstream extensions, contribute missing
features, and wait for upstream releases would place external projects on the
Core critical path and surrender control over the product roadmap.

The decision must preserve the existing browser-independent Rust Core,
validated source-anchor semantics, and raw-source-coordinate semantics. It must
not turn Frontend Analysis into a browser engine or require complete language
support before useful bounded analysis can ship.

## Decision

Frontend Analysis will own purpose-built lossless source parsers in Rust for:

```text
HTML → CSS → ECMAScript
```

HTML is the first active language domain. Each parser will be implemented
incrementally through explicit capability boundaries, beginning with lossless
tokenization and exact source-backed observations before broader grammar,
static-semantics, or tree-construction coverage.

Within an explicitly supported capability, project-owned parsers are the
semantic authority for Frontend Analysis source-backed observations. They must
preserve retained raw UTF-8 provenance and the accepted zero-based half-open
coordinate semantics.

The parser program must distinguish:

- authored syntax from decoded or normalized values;
- clean success from diagnostics, recovery, partial processing, unsupported
  capability, resource termination, abort, and internal failure;
- explicit source syntax from implied or synthesized structure; and
- parser capability from language-specification completeness.

Third-party parsers and browser engines may be used for differential testing,
interoperability comparison, fixture discovery, performance comparison, and
optional adapters. Their APIs, features, maintainer decisions, and releases are
not mandatory dependencies and must not block the Core parser roadmap.

Parser-native external types, errors, and object graphs must not become Core
public contracts. No generic cross-language AST, plugin framework, transport,
serialization format, crate topology, async runtime, concurrency model, or
stable public parser API is selected by this ADR.

The first integrated vertical slice is a bounded HTML tokenizer and analysis
parser connected to the existing `SourceText`, `SourceId`, `SourceRange`,
`SourceAnchor`, and raw-coordinate contracts.

`docs/architecture/PRINCIPLES.md`, `docs/architecture/LAYERS.md`, and
`docs/architecture/RUST_CORE_CONTRACTS.md` were reviewed and remain unchanged.
Their browser independence, Core ownership, dependency direction, evidence,
determinism, error, panic, visibility, compatibility, async, concurrency, and
unsafe constraints already permit this decision. The new specialized
`docs/architecture/SOURCE_PARSER_OWNERSHIP.md` contract defines the parser-specific
rules without weakening or duplicating those general contracts.

## Alternatives Considered

### Continue qualifying and adopting external Rust parsers

This minimizes initial implementation volume and can reuse mature grammar and
recovery behavior. It was not selected because qualification history showed
that exact source evidence and semantic-validation contracts may remain
unavailable, unstable, or controlled by another roadmap. Candidate replacement
would repeatedly reopen Core ownership and compatibility questions.

### Pursue upstream source-provenance and semantic APIs

This can improve upstream ecosystems and avoid local implementations. It was not
selected as the product-critical strategy because proposal review, acceptance,
implementation, release, and long-term compatibility remain externally owned.
Upstream contribution may continue only as optional work that does not block
Frontend Analysis.

### Use implementation-language-neutral external frontends

This permits best-fit parsers and stronger process isolation. It was not selected
for the first source-parser foundation because it adds transport, schema,
version-skew, deployment, and failure-boundary cost before a stable semantic
contract has been proven. Future adapters may still use polyglot or process
boundaries behind Core-owned contracts.

### Own tokenizers but delegate grammar and semantics

This improves raw provenance while reducing implementation scope. It was not
selected as the durable strategy because grammar recognition, recovery,
static semantics, and synthesized structure would still depend on external
capabilities and could not be assumed to preserve the evidence contract.
Tokenizers remain the first incremental stage, not the final ownership boundary.

### Own complete purpose-built parsers incrementally in Rust

This has the highest implementation and maintenance cost, but it preserves
semantic ownership, roadmap control, evidence integrity, deterministic behavior,
and native/WASM reuse. This alternative was selected with bounded capability
slices to control risk and avoid attempting complete browser-engine behavior at
once.

## Consequences

### Positive

- Core development no longer waits for external parser APIs or releases.
- Source evidence is designed from product requirements rather than recovered
  from implementation-specific outputs.
- Native and WebAssembly delivery can reuse the same Rust implementation.
- Diagnostics, recovery, partial processing, and unsupported capability remain
  first-class domain states.
- Language frontends can evolve incrementally without claiming unsupported
  specification completeness.
- Third-party implementations remain useful comparison oracles without defining
  Core contracts.

### Negative

- The project assumes substantial long-term implementation and maintenance work.
- HTML recovery and tree construction, CSS contextual parsing, and ECMAScript
  grammar plus static semantics require sustained standards tracking.
- Correctness, security, fuzzing, resource containment, and conformance evidence
  become repository-owned responsibilities.
- Initial feature breadth will be smaller than adopting a general-purpose parser.
- Contributors must understand both language specifications and Frontend Analysis
  evidence contracts.

### Risks

- Scope may expand into browser-engine replacement. Mitigation: capability-bounded
  Issues, explicit out-of-scope declarations, and separate tree/runtime work.
- Premature public APIs may freeze unstable internals. Mitigation: narrow
  visibility and focused compatibility approval before public exposure.
- A project-owned parser may validate itself incorrectly. Mitigation:
  candidate-independent gold fixtures, property tests, fuzzing, deterministic
  replay, standards-derived expectations, and non-authoritative differential
  testing.
- Generic abstractions may erase language-specific semantics. Mitigation: no
  shared parser framework without demonstrated cross-language invariants.
- Adversarial input may cause excessive CPU, memory, recursion, or panics.
  Mitigation: explicit resource and termination contracts, safe Rust, bounded
  validation, and panic prohibition at Core boundaries.

### Reversibility

The decision is reversible through a superseding ADR. Reversal would require a
replacement frontend to prove equivalent or stronger source provenance,
capability, diagnostic, recovery, determinism, security, and compatibility
contracts. Project-owned parser outputs must not be silently reinterpreted as
external-parser outputs during migration. Existing parser evidence and accepted
source-coordinate semantics remain durable historical and compatibility inputs.

## Compatibility and Migration

This ADR creates no stable Rust API, ABI, serialized format, protocol, package,
or SemVer promise. Initial parser types and operations should use the narrowest
useful visibility until a named consumer and compatibility contract are
approved.

Existing `SourceText`, `SourceId`, `SourceRange`, `SourceAnchor`, and
`RawSourceCoordinate` semantics remain unchanged. Parser observations must bind
to one retained source identity and validate exact ranges through those
contracts.

Historical external-parser qualifications remain valid candidate evidence but
no longer authorize or block production parser ownership. Open or future
external-parser work must be classified as optional adapter, interoperability,
benchmark, or upstream work.

The migration sequence is:

```text
ADR and normative contracts
    ↓
HTML tokenizer architecture and domain contracts
    ↓
bounded HTML tokenizer
    ↓
source-backed HTML analysis parser
    ↓
Core integration
    ↓
HTML lessons
    ↓
CSS
    ↓
ECMAScript
```

Native and future WASM execution must preserve equivalent analysis meaning.
JavaScript bindings, serialization, packaging, and product integration require
separate decisions.

## Security and License Impact

All source input is treated as untrusted. Malformed or adversarial input must not
cause uncontrolled panic across Core boundaries. Resource exhaustion,
non-termination, partial output, abort, and unsupported capability require
explicit handling and validation. Repository-authored `unsafe Rust` is not
authorized by this ADR.

No dependency is introduced by this decision. The repository remains MIT
licensed. Future dependencies, standards fixtures, generated data, and external
corpora require focused license, provenance, supply-chain, and maintenance
review.

## Validation

Implementation evidence must include, proportionate to each capability:

- specification-derived candidate-independent fixtures with exact retained
  UTF-8 ranges;
- source identity, range bounds, containment, ordering, and raw-spelling tests;
- malformed, recovered, partial, unsupported, resource-limited, abort, and
  termination cases;
- deterministic repeated-run comparison;
- property and fuzz testing with durable regression fixtures;
- panic and non-termination containment;
- bounded allocation, recursion, source-size, and execution observations;
- differential comparison against independent implementations without treating
  them as authority;
- dependency, feature, license, public-export, and workspace inventories; and
- native execution plus relevant WASM-target compilation evidence.

Architecture validation must confirm that parser ownership, Core analysis,
Analysis Results, browser adapters, and runtime evidence remain separate and
that no external parser feature is on the critical path.

## Follow-Up

- #106 coordinates the HTML parser and first Core integration.
- #107 coordinates the later CSS parser after HTML lessons are recorded.
- #108 coordinates the later ECMAScript parser and required static semantics.
- #109 through #117 define the first HTML architecture, domain, validation,
  implementation, integration, and tree-construction work.

## Approval

Approved by `YT-TechDev`, maintainer of record, on 2026-08-06.

Durable approval:
[Issue #105 approval record](https://github.com/YT-TechDev/frontend-analysis/issues/105#issuecomment-5202154564)

## References

- [Issue #104](https://github.com/YT-TechDev/frontend-analysis/issues/104)
- [Issue #105](https://github.com/YT-TechDev/frontend-analysis/issues/105)
- [Pull Request #118](https://github.com/YT-TechDev/frontend-analysis/pull/118)
- [HTML workstream #106](https://github.com/YT-TechDev/frontend-analysis/issues/106)
- [CSS workstream #107](https://github.com/YT-TechDev/frontend-analysis/issues/107)
- [ECMAScript workstream #108](https://github.com/YT-TechDev/frontend-analysis/issues/108)
- [Architecture Principles](../architecture/PRINCIPLES.md)
- [Architecture Layers](../architecture/LAYERS.md)
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md)
- [Source Parser Ownership](../architecture/SOURCE_PARSER_OWNERSHIP.md)
- [Raw Source Coordinate Semantics](0005-raw-source-coordinate-semantics.md)
- [First Source-Anchored Slice Qualification](0006-qualify-first-source-anchored-analysis-vertical-slice.md)
