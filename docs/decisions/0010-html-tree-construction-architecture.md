# ADR 0010: Define HTML Tree-Construction Architecture

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-24 |
| Decision owner / approver | YT-TechDev |
| Linked Issue | #349 |
| Related Pull Request | #350 |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | New `docs/architecture/HTML_TREE_CONSTRUCTION.md`; `docs/README.md` authority map. Existing `PRINCIPLES.md`, `LAYERS.md`, `RUST_CORE_CONTRACTS.md`, `SOURCE_PARSER_OWNERSHIP.md`, ADR 0007, and ADR 0008 remain unsuperseded. |

## Context

Frontend Analysis completed its first browser-independent HTML source-analysis
vertical slice without committing to full HTML tree construction. That slice
proved exact authored source evidence, monotonic completion/resource propagation,
and a crate-private Core analysis boundary, but deliberately did not establish
HTML tree semantics, fragment parsing, synthesized structure, browser DOM
compatibility, or a universal parser-result model.

Issue #348 then performed a candidate-independent R1–R10 HTML research program
against a pinned WHATWG HTML source revision. The research falsified several
shortcuts that would otherwise have shaped implementation by convenience:

- complete context-free tokenization cannot be assumed to precede all tree
  construction;
- equal source bytes do not imply equal parsing under different document/fragment
  contexts;
- source order and authored nesting do not determine final tree placement;
- one authored token does not necessarily correspond to one constructed node;
- synthesized structure cannot carry an authored range that does not exist;
- recovery is structural semantics, not diagnostics alone; and
- browser/runtime DOM observations remain comparison evidence rather than
  project-owned source-provenance authority.

The completed #348 research also separated normative algorithm behavior from
implementation resource policy, corrected the adoption-agency `8 / >3`
interpretation, and established that current WPT and html5lib tree-construction
corpora share lineage and therefore cannot be counted as independent semantic
votes without further lineage analysis.

Issue #117 then performed a fresh architecture reassessment. Four materially
different directions were compared: a browser-compatible DOM-like result,
exposed parser-native mutable state, an immutable query-oriented analysis tree
with selective evidence, and an event/evidence-first log with a derived tree.

Candidate C — a Core-private coordinated parser driver, private mutable
construction session, validated freeze, immutable query-oriented tree analysis,
and selective provenance/recovery relations — was selected as the strongest
architecture direction. A focused candidate-independent TC-S1 validation gate
then attempted to falsify that direction using disabled-scripting document-shell
construction. Candidate C survived the gate, including authored/synthesized
provenance and deterministic constructed-identity requirements.

The maintainer explicitly approved Candidate C and TC-S1 in #117. The remaining
architecture requirement before production placement is to record the durable
rationale here and activate a specialized normative contract without freezing
private implementation details.

## Decision

Frontend Analysis adopts the following durable HTML tree-construction
architecture direction:

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

The active specialized normative contract is
[`docs/architecture/HTML_TREE_CONSTRUCTION.md`](../architecture/HTML_TREE_CONSTRUCTION.md).

The decision establishes these durable boundaries:

1. **Core owns tokenizer/tree coordination.** Fuller HTML parsing must be able to
   coordinate tree-construction state with subsequent token production where the
   HTML Standard requires it. `Pull / resumable` is a semantic coordination
   requirement, not a selected Rust interface.
2. **Construction mutation is private.** Open-element stacks, insertion modes,
   active formatting state, template state, namespace dispatch, recovery
   mutation, arenas, and other parser-native mutable structures remain inside a
   bounded construction session.
3. **Durable tree results are immutable and query-oriented.** Consumers receive
   validated analysis meaning rather than mutable parser objects or browser DOM
   compatibility by default.
4. **Provenance domains remain distinct.** Source origin, constructed-node
   identity, final placement, synthesis cause, recovery/action evidence, token
   disposition, and runtime identity/correlation cannot be collapsed when the
   distinction affects supported meaning.
5. **Synthesized structure never fabricates source evidence.** `SourceAnchor`
   remains exact authored-source evidence. Implied/synthesized nodes must carry
   explicit absence of authored origin rather than dummy, inherited, nearest, or
   guessed ranges.
6. **Constructed identity is separate from source and storage identity.** The
   architecture requires deterministic result-scoped constructed-node identity
   meaning, but does not select its concrete encoding.
7. **Recovery, diagnostics, completion, and resources are orthogonal.** A
   supported parse may be complete while containing parse diagnostics, implied
   structure, or structural recovery. Resource exhaustion is project policy and
   is not evidence that source is invalid HTML.
8. **Runtime DOM remains separate authority.** Browser observations may later be
   correlated through the ADR 0008 boundary but cannot redefine project-owned
   source or constructed identity.
9. **Full event sourcing is not the default result architecture.** Selective
   durable evidence is retained to explain supported observations; complete
   mutation replay requires separate justification from a named consumer.
10. **The existing explicit-start-tag capability remains a sibling.** This
    decision does not require rewriting the already accepted batch-tokenizer
    vertical slice around tree construction.
11. **TC-S1 is the first bounded production candidate.** Disabled-Scripting
    Document Shell Construction is approved for the next production-placement
    gate, but this ADR does not itself authorize production implementation.

