# Architecture Decision Records

## Purpose and Authority

This document is the normative process for Architecture Decision Record (ADR)
mechanics in this repository. It governs when an ADR is required, file naming,
required fields, status transitions, and acceptance, rejection, deprecation,
and supersession mechanics. It applies only to this repository unless another
repository explicitly adopts it.

It does not decide who is a maintainer, whether a proposal is technically
correct, substantive architecture, security exceptions, licensing,
implementation scope, or release approval. Those topics remain owned by the
[Documentation Index](../README.md),
[Maintainership](../governance/MAINTAINERSHIP.md),
[Architecture Principles](../architecture/PRINCIPLES.md),
[Architecture Layers](../architecture/LAYERS.md),
[Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md),
[Secure Development](../development/SECURE_DEVELOPMENT.md),
[Security Policy](../../SECURITY.md), [Contributing](../../.github/CONTRIBUTING.md),
and [MIT License](../../LICENSE), as applicable.

## What an ADR Is

An ADR is a concise, durable record of one significant proposed or approved
architecture decision. It records context, drivers, the decision, realistic
alternatives, consequences, approval, and status so future maintainers,
contributors, and implementation agents can understand the rationale after
implementation details change.

One ADR normally represents one coherent decision. A tightly coupled set may
share one ADR only when separating it would make the rationale misleading or
incomplete.

## What an ADR Is Not

An ADR is not design discussion, a substitute for a focused Issue, automatic
approval, a normative architecture contract by default, a task tracker, a
project plan, an implementation guide, a test report, a changelog, a release
note, an AI-conversation transcript, or a complete history of implementation
details. It is not a place for active vulnerability details and cannot override
the MIT License. It summarizes durable rationale and links to evidence instead
of copying entire discussions.

## When an ADR Is Required

An ADR MUST be created for an approved decision that is significant, durable,
architecture- or compatibility-relevant, and does one or more of the following:

1. establishes or materially changes architecture layers, ownership, lifecycle,
   or dependency direction;
2. establishes or materially changes Core-versus-adapter responsibility;
3. chooses or changes foundational workspace or crate boundaries;
4. establishes or materially changes a stable public API, protocol-neutral
   domain contract, serialized representation, or compatibility commitment;
5. selects or changes a foundational async runtime, concurrency model,
   orchestration model, process boundary, IPC model, or cross-cutting runtime;
6. selects a foundational parser, protocol, storage, or infrastructure approach
   whose replacement has material architecture or compatibility cost;
7. introduces an approved repository-authored `unsafe Rust` exception or
   changes its safety boundary;
8. establishes or materially changes a browser-adapter boundary or shared
   adapter normalization strategy;
9. intentionally supersedes an accepted ADR;
10. creates a durable exception to an architecture principle or Rust Core
    default; or
11. resolves a long-lived architecture ambiguity future work would otherwise
    repeatedly revisit.

A trigger does not approve a decision or replace prior approval requirements.
Implementation must not begin before required approval. If a decision changes
a normative contract, that contract must also be updated through approved
scope.

## When an ADR Is Not Required

An ADR is not required for:

- typo, spelling, link, or formatting corrections;
- renaming a private helper without contract impact;
- local refactoring that preserves ownership, behavior, boundaries, and
  compatibility;
- routine implementation, tests, or documentation required by an already
  approved ADR or normative contract;
- a focused bug fix that preserves established contracts;
- reversible local algorithms with no public, serialized, security,
  architecture, or compatibility impact;
- dependency patch updates within an approved dependency contract that create
  no material risk or boundary change;
- task scope, validation logs, or completion reports; or
- experiments outside production that create no contract.

Significance is evaluated by impact, not line count: a small diff can require
an ADR, while a large mechanical diff may not. When uncertain, stop
implementation at the affected boundary and request maintainer classification.

## Decision Records Without an ADR

When an ADR is not required, durable rationale may remain in a focused Issue,
Issue comment, Pull Request description, Pull Request review, approved
documentation, or validation or completion record. The record must still
satisfy Maintainership requirements. No ADR does not mean no rationale.

## Relationship to Issues and Pull Requests

The preferred workflow is to:

1. open or use a focused Issue for a non-trivial proposal;
2. identify relevant contracts and whether an ADR trigger applies;
3. collect evidence, constraints, alternatives, and impact;
4. request explicit maintainer approval;
5. create or finalize the ADR when required;
6. update affected normative documents;
7. implement through focused scope;
8. validate;
9. merge; and
10. record follow-up.

### ADR before implementation PR

This is preferred when the decision should be reviewed independently from its
implementation.

### ADR and normative contract update in one PR

This is allowed after a focused Issue contains sufficient evidence and explicit
approval, or when explicit maintainer PR review provides the required
pre-implementation approval.

### ADR with implementation in one PR

This is allowed only when review remains understandable, implementation has not
begun before required approval, decision and implementation form one coherent
review scope, the maintainer explicitly approves the decision before merge,
affected normative documents are updated, and security and compatibility review
remain proportionate. Do not combine a broad architecture decision with large
implementation. PR merge alone is not implicit decision approval; approval
must be explicit and attributable.

## Relationship to Normative Contracts

An ADR records rationale and status; a normative contract defines the active
repository rule. An ADR cannot silently override a normative contract. An
accepted ADR that changes a contract must update its authoritative document.
If they appear inconsistent, stop implementation and use the conflict process
in the [Documentation Index](../README.md); recency alone does not resolve the
conflict. An ADR may explain why a contract changed, but that contract remains
the operational source of truth for its domain. An ADR may explicitly conclude,
with rationale, that no normative-document update is required.

## File Naming and Numbering

Actual ADRs use `NNNN-short-kebab-title.md`, for example the format
`0001-example-decision-title.md` or `0042-another-decision-title.md`.

- `NNNN` is a four-digit decimal number, starting at `0001`.
- Select the next unused repository number; numbers are never reused.
- Numbers identify records, not priority, authority, or current status.
- The concise, decision-oriented title uses lowercase kebab-case.
- Renaming an accepted ADR is discouraged because links may depend on it.
- If concurrent proposals select the same number, the later branch renames
  before merge.
- `README.md` and `TEMPLATE.md` are not numbered ADRs.

No automated registry or numbering bot is required.

## Required ADR Structure

Every ADR must contain all fields represented by the copyable
[ADR Template](TEMPLATE.md). Required content is:

- title formatted `# ADR NNNN: Decision title`;
- metadata for Status, Date, Decision owner / approver, Linked Issue, Related
  Pull Request, Supersedes, Superseded by, and Affected normative contracts
  (`None` is allowed where applicable);
- Context covering the problem, current state, constraints, invariants,
  drivers, and relevant evidence;
- a concise Decision and its boundary;
- realistic Alternatives Considered, each with benefits, costs, and rejection
  or non-selection rationale (obviously impossible alternatives are omitted);
- Consequences covering positive and negative effects, risks, operational or
  maintenance cost, and reversibility;
- Compatibility and Migration addressing Rust API, serialization, protocol,
  semantic meaning, ordering or determinism, product or adapter impact, and
  migration where applicable (`Not applicable` requires an explanation);
- Security and License Impact, linking to the owning contracts when relevant
  and excluding active vulnerability details;
- Validation evidence needed to demonstrate correct implementation and
  boundaries;
- focused Follow-Up work or `None`;
- Approval containing explicit maintainer approval, identity, date, and a
  durable approval link; and
- References to relevant Issues, PRs, benchmarks, research, contracts, and
  earlier ADRs. Private AI links and full prompt transcripts are not durable
  references.

An `Accepted` status without valid approval evidence is invalid.

## Status Lifecycle

Numbered ADRs use exactly these statuses:

- **Proposed:** under review, not approved, does not authorize implementation,
  links to a focused Issue, and may be revised during discussion.
- **Accepted:** explicitly approved by a valid maintainer, records approval
  evidence, and is active rationale; affected normative contracts are updated
  or the ADR explains why none needs an update.
- **Rejected:** considered but not adopted, does not authorize implementation,
  records useful rejection rationale, and remains discoverable. Reconsideration
  normally uses a new ADR or clearly documented new proposal rather than a
  silent change to Accepted.
- **Deprecated:** no longer recommended or generally applicable, but may remain
  relevant during compatibility, migration, or for history. It needs no
  replacement, explains effective scope and migration or removal consequences,
  and remains discoverable.
