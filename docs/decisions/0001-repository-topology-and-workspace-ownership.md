# ADR 0001: Repository topology and workspace ownership

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-07-30 |
| Decision owner / approver | YT-TechDev |
| Linked Issue | [#41](https://github.com/YT-TechDev/frontend-analysis/issues/41) |
| Related Pull Request | [#47](https://github.com/YT-TechDev/frontend-analysis/pull/47) |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | None — existing architecture contracts already define the required browser-independent and layer boundaries while deliberately deferring workspace and crate structure; this ADR records repository ownership and bootstrap topology without changing those invariants. |

## Context

The project needs a repository owner and a minimal workspace shape before a
Rust Core can be bootstrapped. Choosing that initial location is significant:
repository placement affects governance, contributor discovery, change
coordination, CI, releases, security ownership, and the cost of a later move.
It must not implicitly choose crate boundaries or the final placement of Core,
Browser Adapters, products, or presentation code.

The current repository already contains the accepted governance, architecture,
security, contribution, and validation contracts for Frontend Analysis.
`YT-TechDev` is its maintainer of record. The architecture contracts require a
browser-independent Core, keep browser protocols in Browser Adapters, preserve
the semantic authority of Core and Analysis Results, and deliberately leave
workspace and crate structure undecided. They also apply only to repositories
that explicitly adopt them; a future repository does not inherit them merely
because it contains related code.

There is currently no Rust API, ABI, serialized format, package, crate, browser
support commitment, Minimum Supported Rust Version (MSRV), Cargo workspace, or
production implementation to migrate. The next bootstrap work therefore needs
a narrow topology decision without inventing a production boundary.

Cargo supports a root manifest containing a `[workspace]` table without a root
package. This virtual-workspace model provides the intended workspace-level
coordination point without requiring a placeholder crate. For this proposal,
the initial membership is deliberately empty: zero production members is a
transition state until a separately planned and approved Core-domain milestone
selects the first meaningful crate.

Decision drivers are:

- reuse the repository's current governance and architecture contracts;
- keep the bootstrap incremental, atomic, and reviewable;
- avoid premature repository and package boundaries;
- avoid representing a temporary starting point as a permanent monorepo;
- make a future extraction review depend on objective evidence; and
- preserve an explicit approval boundary between this proposal and all Rust
  implementation.

## Decision

This ADR selects **Option C**.

`YT-TechDev/frontend-analysis` initially owns the Rust Core
workspace and its workspace-level contracts. The repository's current role is
initially **Core-focused**: it provides the governance and tooling foundation
for the browser-independent Core while the first production domain boundary is
still deferred.

The bootstrap workspace model is a root virtual Cargo workspace with zero
production members. No new repository is required for bootstrap. Zero members
is an intentional transition state, not a permanent product topology, and a
placeholder crate must not be introduced merely to satisfy tooling.

This initial ownership does **not** approve a permanent monorepo. It does not
decide that Core, Browser Adapters, products, and presentation must remain in
one repository. The temporary role is explicit so later placement can follow
evidence rather than inertia. A repository extraction trigger starts a new ADR
review; it never authorizes extraction by itself.

At minimum, an extraction review is triggered when objective evidence shows
one or more of the following:

1. a non-Core production component requires an independent release, version,
   or support cadence;
2. a demonstrated repository-level security, permission, or CI ownership
   boundary requires separation;
3. cross-repository consumers require an independently versioned Core
   publication; or
4. repository or CI scale materially couples otherwise unrelated release
   cycles.

Any future extraction decision must use a focused ADR and define:

- target repository ownership;
- explicit governance adoption;
- private security-reporting arrangements;
- protocol and domain authority;
- version coordination;
- release responsibility;
- contributor workflow;
- history-preserving migration;
- compatibility commitments; and
- consumer migration.

Future repositories do not automatically inherit this repository's governance,
architecture, security, validation, or contribution contracts. Adoption must
be explicit in each target repository.

### Decision Boundaries and Deferrals

This ADR does not approve:

- permanent monorepo status;
- creation of another repository;
- an initial crate or package name;
- Cargo or Rust implementation;
- production domain models;
- public Rust APIs;
- serialized formats;
- parser or browser protocol libraries;
- Browser Adapter repository placement;
- CLI, desktop, VS Code, web, or presentation repository placement;
- an async runtime;
- a concurrency model;
- IPC;
- WASM;
- FFI;
- unsafe implementation;
- any third-party dependency;
- `Cargo.lock` policy;
- MSRV;
- package publication;
- release automation; or
- stable compatibility promises.

The first meaningful Core crate is deferred to a separately planned and
approved Core-domain milestone. Issue #42 owns bootstrap toolchain and
validation policy only after this ADR is accepted. Issue #43 owns only the
approved minimal scaffold after Issues #41 and #42. Future focused ADRs own any
repository extraction or product and adapter placement decision.

## Alternatives Considered

### Option A — permanent workspace or monorepo in the current repository

This option would establish the current repository as the permanent home for
Core, Browser Adapters, products, and presentation.

Benefits include one governance authority, straightforward contributor
discovery, and atomic cross-component changes. Shared CI can initially make
cross-layer validation convenient, and retaining all history in place avoids
near-term migration work.

The costs grow as components mature. A single repository can couple Core,
adapters, products, and presentation through shared CI and review surfaces even
when architecture forbids source-dependency coupling. Independent release and
support cadences, package publication, and product-specific permissions or
security ownership become harder to isolate. Repository and CI scale can make
unrelated changes wait on each other. Extracting later may be expensive, but
declaring permanence now would make that cost and inertia greater.

Option A is not proposed because there is no production evidence supporting a
permanent monorepo boundary. It would turn a minimal bootstrap need into an
unnecessary long-term commitment.

### Option B — umbrella/governance repository with Rust Core elsewhere

This option would retain `YT-TechDev/frontend-analysis` as an umbrella and
governance repository while creating or selecting a separate repository to own
Rust Core immediately.

Benefits include an independent Core lifecycle, CI boundary, release cadence,
and version authority from the beginning. A separate Core repository could make
package publication, protocol/version responsibility, and Core-specific
permissions clearer once real consumers and maintainers exist.

Costs include immediate cross-repository coordination, split contributor
discovery, and either governance duplication or explicit adoption work before
production boundaries are known. Protocol and version authority would have to
be defined now. Early atomic changes would span repositories, and the project
would incur repository setup, history, migration, security-reporting, and
maintenance costs without evidence that separation is needed.

Option B is not proposed because it prematurely fragments a project that has no
production crate or independent consumer. Its potential benefits are retained
as objective triggers for a later extraction review.

### Option C — Core-focused workspace here with extraction-review triggers

This option initially places a Core-focused workspace in the current
repository while defining evidence that requires the topology to be reviewed.

Benefits include reuse of accepted governance, early atomic changes, clear
contributor discovery, and avoidance of a premature repository or speculative
crate. It permits incremental bootstrap while making independent release,
security, consumer-versioning, and CI-scale needs explicit review inputs.

Costs include the risk that temporary placement is misread as permanent, and
the possibility of later history and CI migration. Core and future non-Core
planning may temporarily share Issue and repository surfaces. Preventing
inertia requires maintainers to apply the triggers and keep deferred placement
decisions out of bootstrap work.

Option C is selected because it meets the current need with the fewest
unsupported commitments. Its temporary role, objective triggers, and required
future ADR provide a bounded path to separation when evidence exists.

## Consequences

### Positive

- There is one current governance authority for the bootstrap.
- Premature cross-repository coordination is avoided.
- No speculative package or crate boundary is created.
- Extraction review has explicit, objective gates.
- Bootstrap can proceed incrementally in focused, reviewable changes after
  approval.

### Negative

- The provisional repository role requires continuing discipline.
- Future extraction may require non-trivial history and CI migration.
- Core and future non-Core work may temporarily share Issue and repository
  surfaces.
- A zero-member workspace provides a tooling foundation but no production
  behavior.

### Risks

- **Option C is misread as permanent monorepo approval.** Mitigation: state the
  temporary Core-focused role in the decision, index, and downstream bootstrap
  review; reject permanent-topology claims that cite this ADR.
- **Temporary topology becomes permanent by inertia.** Mitigation: require a
  new ADR review when any objective extraction trigger is evidenced and assess
  triggers during relevant release, security, permission, and CI changes.
- **A future repository receives incomplete governance or security setup.**
  Mitigation: require explicit governance adoption and a private vulnerability
  reporting path before migration.
- **Extraction loses history or compatibility context.** Mitigation: require a
  history-preserving migration plan where practical and define compatibility,
  version, governance, and consumer migration before execution.
- **A placeholder crate is introduced to satisfy tooling.** Mitigation: require
  zero production members during bootstrap and defer the first crate to the
  Core-domain milestone.

### Reversibility

The initial placement is reversible, but not through silent repository
movement. Reversal requires a new accepted ADR, satisfaction of an objective
review trigger, and a planned migration covering ownership, governance,
security reporting, authority, versions, releases, contributors, history,
compatibility, and consumers. Until that process is complete, the current
repository remains the accepted initial workspace owner.

## Compatibility and Migration

No existing Rust API, ABI, serialized format, package, crate, browser support
commitment, or MSRV exists. No consumer migration is required now. Runtime
semantics, ordering and determinism commitments, and the meaning of evidence,
certainty, findings, and diagnostics are unchanged. No product or Browser
Adapter placement changes now.

Option C creates a possible future migration but performs no extraction. A
future extraction must preserve repository history where practical and define
compatibility, version coordination, governance adoption, and consumer
migration before execution. It must not silently change protocol-neutral domain
authority or Analysis Result meaning.

## Security and License Impact

This proposal changes no dependency, credential, executable code, repository
permission, security setting, or vulnerability-reporting configuration. The
[Secure Development contract](../development/SECURE_DEVELOPMENT.md) and private
[Security Policy](../../SECURITY.md) remain authoritative. No active
vulnerability details belong in this ADR.

The [MIT License](../../LICENSE) remains unchanged. A future repository split
must establish an explicit private vulnerability-reporting path and explicitly
adopt the governance and security contracts it needs; neither transfers
automatically.

## Validation

Acceptance of this ADR does not claim that downstream bootstrap checks have
passed. Before implementation, and again in the final bootstrap PASS/NO-GO
audit, evidence must confirm:

- a current maintainer has explicitly and durably accepted this ADR;
- root workspace ownership matches this ADR;
- no documentation or implementation claims permanent monorepo approval;
- the root virtual workspace has zero production members during bootstrap;
- no placeholder crate exists;
- no new repository was created for bootstrap;
- no browser protocol or presentation type entered Core;
- no implementation began before the approvals required by Issues #41 and #42;
- Issue #42's approved toolchain and validation policy, when available, is
  followed; and
- the final bootstrap audit records an explicit PASS or NO-GO with the evidence
  and any unavailable checks required by the repository validation contract.

Documentation review of this accepted record must also confirm the ADR number,
required sections, relative links, Accepted status, durable approval evidence,
stated deferrals, unchanged normative architecture invariants, and absence of
private AI links, transcripts, credentials, implementation, or fabricated
approval.

## Follow-Up

- ADR 0001 is accepted through the durable maintainer approval recorded in
  Issue #41. Issue #42 may proceed only after this ADR and index update are
  merged.
- Issue #42 owns bootstrap toolchain and validation policy after this ADR and
  index update are merged.
- Issue #43 may implement only the approved minimal scaffold after Issues #41
  and #42.
- A separately planned and approved Core-domain milestone owns the first
  meaningful crate and domain decision.
- Future focused ADRs own repository extraction and product or Browser Adapter
  placement decisions.

## Approval

Approved by `YT-TechDev`, the current maintainer of record, on 2026-07-30.

Durable approval:
[Issue #41 maintainer architecture decision](https://github.com/YT-TechDev/frontend-analysis/issues/41#issuecomment-5128242841)

The approval is decision-specific and accepts Option C, the initial
Core-focused repository role, the zero-production-member virtual-workspace
transition state, the objective extraction-review triggers, and all stated
boundaries and deferrals. Any substantive change requires renewed maintainer
review.

## References

- [Issue #41: repository topology and workspace ownership](https://github.com/YT-TechDev/frontend-analysis/issues/41)
- [Issue #41 maintainer architecture decision](https://github.com/YT-TechDev/frontend-analysis/issues/41#issuecomment-5128242841)
- [Pull Request #47](https://github.com/YT-TechDev/frontend-analysis/pull/47)
- [Parent Issue #40](https://github.com/YT-TechDev/frontend-analysis/issues/40)
- [Issue #42](https://github.com/YT-TechDev/frontend-analysis/issues/42)
- [Issue #43](https://github.com/YT-TechDev/frontend-analysis/issues/43)
- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Maintainership and Decision Authority](../governance/MAINTAINERSHIP.md)
- [Architecture Principles](../architecture/PRINCIPLES.md)
- [Architecture Layers and Boundaries](../architecture/LAYERS.md)
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md)
- [Validation and Completion Evidence](../development/VALIDATION.md)
- [ADR Process](README.md)
