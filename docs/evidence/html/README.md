# HTML Research Evidence

Status date: 2026-08-27

Classification: task and evidence record; non-normative.

## Current Status

The current HTML evidence checkpoint is:

- [2026-08-27 RAWTEXT Feedback and Post-TC-S8 Evidence Checkpoint](2026-08-27-rawtext-feedback-checkpoint.md).

As of 2026-08-27, production tree construction is merged through TC-S8 at
production semantic baseline `5ffb2eacf0b6cd77b7531a68408cb8e2ceba28b8`.
The repository `main` is `e5c299e3980f6d3de41c6291b86890f19715890d`
after merging the accepted candidate-independent `<style>` RAWTEXT feedback
validation from PR #385. TC-S9 is now the assigned sequence designation and its
production placement is accepted, while the production Issue remains blocked
until the current evidence Leaf is reviewed/merged and production implementation
remains unauthorized.

The older [2026-08 Tree-Construction Frontier Checkpoint](2026-08-tree-construction-frontier-checkpoint.md)
remains a historical TC-S7-validation-era evidence record and is intentionally
not rewritten to reflect later production work.

The sections below preserve the historical first-slice and research/architecture
evidence that led to the current frontier.

## Historical Baseline and Architecture Transition

The first browser-independent HTML source-analysis vertical slice remains
complete. Issue [#116](https://github.com/YT-TechDev/frontend-analysis/issues/116)
was completed through Pull Request
[#131](https://github.com/YT-TechDev/frontend-analysis/pull/131) and squash-merged
to `main` as:

```text
2994ea78907b17cec780a4880547465b5bc1e244
```

The completed bounded operation is conceptually:

```text
&SourceText
+ HtmlTokenizerLimits
        ↓
project-owned HTML tokenizer
        ↓
validated HtmlTokenizerRunResult
        ↓
project-owned explicit-start-tag analysis parser
        ↓
Core validation of projected source-backed occurrence evidence
        ↓
HtmlExplicitStartTagAnalysis
```

This proves one narrow authored-source capability. It does **not** prove complete
HTML Standard parsing, production tree construction, DOM compatibility, or public
API readiness.

After that first slice, the research/architecture program completed the evidence
foundation that later production tree-construction frontiers consumed:

- [#348](https://github.com/YT-TechDev/frontend-analysis/issues/348) completed the
  post-vertical-slice R1–R10 / Wave 1E HTML research program. Its durable
  Research Completion Checkpoint is
  [`issuecomment-5392711890`](https://github.com/YT-TechDev/frontend-analysis/issues/348#issuecomment-5392711890).
- [#117](https://github.com/YT-TechDev/frontend-analysis/issues/117) completed a
  fresh architecture reassessment, candidate-independent TC-S1 validation, and
  explicit maintainer approval of Candidate C / TC-S1. The maintainer decision is
  [`issuecomment-5393598385`](https://github.com/YT-TechDev/frontend-analysis/issues/117#issuecomment-5393598385).
- [ADR 0010](../../decisions/0010-html-tree-construction-architecture.md)
  records the approved architecture rationale.
- [HTML Tree-Construction Architecture](../../architecture/HTML_TREE_CONSTRUCTION.md)
  owns the active specialized normative invariants for the accepted tree-construction
  architecture.

Architecture approval and production implementation remain separate states. At
this historical checkpoint TC-S1 still required its own production-placement
gate; later focused gates and Pull Requests advanced production through TC-S6.
The dated current checkpoint above is authoritative for present production status.

## Authoritative Evidence Sources

Repository evidence:

- [#104 — project-owned lossless parser architecture](https://github.com/YT-TechDev/frontend-analysis/issues/104)
- [#106 — HTML parser workstream](https://github.com/YT-TechDev/frontend-analysis/issues/106)
- [#109–#116 — first HTML tokenizer/parser/Core slice](https://github.com/YT-TechDev/frontend-analysis/issues/116)
- [#112 — candidate-independent validation foundation](https://github.com/YT-TechDev/frontend-analysis/issues/112)
- [#114 — first source-backed HTML analysis-parser model](https://github.com/YT-TechDev/frontend-analysis/issues/114)
- [#117 — tree-construction architecture](https://github.com/YT-TechDev/frontend-analysis/issues/117)
- [#348 — post-vertical-slice HTML research foundation](https://github.com/YT-TechDev/frontend-analysis/issues/348)
- [#349 — ADR 0010 / normative-contract documentation Leaf](https://github.com/YT-TechDev/frontend-analysis/issues/349)
- [PR #131 — completed first Core integration](https://github.com/YT-TechDev/frontend-analysis/pull/131)

Normative external authority for the #348 tree-construction research is pinned in
[HTML research provenance](../../provenance/html.md), including WHATWG HTML commit
`508a037333d8a1806504303aeb489d931fabbef6` and source blob
`68dbcb98bbe1001c6ae2531be2368c608fbafddd`.

Browser or third-party parser behavior is comparison/challenge evidence only.
Current WPT and html5lib tree-construction corpora must not be counted as two
independent confirmations without accounting for their shared lineage.

## Proven Architecture Evidence

### H1 — Retained source is the authority

The integrated slice operates on the exact supplied `SourceText`. Core validates
projected `SourceId`, ranges, retained fragments, and raw tag-name containment
against that source.

The implementation does not establish provenance by:

- searching source text;
- rescanning tag delimiters;
- reconstructing endpoints;
- inferring raw lengths from normalized values; or
- retokenizing source fragments.

This supports the broader invariant that exact source evidence must originate
from owned recognition evidence and remain independently revalidatable.

### H2 — Lower-layer lifecycle meaning remains authoritative

Tokenizer completion, diagnostics, coverage, unsupported capability, and resource
evidence are retained through parser/Core integration rather than translated into
a duplicate higher-level hierarchy.

A higher layer cannot turn an incomplete lower-layer result into complete
success merely because useful occurrences were projected.

### H3 — Capability-specific analysis can precede generic syntax models

The first result answers one explicit source question: recognized authored
start-tag occurrences with exact source evidence. The architecture did not
require a universal HTML AST, DOM model, cross-language event protocol, or
generic `AnalysisResult` foundation first.

This remains an evidence-backed preference for bounded capabilities, not a claim
that HTML will never need a richer tree or syntax representation.

### H4 — Authored syntax and synthesized structure are different domains

The first production slice intentionally stops before HTML tree construction.
The #348/#117 research and architecture work subsequently established the
specialized future tree-construction boundary without changing the existing
operation.

Authored source origin, constructed-node identity, final placement, synthesis
cause, recovery/action evidence, token disposition, and runtime correlation are
not interchangeable domains. A synthesized node must not claim an authored range
that does not exist. Runtime browser DOM observations remain separate evidence
from project-owned source parsing.

### H5 — Crate-private vertical slices reduce premature compatibility commitments

The completed operation remains synchronous and `pub(crate)`. The public-export
delta for the first slice was zero.

This allowed architecture validation without prematurely making tokenizer,
parser, mutable state, or occurrence representation into public compatibility
contracts.

### H6 — Candidate-independent validation is reusable across layers

The same independent evidence foundation was exercised through tokenizer,
analysis parser, and Core integration. Production output did not generate its
own expected oracle.

The TC-S1 architecture-validation gate applied the same discipline to expected
document-shell tree/provenance meaning before production placement. This supports
reusing specification/project-owned gold across layers while keeping each
layer's responsibility independently testable.

## #348 / #117 Tree-Construction Evidence Closure

The #348 research program closed the broad pre-architecture evidence gap and
falsified the following shortcuts for general HTML tree construction:

- universal completed-token-vector → later-tree architecture;
- context-independent tokenization/tree semantics for equal source bytes;
- source/token order as final tree parentage;
- one authored start tag as exactly one final constructed node;
- an exact authored range for every final node;
- simple tag matching/nesting as general HTML tree semantics;
- fragment parsing as document parsing minus implied outer elements;
- foreign content as namespace decoration after ordinary HTML parsing;
- diagnostics-only recovery; and
- browser/runtime DOM as the same authority as project-owned source parsing.

Material corrections were preserved rather than silently overwritten. In
particular, the adoption-agency outer loop has a normative cap of eight, while
`innerLoopCounter > 3` is a state-reduction threshold rather than a hard
three-iteration inner-loop cap. Browser implementation limits such as a fixed
tree depth are not promoted to HTML Standard constants.

The resulting architecture direction approved in #117 is a Core-private
coordinated parser driver with private mutable construction state, validated
freeze, immutable query-oriented tree analysis, and selective provenance/recovery
relations. The architecture does not require a browser-compatible DOM or full
construction-event sourcing.

## Validation Evidence

The #116 completion audit recorded:

- candidate-independent Core gate: **76/76 fixtures** (`72` initial + `4`
  supplemental `REG-` fixtures);
- generated Core gate: **4,096** bounded deterministic inputs, maximum 64 source
  bytes;
- native UTF-8 and raw-spelling vertical slice: **Passed**;
- source-anchor lifetime after the caller `SourceText` handle was dropped:
  **Passed**;
- source identity/range/content/containment corruption checks: **Passed**;
- deterministic repeated-run validation: **Passed**;
- Rust Core workflow run `31241663713`: success;
- CI at completion: **177 passed, 0 failed, 0 ignored**;
- workspace at completion: one package, one workspace member, zero dependencies,
  zero features, one library target; and
- `wasm32-unknown-unknown`: **Not run** because the target was unavailable in the
  execution environment. No WASM runtime claim was made.

These numbers describe the completed bounded slice and are not permanent
repository-wide compatibility promises.

The later TC-S1 candidate-independent architecture-validation gate recorded:

- Candidate C survived all scoped TC-S1 falsification tests;
- constructed-node identity requirements were validated without selecting a
  concrete encoding;
- independently derived document-shell/provenance GOLD was corroborated rather
  than defined by WPT; and
- concrete tree resource constants remained intentionally OPEN.

That validation was architecture evidence rather than a production test result and
did not, by itself, authorize TC-S1 implementation. Later focused placement and
production gates supplied the separate implementation authority.

## Rejected or Unsupported Strong Claims

The current evidence rejects or does not justify the following shortcuts:

- `tokenizer output == final HTML semantic tree`;
- `explicit source tag == runtime/DOM element`;
- `recovered or synthesized structure may reuse a convenient authored range`;
- `parser-native source positions are automatically trusted Core anchors`;
- `higher layers may upgrade incomplete tokenizer evidence`;
- `source/token identity == constructed-node identity`;
- `recovery implies incomplete parsing`;
- `browser agreement establishes Core source/tree provenance`;
- `WPT + current html5lib tree data == two independent semantic votes`;
- `a generic AST/event model must be fixed before a useful analysis capability`;
- `a full DOM-compatible result is required for tree analysis`; and
- `the first start-tag slice or TC-S1 implies complete HTML Standard support`.

## Reusable Lessons for Other Languages

The HTML workstream has established reusable **principles**, not reusable HTML
internals:

1. source-first ownership;
2. exact provenance without rediscovery;
3. explicit bounded capability;
4. monotonic completion/evidence propagation;
5. candidate-independent fixtures before or alongside implementation;
6. deterministic bounded generated validation;
7. crate-private architecture validation before public API commitment;
8. separate ownership for authored and synthesized meaning; and
9. architecture alternatives should be falsified before private implementation
   details become durable contracts.

CSS and ECMAScript must independently prove where these principles apply. HTML
tokenizer states, token types, parser events, and tree semantics must not be
copied across languages by analogy.

## OPEN Research / Architecture / Production Decisions

Broad pre-architecture HTML research under #348 is complete. The following
remain intentionally open, deferred, or separately owned:

- concrete constructed-node identity encoding;
- exact immutable tree storage layout;
- detailed recovery-trace and text-coalescing provenance representation;
- durable token identity for future provenance edges;
- tree-specific resource dimensions and numeric project limits;
- partial-result rollback/checkpoint implementation mechanism;
- cancellation/abort API;
- fragment-context production contract;
- script execution and reentrant parsing implementation;
- runtime DOM correlation contract;
- public HTML API and compatibility commitments;
- serialization/wire formats;
- incremental/streaming parsing;
- browser protocol integration;
- product-facing HTML analysis surfaces; and
- future WASM delivery/runtime contracts.

These OPEN items do not invalidate the approved Candidate C architecture. They
remain subject to focused work when a production capability or named consumer
requires them.

## Production State

At the 2026-08-27 evidence checkpoint:

```text
Architecture direction / Candidate C: APPROVED / UNCHANGED
ADR 0010 / specialized normative contract: ACCEPTED / UNCHANGED
TC-S1 through TC-S8 production: MERGED
current production semantic baseline: 5ffb2eacf0b6cd77b7531a68408cb8e2ceba28b8
current repository main: e5c299e3980f6d3de41c6291b86890f19715890d
RAWTEXT candidate-independent validation: ACCEPTED / MERGED via PR #385
TC-S9 sequence designation: ASSIGNED
TC-S9 production placement: ACCEPTED
TC-S9 production Issue: BLOCKED PENDING CURRENT EVIDENCE LEAF
TC-S9 production implementation: NOT AUTHORIZED
full HTML parser claim: NO
```

See [2026-08-27 RAWTEXT Feedback and Post-TC-S8 Evidence Checkpoint](2026-08-27-rawtext-feedback-checkpoint.md)
for the exact current evidence, falsification results, production-placement
knowledge, and explicit unproved boundaries.

## Evidence-to-Architecture Boundary

This document records what the HTML evidence supports. Normative requirements
are owned by [HTML Tree-Construction Architecture](../../architecture/HTML_TREE_CONSTRUCTION.md)
and the broader architecture contracts it specializes. ADR 0010 preserves the
rationale for that decision.

Future evidence may falsify an invariant, but task/evidence records do not
silently override accepted architecture. A material contradiction must use the
normal maintainer/ADR conflict process.
