# HTML Document Shell Construction: TC-S1

## Purpose and Authority

This document is the non-normative implementation guide for **TC-S1 —
Disabled-Scripting Document Shell Construction**, the first Core-private HTML
tree-construction capability, implemented under Issue #351 on the architecture
approved through Issue #117.

It specializes nothing and supersedes nothing. The normative records remain:

- [ADR 0010 — Define HTML Tree-Construction Architecture](../decisions/0010-html-tree-construction-architecture.md);
- [HTML Tree-Construction Architecture](../architecture/HTML_TREE_CONSTRUCTION.md);
- [Source Parser Ownership](../architecture/SOURCE_PARSER_OWNERSHIP.md);
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md); and
- [Secure Development](SECURE_DEVELOPMENT.md).

Where this guide and any of those appear to disagree, they govern and this
document is wrong.

This document describes what was implemented. It does **not** claim complete
HTML parsing, fragment parsing, foreign content, tables, formatting
reconstruction, scripting or parser reentrancy, a browser-compatible DOM, a
public Rust API, serialization or ABI compatibility, an independent tree
resource policy, or WASM runtime support.

The existing explicit-start-tag capability documented by
[HTML Core Analysis](HTML_CORE_ANALYSIS.md) and
[HTML Analysis Parser](HTML_ANALYSIS_PARSER.md) remains an unchanged sibling.
TC-S1 does not rewrite it, replace it, or reinterpret its results.

## Capability

TC-S1 answers exactly one bounded question:

> For retained source under ordinary document parsing with parser scripting
> disabled, what document shell — the Document root, `html`, `head`, `body`,
> and body text — does the HTML Standard construct, with exact authored or
> explicitly absent provenance, and how far did construction actually commit?

```text
&SourceText + HtmlTokenizerLimits
        ↓
driver — Core-owned TC-S1 coordination and effective completion
        ↓
existing batch tokenizer (unchanged)
        ↓
validated HtmlTokenizerRunResult
        ↓
session — private, exclusively mutable single-run construction state
        ↓
validated freeze
        ↓
result — immutable tree / provenance / action / diagnostic / completion
```

The operation is
`crate::html::tree_construction::driver::construct_html_document_shell`,
wired from `crates/frontend-analysis-core/src/html/mod.rs`.

## Placement and Ownership

| Module | Owns | Must not |
| --- | --- | --- |
| `tree_construction/driver.rs` | The Core TC-S1 operation, tokenizer invocation, token feeding, effective completion, consuming finalization | Hold construction state; reinterpret lower-layer meaning |
| `tree_construction/session.rs` | All mutable single-run state: insertion mode, open shell elements, head pointer, private document mode, `frameset-ok`, identity counter, temporary nodes, committed diagnostics/actions/coverage | Call the tokenizer; escape to a consumer |
| `tree_construction/result.rs` | Immutable validated tree, constructed identity, provenance, action and diagnostic evidence, committed coverage, effective completion, the freeze boundary | Hold mutable state; observe the tokenizer |

No generic DOM, AST, parse-request, parse-session, construction-event-log, or
plugin abstraction is introduced. Nothing here is `pub`.

## Fixed Configuration

TC-S1 is fixed to:

- ordinary document parsing (no fragment parsing);
- parser scripting mode **Disabled**; and
- the tokenizer's initial **Data** state.

There is no parse-request, parse-context, or capability-configuration
parameter, so an unsupported configuration cannot be selected — silently or
otherwise. The only caller-supplied configuration is the existing
`HtmlTokenizerLimits`, which remains the tokenizer's own contract and is not
reinterpreted as a tree budget.

Parser scripting is configuration, not JavaScript execution; nothing in Core
executes script. TC-S1's proved action set moreover contains no cell whose
behavior differs between scripting enabled and disabled, because every
scripting-sensitive element (`script`, `noscript`, `template`) lies outside the
`html`/`head`/`body` shell and is therefore explicitly unsupported.

## Why the Batch Tokenizer Is Correct Here

The architecture requires that tree construction *be able* to control
subsequent tokenization where the HTML Standard demands it. TC-S1's theorem
does not demand it: none of its proved cells changes tokenizer state, and every
tokenizer state that tree construction would otherwise have to control
(`RCDATA`, `RAWTEXT`, script data, PLAINTEXT, `noscript`, foreign content) is
already the tokenizer's own explicit unsupported capability, so those tokens
never reach TC-S1 at all.

The completed token vector is therefore correct input for this slice. No
resumable-tokenizer seam, feedback channel, or tokenizer edit is introduced,
and none is authorized by TC-S1.

## The Proved Action Set

`session::classify` is the complete, exhaustive statement of what TC-S1 proves.
It is a free function of `(insertion mode, admitted token)` only: it takes no
session state, mutates nothing, and returns either the step to apply or typed
unsupported evidence. Every cell not listed below is explicitly unsupported.