Concrete constructed-node encoding, storage layout, token identity, detailed
recovery trace schema, text-coalescing representation, numeric tree resource
limits, rollback/checkpoint mechanics, cancellation, WASM runtime policy,
fragment-context representation, script execution/reentrancy, runtime DOM
correlation shape, public APIs, serialization, and final module/file placement
remain deferred.

## Alternatives Considered

### Candidate A — browser-compatible DOM-like result

A DOM-like result offers familiar traversal and resembles the Standard's tree
vocabulary. It was not selected as the default Core result because authored and
non-authored provenance, ignored/discarded input, reconstruction causes,
recovery placement, and source/runtime identity distinctions still require a
substantial sidecar evidence model. It would also create browser-DOM
compatibility pressure before a named Core consumer requires it.

A future derived DOM-like view remains compatible with this decision.

### Candidate B — exposed parser-native mutable tree

Exposing the mutable arena and parser state would minimize short-term translation
cost and make parser implementation inspection convenient. It was rejected
because it couples durable Analysis Results to allocation and algorithm mechanics,
risks accidental pointer/arena identity, exposes invalid intermediate state, and
obstructs replacement of private parser internals.

### Candidate D — event/evidence-first log with derived tree

A complete construction-event log could provide rich causal replay for node
creation, movement, replacement, reconstruction, and ignored input. It was not
selected as the primary architecture because it would introduce a broad event
vocabulary, replay correctness burden, resource overhead, and compatibility
surface before a named analysis consumer requires full mutation history.

Selective event/action evidence may still be used internally or durably where a
supported query requires it.

### Extend the completed-token-vector start-tag slice directly into a nesting tree

Rejected. The #348 research demonstrated tokenizer/tree feedback, non-local
recovery, implied/synthesized structure, context-sensitive parsing, and
source-versus-final-placement distinctions that cannot be safely generalized
from the existing bounded start-tag pipeline.

### Make browser/runtime DOM the Core tree authority

Rejected. Runtime DOM observations are valuable differential and correlation
evidence, but adopting them as Core source/tree authority would violate browser
independence and erase the distinction between retained authored evidence,
project-owned construction semantics, and engine-specific runtime state.

## Consequences

### Positive

- Core remains browser independent while still modeling HTML tree-construction
  semantics.
- Mutable parser state can evolve without becoming a consumer compatibility
  surface.
- Authored, synthesized, reconstructed, moved/recovered, ignored/discarded, and
  runtime-derived evidence remain representable without fabricated source
  ranges.
- A query-oriented immutable result can support analysis consumers without
  committing to a complete DOM API.
- Candidate-independent validation remains possible because expected tree and
  provenance meaning are defined outside production output.
- Recovery, diagnostics, completion, and resource semantics can evolve without
  collapsing into one ambiguous success/failure state.
- Future fragment, foreign-content, table, formatting, scripting, and runtime
  correlation slices can extend the architecture incrementally.
- Existing explicit-start-tag analysis remains valid and does not require
  migration.

### Negative

- Future tree-construction implementations must maintain an explicit
  construction/freeze boundary instead of returning parser-native objects.
- Provenance and constructed identity require dedicated semantic concepts beyond
  a simple DOM-shaped node list.
- Selective recovery evidence requires deliberate consumer-driven scope rather
  than retaining either nothing or every parser mutation.
- Partial-result support, if later introduced, requires valid semantic
  checkpoints rather than arbitrary stop positions.
- Some concrete implementation decisions remain intentionally open and require
  later focused work.

### Risks

The contract could be over-interpreted as requiring a specific Rust pull-parser
interface. Mitigation: both this ADR and the specialized contract define only
semantic coordination and leave private interface mechanics replaceable.

`Query-oriented` could be misread as a public query API commitment. Mitigation:
the decision selects durable result meaning only and creates no public API.

Constructed identity requirements could accidentally freeze an integer or path
encoding. Mitigation: encoding, cross-run stability, and storage representation
remain explicitly open.

Selective evidence could become either insufficient for later diagnostics or a
hidden full event log. Mitigation: each capability must define the evidence
needed by its accepted observations; richer traces require a named consumer and
focused review.

Resource limits could be mistaken for HTML semantic constants. Mitigation: the
contract separates normative algorithm bounds, project resource policy, and
external implementation limits and selects no numeric tree limits.

### Reversibility

Private implementation remains highly replaceable. Tokenizer interfaces, tree
storage, arenas, algorithms, freeze mechanics, ID encoding, and module layout may
change without a new ADR when durable ownership, provenance, identity,
completion, authority, and result semantics remain unchanged.

Replacing the coordinated Core ownership model, exposing parser-native mutable
state as the durable result, adopting browser DOM as Core authority, collapsing
source/constructed/runtime identity, permitting fabricated provenance, or
changing the immutable result lifecycle would be material architecture changes
and require a new ADR or explicit supersession.

## Compatibility and Migration

No current public Rust API, serialization, ABI, browser protocol, workspace
member, dependency, or product contract changes by accepting this decision.

