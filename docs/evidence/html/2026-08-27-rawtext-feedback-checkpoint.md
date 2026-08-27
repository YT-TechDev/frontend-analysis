# HTML RAWTEXT Feedback and Post-TC-S8 Evidence Checkpoint

Status date: 2026-08-27

Classification: task and evidence record; non-normative.

## Purpose and Authority Boundary

This record preserves the HTML tree-construction evidence established after the
2026-08-26 tree-construction frontier checkpoint. It records the completion of
TC-S7 and TC-S8 production, the accepted candidate-independent `<style>` RAWTEXT
feedback validation, and the focused TC-S9 production-placement decision.

This record is deliberately an **evidence checkpoint**, not a normative
architecture document and not production authorization. It does not amend or
supersede:

- [ADR 0010 — Define HTML Tree-Construction Architecture](../../decisions/0010-html-tree-construction-architecture.md);
- [HTML Tree-Construction Architecture](../../architecture/HTML_TREE_CONSTRUCTION.md);
- [Source Parser Ownership](../../architecture/SOURCE_PARSER_OWNERSHIP.md);
- the durable #117 maintainer decisions; or
- the historical [2026-08 Tree-Construction Frontier Checkpoint](2026-08-tree-construction-frontier-checkpoint.md).

The evidence boundary remains:

```text
Evidence record
!= normative architecture
!= production implementation
!= production authorization
!= public compatibility contract
!= browser/runtime DOM authority
```

The older 2026-08 checkpoint remains a historical statement of the TC-S7
validation-era frontier. This record is additive; it does not rewrite that
history to make later production work appear earlier than it occurred.

## Exact Repository Authority

Current repository authority at this checkpoint:

```text
main:
e5c299e3980f6d3de41c6291b86890f19715890d

tree:
d9bb7c2f0c90fb23ff24865fca5f2ffde5143e59

HEAD subject:
test(html): pin selected in-head style RAWTEXT feedback lifecycle (#385)
```

The current `main` tree is the squash-merge tree of the accepted RAWTEXT
candidate-independent validation. Post-merge Rust Core validation is:

```text
Rust Core #622
run: 33038311382
job: 98405966771
result: PASS
```

At the recorded base checkpoint, immediately before this evidence Leaf PR was
opened, the live repository inventory identified only unrelated Dependabot PR
#149 as open.

## Durable Program and Research Authority

The governing evidence/architecture chain remains:

- #348 — post-vertical-slice R1-R10 / Wave 1E HTML research foundation;
- #117 — HTML tree-construction architecture and production-program authority;
- ADR 0010 — accepted Candidate C rationale;
- `docs/architecture/HTML_TREE_CONSTRUCTION.md` — active specialized normative
  contract;
- #384 / PR #385 — accepted candidate-independent `<style>` RAWTEXT feedback
  validation;
- #117 `issuecomment-5434236001` — RAWTEXT validation completion checkpoint;
- #117 `issuecomment-5434360274` — accepted TC-S9 production-placement
  checkpoint.

Candidate C remains unchanged:

```text
SourceText + Parse Configuration
        ↓
Core-owned Parse Coordinator
        ↕
Pull / Resumable Token Production
        ↓
Private Mutable Tree-Construction Session
        ↓
Validated Freeze
        ↓
Immutable Query-Oriented Tree Analysis
        +
Selective Provenance / Recovery Relations
```

The RAWTEXT validation does not create a new architecture direction. It is the
first bounded successor that directly validates the already-approved
coordinator/tokenizer feedback requirement.

## Production Progression Through TC-S8

The preceding evidence checkpoint recorded production through TC-S6 and
candidate-independent validation for TC-S7. Production subsequently advanced as
follows.

### TC-S7 — selected InBody `</body>` transition over the open bounded stack

Validation authority:

```text
Issue / PR:
#374 / #375

accepted validation head:
2b5a41749ee7cd32e4504b6682aeecc415937c9f

accepted validation tree:
503cc2fc549638ac58cfb2931a268193a27bbb39

independent validation review:
5029212073

validation squash merge:
5e9ff4d539ebc75a7f812185ef55f0e85e881e72
```

Production authority:

