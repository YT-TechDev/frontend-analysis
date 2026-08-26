# CSS Research Evidence

Status date: 2026-08-26

Classification: task and evidence record; non-normative.

## Current Status

The CSS workstream has completed a **Semantic Foundation Freeze**. Issue
[#184](https://github.com/YT-TechDev/frontend-analysis/issues/184) was completed
through Pull Request
[#185](https://github.com/YT-TechDev/frontend-analysis/pull/185). The reviewed
feature head was:

```text
3f81b05789bcbdcce40ce610fd6027015054aa9a
```

and the squash merge on `main` was:

```text
a2bad36cffa9bb67a547977930ad4bea308e94e9
```

The freeze establishes a stable semantic foundation for continued CSS research.
It does **not** claim complete CSS parsing, selector semantics, CSSOM, cascade,
computed style, layout, paint, or browser-runtime applicability.

Future production expansion requires a focused unfreeze/follow-up decision;
research may continue without silently mutating the frozen contracts.

A fresh 2026-08-26 repository review found no CSS production semantic Pull
Request later than #185. The current status is recorded in
[2026-08 Semantic Foundation Status Checkpoint](2026-08-semantic-foundation-status-checkpoint.md).
This is deliberately a status confirmation, not a claim of post-freeze semantic
progress.

## Authoritative Evidence Sources

Repository evidence:

- [#104 — project-owned lossless parser architecture](https://github.com/YT-TechDev/frontend-analysis/issues/104)
- [#107 — CSS parser workstream and durable invariants](https://github.com/YT-TechDev/frontend-analysis/issues/107)
- [#137 — transactional CSS syntax-parser architecture](https://github.com/YT-TechDev/frontend-analysis/issues/137)
- [#138 — source-backed declaration-analysis contracts and regressions](https://github.com/YT-TechDev/frontend-analysis/issues/138)
- [#140 — first CSS parser-to-Core integration](https://github.com/YT-TechDev/frontend-analysis/issues/140)
- [#181 — source-backed selector qualification contracts](https://github.com/YT-TechDev/frontend-analysis/issues/181)
- [#184 — CSS Semantic Foundation Freeze](https://github.com/YT-TechDev/frontend-analysis/issues/184)
- [PR #185 — freeze integration](https://github.com/YT-TechDev/frontend-analysis/pull/185)

Normative CSS behavior remains governed by the applicable CSSWG specifications.
The exact specification snapshots/profile used by a capability must be recorded
with that capability. Browser engines and external parsers remain differential
or interoperability evidence rather than Core semantic authority.

## Frozen Foundation

The #184 freeze covers the established foundation for:

- tokenizer evidence, lifecycle, and resource contracts;
- structural parser evidence and source-backed context contracts;
- declaration occurrence and parent-context ownership;
- Core source/context reconciliation;
- structural context families and ancestry;
- source-backed selector qualification domain/profile/outcome contracts;
- bounded production `CoreV1` selector qualification;
- selector-specific lifecycle and resource semantics; and
- candidate-independent selector conformance gold.

The foundation remains capability-bounded and crate-private where currently
implemented.

## Proven Architecture Evidence

### C1 — Raw source and semantic preprocessing must coexist without conflation

CSS preprocessing may change the semantic code-point stream while exact evidence
still refers to the retained raw UTF-8 source. A normalized stylesheet copy does
not replace source authority.

The durable model therefore requires an honest relationship between semantic
input and retained bytes rather than pretending they are identical.

### C2 — Raw spelling and interpreted meaning are separate evidence

Decoded identifiers, semantic token values, normalized values, or future CSSOM
serialization cannot substitute for authored spelling or authored endpoints.

This is load-bearing for escaped identifiers, comments, `!important`,
preprocessing-sensitive input, and source navigation.

### C3 — Exact endpoints come from owned lexical/parser evidence

The CSS workstream rejects the following as provenance mechanisms:

- source search;
- delimiter scanning;
- endpoint reconstruction;
- decoded-length inference;
- reparsing selected fragments; and
- a second tokenizer used to manufacture locations.

Core may revalidate projected evidence against `SourceText`; it must not
rediscover the syntax that produced the evidence.

### C4 — Transactional parsing must roll back semantic evidence, not only cursor position

CSS parsing can require speculative interpretation. The accepted parser model
therefore treats rollback as a transaction over all temporary state that could
leak a false interpretation, including as applicable:

- cursor position;
- temporary observations;
- branch-local diagnostics;
- recovery/discard evidence;
- context state; and
- transactional resource/context bookkeeping.

Failed speculative branches must not leave durable evidence that they succeeded.

### C5 — Structural context is not semantic qualification

The selector research established the following durable distinction:

```text
QualifiedRuleBlock
!= selector grammar qualified
!= standards-profile selector valid
!= selector matched against a DOM/tree
!= specificity result
!= property/value validity
!= CSSOM membership
!= cascade/runtime applicability
```

A structural parser may retain a qualified-rule block and exact prelude without
thereby asserting selector validity.

### C6 — Context is first-class evidence

Declaration-shaped syntax cannot be interpreted correctly solely from its local
`name: value` shape. Style-rule declarations, descriptors, keyframes, nested
rules, malformed/recovered constructs, and unsupported contexts remain distinct.

Likewise selector qualification derives its grammar mode from retained structural
ancestry rather than rescanning source text.

Current bounded modes include concepts equivalent to:

- normal selector list;
- nested relative selector list; and
- scoped relative selector list.

These semantic concepts do not require a permanent public enum or AST.

### C7 — Unsupported, invalid, and indeterminate are different semantic outcomes

The selector foundation distinguishes outcomes equivalent to:

- qualified by the selected grammar;
- invalid for the selected grammar;
- unsupported by the selected grammar profile; and
- indeterminate because required semantic context is unavailable.

For example, a named namespace prefix cannot be declared invalid merely because
the current selector capability lacks a semantic namespace environment.

This supports a broader project rule: inability to establish a semantic claim is
not equivalent to negative semantic evidence.

### C8 — Semantic resources belong to the semantic capability

Selector qualification owns selector-specific algorithm/depth/observation
resources independently from tokenizer/parser resource counters.

Resource refusal must not fabricate a partial current observation or rewrite the
upstream structural lifecycle. Previously committed observations remain
preservable where the capability contract allows it.

### C9 — Relation provenance must survive future storage choices

Structural ancestry, declaration ownership, selector grammar context, authored
selector evidence, and later semantic relations may share storage in the future,
but their meanings cannot collapse into one untyped edge relation.

No universal CSS evidence graph is frozen by the current foundation.

## Historical Regression Evidence Preserved

The project-owned CSS path retains earlier parser-qualification failures as
regression knowledge rather than reopening external-parser adoption.

### Escaped raw property-name provenance

```css
a{c\6F lor/**/:red;}
```

Historical expected raw UTF-8 evidence:

```text
declaration:   [2,19)
property name: [2,10)
value:         [15,18)
priority:      absent
semicolon:     [18,19)
```

The stock `cssparser` qualification demonstrated that a useful parser can still
fail the exact project provenance boundary when a mandatory authored endpoint is
not publicly provable without reconstruction.

### Silent priority-loss regression

```css
a{color:red !/**/Im\70 ortant;}
```

Historical expected evidence:

```text
complete:  [2,30)
value:     [8,11)
priority:  [12,29)
semicolon: [29,30)
```

CSSKit Gate 1 demonstrated that a parser result can retain an apparently valid
prefix while silently losing authored priority-shaped source. The project-owned
pipeline therefore preserves material recovery/discard/lifecycle evidence and
must not represent such loss as clean success.

## Selector Foundation Evidence

The bounded `CoreV1` selector capability validates a useful modern subset without
claiming complete Selectors Level 4 support. Evidence includes bounded support
for core selector structure such as type/universal/class/ID/attribute selectors,
combinators, nesting, and selected pseudo constructs.

Important semantic policies include:

- `:is()` / `:where()` use forgiving-list semantics where supported;
- `:not()` remains unforgiving under the selected profile;
- `:has()` uses relative-selector semantics and rejects unsupported nested
  `:has()` behavior under the bounded profile;
- unsupported or indeterminate features are not silently dropped;
- named namespace prefixes remain indeterminate when the required semantic
  namespace environment is unavailable; and
- selector qualification neither computes specificity nor performs DOM
  matching/cascade/runtime applicability.

The capability consumes existing retained structural/token evidence; it does not
introduce a second tokenizer/parser or require a retained universal selector AST.

## Validation Evidence

The #184 freeze audit recorded a complete repository validation pass for the
frozen CSS foundation, including:

- Rust formatting and Clippy checks;
- locked Cargo metadata/workspace validation;
- **742 tests passed** at the freeze baseline;
- candidate-independent selector gold;
- selector lifecycle/resource matrices;
- repeated-run determinism;
- source/run identity validation, including same-`SourceId`/different-content
  corruption rejection; and
- no production dependency, workspace, public-export, browser, runtime, I/O,
  serialization, async/concurrency, or repository-authored `unsafe` expansion.

Validation counts describe the freeze baseline and are not permanent compatibility
promises.

## Rejected or Unsupported Strong Claims

Current CSS evidence rejects or does not justify:

- `CSS preprocessing output == retained source evidence`;
- `decoded property name/value == authored spelling`;
- `declaration-shaped syntax is one universal declaration domain`;
- `QualifiedRuleBlock == valid selector`;
- `selector grammar valid == DOM matchable`;
- `selector qualification == browser support`;
- `selector qualification == specificity/cascade applicability`;
- `missing namespace environment == invalid selector`;
- `parser rollback only needs to rewind a token cursor`;
- `external parser/browser agreement defines project correctness`;
- `one generic CSS AST/evidence graph is required now`; and
- `the Semantic Foundation Freeze means CSS research or CSS semantics are complete`.

## OPEN Research / Architecture

The following remain outside the frozen foundation or require separate follow-up:

- broader selector/profile coverage;
- semantic `@namespace` environment;
- selector specificity;
- DOM/tree selector matching;
- full `@scope` semantic/cascade behavior;
- property and value grammar semantics;
- CSSOM representation;
- cascade, layers, origins, importance, specificity, scope proximity and winner
  selection;
- inheritance and computed/used values;
- browser/runtime applicability and interoperability;
- layout and paint integration;
- broader malformed/recovery and true-EOF research as new capabilities expand;
- public CSS API/serialization; and
- any physical universal graph/AST/state representation.

## Evidence-to-Architecture Boundary

The Semantic Foundation Freeze means the established foundation should not be
silently rewritten by later research. It does not authorize implementation of
all OPEN capabilities. A material production expansion must explicitly state
which frozen contracts it consumes, which new semantic responsibility it owns,
and whether a focused architecture/ADR update is required.