**Admission** (a property of the token alone, decided before any insertion mode
runs): a tag is admitted only when its interpreted name is `html`, `head`, or
`body`, it carries no attributes, and it is not self-closing. Character tokens
and end of file are always admitted.

| Insertion mode | Admitted token | Action |
| --- | --- | --- |
| Initial | any | missing-DOCTYPE diagnostic, quirks document mode, reprocess in *before html* |
| Before html | `<html>` | insert **authored** `html`, → *before head* |
| Before html | anything else | insert **synthesized** `html`, reprocess in *before head* |
| Before head | `<head>` | insert **authored** `head`, → *in head* |
| Before head | anything else | insert **synthesized** `head`, reprocess in *in head* |
| In head | `</head>` | close `head`, → *after head* |
| In head | `<head>` | duplicate-head diagnostic; no node, no identity |
| In head | anything else | close `head` (implied), reprocess in *after head* |
| After head | `<body>` | insert **authored** `body`, clear `frameset-ok`, → *in body* |
| After head | anything else | insert **synthesized** `body`, reprocess in *in body* |
| In body | characters | insert or coalesce text |
| In body | `<body>` | duplicate-body diagnostic; no node, no identity; clear `frameset-ok` |
| In body | `</body>` | acknowledged end tag, → *after body* |
| In body | end of file | stop parsing |
| After body | `</html>` | acknowledged end tag, → *after after body* |
| After body | end of file | stop parsing |
| After after body | end of file | stop parsing |

No cell is admitted merely because a tag happens to be named `html`, `head`, or
`body`. A shell name reached in a document position the GOLD does not prove —
`<html>` in *in body*, `<head>` in *after head*, `</head>` in *after head* — is
refused exactly like `<p>`.

*After body* is supported only for the two cells the accepted GOLD proves: the
`</html>` end tag and end of file. The remaining `after body` rules the HTML
Standard defines — whitespace handling, comments, and reprocessing anything else
back in *in body* — are not proved by TC-S1 and stay refused. Supporting the
proved end-of-file cell is not a claim that *after body* is implemented.

Outside *in body*, the HTML Standard treats whitespace characters differently
from other characters, and the project-owned tokenizer emits contiguous Data
runs rather than one token per character. TC-S1 proves no whitespace-sensitive
handling and no character-run splitting, so a run whose handling would depend
on that distinction is refused rather than guessed at. Inside *in body*, all
characters are inserted identically, so no refusal is needed there.

## Constructed Identity

`HtmlConstructedNodeId` means **committed semantic creation-event order** and
nothing else. It is not a vector index, arena slot, pointer, allocation order,
`SourceId`, source range, token index, final tree position, or runtime
identity. It exposes no raw-value accessor.

The identity counter resolves the only fallible part of a creation event up
front (`reserve`), so every other fallible step — insertion-parent resolution,
open-element uniqueness, authored-evidence availability — is settled before the
first mutation, and the counter advances (`commit`) only once the whole
creation action has committed. A refused or unsupported action therefore
consumes no identity.

Relationships are stored as explicit identities, and both the session and the
frozen result resolve them by *searching* for a matching identity rather than
by indexing storage. Freeze independently validates uniqueness, mutual
parent/child recording, parent-before-child creation order, and full
reachability from the root.

No cross-result, cross-run, cross-edit, cross-revision, public, serialized, or
runtime-correlation stability is promised.

## Provenance

| Node | Authored source |
| --- | --- |
| Document root | none — explicit absence |
| Authored `html`/`head`/`body` | the exact retained complete start-tag anchor and raw-name anchor, cloned unchanged from the validated token |
| Synthesized `html`/`head`/`body` | none — explicit absence, with a synthesis cause |
| Text | the exact ordered non-empty authored character contributions |

Absence is explicit semantic absence. No empty range, dummy anchor,
nearest-token anchor, or parent anchor ever stands in for it.

**Trigger evidence is not authored origin.** Every action records the token
that caused it, including actions that create structure the token did not
author. End of file has no authored extent and receives no anchor at all. A
token that implies structure is never presented as that structure's origin, and
freeze rejects a result in which an unsupported trigger's range is also a
node's authored origin.

End tags, duplicate-body recovery, ignored duplicate-head handling,
reprocessing, and unsupported input create action and disposition evidence, not
placeholder nodes and not fake origins.

Text nodes retain the interpreted characters plus each contribution's exact
originating character-token anchor, in order. Adjacent runs coalesce into one
text node with multiple ordered contributions. There is no source rescanning,
source searching, endpoint reconstruction, or second tokenizer anywhere in this
subsystem.

## Valid Checkpoints and Freeze

Rule selection and mutation are separated, so an unsupported cell cannot mutate
anything and the session is a valid semantic construction checkpoint at every
instant. No rollback, snapshot, resume, cancellation, or generic checkpoint
framework is needed or introduced.

