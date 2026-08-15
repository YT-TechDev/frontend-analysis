# First ECMAScript Standard Qualification Validation Foundation

Status: implementation foundation in progress under Issue #197.

## Authority hierarchy

```text
ECMA-262 2026 snapshot
  d89c03f2db8a597bc915b363a6518d0cc8acdbc0
+ Unicode 17.0.0
        ↓
normative grammar / profile / static-validity obligations
        ↓
project-owned candidate-independent validation gold
        ↓
pinned Test262 challenge evidence
  3655e7464de3d52643ecddd4b5f9f4f3e7f62398
        ↓
future production implementation
```

Production behavior, an external parser/browser, and Test262 pass/fail output do
not define expected Frontend Analysis semantics.

## Inventory granularity

Gate 1 audited 37 Early Error clause containers. A later rule-level applicability
audit refined that set to 34 containers with at least one active rule under the
frozen first profile. A container is not a normative rule count and is not a test
count.

The audit-only containers `EE-16` (`if`), `EE-17` (`do` / `while`), and `EE-18`
(`while`) contain labelled-function Early Error rules that are inactive when the
selected Normative Optional web-source feature set is empty. They remain in the
inventory as explicit applicability evidence rather than disappearing.

The completed rule-level expansion currently records 193 snapshot-local rule
units: 183 active first-envelope obligations and 10 rules audited as inactive
under the frozen envelope. Inactivity may come from the selected Parse Goal as
well as the Normative Optional policy, so validation uses the broader term
`EnvelopeInactiveRule`.

For example, the Module-only identifier restrictions in `EE-04` remain visible
but inactive under the Script-only envelope, while the labelled-function rules
in `EE-16` through `EE-20` and `EE-23` are inactive where the selected
web-compatibility extension is not implemented.

Rule expansion completeness and executable gold completeness remain separate.
Every audited production/rule condition from the pinned snapshot now has an
explicit active or envelope-inactive validation unit, but most active rules do
not yet have materialized executable gold fixtures. Aggregate conformance
therefore remains fail-closed.

The detailed inventory remains the authority for rule identity, normative
locator, Q7 dependency, gold obligation, and fixture mapping. A separate literal
37-row `EXPECTED_RULE_COUNTS` table independently freezes each container's
active and envelope-inactive counts. Validation compares every row and rejects
missing, unexpected, substituted, or kind-count-mismatched rule identities even
when an accidental global total would still equal 193.

```text
container audited
!= container has an active first-profile rule
!= rule-level coverage
!= aggregate qualification completeness
```

The executable foundation materializes gold for a small subset of those active
rule units and records explicit fixture references from the rule inventory.
Uncovered active rules remain visible with empty materialized-fixture sets, so
they cannot disappear from aggregate coverage accounting.

## Independent completeness dimensions

The validation foundation keeps the following dimensions conceptually separate:

- grammar coverage;
- Early Error rule coverage;
- validity-dependency coverage;
- RegExp Pattern coverage;
- profile/context coverage;
- Unicode-data coverage;
- source/provenance coverage;
- lifecycle/verdict coverage;
- determinism coverage; and
- Test262 challenge coverage.

Passing any one dimension does not imply complete Standard Qualification.

## Incremental implementation honesty

A source may be valid under the frozen ES2026 envelope even while production
coverage is still incomplete.

```text
implementation coverage pending
!= SyntaxRejected
!= StaticSemanticsRejected
!= ProfileRejected
```

The test-owned gold model can record the normative expected qualification and a
separate implementation-coverage state. This is validation metadata, not a
requirement for a permanent production result variant.

Request applicability is also independent from a source qualification verdict.
Module, Direct Eval, and Function Constructor requests are outside this first
request envelope. Their gold fixtures retain `UnsupportedGoal` or
`UnsupportedSourceContext` applicability with no qualification verdict.

## Source authority

