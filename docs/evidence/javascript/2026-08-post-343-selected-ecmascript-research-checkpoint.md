# Post-#343 Selected ECMAScript Research Checkpoint

Status date: 2026-08-24 JST

Classification: supporting research evidence; non-normative; candidate durable checkpoint.

Review state: independently adversarially reviewed against the live repository baseline and external normative/challenge evidence. This file is **not** repository authority until separately accepted through the repository's normal Issue / review / merge process.

## 1. Purpose

This record closes the research/evidence gap that accumulated after the accepted var-enabled selected-slice completion theorem in #313 while production and candidate-independent leaf validation continued through #343.

It does **not** define the whole ECMAScript language, complete Standard Qualification, a public parser, a general AST/IR, runtime binding semantics, browser behavior, or a new production frontier.

Its exact question is:

> At the live #343 checkpoint, what is the current **selected qualification source envelope**, which frozen first-envelope Early Error identities are required/reachable from that selected slice, which identities remain structurally non-triggering or envelope-inactive, which semantic owners are proven, and which future frontiers remain explicitly deferred?

This record preserves historical authorities as historical. It does not rewrite #268, #301, #307/#309, #313, or any post-#313 leaf oracle as though later source coverage existed at the time.

## 2. Live Repository Checkpoint

Verified live authority at research closure:

```text
repository: YT-TechDev/frontend-analysis
branch: main
commit: a65df8342ec7ee34358b1c8c2c96ca9116fe8d89
tree: 934b8a45165658cf299fe152d7436e84edeec3f5
parent: 965b276a4cffada75bede599d13f8190e69726d0
subject: feat(ecmascript): recognize selected top-level var EE-04-R08 initializer source positions (#343)
```

At verification time the only open pull request was unrelated Dependabot PR #149.

This checkpoint is invalidated as a *live-main* statement by any later relevant ECMAScript merge. Its evidence lineage remains historical even after such invalidation.

## 3. Evidence Authority and Envelope

Primary authority remains the frozen first qualification foundation:

```text
ECMA-262, 17th edition, 2026
snapshot: tc39/ecma262@d89c03f2db8a597bc915b363a6518d0cc8acdbc0

Unicode 17.0.0

current qualification Test262 challenge revision:
tc39/test262@3655e7464de3d52643ecddd4b5f9f4f3e7f62398
```

Authority direction remains:

```text
pinned normative specification / normative Unicode data
        ↓
project-owned candidate-independent validation evidence
        ↓
pinned Test262 challenge evidence within independently mapped scope
        ↓
production conformance evidence
```

Production behavior, browser behavior, external parser behavior, and Test262 pass/fail behavior do not define Frontend Analysis semantics.

The selected qualification envelope remains:

```text
source authority: Core UTF-8 SourceText
source context: Independent Source Unit
parse goal: Script
strict: false
Yield parameter: false
Await parameter: false
mandatory legacy source policy: included
Normative Optional source-static policy: not included
```

## 4. Terminology Guard

This document deliberately uses **current selected qualification source envelope**, not "current ECMAScript source-space".

The selected recognizer covers only a bounded subset. `UnsupportedCoverage` means that this selected theorem does not issue a source-standard verdict for that input. It does **not** mean the source is invalid ECMAScript.

Likewise, "9 required/reachable" means required/reachable **from the current selected slice within the frozen 193-rule first-envelope inventory**. It does not mean that only nine Early Errors are active in ECMA-262 or in the first qualification envelope generally.

## 5. Current Selected Qualification Source Envelope

### 5.1 Whole-source transactionality

Recognition is transactional over the authoritative `SourceText`. Tentative selected facts are returned only when the complete source is consumed by selected items plus selected trivia. Failed or unsupported tails do not commit a selected-success prefix.

Selected trivia currently consists of:

- U+0009 CHARACTER TABULATION;
- U+000B LINE TABULATION;
- U+000C FORM FEED;
- U+FEFF ZERO WIDTH NO-BREAK SPACE;
- LF, CR, LS, PS; and
- Unicode `Space_Separator` code points recognized by the project Unicode owner.

Comments are not selected trivia in this slice.

### 5.2 Positive recognition partitions

