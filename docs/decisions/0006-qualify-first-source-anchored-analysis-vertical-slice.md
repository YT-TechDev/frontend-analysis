# ADR 0006: Qualify the First Source-Anchored Analysis Vertical Slice

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-07-31 |
| Decision owner / approver | `YT-TechDev` |
| Linked Issue | [#82](https://github.com/YT-TechDev/frontend-analysis/issues/82) |
| Related Pull Request | [#84](https://github.com/YT-TechDev/frontend-analysis/pull/84) |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | None — this record defers production expansion within the existing architecture, Rust Core, security, and validation contracts and changes none of them. |

## Context

Accepted [ADR 0003](0003-validated-source-anchors-first-rust-core-domain.md),
[ADR 0004](0004-validated-source-anchor-semantics.md), and
[ADR 0005](0005-raw-source-coordinate-semantics.md) established the current
browser-independent Rust Core: exact retained UTF-8 source text,
caller-supplied `SourceId` provenance, validated half-open UTF-8 byte ranges,
retained `SourceAnchor` evidence, and raw zero-based line and UTF-8
byte-column projections of validated anchor endpoints.

That Core owns source evidence. It does not yet own an analysis. Introducing a
parser, an analysis input contract, an Analysis Result model, a diagnostic or
evidence taxonomy, or a product consumer before a concrete analysis question
and consumer are known would reserve broad names and compatibility semantics
prematurely. Milestone #5 was therefore executed as a non-versioned
architecture gate that adds no production Rust surface.

The decision problem is: does exactly one concrete, browser-independent,
source-anchored analysis vertical slice satisfy the repository's
production-readiness gates? If so, which consumer, question, ownership
boundary, provenance contract, and validation envelope qualify it for a
separately planned production milestone? If not, which evidence is missing and
what objective triggers permit reconsideration?

Three research Leaves produced the evidence, each independently reviewed.

[Issue #79](https://github.com/YT-TechDev/frontend-analysis/issues/79)
recorded `NO CANDIDATE IDENTIFIED`. It compared serious HTML, CSS,
ECMAScript, and no-production alternatives and found that no durable
repository record names a concrete consumer surface, an input-acquisition
path, a workflow step requesting an analysis answer, a decision or action
changed by that answer, or a downstream commitment or current integration; and
that available evidence cannot uniquely select one grammar family and one
analysis question. The research also found that existing Core semantics are
sufficient as the source-evidence foundation, so the v0.4.0 contracts are not
the blocker.

[Issue #80](https://github.com/YT-TechDev/frontend-analysis/issues/80)
recorded `NOT APPLICABLE — NO BOUNDED CANDIDATE`. Its hard dependency was
satisfied procedurally, but its substantive target did not exist: without one
selected grammar, one selected question, the exact structures requiring spans,
and the material recovery cases, a parser capability matrix cannot be scoped
honestly. No parser was inspected, selected, ranked, approved, or rejected.
This result is not `BOUNDARY NOT QUALIFIED` and makes no negative claim about
any parser.

[Issue #81](https://github.com/YT-TechDev/frontend-analysis/issues/81)
recorded `DEFERRED — NO CANDIDATE-SPECIFIC VALIDATION ENVELOPE`. Validation of
the architecture-gate deferral package is bounded and feasible now, but
candidate-specific production validation is blocked: threat classes, resource
limits, fixture families, deterministic ordering, and failure semantics are
each defined relative to an input shape, a result meaning, and a grammar that
do not exist. Reusable qualification gates were recorded as future observable
properties only, and
[Secure Development](../development/SECURE_DEVELOPMENT.md) and
[Validation and Completion Evidence](../development/VALIDATION.md) remain the
normative owners.

These findings form one causal chain rather than three separate conclusions.
The absence of a consumer and a uniquely selected question removes the target
a parser-provenance audit requires, which removes the input shape, result
meaning, and failure model a validation envelope requires. The chain begins at
the consumer, not at the Core contracts and not at any parser.

Two constraints from the independent reviews bound this record. First,
`publish = false` prevents Cargo registry publication but does not prevent Git
or path dependency consumption, and repository popularity signals are not
proof of consumer absence; neither is used as a load-bearing argument here.
Second, the grammar preprocessing observations collected during research are
dated deferred context about specifications, not evidence about any parser's
source-span capability.

The decision drivers are the durable architecture constraints already in
force: browser independence, explicit ownership, boundary-driven abstraction
that is not justified by anticipated reuse, narrowest useful visibility, and
the rule that deferred does not mean unrestricted.

## Decision

No source-anchored analysis vertical slice is qualified at this time.
Production Rust Core domain expansion remains deferred.

The decision package is:

1. **No slice is qualified.** No candidate satisfied the consumer and
   selection gates, so none proceeds to production milestone planning.
2. **Production expansion is deferred.** The Rust Core gains no new domain,
   and Milestone #5 closes as an architecture gate without a versioned
   production change.
3. **Nothing is selected or authorized.** This record selects no grammar
   family, analysis question, parser, dependency, result model, error model,
   crate boundary, module, public API, serialization format, source-map
   capability, Browser Adapter contract, product surface, package version, or
   release work.
4. **Existing contracts are unchanged and are not the blocker.** The ADR 0004
   `SourceAnchor` and ADR 0005 `RawSourceCoordinate` semantics remain exactly
   as accepted. Research found them sufficient as the source-evidence
   foundation; this deferral is not a criticism of them and does not
   reinterpret them.
5. **The blocker is identified precisely.** It is the absence of a concrete
   consumer and a uniquely selected analysis question, and consequently the
   absence of a bounded parser-provenance target and a candidate-specific
   validation envelope.
6. **Reconsideration requires objective durable evidence.** It must not be
   inferred from roadmap language, milestone titles, generic product intent,
   or layer-boundary documentation.
7. **Satisfying the triggers authorizes planning only.** It permits a new
   architecture-first production milestone planning execution. It does not
   automatically authorize implementation.

This is an explicit deferral, not a rejection of HTML, CSS, ECMAScript, or any
parser, and not a finding that source-anchored analysis is infeasible. All
three grammar families may contain valid future questions.

### Objective reconsideration gate

Candidate-specific production planning may be reconsidered only when all ten
items exist as durable repository evidence:

1. one concrete consumer surface;
2. one input-acquisition path;
3. one workflow step at which the answer is requested;
4. one decision or action changed by the answer;
5. exactly one analysis question, expressible in one sentence;
6. exactly one grammar family;
7. one bounded parser and raw-source provenance target;
8. minimum result and failure semantics, including successful zero-result
   versus indeterminacy;
9. measurable resource and validation targets based on measurement or
   normative evidence; and
10. lawful, minimal fixture provenance and licensing feasibility.

A future narrow candidate may make some validation categories `Not
applicable`. The gate requires these questions to be resolved, not every
possible mechanism to be built.

A maintainer-approved durable selection may satisfy the selection elements
when it is explicit, decision-specific, and attributable.

## Alternatives Considered

### 1. Qualify one HTML source-anchored analysis slice now

**Benefits:** HTML is central to frontend analysis; several well-formed
questions exist, such as locating start tags missing a required attribute or
duplicate identifier values, and each would genuinely require source-anchored
evidence.

**Costs:** No consumer or decision distinguishes one HTML question from
equally plausible alternatives. HTML normalizes newlines before tokenization,
so a conforming parser's native positions are positions in a transformed
stream; without a selected question the required structures, material recovery
cases, and acceptable provenance loss cannot be stated, leaving the provenance
audit unbounded.

**Not selected because:** the consumer and selection gates fail, not because
HTML is unsuitable. The preprocessing observation is a specification fact
requiring later parser investigation, not evidence that any HTML parser is
inadequate.

### 2. Qualify one CSS source-anchored analysis slice now

**Benefits:** CSS questions can be narrow and bounded, such as locating
declarations carrying a priority flag, and are answerable from one stylesheet
without runtime state.

**Costs:** Identical selection failure. CSS preprocessing replaces carriage
return, form feed, and CRLF with a single line feed and replaces null and
surrogate code points, so raw-offset correspondence requires proof against a
selected parser and question.

**Not selected because:** no consumer decision selects a CSS question over the
alternatives, and the downstream provenance target is consequently unbounded.

### 3. Qualify one ECMAScript source-anchored analysis slice now

**Benefits:** Well-formed questions exist, such as locating specific statement
forms, and ECMAScript does not mandate the same newline-replacement
preprocessing described for the HTML and CSS alternatives.

**Costs:** ECMA-262 defines source text as a sequence of Unicode code points and
does not define UTF-8 byte-position semantics. A future parser may expose a
position unit that requires explicit conversion and provenance validation
before it can produce authoritative Core raw byte offsets. ECMAScript also
recognizes line separator and paragraph separator as line terminators in
addition to line feed and carriage return, while the accepted Core raw newline
rule deliberately treats them as ordinary content; derived line indexes
therefore diverge for sources containing them. More decisively, the same
consumer and selection failure applies, and a full syntactic grammar widens
rather than narrows the provenance target.

**Not selected because:** no consumer decision selects one ECMAScript question.
The encoding-unit and line-terminator differences are deferred provenance
questions for a later bounded parser audit, not defects in ADR 0005 and not
evidence that any parser is inadequate.

### 4. Create a parser-neutral request boundary first

**Benefits:** Would appear to unblock later work by fixing an input shape
early, and would give future parsers a stable target.

**Costs:** An input contract designed without a known analysis question would
encode guessed requirements. It would reserve broad naming and compatibility
semantics before any consumer exists, contradicting boundary-driven
abstraction, which does not justify an abstraction merely because a future
feature might use it.

**Not selected because:** it inverts the required order by fixing a boundary
before the demonstrated variation point exists.

### 5. Create a generic Analysis Result, Finding, or Diagnostic foundation first

**Benefits:** Would provide vocabulary for future analyses and appear to make
progress on the result contract layer.

**Costs:** Result meaning follows the qualified analysis question and cannot
be designed generically in advance. Introducing such a taxonomy now would
reserve the broadest names in the project, commit to certainty and evidence
semantics with no analysis to validate them against, and create compatibility
obligations for contracts nothing consumes.

**Not selected because:** it is precisely the premature-reservation risk that
Milestone #5 exists to prevent.

### 6. Perform a broad parser survey first

**Benefits:** Would accumulate capability, licence, and maintenance evidence
that a future slice will eventually need.

**Costs:** Without a selected grammar and question there is no acceptance
criterion, so the survey's conclusions would be unfalsifiable. Parser
capability evidence is perishable, so it would likely be stale by the time a
candidate is selected, while creating anchoring pressure toward whichever
parser looked best in an untargeted comparison. It also risks materially
pre-selecting a dependency without the required review.

**Not selected because:** it would convert a bounded gate into an unbounded
survey and would reduce, not improve, the quality of the eventual decision.

### 7. Preserve the current production boundary and defer the slice

**Benefits:** Keeps the accepted v0.4.0 contracts stable and validated;
reserves no broad names; adds no production Rust surface, dependency, or
compatibility obligation; records precisely which evidence is missing; and
preserves every future option, including all three grammar families.

**Costs:** Milestone #5 produces no production capability, the project's first
analysis remains unscheduled, and the collected deferred context will require
re-verification when a candidate appears.

**Selected because:** it is the only alternative consistent with the evidence.
It records an honest, specific deferral with objective reconsideration
triggers rather than manufacturing a candidate to preserve milestone momentum.

## Consequences

### Positive

- The accepted v0.4.0 source-evidence contracts remain stable, validated, and
  unburdened by speculative dependents.
- No broad generic vocabulary is reserved, so the eventual result and input
  contracts can be shaped by a real analysis question.
- No parser, dependency, or compatibility obligation is incurred.
- The blocker is stated precisely enough to be actionable: name a consumer and
  a decision, and the chain unblocks in order.
- All three grammar families remain fully available; none is disqualified.
- The architecture-gate records provide reusable qualification gates that any
  future slice can be measured against.

### Negative

- Milestone #5 delivers no production capability, and the first analysis
  domain remains unscheduled.
- The repository retains source-evidence primitives with no in-repository
  consumer, which may appear incomplete to an outside reader.
- Research effort produced deferred context rather than an implementable
  design.
- Dated specification observations will require re-verification later.

### Risks

| Risk | Mitigation |
| --- | --- |
| `Accepted` status is misread as authorizing production work | The Decision, Approval, and Validation sections state explicitly that acceptance approves the deferral decision only |
| Deferral is misread as rejecting a grammar or a parser | The record states repeatedly that no parser was evaluated and no grammar is disqualified |
| Reconsideration triggers are satisfied by inference from roadmap language | The gate requires durable, explicit, attributable evidence and forbids inference |
| Pressure to qualify a fallback slice to restore momentum | The gate is objective and enumerated; momentum is not evidence |
| Deferred context becomes stale and is reused as current fact | The context is dated and marked deferred; re-verification is required when a grammar is selected |
| The absence of a consumer persists indefinitely | Accepted as an honest project state; the ADR makes the missing evidence explicit rather than hiding it |

### Reversibility

This decision is inexpensive to reverse because it creates nothing to undo. No
production code, dependency, public API, or compatibility promise is
introduced, so reversal requires no migration, deprecation, or removal.

Exit conditions are the ten reconsideration gate items. When they are
satisfied as durable evidence, a new architecture-first production milestone
proposal may be planned; that proposal independently defines its crate or
module boundary, public API, error model, dependency policy, version change,
tests, documentation, Pull Request sequence, and final audit. Reversal
therefore costs a new focused proposal and its approval, not a rewrite.

## Compatibility and Migration

This decision changes no current contract. Specifically, there is no change
to:

- the Rust crate-root public API, which remains exactly `RawSourceCoordinate`,
  `SourceAnchor`, `SourceId`, `SourceRange`, `SourceRangeError`, and
  `SourceText`;
- existing source-anchor or raw-coordinate semantics, including validation
  precedence, half-open UTF-8 byte ranges, valid empty ranges, exact source
  preservation, caller-supplied provenance, and raw newline handling;
- serialized representations, of which none exist;
- browser protocols;
- product or adapter behaviour, of which none exists;
- package version `0.4.0`, which remains private with `publish = false`;
- `Cargo.lock`;
- package, workspace-member, dependency, feature, or target counts, which
  remain one package, one member, zero dependencies, zero features, and one
  library target;
- Rust toolchain policy, which remains the pinned `1.97.1` without an MSRV
  promise;
- MSRV, ABI, external SemVer, publication, or release promises, of which none
  are made.

Ordering and determinism semantics are unchanged because no analysis output
exists. No migration is required because no production contract changes.

## Security and License Impact

This is a documentation-only proposal. It adopts no parser, dependency,
fixture corpus, copied specification content, external service, or new
execution mechanism, and it triggers no dependency or licence review.

Future parser or dependency adoption requires its own proportional
supply-chain, maintenance, security history, licence-compatibility, public-API
exposure, and replacement-cost review under
[Secure Development](../development/SECURE_DEVELOPMENT.md). Untrusted input
handling, source-content safety, honest failure over security-relevant partial
success, resource limits, deterministic validation, fixture provenance, and
private vulnerability handling all remain governed by
[Secure Development](../development/SECURE_DEVELOPMENT.md),
[Validation and Completion Evidence](../development/VALIDATION.md), and the
[Security Policy](../../SECURITY.md).

No active vulnerability details and no source payloads belong in this ADR. The
architecture-gate research produced no security-sensitive finding requiring
private handling.

## Validation

The checks below are required for the future Proposed ADR Pull Request and the
later status-recording Pull Request. None of them is claimed to have passed
yet.

```bash
python3 .github/scripts/validate-rust-workspace-state.py .
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --doc --workspace --all-features
cargo metadata --offline --format-version 1 --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
git diff --check
git diff -- Cargo.lock
git status --short
```

Additionally required for each Pull Request:

- complete changed-file and final-diff review;
- ADR structure review covering every required metadata field and section;
- relative-link resolution for every link in the ADR and index entry;
- decision-index consistency between the ADR status and its table row;
- approval-evidence review, including that `Accepted` is never set without
  explicit, attributable, durable approval;
- package version `0.4.0` unchanged;
- `Cargo.lock` byte-identical;
- exactly one package, one workspace member, zero dependencies, zero features,
  and one library target;
- current crate-root public API inventory unchanged;
- successful live CI with complete job-log inspection rather than run
  summaries alone;
- the final independent audit under
  [Issue #83](https://github.com/YT-TechDev/frontend-analysis/issues/83).

A disposable external-consumer check is `Not applicable` for this
architecture-gate milestone because no public API changes.

The current test count is not a compatibility promise and is not fixed by this
ADR. It may be recorded later as execution evidence only.

## Follow-Up

- Complete the independent final architecture-readiness audit under
  [Issue #83](https://github.com/YT-TechDev/frontend-analysis/issues/83).
- Close Parent
  [Issue #78](https://github.com/YT-TechDev/frontend-analysis/issues/78) and
  Milestone #5 once the audit records its result.
- Take no production action. If and when the ten reconsideration gate items
  are satisfied as durable evidence, open a focused proposal for a new
  architecture-first production milestone planning execution. That proposal,
  not this ADR, defines any future crate boundary, public API, error model,
  dependency policy, version change, tests, documentation, and audit.

## Approval

**Accepted by `YT-TechDev` on 2026-07-31.**

The repository owner explicitly approved the ADR 0006 deferral decision in the
[repository-owner decision on Issue #82](https://github.com/YT-TechDev/frontend-analysis/issues/82#issuecomment-5144134648).

`Accepted` approves `PRODUCTION WORK DEFERRED — NO QUALIFIED ANALYSIS SLICE`,
the seven-item decision package, and the ten-item objective reconsideration
gate recorded by this ADR.

This acceptance approves the decision to defer. It does not approve or
authorize production implementation, a production milestone, a grammar
family, an analysis question, a parser, a dependency, a result model, an error
model, a crate, a module, a public API, serialization, source-map capability, a
Browser Adapter contract, a product surface, a package-version change,
publication, or release work.

This accepted deferral authorizes only a future architecture-first production
milestone planning execution after the objective reconsideration gate is
satisfied as durable evidence. It does not automatically authorize
implementation.

## References

- [Issue #78: coordination parent](https://github.com/YT-TechDev/frontend-analysis/issues/78)
- [Issue #79: consumer and analysis-question research](https://github.com/YT-TechDev/frontend-analysis/issues/79)
- [Issue #79 research record](https://github.com/YT-TechDev/frontend-analysis/issues/79#issuecomment-5142769010)
- [Issue #79 independent review](https://github.com/YT-TechDev/frontend-analysis/issues/79#issuecomment-5142802510)
- [Issue #80: grammar, parser, and raw-source position research](https://github.com/YT-TechDev/frontend-analysis/issues/80)
- [Issue #80 research record](https://github.com/YT-TechDev/frontend-analysis/issues/80#issuecomment-5142860419)
- [Issue #80 independent review](https://github.com/YT-TechDev/frontend-analysis/issues/80#issuecomment-5142876839)
- [Issue #81: untrusted-input and validation research](https://github.com/YT-TechDev/frontend-analysis/issues/81)
- [Issue #81 research record](https://github.com/YT-TechDev/frontend-analysis/issues/81#issuecomment-5143216527)
- [Issue #81 independent review](https://github.com/YT-TechDev/frontend-analysis/issues/81#issuecomment-5143332365)
- [Issue #82: this decision](https://github.com/YT-TechDev/frontend-analysis/issues/82)
- [Issue #83: independent final audit](https://github.com/YT-TechDev/frontend-analysis/issues/83)
- [ADR 0003: Validated Source Anchors as the first Rust Core domain](0003-validated-source-anchors-first-rust-core-domain.md)
- [ADR 0004: Validated Source Anchor Semantics](0004-validated-source-anchor-semantics.md)
- [ADR 0005: Raw Source Coordinate Semantics](0005-raw-source-coordinate-semantics.md)
- [Architecture Principles](../architecture/PRINCIPLES.md)
- [Architecture Layers and Boundaries](../architecture/LAYERS.md)
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md)
- [Validated Source Anchors Guide](../architecture/VALIDATED_SOURCE_ANCHORS.md)
- [Raw Source Coordinates Guide](../architecture/RAW_SOURCE_COORDINATES.md)
- [Validation and Completion Evidence](../development/VALIDATION.md)
- [Secure Development](../development/SECURE_DEVELOPMENT.md)
- [Maintainership and Decision Authority](../governance/MAINTAINERSHIP.md)
- [Architecture Decision Record Process](README.md)
- [ADR 0006 repository-owner approval](https://github.com/YT-TechDev/frontend-analysis/issues/82#issuecomment-5144134648)
