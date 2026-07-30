# Implementation Agent Workflow

## Purpose and Authority

This document is the normative detailed workflow for implementation agents in
this repository. It expands the concise [Shared Agent Contract](../../AGENTS.md)
without replacing the active Leaf Issue or explicit task, maintainer approval,
specialized normative contracts, [Contributing](../../.github/CONTRIBUTING.md),
the [Claude Code Supplement](../../CLAUDE.md), or [ADR Process](../decisions/README.md)
mechanics.

The active Leaf Issue or explicit task defines the requested result and
implementation scope. Specialized documents continue to own architecture,
security, Rust, governance, compatibility, and contribution rules. This
workflow determines how an agent executes within those contracts; compliance
with it is not approval for a technical decision. The [Documentation
Index](../README.md) owns documentation classification and source precedence,
and [Maintainership](../governance/MAINTAINERSHIP.md) owns approval authority.

## Normative Language

- **MUST** and **MUST NOT** identify mandatory workflow requirements.
- **SHOULD** and **SHOULD NOT** identify the default expectation; a deviation
  requires a recorded reason.
- **MAY** identifies an option permitted only while active scope and every
  applicable specialized contract remain satisfied.

These terms create workflow requirements, not technical architecture.

## Workflow Outcomes

These are the only primary outcomes.

### Proceed

Use **Proceed** when the repository and active task are resolved, result and
scope are clear, applicable contracts are available and consistent, required
approval already exists, implementation stays within the approved boundary,
and proportionate validation is possible. Proceed does not authorize
unrequested repository side effects.

### Stop and Escalate

Use **Stop and Escalate** when an approval boundary is unresolved,
authoritative contracts conflict, ownership or architecture is undefined,
implementation needs unauthorized scope expansion, validation suggests the
approved design is invalid, security or vulnerability handling must be private,
or a repository side effect is unauthorized. Stop at the affected boundary,
preserve evidence, and request maintainer resolution.

### Partial

Use **Partial** only when a separable subset of approved scope is safely
complete, the incomplete portion and reason are explicit, the completed subset
does not imply that the Issue is finished, and no unresolved contract is
hidden. Partial work MUST NOT automatically close the Issue.

### Blocked

Use **Blocked** when execution cannot safely continue because required
authority or a required source is unavailable, an environment or permission is
inaccessible, a contract conflict remains unresolved, or required validation
has no approved substitute. Blocked is a completion-report result, not
permission to invent a workaround.

## Required Inputs

| Input | Required content | Resolution rule |
| --- | --- | --- |
| Repository | Exact repository and default branch | Resolve from the task or live checkout. |
| Active work item | Focused Leaf Issue or explicit task | It MUST identify desired result and scope. |
| Base and working branch | Target base and current branch | Inspect live Git state when available. |
| Desired result | Observable or contractual outcome | Do not substitute implementation steps. |
| In scope | Files, modules, behavior, or responsibility | Use the smallest coherent ownership boundary. |
| Out of scope | Adjacent work intentionally excluded | Use it to prevent opportunistic expansion. |
| Invariants | Architecture, ownership, security, compatibility, and behavior that remain true | Select the owning specialized contracts. |
| Approval state | Required maintainer approvals | Approval MUST be explicit and durable. |
| Validation | Required checks and evidence | Use the active Issue and existing repository capabilities. |
| Repository side effects | Commit, push, PR, merge, release, publish, or settings actions | Each action requires explicit authorization. |
| Completion expectation | Report, commit, PR, or another requested result | Do not infer an unrequested side effect. |

Agents MUST resolve information from the live repository when possible and
MUST NOT ask for information safely obtainable from the active Issue,
repository metadata, Git state, existing documents, tests, or configuration.
Stop when a critical input cannot be resolved without guessing a contract.

## Stage 1: Resolve Execution Context

The agent MUST:

1. confirm the repository;
2. confirm the default branch and requested base;
3. inspect the current branch and working-tree state when available;
4. identify the active Issue or explicit task;
5. confirm Issue state and milestone when relevant;
6. check for duplicate active work when a branch or PR is required;
7. identify requested repository side effects; and
8. record environment limitations affecting execution or validation.

Live repository state is authoritative over stale prompt metadata. Preserve and
report unrelated or uncommitted user work; never reset, discard, stash, or
rewrite it without explicit authorization. Duplicate active work is a stop
condition unless coordination or continuation was explicitly requested. A
broad repository inventory is not required.