Positive recognition is an exact three-way representation partition:

```text
HistoricalFlat
  selected top-level LexicalDeclaration+

BlockEnabledWithoutVar
  top-level selected LexicalDeclaration / selected one-level Block
  at least one selected Block
  no selected VariableStatement

VariableEnabled
  top-level selected LexicalDeclaration / selected one-level Block / selected top-level VariableStatement
  at least one selected VariableStatement
```

`SelectedBlock` remains one level and non-empty. Its body remains `LexicalDeclaration+`; Block-contained `var` is not selected.

### 5.3 Selected lexical declarations

The retained lexical declaration slice owns `let` / `const`, 1..N simple selected bindings, authored semicolon or final EOF-only ASI, and the existing selected initializer atoms.

The currently selected lexical initializer atoms include:

- selected decimal integer;
- direct BooleanLiteral `true` / `false`;
- direct `null`;
- direct `this`;
- direct escape-free StringLiteral;
- direct IdentifierReference;
- escaped non-ReservedWord IdentifierReference; and
- escaped IdentifierName whose decoded StringValue is an unconditionally reserved word, retained only as EE-04-R08 rejection evidence.

This is intentionally wider than the current var initializer slice.

### 5.4 Selected top-level `var`

The current var source family is conceptually:

```text
SelectedVariableStatement
  ::= var SelectedVariableDeclarationList SelectedVarTerminator

SelectedVariableDeclarationList
  ::= 1..N SelectedVariableDeclaration

SelectedVariableDeclaration
  ::= SelectedBindingIdentifier
   |  SelectedBindingIdentifier = SelectedDecimalInteger
   |  SelectedBindingIdentifier = SelectedDirectIdentifierReference
   |  SelectedBindingIdentifier = SelectedEscapedNonReservedIdentifierReference
   |  SelectedBindingIdentifier = SelectedEscapedReservedIdentifierName[C6 rejection route]

SelectedVarTerminator
  ::= AuthoredSemicolon
   |  AutomaticAtEof
```

The C6 route is **recognized source with retained static-rejection evidence**, not a positive accepted initializer semantic class.

Current var does not select Boolean/null/`this`/string initializers, BindingPattern/destructuring, `for (var ...)`, Block-contained `var`, comments, or non-EOF ASI.

### 5.5 Disposition partition

The selected integration distinguishes:

```text
UnsupportedCoverage
SyntaxRejected            <- definitive selected Grammar evidence only
StaticSemanticsRejected   <- retained selected static-semantic evidence
SelectedAcceptedIncomplete
ResourceLimited
InternalFailure
```

`Qualified` is not produced by this selected integration.

## 6. Frozen Inventory Theorem

The frozen first-envelope inventory remains:

```text
37 audited Early Error clause containers
34 containers with at least one active first-envelope rule
193 audited rule identities
  183 active NormativeRule identities
   10 EnvelopeInactiveRule identities
```

The inventory is identity-bearing, not merely a global count: each container's expected active/inactive rule counts are fixed and missing/substituted identities fail closed.

## 7. Current Selected-Slice Reachability Closure

### 7.1 Required/reachable identities

Exactly these nine frozen rule identities are required/reachable from the current selected slice:

- `EE-01-R01`
- `EE-01-R02`
- `EE-04-R08`
- `EE-14-R01`
- `EE-15-R01`
- `EE-15-R02`
- `EE-15-R03`
- `EE-36-R01`
- `EE-36-R02`

Therefore:

```text
current selected required/reachable = 9
active but structurally non-triggering = 174
envelope-inactive = 10
structurally non-triggering or inactive total = 184
frozen universe = 193
```

The 10 envelope-inactive identities remain visible in the 184-side accounting; they are not silently reclassified as active selected obligations.

### 7.2 Explicit non-triggering / inactive identity set

The complement of the nine current selected required identities in the frozen 193-rule universe is the following explicit 184-identity set:

- **EE-02:** `EE-02-R01`
- **EE-03:** `EE-03-R01`
- **EE-04:** `EE-04-R01`, `EE-04-R02`, `EE-04-R03`, `EE-04-R04`, `EE-04-R05`, `EE-04-R06`, `EE-04-R07`, `EE-04-I01`, `EE-04-I02`
- **EE-05:** `EE-05-R01`, `EE-05-R02`, `EE-05-R03`, `EE-05-R04`
- **EE-06:** `EE-06-R01`
- **EE-07:** `EE-07-R01`, `EE-07-R02`, `EE-07-R03`, `EE-07-R04`, `EE-07-R05`
- **EE-08:** `EE-08-R01`
- **EE-09:** `EE-09-R01`, `EE-09-R02`
- **EE-10:** `EE-10-R01`, `EE-10-R02`
- **EE-11:** `EE-11-R01`, `EE-11-R02`
- **EE-12:** `EE-12-R01`, `EE-12-R02`, `EE-12-R03`, `EE-12-R04`
- **EE-13:** `EE-13-R01`, `EE-13-R02`, `EE-13-R03`, `EE-13-R04`
- **EE-14:** `EE-14-R02`
- **EE-16:** `EE-16-I01`, `EE-16-I02`, `EE-16-I03`
- **EE-17:** `EE-17-I01`
- **EE-18:** `EE-18-I01`
- **EE-19:** `EE-19-R01`, `EE-19-I01`
- **EE-20:** `EE-20-R01`, `EE-20-R02`, `EE-20-R03`, `EE-20-R04`, `EE-20-R05`, `EE-20-I01`
- **EE-21:** `EE-21-R01`
- **EE-22:** `EE-22-R01`
- **EE-23:** `EE-23-R01`, `EE-23-I01`
- **EE-24:** `EE-24-R01`, `EE-24-R02`
- **EE-25:** `EE-25-R01`
- **EE-26:** `EE-26-R01`, `EE-26-R02`, `EE-26-R03`
- **EE-27:** `EE-27-R01`, `EE-27-R02`
- **EE-28:** `EE-28-R01`, `EE-28-R02`, `EE-28-R03`, `EE-28-R04`, `EE-28-R05`, `EE-28-R06`, `EE-28-R07`, `EE-28-R08`, `EE-28-R09`, `EE-28-R10`, `EE-28-R11`, `EE-28-R12`, `EE-28-R13`
- **EE-29:** `EE-29-R01`, `EE-29-R02`, `EE-29-R03`, `EE-29-R04`, `EE-29-R05`
- **EE-30:** `EE-30-R01`, `EE-30-R02`, `EE-30-R03`, `EE-30-R04`, `EE-30-R05`
- **EE-31:** `EE-31-R01`, `EE-31-R02`, `EE-31-R03`, `EE-31-R04`, `EE-31-R05`, `EE-31-R06`, `EE-31-R07`, `EE-31-R08`, `EE-31-R09`, `EE-31-R10`, `EE-31-R11`, `EE-31-R12`, `EE-31-R13`
- **EE-32:** `EE-32-R01`, `EE-32-R02`, `EE-32-R03`, `EE-32-R04`, `EE-32-R05`, `EE-32-R06`, `EE-32-R07`, `EE-32-R08`, `EE-32-R09`, `EE-32-R10`, `EE-32-R11`, `EE-32-R12`, `EE-32-R13`, `EE-32-R14`, `EE-32-R15`
- **EE-33:** `EE-33-R01`, `EE-33-R02`, `EE-33-R03`, `EE-33-R04`, `EE-33-R05`, `EE-33-R06`, `EE-33-R07`, `EE-33-R08`, `EE-33-R09`, `EE-33-R10`, `EE-33-R11`, `EE-33-R12`, `EE-33-R13`, `EE-33-R14`, `EE-33-R15`, `EE-33-R16`, `EE-33-R17`, `EE-33-R18`, `EE-33-R19`, `EE-33-R20`
- **EE-34:** `EE-34-R01`, `EE-34-R02`, `EE-34-R03`, `EE-34-R04`, `EE-34-R05`, `EE-34-R06`, `EE-34-R07`, `EE-34-R08`, `EE-34-R09`, `EE-34-R10`, `EE-34-R11`, `EE-34-R12`, `EE-34-R13`
- **EE-35:** `EE-35-R01`, `EE-35-R02`, `EE-35-R03`, `EE-35-R04`, `EE-35-R05`, `EE-35-R06`
- **EE-36:** `EE-36-R03`, `EE-36-R04`, `EE-36-R05`, `EE-36-R06`, `EE-36-R07`, `EE-36-R08`
- **EE-37:** `EE-37-R01`, `EE-37-R02`, `EE-37-R03`, `EE-37-R04`, `EE-37-R05`, `EE-37-R06`, `EE-37-R07`, `EE-37-R08`, `EE-37-R09`, `EE-37-R10`, `EE-37-R11`, `EE-37-R12`, `EE-37-R13`, `EE-37-R14`, `EE-37-R15`, `EE-37-R16`, `EE-37-R17`, `EE-37-R18`, `EE-37-R19`, `EE-37-R20`, `EE-37-R21`, `EE-37-R22`, `EE-37-R23`, `EE-37-R24`, `EE-37-R25`, `EE-37-R26`

