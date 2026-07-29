# Maintainership and Decision Authority

## Purpose and Scope

This document defines technical and repository-governance authority for
Frontend Analysis. It assigns accountability for decisions while keeping
contributors able to propose and implement focused changes. It applies to this
repository and remains applicable if the project later adds repositories or
presentation layers.

This contract does not define legal ownership, employment relationships,
organization-wide authority, a corporate structure, a formal voting system, a
detailed release procedure, Code of Conduct enforcement details, or security
vulnerability handling details. Contribution workflow is defined in
[Contributing to Frontend Analysis](../../.github/CONTRIBUTING.md). Conduct and
vulnerability matters follow the [Code of
Conduct](../../.github/CODE_OF_CONDUCT.md) and [Security
Policy](../../SECURITY.md). Security design and implementation requirements
follow [Secure Development](../development/SECURE_DEVELOPMENT.md).

## Current Maintainer

`YT-TechDev` is the current repository owner and maintainer of record. The
current maintainer has final repository decision authority within the
boundaries of approved project contracts.

This authority is accountable and evidence-based. It does not permit silent or
undocumented contract changes, and significant decisions require durable
rationale. The designation states current repository decision authority; it is
not a legal ownership declaration, an irrevocable or organization-wide role, a
restriction on adding maintainers, or a guarantee of availability or response
time.

## Maintainer Responsibilities

The maintainer is responsible for:

- protecting project purpose, browser independence, and architectural
  integrity;
- approving architecture, lifecycle, module or crate boundaries, dependency
  direction, and Core versus adapter ownership;
- reviewing public API, serialized-contract, deprecation, and compatibility
  impact;
- reviewing dependencies, toolchain choices, and supply-chain impact under
  Secure Development;
- approving security-sensitive changes, security contracts and exceptions,
  and any proposed `unsafe Rust`;
- approving release scope, versioning, tags, publication, release notes, and
  compatibility declarations without this document prescribing a release
  workflow;
- maintaining repository governance and community-health documents;
- reviewing validation evidence and deciding whether a Pull Request is ready
  to merge;
- documenting significant technical decisions and their rationale;
- preventing unrelated or speculative scope expansion; and
- performing the Code of Conduct responsibilities assigned to project
  maintainers by the adopted standard.

Maintainer approval does not replace the evidence, focused scope, validation,
or documentation required by the applicable repository contracts.

## Decision Classes and Approval Boundaries

The following table distinguishes decisions that require approval before
implementation from routine changes reviewed through the ordinary Pull Request
process.

| Decision class | Who may propose | Required approval | Required durable record |
| --- | --- | --- | --- |
| Trivial maintenance | Contributors and agents | No advance approval when the contributor guide permits a direct Pull Request; maintainer review is required before merge | The Pull Request normally records the change and validation |
| Scoped implementation within approved contracts | Contributors and agents | Maintainer Pull Request review before merge; prior approval is inherited only from the focused, approved scope | Approved Issue or contract plus the implementation Pull Request and validation evidence |
| Architecture and ownership | Anyone | Explicit maintainer approval in a focused Issue or approved architecture record before implementation | The approval, evidence, alternatives, boundaries, and rationale in that Issue or record |
| Public API and compatibility | Anyone | Explicit maintainer approval before implementation | Focused Issue or approved contract recording API and compatibility impact, plus the implementing Pull Request |
| Dependencies and toolchain | Anyone | Explicit maintainer approval before addition or selection | Focused Issue recording need and proportionate Secure Development review, plus the implementing Pull Request |
| Security and `unsafe Rust` | Anyone may propose; agents may only identify risks and alternatives | Explicit maintainer approval in a focused Issue before implementation | Focused Issue with the required security analysis, exception rationale, and validation plan; active vulnerability details remain private under the Security Policy |
| Releases and publication | Contributors and agents may prepare material only in approved release scope | Exclusive maintainer approval before tagging or publication | Approved release task and durable release or publication record |
| Repository governance and settings | Anyone | Explicit maintainer approval before implementation or settings changes | Focused Issue, approved governance change, or attributable maintainer decision, plus a Pull Request when files change |

### Trivial Maintenance

Typo fixes, broken links, isolated formatting corrections, and non-behavioral
documentation corrections may be proposed and implemented by contributors or
agents. A dedicated Issue may be unnecessary under the contributor guide.
Maintainer review remains required before merge, and the Pull Request normally
serves as the durable record.

### Scoped Implementation Within Approved Contracts

Contributors and agents may implement an approved Leaf Issue, an internal bug
fix that preserves established public behavior, or tests and documentation
required by an approved change. Work must remain inside the approved scope.
Maintainer Pull Request review is required. An unresolved architecture,
contract, or ownership question stops implementation at that boundary.

### Architecture and Ownership

Domain ownership and lifecycle, module or crate boundaries, Core versus adapter
responsibility, dependency direction, synchronous versus asynchronous
boundaries, concurrency ownership, and evidence-model ownership require
explicit maintainer approval before implementation. Anyone may propose these
decisions, but approval must be recorded in a focused Issue or an approved
architecture record. A Pull Request cannot silently introduce the decision.
This document does not select the architecture.

