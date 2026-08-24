# HTML Tree-Construction Architecture

## Purpose and Authority

This document is the specialized normative architecture contract for
browser-independent HTML tree construction in Frontend Analysis.

It records the durable invariants approved through Issue #117 after the
candidate-independent HTML research program in #348. The architectural rationale
for these invariants is preserved by
[ADR 0010](../decisions/0010-html-tree-construction-architecture.md).

This contract specializes, but does not supersede:

- [Architecture Principles](PRINCIPLES.md);
- [Architecture Layers and Boundaries](LAYERS.md);
- [Rust Core Contracts](RUST_CORE_CONTRACTS.md);
- [Source Parser Ownership](SOURCE_PARSER_OWNERSHIP.md);
- [ADR 0007 — Own Lossless Source Parsers](../decisions/0007-own-lossless-source-parsers.md); and
- [ADR 0008 — Define Browser Runtime Evidence Normalization and Core Import Ownership](../decisions/0008-browser-runtime-evidence-normalization-and-core-import.md).

When this contract and a more general architecture document both apply, each
continues to govern its explicit topic. Apparent contradictions must use the
conflict process in the [Documentation Index](../README.md); recency alone does
not resolve them.

This contract does not itself authorize a production implementation. Production
placement remains owned by separately approved focused Issues.

## Scope and Non-Goals

This contract governs durable architecture for project-owned HTML tree
construction, including:

- ownership of tokenizer/tree-constructor coordination;
- parse-context and capability configuration meaning;
- private mutable construction state;
- validated freeze into immutable analysis results;
- constructed-node identity requirements;
- authored and non-authored provenance distinctions;
- recovery, diagnostics, completion, partial-state, unsupported, and resource
  semantics;
- browser-runtime authority separation; and
- incremental capability extension rules.

It does not define:

- a complete implementation of the WHATWG HTML parsing algorithms;
- a browser-compatible DOM API;
- public Rust APIs, serialization, ABI, or wire formats;
- concrete Rust types, traits, module paths, collection choices, arenas, or
  allocation strategy;
- concrete constructed-node ID encoding;
- numeric tree resource limits;
- async, concurrency, cancellation, or task-runtime APIs;
- JavaScript execution or parser reentrancy implementation;
- browser protocol acquisition;
- product-specific CLI, desktop, VS Code, or web behavior; or
- complete support for every document, fragment, foreign-content, formatting,
  table, scripting, or recovery semantic.

A capability not yet implemented remains explicitly unsupported even when this
architecture leaves room for it.

## Conceptual Architecture Model

The approved responsibility model is:

```text
SourceText + Parse Configuration
        ↓
Core-owned Parse Coordinator
        ↕
Pull / Resumable Token Production
        ↓
Private Mutable Tree-Construction Session
        ↓
Validated Freeze
        ↓
Immutable Query-Oriented Tree Analysis
        +
Selective Provenance / Recovery Relations
```

This is a semantic ownership and lifecycle model. `Pull / Resumable Token
Production` means the tree-construction process can request token production and
cause specification-required tokenizer control to take effect before subsequent
tokens are produced. It does not require a Rust `Iterator`, callback, channel,
async stream, or any other specific interface.

The existing explicit-start-tag analysis capability remains a sibling bounded
capability. This contract does not require it to be rewritten around tree
construction.

## Semantic Terms and Distinct Identity Domains

Tree-construction work MUST preserve distinctions among the following domains
when the supported capability makes them relevant:

- **source identity** — the retained `SourceText` identity;
- **authored source evidence** — exact source-backed token/range evidence;
- **constructed-node identity** — result-scoped identity for a constructed tree
  observation;
- **final placement** — the node's constructed parent/order relationship;
- **synthesis cause** — the reason a node with no authored tag exists;
- **recovery/action evidence** — supported structural recovery or transformation
  that explains a result;
- **token disposition** — how supported input was consumed, ignored, discarded,
  reprocessed, stopped, or otherwise handled when no node is the complete
  explanation; and
- **runtime identity/correlation** — separately qualified browser-runtime
  observation identity.

These domains MUST NOT be collapsed merely because one implementation can use a
single index or object to carry several of them internally.

## Ownership Boundaries

### Retained source contracts

`SourceText`, `SourceId`, `SourceRange`, `SourceAnchor`, and accepted raw
coordinate contracts remain authoritative for exact retained authored source.

They MUST NOT be reinterpreted as constructed-node identity or final tree
placement.

### Tokenizer

