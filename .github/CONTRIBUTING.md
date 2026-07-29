# Contributing to Frontend Analysis

## Welcome

Frontend Analysis is a browser-independent platform for analyzing, visualizing,
and diagnosing modern web applications. Contributions from new and returning
contributors are welcome, whether they are written directly or with assistance
from development tools. We prioritize maintainability, explicit boundaries, and
reviewable evidence. Small, focused contributions are generally preferable to
broad rewrites.

## Before You Start

- Search existing Issues and Pull Requests and confirm that similar work is not
  already active.
- Read only the repository documentation relevant to the proposed change.
- Identify the problem and desired result before writing code.
- Do not begin broad implementation while an architectural question remains
  unresolved.

A typo or other trivial correction does not need advance permission when it
meets the direct Pull Request criteria below.

## Choose the Right Workflow

### Trivial, self-contained maintenance

Typo corrections, broken links, minor formatting fixes, clearly isolated
documentation corrections, and mechanical changes with no behavioral or
contract impact may use a direct, focused Pull Request without a dedicated
Issue when:

- the scope is obvious;
- no architecture, API, security, dependency, or compatibility decision is
  involved; and
- the change can be validated independently.

### Non-trivial changes

Open a focused Issue before implementation when work affects architecture or
layer boundaries, domain ownership or lifecycle, public APIs, serialized
formats, compatibility, dependencies, security boundaries, untrusted input,
browser adapters, concurrency or asynchronous behavior, `unsafe Rust`, multiple
modules or contract domains, or behavior that requires design discussion.

Complex work may be decomposed into coordinated Issues when one Pull Request
would otherwise mix distinct responsibilities. The detailed decomposition
model is deferred to the relevant approved Issue.

## Architecture-First Changes

Implementation must follow approved architecture and ownership boundaries.
Contributors must not:

- move browser-specific protocol concepts into Core;
- introduce reverse dependencies from Core to UI or browser implementations;
- create public APIs accidentally;
- resolve missing ownership through arbitrary cloning or shared mutation;
- introduce speculative abstractions; or
- select dependencies, runtimes, or frameworks merely to complete a task.

When architecture is unresolved, document the question and relevant evidence
or alternatives, stop implementation at that boundary, and request maintainer
review. Detailed architecture policy remains deferred to the relevant approved
Issue.

## Issues and Scope

A non-trivial Issue should provide enough context for review:

- the problem or motivation and desired result;
- in-scope and out-of-scope behavior;
- constraints and invariants;
- compatibility or public-contract impact;
- validation expectations; and
- known risks.

An Issue defines implementation scope but cannot silently override existing
repository contracts. During implementation, unrelated cleanup is prohibited
by default. Report scope expansion; additional work should normally receive a
separate Issue. Do not add future infrastructure speculatively, and stop when a
required decision remains unresolved.

## Branches and Pull Requests

The normal contribution flow is:

1. Create or use a focused branch.
2. Make the smallest coherent change.
3. Inspect the final diff.
4. Run relevant validation.
5. Open a focused Pull Request against `main`.
6. Resolve review conversations.
7. Merge only after repository requirements are satisfied.

Direct changes to `main` are not the normal contribution path. One Pull Request
should normally represent one coherent leaf-sized or focused change; do not
bundle unrelated changes. Pull Request titles and descriptions must explain the
durable technical result. Draft Pull Requests may be used before work is ready
for final review. Final Pull Requests must not claim checks that were not run.
Merge strategy and repository settings remain controlled by maintainers.

## Implementation Requirements

Contributors must:

- follow the active Issue when one is required;
- preserve existing behavior unless the change explicitly modifies it;
- use the narrowest useful visibility;
- preserve browser independence;
- keep ownership and mutation explicit and avoid global mutable state;
- avoid speculative concurrency, asynchronous boundaries, and unnecessary
  abstractions;
- document public behavior and non-obvious invariants;
- keep changes deterministic where practical;
- add or update tests when behavior changes; and
- never weaken tests merely to make a change pass.

All changes must also follow [Secure Development](../docs/development/SECURE_DEVELOPMENT.md).

## Validation and Documentation

Until the repository-wide validation contract is approved, every contributor
must:

- inspect the final diff;
- run validation appropriate to the changed files and behavior;
- update documentation when contracts or behavior change;
- accurately record commands and manual checks; and
- report skipped, unavailable, blocked, or failing validation rather than
  claiming checks that were not run.

For documentation-only changes, validation should include, as applicable,
Markdown rendering review, relative-link validation, spelling and terminology
review, scope review, and checks for stale or contradictory statements.

For future Rust changes, baseline validation categories include formatting,
linting, tests, build or type validation, and documentation validation. Add
checks proportional to API, security, platform, or compatibility impact. Use
the commands available in the repository at that time rather than inventing or
claiming unavailable checks.

## AI-Assisted Contributions

AI-assisted contributions are allowed, but no contributor is required to use
AI. The human contributor or operator remains responsible for scope,
architecture compliance, correctness, licensing, security, validation, review
of generated changes, and durable technical explanations.

A Pull Request should concisely disclose meaningful AI assistance when it
affects implementation or review provenance. Model marketing, complete prompt
transcripts, private ChatGPT or Claude links, conversation exports, and
unrelated personal usage data are not required. Durable records must contain
the technical change, validation evidence, known limitations, and unresolved
risks; they must not depend on private AI conversation history.

Implementation agents cannot approve architecture changes, public API changes,
security exceptions, new dependencies, `unsafe Rust`, compatibility policy, or
releases.

## Security-Sensitive Changes

Security-sensitive work must follow both the [Security Policy](../SECURITY.md)
and [Secure Development](../docs/development/SECURE_DEVELOPMENT.md). It requires
a focused Issue, explicit maintainer review, evidence proportional to the risk,
and private handling of active vulnerability details.

## Review and Completion

Maintainer review evaluates at least correctness, scope, architecture
boundaries, ownership and type contracts, compatibility, security, tests and
validation, documentation, and maintenance cost. A contribution is not
complete merely because it compiles or renders.

A completion record should state:

- files or behavior changed;
- validation performed and checks not run;
- deviations from the Issue; and
- unresolved risks or follow-up work.

Merge requires maintainer approval and satisfaction of repository requirements.

## Reporting Security Vulnerabilities

Suspected vulnerabilities must not be reported in public Issues, Pull Requests,
or Discussions. Follow the private reporting process in the
[Security Policy](../SECURITY.md).
