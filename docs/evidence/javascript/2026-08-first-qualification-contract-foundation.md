# First ECMAScript Qualification Contract Foundation

Status: implementation foundation under Issue #199.

## Scope

This foundation introduces only the crate-private semantic contract required by later first-envelope ECMAScript implementation leaves.

It does not implement lexical analysis, parsing, Early Error evaluation, RegExp Pattern qualification, or aggregate Standard Qualification.

## Fixed envelope

The contract owns one envelope by construction:

```text
ECMA-262 2026 / 17th edition
snapshot d89c03f2db8a597bc915b363a6518d0cc8acdbc0
Unicode 17.0.0
Core UTF-8 SourceText
Independent Source Unit
Script
mandatory selected-snapshot language
mandatory Legacy
Normative Optional source/static set = empty
```

There is no generic caller-configurable edition, parse-goal, or profile object in this leaf.

## Lifecycle and verdict

The contract keeps processing lifecycle separate from source-standard qualification meaning:

```text
Complete
ResourceLimited
InternalFailure
```

is distinct from:

```text
Qualified
SyntaxRejected
StaticSemanticsRejected
ProfileBoundaryRejected
```

`Complete` does not imply `Qualified`.

Resource or internal incompleteness carries no whole-source qualification verdict.

## Implementation-incomplete honesty

This leaf intentionally does not introduce a permanent `ImplementationIncomplete` qualification verdict.

Partial grammar/static-semantic implementations remain unable to construct an aggregate `Qualified` result. The complete-qualified constructor requires a private proof token for which no production constructor exists in this foundation.

A later aggregate owner must add the reviewed construction path only after satisfying the complete first-envelope prerequisite set.

## Source-backed evidence

Authored evidence is accepted only when the supplied `SourceAnchor` retains the exact authoritative `SourceText` identity and complete retained UTF-8 bytes under existing Core contracts.

Evidence distinguishes:

```text
Authored(SourceAnchor)
Derived
```

Derived semantic material is therefore not forced to fabricate an authored source range.

The rejection contract records only coarse ownership needed by this stage:

```text
Grammar
StaticSemantics
ProfileBoundary
```

No durable rule-ID schema, diagnostic prose, severity model, localization, serialization, or presentation contract is introduced.

## Determinism

Equivalent meaning is based on fixed semantic fields and existing exact-source reconciliation. Allocation identity is not semantic input.

No hash iteration, global mutable state, async runtime, concurrency mechanism, or unsafe Rust is introduced.

## Validation ownership

Tests reuse the candidate-independent #197 gold/model files as validation authority and compare the production contract against those expected semantics.

The production contract does not define its own expected correctness.

## Compatibility boundary

The implementation remains inside the existing crate-private `ecmascript` module.

`lib.rs` receives no ECMAScript public re-export, and no dependency, Cargo target, crate/workspace, or serialization surface is added.