```text
Issue / PR:
#378 / #379

accepted production head:
9a2acf6918f05f37b75cc5c926087b6ad4d763dd

accepted production tree:
ab2c612a9ba4be4c3b4f32e822fc553655a15d7f

fresh exact-head review:
5030625962

final merge-gate review:
5031094651

Rust Core #601:
run 32970679552 — PASS

production squash merge:
024e0e62520d1c7e298605eb4539434ef6c8d5e7
```

TC-S7 established the bounded plain InBody `</body>` transition over
`[html, body] ++ B* ++ P?`, preserving selected/P identity and open-stack state,
with the accepted diagnostic cardinality and AfterBody composition. It did not
change Candidate C, tokenizer ownership, the Core/public boundary, generic
scope/implied-end machinery, or resource policy.

### TC-S8 — selected InBody `</html>` transition

Candidate-independent validation authority:

```text
Issue / PR:
#380 / #381

accepted validation head:
b54a4fc585ad7f5350526fbd92bcab68a303ac4d

accepted validation tree:
951f6063e3d2d1fe2fd536b5cf57d83f93af2a8a

independent exact-head acceptance review:
5035767783

Rust Core #610:
run 32997205861
job 98269476494 — PASS

validation squash merge:
7f43dd38486c07af2c694813a9035a2cc626cfa6
```

Production authority:

```text
Issue / PR:
#382 / #383

accepted production head:
42d77ba8422af38b43b9e6f97a20d0354c1bd54b

accepted production tree:
b7fd54b01110d04968ab09f8485e3c4cce8cccfc

independent exact-head acceptance review:
5036632975

pre-merge Rust Core #613:
run 33031743837
job 98385598275 — PASS

production squash merge:
5ffb2eacf0b6cd77b7531a68408cb8e2ceba28b8

post-merge Rust Core #614:
run 33032336864
job 98387449968 — PASS
```

TC-S8 established the bounded plain InBody `</html>` transition over the same
bounded stack theorem, including the accepted body-end audit relation,
`ReprocessedToken -> AcknowledgedShellEndTag { Html }` chronology, stack and
identity preservation, and AfterAfterBody transition. The production completion
checkpoint is #117 `issuecomment-5433470518`.

## Strongest Production Theorem Before TC-S9

Production is merged through TC-S8. The selected bounded open-stack theorem
remains:

```text
[html, body] ++ B* ++ P?

B in {Div, Section}
count(P) <= 1
P present => P is current
```

TC-S7 and TC-S8 extend lifecycle transitions around this stack but do not turn
that theorem into general InBody parsing. In particular, production still does
not prove generic scope, generic implied ends, arbitrary element names, active
formatting elements, tables, templates, foreign content, fragments, or
script/reentrant parsing.

## Fresh Post-TC-S8 Frontier

The fresh post-TC-S8 reassessment selected the following bounded successor for
candidate-independent validation:

**Selected In-Head `<style>` RAWTEXT Feedback Lifecycle**.

The candidate was selected because it introduces the first unavoidable
**tree-construction -> tokenizer-state feedback round trip** while remaining
narrow enough to avoid RCDATA character references, script execution,
reentrancy, generic scope/implied-end machinery, fragments, tables/templates,
and foreign content.

The critical challenge fixture is conceptually:

```html
<head><style><b>x</style><body>
```

The `<b>` bytes after the `<style>` start tag must be recognized as RAWTEXT
character data rather than as a Data-state start tag. The later `<body>` acts as
a return-to-Data sentinel.

This candidate therefore directly challenges the previously local success of a
completed batch token vector. It does not invalidate batch tokenization for
bounded capabilities whose theorem has no tree-directed tokenizer feedback.

## Focused Normative and Challenge-Evidence Freshness

The broad #348 research pin remains historical research authority. The focused
RAWTEXT validation additionally pinned the current relevant WHATWG/WPT state:

```text
WHATWG HTML main:
df14ce3085887cc99d821d238c5192857904de58

tree:
2244cb816f3ab2882fccab4cd77c3bcc7183fd09

source blob used by the validation:
16cdaecdb5f3db29eac0753a49f401b221ba9247

WPT validation pin:
5ce815a83b2601ce920e39f001cd7e77642ea860

tree:
d69ddc0460fd07c6f3f2bc85d6a1528b42dae278
```

