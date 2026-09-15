# HTML Research Evidence

Status date: 2026-09-15

Classification: task and evidence record; non-normative.

## Current Status

The current durable HTML evidence checkpoint is:

- [2026-09 Post-TC-S10 Accepted Baseline Checkpoint](2026-09-post-tc-s10-accepted-baseline-checkpoint.md).

The HTML workstream is currently **pauseable and accepted through TC-S10**.
The last accepted HTML semantic production expansion is PR #400, and the durable
post-TC-S10 provenance/resume baseline is:

```text
commit: 77790be7a3e8979985fb6a045b8aff2928a8ea40
tree:   732052dc1e6993baf80f333edc49e549f581714f
subject: docs(html): record TC-S10 research provenance (#401)
```

TC-S9 established selected InHead `<style>` RAWTEXT tree↔tokenizer feedback.
TC-S10 then established selected InHead `<title>` RCDATA plus Named Character
Reference semantics, complete canonical WHATWG Named data, non-committing
maximum-match observation, resource-atomic source consumption, and separate
Style/Title lifecycle ownership.

The later #557 compiler-identity regression-harness hardening did not expand HTML
production semantics or reopen TC-S10.

No TC-S11 or other next HTML successor is preselected by this evidence record.
General RCDATA, `<textarea>`, Numeric Character References, script/reentrant
parsing, fragments, templates, tables/foster parenting, active formatting /
adoption agency, foreign content, runtime DOM correlation, public APIs, and
serialization remain open or deferred until a focused theorem requires them.

The older checkpoints remain historical evidence and are intentionally not
rewritten to pretend later work was known at the time:

- [2026-08-27 RAWTEXT Feedback and Post-TC-S8 Evidence Checkpoint](2026-08-27-rawtext-feedback-checkpoint.md)
- [2026-08 Tree-Construction Frontier Checkpoint](2026-08-tree-construction-frontier-checkpoint.md)

The sections below preserve the historical first-slice and research/architecture
evidence that led to the accepted TC-S10 frontier. The dated 2026-09 checkpoint
above is authoritative for current HTML resume status.

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

Architecture approval and production implementation remain separate states. The
accepted production lineage later advanced incrementally through TC-S10 rather
than turning the broad architecture research into an all-HTML implementation.

## Authoritative Evidence Sources

Repository evidence:

- [#104 — project-owned lossless parser architecture](https://github.com/YT-TechDev/frontend-analysis/issues/104)
- [#106 — HTML parser program / workstream index](https://github.com/YT-TechDev/frontend-analysis/issues/106)
- [#106 post-TC-S10 catch-up checkpoint](https://github.com/YT-TechDev/frontend-analysis/issues/106#issuecomment-5677924039)
- [#109–#116 — first HTML tokenizer/parser/Core slice](https://github.com/YT-TechDev/frontend-analysis/issues/116)
- [#112 — candidate-independent validation foundation](https://github.com/YT-TechDev/frontend-analysis/issues/112)
- [#114 — first source-backed HTML analysis-parser model](https://github.com/YT-TechDev/frontend-analysis/issues/114)
- [#117 — tree-construction architecture and rolling frontier authority](https://github.com/YT-TechDev/frontend-analysis/issues/117)
- [#348 — post-vertical-slice HTML research foundation](https://github.com/YT-TechDev/frontend-analysis/issues/348)
- [#349 — ADR 0010 / normative-contract documentation Leaf](https://github.com/YT-TechDev/frontend-analysis/issues/349)
- [PR #131 — completed first Core integration](https://github.com/YT-TechDev/frontend-analysis/pull/131)
- [PR #389 — accepted TC-S9 Style/RAWTEXT production](https://github.com/YT-TechDev/frontend-analysis/pull/389)
- [#390 / PR #391 — TC-S10 candidate-independent semantic validation](https://github.com/YT-TechDev/frontend-analysis/issues/390)
- [#392 / PR #393 — complete deterministic WHATWG Named-data foundation](https://github.com/YT-TechDev/frontend-analysis/issues/392)
- [#396 / PR #397 — compiler-sealed ownership validation](https://github.com/YT-TechDev/frontend-analysis/issues/396)
- [#398 / PR #399 — production canonical Named-data owner](https://github.com/YT-TechDev/frontend-analysis/issues/398)
- [#394 / PR #400 — accepted TC-S10 production](https://github.com/YT-TechDev/frontend-analysis/issues/394)
- [PR #401 — TC-S10 research provenance completion](https://github.com/YT-TechDev/frontend-analysis/pull/401)
- [#557 — post-TC-S10 compiler-identity validation-harness hardening](https://github.com/YT-TechDev/frontend-analysis/issues/557)

Normative external authority and the exact TC-S10 source/data pins are recorded in
[HTML research provenance](../../provenance/html.md). Browser or third-party
parser behavior remains comparison/challenge evidence only. WPT and html5lib
must not be counted as independent semantic votes without accounting for shared
lineage.

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

The first production slice intentionally stopped before HTML tree construction.
The #348/#117 research and architecture work subsequently established the
specialized tree-construction boundary without changing the existing operation.

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

Later TC-S1–TC-S10 validation retained that discipline at materially load-bearing
frontiers, including tree provenance, recovery, tokenizer↔tree feedback, and
Title/RCDATA/Named semantics.

### H7 — Tokenizer, tree construction, and coordinator own different causal roles

TC-S9 and TC-S10 established that tree construction may need to direct future
tokenizer semantics without taking ownership of tokenizer lexical state.

The accepted split is:

```text
tokenizer
  owns cursor progression, preprocessing, lexical state, appropriate-end-tag
  recognition, Character Reference processing, lexical diagnostics/resources

Tree construction
  owns insertion modes, open elements, constructed nodes, tree recovery/actions,
  final placement, element-specific semantic requests

Core coordinator
  owns causal sequencing of tree feedback before later source production
```

A generic mode-carrying parser-control API is not implied by the selected
feedback seams.

### H8 — Authored source and interpreted text contributions remain distinct

TC-S10 makes this distinction load-bearing. A Named Character Reference may have
one exact authored source contribution while producing one or two interpreted
Unicode scalars. Decoded output is never reintroduced as tokenizer input, and
interpreted `</title>` text is not an authored Title end-tag token.

### H9 — Non-committing observation is not source consumption

Named maximum-match discovery may inspect bounded future source but must not
advance the authoritative cursor, commit preprocessing diagnostics, advance
coverage, or mutate retained evidence. Only selected source units pass through
the authoritative consumption lifecycle.

### H10 — Fallible semantic work must precede authoritative source commitment

TC-S10 reinforced the project-wide commit order:

```text
discover
→ preflight / prepare evidence
→ construct fallible semantic values
→ authoritative source consumption
→ non-refusing commit
```

Resource refusal before consumption preserves prior valid evidence and cannot
leave a partially committed Named reference.

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

## Historical Validation Evidence

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

These numbers describe that completed bounded slice and are not permanent
repository-wide compatibility promises.

The later TC-S1 candidate-independent architecture-validation gate established
that Candidate C survived the scoped falsification program without selecting a
concrete constructed-node identity encoding or pretending the validation gate
itself authorized production.

For current TC-S1–TC-S10 chronology and exact accepted heads, consult the
[2026-09 checkpoint](2026-09-post-tc-s10-accepted-baseline-checkpoint.md), #117,
and the focused Issues/PRs.

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
- `a full DOM-compatible result is required for tree analysis`;
- `a completed Data-state token vector is always sufficient before tree construction`;
- `one generic tokenizer mode switch is implied by selected Style/Title feedback`;
- `decoded character-reference output may be fed back as authored tokenizer input`;
- `Named Character Reference support implies Numeric Character Reference support`;
- `selected Title RCDATA implies general RCDATA or textarea support`; and
- `TC-S10 implies complete HTML Standard support`.

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
8. separate ownership for authored, interpreted, constructed, and runtime meaning;
9. architecture alternatives should be falsified before private implementation
   details become durable contracts; and
10. fallible preparation should precede authoritative consumption when partial
    semantic commit would be dishonest.

CSS and ECMAScript must independently prove where these principles apply. HTML
tokenizer states, token types, parser events, and tree semantics must not be
copied across languages by analogy.

## OPEN Research / Architecture / Production Decisions

Broad pre-architecture HTML research under #348 is complete. The following
remain intentionally open, deferred, or separately owned:

- Numeric Character References;
- RCDATA NUL recovery;
- general RCDATA / `<textarea>` coordination;
- Data-state / AttributeValue character references;
- Script Data and script/reentrant parsing;
- concrete constructed-node identity encoding;
- exact immutable tree storage layout;
- detailed recovery-trace and text-coalescing provenance representation;
- durable token identity for future provenance edges;
- tree-specific resource dimensions and numeric project limits;
- fragment-context production contract;
- templates;
- tables and foster parenting;
- active formatting elements / adoption agency;
- foreign content / namespace integration;
- broader scope and implied-end algorithms;
- runtime DOM correlation contract;
- public HTML API and compatibility commitments;
- serialization/wire formats;
- incremental/streaming parsing;
- browser protocol integration;
- product-facing HTML analysis surfaces; and
- future WASM delivery/runtime contracts.

These OPEN items do not invalidate the approved Candidate C architecture or the
accepted TC-S1–TC-S10 production lineage. They remain subject to focused work
when a concrete capability or named consumer requires them.

## Production State

At the 2026-09-15 evidence checkpoint:

```text
Architecture direction / Candidate C: APPROVED / UNCHANGED
ADR 0010 / specialized normative contract: ACCEPTED / UNCHANGED
TC-S1 through TC-S10 production: ACCEPTED / MERGED
TC-S9 Style/RAWTEXT feedback: ACCEPTED / MERGED via PR #389
TC-S10 Title/RCDATA/Named: ACCEPTED / MERGED via PR #400
TC-S10 research provenance: RECORDED / MERGED via PR #401
post-TC-S10 validation-harness hardening: ACCEPTED / NO SEMANTIC EXPANSION
post-TC-S10 durable resume baseline: 77790be7a3e8979985fb6a045b8aff2928a8ea40
post-TC-S10 durable resume tree: 732052dc1e6993baf80f333edc49e549f581714f
HTML second bounded production wave: PAUSEABLE / CLOSED AT TC-S10
next HTML successor: NOT PRESELECTED
full HTML parser claim: NO
```

See [2026-09 Post-TC-S10 Accepted Baseline Checkpoint](2026-09-post-tc-s10-accepted-baseline-checkpoint.md)
for the concise current capability envelope, accepted lineage, ownership results,
falsified assumptions, and explicit deferred boundaries.

## Evidence-to-Architecture Boundary

This document records what the HTML evidence supports. Normative project
requirements are owned by [HTML Tree-Construction Architecture](../../architecture/HTML_TREE_CONSTRUCTION.md)
and the broader architecture contracts it specializes. ADR 0010 preserves the
rationale for that decision.

Future evidence may falsify an invariant, but task/evidence records do not
silently override accepted architecture. A material contradiction must use the
normal maintainer/ADR conflict process.