The project-owned tokenizer owns input preprocessing, tokenizer state, token
creation, tokenizer diagnostics, tokenizer-local completion/resource evidence,
and exact tokenizer-owned source observations.

It MUST NOT own tree insertion modes, the open-element stack, active formatting
elements, final constructed relationships, or browser-runtime truth.

### Core parse coordinator

The browser-independent Core owns coordination between parse configuration,
token production, specification-required tokenizer control, and tree
construction.

The coordinator MUST preserve lower-layer completion/resource meaning and MUST
NOT introduce browser-protocol behavior or presentation concerns.

### Tree constructor and private construction session

The tree constructor owns the parser state required by supported HTML tree
construction, including insertion-mode state, open elements, formatting/template
state, namespace/foreign-content dispatch, recovery decisions, node creation,
and construction mutation as applicable to the capability.

Mutable construction state MUST remain private to a bounded construction run.
Pointers, arena slots, allocation order, or mutable parser object identities MUST
NOT become durable Analysis Result identity by convenience.

### Result finalization

A finalization boundary validates the supported construction state and freezes
it into immutable analysis meaning. Consumers MUST NOT observe parser-native
mutable internals as the durable result contract.

### Browser adapters and runtime correlation

Browser Adapters own protocol-specific acquisition and normalization under ADR
0008. Runtime observations MAY later be correlated with project-owned tree
analysis, but browser/runtime identity MUST NOT become source origin or
project-owned constructed-node authority.

## Parse Request and Effective Parse Context

Every tree-construction capability MUST declare the parse configuration that can
change semantics. Depending on capability scope this can include document versus
fragment mode, context element and namespace, parser scripting mode, document
mode, or other specification-required context.

Configuration combinations MUST be validated rather than accepted as arbitrary
independent switches when one value is normatively derived from another. For
example, an initial tokenizer state that is derived from fragment context MUST
not be treated as an unrelated user-selected value that can contradict that
context.

Unsupported configuration MUST remain explicit and MUST NOT be translated into
successful analysis of a different configuration.

Parser scripting configuration and JavaScript execution are distinct. Supporting
a scripting mode does not authorize JavaScript execution inside Core.

## Tokenizer / Tree-Constructor Coordination

Full HTML parsing MUST NOT assume that a completed, context-free token vector is
universally sufficient for later tree construction.

The architecture MUST be capable of specification-required coordination where
tree-construction state affects subsequent tokenization. Coordination MAY be
implemented using any private mechanism that preserves the same semantics and
accepted Core boundaries.

Token reprocessing within tree construction MUST NOT require parser-native
mutable state to escape the construction session. A token MAY be processed more
than once by tree-construction rules while remaining one source/token observation
when that is the supported semantic meaning.

Existing bounded capabilities that correctly consume a completed tokenizer run
MAY continue doing so when their own theorem does not require feedback.

## Private Construction-Session Lifecycle

A construction session MUST have one clear owner and a bounded lifetime within a
parse run.

During construction the implementation MAY mutate private tree storage and
parser state as required by the supported algorithms. Those temporary states are
not Analysis Results.

A session MUST preserve enough information to validate every durable observation
that will survive freeze. It MUST NOT recover source provenance later through
source searching, guessed offsets, or reconstructed authored ranges.

Implementation may replace arenas, vectors, stacks, maps, algorithms, or other
private storage without an architecture change when durable meaning and
contracts remain unchanged.

## Valid Construction Checkpoints and Freeze

An immutable result may expose only a state that satisfies the invariants of its
supported capability.

If incomplete/partial output is supported, the exposed snapshot MUST correspond
to a valid semantic construction checkpoint. A stop in the middle of a
multi-step mutation MUST NOT leak a structurally invalid intermediate state.

The implementation may finish an atomic operation, roll it back, retain the
previous valid checkpoint, or use another private mechanism. This contract does
not select that mechanism.

Coverage for a tree result MUST NOT be assumed to be only a source-byte prefix.
Where progress meaning requires parser/construction state, the capability must
record sufficient explicit evidence instead of pretending byte coverage alone is
complete.

## Immutable Query-Oriented Tree Analysis

The durable default Core result is an immutable, browser-independent,
query-oriented representation of the supported constructed meaning.

`Query-oriented` means the result is designed to answer approved analysis
questions without exposing parser mutation mechanics. It does not authorize a
public query API or fix one physical tree representation.

A future DOM-like view MAY be derived when a named consumer requires it. Such a
view must not redefine source provenance, constructed identity, or browser-runtime
authority.