`result::freeze` is the single finalization boundary. It validates identity
inventory, structure, node evidence, action and diagnostic evidence, source
binding against the exact supplied `SourceText`, coverage, and completion,
returning a typed `HtmlTreeFreezeError` otherwise. A freeze or session
invariant failure is an operation/boundary error — neither an HTML parse
diagnostic nor unsupported input.

## Completion, Diagnostics, and Coverage

Effective `Complete` requires all three of:

1. the retained `HtmlTokenizerRunResult` is `Complete`;
2. every emitted token was processed through end of file by supported actions;
   and
3. freeze succeeded.

Freeze enforces all three independently, so a completion upgrade is rejected
rather than trusted.

```text
Tokenizer Complete + supported through EOF + freeze ok  → Complete
Tokenizer Complete + tree stopped at unproved input     → Incomplete(UnsupportedCapability)
Tokenizer Incomplete(any cause)                         → Incomplete(LowerLayerIncomplete)
Session or freeze invariant failure                     → Err
```

`LowerLayerIncomplete` deliberately carries no detail: the tokenizer's exact
`UnsupportedCapability`, `ResourceLimit`, `InvalidConfiguration`, and
`InternalInvariantFailure` meaning stays authoritative on the retained run and
is never copied into a lossy duplicate.

Tree diagnostics are authored-input evidence and are independent of completion:
a `Complete` TC-S1 result normally carries the missing-DOCTYPE diagnostic, and
a duplicate shell start tag adds a second. Tree unsupported input is **not**
evidence that the source is invalid HTML.

Committed tree coverage records both the committed source prefix and the number
of completely processed tokens, because byte coverage alone is not sufficient
progress evidence. It is a different measurement from the retained tokenizer
run's own coverage and must not be conflated with it: for `<body><p>` the
tokenizer processed all nine bytes while the tree committed only `[0,6)`.

## Resources and Structural Boundedness

**No tree resource limit, dimension, type, or numeric constant is introduced.**
TC-S1 is structurally bounded by the existing tokenizer limits plus the finite
shell theorem:

- **Open elements** are bounded to the admitted shell. A shell name can be
  pushed only while it is not already open, and `head` is closed before `body`
  opens, so the open-element state never exceeds two entries — enforced by an
  invariant, not by a depth limit.
- **Shell and root node count is constant apart from text**: the Document root
  plus at most one each of `html`, `head`, and `body`.
- **Text nodes and contributions are bounded by emitted character evidence**:
  each character token contributes at most one contribution and at most one new
  text node, and character tokens are already bounded by the tokenizer's
  emitted-token limit.
- **Diagnostics, actions, and dispositions are bounded by processed emitted
  tokens plus a fixed shell contribution.**
- **Each admitted token follows a finite mode path.** Reprocessing is only ever
  expressed as a transition to a *strictly later* insertion mode, and
  `switch_mode` refuses any transition that is not strictly forward. A finite
  strictly increasing walk over a finite ordered enum cannot loop.
- **There is no recursion** in construction and **no independent tree loop**.
- **Unsupported actions stop before mutation**, structurally, because
  classification is a pure function that never touches session state.

Implementation revealed no independently unbounded or refusing tree state, so
no resource-policy gate was required.

## Validation

Test-only, all inside the nine-file envelope:

- `crates/frontend-analysis-core/src/html/tree_construction/validation.rs` —
  the candidate-independent GOLD and focused tests. Expected meaning is
  authored against a small independent model (`GoldNode`, `GoldOrigin`,
  `GoldDiagnostic`, `GoldCompletion`) that deliberately does not reuse the
  production result enums as its oracle, and is never generated from production
  output.
- `crates/frontend-analysis-core/src/html/tokenizer/validation/tree_construction_gate.rs` —
  the cross-layer gate, registered beside the existing parser and
  Core-analysis gates. It reuses the existing 76 candidate-independent
  tokenizer fixtures (72 initial plus 4 supplemental `REG-`) and the existing
  deterministic 4,096-input generator **without editing or copying either**.

### Candidate-independent GOLD