- **Superseded:** replaced by another accepted ADR, links to that replacement,
  is linked back from it, remains discoverable, and is not deleted.

Allowed transitions are `Proposed` → `Accepted`, `Proposed` → `Rejected`,
`Accepted` → `Deprecated`, `Accepted` → `Superseded`, and `Deprecated` →
`Superseded` when a replacement is later accepted. Other transitions require a
new proposal and explicit maintainer review. Never silently reactivate a
Rejected, Deprecated, or Superseded ADR.

## Creating and Accepting an ADR

1. Determine that a trigger applies.
2. Select the next unused number.
3. Copy `TEMPLATE.md` and remove its guidance comments.
4. Create the ADR as `Proposed` and link its focused Issue.
5. Complete every required section and identify affected contracts.
6. Collect review and explicit approval.
7. Change to `Accepted` only after valid, explicit approval exists.
8. Update affected normative documents.
9. Merge the ADR and contract updates before or with approved implementation.
10. Add the ADR to the index below.

Because this repository is currently solo-maintained, `YT-TechDev` may author
and approve an ADR as maintainer of record. Approval must still be explicit,
attributable, decision-specific, and durable. Self-authorship removes none of
the evidence, alternative, or impact requirements. Implementation-agent text
is not maintainer approval. No second approver or quorum is required.

## Editing an Accepted ADR

Without a new ADR, an accepted record may receive typo, broken-link, or
formatting corrections; clarification that does not change the decision,
scope, consequences, or compatibility meaning; and references to completed
implementation or validation.

A new ADR is required to change the decision, ownership, scope, invariants,
compatibility commitment, safety boundary, dependency direction, or material
consequences. Do not rewrite historical rationale to make the past decision
appear different. Use an explanatory note and linked follow-up for significant
corrections.

## Rejecting a Proposal

A Proposed ADR may become Rejected when the maintainer explicitly rejects it,
another alternative is selected, evidence shows it is unnecessary or invalid,
or project scope changes before acceptance. Record the reason and durable
rejection link. It does not become a normative contract. Preserve a reviewed
rejected ADR when its rationale remains useful; a trivial abandoned draft that
never received meaningful review need not be merged.

## Deprecating a Decision

Deprecation requires the reason the decision is no longer recommended, affected
scope, compatibility and migration consequences, current replacement guidance
if any, explicit maintainer approval, and normative contract updates where
applicable. Deprecation is not silent deletion.

## Superseding a Decision

1. Create a new Proposed ADR with a new number.
2. Identify the existing Accepted or Deprecated ADR and why it is insufficient.
3. Document evidence, alternatives, compatibility, migration, security, and
   maintenance impact.
4. Obtain explicit maintainer approval.
5. Mark the new ADR Accepted and the old ADR Superseded.
6. Add bidirectional links and list every replaced ADR.
7. Update every affected normative contract.
8. Implement and validate through focused scope.

Never overwrite the old ADR with its replacement decision.

## Security, Licensing, and Sensitive Information

ADRs must comply with [Secure Development](../development/SECURE_DEVELOPMENT.md).
Active vulnerability details use the private [Security Policy](../../SECURITY.md)
process. A public ADR may summarize a security decision only after disclosure
risk is resolved and only to a safe extent. Do not add credentials, private
source data, personal information, private incident details, or exploit
instructions.

An ADR cannot approve an unsafe exception without the focused security Issue
and required evidence. It cannot override or reinterpret the
[MIT License](../../LICENSE). Dependency and protocol decisions record material
license impact when relevant.

## Ownership and Approval

Anyone may propose an ADR, and contributors or agents may research, draft, and
update it within scope. Only a maintainer with relevant authority may approve
it. [Maintainership](../governance/MAINTAINERSHIP.md) determines current
authority; `YT-TechDev` is currently the maintainer of record. Agents cannot
approve ADRs, and status text alone is not approval.

Approval must be explicit, attributable, decision-specific, durable, and
recorded before implementation when prior approval is required. A maintainer
may reject, deprecate, or supersede only with durable rationale. Future
repository- or domain-scoped maintainers may approve only within documented
authority.

## Discoverability