## Stage 2: Select Authoritative Context

Always read the active Issue or explicit task, root `AGENTS.md`, and directly
affected files. Read this workflow when the task is non-trivial, escalation
classification is needed, validation or reporting is complex, the root
contract routes here, or the active task requires it.

Select additional sources by impact:

| Impact | Authoritative source |
| --- | --- |
| Documentation classification, source conflict, or supersession | [Documentation Index](../README.md) |
| Approval, authority, or unresolved decision | [Maintainership](../governance/MAINTAINERSHIP.md) |
| Architecture principles | [Architecture Principles](../architecture/PRINCIPLES.md) |
| Layer ownership and dependency direction | [Architecture Layers](../architecture/LAYERS.md) |
| Rust Core ownership, types, errors, concurrency, async, visibility, compatibility, or unsafe | [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md) |
| Dependencies, untrusted input, secrets, network trust, sensitive data, or security exceptions | [Secure Development](SECURE_DEVELOPMENT.md) |
| Significant architecture decision or supersession | [ADR Process](../decisions/README.md) |
| Contribution, branches, PRs, reviews, or shared Git workflow | [Contributing](../../.github/CONTRIBUTING.md) |
| Vulnerability reporting | [Security Policy](../../SECURITY.md) |
| Claude Code-specific execution | [Claude Code Supplement](../../CLAUDE.md), only when Claude Code is active |

Do not load every document for every task. Expand context only when discovered
impact justifies it. A small diff can require a specialized contract; a large
mechanical diff need not require unrelated architecture sources. Private chat,
automatic memory, prior sessions, and agent conclusions are not durable sources
of truth. An Issue may narrow scope but cannot silently weaken a specialized
contract.

## Stage 3: Confirm the Work Contract

Before implementation, establish an explicit internal contract covering:

- desired result, in-scope responsibility, and out-of-scope work;
- affected owner and lifecycle, and invariants;
- public API, serialized-format, and compatibility impact;
- dependency, security, concurrency, async, and `unsafe` impact;
- documentation impact and validation plan; and
- requested repository side effects.

A concise confirmation is sufficient for a trivial, fully specified change; a
verbose plan is not mandatory. For non-trivial work, missing ownership or
contract impact is a stop condition. The agent MUST NOT treat existing code, a
passing test, or an implementation plan as approval; infer permission from
assignment or milestone placement; or begin while required approval is
unresolved.

## Stage 4: Inspect the Minimum Implementation Context

Identify the smallest evidence set needed. Depending on impact, it MAY include
directly affected files, owned types and interfaces, callers and consumers,
relevant tests, controlling configuration, focused history, current public or
serialized surfaces, dependency declarations, and existing validation scripts.

Inspect enough to protect invariants and discover hidden consumers, but do not
scan unrelated modules or use broad exploration instead of a clear question.
Use history to understand an owned contract, not to reconstruct private intent.
Do not edit generated or vendored content without explicit scope. If inspection
reveals a new approval boundary, stop and reclassify.

Record a short implementation hypothesis: what must change, what must not
change, and how the change will be validated. The hypothesis is not
architecture approval.

## Stage 5: Implement the Smallest Coherent Change

Implementation MUST have one coherent responsibility and explicit ownership.
It MUST NOT include unrelated cleanup, hidden behavior changes, speculative
abstraction, future infrastructure, an unapproved dependency or public API,
an architecture-contract edit for implementation convenience, weakened tests
or validation, or a bypass through global state, generic utility modules,
cloning, shared mutation, or feature flags. Review generated changes to the
same standard as human-written work.

A necessary adjacent change MAY be included only when it is required for
correctness, inside the approved contract, crosses no new approval boundary,
is included in validation, and is explicitly reported. If it affects
architecture, API, serialization, compatibility, dependency, security,
concurrency, async, or unsafe scope, stop and escalate.

## Stage 6: Validate Proportionately

Plan validation from (1) active-Issue requirements, (2) affected specialized
contracts, and (3) relevant existing repository checks. Validate behavior or
contract meaning, not syntax or compilation alone.

Agents MUST run available relevant checks, record exact checks and results,
inspect failures rather than retry blindly, use focused checks before broader
checks when appropriate, preserve failure evidence, distinguish introduced
from pre-existing failures when evidence permits, and report omitted checks
with reasons.

