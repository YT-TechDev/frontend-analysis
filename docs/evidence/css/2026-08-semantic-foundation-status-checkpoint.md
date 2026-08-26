# CSS Semantic Foundation Status Checkpoint

Status date: 2026-08-26

Classification: task and evidence record; non-normative.

## Purpose and Status

This record preserves the CSS evidence state that can currently be supported by
durable repository authority.

The latest established CSS semantic production authority remains the **CSS
Semantic Foundation Freeze** completed by Issue #184 and Pull Request #185.
A fresh Pull Request inventory at this checkpoint found no later CSS production
semantic Pull Request. Repository work after #185 includes documentation and
other language tracks, but that must not be rewritten as additional CSS semantic
coverage.

This record therefore updates status without inventing post-freeze progress.

It does **not** amend or override architecture contracts, ADRs, provenance
ledgers, or future focused CSS decisions.

## Exact Freeze Authority

Semantic Foundation Freeze:

```text
Issue:
#184

Pull Request:
#185

reviewed feature head:
3f81b05789bcbdcce40ce610fd6027015054aa9a

squash-merge commit on main:
a2bad36cffa9bb67a547977930ad4bea308e94e9
```

The freeze is the latest CSS semantic authority identified by the 2026-08-26
live repository review.

Current repository `main` has advanced for HTML, ECMAScript, documentation, and
other work, but this does not itself advance CSS semantics.

## Evidence Boundary

The CSS evidence record follows the same distinction as the other language
tracks:

```text
source evidence
!= structural parser fact
!= semantic qualification
!= browser/runtime observation
!= architecture representation choice
```

In particular:

- a parser recognizing a declaration does not establish property/value grammar
  validity;
- a selector being structurally represented does not establish complete
  Selectors conformance;
- selector qualification does not establish DOM matching or cascade behavior;
- preserved browser/runtime data does not redefine project-owned source-parser
  authority; and
- frozen evidence does not mean the CSS track is complete.

## Frozen Foundation

The freeze consolidates a bounded source-to-semantic foundation spanning:

```text
retained SourceText
  ↓
project-owned CSS tokenizer contracts / lifecycle
  ↓
source-backed structural parser evidence
  ↓
declaration occurrence and context evidence
  ↓
Core source/context reconciliation
  ↓
structural selector representation
  ↓
bounded selector qualification (CoreV1)
```

The established foundation includes the following classes of evidence.

### Tokenizer and lifecycle evidence

The CSS track established project-owned tokenizer/result contracts with explicit:

- source-backed evidence;
- completion versus incompleteness;
- diagnostics and recovery;
- resource usage and limits;
- deterministic behavior; and
- candidate-independent validation.

The freeze does not mean every CSS Syntax tokenization state or recovery family
has production coverage.

### Structural declaration and context evidence

The accepted parser/Core path retains source-backed declaration observations and
structural context rather than treating a declaration as an isolated normalized
name/value pair.

Established context work includes bounded evidence for:

- nested qualified-rule contexts;
- nested group-rule contexts;
- descriptor contexts including `@font-face` and `@property`;
- `@page` and page-margin contexts;
- keyframes contexts; and
- Core-side context/source reconciliation.

These are structural/context foundations. They are not a complete CSS property
or descriptor semantic engine.

### Regression evidence retained by the freeze

Two source-sensitive regressions remain particularly important because they
falsify lossy reconstruction approaches.

Escaped property spelling:

```css
a{c\6F lor/**/:red;}
```

Established ranges:

```text
declaration: [2,19)
property:    [2,10)
value:       [15,18)
priority:    absent
semicolon:   [18,19)
```

Priority/comment/escape case:

```css
a{color:red !/**/Im\70 ortant;}
```

Established ranges:

```text
complete declaration: [2,30)
value:                [8,11)
priority:             [12,29)
semicolon:            [29,30)
```

These cases support the requirement that authored source evidence, comments,
escapes, priority evidence, and normalized semantic interpretation must not be
collapsed into a reconstructed string model.

## CoreV1 Selector Qualification Evidence

The freeze includes a bounded selector-qualification profile rather than a claim
of complete Selectors support.

Within the accepted `CoreV1` profile, evidence covers bounded structural and
qualification handling for selected combinations of:

- type and universal selectors;
- class and ID selectors;
- attribute selectors;
- combinators;
- nesting; and
- selected pseudo-class / functional-pseudo constructs.

Important qualification distinctions retained by the evidence include:

- `:is()` / `:where()` use forgiving semantics where the supported profile
  actually covers them;
- `:not()` remains unforgiving;
- `:has()` is relative-selector-sensitive and nested `:has()` is rejected in the
  bounded profile; and
- named namespace prefixes cannot be treated as resolved without a namespace
  environment and therefore remain indeterminate where that environment is not
  supplied.

The profile does **not** establish selector specificity, DOM element matching,
style rule application, cascade, inheritance, computed values, or runtime style
resolution.

## Freeze Validation Evidence

At the #184/#185 freeze point, the repository recorded:

```text
workspace tests: 742 passed
```

That number is a **historical freeze-baseline validation count**, not the current
repository-wide test count.

The freeze evidence also includes:

- candidate-independent selector conformance GOLD;
- resource/adversarial matrices;
- deterministic repeated execution;
- source-identity and same-SourceId/different-content corruption rejection;
- exact source/context reconciliation; and
- zero expansion of dependencies, workspace members, public API, browser/runtime
  ownership, I/O, serialization, async, concurrency, or `unsafe` boundaries.

Passing that validation freezes the supported foundation. It does not silently
qualify every CSS grammar or runtime behavior.

## Production Lineage Included in the Freeze

The freeze incorporates the earlier focused CSS progression, including:

- #143 lexical contracts;
- #145 tokenizer lifecycle;
- #146 candidate-independent tokenizer foundation;
- #154 bounded lossless CSS tokenizer;
- #156 declaration/parser-stage contract foundation;
- #161 discard evidence;
- #162 EOF recovery;
- #163 bounded source-backed declaration parser;
- #164 Core declaration integration;
- #173 context contracts;
- #174 nested qualified-rule contexts;
- #175 nested group-rule contexts;
- #176 descriptor contexts;
- #177 `@page` / page-margin contexts;
- #178 keyframes contexts;
- #179 Core context revalidation;
- #183 selector contracts / candidate-independent GOLD; and
- #184 / PR #185 Semantic Foundation Freeze and bounded CoreV1 selector
  qualification.

This list is an evidence lineage, not a claim that every Issue number corresponds
to a separate stable public API or universal CSS subsystem.

## Fresh 2026-08-26 Repository Check

A live search of closed and open Pull Requests was performed for the CSS track.
No CSS production semantic Pull Request later than #185 was identified.

PR #300 added provenance documentation only. Under the repository's explicit
boundary:

```text
provenance != evidence conclusion != architecture decision
```

that documentation change is not CSS semantic production progress.

Therefore the honest current classification is:

```text
CSS Semantic Foundation Freeze:      ESTABLISHED / CURRENT
post-#185 CSS semantic production:   NONE IDENTIFIED
CSS track complete:                  NO
broader CSS semantics:               OPEN
```

A future CSS change that materially extends semantics should create a new dated
evidence checkpoint rather than retroactively widening this freeze.

## Strong Claims Not Established

The current CSS evidence does **not** establish:

- complete CSS Syntax coverage;
- complete Selectors coverage;
- a complete namespace environment/resolution model;
- selector specificity;
- DOM selector matching;
- full pseudo-class or pseudo-element semantics;
- complete property grammar or value grammar;
- shorthand expansion;
- CSS-wide value computation;
- custom-property substitution semantics as a complete system;
- `@scope` cascade semantics;
- cascade origin/layer/order resolution;
- inheritance;
- computed/used/actual values;
- CSSOM compatibility;
- browser style-engine equivalence;
- runtime style mutation semantics;
- layout, paint, compositing, or rendering diagnostics;
- complete malformed-input recovery;
- universal graph/AST/event representation;
- public CSS analysis API or serialization format; or
- completion of the CSS research program as a whole.

## Current Evidence Status

```text
lossless source/token foundation:       ESTABLISHED within bounded capability
structural declaration/context layer:  ESTABLISHED within bounded capability
Core source/context reconciliation:    ESTABLISHED within bounded capability
CoreV1 selector qualification:         FROZEN / BOUNDED
Semantic Foundation Freeze #184/#185: ESTABLISHED / CURRENT
post-freeze semantic production:       NONE IDENTIFIED as of 2026-08-26
full CSS semantics:                    NOT CLAIMED
```

## Update Rule

Do not advance this status because unrelated `main` commits land. Add a new CSS
evidence checkpoint only when durable CSS research, validation, architecture, or
production authority materially changes the supported semantic envelope.
