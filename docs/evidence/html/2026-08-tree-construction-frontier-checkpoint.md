# HTML Tree-Construction Evidence Checkpoint

Status date: 2026-08-26

Classification: task and evidence record; non-normative.

## Purpose and Status

This record preserves the HTML evidence that is actually established in the
repository as of the status date. It updates the evidence-layer current position
without rewriting the historical first tokenizer/parser/Core vertical-slice
record in [README.md](README.md).

This record does **not** amend or override:

- [ADR 0007 — Own Lossless Source Parsers](../../decisions/0007-own-lossless-source-parsers.md);
- [ADR 0010 — Define HTML Tree-Construction Architecture](../../decisions/0010-html-tree-construction-architecture.md);
- [HTML Tree-Construction Architecture](../../architecture/HTML_TREE_CONSTRUCTION.md);
- [Source Parser Ownership](../../architecture/SOURCE_PARSER_OWNERSHIP.md); or
- durable maintainer decisions recorded on repository Issues and Pull Requests.

The evidence boundary remains:

```text
Evidence record
!= normative architecture
!= production authorization
!= provenance ledger
!= browser/runtime DOM authority
```

## Exact Repository State

Current repository main after the TC-S7 validation merge:

```text
main:
5e9ff4d539ebc75a7f812185ef55f0e85e881e72

tree:
503cc2fc549638ac58cfb2931a268193a27bbb39

HEAD subject:
test(html): pin selected in-body body-end transition semantics (#375)
```

The current production semantic baseline remains TC-S6 at:

```text
production commit:
5d0366b21dda5847062f7dadfe368129b6d33f0a

production tree:
5fd79a4988fd2b338bdaabf65b554d8359eedf1b
```

Both merge commits are GitHub-verified. The TC-S7 merge adds validation-only
evidence and does not change production tree-construction semantics.

Durable program authorities:

- [#348 — post-vertical-slice HTML research foundation](https://github.com/YT-TechDev/frontend-analysis/issues/348) — OPEN research/evidence authority;
- [#117 — HTML tree-construction architecture](https://github.com/YT-TechDev/frontend-analysis/issues/117) — OPEN architecture/program authority;
- [ADR 0010](../../decisions/0010-html-tree-construction-architecture.md) — accepted Candidate C architecture;
- [HTML Tree-Construction Architecture](../../architecture/HTML_TREE_CONSTRUCTION.md) — specialized normative contract.

## Normative Research Authority and Freshness

The accepted tree-construction research pin remains:

```text
WHATWG HTML commit:
508a037333d8a1806504303aeb489d931fabbef6

source blob:
68dbcb98bbe1001c6ae2531be2368c608fbafddd
```

Freshness was rechecked during the post-TC-S6 frontier reassessment against:

```text
WHATWG HTML main:
ae6c5d8ddfe6c819730f8f766d550dd1417e66c9

WPT master:
719d5e38fdd0903a18ed9007aba816c98cc491e0
```

There is real parser-path drift after the earlier freshness checkpoint. In
particular WHATWG commit
`ca693db19248e5617c1134ae5d5e7c6adc99f47f` changes parser element insertion
when custom-element reactions can reparent nodes, with corresponding WPT
challenge evidence at `c0cecef6bc584af0f4959ae006d1e8f0d8ef7ea7`.

That change was classified as **non-material to the selected TC-S7 validation
candidate**, because TC-S7 performs no element creation/insertion and remains in
the disabled-scripting bounded theorem. This is not a claim that upstream HTML
parsing had no changes. Future candidates involving insertion, custom elements,
script execution, or reentrancy must reassess the change rather than inherit the
TC-S7 classification.

## Historical First Vertical Slice Remains Established

The original source-analysis vertical slice remains valid evidence:

```text
retained SourceText
  -> project-owned bounded HTML tokenizer
  -> validated tokenizer result
  -> source-backed explicit-start-tag analysis parser
  -> Core source-evidence reconciliation
```

It established, within its bounded capability:

- exact source-backed token and occurrence evidence;
- raw spelling distinct from interpreted meaning;
- candidate-independent tokenizer GOLD and generated validation;
- monotonic completion/resource/unsupported propagation;
- no source rediscovery at the Core boundary; and
- a browser-independent internal analysis path.

It did **not** prove tree construction, full HTML parsing, fragment parsing,
runtime DOM equivalence, or context-independent tokenization as a universal
architecture. Those historical limits remain important even though the tree
program subsequently advanced.

## Candidate C Architecture Evidence

Research #348 and the #117 decision sequence selected Candidate C:

```text
SourceText + Parse Configuration
        ↓
Core-owned Parse Coordinator
        ↕
Tokenizer
        ↓
Private mutable Tree-Construction Session
        ↓
Validated Freeze
        ↓
Immutable query-oriented tree analysis
        + selective provenance / recovery evidence
```

Production TC-S1 through TC-S6 have exercised this ownership model without
requiring the Core to expose parser-native mutable state or browser DOM identity.

The durable evidence distinctions currently exercised include:

```text
authored source origin
!= synthesized origin absence
!= constructed-node identity
!= action / recovery trigger
!= final placement
!= runtime/browser identity
```

## Merged Tree-Construction Production Progression

The merged production progression is:

| Frontier | Established production meaning | Production PR | Squash merge |
| --- | --- | --- | --- |
| TC-S1 | Disabled-scripting document shell construction; authored/synthesized shell provenance; validated freeze; result-scoped semantic creation identity | [#352](https://github.com/YT-TechDev/frontend-analysis/pull/352) | `2e75bbf6819cee2758f87c27cd4195894caef39d` |
| TC-S2 | Selected AfterBody uniform character-run handling; whitespace delegation; non-whitespace same-token reprocess; mixed aggregate refusal | [#356](https://github.com/YT-TechDev/frontend-analysis/pull/356) | `c497afb427b32f11368c1e08f7eac83bb0ede50e` |
| TC-S3 | Selected no-attribute InBody `div` construction and matching/unmatched end handling in a closed authored-only selected domain | [#360](https://github.com/YT-TechDev/frontend-analysis/pull/360) | `6615505b1529e3b94878d2fa274d8018f5819f77` |
| TC-S4 | Closed selected domain extended to exactly `Div | Section`; nearest same-name target; heterogeneous current-first recovery distinct from matching closure | [#364](https://github.com/YT-TechDev/frontend-analysis/pull/364) | `394f0501d234ac98a54a82e30fd9c200435b0c77` |
| TC-S5 | Separate bounded Paragraph domain; authored P, start-triggered P closure, unmatched-`</p>` source-less synthesis, P-specific freeze validation | [#368](https://github.com/YT-TechDev/frontend-analysis/pull/368) | `85f96b473b7fc0c251e2a58a3492564ebc6d8577` |
| TC-S6 | Selected `</div>` / `</section>` over current P; bounded non-noop implied-P pop; target resolution before mutation; composition with TC-S4 recovery | [#372](https://github.com/YT-TechDev/frontend-analysis/pull/372) | `5d0366b21dda5847062f7dadfe368129b6d33f0a` |

This table records bounded production frontiers. It is not a claim that each row
implements the full corresponding HTML Standard algorithm family.

## Current Merged Bounded Tree Theorem

The strongest merged selected stack theorem is:

```text
[html, body] ++ B* ++ P?

B in {Div, Section}
count(P) <= 1
P present => P is current
```

Within the accepted cells, evidence supports all of the following.

### Authored and synthesized structure remain distinct

- `Div | Section` selected ordinary nodes are authored-only.
- Paragraph is a separate domain rather than a widened selected-ordinary type.
- unmatched authored `</p>` can synthesize a source-less P with explicit
  synthesis cause;
- the unmatched end tag is trigger / diagnostic / closure evidence, never the
  synthesized node's authored origin;
- omitted or implied actions do not receive fabricated end-tag anchors.

### Matching closure, recovery, and implied pop are distinct relations

- a selected element's own matching end tag is matching-closure evidence;
- TC-S4 ancestor-end recovery pops intervening selected nodes without
  fabricating matching end tags for them;
- TC-S5 Paragraph closure causes remain distinct from selected recovery;
- TC-S6 P implied-pop caused by a selected target end tag is neither Paragraph
  matching closure nor TC-S4 selected recovery.

### Refusal is transactional

The validated architecture repeatedly requires unsupported or excluded cells to
be selected before mutation. Focused tests exercise zero mutation, zero identity
admission, unchanged stack/mode/evidence, and exact unsupported trigger behavior
for bounded exclusions.

This is a local theorem over the represented state. It does not establish a
universal rollback mechanism for arbitrary future HTML algorithms.

### Constructed identity is semantic, not storage/source identity

`HtmlConstructedNodeId` is used as result-scoped committed semantic
creation-event identity. Validation has challenged private storage ordering,
SourceId variation, recovery placement, synthesized nodes, and identity
non-allocation for close/recovery/ignore actions.

No cross-run, cross-edit, serialized, browser-node, or public stable encoding is
established.

### Freeze is an independent validation boundary

Across the accepted frontiers, freeze validation has been strengthened to
correlate durable relations with retained tokenizer evidence and reject corrupt
identity, trigger, ordering, final-open-state, diagnostic, synthesis, closure,
and recovery combinations.

A green construction session is not by itself the durable-result oracle.

### Diagnostics, recovery, and completion remain orthogonal

Supported recovery may coexist with `Complete`. Lower-layer tokenizer
incompleteness is never upgraded by tree construction. Unsupported tree cells
remain incomplete rather than becoming false clean absence.

## TC-S7 — Accepted and Merged Validation, Production Still Unimplemented

The fresh post-TC-S6 reassessment selected:

**TC-S7 — Selected In-Body `</body>` Transition over the Open Bounded Stack with
Stack Preservation and After-Body Successor Composition**.

Durable selection checkpoint:
[#117 issuecomment-5423488628](https://github.com/YT-TechDev/frontend-analysis/issues/117#issuecomment-5423488628).

Candidate-independent validation authority:

```text
Issue:
#374

PR:
#375

reviewed head:
2b5a41749ee7cd32e4504b6682aeecc415937c9f

reviewed tree:
503cc2fc549638ac58cfb2931a268193a27bbb39

independent exact-head review:
5029212073

official exact-head CI:
Rust Core #587 — PASS

squash merge:
5e9ff4d539ebc75a7f812185ef55f0e85e881e72
```

Validation verdict:

```text
A. TC-S7 CANDIDATE-INDEPENDENT VALIDATION ACCEPTED — PLACEMENT GATE READY
```

At this checkpoint PR #375 is **merged** into `main` as
`5e9ff4d539ebc75a7f812185ef55f0e85e881e72`. Therefore:

```text
TC-S7 candidate-independent validation: ACCEPTED / MERGED
TC-S7 validation evidence on main:      YES
TC-S7 production placement:             NOT YET DECIDED
TC-S7 production implementation:        NOT AUTHORIZED
TC-S7 production behavior on main:      NO
```

The validation proves, for its closed candidate state, that authored InBody
`</body>` preserves the bounded open stack, distinguishes P-only from non-empty
`Div | Section` diagnostic cardinality, and composes the retained current node
with accepted AfterBody EOF / whitespace delegation / non-whitespace same-token
reprocess semantics. It does not convert that validation into production
behavior.

## Strong Claims Not Established

The current evidence does **not** establish:

- full HTML tokenization or full HTML parsing;
- general InBody coverage;
- arbitrary HTML element-name support;
- a generic HTML scope or button-scope engine;
- a generic implied-end generator;
- production TC-S7 `</body>`-over-open-stack handling;
- InBody `</html>` over the bounded open stack;
- arbitrary shell/open-stack crossings;
- list-item / `dd` / `dt` algorithms;
- active formatting elements or adoption agency;
- tables, implied table structure, or foster parenting;
- templates or template insertion modes;
- SVG/MathML foreign content or namespace switching;
- fragment parsing or context-element semantics;
- RCDATA/RAWTEXT/script-data/PLAINTEXT tree-directed tokenizer feedback;
- JavaScript execution, parser reentrancy, custom-element reaction semantics, or
  browser runtime parsing behavior;
- a browser-compatible DOM result;
- public HTML tree API or serialization;
- runtime DOM correlation semantics;
- universal numeric tree depth/node/work/memory limits;
- cancellation/resume semantics; or
- complete WASM runtime policy.

RCDATA/RAWTEXT or an equivalent future tokenizer-feedback slice remains a
strategically important architecture milestone; the current batch-tokenizer
success of the bounded tree slices does not prove tree/tokenizer independence in
general.

## Current Evidence Status

```text
first tokenizer/parser/Core vertical slice: ESTABLISHED
#348 R1–R10 / Wave 1E research:          COMPLETE
Candidate C architecture / ADR 0010:     ACCEPTED
TC-S1 through TC-S6 production:          MERGED
current repository main:                 5e9ff4d539ebc75a7f812185ef55f0e85e881e72
current production semantic baseline:    5d0366b21dda5847062f7dadfe368129b6d33f0a
TC-S7 validation:                        ACCEPTED / MERGED
TC-S7 production placement:              NOT YET DECIDED
TC-S7 production implementation:         NOT AUTHORIZED
full HTML parser claim:                   NO
```

## Update Rule

Add a new dated evidence record when a later frontier materially changes the
supported theorem, upstream-relevance classification, provenance/recovery model,
or research status. Do not rewrite this record to make validation-only evidence or a future
candidate look historically production-complete.