Agents MUST NOT claim an unrun check; add a validator solely for a simple
documentation task unless required; weaken or delete tests; treat skipped tests
as passing; claim CI when none ran; or claim browser, platform, security,
performance, or compatibility validation without evidence.

Documentation validation SHOULD cover relevant changed-file scope, relative
links, Markdown structure or rendering, placeholders, private or sensitive
content, trailing whitespace, final newline, and diff whitespace. Future code
validation SHOULD use repository-owned commands when available. This workflow
does not prescribe package, compiler, browser, or platform commands before the
repository adopts them.

## Stage 7: Review the Final Diff

After validation, inspect the changed-file list, full diff, untracked files
when available, generated files, lockfile or dependency changes, public
visibility changes, serialized formats, configuration or settings changes,
documentation impact, test changes, and formatting churn.

Confirm every file is required; the diff matches the active Issue; no
out-of-scope cleanup, hidden compatibility break, sensitive data, private AI
link, or prompt transcript exists; required documentation is present; and
validation evidence is not overstated. A newly revealed contract boundary
returns the outcome to Stop and Escalate. A PR MUST NOT be represented as ready
while scope or contract issues remain unresolved.

## Stage 8: Complete or Escalate

### Complete

Complete only when the desired result is satisfied, scope is respected,
required validation ran or its limitation is explicitly accepted, the final
diff was reviewed, no approval boundary remains unresolved, and requested side
effects were completed or accurately reported unavailable.

### Escalate

An escalation record MUST include observed repository evidence, affected files
or contracts, why work cannot safely continue, the approval boundary,
realistic alternatives and trade-offs, a focused recommended next step, work
already completed, and validation already performed. An agent MAY recommend a
focused Issue or ADR classification but cannot approve it.

### Partial or Blocked

Apply the dedicated rules below and do not use either result to conceal an
unresolved contract.

## Escalation Matrix

| Trigger | Agent action | Required authority or document | Implementation status |
| --- | --- | --- | --- |
| Unresolved ownership or lifecycle | Stop at the boundary; identify candidate owners and evidence; do not use cloning, globals, shared mutation, or generic utilities as a substitute. | Maintainership, Principles, Layers, and Rust Core Contracts when applicable | Blocked pending explicit resolution. |
| Architecture boundary change | Stop; identify the layer and dependency direction; classify whether an ADR is required. | Maintainership, Principles, Layers, ADR Process | Implementation waits. |
| Browser-specific behavior entering Core | Stop; preserve protocol ownership in the Browser Adapter; identify the missing browser-independent contract. | Principles, Layers, Maintainership | Implementation waits. |
| Public API, exported type, serialized format, or compatibility change | Stop; enumerate compatibility dimensions; require focused approval and documentation. | Maintainership, Rust Core Contracts, applicable future public-contract source, and ADR Process when triggered | Implementation waits. |
| Dependency addition, removal, foundational change, or major update | Stop before declarations change; collect need, alternatives, maintenance, security, license, and compatibility evidence. | Secure Development, Maintainership, ADR Process when foundational | Implementation waits. |
| Security-sensitive behavior | Stop at the boundary; route vulnerability details privately and redact sensitive evidence. | Secure Development, Security Policy, Maintainership | Implementation waits or enters the private process. |
| Proposed `unsafe Rust` | Stop and do not write unsafe code; require a focused approved Issue and its full evidence contract. | Rust Core Contracts, Secure Development, Maintainership, ADR Process when required | Prohibited until explicit approval. |
| New concurrency, shared mutation, async, cancellation, streaming, process, IPC, or runtime boundary | Stop; identify owner, lifecycle, ordering, cancellation, backpressure, errors, and determinism; do not select synchronization or runtime by convenience. | Rust Core Contracts, Layers, Maintainership, ADR Process when foundational | Implementation waits. |
| Required scope expansion | Stop; separate a correctness-required adjacent change from a new responsibility; propose focused follow-up. | Active Issue, Maintainership | Existing scope continues only if separable and safe. |
| Conflicting sources of truth | Stop; classify each source and topic owner; do not choose by recency. | Documentation Index, Maintainership | Blocked pending durable resolution. |
| Validation failure implying redesign | Preserve evidence; classify implementation versus contract defect; do not weaken tests or invariants. | Active Issue, affected contracts, Maintainership for redesign | Return to scoped implementation for an implementation defect; otherwise stop. |
| Unauthorized repository side effect | Do not perform it; report required authorization or permission. | Active task, Shared Agent Contract, Contributing, and Maintainership when applicable | Implementation may be complete while the side effect is unavailable. |

