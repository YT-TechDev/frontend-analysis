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

Non-trivial work should use the proportional hierarchy in the
[Issue Model](../docs/development/ISSUE_MODEL.md). Use a standalone executable
Leaf when one coherent responsibility needs durable scope; add Parent and Child
Issues only when they provide real coordination or workstream ownership. Do not
create them when a standalone Leaf—or the direct trivial Pull Request path
above—is sufficient. Before implementation, an executable Leaf must resolve
applicable design and approval boundaries.

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

Maintainer responsibilities and approval boundaries are defined in
[Maintainership and Decision Authority](../docs/governance/MAINTAINERSHIP.md).

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

## Current Rust Workspace Validation

`rustup` is a prerequisite. The repository toolchain file selects Rust `1.97.1`
with rustfmt and Clippy. Prepare that exact toolchain when it is not already
installed:

```bash
rustup toolchain install 1.97.1 \
  --profile minimal \
  --component rustfmt \
  --component clippy
```

Run the current production Rust baseline locally:

```bash
python3 .github/scripts/validate-rust-workspace-state.py .
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo metadata --offline --format-version 1 --locked
```

The validator must print `production`. Metadata must report exactly one package
and workspace member: `frontend-analysis-core` at its approved manifest path,
with zero dependencies and only its approved library target. `Cargo.lock` is
committed, and metadata validation uses `--locked`. Formatting, Clippy, and
tests are applicable to the current source. Returning the workspace to zero
members is a policy failure.

Do not treat the toolchain pin as an MSRV. A toolchain change requires a focused
Issue and Pull Request, official release review, validation, and maintainer
approval. Passing CI does not authorize API, dependency, architecture, unsafe,
parser, browser, serialization, or release changes. [ADR 0001](../docs/decisions/0001-repository-topology-and-workspace-ownership.md)
owns topology, [ADR 0002](../docs/decisions/0002-rust-bootstrap-toolchain-and-validation-policy.md)
owns the toolchain policy, and [Validation and Completion Evidence](../docs/development/VALIDATION.md)
owns validation applicability and evidence.

## Validation and Documentation

Every contribution must follow [Validation and Completion
Evidence](../docs/development/VALIDATION.md). The active Issue and affected
specialized contracts may require additional evidence. Contributors must:

- inspect the final diff;
- run validation appropriate to the changed files and behavior;
- update documentation when contracts or behavior change;
- accurately record commands and manual checks; and
- report Failed, Blocked, unavailable, Not run, and Not applicable checks
  honestly rather than claiming checks that were not performed.

Documentation-only changes use the contract's documentation baseline. Rust
source-level work uses the current Rust baseline plus any change-class checks
selected by the active Issue and affected contracts. Passing validation does
not grant architecture, API, dependency, security, compatibility, `unsafe`, or
release approval.

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
