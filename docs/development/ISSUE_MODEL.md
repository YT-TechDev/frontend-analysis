# Issue Hierarchy and Slicing Model

## Purpose and Authority

This document is the normative source for Issue hierarchy and slicing mechanics
in Frontend Analysis. It defines how work is represented, decomposed,
coordinated, and completed in this repository. Another repository must
explicitly adopt this model before it applies there.

This document does not approve architecture, replace the active Issue, replace
[Maintainership](../governance/MAINTAINERSHIP.md), define Pull Request review
policy, define detailed validation commands or release policy, require GitHub
Projects or sub-Issue automation, or define production features. The active
Issue defines scoped work but cannot override the specialized contracts listed
in the [Documentation Index](../README.md). [Contributing](../../.github/CONTRIBUTING.md)
governs contributor and Pull Request workflow, and the
[Implementation Agent Workflow](AGENT_WORKFLOW.md) governs agent execution
within an Issue contract.

## Normative Language

In this document:

- **MUST** and **MUST NOT** identify mandatory Issue-model requirements.
- **SHOULD** and **SHOULD NOT** identify the default expectation; a deviation
  requires a durable reason.
- **MAY** identifies a permitted option when clarity, ownership, and every
  applicable contract remain satisfied.

## Core Principles

1. **Use the smallest useful structure.** Hierarchy clarifies ownership and
   execution; it does not exist to create ceremony.
2. **Slice by coherent responsibility or contract.** Do not slice by arbitrary
   file, heading, line, or tiny mechanical-step counts.
3. **Resolve design before execution.** A Leaf MUST NOT transfer unresolved
   architecture decisions to an implementation agent.
4. **Keep one durable owner per responsibility.** Each level owns distinct
   scope and does not duplicate lower-level implementation contracts.
5. **Make dependencies explicit.** Durable GitHub records expose ordering and
   blocking.
6. **Prefer one Leaf to one coherent Pull Request.** Review and completion
   evidence remain understandable.
7. **Do not broaden a Pull Request silently.** Change the Issue contract first
   or create focused follow-up work.
8. **Use public durable records.** Normal work MUST be understandable without
   private AI conversations.
9. **Keep sensitive work private.** The security process overrides public
   discoverability when disclosure would be unsafe.
10. **Do not infer approval from hierarchy.** Parent, Child, Leaf, milestone,
    assignment, relationship, or checklist state grants no approval.

## Choosing the Smallest Useful Structure

Choose the first structure that expresses the actual responsibility and
coordination need.

### Direct trivial Pull Request

No Issue is required when all of the following are true:

- the change is obvious and isolated;
- it is a typo, broken link, or minor formatting correction;
- it has no architecture, behavior, API, compatibility, dependency, security,
  or contract impact; and
- review scope is immediately understandable.

This path remains governed by [Contributing](../../.github/CONTRIBUTING.md).

### Standalone Leaf Issue

Use a standalone Leaf when work is non-trivial or needs durable scope, involves
one coherent responsibility, gains no useful ownership from a Parent or Child,
and can normally be completed by one focused Pull Request. Required design and
approval must already be resolved or be resolved durably in that focused Issue
before implementation.

### Parent with direct Leaf Issues

A Parent MAY link Leaves directly when one repository-level outcome has
multiple executable slices, separate Child workstreams add no meaningful
boundary, every Leaf is independently reviewable, and the Parent clearly owns
coordination and completion. Do not create empty Children merely to produce a
three-level shape.

### Parent, Child, and Leaf hierarchy

Use all three levels when an outcome crosses multiple contract domains or
coherent workstreams, each Child owns a distinct boundary, each workstream has
one or more executable slices, and ordering, dependencies, or final validation
need aggregate coordination.

### Invalid or exceptional structures

A Child without a Parent is prohibited: without a Parent to decompose, the
Issue is a standalone Leaf. Hierarchy deeper than Parent to Child to Leaf
SHOULD NOT be used. An exception requires a demonstrated coordination need,
explicit maintainer approval, a durable rationale, and confirmation that a
separate Parent or repository would not be clearer. This model defines no
fourth-level terminology.

## Standalone Leaf Issue

A standalone Leaf is the same executable contract as any other Leaf but has no
coordination Parent or Child. It:

- owns one coherent, non-trivial responsibility;
- MUST satisfy every Leaf required section;
- MAY be assigned directly to a milestone when executable;
- normally maps to one coherent Pull Request; and
- MUST NOT be classified as a Child merely because follow-up work exists.

Follow-up work normally uses separately linked Issues.

## Parent Issue

A Parent owns a repository-level, architectural, product-level,
migration-level, or release-level outcome. It coordinates work and normally
does not contain each Leaf's complete implementation contract.

A Parent MUST define its overall Goal and Motivation; durable Invariants; In
Scope and Out of Scope; Constraints; Child workstreams or direct Leaf slices;
dependencies and ordering; aggregate Acceptance Criteria and Validation;
Risks; Completion Evidence; deferred work; and milestone policy. It remains
the durable coordination index.

A Parent MUST NOT duplicate complete Leaf instructions, hide unresolved
architecture in a Leaf list, treat completion of Children as automatic Parent
completion, close before aggregate acceptance and validation, or use one broad
Pull Request for unrelated workstreams. A Parent normally has no implementation
Pull Request. It MAY receive explicitly scoped coordination-only documentation
changes, but those do not replace executable Leaf contracts.

## Child Issue

A Child owns one coherent contract domain or workstream beneath a Parent, such
as security and vulnerability handling, contribution and maintainership,
architecture documentation, agent execution contracts, Issue and validation
governance, or a final repository audit.

A Child MUST link its Parent and define its Goal, Motivation, owned contract
domain, In Scope, Out of Scope, Constraints, boundaries among Leaves,
dependencies and ordering, workstream Acceptance Criteria and aggregate
Validation, discoverable Leaf links, and Completion Evidence.

A Child MUST NOT repeat every Leaf field, implement unrelated responsibilities,
delegate unresolved cross-workstream architecture to a Leaf, or close before
required Leaves and workstream validation complete. A Child normally has no
implementation Pull Request. Omit a Child that would contain one Leaf and add
no material contract boundary; retain it only when its workstream ownership is
genuinely useful.

## Leaf Issue

A Leaf is the executable implementation contract and owns one coherent,
reviewable result. A Leaf MUST define:

- Parent hierarchy, or `Standalone Leaf`;
- Goal or Desired Result, plus Motivation when it is not obvious;
- Authoritative Context;
- In Scope and Out of Scope;
- Constraints and Invariants;
- Approval State or each required approval boundary;
- observable Acceptance Criteria;
- Validation without inventing unavailable commands;
- Dependencies and Blocking Order;
- Stop Conditions;
- Completion Report requirements;
- expected Pull Request slicing;
- required documentation impact; and
- the security or private-reporting route when applicable.

A Leaf SHOULD state the effect on architecture, ownership and lifecycle, public
API, serialized formats, compatibility, dependencies, security, concurrency,
async, and `unsafe Rust`. Use `None` or an explicit no-impact statement when it
improves clarity.

A Leaf MUST NOT ask an agent to "design as needed," delegate ownership to
implementation, use vague acceptance such as "works correctly" without an
observable criterion, require indescribable validation, mix unrelated cleanup,
silently add future infrastructure, require private AI links, or close merely
because code was generated. Design discussion MAY occur in a Leaf, but
implementation begins only after required decisions and approvals are durable.

## Required Sections by Level

| Section | Parent | Child | Leaf / Standalone Leaf |
| --- | --- | --- | --- |
| Hierarchy | Required; list Children and direct Leaves | Required; link Parent and Leaves | Required; link owners or state `Standalone Leaf` |
| Goal / Desired Result | Required aggregate Goal | Required workstream Goal | Required executable result |
| Motivation | Required | Required | Required when not obvious |
| Invariants | Required aggregate invariants | Required for workstream | Required constraints and invariants |
| Authoritative Context | Required when applicable | Required when applicable | Required |
| In Scope | Required | Required | Required |
| Out of Scope | Required | Required | Required |
| Constraints | Required | Required | Required |
| Workstreams / Leaf Boundaries | Required | Required | Not normally applicable |
| Dependencies / Ordering | Required | Required | Required; use `None` if absent |
| Approval State | Required when aggregate approval applies | Required when workstream approval applies | Required |
| Acceptance Criteria | Required aggregate criteria | Required workstream criteria | Required observable criteria |
| Validation | Required aggregate validation | Required aggregate workstream validation | Required proportionate validation |
| Stop Conditions | Required when applicable | Required when applicable | Required |
| Risks | Required | Required when applicable | Required when applicable |
| Completion Evidence / Report | Required aggregate evidence | Required workstream evidence | Required completion report |
| Pull Request Slicing | Owned by lower level; exceptions noted | Owned by Leaves; coordinate exceptions | Required |
| Milestone Assignment | Required policy | Required when applicable | Required; milestone or `None` |
| Deferred Work | Required; use `None` if absent | Required; use `None` if absent | Required when applicable |