## Defect Classification

### Implementation Defect

A typo, incorrect local logic, missing branch handling within approved behavior,
or implementation-caused test mismatch is an implementation defect. Fix it
within approved scope, validate, and report. No new approval is needed when
ownership and contracts remain unchanged.

### Contract or Architecture Defect

Undefined ownership, incompatible normative requirements exposed by tests, an
approved API unable to meet its invariants, conflicting Adapter and Core
responsibilities, or behavior requiring a new dependency or async boundary is
a contract or architecture defect. Stop, preserve evidence, escalate, and do
not patch around the contract.

### Environment or Tooling Defect

An unavailable compiler, runtime, credential, platform, network, CI facility,
or required write capability is an environment or tooling defect. Report the
limitation and use an existing approved substitute only when it gives
meaningful evidence. Do not claim equivalence without justification or install
new infrastructure outside scope.

A pre-existing unrelated defect is reported and left unchanged unless it
directly blocks the task, a focused adjacent fix is correctness-required and
permitted, no new boundary is crossed, and the change is explicitly reported.

## Validation Contract

| Change class | Minimum evidence |
| --- | --- |
| Documentation-only | Links, scope, placeholders/private content, Markdown, whitespace, and diff review |
| Internal implementation | Focused behavior tests plus existing formatting, lint, and type checks |
| Public or compatibility surface | Focused tests, consumer or fixture evidence, migration review, and documentation review |
| Dependency change | Necessity, alternatives, manifest and lock diff, and security, license, and compatibility review |
| Security-sensitive | Secure Development-required evidence and redaction |
| Concurrency or async | Ordering, cancellation, errors, lifecycle, backpressure, and determinism evidence |
| Unsafe | Full focused unsafe evidence and approval before implementation |
| Browser adapter | Protocol isolation, normalization loss or unsupported evidence, and Core independence |
| Core behavior | Browser-independent inputs, deterministic meaning, owned errors, and no outer-dependency leakage |

This table does not prescribe tools not adopted by the repository. The active
Issue and repository capabilities determine exact checks.

## Partial and Blocked Work

### Partial

The report MUST identify the completed and incomplete subsets, why they are
separable, validation for completed work, unresolved risks, whether a branch or
PR should remain draft or unmerged, and the needed follow-up Issue or decision.
Partial work MUST NOT use `Closes #<issue>` unless the Issue is actually
complete.

### Blocked

The report MUST identify the blocker, evidence, affected contract, attempted
safe steps, checks performed, unavailable or unsafe next step, and required
maintainer decision or environment capability. Blocked work MUST NOT guess,
claim completion, hide unresolved architecture behind a TODO, create
placeholder architecture, or weaken validation.

## Repository Side Effects

The [Shared Agent Contract](../../AGENTS.md) provides the common restriction,
and [Contributing](../../.github/CONTRIBUTING.md) owns shared Git and PR
workflow. Implementation permission and repository-mutation permission are
separate. An agent may edit scoped files when implementation is authorized,
but may create or amend a commit, push or force-push, create or update a PR,
merge, tag or release, publish, modify repository settings, create labels,
milestones, secrets, environments, or permissions, add CI, bots, automation,
or schedules, or contact external parties only when explicitly requested and
permitted.

When authorized, inspect status and the final diff, protect unrelated user
changes, perform the smallest side effect, report the exact result, stop on
permission failure or head movement, and do not substitute a different side
effect.

## Completion Report

Use the following schema. Keep sections concise and omit optional empty detail
only when the result remains unambiguous.

# Completion Report

## Result

One of: **Completed**, **Partial**, or **Blocked**.

## Desired Result

State whether the requested observable or contractual result was achieved.

## Changed Scope

List files, behavior, contracts, and necessary adjacent changes.

## Validation Performed

For each check, state the command or manual check, result, and relevant evidence.

## Validation Not Performed

For each omitted check, state the check, reason, and resulting limitation or risk.

## Contract Impact

State the impact or `None` for architecture; ownership and lifecycle; public
API; serialized format; compatibility; dependency; security; concurrency;
async; `unsafe Rust`; and documentation.

## Deviations

List deviations from the active Issue or planned validation and any unrequested
changes, which should normally be none.

## Risks and Limitations

List known risks, unverified assumptions, environment limitations, and
compatibility concerns.

## Unresolved Decisions or Blockers

List each needed decision, owning authority, evidence, and alternatives.

