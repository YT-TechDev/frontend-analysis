# HTML Post-TC-S10 Accepted Baseline Checkpoint

Status date: 2026-09-15

Classification: task and evidence record; non-normative.

## Purpose

This document is the durable resume point for the HTML workstream after the
second bounded production wave closed through TC-S10.

It does not replace the detailed research, architecture, validation, placement,
review, or provenance records owned by their focused authorities. It summarizes
what is accepted, what was falsified, which ownership boundaries survived, and
which HTML domains remain deliberately open so that later work can resume without
reconstructing the entire Issue/PR history.

The current live repository head when this checkpoint was prepared is:

```text
main: 10a203133426975cadc5f410dea521e3f869264f
tree: 6e210b7b0b4bd3071a0af6f73b93e7e4006c2bb8
```

That live head contains later CSS and ECMAScript work and is recorded only to
locate this checkpoint in repository history. It is not an HTML semantic
baseline by itself.

The accepted post-TC-S10 HTML resume baseline is:

```text
commit: 77790be7a3e8979985fb6a045b8aff2928a8ea40
tree:   732052dc1e6993baf80f333edc49e549f581714f
subject: docs(html): record TC-S10 research provenance (#401)
```

The immediately preceding semantic production merge is PR #400:

```text
merge commit: fd8789dd853069b4126e429763c2666bd70eff6c
reviewed head: b88929450066034e9859f67fec89cc078f14657a
reviewed tree: 07cdc10a993aec34f4bee07e9430b4193b9e7ea7
```

PR #401 then added provenance only. The resulting tree was later reused as the
trusted post-TC-S10 HTML baseline during unrelated CSS rollback work.

## Durable Authority Map

The accepted authority chain is:

```text
#106
HTML program / workstream index
        ↓
#348
post-vertical-slice HTML research / falsification foundation
        ↓
#117
HTML tree-construction architecture + rolling frontier / placement authority
        ↓
focused validation / evidence / production Leaves
        ↓
accepted main
```

Repository architecture contracts:

- [ADR 0010 — HTML tree-construction architecture](../../decisions/0010-html-tree-construction-architecture.md)
- [HTML Tree-Construction Architecture](../../architecture/HTML_TREE_CONSTRUCTION.md)

Research/source provenance:

- [HTML provenance ledger](../../provenance/html.md)

Program-level catch-up index:

- [#106 HTML Program Catch-up Checkpoint](https://github.com/YT-TechDev/frontend-analysis/issues/106#issuecomment-5677924039)

Important ownership distinction:

- #348 explains why the broad research conclusions are justified;
- #117 owns the accepted Candidate C architecture and the detailed rolling
  frontier/placement history;
- focused Issues and PRs own candidate-independent and production theorems;
- `docs/provenance/html.md` records external sources actually used;
- this checkpoint is the concise accepted-state resume record.

## Phase 1 — Project-Owned Source Analysis Foundation

The first browser-independent vertical slice completed under #109–#116 and PR
#131. It established:

```text
SourceText
→ project-owned tokenizer
→ project-owned source-backed HTML analysis parser
→ Core source/evidence validation
→ bounded analysis result
```

Durable principles from that slice remain active:

- retained UTF-8 source is authoritative;
- `SourceId` and zero-based half-open byte ranges remain exact evidence;
- source evidence originates during owned recognition;
- source search, rescanning, retokenization, endpoint reconstruction, and
  decoded-length inference are not provenance mechanisms;
- lower-layer incomplete/resource/unsupported state cannot be upgraded by a
  higher layer;
- candidate-independent expected semantics are not derived from production
  output; and
- the first useful slice did not require public parser APIs, serialization,
  async/concurrency, unsafe Rust, or browser-runtime coupling.

## Phase 2 — Broad Post-Vertical-Slice Research

Issue #348 completed the R1–R10 / Wave 1E research program and supplied the
falsification foundation for later tree construction.

Material conclusions preserved from that research include:

- a universal completed-token-vector → later-tree architecture is insufficient;
- equal source bytes do not imply equal parsing semantics independent of
  document/fragment/context/scripting configuration;
- source/token order does not determine final tree parentage;
- one authored start tag is not equivalent to one constructed node;
- not every constructed node has an authored source range;
- simple tag matching/nesting is not a complete model of HTML recovery;
- authored, synthesized, recovered/moved, ignored/discarded, and runtime-observed
  evidence remain distinct;
- WPT/html5lib/browser/parser observations are challenge/corroboration evidence,
  not source-provenance authority; and
- resource policy is project policy, not an HTML-validity rule.

Historical overstatements were corrected rather than silently rewritten. In
particular, adoption-agency progress limits and implementation-specific tree
limits were not promoted to universal HTML Standard rules.

## Phase 3 — Candidate C Tree-Construction Architecture

Issue #117 consumed the #348 evidence and accepted Candidate C:

```text
SourceText + Parse Configuration
        ↓
Core-owned Parse Coordinator
        ↕
private / resumable tokenizer
        ↓
private mutable tree-construction session
        ↓
validated freeze
        ↓
immutable query-oriented tree analysis
+
selective provenance / recovery relations
```

The architecture preserves at least:

```text
source origin
!= constructed identity
!= final placement
!= synthesis cause
!= recovery/action evidence
!= runtime identity
```

The tokenizer owns lexical state and source cursor progression. Tree
construction owns insertion modes, open-element state, constructed-node meaning,
and tree recovery. The Core coordinator owns causal sequencing between the two.
Browser/runtime DOM remains a separate authority.

## Phase 4 — Bounded Tree-Construction Production Through TC-S8

The accepted lineage advanced incrementally rather than attempting full HTML in
one implementation.

### TC-S1 — disabled-scripting document shell

Production: #351 / PR #352.

Established the first Candidate C production slice, including authored versus
synthesized shell distinction, deterministic constructed identity, validated
freeze, exact source provenance, and separation of diagnostics/completion.

### TC-S2 — selected AfterBody uniform character-run handling

Validation: #353 / PR #354. Production: #355 / PR #356.

Added bounded same-token reprocessing/recovery while retaining whole-token source
evidence.

### TC-S3 — selected InBody no-attribute `div`

Validation: #357 / PR #358. Production: #359 / PR #360.

Added the first bounded selected ordinary authored-element domain without
turning shell representation into a speculative generic element model.

### TC-S4 — heterogeneous `div` / `section` closure recovery

Validation: #361 / PR #362. Production: #363 / PR #364.

Established nearest selected target semantics and kept matching closure distinct
from intervening recovery-pop evidence.

### TC-S5 — selected `p` lifecycle

Validation: #365 / PR #366. Production: #367 / PR #368.

Established a dedicated Paragraph domain with authored P, source-less synthesized
P for unmatched `</p>`, distinct closure causes, and P-specific EOF behavior.

### TC-S6 — selected ordinary end tag over current P

Validation: #369 / PR #370. Production: #371 / PR #372.

Established bounded non-noop implied-end composition: target existence is
resolved before mutation; an implied P pop remains distinct from selected
ordinary-element recovery; and one authored trigger may cause multiple distinct
semantic relations without becoming fabricated source origin.

### TC-S7 — bounded InBody `</body>` transition

Accepted validation authority is #374 / PR #375. Historical #373 is not the
accepted validation authority. Production: #378 / PR #379.

Established selected body-end audit plus `InBody → AfterBody` while preserving
the bounded open stack rather than manufacturing closure.

### TC-S8 — bounded InBody `</html>` transition

Validation: #380 / PR #381. Production: #382 / PR #383.

Composed body-end audit semantics with driver-owned same-token redispatch:

```text
InBody
→ AfterBody
→ reprocess the same retained token
→ AfterAfterBody
```

without collapsing token identity or popping the retained bounded stack.

## Phase 5 — TC-S9 Style / RAWTEXT Feedback Lifecycle

Validation: #384 / PR #385. Evidence checkpoint: #386 / PR #387. Production:
#388 / PR #389. PR #389 merged as:

```text
734b6da35d2f222a464ec92106a58fe96fdac95a
```

TC-S9 proved the first real tree-directed tokenizer feedback lifecycle:

```text
Data emits authored <style>
→ tree inserts Style
→ tree requests semantic RAWTEXT entry
→ coordinator applies feedback before any later source production
→ tokenizer processes RAWTEXT
→ appropriate authored </style> returns tokenizer to Data
→ tree closes Style and restores InHead
```

This falsified the assumption that all relevant tokens can always be produced in
one context-free batch before tree construction.

Ownership remained explicit:

- tokenizer: lexical state, cursor, RAWTEXT recognition, appropriate-end-tag
  matching;
- tree: insertion mode, Style node/lifecycle, semantic feedback request;
- coordinator: causal ordering.

No second tokenizer, source rescan/reparse, tree-owned lexical state, public
resumable protocol, or generic tokenizer-mode switch API was required.

## Phase 6 — TC-S10 Title / RCDATA / Named Character Reference

TC-S10 intentionally selected a successor that added new architecture pressure
rather than a mechanically similar second RAWTEXT element.

### Candidate-independent semantic validation

#390 / PR #391 validated the selected InHead `<title>` lifecycle through RCDATA
and Named Character Reference semantics, including:

- tree-directed RCDATA entry;
- appropriate `</title>` recognition;
- Character Reference entry from RCDATA;
- Named Character Reference maximum-match behavior;
- non-committing lookahead requirements;
- exact authored contribution versus interpreted-output identity;
- unresolved/ambiguous ampersand behavior;
- one authored reference producing more than one Unicode scalar; and
- explicit Numeric Character Reference deferral.

### Complete deterministic Named Character Reference data foundation

#392 / PR #393 established the complete official WHATWG Named Character
Reference data foundation:

```text
2231 generated semantic entries
retained official entities.json bytes
separate normative snapshot identity and dataset-byte identity
reproducible generator
independent verifier
checked-in deterministic Rust data
```

The data gate deliberately did not authorize production consumption by itself.

### Compiler-sealed ownership prerequisites

The first TC-S10 production attempt exposed a repository-integrity proof defect
around the generated Named data. The project stopped and replaced the fragile
handwritten Rust-source scanner rather than treating it as authority.

Accepted successor chain:

```text
#396 / PR #397
compiler-sealed ownership mechanical validation

#398 / PR #399
production compiler-sealed canonical Named-data owner
```

The accepted theorem uses Rust privacy/coherence plus wrapper-only production
consumption. Repository source-text parsing is not the ownership oracle.

### Accepted TC-S10 production

Issue #394 / PR #400 is the accepted production implementation. An earlier PR
#400 head was rejected with five blocking findings; the remediated exact head was
independently reviewed before merge.

The accepted selected lifecycle proves:

- Title-specific semantic RCDATA feedback;
- coordinator application of RCDATA before later source production;
- appropriate Title end-tag handling and return to Data;
- selected Named Character Reference maximum matching;
- bounded non-committing observation that does not commit cursor progress,
  preprocessing diagnostics, coverage, or semantic evidence;
- canonical Named-data consumption through the compiler-sealed owner;
- one- and two-scalar interpreted output without fabricated source subdivision;
- semicolonless and unresolved-name diagnostic/contribution boundaries;
- source-authoritative handling of unmatched candidate delimiters;
- transition cost that does not scale one outer state step per matched identifier
  byte;
- resource-atomic ordering:

  ```text
  discover
  → preflight / prepare evidence
  → construct fallible semantic values
  → consume authoritative source
  → non-refusing commit
  ```

- separate Style/RAWTEXT and Title/RCDATA lexical and durable lifecycle
  ownership; and
- retention of standalone tokenizer deferred boundaries outside coordinated
  Title.

PR #401 then recorded the exact TC-S10 research and source provenance without
changing production semantics.

## Post-TC-S10 Non-Semantic Hardening

Issue #557 later hardened the compiler-identity regression harness after a
transient wrapper-process failure. It explicitly preserved the accepted #396 /
#397 ownership theorem and changed no HTML production semantics, Named data,
public API, dependency, toolchain, or workflow contract.

This is validation-harness maintenance, not TC-S11 or a new HTML semantic leaf.

## Current Accepted HTML Capability Boundary

At this checkpoint, the HTML track is **pauseable and accepted through TC-S10**.
It proves substantially more than the original explicit-start-tag slice, but it
does not claim complete HTML Standard parsing.

Accepted bounded capability includes:

```text
project-owned source-backed tokenizer/parser/Core path
Candidate C bounded tree construction
authored versus synthesized provenance
selected document shell / AfterBody handling
selected Div / Section / Paragraph semantics
bounded recovery and implied-P behavior
bounded body/html insertion-mode transitions
Style RAWTEXT tree↔tokenizer feedback
Title RCDATA tree↔tokenizer feedback
selected Named Character Reference semantics
complete canonical WHATWG Named Character Reference data foundation
```

The following stronger claims remain false or unproved:

```text
TC-S10 == complete HTML parser
selected RCDATA == general RCDATA
Title RCDATA == textarea support
Named references == Numeric references
authored/interpreted equivalence
constructed identity == source identity
project tree == browser DOM
a selected feedback seam == generic tokenizer control API
```

## Explicit Deferred / Open HTML Domains

No next HTML successor is preselected by this checkpoint.

Materially open or deliberately deferred domains include, as applicable to a
future focused theorem:

- Numeric Character References;
- RCDATA NUL recovery;
- general RCDATA and coordinated `<textarea>`;
- Data-state / AttributeValue character references;
- Script Data and script/reentrant parser behavior;
- general InHead/InBody coverage beyond the selected profile;
- fragments and fragment-context parsing;
- templates;
- tables and foster parenting;
- active formatting elements / adoption agency;
- foreign content / namespace integration;
- broader scope and implied-end algorithms;
- generic tokenizer/tree recovery coverage;
- runtime DOM correlation;
- public APIs and serialization;
- incremental/streaming APIs; and
- browser-protocol integration.

These open domains do not invalidate Candidate C or the accepted TC-S1–TC-S10
results. They require fresh focused evidence when selected.

## Resume Rule

When HTML work resumes:

1. start from this checkpoint plus #106 / #117;
2. revalidate live repository and current WHATWG authority;
3. do not rerun settled TC-S1–TC-S10 research unless relevant authority or
   repository semantics changed;
4. select a concrete next capability based on demonstrated new architecture or
   coverage pressure;
5. use candidate-independent validation where expected semantics are
   load-bearing; and
6. preserve exact source/interpreted/constructed/runtime distinctions.

Current program state:

```text
HTML architecture: ACCEPTED / Candidate C unchanged
TC-S1 through TC-S10 production: ACCEPTED / MERGED
TC-S10 provenance: RECORDED
HTML second bounded production wave: PAUSEABLE / CLOSED AT TC-S10
next HTML successor: NOT PRESELECTED
full HTML Standard support: NOT CLAIMED
```