Aggregate fields describe coordination rather than copying Leaf implementation
detail. Parent validation demonstrates the overall outcome; Child validation
demonstrates the workstream; Leaf validation demonstrates its executable
result. The Issue Model requires Leaves to identify validation; [Validation and
Completion Evidence](VALIDATION.md) defines general evidence, the active Leaf
adds task-specific requirements, and specialized contracts add domain evidence.

## Relationships and Discoverability

Relationships MUST use durable links in Issue bodies or maintainer comments.
A hierarchical Leaf should use a block equivalent to:

```markdown
## Parent hierarchy

- Parent: #<parent>
- Child: #<child>
- Milestone: `<milestone or None>`
```

A standalone Leaf should use:

```markdown
## Hierarchy

- Type: Standalone Leaf
- Milestone: `<milestone or None>`
```

The Parent lists Children and direct Leaves; each Child lists its Parent and
Leaves; each hierarchical Leaf links its Parent and Child. A relationship MUST
be understandable from at least the Leaf and its immediate owner. GitHub's
sub-Issue UI MAY supplement these links. Projects, bots, labels, dashboards,
or automation are not required. Update both directions in the same focused
planning activity when practical. Correct a missing reverse link as
coordination work; doing so grants no architecture approval.

## Dependencies and Blocking Order

Every non-trivial Issue MUST identify `Depends on`, `Blocks`, ordering
constraints, parallelizable work, approval prerequisites, and applicable
external-repository dependencies. Use `None` when absent.

- A **hard dependency** means work cannot safely begin or complete first.
- An **ordering preference** improves efficiency but is not required for
  correctness.
- **Parallel work** has independent scope and non-conflicting contracts.

Dependency links grant no authority. Assignment or milestone placement does
not remove a blocker. Implementation MUST NOT begin while a hard prerequisite
or approval is unresolved. Cycles are prohibited; if one appears, stop and
redesign the decomposition. Parents and Children expose cross-Leaf ordering,
Leaves MUST NOT infer hidden prerequisites from private chat, and completing a
dependency does not automatically close a dependent Issue.

## Pull Request Slicing

One Leaf SHOULD map to one coherent Pull Request, and one Pull Request SHOULD
close one Leaf. The Pull Request implements the complete approved result, links
the Leaf, reports validation accurately, and excludes unrelated changes.

### Multiple Pull Requests for one Leaf

This exception is justified only by independently reviewable ordered migration
steps, platform-specific evidence unsafe to combine, generated and authored
changes needing separate review, an Issue-level inseparable implementation and
follow-up unsafe to review together, or repository or review limits that make
one Pull Request impractical.

Before or during the first Pull Request, record the durable reason, planned
slices and order, intermediate-state safety, which Pull Request updates
contracts, which supplies final validation, and closure behavior. Earlier
Pull Requests use `Part of #<leaf>` or another non-closing link. No Pull Request
uses `Closes #<leaf>` until the Leaf is actually complete.

### One Pull Request for multiple Leaves

This is discouraged and allowed only when the Leaves are technically
indivisible, no unrelated responsibility is hidden, acceptance and validation
remain independently traceable, explicit maintainer approval is recorded, and
every closed Leaf is genuinely complete. Otherwise split the Pull Request.

### Pull Request without an Issue

Only the direct trivial-maintenance path in
[Contributing](../../.github/CONTRIBUTING.md) permits this. Non-trivial work
SHOULD have a Leaf contract before implementation.

## Scope Expansion

Classify discovered work before implementing it.

### In-scope clarification

This retains the desired result and owner, creates no new approval boundary,
and makes acceptance or validation more precise. Update the Leaf when needed
and durably record the reason.

### Correctness-required adjacent change

This MAY remain in the Leaf only when required for the approved result, within
the same coherent responsibility, explicitly reported and validated, and free
of a new architecture, API, compatibility, dependency, security, concurrency,
async, or unsafe boundary.