A complete event log is not required by default. Durable recovery/action evidence
SHOULD be selective and sufficient to explain supported observations. A later
consumer may justify a richer trace through a focused architecture decision.

## Constructed-Node Identity Invariants

Every supported constructed node MUST have identity semantics sufficient for
stable relationships within the declared result scope.

Constructed-node identity MUST:

- be distinct from `SourceId`, source ranges, token indexes, browser protocol
  node IDs, and private storage identity;
- be scoped to one parse result or another explicitly approved analysis scope;
- have deterministic semantic meaning for equal source, configuration,
  capability, and implementation revision;
- survive private storage replacement, compaction, or freeze without changing
  merely because storage changed;
- remain the identity of the same constructed observation if final placement is
  changed by supported recovery;
- allow multiple constructed nodes to share one authored source origin; and
- support nodes with no authored source origin.

Constructed-node identity MUST NOT be defined solely by pointer/address, arena
slot, incidental allocation order, authored range, token index, final
parent/child position, content hash, or browser-runtime identity.

The concrete identity encoding remains OPEN. No integer, path, composite key, or
cross-run/cross-edit stability promise is created by this contract.

## Source Origin, Synthesis, Placement, Recovery, and Token Disposition

Durable evidence MUST be sufficient to distinguish why a supported observation
exists and where its authored evidence originated.

### Source origin

An authored origin is exact retained-source evidence. It MUST use accepted source
contracts and MUST NOT be fabricated from normalized meaning or final tree
position.

One constructed node MAY have zero, one, or multiple source origins when the
supported capability proves that relationship. Multiple constructed nodes MAY
share one authored origin, for example when later capabilities support
reconstruction semantics.

### Synthesis

Wholly synthesized structure MUST carry explicit non-authored meaning. It MUST
NOT receive a dummy, inherited, nearest, empty, or guessed `SourceAnchor` merely
to satisfy a uniform field shape.

A token that triggers synthesized structure is not thereby the authored origin
of that structure. Trigger evidence and authored origin are separate concepts.

### Final placement

Constructed parentage/order is result meaning independent of source nesting or
source order. Recovery that changes placement MUST NOT rewrite authored origin.

### Recovery/action evidence

When a supported query requires an explanation of structural recovery, the
result MUST retain sufficient supported evidence to identify the relevant action
or cause. This does not require every temporary parser mutation to become a
durable event.

### Token disposition

A supported token may be relevant even when it produces no node. Ignored,
discarded, reprocessed, unsupported, or stopped input MUST NOT require fake
placeholder nodes solely to make the disposition observable.

## Authored and Synthesized Structure Invariants

The following are mandatory:

1. synthesized nodes have no fabricated authored range;
2. omitted end tags receive no invented end-tag anchor;
3. foster/recovery placement, when later supported, changes constructed
   relationships without erasing authored origin;
4. reconstruction, when later supported, may create distinct constructed
   identities that share authored evidence;
5. source-backed text/provenance, when coalesced by a supported capability, must
   retain sufficient exact authored origin information for accepted queries; and
6. absence of authored provenance is explicit semantic absence, not an empty or
   sentinel source range.

## Diagnostics, Recovery, Completion, Coverage, and Resources

These dimensions are related but MUST remain semantically distinct.

A supported parse MAY be `Complete` while containing parse diagnostics, implied
structure, or supported structural recovery. Recovery is not automatically
partial or incomplete.

A higher layer MUST NOT upgrade required lower-layer incomplete evidence to
complete success.

Resource exhaustion is project implementation policy. It MUST NOT be reported as
proof that source is invalid HTML merely because processing stopped.

Normative algorithm bounds, when the HTML Standard contains them, are semantic
algorithm behavior and MUST NOT be mislabeled as project resource budgets.
Likewise, implementation-specific browser limits MUST NOT be promoted to HTML
Standard constants.

This contract selects no numeric tree depth, node count, mutation count, memory,
stack, or WASM limit.

## Partial, Unsupported, Invalid, Aborted, and Invariant-Failure States

Capabilities MUST distinguish states whose meanings differ.

At minimum, architecture must be able to represent or reject distinctly:

- complete supported analysis;
- unsupported capability/configuration;
- resource-limited incompleteness;
- invalid configuration;
- caller/host abort or cancellation if such a mechanism is later introduced;
- lower-layer incompleteness required by the capability; and
- internal invariant failure.

Unsupported coverage is not evidence that the source is invalid HTML.