This identity set is a current research closure. A future executable completion successor should independently literalize and validate the partition against `RULE_UNITS`; it should not merely assert `193 - 9`.

## 8. Post-#313 Delta-Composition Theorem

#313 was a full var-enabled selected-slice completion closure for its then-current grammar. It remains historical evidence, but its grammar string is not current after later widenings.

The accepted post-#313 source/semantic deltas relevant to the current source envelope are:

| Delta | Validation authority | Production successor | Effect on selected source envelope | Newly reachable frozen EE identity outside the #313 nine? |
| --- | --- | --- | --- | --- |
| same-source var correspondence | #314/#315 | #316/#317 | semantic capability only; no grammar reachability widening | No |
| final top-level var EOF-only ASI | #318/#319 | #320/#321 | statement terminator widening | No |
| 1..N VariableDeclarationList | #322/#323 | #324/#325 | declarator cardinality widening | No |
| selected decimal-integer var initializer | #326/#327 | #328/#329 | initializer leaf widening | No |
| direct IdentifierReference var initializer | #330/#331 plus #332/#333 composition | #334/#335 | reference-bearing source position + correspondence widening | No |
| escaped non-ReservedWord IdentifierReference var initializer | #336/#337 | #338/#339 | escaped reference source position + decoded semantic identity | No |
| escaped ReservedWord initializer EE-04-R08 source position | #340/#341 | #342/#343 | new source route to already-required EE-04-R08 | No |

Hence, for each post-#313 grammar/static delta `Di`:

```text
NewlyReachableFrozenEE(Di) ⊆ R313
```

and therefore:

```text
R343 = R313
|R343| = 9
|U - R343| = 184
```

### 8.1 Why the deltas do not add a tenth identity

- **EOF-only ASI** changes terminator provenance, not an Early Error owning production.
- **1..N var declarators** increases authored contributors but duplicate `var` names do not create a new duplicate-var Early Error; the already-reachable Script lexical/var intersection remains the relevant selected Script rule.
- **Decimal integer** is restricted to `0 | [1-9][0-9]*` in a non-strict Script envelope, so the strict LegacyOctal / NonOctalDecimal rule remains non-triggering.
- **Direct IdentifierReference** does not enable strict-only, Module-only, Yield-parameter, Await-parameter, `super`, NewTarget, label, break/continue, private-name, object, assignment, function/class, or RegExp owning syntax.
- **Escaped non-ReservedWord IdentifierReference** remains behind position-valid Unicode-identifier and C1/C6 firewalls; it does not launder invalid escaped positions into positive reference semantics.
- **C6 escaped ReservedWord** reaches the already-required Identifier Early Error. Unicode escape spelling does not turn the source into the corresponding ReservedWord keyword production such as actual `super` syntax.

## 9. Static-Semantics Ownership and Evidence Ordering

The current selected static owner consumes retained source-backed facts and does not reparse authoritative source.

Current private evidence-selection tiers are:

```text
Tier 1
  declaration / binding-local selected checks
  including EE-01, EE-04-R08, EE-15 and var binding-local checks

Tier 2
  selected Block-local lexical duplicate
  EE-14-R01

Tier 3
  top-level Script lexical duplicate
  EE-36-R01

Tier 4
  top-level Script lexical / selected top-level var name collision
  EE-36-R02
```

