# HTML Research Evidence

Status date: 2026-08-12

Classification: task and evidence record; non-normative.

## Current Status

The first browser-independent HTML source-analysis vertical slice is complete.
Issue [#116](https://github.com/YT-TechDev/frontend-analysis/issues/116) was
completed through Pull Request
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
HTML Standard parsing, tree construction, DOM compatibility, or public API
readiness.

## Authoritative Evidence Sources

Repository evidence:

- [#104 — project-owned lossless parser architecture](https://github.com/YT-TechDev/frontend-analysis/issues/104)
- [#106 — HTML parser workstream](https://github.com/YT-TechDev/frontend-analysis/issues/106)
- [#109–#116 — first HTML tokenizer/parser/Core slice](https://github.com/YT-TechDev/frontend-analysis/issues/116)
- [#112 — candidate-independent validation foundation](https://github.com/YT-TechDev/frontend-analysis/issues/112)
- [#114 — first source-backed HTML analysis-parser model](https://github.com/YT-TechDev/frontend-analysis/issues/114)
- [#117 — tree-construction architecture boundary](https://github.com/YT-TechDev/frontend-analysis/issues/117)
- [PR #131 — completed first Core integration](https://github.com/YT-TechDev/frontend-analysis/pull/131)

Normative external authority for HTML behavior remains the WHATWG HTML Standard
and its referenced Infra/Encoding specifications. Browser or third-party parser
behavior is comparison evidence only.

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

The first result answers one explicit source question: recognized authored start
-tag occurrences with exact source evidence. The architecture did not require a
universal HTML AST, DOM model, cross-language event protocol, or generic
`AnalysisResult` foundation first.

This remains an evidence-backed preference for bounded capabilities, not a claim
that HTML will never need a richer tree or syntax representation.

### H4 — Authored syntax and synthesized structure are different domains

The first slice intentionally stops before HTML tree construction. Matching,
nesting, implied elements, foster parenting, formatting-element reconstruction,
foreign-content integration, and synthesized-node provenance remain owned by the
separate tree-construction architecture in #117.

A synthesized node must not claim an authored range that does not exist.
Runtime browser DOM observations are also separate evidence from project-owned
source parsing.

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

This supports reusing specification/project-owned gold across layers while
keeping each layer's responsibility independently testable.

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

## Rejected or Unsupported Strong Claims

The current evidence rejects or does not justify the following shortcuts:

- `tokenizer output == final HTML semantic tree`;
- `explicit source tag == runtime/DOM element`;
- `recovered or synthesized structure may reuse a convenient authored range`;
- `parser-native source positions are automatically trusted Core anchors`;
- `higher layers may upgrade incomplete tokenizer evidence`;
- `a generic AST/event model must be fixed before a useful analysis capability`;
- `agreement with one parser/browser is sufficient correctness evidence`; and
- `the first start-tag slice implies complete HTML Standard support`.

## Reusable Lessons for Other Languages

The HTML workstream has established reusable **principles**, not reusable HTML
internals:

1. source-first ownership;
2. exact provenance without rediscovery;
3. explicit bounded capability;
4. monotonic completion/evidence propagation;
5. candidate-independent fixtures before or alongside implementation;
6. deterministic bounded generated validation;
7. crate-private architecture validation before public API commitment; and
8. separate ownership for authored and synthesized meaning.

CSS and ECMAScript must independently prove where these principles apply.
HTML tokenizer states, token types, parser events, and tree semantics must not be
copied across languages by analogy.

## OPEN Research / Architecture

The following remain intentionally open or separately owned:

- complete HTML tokenizer coverage;
- HTML tree construction under #117;
- authored/implied/synthesized/reconstructed/foster-parented node identity;
- fragment parsing configuration;
- browser-runtime DOM correlation;
- public HTML API and compatibility commitments;
- serialization/wire formats;
- incremental/streaming parsing;
- browser protocol integration;
- product-facing HTML analysis surfaces; and
- future WASM delivery contracts.

## Evidence-to-Architecture Boundary

This document records what the current HTML evidence supports. Any promotion of
these findings into new normative architecture, public API, crate layout, or
compatibility policy requires the normal maintainer/ADR process.