At the production-placement gate, WHATWG HTML remained at the same commit. WPT
had advanced one commit to:

```text
WPT master:
d220e42320812cb466cf61960ed058e7ef9cc19a

tree:
c45aa88ee68d49e36a8052ab1817704f19444256
```

That WPT delta concerns scroll/anchor-positioned layout behavior and is
non-material to the selected HTML parsing theorem. The current WHATWG head's
fragment-context editorial change is likewise non-material to this
ordinary-document candidate.

The authority classification remains strict:

```text
WHATWG HTML normative algorithm text: primary semantic authority
WPT / html5lib-derived parsing data:  challenge / corroboration evidence
browser output:                        comparison evidence
production implementation output:     never the candidate-independent oracle
```

WPT and html5lib data with shared lineage are not counted as independent votes.

## Candidate-Independent RAWTEXT Validation Authority

Validation authority:

```text
Issue:
#384

PR:
#385

accepted exact validation head:
6ffda93ca6bcd4465152f45a2e7621a33e1f23fa

accepted tree:
d9bb7c2f0c90fb23ff24865fca5f2ffde5143e59

fresh exact-head re-review:
5037122143

pre-merge Rust Core #621:
run 33037207442
job 98402413230 — PASS

squash merge / current main:
e5c299e3980f6d3de41c6291b86890f19715890d

post-merge Rust Core #622:
run 33038311382
job 98405966771 — PASS
```

The validation is test-only and candidate-independent. Its transition machine,
hand-authored normative GOLD, source evidence, and production implementation are
not collapsed into one oracle.

## Validated Causal Theorem

For an accepted appropriate-close path, the validation establishes the bounded
ordering:

```text
Data
→ authored <style> recognized/emitted
→ tree inserts Style in InHead
→ tree requests RAWTEXT
→ coordinator applies RAWTEXT before later source production
→ original insertion mode InHead retained
→ tree enters Text
→ subsequent source recognized under RAWTEXT
→ RAWTEXT characters inserted under Style
→ appropriate </style> recognized
→ tokenizer returns RAWTEXT -> Data
→ Text consumes the end tag
→ Style popped
→ original InHead restored
→ subsequent source produced under Data
```

This is a causal theorem, not only a final-tree theorem. A parser that first
produces all later source under Data and repairs the result afterward does not
satisfy it.

EOF is deliberately distinct. When EOF is encountered while the selected Style
is still open in Text/RAWTEXT, the validation proves the required Style pop and
original-mode restoration but exposes the mandatory same-EOF-token reprocess as
an explicit **non-complete** checkpoint. It does not pretend that the bounded
validation model completed later insertion-mode processing that it did not
model.

## Falsification Results

The validation explicitly challenged and rejected the following shortcuts.

### F1 — a completed Data token vector is sufficient

**Falsified.** Tag-shaped RAWTEXT input such as `<b>x` has different lexical
history under Data and RAWTEXT. The selected theorem requires feedback before
that source is produced.

### F2 — tokenizer feedback may be applied late

**Falsified.** Producing the first post-`<style>` input under Data before
applying RAWTEXT observes the wrong token history.

### F3 — downstream reinterpretation or reparsing is equivalent

**Falsified.** Tree/Core cannot repair an already-produced Data-token history by
rescanning source or relabeling Data tokens. Tokenizer lexical ownership and
source progression remain authoritative.

### F4 — one-way control such as PLAINTEXT proves resumability

**Falsified as an equivalence claim.** The selected theorem proves both entry
into RAWTEXT and return to Data with later token production.

### F5 — original insertion mode can be inferred afterward

**Falsified.** The bounded tree state retains original `InHead` across Text and
restores it explicitly.

### F6 — a mid-feedback or inconsistent state is safe to freeze

**Falsified.** Corruption probes reject outstanding feedback, inconsistent
Text/original-mode state, wrong tokenizer/tree restoration, close-evidence
mismatch, impossible RAWTEXT-derived identity, and fabricated EOF close.

### F7 — generic scope or generic implied ends are prerequisites