### New responsibility or contract change

Create or update a focused Issue before work that adds an owner, changes
architecture or a public or serialized contract, adds or materially changes a
dependency, changes security, introduces concurrency or async, proposes
`unsafe`, changes compatibility, creates a separate reviewable result, or
expands the Parent or Child workstream.

Never silently broaden the Pull Request, add unrelated cleanup, or change a
normative contract merely to ease implementation. Stop at unresolved approval
boundaries. Update Parent or Child coordination when its workstream changes,
create a new Leaf for a separable responsibility, and use an ADR when the
[ADR Process](../decisions/README.md) triggers one.

## Milestone Assignment

For the repository-foundation model, milestones track executable Leaves and
final validation. Standalone Leaves MAY be assigned directly. Coordination-only
Parents and Children normally are not assigned and remain discoverable through
hierarchy links. A final-validation Leaf is assigned because it produces the
milestone readiness decision. Assign an Issue only when its completion directly
contributes to the milestone outcome. Assignment grants no approval and removes
no dependency.

A future milestone MAY adopt another documented model only with durable
maintainer approval, an explicit and understandable assignment model, and no
coordination-only records obscuring executable progress. This document does
not define release policy.

Each repository owns its milestone assignment for cross-repository work; one
repository's milestone does not control another's Issue. Coordinating links MAY
describe the combined outcome. Vulnerability details MUST NOT be exposed for
milestone visibility; private security tracking applies until public recording
is safe.

## Completion and Closure

### Leaf completion

A Leaf is complete only when its result is achieved, acceptance criteria and
required validation are recorded, the final diff or durable non-code result is
reviewed, required documentation is updated, no blocker remains, requested
repository side effects are accurately recorded, follow-up work is separated,
and a durable completion report exists. A merged Pull Request normally closes
it with an accurate closing reference. Documentation-only and decision-only
Leaves may complete without production code. A multi-Pull-Request Leaf closes
only after all slices and final validation complete.

### Child completion

A Child closes only when required Leaves, workstream acceptance, and
workstream-level validation are complete; deferred or unresolved work is
explicit; and Leaf links and evidence are current. An implementation Pull
Request MUST NOT close a coordination Child unless it explicitly and genuinely
completes the entire Child contract.

### Parent completion

A Parent closes only when required Children and direct Leaves are complete,
aggregate acceptance and final validation are satisfied, risks and deferred
work are recorded, completion evidence links relevant Issues and Pull Requests,
and the milestone outcome is determined when applicable. Child completion is
necessary but not sufficient.

Use `Closes #<leaf>` only when that Pull Request completes the Leaf and `Part of
#<leaf>` or a normal link for intermediate work. Parent and Child completion
normally uses a final evidence comment or focused coordination update. Do not
accidentally close a Parent or Child from an unrelated Leaf Pull Request. No
automation is required.

## Architecture and Approval Boundaries

Hierarchy records work ownership, not decision authority. A Leaf proceeds only
when applicable architecture is governed by an approved normative contract,
explicitly approved in the Issue, recorded in an accepted ADR when required,
and updated in its authoritative document when the active rule changes.

[Maintainership](../governance/MAINTAINERSHIP.md) owns approval authority;
[Architecture Principles](../architecture/PRINCIPLES.md),
[Architecture Layers](../architecture/LAYERS.md), and
[Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md) own their
respective boundaries. The [Shared Agent Contract](../../AGENTS.md) and
[Implementation Agent Workflow](AGENT_WORKFLOW.md) govern agent behavior.

Agents MAY identify missing decisions, preserve evidence, propose alternatives,
implement approved scope, validate, and report. They MUST NOT select unresolved
architecture, invent ownership, approve dependencies or public contracts,
approve security exceptions or unsafe code, or mark their proposals accepted.
A Parent or Child MUST NOT create vague Leaves so implementation can discover
the design.

## Cross-Repository Work

Each repository retains its maintainers, authority, Issue contracts, Pull
Requests, validation, milestones, and releases. A coordinating Issue MAY state
a shared outcome, link repository-specific Issues, record ordering and
compatibility dependencies, and identify the source-of-truth repository for a
shared contract when explicitly approved.

