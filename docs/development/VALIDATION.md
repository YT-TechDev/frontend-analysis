# Validation and Completion Evidence

## Purpose and Authority

This document is the normative repository-wide source for general validation
and completion evidence. It determines how validation is selected, the minimum
evidence by change class, how checks are recorded, how omissions and failures
are handled, and what validation evidence completion requires.

It does not approve architecture, APIs, dependencies, security exceptions,
`unsafe Rust`, or releases; replace an active Issue, a specialized contract, or
maintainer review; require unavailable infrastructure; or define
feature-specific tests. Another repository must explicitly adopt this contract
before it applies there. The [Documentation Index](../README.md) classifies
repository sources, and the [MIT License](../../LICENSE) remains unchanged.

## Normative Language

**MUST** and **MUST NOT** state mandatory evidence requirements. **SHOULD** and
**SHOULD NOT** state defaults whose deviation requires a recorded reason.
**MAY** states permission when the active Issue and specialized contracts remain
satisfied. These terms govern evidence, not technical approval.

## Validation Ownership

| Owner | Owns |
| --- | --- |
| Active Leaf or explicit task | Desired result, task-specific acceptance criteria and checks, exclusions, and target environment |
| This document | General principles, statuses, baseline evidence, manual records, failure handling, and the validation portion of completion evidence |
| Specialized normative contract | Domain invariants and additional architecture, Rust, security, adapter, compatibility, dependency, concurrency, async, or unsafe evidence |
| [Implementation Agent Workflow](AGENT_WORKFLOW.md) | Execution stages, stop and escalation behavior, final-diff review, and overall completion-report workflow |
| [Issue Model](ISSUE_MODEL.md) | The requirement that an executable Leaf defines validation and completion expectations |
| [Pull Request template](../../.github/PULL_REQUEST_TEMPLATE.md) | A non-authoritative evidence-collection form |
| Maintainer | Evidence-sufficiency judgment and approval under [Maintainership](../governance/MAINTAINERSHIP.md) |

All applicable owners apply; none silently cancels another. A task MAY add
stricter checks but cannot silently remove specialized requirements. Automated
checks do not create approval. Maintainer judgment cannot make a failed
invariant pass without an approved contract or scope change.

## Core Principles

1. **Validate the changed contract.** Compilation, rendering, or one passing
   test is not universal completion evidence.
2. **Proportionality follows risk and impact.** Select evidence by changed
   responsibility, not line count.
3. **Identify the validation target.** Evidence MUST identify the tested commit
   or Pull Request head.
4. **Inspect the final diff.** Every completion claim requires changed-file and
   final-diff review.
5. **Use repository-owned checks.** Prefer approved commands, fixtures, and
   workflows that exist in the repository.
6. **Do not invent unavailable success.** Report unavailable tooling as a
   limitation, never as Passed.
7. **Preserve failures.** Do not weaken tests, delete evidence, retry blindly,
   or hide a failure.
8. **Separate automated evidence, manual evidence, and maintainer judgment.**
9. **Make results reproducible.** Record enough target, environment, input, and
   method context to repeat the result.
10. **Protect sensitive information.** Apply security redaction and private
    reporting requirements to logs and artifacts.
11. **Require no private AI dependency.** Durable evidence MUST stand without
    private conversation links or prompt transcripts.
12. **Do not confuse passing checks with authorization.** Passing checks do not
    approve a contract change.

## Validation Planning

Before implementation or validation, identify the target, desired result and
acceptance criteria, changed contract classes, applicable baselines,
specialized evidence, exact available and manual checks, environment and
platform requirements, fixtures, expected evidence and artifacts, unavailable
checks, stop conditions, and who judges sufficiency. Plan early enough to avoid
implementing an untestable design first.

A lightweight documentation correction needs only a concise plan. Public,
security, dependency, adapter, concurrency, async, performance, or unsafe work
requires an explicit plan proportional to risk.

## Evidence Status Vocabulary

Individual checks MUST use exactly these statuses:

- **Passed**: the automated check ran or manual review occurred; expected and
  observed outcomes were recorded; evidence supports the claim; warnings and
  limitations remain visible.
- **Failed**: the check ran but did not meet the expected result. Preserve the
  failure; a required failure blocks completion.
- **Not run**: the check was considered but intentionally not executed. Record
  the reason and resulting limitation or risk. This includes skipped checks;
  do not use a bare `Skipped` status.
- **Blocked**: a required or attempted check cannot proceed because of an
  unresolved dependency, permission, environment, tool, platform, or contract
  boundary. Record the blocker and required resolution.
- **Not applicable**: the category does not apply. Give a concise rationale
  when that is not obvious.

Every required check has one status. Required Failed or Blocked evidence
normally prevents **Completed**. Required Not run evidence prevents Completed
unless the active Issue or owner permits an approved substitute. Optional
checks MAY be Not run with rationale. A maintainer cannot relabel Failed as
Passed; changing a requirement requires a durable Issue or contract update.

## Universal Completion Checks

Every non-trivial change MUST record:

1. the active Issue or trivial-maintenance rationale;
2. tested revision or Pull Request head SHA;
3. final changed-file list and final-diff inspection;
4. scope and unrelated-change review;
5. acceptance-criteria review;
6. active-Issue and applicable specialized validation;
7. exact commands and manual checks, with a status for every required check;
8. checks Not run or Blocked, with reasons;
9. documentation impact review;
10. sensitive-data and private-link review;
11. known limitations and residual risks;
12. follow-up work or `None`; and
13. a completion result consistent with the evidence.

A truly trivial direct Pull Request MAY be concise, but still identifies the
change, inspects the diff, and reports validation honestly.

## Validation Matrix

| Change class | Baseline evidence | Additional owner |
| --- | --- | --- |
| Documentation-only | Documentation baseline and universal checks | Documentation Index and affected contract |
| Root agent or governance contract | Documentation baseline, routing/import review, scenarios | Shared Agent Contract, Maintainership |
| Internal Rust implementation | Future Rust baseline, focused behavior/regression tests | Rust Core Contracts |
| Public Rust API | Rust baseline and intentional public-surface evidence | Rust Core Contracts, Maintainership |
| Serialized representation | Rust baseline and representation/semantic compatibility evidence | Owning public/result contract |
| Dependency or toolchain | Need, resolution, supply-chain, and baseline evidence | Secure Development, Maintainership |
| Security-sensitive | General baseline plus threat, boundary, regression, and redaction evidence | Secure Development and Security Policy |
| Browser adapter or normalization | Adapter fixtures, boundary and normalization evidence | Architecture Layers and Rust Core Contracts |
| Core analysis behavior | Deterministic semantic and Analysis Result regression evidence | Architecture contracts |
| Concurrency or async | Lifecycle, ordering, failure, cancellation, boundedness, and determinism evidence | Rust Core Contracts |
| Performance | Reproducible baseline/candidate measurement plus correctness | Active Issue and owning contract |
| Repository-authored unsafe | Approved complete unsafe contract plus selected validation | Rust Core Contracts and Secure Development |
| Repository configuration | Source and live-state evidence, permission review, rollback | Maintainership and affected service contract |
| Cross-platform or browser-specific behavior | Applicable baseline and identified environment matrix | Active Issue and affected adapter/platform contract |

This is a router. The following sections own the detailed general evidence;
specialized contracts add their requirements.

## Documentation Changes

As applicable, documentation evidence MUST cover final Markdown structure;
relative links; anchors and headings used by links; terminology; source-of-truth
ownership; stale, future, duplicate, or contradictory statements; intentional
versus unresolved placeholders; sensitive or personal data, private links, and
private AI links; code-fence and table rendering; trailing whitespace; final
newline; `git diff --check` when Git is available; GitHub recognition or
placement for community-health files; and normative scenario/example
consistency.

No Markdown linter is required while none is configured. Record it as Not run
or Not applicable with the reason. Rendering alone does not prove correct
meaning or valid links.

## Future Rust Baseline