Use [TEMPLATE.md](TEMPLATE.md) to create an ADR. When an ADR is added or changes
status, update this table in the same focused change. Keep all Rejected,
Deprecated, and Superseded links and sort by ADR number ascending. Table order
does not confer authority.

| ADR | Title | Status | Date | Replaces |
| --- | --- | --- | --- | --- |
| [0001](0001-repository-topology-and-workspace-ownership.md) | Repository topology and workspace ownership | Accepted | 2026-07-30 | None |
| [0002](0002-rust-bootstrap-toolchain-and-validation-policy.md) | Rust bootstrap toolchain and validation policy | Accepted | 2026-07-30 | None |
| [0003](0003-validated-source-anchors-first-rust-core-domain.md) | Establish Validated Source Anchors as the first Rust Core domain | Accepted | 2026-07-31 | None |
| [0004](0004-validated-source-anchor-semantics.md) | Define Validated Source Anchor Semantics | Accepted | 2026-07-31 | None |
| [0005](0005-raw-source-coordinate-semantics.md) | Raw Source Coordinate Semantics | Accepted | 2026-07-31 | None |
| [0006](0006-qualify-first-source-anchored-analysis-vertical-slice.md) | Qualify the First Source-Anchored Analysis Vertical Slice | Accepted | 2026-07-31 | None |
| [0007](0007-own-lossless-source-parsers.md) | Own Lossless Source Parsers | Accepted | 2026-08-06 | None |

## Representative Classifications

| Scenario | ADR required | Required durable record | Approval location | Normative document update | Implementation may proceed |
| --- | --- | --- | --- | --- | --- |
| Rename a private helper without contract impact | No | Focused PR normally suffices | Maintainer review before merge | None when behavior and boundaries remain unchanged | Yes, as routine scoped work |
| Choose initial crate boundaries | Yes | Focused architecture Issue and ADR | Explicit maintainer approval before implementation | Relevant architecture and future workspace contracts | No; implementation waits |
| Expose a stable public protocol model | Yes | Focused public-contract Issue and ADR | Explicit maintainer approval before implementation | Public API, protocol-neutral domain, compatibility, and architecture contracts as applicable | No; implementation waits |
| Choose a foundational async runtime | Yes | Focused architecture and dependency Issue and ADR | Explicit maintainer approval before introduction | Architecture, Rust Core, dependency, and compatibility contracts as applicable | No; implementation waits |
| Correct a typo | No | Focused PR | Ordinary maintainer review | No update beyond the correction | Yes |
| Permit a contained repository-authored `unsafe` implementation | Yes | Focused security Issue, ADR, and all Rust Core unsafe evidence | Explicit maintainer approval before implementation | Secure Development or Rust Core only if its rule changes; otherwise link both | No; implementation waits and active vulnerability details remain private |
| Replace an accepted browser-adapter boundary | Yes; a new ADR supersedes the old ADR | Focused architecture Issue and new ADR; retain the old ADR | Explicit maintainer approval before implementation | `LAYERS.md`, `PRINCIPLES.md`, compatibility, and other affected contracts | No; implementation waits |
| Implement a parser algorithm already allowed by accepted contracts | No, unless it creates a foundational dependency or public compatibility commitment | Issue or PR evidence normally suffices | Maintainer Issue or PR review appropriate to scope | None unless an existing contract changes | Yes within accepted contracts; otherwise stop |
| Change Analysis Result semantic meaning | Yes | Focused domain or result-contract Issue and ADR | Explicit maintainer approval before implementation | Result/domain contracts and compatibility impact | No; implementation waits |
| Update a dependency patch version with no contract change | No | Dependency review and PR evidence | Maintainer PR review; escalate if security, compatibility, or public exposure changes | None when contracts remain unchanged | Yes after proportionate dependency review |

The async-runtime classification concerns a foundational Core, orchestration,
or project runtime. A strictly local adapter implementation detail with no
durable boundary, public, compatibility, or cross-cutting impact may use an
Issue or PR record instead. These classifications select no architecture and
do not approve `unsafe Rust`.

## Deferred Decisions

This process creates no ADR and makes no decision about crate boundaries,
public APIs, protocol models, async runtime, concurrency model, parser library,
storage, orchestration, browser-adapter boundary changes, serialization,
unsafe implementation, or any other production architecture choice. Each
remains deferred to a focused, approved proposal.