### Public API and Compatibility

New public APIs; changes to public types or traits; serialized formats;
protocol-neutral domain contracts; compatibility policy; and deprecation or
removal require explicit maintainer approval before implementation. The durable
record must describe compatibility impact. Accidental public exposure and the
mere existence of implementation do not constitute approval.

### Dependencies and Toolchain Decisions

New runtime or development dependencies, major dependency updates, parser or
async-runtime selection, and toolchain or minimum-supported-Rust-version
decisions require explicit maintainer approval before addition or selection.
Review must follow Secure Development and document demonstrated need and
material supply-chain impact. Speculative dependencies are prohibited. This
contract selects no dependency, runtime, or toolchain version.

### Security and Unsafe Rust

Security-contract changes and exceptions, credential and network trust
boundaries, sandbox or process execution, and `unsafe Rust` require explicit
maintainer approval in a focused Issue before implementation. `unsafe Rust`
remains prohibited without its required documented exception. Implementation
agents cannot grant approval. Active vulnerability details follow the Security
Policy rather than a public record.

### Releases and Publication

Version selection, release scope, Git tags, package or artifact publication,
release notes, and compatibility declarations are exclusively approved by
maintainers. Agents may prepare artifacts only within an approved release task
and cannot decide that a release should occur. This document does not establish
a detailed release process.

### Repository Governance and Settings

Branch protection, merge strategy, CI permissions, security settings,
repository templates, governance contracts, and maintainer access require
maintainer approval. Implementation tools must not change repository settings
or governance files without explicit scope and approval.

## Contributors

Contributors may identify problems, provide evidence, propose designs, compare
alternatives, create focused Issues, implement approved changes, review Pull
Requests, and document risks and trade-offs. Human contributors remain
responsible for changes they submit, including agent-assisted changes.

Maintainer authority is not acquired merely through contribution count,
authorship of an Issue or Pull Request, operating an implementation agent,
maintaining a fork, or informal agreement outside durable repository records.
A proposal is not approved merely because implementation has begun.

## Implementation Agents

Codex, Claude Code, and other implementation agents are execution tools, not
maintainers. They may inspect approved context, implement scoped changes, run
validation, report evidence, identify ambiguity, propose alternatives, and
prepare Pull Requests and completion reports.

Agents must not approve architecture, ownership boundaries, public APIs,
compatibility policy, dependencies, security exceptions, `unsafe Rust`,
releases, maintainer appointments, or repository-governance changes. An
agent-generated statement such as "approved," "safe," or "ready to release"
is not maintainer approval. An agent must stop at and escalate a boundary when
the required authority is absent.

## Valid Approval

Valid maintainer approval must be explicit, attributable to a current
maintainer, attached to the relevant decision, recorded in a durable repository
location, and issued before implementation when prior approval is required.
Valid locations include:

- a focused GitHub Issue or Issue comment;
- an approved Pull Request review;
- an approved governance or architecture document; or
- a future approved architecture decision record.

Approval is not established by silence, lack of objections, an agent's
conclusion, a private AI conversation, an unrecorded verbal discussion,
implementation already existing, an unrelated Pull Request merge, an Issue
merely being opened, or a contributor approving their own proposal without
maintainer authority.

For routine implementation that requires no prior design approval, the
maintainer's final Pull Request review and merge decision may approve that
scoped implementation. It cannot retroactively authorize an undisclosed
architecture or security change.

## Architecture and Contract Changes

An approved contract may change only through a proposal that:

1. identifies the existing contract;
2. explains the motivation;
3. provides relevant evidence;
4. describes compatibility and migration impact;
5. receives explicit maintainer approval;
6. updates the authoritative document when its normative contract changes; and
7. is implemented through focused scope.

An Issue or Pull Request must not silently weaken architecture boundaries,
security requirements, compatibility commitments, validation expectations, or
contributor responsibilities.

## Conflict Resolution and Escalation

When a technical decision is unresolved:

1. stop implementation at the unresolved boundary;
2. identify the relevant existing contract;
3. describe the disagreement or ambiguity;
4. collect evidence and realistic alternatives;
5. document trade-offs, risks, and compatibility impact;
6. request maintainer review;
7. record the maintainer decision;
8. update authoritative documentation when required; and
9. resume implementation only within the approved decision.

Evidence and long-term maintainability take priority over implementation
speed. Majority preference does not automatically decide architecture, and
implementation progress does not create authority. Repeated argument without
new evidence does not require immediate scope expansion. The maintainer makes
the final repository decision and documents significant rationale.

Conduct-related conflicts remain governed by the Code of Conduct. Security
vulnerabilities remain governed by the Security Policy.

## Durable Decision Records

Significant decisions must remain understandable without private ChatGPT or
Claude conversations, unshared meeting context, personal memory, or
undocumented assumptions. A durable record must contain enough information to
understand:

- the problem and decision;
- important alternatives;
- constraints and invariants;
- material trade-offs;
- compatibility and, where relevant, security impact;
- validation expectations; and
- follow-up work.