Internal invariant failure is not normal source behavior. Panic handling remains
owned by Rust/security contracts and any focused implementation decision; a
caught panic, if future work introduces one, MUST NOT silently become ordinary
unsupported input.

A capability MAY choose not to expose partial tree snapshots. If it does expose
them, the valid-checkpoint rules in this contract apply.

## Runtime DOM Authority and Derived DOM-Like Views

Project-owned tree construction and browser/runtime DOM observation are separate
authority domains.

Browser agreement MAY challenge or corroborate project findings but MUST NOT
replace normative derivation or project-owned source provenance. Runtime
observations retain their own origin, identity, lifecycle, completeness/lossiness,
and correlation basis under ADR 0008.

A derived DOM-like Core view MAY be introduced later if justified by a named
consumer. It MUST remain a view of project-owned analysis meaning and MUST NOT
implicitly adopt browser mutation APIs, protocol identity, live-object
semantics, or full DOM compatibility.

## Capability Envelopes and Extension Rules

Every production tree-construction slice MUST define its exact supported semantic
envelope and candidate-independent expected semantics before implementation.

A slice MUST state relevant parse configuration, supported tokenizer/tree states,
accepted node/provenance observations, diagnostics/recovery meaning, completion
and resource boundaries, and the first excluded capability.

A slice MUST NOT present partial HTML coverage as complete HTML support.
Unsupported adjacent Standard behavior must remain explicit.

Later slices may add fragments, tokenizer-state feedback, foreign content,
tables/foster parenting, formatting reconstruction, adoption-agency behavior,
templates, scripting-dependent semantics, or other features only through focused
validated scope.

The first approved bounded production candidate under #117 is TC-S1,
Disabled-Scripting Document Shell Construction. TC-S1 does not itself prove full
HTML parsing, fragment semantics, scripting/reentrancy, foreign content, table
recovery, formatting reconstruction, adoption-agency behavior, public API
readiness, or universal tree resource policy.

## Compatibility and Evolution

This contract creates no public Rust API, serialization, ABI, browser protocol,
or product compatibility promise.

Private implementation details may evolve without a new ADR when the approved
ownership, provenance, lifecycle, identity, authority, completion, and result
semantics remain unchanged.

Material changes to those durable semantics require the repository's normal
architecture decision process. Public APIs, serialization, new crates,
dependencies, async/concurrency, unsafe Rust, or browser-runtime ownership changes
require their own approval when existing contracts trigger it.

Constructed identities are not promised stable across parse results, process
runs, source edits, or implementation revisions unless a future approved
contract explicitly establishes such compatibility.

## Implementation Gate

This contract does not authorize production merely by existing.

Before a production capability begins, its focused Issue MUST establish placement,
exact supported theorem, ownership, compatibility impact, required validation,
and any unresolved architecture prerequisites. Candidate-independent expected
semantics MUST remain separable from production output.

No public API, serialization, new crate, dependency, async, concurrency, or
repository-authored `unsafe Rust` is authorized by this contract.

## Validation Expectations

Architecture and implementation review for tree-construction work MUST verify, as
applicable:

- exact normative authority and capability configuration;
- candidate-independent expected tree meaning;
- authored versus synthesized provenance honesty;
- deterministic result ordering and constructed identity requirements;
- monotonic lower-layer completion/resource propagation;
- valid partial/checkpoint semantics when partial results exist;
- unsupported boundaries without false invalid-source claims;
- browser/runtime evidence kept in its separate authority domain;
- no accidental public or serialized compatibility commitment; and
- changed-file/dependency/workspace scope against the focused Issue.

External parser/browser corpora MAY be used as challenge evidence after the
project expected semantics are independently derived. Shared corpus lineage MUST
be considered before counting evidence as independent corroboration.

## References

- Issue #117 — HTML tree-construction architecture and maintainer approval
- Issue #348 — post-vertical-slice HTML research / evidence foundation
- Issue #349 — focused ADR 0010 and normative-contract documentation Leaf
- [ADR 0010](../decisions/0010-html-tree-construction-architecture.md)
- [Source Parser Ownership](SOURCE_PARSER_OWNERSHIP.md)
- [Architecture Principles](PRINCIPLES.md)
- [Architecture Layers and Boundaries](LAYERS.md)
- [Rust Core Contracts](RUST_CORE_CONTRACTS.md)
- [HTML research evidence](../evidence/html/README.md)
- [HTML research provenance](../provenance/html.md)
