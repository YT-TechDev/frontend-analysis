# CSS Authored-Value Qualification and Transform Coverage Checkpoint

Status date: 2026-09-14

Classification: task and evidence record; non-normative.

## Purpose and Status

This checkpoint preserves the accepted CSS research and production state after a
large post-freeze expansion of authored declaration-value qualification. It is
the durable resume point for the CSS workstream before active language work
returns to ECMAScript.

This record does **not** declare CSS complete. It records that the CSS track has
reached a mature, pauseable point where the next highest-value architecture
pressure is no longer another mechanically similar property leaf.

The checkpoint is owned by [Issue #686](https://github.com/YT-TechDev/frontend-analysis/issues/686).
It is evidence, not an architecture contract, and does not replace the focused
Issues, Pull Requests, accepted reviews, or normative CSS specifications that
authorized individual capabilities.

## Exact Accepted Baseline

Accepted `main` when this checkpoint was created:

```text
commit: a5bdbe40ebfdd575ea6ea61dbffe7e7e956eeb6b
tree:   57aeebe9e693f5e44755a0d874f9ba3055617633
subject: feat(css): complete selected transform coverage with authored perspective() argument (#685)
```

[PR #685](https://github.com/YT-TechDev/frontend-analysis/pull/685) is merged and
[Issue #684](https://github.com/YT-TechDev/frontend-analysis/issues/684) is
completed. The reviewed final candidate tree and accepted `main` tree are
identical.

## Relationship to the 2026-08 Foundation Freeze

The earlier
[2026-08 Semantic Foundation Status Checkpoint](2026-08-semantic-foundation-status-checkpoint.md)
remains valid historical evidence for the source/tokenizer/parser/context and
bounded selector foundation frozen by #184/#185.

Its *current-status* statement that no later CSS semantic production had been
identified was true on 2026-08-26 but is no longer the current repository state.
That historical record must not be rewritten to predict later work.

The post-freeze CSS track added a substantial bounded authored-value
qualification layer through focused Issues and Pull Requests while preserving
the frozen source/provenance contracts. Detailed leaf-by-leaf durable research
is retained primarily in
[#418](https://github.com/YT-TechDev/frontend-analysis/issues/418) and the
individual implementation Issues/PRs; this checkpoint summarizes the resulting
capability envelope rather than duplicating every leaf history.

## Current CSS Maturity

At this baseline the CSS track has two established layers of evidence:

```text
frozen source / tokenizer / structural parser / context / selector foundation
                                +
        bounded authored declaration-value qualification capabilities
```

The authored-value layer demonstrates reusable mechanics across selected
property grammars without claiming a universal CSS grammar engine.

Established capability classes include:

- property-specific direct keyword qualification with placement-sensitive
  treatment of CSS-wide and other whole-value mechanisms;
- direct integer, number, percentage, length, and length-percentage membership;
- exact numeric sign, decimal, fraction, and exponent evidence without
  machine-number conversion;
- exact zero and signed-zero handling where grammar requires it;
- finite direct ranges such as non-negative length without unit conversion;
- property-local unions with authored semantic identity preserved;
- fixed and bounded cardinality;
- authored omission distinct from explicitly authored default-equivalent
  syntax;
- repeated grammar items with lexical/source order preserved;
- grammar-native Function argument qualification at a fresh function-relative
  depth origin;
- function-body relative-depth-zero comma partitioning with nested
  Function/block delimiter isolation;
- exact tokenizer-owned evidence for selected arguments;
- parser-authoritative true-EOF behavior;
- lower-layer resource and `Incomplete` lifecycle preservation;
- deferred/arbitrary substitution authority above local apparent shape;
- direct Invalid, selected-inner opaque Function uncertainty, and outer
  unselected Function precedence; and
- generic open-world containment for unknown/future Function names.

These are demonstrated capabilities, not a commitment to factor them into one
generic parser, AST, grammar DSL, property registry, or public API.

## Evidence and Ownership Boundary

The post-freeze work preserved the existing evidence model:

```text
authored source evidence
!= interpreted semantic value
!= constructed semantic identity
!= runtime / computed identity
```

Responsibility remains split by owned evidence rather than convenience:

- the tokenizer owns retained lexical identity and exact token evidence;
- the parser owns structural extents, balanced structure, completion, recovery,
  true EOF, and parser resource lifecycle;
- value qualification owns the selected property/function grammar profile and
  its semantic classification; and
- browser/runtime behavior remains outside authored-source qualification unless
  a separate approved evidence path explicitly imports it.

The qualifier must not manufacture source truth by raw-source search, source
rescanning, retokenization, endpoint reconstruction, decoded-length inference,
or equivalent reconstruction.

## Outcome and Failure Semantics

The value-qualification work reinforced that semantic outcomes must remain
meaningfully distinct.

A representative bounded profile distinguishes:

```text
Qualified
InvalidForSelectedValueGrammar
Unsupported / outside selected semantic capability
```

while lower-layer parser/tokenizer lifecycle outcomes such as resource-driven
`Incomplete` remain authoritative and are not upgraded by the qualification
layer.

Important reusable boundaries include:

- directly visible selected-grammar invalidity remains Invalid rather than
  being softened because another opaque construct is present;
- a complete ordinary Function in an otherwise feasible typed slot may remain
  Unsupported when its semantic result would require unowned CSS math/type
  evaluation;
- Function-plus-additional-material can be directly Invalid when the authored
  slot shape itself proves the mismatch;
- deferred substitution that can alter surrounding shape remains higher
  authority than local apparent cardinality; and
- inability to qualify an unknown/future Function under the selected profile is
  not evidence that the authored Function is standards-invalid.

## Direct Numeric and Unit Evidence

The accepted authored numeric profiles rely on retained decimal structure rather
than machine conversion.

This supports exact distinctions such as:

```text
0
+0
-0
.0
0e100
-0px
.5px
```

where the grammar needs those distinctions or needs to prove that signed zero is
still zero. Range checks such as direct `<length [0,∞]>` are performed from
retained sign/significand evidence and recognized unit identity; they do not
require floating-point conversion, unit conversion, computed values, or
render-time normalization.

The accepted standalone `perspective: none | <length [0,∞]>` work in
[#440](https://github.com/YT-TechDev/frontend-analysis/issues/440) /
[PR #441](https://github.com/YT-TechDev/frontend-analysis/pull/441) is one durable
carrier of this direct-length theorem.

## Transform Capability Closure

The durable transform capability conclusion is recorded in
[#418 comment 5646151746](https://github.com/YT-TechDev/frontend-analysis/issues/418#issuecomment-5646151746):

```text
FRONTIER CLOSED
TRANSITION TO COVERAGE COMPLETION
```

The capability-frontier proof used a small representative set rather than every
Function name. In particular, the accepted `matrix()` / `scale()` /
`translate3d()` / `rotate3d()` set demonstrated the required architecture for:

- exact and bounded comma arity;
- homogeneous union slots and heterogeneous positional slots;
- authored omission;
- exact-zero alternatives;
- retained evidence and source order;
- function-local relative-depth comma partitioning;
- nested comma isolation;
- parser-authoritative true EOF;
- direct Invalid versus selected-inner opaque precedence;
- outer unselected Function precedence in the absence of decisive direct
  Invalid;
- deferred substitution authority; and
- tokenizer/parser/qualifier ownership separation.

After that closure, remaining current transform Functions were treated as
coverage work unless implementation evidence falsified the theorem. No such
falsification occurred.

## Current Normative Transform Coverage

[Issue #684](https://github.com/YT-TechDev/frontend-analysis/issues/684) and
[PR #685](https://github.com/YT-TechDev/frontend-analysis/pull/685) completed the
final current-normative selected Function leaf with authored
`perspective([<length [0,∞]> | none])`.

Accepted current state:

```text
current normative selected transform Function kinds:   21 / 21
current normative unselected transform Function kinds: 0
generic unknown/future Function containment:           preserved
```

The selected current normative Function set is:

1. `matrix()`
2. `scale()`
3. `translate3d()`
4. `rotate3d()`
5. `translate()`
6. `translateX()`
7. `translateY()`
8. `translateZ()`
9. `scaleX()`
10. `scaleY()`
11. `scaleZ()`
12. `scale3d()`
13. `rotate()`
14. `rotateX()`
15. `rotateY()`
16. `rotateZ()`
17. `skew()`
18. `skewX()`
19. `skewY()`
20. `matrix3d()`
21. `perspective()`

Coverage completion is deliberately narrower than several stronger claims:

```text
current normative selected coverage complete
!= closed universe of possible Function names
!= complete CSS math semantics
!= computed transform semantics
!= matrix execution / interpolation
!= CSSOM / rendering equivalence
```

Unrecognized or future Function names still use the existing bounded-profile
`UnselectedTransformFunction` containment path. Completing current normative
coverage did not convert open-world uncertainty into authored invalidity.

The post-#685 novelty check did not identify a new reusable architecture theorem
that required another #418 update. The final leaf applied the already-recorded
closure rather than expanding it.

## Important Falsified or Rejected Assumptions

The current CSS evidence preserves several negative results because they protect
future work from repeating failed designs.

### Parser success is not value-grammar validity

A structurally retained declaration does not itself prove that its value matches
a selected property grammar. Structural evidence and semantic qualification
remain separate stages.

### Every Function is not automatically Invalid or Unsupported

Function handling is placement- and capability-sensitive. Deferred substitution,
whole-value mechanisms, typed Function results outside the current slice,
Function-plus-junk, and unknown outer Function names do not share one universal
classification rule.

### Machine numeric conversion is not required for direct authored qualification

Retained decimal/sign/exponent evidence is sufficient for the accepted direct
integer, zero, signed-zero, and finite-range theorems. Converting to `f32`,
`f64`, or machine integers would lose evidence without solving an owned problem.

### Default-equivalent meaning does not erase authored omission

When a grammar permits an omitted component, omission remains distinct authored
evidence even if downstream semantics supply a value equivalent to an explicit
component.

### Transform coverage completion does not close the Function universe

The final transition from `perspective()` as the last normative unselected
sentinel to generic `unknownfunction()` containment was a theorem migration, not
permission to classify unknown/future Functions as Invalid.

### Repetition of similar leaves does not justify a generic CSS grammar engine

The accepted work repeatedly reused narrow scalar/evidence mechanics while
keeping property- and Function-specific semantic identity local. No generic CSS
Function AST, grammar DSL, universal property registry, or cross-domain value
framework is required by the current evidence.

## Validation Maturity

The post-freeze value work uses candidate-independent expected outcomes derived
from normative/project authority rather than from the production implementation
being tested.

High-risk leaves also used focused adversarial mutation sealing where it
controlled a real semantic risk. The final `perspective()` transform leaf, for
example, independently sealed direct range/zero behavior, opaque Function versus
Function-plus-junk, authored `perspective(none)` identity, current-normative
coverage completion, generic unknown-Function containment, and lower-layer
`Incomplete` preservation before merge.

This checkpoint does not turn mutation testing or independent review into a
mandatory ceremony for every future CSS change. Validation remains proportional
to the novelty and risk of the candidate.

## Intentional Deferrals and Non-Claims

The CSS workstream is pauseable, not complete. The following remain intentionally
outside this checkpoint's completion claim.

### Deferred researched surfaces

- `font-family` remains deferred after focused research found that a
  source-honest representation for quoted versus constructed multi-Ident family
  identity was feasible, but the normative/interoperability boundary was not
  stable enough to justify production selection. See
  [#418 comment 5600660403](https://github.com/YT-TechDev/frontend-analysis/issues/418#issuecomment-5600660403).
- `perspective-origin` remains deferred because a Values-4-only `<position>`
  theorem would not be durable against the evolving logical-position surface.
  See
  [#418 comment 5613485808](https://github.com/YT-TechDev/frontend-analysis/issues/418#issuecomment-5613485808).
- Other CSS properties and value grammars remain available for later focused
  selection when they add useful new pressure or concrete product need. This
  checkpoint does not create a mandate to exhaust a property list.

### Explicit non-claims

No claim is made here of:

- complete CSS Syntax or Selectors coverage;
- complete property/value grammar coverage;
- complete CSS math evaluation or numeric type algebra;
- cascade, origins, layers, specificity, scope proximity, inheritance, or
  winner selection;
- computed, used, or actual value processing;
- shorthand expansion as a general framework;
- CSSOM representation or browser-engine equivalence;
- DOM selector matching;
- layout, paint, compositing, or rendering semantics;
- a public CSS analysis API or serialized format; or
- a universal CSS AST, evidence graph, grammar DSL, or state model.

The abandoned specificity work #402-#410 remains abandoned. This checkpoint must
not be used to reconstruct or restart that workstream.

## Pause and Resume Rule

CSS is now safe to pause at this baseline.

A future CSS restart should begin by:

1. refreezing the then-current repository object;
2. reading this checkpoint plus the focused authority for the candidate area;
3. checking whether relevant CSSWG authority materially changed;
4. reusing settled scalar/evidence theorems when their authority and repository
   state remain valid; and
5. selecting a focused leaf only when it adds demonstrated value or pressure.

Do not replay the entire historical CSS research sequence merely because time
has passed.

## Workstream Transition

After this checkpoint is merged, the active language research/implementation
workstream moves to ECMAScript.

That transition does not make this CSS evidence record authority for
ECMAScript. ECMAScript resumes from its own accepted architecture, evidence,
repository state, and current normative authority.

The CSS workstream remains available for focused future work when a concrete
need, new capability pressure, or changed authority justifies reopening it.