This section becomes operational only after an approved Rust workspace exists.
During the current documentation-only foundation milestone, Rust commands are
Not applicable. Future baseline categories are formatting, linting,
compile/type checking, tests, documentation, and build or artifact verification
when the task produces an artifact. Standard baseline forms are:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo doc --workspace --all-features --no-deps
```

These are future forms, not currently available commands. Once a workspace
defines repository-owned commands, use those. `--all-features` applies only
when all features are intended to compose; mutually exclusive,
platform-specific, or capability-specific features require an Issue-defined
matrix. Platform targets require the relevant environment or an honestly
reported limitation. Treat documentation warnings as errors when an approved
cross-platform method exists. Add `cargo build` for deliverable artifacts or
build-specific behavior, and examples, doctests, benches, or target checks only
when affected.

This contract creates no MSRV, channel, feature, target, or CI policy and
selects no extra test, lint, coverage, or compatibility tool. Successful
compilation or `cargo check` never replaces behavioral tests.

## Internal Rust Changes

A private implementation or refactor requires, when available, the Rust
baseline, focused changed-behavior tests, defect regression evidence, and
evidence supporting any claim that public APIs and serialized meaning are
unchanged. Review affected ownership, errors, panics, determinism, visibility,
and the final diff. Confirm no accidental dependency, feature, re-export,
`Send`, `Sync`, or serialization change. Require performance evidence only when
performance is claimed. A no-behavior-change refactor still needs behavioral
evidence; “no public API change” requires exposed-surface review.

## Public API and Serialized Contracts

### Public Rust API

As applicable, require an intentional public-surface diff; rustdoc/public
documentation review; consumer or compile-use evidence; error, trait-bound,
generic, feature, auto-trait, and re-export review; compatibility and migration
analysis; supported-use examples or doctests; deprecation/replacement plan; and
approval evidence. Use no SemVer tool until the repository adopts one.

### Serialized Representation

Separately require the exact schema or representation diff; round-trip
evidence; backward/forward compatibility; unknown field/value behavior;
absent, partial, unsupported, redacted, and conflicting states; ordering and
determinism; defaults; malformed and boundary inputs; migration evidence; and
stable fixture/golden evidence only under an approved framework. Debug output
is not serialization evidence. Unchanged bytes do not prove unchanged semantic
meaning.

## Dependencies and Supply Chain

A dependency addition, removal, major update, parser, runtime, build tool, or
toolchain change MUST satisfy [Secure Development](SECURE_DEVELOPMENT.md) and
show need and alternatives; direct/transitive impact; manifest and lockfile
diff or artifact-specific lockfile rationale; features/default features;
platform and future Rust compatibility; maintenance/release activity;
reasonably available security history/advisories; license compatibility;
public type leakage; build-script, native, network, and privileged behavior;
reproducibility; replacement/removal cost; post-change baseline validation; and
that unrelated resolution was not concealed.

No advisory, license, or dependency-tree product is mandated. Record the tool
or manual source used. Convenience alone does not justify a dependency.

## Security-Sensitive Changes

Security-sensitive work MUST satisfy this contract, [Secure Development](SECURE_DEVELOPMENT.md),
and the [Security Policy](../../SECURITY.md). Require proportional security
review; threat/abuse analysis; accepted/rejected inputs; malformed, boundary,
and resource-limit evidence where applicable; secret/private-data and artifact
redaction review; permissions/least privilege; network, file, parser, process,
credential, or persistence-boundary evidence; security regressions; residual
risk; explicit maintainer approval; and private handling of active
vulnerability details.

Public records MUST NOT expose exploit instructions, active vulnerability
details, credentials, private user or infrastructure data, or sensitive private
remediation links. General Passed status waives no security requirement.

## Browser Adapter and Normalization

Validate the adapter boundary, not only connectivity. As applicable, prove that
protocol types remain in adapters; known values normalize correctly; unknown
and unsupported observations remain distinguishable from absence; missing,
partial, malformed, and conflicting observations remain honest; lossy mapping
is explicit; adapter errors do not leak into Core public contracts; identifiers,
timestamps, units, coordinates, and ordering retain approved meaning;
equivalent normalized inputs retain equivalent Core meaning; engine/protocol
assumptions and fixture source/version are recorded; Core validation uses no
protocol type; and artifacts are redacted.

Live-browser records identify browser, version, OS, connection mode, and exact
steps. Fixtures and payloads support deterministic regression but do not prove
live runtime behavior. Report unavailable browser validation honestly. This
contract selects no browser automation or protocol library.

## Deterministic Analysis and Regression Evidence

Core analysis and Analysis Result evidence MUST cover equal normalized inputs
and approved configuration; semantically equivalent outputs; contractual
stable ordering and explicit non-semantic ordering; no hidden dependence on
hash iteration, allocation addresses, timestamps, scheduler completion, or
ambient environment; partial, unsupported, unknown, and conflicting evidence;
reproducible fixtures; focused defect regressions; diagnostic/certainty meaning;
and no presentation-layer semantic redefinition.

For intentionally variable metadata, compare owned semantic fields rather than
claiming byte identity without a contract. This document invents no Analysis
Result schema.

## Concurrency and Async

Approved changes to concurrency, shared mutation, async, cancellation,
streaming, channels, processes, IPC, or runtime boundaries require evidence for
owner/lifecycle; ordering; cancellation/interruption; shutdown;
boundedness/backpressure; error propagation/aggregation; races; deadlock,
starvation, and lock ordering; locks across callbacks or suspension;
deterministic semantics; cleanup; partial completion; deterministic or
single-thread comparison where possible; platform behavior; and public `Send`
or `Sync` impact. A happy-path test is insufficient. This contract selects no
runtime, executor, channel, model checker, or synchronization mechanism.

## Performance and Unsafe Rust

### Performance Claims

A performance claim requires a reproducible benchmark or profile recording
exact baseline and candidate revisions, relevant hardware/environment, tool and
version, dataset and size, configuration, warmup/iterations where applicable,
values and units, variability/noise treatment, interpretation, correctness
regressions, limitations, and a safe raw artifact or durable summary.

One unrepeatable timing, cherry-picked output, changed baseline input,
performance without correctness, and speculative optimization are insufficient.
No framework or threshold is selected here.

### Repository-Authored Unsafe Rust

Unsafe remains prohibited without an explicitly approved Issue. Before any
implementation, apply the complete evidence and containment contract in [Rust
Core Contracts](../architecture/RUST_CORE_CONTRACTS.md) and security rules in
[Secure Development](SECURE_DEVELOPMENT.md).

The record identifies, as applicable, the approved boundary; safety invariants;
edge and regression tests; performance evidence when it is the justification;
platform assumptions; static, dynamic, fuzz, sanitizer, interpreter, and
platform checks selected by the approved Issue; omitted checks and residual
risk; containment/removal strategy; and maintainer approval. Miri, sanitizers,
fuzzing, and benchmarks are not universally mandatory before adoption.
Compilation alone is never sufficient. An unapproved proposal stops before
implementation.

## Repository Configuration

For repository, security, branch, workflow, permission, secret, environment,
or service configuration, record the intended setting; observable prior and
resulting live state; least privilege and secrets review; changed tracked
files; external changes; actor/timestamp when relevant; rollback; service
recognition; dry run/test; contributor/automation impact; permission limits;
and an explicit statement when no live verification occurred.

A source diff does not prove a remote setting changed. Prefer structured state
over screenshots. This Issue changes no repository configuration.

## Manual Validation

Each manual scenario MUST record target revision, objective, environment and
versions, preconditions, input/fixture, exact steps, expected and actual
observations, status, artifacts, limitations, and cleanup/restoration. For
browser/platform behavior, include relevant OS, browser/engine and version,
connection/execution mode, and hardware when material.

Manual validation is valid when automation is unavailable or interaction is
inherent, but is not automatically automated regression coverage. Redact logs
and screenshots; do not use OCR or screenshots when structured evidence is
available.

## Failed, Blocked, and Unavailable Validation

### Failed

Record the exact failing check, target, output or durable summary, whether it
appears introduced/pre-existing/unresolved, acceptance impact, and next safe
action. A required failure normally yields Partial or Blocked. Never weaken or
delete the check to obtain Passed.

### Blocked

Record the required check, blocker, needed environment/authority, safe attempts,
approved substitute if any, residual risk, and responsible next step.

### Not Run

Record the considered check, reason, why it is optional or unobtainable,
limitation, and required follow-up. Unavailable checks do not disappear.

## Pre-Existing and Flaky Failures

For a pre-existing failure, when practical reproduce it at a recorded base
revision, compare signatures, show why the scoped change did not introduce it,
run focused changed-behavior checks, and report the repository-wide failure.
Never claim the full suite passed or fix an unrelated defect unless the Issue
Model's bounded adjacent-change rule permits it.

For flaky/intermittent failure, do not retry until success and report only that
success. Record every relevant attempt or justified sample, frequency,
environment, signature, investigation, scoped-change relationship, confidence,
and risk. One successful retry does not make a flaky required check Passed.
Create focused follow-up when warranted.

## Validation Record

| Field | Required content |
| --- | --- |
| Check | Stable descriptive name |
| Requirement source | Active Issue, this document, or specialized contract |
| Target | Commit SHA, PR head, file set, artifact, or environment |
| Method | Exact automated command, manual scenario, review, or structured external check |
| Environment | Relevant OS, toolchain, browser, platform, or service |
| Expected | Expected result |
| Observed | Actual result |
| Status | Passed, Failed, Not run, Blocked, or Not applicable |
| Evidence | Output summary, artifact, fixture, log, or durable reference |
| Limitations | Remaining uncertainty or risk |
| Follow-up | Required next action or `None` |

Copy commands accurately. Summarize rather than paste excessive logs, and link
safe artifacts where available. Private AI links are never evidence.

## Completion Evidence

The overall report remains owned by the [Implementation Agent Workflow](AGENT_WORKFLOW.md).
Its validation portion MUST include:

- **Validation target:** repository/revision, PR head when applicable, and
  relevant environment.
- **Validation performed:** each exact command/manual method, status, evidence,
  and limitation using the [validation record](#validation-record).
- **Validation not passed:** Failed, Not run, Blocked, and useful Not applicable
  entries with reason and impact.
- **Final review:** changed files, final diff, untracked/generated/lock/config
  changes, contract/documentation impact, sensitive-data review, and unrelated
  change review.
- **Validation conclusion:** “evidence supports completion,” “evidence supports
  only partial completion,” or “completion is blocked,” consistent with the
  workflow's Completed, Partial, or Blocked result.

Validation does not replace report sections for scope, impacts, deviations,
risks, blockers, follow-up, and repository side effects.

## Maintainer Judgment

Checks show whether tested properties were observed. Maintainers judge whether
the right properties were tested, evidence is sufficient, scope/contracts are
satisfied, residual risk is acceptable within their authority, more evidence
is needed, or a decision/Issue must change.

Passing checks approve no architecture. Maintainers must not ignore a failed
mandatory invariant without a durable contract or scope change. Requests for
more evidence SHOULD identify the protected risk or contract. Approval remains
explicit and attributable. Reproducible local/manual evidence can support
completion without CI; absence of evidence is never success.

## Representative Scenarios

Each row records change classes; baseline; specialized evidence; command
categories; manual evidence; omitted/unavailable checks; approval; and the
completion result.

| Scenario | Change classes | Baseline | Specialized evidence | Commands/categories | Manual evidence | Omitted/unavailable | Approval boundary | Completion result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `SECURITY.md` change | Documentation, security | Documentation and universal checks | Policy discoverability, Secure Development consistency, private route and sensitive-content review | Link resolution, structure, `git diff --check` | GitHub placement/discoverability when accessible; never send a test report | Rust/browser Not applicable; live recognition Not run if inaccessible | Maintainer security review | Only when the private route remains usable and clear |
| Root agent-file change | Documentation, agent contract | Documentation and universal checks | Import/routing, ownership/duplication, concise size, representative task simulations | Link/import review, structure, `git diff --check` | Safe runtime tool-context check when available | Runtime check Not run if unavailable; current Rust baseline Not applicable | Maintainer contract review | Only when routing and simulations agree |
| Future private Rust refactor | Internal Rust | Future Rust baseline and universal checks | Behavior/regression; unchanged API, serialization, semantics, dependencies; ownership/panic/error/visibility/determinism | Format, lint, check/build as affected, tests, docs | Only affected interactive/platform behavior | Public approval Not applicable absent public change | Existing contract plus ordinary review | Introduced required failure blocks completion |
| Future public Rust type change | Public API | Rust baseline and universal checks | Surface/rustdoc/use, compatibility, migration/deprecation, features/platforms; serialization only if affected; ADR if triggered | Format, lint, check/build as affected, tests, docs, consumer compile use | Supported consumer scenario where useful | Unsupported platforms recorded | Explicit maintainer approval | Compile success alone cannot complete |
| New dependency | Dependency/toolchain | Universal and applicable Rust baseline | Need/alternatives, manifest/lock/transitives, features, maintenance, security, license, platform, leakage; ADR if foundational | Resolution/tree tools actually adopted plus baseline | Manual source review | Unavailable advisory sources recorded | Explicit approval before addition | No addition or completion before approval/evidence |
| Browser adapter normalization | Adapter, Core boundary | Rust baseline when code exists and universal checks | Known/unknown/unsupported/missing/partial/malformed/lossy mappings, containment, units/order/errors, deterministic fixtures, engine version, redaction | Adapter fixtures/regressions and Rust baseline | Live browser only for a live claim | Live check honestly unavailable; fixtures never relabeled live | Architecture/adapter owner approval as required | Only claims actually evidenced behavior |
| Performance optimization using `unsafe` | Performance, unsafe, security | Rust baseline, correctness, reproducible measurement | Safe alternatives, complete unsafe containment, selected static/dynamic/fuzz/sanitizer/interpreter/platform evidence | Approved Rust checks and baseline/candidate benchmark | Platform/hardware reproduction | Every unavailable check and residual risk | Focused Issue and explicit approval before implementation | Compilation or one timing never completes; unapproved work stops |
| Branch-protection or workflow setting | Repository configuration, security | Universal/configuration evidence | Before/after live state, permissions, rollback, recognition | Source checks and available service/dry-run checks | Live setting verification | Permission-limited verification Blocked/Not run | Authorized maintainer/service actor | Local diff alone cannot claim remote completion |
| Required browser check unavailable | Browser/platform | Applicable baseline | Required environment and approved substitute | Fixture checks if approved, never as live evidence | Intended browser scenario recorded | Blocked if mandatory; Not run if optional, with risk | Owner approves only a permitted substitute | No false browser claim; mandatory unsatisfied check blocks |
| Pre-existing unrelated test failure | Affected code plus failure handling | Applicable baseline and focused checks | Base reproduction/signature comparison and changed-behavior evidence | Full suite plus focused test commands | Manual reproduction only if needed | Failure remains Failed, never hidden | Active Issue must permit limitation | Complete only if scoped correctness is established and limitation permitted |

## Deferred Automation and Tooling

This contract defers CI workflows, required status checks, Markdown-linter
selection, test frameworks beyond Rust defaults, coverage thresholds,
compatibility tools, browser automation, benchmark frameworks, fuzzing, Miri
policy, sanitizers, mutation testing, snapshot/golden frameworks, release
qualification, target/browser matrices, MSRV, performance thresholds, artifact
signing, and automatic validation reports.

Focused future Issues MAY adopt them. Adoption must update this authoritative
contract and the relevant workflow. See also [Contributing](../../.github/CONTRIBUTING.md),
the [Shared Agent Contract](../../AGENTS.md), [Architecture Principles](../architecture/PRINCIPLES.md),
[Architecture Layers](../architecture/LAYERS.md), and the [ADR Process](../decisions/README.md).
