# CSS Research Evidence

Current-status review: 2026-10-08 JST (dated checkpoints remain historical)

Classification: task and evidence record; non-normative.

## Current Status

CSS is **STRONG BOUNDED BASELINE / PAUSED / RESUMABLE ON CONCRETE TRIGGER**. This is adopted under [#418](https://github.com/YT-TechDev/frontend-analysis/issues/418#issuecomment-6041412391) and indexed by [#107](https://github.com/YT-TechDev/frontend-analysis/issues/107#issuecomment-6041424955). [#914](https://github.com/YT-TechDev/frontend-analysis/issues/914) completed, [PR #915](https://github.com/YT-TechDev/frontend-analysis/pull/915) merged, and no new CSS feature or broad research frontier is selected.

Accepted bounded public answers:
- `fa css-selectors`: selected `CoreV1` selector-context qualification under [ADR 0011](../../decisions/0011-establish-cli-product-and-css-core-v1-consumer-boundary.md), including source/typed outcome/stage completeness evidence.
- `fa css-transforms`: selected direct-authored `transform` qualification of retained ordinary declarations with declaration/property/value/optional-priority and owning-context evidence (PR #915).

The #184/#185 Semantic Foundation and post-freeze selected authored values remain accepted. Current-normative `transform` Function coverage is **21 / 21**, while unknown/future Function containment stays open-world. **Zero selected transform observations do not establish whole-source absence.** Neither public answer claims complete CSS, matching, specificity, cascade, CSSOM, computed/used values, layout, rendering or browser behavior; upstream incompleteness cannot be upgraded.

The [2026-09 authored-value checkpoint](2026-09-authored-value-qualification-and-transform-coverage-checkpoint.md) is still valid at its dated baseline, not current next-step authority:

```text
main: a5bdbe40ebfdd575ea6ea61dbffe7e7e956eeb6b
tree: 57aeebe9e693f5e44755a0d874f9ba3055617633
```

The [2026-08 Semantic Foundation checkpoint](2026-08-semantic-foundation-status-checkpoint.md) likewise remains historical. Future work uses then-LIVE main, concrete #418 restart triggers, and the [whole-project PAUSE](https://github.com/YT-TechDev/frontend-analysis/issues/104#issuecomment-6044454270), not an automatic continuation of either earlier checkpoint.

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
- [#418 — durable post-freeze CSS value-qualification research and transform capability closure](https://github.com/YT-TechDev/frontend-analysis/issues/418)
- [#684 — final current-normative transform coverage leaf](https://github.com/YT-TechDev/frontend-analysis/issues/684)
- [PR #685 — `perspective()` transform coverage completion](https://github.com/YT-TechDev/frontend-analysis/pull/685)
- [#686 — 2026-09 CSS evidence checkpoint](https://github.com/YT-TechDev/frontend-analysis/issues/686)
- [#857 / PR #858 — selector Product consumer](https://github.com/YT-TechDev/frontend-analysis/pull/858)
- [ADR 0011 — CLI Product/Core consumer authority](../../decisions/0011-establish-cli-product-and-css-core-v1-consumer-boundary.md)
- [#914 / PR #915 — transform Product consumer](https://github.com/YT-TechDev/frontend-analysis/pull/915)
- [#418 — current CSS PAUSE adoption](https://github.com/YT-TechDev/frontend-analysis/issues/418#issuecomment-6041412391)

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
implemented. Later authored-value qualification extends this foundation through
focused capabilities; it does not rewrite these source/provenance contracts.

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

The later authored-value work reinforces the same discipline for its own outcome
space: direct selected-grammar invalidity, unsupported/opaque mechanisms, and
lower-layer incompleteness remain distinct rather than collapsing into a boolean.

### C8 — Semantic resources belong to the semantic capability

Selector qualification owns selector-specific algorithm/depth/observation
resources independently from tokenizer/parser resource counters.

Resource refusal must not fabricate a partial current observation or rewrite the
upstream structural lifecycle. Previously committed observations remain
preservable where the capability contract allows it.

Later declaration-value qualification preserves the same lower-layer authority:
parser/tokenizer `Incomplete` is not upgraded merely because a retained committed
prefix can be qualified.

### C9 — Relation provenance must survive future storage choices

Structural ancestry, declaration ownership, selector grammar context, authored
selector evidence, and later semantic relations may share storage in the future,
but their meanings cannot collapse into one untyped edge relation.

No universal CSS evidence graph is frozen by the current foundation.

## Authored-Value Qualification Evidence

The post-#185 work establishes a bounded authored-value layer over selected
properties and transform Functions. The current capability envelope is summarized
in the
[2026-09 checkpoint](2026-09-authored-value-qualification-and-transform-coverage-checkpoint.md).

Durable mechanics include direct keyword and scalar membership, retained exact
numeric/unit evidence, zero/signed-zero and finite-range qualification,
property-local unions, fixed/bounded cardinality, authored omission, repetition,
function-relative argument qualification, nested delimiter isolation,
parser-authoritative true EOF, deferred substitution precedence, and explicit
Invalid/Unsupported/open-world containment boundaries.

Current normative `transform` Function selected coverage is complete at 21 / 21.
That is a selected authored-source coverage statement only; it does not imply
computed transform matrices, interpolation, CSSOM, or rendering semantics.

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

That count is historical and is not the current repository-wide test count.
Post-freeze authored-value leaves added their own focused candidate-independent
and regression evidence; high-risk leaves used adversarial mutation sealing and
independent exact-head review where materially useful. The 2026-09 checkpoint
records the resulting maturity without turning those techniques into mandatory
ceremony for every future leaf.

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
- `one generic CSS AST/evidence graph is required now`;
- `current normative transform coverage complete == closed Function universe`;
  and
- `the Semantic Foundation Freeze or the 2026-09 checkpoint means CSS research
  or CSS semantics are complete`.

## OPEN Research / Architecture

The following remain outside the current supported envelope or require separate
focused follow-up:

- broader selector/profile coverage;
- semantic `@namespace` environment;
- selector specificity;
- DOM/tree selector matching;
- full `@scope` semantic/cascade behavior;
- broader property and value grammar semantics beyond the selected authored
  profiles;
- deferred value surfaces such as `font-family` and `perspective-origin` under
  their recorded authority constraints;
- CSSOM representation;
- cascade, layers, origins, importance, specificity, scope proximity and winner
  selection;
- inheritance and computed/used values;
- browser/runtime applicability and interoperability;
- layout and paint integration;
- broader malformed/recovery research as new capabilities expand;
- public CSS API/serialization; and
- any physical universal graph/AST/state representation.

The abandoned specificity chain #402-#410 remains abandoned and is not a resume
point for future CSS work.

## Evidence-to-Architecture Boundary

The frozen foundation and later authored-value evidence should not be silently
rewritten by future research. They do not authorize implementation of all OPEN
capabilities.

A material production expansion must explicitly state which established
contracts it consumes, which new semantic responsibility it owns, whether
relevant normative authority changed, and whether a focused architecture/ADR
update is required.

For a concrete CSS restart, begin from then-LIVE main and the adopted #418
resume triggers. Preserve the dated checkpoints as historical evidence rather
than treating them as current next-step authority.