A full architecture decision record is not required for every trivial change.
Until a formal architecture-decision-record process is approved, focused
Issues, Pull Requests, and approved documentation are valid durable records.

## Authority Scenarios

These outcomes apply the preceding rules deterministically. "Evidence" below
means durable evidence in the named record, not private or transient context.

| Scenario | Who may proceed and whether work stops | Approver and approval record | Required durable evidence |
| --- | --- | --- | --- |
| 1. An external contributor fixes a documentation typo | The contributor may implement directly; work need not stop if no contract is affected | Maintainer review before merge; the Pull Request is the record | Focused diff and proportionate documentation validation |
| 2. An agent implements an approved Leaf Issue without changing contracts | The agent may implement within scope; stop only if an unresolved boundary appears | Maintainer Pull Request review; the approved Issue and Pull Request are the records | Scope conformance, final diff, validation results, and disclosed limitations |
| 3. A contributor proposes a runtime dependency | The contributor may investigate and propose; addition must not begin before approval | Maintainer before implementation, in a focused Issue | Current need, alternatives, maintenance and security history, transitive and license impact, compatibility, API exposure, and replacement cost as required by Secure Development |
| 4. A contributor proposes a public Rust API | The contributor may propose; implementation stops until approval | Maintainer before implementation, in a focused Issue or approved public contract | API purpose, alternatives, ownership, compatibility, deprecation implications, risks, and validation plan |
| 5. A browser adapter concept begins leaking into Core | Anyone may identify the leak; implementation stops at the ownership boundary | Maintainer in a focused architecture Issue or approved architecture record | Existing browser-independence contract, concrete leak, alternatives, dependency direction, trade-offs, and compatibility impact |
| 6. An optimization proposes `unsafe Rust` | Safe alternatives and evidence may be investigated; `unsafe` implementation stops | Maintainer before implementation, in a focused Issue | Every exception item required by Secure Development, including safety invariants and benchmark evidence for a performance justification |
| 7. An implementation agent decides a release is ready | The agent may report validation or prepare material only under an approved task; release action stops | Maintainer through an approved release task and durable release or publication record | Approved scope, version and compatibility decision, validation, artifacts, notes, and known risks |
| 8. A contributor begins implementing an unapproved architecture proposal | The contributor may preserve exploratory evidence, but repository implementation stops | Maintainer before implementation, in a focused Issue or approved architecture record | Problem, current contract, alternatives, boundaries, trade-offs, compatibility impact, and validation expectations |
| 9. A maintainer tries to weaken a security contract in an unrelated Pull Request | No participant may treat the unrelated change as approval; implementation stops and scope is separated | Explicit attributable maintainer approval before implementation, in a focused Issue; the authoritative security document must be updated if approved | Existing contract, motivation, threat or abuse analysis, alternatives, residual risks, compatibility impact, and validation plan; active vulnerability details remain private |
| 10. A regular contributor with many valuable contributions requests maintainer status | The contributor may request consideration; no maintainer authority exists yet and ordinary contributions may continue | Existing maintainer, recorded publicly with authority scope before access is granted | Project need, appointment decision, defined responsibility and authority scope, access alignment, and any material governance update |
| 11. Ownership ambiguity appears during implementation | Contributors or agents document it; implementation stops only at the ambiguous boundary | Maintainer in a focused Issue, Issue comment attached to the approved task, or approved architecture record before affected work resumes | Relevant contract, ambiguous ownership and lifecycle, realistic alternatives, dependency and compatibility effects, risks, and decision rationale |
| 12. Two contributors prefer different architectures without new evidence | They may summarize existing positions; disputed implementation stops, and repeated argument does not expand scope | Maintainer in the focused Issue or approved architecture record | Existing contract, available evidence, alternatives, long-term maintenance trade-offs, risks, compatibility impact, and reasoned final decision |

## Code of Conduct Responsibilities

Under the [Code of Conduct](../../.github/CODE_OF_CONDUCT.md), project
maintainers perform the Community Moderator and Community Manager
responsibilities described there. That document remains authoritative for
conduct expectations, reporting, investigation, enforcement, and
confidentiality.

## Adding Future Maintainers

Additional maintainers may be added when project needs justify shared
authority. An existing maintainer must explicitly approve the appointment, and
the appointment and authority scope must be recorded publicly. Repository
access must match the documented responsibility. Authority may be
repository-wide or limited to a clearly defined domain. This document must be
updated when the authority model materially changes.

Maintainer status is not automatically acquired through contribution volume or
tool access. This path does not establish voting thresholds, elections,
mandatory committees, fixed contribution requirements, tenure, compensation,
legal ownership transfer, or detailed removal or succession procedures.

## Deferred Governance Decisions

The following matters remain deferred until a concrete need is addressed in a
focused Issue with explicit maintainer approval:

- formal voting rules;
- maintainer election procedures;
- maintainer removal and succession procedures;
- organization-wide governance;
- legal ownership transfer;
- repository-wide `CODEOWNERS` enforcement;
- detailed release management;
- a formal appeals process;
- role-specific service-level expectations; and
- guaranteed response times.

Deferred does not mean unrestricted. None of these mechanisms or commitments
may be introduced without the required focused and approved decision.