**Falsified for the closed candidate.** The selected `[html, head, style?]`
state does not require either machinery.

### F8 — browser/WPT/html5lib output may define the oracle

**Falsified by construction.** Expected results are hand-authored from pinned
normative rules; external implementations/corpora may challenge them but do not
generate them.

## Proven Ownership Knowledge

### Coordinator ownership

The Core coordinator owns the ordering between token production and tree
dispatch. It must apply a tree-directed tokenizer-state request before any later
source unit whose interpretation depends on that request is produced.

The coordinator does not own lexical recognition or tree mutation. It owns the
causal sequencing between them.

### Tokenizer ownership

The tokenizer owns:

- source cursor / next-input progression;
- lexical state;
- Data versus RAWTEXT recognition;
- appropriate-end-tag recognition;
- RAWTEXT-to-Data transition;
- emitted token/source evidence; and
- tokenizer diagnostics/completion/resource meaning.

The tree must not scan source to find `</style>` and must not pass a guessed
closing-tag range to the tokenizer.

### Tree-construction ownership

The private tree session owns:

- `InHead` / `Text` insertion-mode meaning;
- retained original insertion mode;
- selected Style creation/open state;
- text insertion;
- Style pop/closure semantics; and
- tree-side diagnostic/recovery meaning.

The tree does not own the tokenizer cursor or private lexical-state enum.

## Proven Provenance and Identity Knowledge

The selected candidate preserves these domains as distinct:

```text
authored <style> start origin
!= authored RAWTEXT character contribution
!= authored appropriate end-tag trigger / close evidence
!= constructed Style identity
!= EOF-triggered recovery
!= final placement
```

The validation establishes:

- exact `SourceId` and start-tag range retention;
- exact ordered RAWTEXT text contribution evidence;
- mixed-case appropriate close matching without erasing raw spelling;
- no constructed `b` identity for tag-shaped RAWTEXT `<b>` bytes;
- no fabricated authored end-tag range at EOF;
- SourceId perturbation changes source identity evidence without changing
  semantic RAWTEXT/tree meaning; and
- deterministic repeated runs.

The existing architecture rule remains: constructed identity is semantic,
result-scoped identity, not source range, token index, storage position,
pointer, browser node identity, or final placement.

## Completion, Negative Space, and Freeze

The validation reinforces that completion, recovery, diagnostics, unsupported
coverage, and resources are orthogonal.

- Lower-layer incompleteness is never upgraded.
- Excluded Style syntax is transactionally refused before candidate tree
  mutation, identity allocation, or feedback application.
- A complete accepted close path cannot freeze with tokenizer still RAWTEXT,
  tree still Text, Style still open, or feedback outstanding.
- Retained close provenance must correspond to the emitted appropriate end-tag
  evidence.
- Tag-shaped RAWTEXT content cannot acquire impossible constructed element
  identity.
- EOF recovery cannot fabricate an authored close.

A green mutable session is not by itself a durable-result oracle; freeze remains
an independent validation boundary.

## Architecture Consequence

The validation verdict is:

```text
Candidate C:                 VALIDATED / UNCHANGED
ADR 0010 amendment:          NO
HTML_TREE_CONSTRUCTION.md:   NO NORMATIVE AMENDMENT
```

The specialized contract already requires Core-owned tokenizer/tree
coordination and explicitly rejects universal reliance on a completed
context-free token vector. It also explicitly permits bounded predecessors to
continue using batch tokenization where their theorem requires no feedback.

The RAWTEXT validation therefore exercises an existing architectural promise
rather than adding a new one.

## TC-S9 Production-Placement Evidence

After validation acceptance, #117 `issuecomment-5434360274` assigned the
sequence designation:

**TC-S9 — Selected In-Head `<style>` RAWTEXT Feedback Lifecycle**.

This designation names only the validated bounded successor. It is not a claim
of production implementation.

The accepted placement is:

**Core-coordinator-owned resumable token production over the existing private
tokenizer Engine, while preserving the existing batch tokenizer as a compatible
bounded entry point.**

Repository evidence motivating that placement is concrete:

- the existing tokenizer already owns a single-forward `Cursor` and private
  `Engine`;