This is a project evidence-selection policy, not an ECMA-262 diagnostic-order mandate.

Current Blocks contain no local var contributor, so EE-14-R02 remains structurally non-triggering.

## 10. Qualification Lifecycle Theorem

The selected integration preserves staged implementation honesty:

```text
recognized + selected static accepted
→ SelectedAcceptedIncomplete
```

A selected grammar rejection becomes `SyntaxRejected`; a selected static rejection becomes `StaticSemanticsRejected`; unsupported selected coverage remains `UnsupportedCoverage`.

No production `CompleteQualificationWitness` constructor is available to this path. `Qualified` therefore remains unreachable from the current selected integration.

## 11. Same-Source Correspondence Firewall

The var-enabled correspondence capability consumes only the exact var-enabled selected-static acceptance witness.

It may report:

```text
VisibleSelectedLexicalBinding { binding, region }
SameSourceSelectedVarNameContributors { contributors }
NoSelectedSameSourceContributor
```

Var contributors preserve every authored selected top-level `var` declaration occurrence in authored order; they are provenance contributors, not one deduplicated logical runtime target.

The capability explicitly does **not** claim runtime `ResolveBinding`, Environment Record identity, hoisting, TDZ, initialization, execution order, runtime unresolvability, `ReferenceError`, or value flow.

Current var initializer references are top-level correspondence inputs. Block-contained var is therefore a genuine future correspondence/domain widening, not an already-supported case.

## 12. Historical-to-Current Evidence Lineage

Historical authorities remain immutable:

```text
flat selected completion
→ one-level Block selected completion
→ bounded top-level var selected completion (#313)
→ post-#313 bounded successor oracles
→ #343 current research checkpoint (this record)
```

The old #313 completion file remains valid for its historical checkpoint but is stale as a literal description of current var grammar because it still records:

```text
VariableStatement ::= var SelectedBindingIdentifier ;
```

This record does not amend that historical theorem. A new completion successor is required for executable current closure.

## 13. Evidence-Lineage Clarifications / Errata

### 13.1 Test262 revision roles

`docs/evidence/javascript/README.md` records `be13516fb6441b950ba8a3df97eb34062c186972` as the Test262 revision used by an earlier research audit.

The current first Standard Qualification validation foundation and provenance ledger use `3655e7464de3d52643ecddd4b5f9f4f3e7f62398` as the qualification challenge revision.

These are different historical roles, not a semantic contradiction. Future current-state summaries should label them explicitly as **historical research pin** versus **current qualification challenge pin**.

### 13.2 #336 reserved-name count marker

The #336 escaped-IdentifierReference successor oracle contains an asserted `UNCONDITIONALLY_RESERVED_NAME_COUNT = 35` marker. That marker does not define the current C6 set and is superseded for current-state reporting by #340's explicit 36-member C6 list.

The current checkpoint therefore records the explicit #340 set as current authority and treats the #336 count as historical metadata mismatch. Historical source remains immutable.

## 14. External Challenge Evidence

External evidence is corroborating, not defining.

The official ECMAScript 2026 publication independently confirms the load-bearing rules used by this closure:

- Block rejects duplicate `LexicallyDeclaredNames` and intersections of `LexicallyDeclaredNames` with `VarDeclaredNames`.
- Script rejects duplicate lexical names and Script lexical/var intersections, and separately owns `Contains super`, NewTarget, label, break/continue, and private-identifier rules.
- Identifier Early Errors use `StringValue`; Unicode escapes cannot be used to obtain an Identifier whose code-point sequence equals an excluded ReservedWord.

Pinned Test262 challenge evidence at the current qualification revision includes Block lexical/var redeclaration negatives in both authored orders, e.g. `{ var f; let f }` and `{ let f; var f }`. This corroborates why Block-contained var would activate a currently non-triggering Block rule.

## 15. Deferred Frontiers

The following are **not** part of this current closure:

### 15.1 Block-contained `var`

High-value deferred frontier. It is not a trivial parser widening because it can make EE-14-R02 reachable and requires new reasoning for:

- Block-local `VarDeclaredNames`;
- Script-level propagation of var-declared names;
- Block-local vs Script-level collision evidence ordering;
- var initializer correspondence from a Block region; and
- representation/domain ownership without silently collapsing existing capability contracts.

A candidate-independent oracle is required before production placement.

### 15.2 `for (var ...)`

Deferred statement/header/topology frontier. It must not be inferred from top-level `VariableStatement` support.

### 15.3 BindingPattern / destructuring

Deferred binding-domain widening. Current selected var uses simple selected BindingIdentifier bindings only.

### 15.4 Comments and non-EOF ASI

Deferred source/trivia/terminator frontiers. Current selected trivia does not include comments and var ASI is EOF-only.

### 15.5 Additional top-level var initializer atoms

Boolean/null/`this`/escape-free string are already selected for lexical declarations but not for current var. These are potential bounded top-level source widenings; they are not automatically Stage-D blockers. They require fresh frontier reassessment and candidate-independent evidence before production.

## 16. Research Closure Decision

The adversarial research performed for this checkpoint found:

```text
no production contradiction requiring rollback
no tenth current selected Early Error identity
no evidence that Block-contained var is already selected
no current Qualified path
no runtime-binding contamination of same-source correspondence
no need to unify flat / Block / var result representations merely to close #343
```

The durable conclusion is:

```text
CURRENT SELECTED SEMANTIC CLOSURE: PASS
CURRENT 9 / 184 REACHABILITY CLOSURE: PASS
CURRENT SOURCE-DISPOSITION BOUNDARY: PASS
CURRENT LIFECYCLE FIREWALL: PASS
CURRENT CORRESPONDENCE/RUNTIME FIREWALL: PASS
CURRENT AGGREGATE EXECUTABLE COMPLETION AUTHORITY: STALE / NEEDS SUCCESSOR
BLOCK-CONTAINED VAR: DEFERRED / NEEDS NEW RESEARCH ORACLE
```

This checkpoint is therefore sufficient research input for a **new candidate-independent post-#343 completion successor**, but it does not itself authorize production widening.

## 17. Recommended Next Repository Sequence

```text
1. Freeze/review this research checkpoint as durable evidence.
2. Add a validation-only post-#343 completion successor.
3. Preserve historical completion authorities zero-diff.
4. Perform a fresh production-frontier reassessment from the new current authority.
5. If Block-contained var wins, run its candidate-independent semantic oracle before Stage-D placement.
6. Only then create a production Issue.
```

## 18. Repository Evidence References

Primary project evidence at the live checkpoint:

- `docs/architecture/JAVASCRIPT_ARCHITECTURE.md`
- `docs/decisions/0009-javascript-semantic-analysis-architecture.md`
- `docs/evidence/javascript/README.md`
- `docs/evidence/javascript/2026-08-first-standard-qualification-validation-foundation.md`
- `docs/provenance/ecmascript.md`
- `crates/frontend-analysis-core/src/ecmascript/qualification.rs`
- `crates/frontend-analysis-core/src/ecmascript/selected_lexical_slice.rs`
- `crates/frontend-analysis-core/src/ecmascript/selected_static_semantics.rs`
- `crates/frontend-analysis-core/src/ecmascript/selected_qualification_integration.rs`
- `crates/frontend-analysis-core/src/ecmascript/selected_variable_statement_name_correspondence.rs`
- `crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/inventory.rs`
- `crates/frontend-analysis-core/src/ecmascript/qualification_validation_tests/selected_variable_statement_slice_completion.rs`
- post-#313 candidate-independent successor modules for #318, #322, #326, #330/#332, #336 and #340.

External corroboration:

- ECMAScript 2026, Block static semantics: `https://tc39.es/ecma262/2026/multipage/ecmascript-language-statements-and-declarations.html`
- ECMAScript 2026, Identifier static semantics: `https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html`
- ECMAScript 2026, Script static semantics: `https://tc39.es/ecma262/2026/multipage/ecmascript-language-scripts-and-modules.html`
- Test262 challenge revision: `tc39/test262@3655e7464de3d52643ecddd4b5f9f4f3e7f62398`