Gold source text and expected ranges are authored independently of the future
ECMAScript parser. Existing Core `SourceText` / `SourceAnchor` semantics validate
UTF-8 byte boundaries where appropriate.

Synthetic or derived material, including Automatic Semicolon Insertion evidence,
must not receive fabricated authored ranges.

## Test262 effective-source evidence

A Test262 path alone is not evidence identity. The selection foundation preserves
frontmatter-sensitive effective source behavior.

Selected Test262 source bodies are not stored in the repository. An external
materializer must read the exact file from the pinned Git revision, establish
its Git blob identity, and verify the relevant normalized frontmatter before
the source bytes may enter selection or transformation.

For the first Script gate:

```text
ordinary   -> non-strict + exact Test262 strict-prefix variant
onlyStrict -> strict variant only
noStrict   -> non-strict variant only
raw        -> exact unmodified Script source when independently applicable
module     -> excluded
```

`negative.phase: resolution` and `runtime`, non-deterministic evidence, host-only
evidence, and proposal/post-ES2026 evidence are excluded from this static source
gate. A parse-negative test is selectable only after independent mapping to the
frozen normative inventory.

Line endings in the source body are preserved exactly by the effective-source
transformation.

## Current completion boundary

The pinned rule-level inventory expansion is complete for the frozen first
envelope:

```text
37 audited clause containers
34 containers with active first-envelope rules
193 snapshot-local audited rule units
  183 active
   10 envelope-inactive
```

The executable gold set is intentionally **not** complete. Only a small initial
subset of active rule units has materialized fixture references. The validation
foundation therefore keeps two separate gates:

```text
rule inventory expansion complete
!= executable gold coverage complete
```

Aggregate Standard Qualification conformance remains false until every active
rule, grammar/profile/context obligation, source/provenance requirement, Unicode
obligation, lifecycle boundary, determinism check, and selected Test262 challenge
has the required executable evidence.

## Pinned Test262 seed evidence

The initial Test262 selection manifest identifies four real files from the exact
pinned revision `3655e7464de3d52643ecddd4b5f9f4f3e7f62398`.
Only revision, path, expected Git blob, relevant frontmatter identity, and
independent normative mapping are retained in the repository.

| Test262 path | Git blob | First-envelope mapping | Effective variants |
| --- | --- | --- | --- |
| `test/language/statements/with/strict-script.js` | `9514530c03705c8461a7aa42aeac52a0c027ed26` | `EE-23-R01` | strict only |
| `test/language/statements/class/syntax/early-errors/class-definition-evaluation-scriptbody-duplicate-binding.js` | `c5f3dff7cc6e23dfd729954698301e156a76985b` | `EE-36-R01` | non-strict + strict |
| `test/language/global-code/new.target.js` | `e8c55b1ce90046bda2201a51a49662a24b845430` | `EE-36-R04` | non-strict + strict |
| `test/language/literals/regexp/early-err-modifiers-code-point-repeat-i-1.js` | `055cebc72a993126baf3517420fbc85d929d1a49` | `EE-37-R04` | non-strict + strict |

Each seed is selected only because a normative rule unit already exists. Test262 path, metadata, or pass/fail behavior does not create the rule mapping. The selector verifies the pinned revision, retained path/blob/source identity, parse-negative metadata, first-envelope filtering, and deterministic effective-source transformation.

The removed `test/built-ins/**` Basic Emoji case is not authoritative for this
first gate. The replacement RegExp modifier case challenges the Pattern
modifier duplicate rule `EE-37-R04`; it is distinct from the RegExp literal
outer-flag rule exercised by `/a/gg`.

A runtime-only duplicate-flag `RegExp` constructor test was explicitly not selected: runtime `new RegExp(...)` behavior is outside this parse/static-source evidence gate even though it tests related flag semantics.

This seed is intentionally small. Test262 challenge coverage can grow as independently mapped rule units acquire evidence; Test262 coverage is never the aggregate qualification-completeness authority.