| Case | Source | Required observation |
| --- | --- | --- |
| G1 | `` (empty) | `R → H(Hd, B)`; all synthesized, no origins; end-of-file trigger; 1 diagnostic; Complete |
| G2 | `hello` | `R → H(Hd, B(T:"hello"))`; text origin `[0,5)`; shell synthesized; 1 diagnostic; Complete |
| G3 | `<html><head></head><body></body></html>` | authored H `[0,6)`/`[1,5)`, Hd `[6,12)`/`[7,11)`, B `[19,25)`/`[20,24)`; end tags action-only; 1 diagnostic; Complete |
| G4 | `<body></body>` | H/Hd synthesized; authored B `[0,6)`/`[1,5)`; the body end tag is action-only; 1 diagnostic; Complete |
| G5 | `<head></head>` | H/B synthesized; authored Hd `[0,6)`/`[1,5)`; the head end tag closes the authored head; end of file triggers B; 1 diagnostic; Complete |
| G6 | `<body>x</body>` | authored B `[0,6)` and text `[6,7)`; H/Hd synthesized; the body end tag is action-only; 1 diagnostic; Complete |
| G7 | `<body><body></body>` | exactly one B, authored by the first `[0,6)`; the second `[6,12)` is duplicate-body action with no node; the body end tag is action-only; 2 diagnostics; Complete |
| Auxiliary | `<head><head></head>` | exactly one authored Hd from the first `[0,6)`; the second `[6,12)` ignored with no node; the head end tag closes the authored head; H/B synthesized; 2 diagnostics; Complete |
| G8 | `<body><p>` | frozen `R → H(Hd, B)`; B authored `[0,6)`; no `p` node and no `p` identity; exact `<p>` `[6,9)` unsupported; tree commit `[0,6)`; 1 diagnostic; Incomplete(UnsupportedCapability) |
| G9 | repeat G1–G8 | identical semantic creation correspondence, tree, order, origins, actions, diagnostics, completion, and checkpoint; no raw identity encoding asserted |
| G10 | no fabricated input or limit | structural boundedness and lower-layer resource propagation only |

### Cross-layer gate

For every one of the 76 fixtures and every one of the 4,096 generated inputs,
with no `catch_unwind`, so a production panic fails naturally:

- panic freedom;
- retained tokenizer evidence unchanged across the tree boundary;
- valid frozen relationships — parentless Document root, unique
  creation-ordered identities, mutual parent/child links, parent before child;
- exact retained source binding, cross-checked against the fixture gold's own
  authored spans rather than against production output;
- no false `Complete`, cross-checked against the fixture's independently
  authored tokenizer completion;
- honest unsupported propagation, including that committed tree coverage never
  runs past an unsupported trigger and that the trigger never leaks as a node's
  authored origin; and
- a deterministic semantic result across repeats and caller-supplied source
  identities.

The gate additionally re-runs the whole corpus under five hostile tokenizer
configurations (tiny source-byte, emitted-token, and transition-step limits;
zero transition-step and zero emitted-token invalid configurations) and
requires that a refused or truncated lower layer never becomes an effective
`Complete` tree.

**TC-S1 does not claim the tokenizer corpus is supported input.** 18 of the 76
candidate-independent fixtures reach effective `Complete` under TC-S1; the rest
are honestly reported as tree-unsupported or as retained lower-layer
incompleteness. The gate asserts a strict subset so a future change cannot
quietly claim otherwise.

## Sensitive Output

No error, `Debug`, or `Display` surface in this subsystem exposes arbitrary
authored source content. Every error variant carries only structural evidence:
constructed identities, roles, counts, `SourceId`, and `SourceRangeError`.
Node, text, action, and diagnostic `Debug` projections report source
identities, ranges, and byte lengths, never fragments. Malformed and untrusted
input produce typed results, never uncontrolled panics.

## Public Visibility and Deltas

Everything is `pub(crate)` or narrower; `HtmlDocumentShellParts`, the identity
counter, the session, and the freeze boundary are `pub(super)`.

```text
public Rust exports:                 0
public fields/types/functions:       0
serialization / ABI / wire format:   0
Cargo.toml / Cargo.lock:             0
workspace members/packages/targets:  0
dependencies / features:             0
async / runtime:                     0
threads/concurrency/channels/locks:  0
new Rc / Arc / interior mutability:  0
repository-authored unsafe:          0
browser protocol / runtime DOM:      0
filesystem / network / process:      0
public Send / Sync promise:          0
tree resource limits / constants:    0
```

## Limitations

TC-S1 proves the disabled-scripting document shell and nothing more. It does
not implement or claim: fragment parsing; RCDATA, RAWTEXT, or script-data
tokenizer feedback; JavaScript execution or parser reentrancy; foreign content,
SVG, or MathML; templates; tables or foster parenting; active formatting
reconstruction or adoption-agency behavior; a generic DOM, AST, or construction
event log; attribute semantics, including attribute merging on duplicate shell
tags; whitespace-sensitive character handling outside *in body*; document-mode
result exposure; runtime DOM correlation; arbitrary partial snapshots, resume,
rollback, or cancellation; a public Rust API or serialization; or WASM runtime
behavior.

It also does not implement *after body* or *after after body* generally: only
the `</html>` and end-of-file cells the accepted GOLD proves are supported
there, and every other rule in those modes is refused.

Adjacent HTML Standard behavior that TC-S1 does not prove remains explicitly
unsupported rather than approximated. Extending any of it requires its own
focused, validated scope.