The existing explicit-start-tag operation remains crate-private and remains a
sibling bounded capability. Its authored occurrence identity and retained token
index do not migrate into constructed-node identity.

ADR 0007 remains authoritative for project-owned parser strategy. ADR 0008
remains authoritative for browser-runtime evidence normalization and import.
This ADR specializes their application to HTML tree construction and does not
supersede them.

Constructed identity is scoped to a result or another explicitly approved
analysis scope. No promise is made that identities remain stable across parse
results, process runs, source edits, implementation revisions, serialization, or
browser runtime objects.

No migration of production code is required by this documentation acceptance.
TC-S1 still requires a separate production-placement gate before implementation.

## Security and License Impact

No dependency, unsafe Rust, FFI, process boundary, browser protocol library,
serialization format, async runtime, or license change is selected.

HTML source is untrusted input. Future implementation remains subject to
[Secure Development](../development/SECURE_DEVELOPMENT.md),
[Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md), and the specialized
[HTML Tree-Construction Architecture](../architecture/HTML_TREE_CONSTRUCTION.md)
for resource exhaustion, panic/invariant handling, provenance honesty, private
mutation, and incomplete-result behavior.

The repository remains MIT licensed. No third-party parser or runtime code is
introduced by this decision.

## Validation

Acceptance is supported by the durable #348/#117 authority chain:

1. **Passed** — #348 R1–R10 / Wave 1E research completed with no material
   contradiction required before architecture reassessment;
2. **Passed** — #117 fresh reassessment compared materially distinct candidates
   and selected Candidate C as the strongest technical direction;
3. **Passed** — the TC-S1 candidate-independent gate derived expected semantics
   independently from production output and Candidate C survived all scoped
   falsification tests;
4. **Passed** — constructed-node identity requirements were validated without
   freezing a concrete encoding;
5. **Passed** — the maintainer explicitly approved Candidate C and TC-S1 in #117;
6. **Passed** — existing `PRINCIPLES.md`, `LAYERS.md`, `RUST_CORE_CONTRACTS.md`,
   `SOURCE_PARSER_OWNERSHIP.md`, ADR 0007, and ADR 0008 were audited with no
   architecture contradiction found;
7. **Passed** — the dedicated normative-contract boundary was defined separately
   from this ADR's rationale;
8. **Passed** — WPT remained challenge/corroboration evidence rather than semantic
   authority for project provenance; and
9. **Passed** — no public API, serialization, dependency, crate, async,
   concurrency, unsafe, or production tree-construction implementation is
   authorized by this acceptance.

Pull Request #350 must still pass focused documentation review, link/structure
validation, exact changed-file scope, and repository CI applicable to the final
diff before merge.

## Follow-Up

After this ADR and the specialized normative contract merge:

1. record the documentation merge/completion link in #117 without rewriting its
   historical Issue body;
2. run the focused TC-S1 Disabled-Scripting Document Shell Construction
   production-placement gate;
3. create a production implementation Issue only if that placement gate accepts
   exact placement and validation scope;
4. address deferred constructed identity/storage/resource/partial/cancellation,
   fragment, scripting, runtime-correlation, public API, serialization, and WASM
   questions only through their owning focused work when required; and
5. add later tree-construction semantic slices incrementally with
   candidate-independent expected semantics.

No production implementation begins automatically from this ADR.

## Approval

Approved by maintainer `YT-TechDev` on 2026-08-24 in Issue #117:

https://github.com/YT-TechDev/frontend-analysis/issues/117#issuecomment-5393598385

The approval explicitly adopts Candidate C as the HTML tree-construction
architecture direction, adopts TC-S1 as the first bounded production candidate,
requires ADR 0010 and a dedicated normative HTML tree-construction contract, and
keeps production placement blocked until those documentation requirements are
accepted.

## References

- Issue #348 — HTML post-vertical-slice R1–R10 research foundation
- #348 Research Completion Checkpoint — `issuecomment-5392711890`
- Issue #117 — HTML tree-construction architecture
- #117 Fresh Architecture Reassessment — `issuecomment-5393149562`
- #117 TC-S1 Candidate-Independent Validation — `issuecomment-5393575798`
- #117 Maintainer Architecture Decision — `issuecomment-5393598385`
- #117 ADR / Normative Contract Definition Checkpoint — `issuecomment-5393819455`
- Issue #349 — focused documentation implementation
- Pull Request #350 — ADR 0010 and normative-contract activation
- [HTML Tree-Construction Architecture](../architecture/HTML_TREE_CONSTRUCTION.md)
- [HTML research evidence](../evidence/html/README.md)
- [HTML research provenance](../provenance/html.md)
- [Source Parser Ownership](../architecture/SOURCE_PARSER_OWNERSHIP.md)
- [Architecture Principles](../architecture/PRINCIPLES.md)
- [Architecture Layers and Boundaries](../architecture/LAYERS.md)
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md)
- [ADR 0007 — Own Lossless Source Parsers](0007-own-lossless-source-parsers.md)
- [ADR 0008 — Define Browser Runtime Evidence Normalization and Core Import Ownership](0008-browser-runtime-evidence-normalization-and-core-import.md)