- after committing a context-dependent start tag, the current Engine already
  stops at an exact post-tag `ContextDependentTokenizerMode` boundary before
  consuming later input;
- `<style>` is already classified at that boundary as `RawText`;
- current tree construction already keeps mutable session state private and
  keeps same-token tree redispatch in the Core-owned driver.

The placement therefore selects **generalization of an existing private seam**,
not a second tokenizer or source rescan.

Conceptually:

```text
existing tokenizer Engine
→ emit <style> under Data
→ suspend at existing post-tag context-mode boundary
→ coordinator dispatches <style>
→ tree inserts Style + requests EnterRawText
→ coordinator applies/resumes RAWTEXT
→ tree acknowledges feedback, retains InHead, enters Text
→ later source production resumes
```

The existing batch `tokenize()` contract remains a sibling compatibility path:
without tree coordination, the same context-dependent boundary remains the
existing deferred unsupported result rather than silently enabling tree-driven
RAWTEXT behavior for unrelated consumers.

The placement also retains existing driver-owned same-token redispatch for the
Text-mode EOF reprocess. No second EOF token, tokenizer retry loop, or public
parser-control protocol is selected.

## Production Placement Is Not Production Behavior

At this checkpoint TC-S9 is **not implemented in production**.

The accepted placement does not mean that current production can parse Style
RAWTEXT. Current production tokenizer state remains the bounded Data/tag/
attribute subset, and current tree session/result domains do not yet contain the
production Style/Text successor.

The placement authorizes only a future focused production Issue after this
knowledge checkpoint is reviewed and merged. It does not authorize convenient
scope widening during implementation.

## Strong Claims Not Established

This evidence does **not** establish:

- general RAWTEXT-element support;
- RCDATA or character-reference handling;
- ScriptData or JavaScript execution;
- PLAINTEXT;
- arbitrary `style` attributes/self-closing syntax in the TC-S9 tree theorem;
- general InHead parsing;
- CSS parsing or CSSOM semantics;
- fragment parsing or fragment-context tokenizer initialization;
- foreign-content namespace switching;
- templates, table construction, foster parenting, or active formatting/adoption
  agency;
- generic HTML scope/button-scope machinery;
- a generic implied-end generator;
- parser reentrancy, document.write-style input insertion, or custom-element
  reaction semantics;
- a public tokenizer/session/coordinator API;
- serialization or wire compatibility for parser state;
- a generic async/streaming parser protocol;
- general cancellation/resume semantics;
- universal numeric tree resource limits; or
- a complete HTML parser claim.

A future candidate requiring any of these must independently justify the new
architecture pressure rather than inheriting it from TC-S9.

## Current Evidence Status

```text
first tokenizer/parser/Core vertical slice:      ESTABLISHED
#348 R1-R10 / Wave 1E research:                 COMPLETE
Candidate C / ADR 0010:                         ACCEPTED / UNCHANGED
TC-S1 through TC-S8 production:                 MERGED
current production semantic baseline:           5ffb2eacf0b6cd77b7531a68408cb8e2ceba28b8
RAWTEXT candidate-independent validation:       ACCEPTED / MERGED
current repository main:                        e5c299e3980f6d3de41c6291b86890f19715890d
TC-S9 sequence designation:                     ASSIGNED
TC-S9 production placement:                     ACCEPTED
TC-S9 production Issue:                         BLOCKED PENDING EVIDENCE LEAF
TC-S9 production implementation:                NOT AUTHORIZED
full HTML parser claim:                          NO
```

## Next Gate

The next gate is the focused review and merge of this evidence Leaf.

Only after the evidence checkpoint is accepted on `main` may a dedicated TC-S9
production Issue be created from #117 `issuecomment-5434360274`. That later
Issue must preserve the accepted placement, validation theorem, negative-space
boundaries, and independent production-test discipline.

No production code is authorized by this document.

## Update Rule

Add a new dated evidence record when later TC-S9 production, a later successor,
or material upstream evidence changes the supported theorem, ownership model,
provenance/freeze knowledge, or research status.

Do not edit the historical TC-S7-era checkpoint to make later validation or
production work appear historically complete. Do not edit this record later to
make TC-S9 look production-complete before a separately reviewed production
merge establishes that fact.