## Follow-Up

List focused follow-up work or a required Issue or ADR; use `None` when absent.

## Repository Side Effects

Only when performed or requested, list branch, commit SHA, pushed ref, PR URL,
merge SHA, or an unavailable action and its reason.

A report MUST NOT claim checks or side effects not performed, include private
AI links or full prompt transcripts, or present workflow compliance as
architecture approval. Durable records must stand on repository evidence and
distinguish fact, assumption, proposal, and blocker.

## Representative Simulations

The table records selected and intentionally omitted context, deterministic
outcome, permitted implementation and side effects, validation, and final
report result. “Specialized contracts” in the omitted column means sources
without an impact in that scenario, not the directly affected sources named in
the selected column.

| # | Scenario | Selected authoritative documents | Intentionally omitted documents | Outcome | Allowed implementation and repository side effects | Validation | Completion-report result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Fully specified documentation Leaf Issue with an explicitly requested PR | Active Issue, AGENTS, affected file, Documentation Index, Contributing, this workflow | Rust Core, architecture, and security contracts | Proceed | Focused documentation and the explicitly authorized PR only | Links, scope, placeholders, Markdown, and diff | Completed |
| 2 | Rust task needs shared mutable state but has no owner or lifecycle | Active Issue, AGENTS, Layers, Rust Core Contracts, Maintainership, this workflow | Security and contribution sources absent another impact | Stop and Escalate, then Blocked | Evidence and alternatives only; no `Arc<Mutex<_>>`, clone, global, utility bypass, commit, or PR | Preserve ownership and lifecycle evidence | Blocked pending ownership resolution |
| 3 | An unapproved parser or runtime dependency would simplify work | Active Issue, AGENTS, Secure Development, Maintainership, ADR Process if foundational, this workflow | Unaffected architecture and tool-specific sources | Stop and Escalate, then Blocked | Research need and alternatives only; no manifest, lockfile, or repository side effect | Security, maintenance, license, compatibility, and ADR classification evidence | Blocked pending approval |
| 4 | Required tests expose an architecture defect | Active Issue, AGENTS, affected tests, Principles, Layers, Maintainership, ADR Process, this workflow | Unaffected security and contribution sources | Stop and Escalate, then Blocked | Preserve failure and propose focused review; no weakened test or bypass | Failure evidence and contract-defect classification | Blocked pending architecture resolution |
| 5 | Nearby cleanup is unrelated to a scoped bug fix | Active Issue, AGENTS, affected code and tests, impact-selected contracts, this workflow | Unaffected specialized contracts and cleanup files | Proceed | Complete only the bug fix; no unrelated cleanup or side effect beyond the request | Focused regression and relevant repository checks; final diff | Completed |
| 6 | Code change is authorized but no commit is requested | Active task, AGENTS, affected code and tests, impact-selected contracts, this workflow | Contributing unless shared Git workflow otherwise matters; tool-specific sources | Proceed | Implement and validate in the working tree; no commit, push, or PR | Relevant focused and repository checks; status and diff | Completed with working-tree changes reported |
| 7 | Documentation task has no Markdown linter | Active task, AGENTS, affected documentation, Documentation Index, this workflow | Unaffected code, Rust, architecture, security, and tool-specific sources | Proceed | Focused documentation; no new linter or unrequested side effect | Repository-local links, structure, whitespace, rendering review, and diff; linter reported not configured | Completed |
| 8 | Test suite has a pre-existing unrelated failure | Active task, AGENTS, affected implementation and tests, impact-selected contracts, this workflow | Unaffected specialized and tool-specific sources | Proceed if evidence proves separability; otherwise Stop and Escalate or Blocked | Approved implementation only; do not fix unrelated failure unless the bounded adjacent-change rule applies | Focused passing evidence plus preserved and accurately reported suite failure | Completed with limitation, or Blocked if correctness cannot be established |

These simulations validate workflow classification; they do not approve the
technical decisions represented by their scenarios.

## Deferred Tool-Specific Behavior

Root [Claude Code Supplement](../../CLAUDE.md) continues to own Claude
Code-specific application behavior. Future tool-specific supplements MAY add
constraints but cannot weaken `AGENTS.md` or this workflow. Model-specific
prompt engineering is not repository policy. Permission modes, hooks, skills,
subagents, memory, and settings remain tool-specific and out of scope unless a
future focused Issue approves them. This document creates no tool-specific file
or behavior.