It cannot approve or automatically close another repository's work, transfer
maintainer authority, treat one Pull Request as evidence for an unvalidated
repository, or impose this model where it has not been adopted. For a
cross-repository protocol change, create the focused contract or coordination
Issue in the repository owning the decision, create repository-specific
executable Leaves, state ordering and compatibility, obtain approval in every
authority domain, and record final integration evidence. Do not use one giant
cross-repository implementation Issue. This model chooses no protocol or
repository topology.

## Security-Sensitive Work

Public discoverability yields to the [Security Policy](../../SECURITY.md) and
[Secure Development](SECURE_DEVELOPMENT.md). For an active vulnerability:

- MUST NOT create a public Issue containing sensitive details;
- use GitHub Private Vulnerability Reporting and the Security Policy process;
- keep remediation records private or restricted where available;
- apply equivalent ownership, scope, approval, validation, and completion
  discipline without exposure;
- create a public follow-up only when safe and useful; and
- redact exploits, credentials, private data, and affected-user information,
  including revealing links from public records.

Hierarchy remains proportional: a focused fix may use one private executable
work item, while coordinated remediation may use private workstreams. Public
milestone visibility is not required. This document does not redefine the
security process.

## Repository Foundation Example

The live `v0.1.0 — Repository Foundation` tree is a justified three-level
example:

- [Parent #1](https://github.com/YT-TechDev/frontend-analysis/issues/1) —
  repository governance and agent execution foundations
  - [Child #2](https://github.com/YT-TechDev/frontend-analysis/issues/2) —
    security and vulnerability handling: [Leaf #8](https://github.com/YT-TechDev/frontend-analysis/issues/8),
    [Leaf #9](https://github.com/YT-TechDev/frontend-analysis/issues/9)
  - [Child #3](https://github.com/YT-TechDev/frontend-analysis/issues/3) —
    contribution and maintainer governance: [Leaf #10](https://github.com/YT-TechDev/frontend-analysis/issues/10),
    [Leaf #11](https://github.com/YT-TechDev/frontend-analysis/issues/11),
    [Leaf #12](https://github.com/YT-TechDev/frontend-analysis/issues/12),
    [Leaf #13](https://github.com/YT-TechDev/frontend-analysis/issues/13)
  - [Child #4](https://github.com/YT-TechDev/frontend-analysis/issues/4) —
    architecture documentation source of truth: [Leaf #14](https://github.com/YT-TechDev/frontend-analysis/issues/14),
    [Leaf #15](https://github.com/YT-TechDev/frontend-analysis/issues/15),
    [Leaf #16](https://github.com/YT-TechDev/frontend-analysis/issues/16),
    [Leaf #17](https://github.com/YT-TechDev/frontend-analysis/issues/17)
  - [Child #5](https://github.com/YT-TechDev/frontend-analysis/issues/5) —
    implementation-agent contracts: [Leaf #18](https://github.com/YT-TechDev/frontend-analysis/issues/18),
    [Leaf #19](https://github.com/YT-TechDev/frontend-analysis/issues/19),
    [Leaf #20](https://github.com/YT-TechDev/frontend-analysis/issues/20)
  - [Child #6](https://github.com/YT-TechDev/frontend-analysis/issues/6) —
    Issue slicing and validation governance: [Leaf #21](https://github.com/YT-TechDev/frontend-analysis/issues/21),
    [Leaf #22](https://github.com/YT-TechDev/frontend-analysis/issues/22)
  - [Child #7](https://github.com/YT-TechDev/frontend-analysis/issues/7) —
    repository foundation consistency audit: [Leaf #23](https://github.com/YT-TechDev/frontend-analysis/issues/23)

Parent #1 coordinates the repository outcome; Children #2 through #7 own
coherent workstreams; Leaves #8 through #23 are executable or final-validation
contracts. [Milestone #1](https://github.com/YT-TechDev/frontend-analysis/milestone/1)
tracks executable Leaves and final validation, not coordination Parents or
Children. Leaf #23 is final validation and begins after prerequisite Leaves;
it provides the readiness result before Rust bootstrap proceeds. This tree is
an example of justified decomposition, not a requirement for ordinary work.
No live Issue or milestone is modified by this example.

## Representative Classifications

Every scenario states structure and links, milestone treatment, approval,
Pull Request slicing, security treatment, and its proceed or stop condition.

| Scenario | Recommended structure and links | Milestone | Approval boundary | Pull Request slicing | Security handling | Proceed or stop |
| --- | --- | --- | --- | --- | --- | --- |
| One-line typo fix | Direct focused Pull Request; no hierarchy | None unless explicitly required | Ordinary maintainer review; no architecture approval | One Pull Request | Confirm no security impact | Proceed; stop if review reveals contract impact |
| New Core domain subsystem | Parent for a multi-domain repository outcome; Children only for real domain model, analysis, public-boundary, migration, or similar workstreams; executable linked Leaves | Executable Leaves and final validation, not coordination Issues | Resolve architecture, ownership, API, compatibility, concurrency, async, dependencies, and any ADR trigger first | One coherent Pull Request per Leaf by default | Apply security contracts without exposing sensitive data | Stop affected implementation until design and approvals are durable |
| Browser adapter | Standalone Leaf for one approved coherent adapter change; Parent/Child/Leaf for transport, normalization, compatibility, multiple engines, or multiple slices | Executable Leaves when part of the milestone | Adapter owns protocol detail; approve Core boundary and any protocol dependency first | Adapter-specific Leaves and tests; one Pull Request per Leaf by default | Apply network, input, and logging controls | Proceed only inside approved boundaries; do not choose a protocol library here |
| Cross-repository protocol change | Coordinating Parent or contract Issue in the protocol-owning repository; repository-specific linked Leaves | Repository-local only | Every affected repository retains approval; no transfer by link | Multiple repository-specific Pull Requests | Coordinate sensitive material privately | Stop until all required approvals and compatibility order are recorded |
| Security vulnerability fix | Smallest private executable item; private workstreams only if remediation needs them; no sensitive public Issue | Public visibility not required | Explicit security and maintainer approval | Focused private fix by default; coordinate slices privately | Security Policy route, private validation and disclosure, safe public follow-up only | Proceed only in the private process |
| Repository-foundation milestone | Parent #1, Children #2–#7, Leaves #8–#23 | Milestone #1 contains executable Leaves and final validation | Specialized contracts and Maintainership remain authoritative | One Pull Request per Leaf by default | Sensitive work stays on the private route | Leaf #23 gives final PASS or NO-GO; Rust bootstrap waits for foundation completion |
| One non-trivial documentation contract | Standalone Leaf; no artificial Parent or Child | Only when part of an explicit milestone | Maintainer approval for the normative contract | One coherent Pull Request | Check and route any sensitive content privately | Proceed after required approval is durable |
| Leaf discovers a second independent responsibility | Keep original Leaf; create a linked new Leaf and update owning Parent or Child where applicable | Assign the new Leaf only if it directly contributes | Classify and obtain any new approval independently | Separate Pull Requests with explicit dependency order | Keep sensitive discovery private when applicable | Stop scope expansion; resume each Leaf only within its contract |

## Review Checklist

- [ ] Is an Issue required, or is the direct trivial Pull Request sufficient?
- [ ] Is this one coherent executable responsibility?
- [ ] Would a standalone Leaf be clearer than hierarchy?
- [ ] Does a Parent own a real aggregate outcome?
- [ ] Does each Child own a distinct workstream?
- [ ] Are Leaf boundaries based on contracts rather than file counts?
- [ ] Are architecture and ownership resolved?
- [ ] Are dependencies and ordering explicit?
- [ ] Are milestone assignments limited to executable work and final validation?
- [ ] Does each Leaf normally map to one coherent Pull Request?
- [ ] Are Pull Request slicing exceptions justified?
- [ ] Is scope expansion classified?
- [ ] Are security-sensitive details private?
- [ ] Is the hierarchy understandable without AI chat context?
- [ ] Are completion and closure rules explicit?
- [ ] Is hierarchy depth no greater than necessary?

Completing this checklist does not approve the work.

## Deferred Process Automation

The following remain deferred: GitHub Projects configuration, sub-Issue
automation, automatic relationship synchronization, label taxonomy, milestone
automation, dependency bots, Issue generators, automatic Pull-Request-to-Leaf
enforcement, release planning, and cross-repository orchestration tooling.
Durable Issue links are sufficient for this repository today.

Issue completion uses [Validation and Completion Evidence](VALIDATION.md) while
this document retains hierarchy, slicing, Leaf-field, and closure ownership.
The [MIT License](../../LICENSE) remains unchanged.
