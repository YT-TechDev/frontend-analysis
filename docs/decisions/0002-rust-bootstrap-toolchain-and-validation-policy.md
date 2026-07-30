# ADR 0002: Rust bootstrap toolchain and validation policy

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-07-30 |
| Decision owner / approver | YT-TechDev |
| Linked Issue | [#42](https://github.com/YT-TechDev/frontend-analysis/issues/42) |
| Related Pull Request | [#48](https://github.com/YT-TechDev/frontend-analysis/pull/48) |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | None at this decision stage — existing maintainership, architecture, Rust Core, security, and validation contracts already require explicit approval and evidence while deliberately selecting no toolchain or workspace implementation. This ADR defines the bootstrap policy within those boundaries. Issue #44 may later update contributor-facing documentation to reflect accepted commands without weakening the owning contracts. |

## Context

[ADR 0001](0001-repository-topology-and-workspace-ownership.md) establishes
this repository as the initial Core-focused Rust workspace owner. It selects a
root virtual workspace with zero production members as a temporary bootstrap
transition, without approving a permanent monorepo, crate, package, API,
dependency, runtime, publication, release, or product boundary. This decision
preserves that topology decision rather than reopening it.

The repository currently has no Cargo manifest, Rust source, toolchain file,
workflow, package, dependency graph, or `Cargo.lock`. Bootstrap nevertheless
needs a reproducible development and CI baseline, an honest validation meaning,
and explicit boundaries that prevent tooling setup from inventing production
architecture.

Issue planning named Rust `1.97.0`. Official Rust release evidence current on
2026-07-30 identifies `1.97.1`, published on 2026-07-16 to address an LLVM
optimization miscompilation by backporting an LLVM fix and disabling the
`1.97.0` IR change that increased exposure. The point release therefore
supersedes the stale planning candidate.

Cargo documents virtual workspaces and requires an explicit resolver when no
root package edition exists. Edition 2024 implies resolver 3 for packages, but
a virtual root must still say `resolver = "3"`. The Cargo Book describes a
workspace as one or more packages and does not broadly guarantee a permanent
empty workspace, although Cargo's regression inputs include an empty-members
manifest. Zero-member bootstrap therefore needs proof against the exact pinned
Cargo version, not an assumption or a placeholder crate.

The decision drivers are reproducibility, meaningful non-vacuous evidence,
least privilege, no speculative production contract, contributor clarity,
maintainable toolchain ownership, and reversibility.

## Decision

This ADR records the accepted bootstrap policy. Acceptance authorizes only the
downstream work explicitly assigned to Issues #43 through #46 and does not
itself implement or validate that work.

### Root workspace contract

Under this accepted policy, Issue #43 may create only this root virtual
workspace:

```toml
[workspace]
members = []
resolver = "3"
```

This is a temporary zero-production-member transition, not a permanent Cargo
topology or evidence of a production crate boundary. The Cargo Book does not
provide a broad empty-workspace support promise. Issue #43 must verify exact
pinned Cargo behavior: `cargo metadata --format-version 1 --no-deps` must
succeed and objectively report both zero `packages` and zero
`workspace_members`. Failure makes Issue #43 `Blocked`; no placeholder crate
may be created to make validation pass. Failure instead requires focused
architecture reconsideration.

### Toolchain, Edition, and resolver

Issue #43 will pin this exact development and CI toolchain:

```toml
[toolchain]
channel = "1.97.1"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

No additional targets are installed. An exact version avoids floating
`stable` drift. `1.97.1` replaces the stale `1.97.0` candidate because the
point release addresses a compiler miscompilation. The pin is a repository
development/CI reproducibility baseline, not an MSRV promise or compatibility
floor.

The first actual package will use Edition 2024. The virtual workspace root
will explicitly select resolver 3. No package exists in this milestone, so no
`workspace.package.edition` or other `[workspace.package]` table will exist for
a nonexistent member to inherit.

### Toolchain update ownership

`YT-TechDev`, as maintainer of record, owns approval of toolchain changes. Each
pin change requires a focused Issue and Pull Request reviewing official release
notes and component availability, then rerunning local and CI metadata
validation. Updates are neither automatic nor tied to floating `stable`.
Urgent correctness or security point releases receive prompt focused review.
A toolchain update must not silently change MSRV, public API, targets,
dependencies, publication, or release policy.

### MSRV and workspace tables

Bootstrap establishes no `rust-version` and no Minimum Supported Rust Version
(MSRV). Selecting an MSRV requires a future focused compatibility decision
after real consumers and package constraints exist.

The zero-member scaffold will not create `[workspace.package]`,
`[workspace.dependencies]`, `[workspace.lints]`, custom `[profile.*]`,
`[patch]`, or `[workspace.metadata]`. With no member, these tables cannot be
inherited or enforce source policy and would imply an operational policy that
does not exist.

### Formatting, Clippy, and unsafe code

`rustfmt` and Clippy are installed for future readiness. Source formatting and
Clippy execution are `Not applicable` while no package or target exists; their
absence is not reported as a vacuous source-validation pass. Bootstrap does
not enable the full Clippy `pedantic`, `restriction`, or `nursery` groups and
does not establish permanent source-wide warnings-as-errors. The first package
requires focused lint-scope review.

The repository rule remains: `unsafe Rust is prohibited without an explicitly
approved focused Issue.` For the first real member, the selected enforcement is
workspace-inherited Rust lint `unsafe_code = "deny"`, with every member opting
in through `[lints] workspace = true`. `deny` is chosen instead of irreversible
`forbid(unsafe_code)` so a future explicitly approved, contained exception
remains technically expressible. This ADR approves neither unsafe code nor an
exception, and the zero-member scaffold adds no `[workspace.lints]`.

### Dependencies, features, and Cargo.lock

Bootstrap has zero third-party Rust dependencies, no
`[workspace.dependencies]`, features, build scripts, examples, doctest
baseline, benchmarks, fuzzing, snapshots, custom profiles, generated code, or
`Cargo.lock`. Lockfile ownership is deferred until an actual package and
dependency graph can justify deterministic application or library behavior.

### Local bootstrap validation

Issue #43 must record these commands and inspections:

- `rustup show active-toolchain`;
- `rustc --version --verbose`;
- `cargo --version --verbose`;
- `cargo fmt --version`;
- `cargo clippy --version`;
- `cargo metadata --format-version 1 --no-deps`;
- objective assertions that `packages` and `workspace_members` are empty;
- final changed-file and diff review; and
- confirmation of no `.rs`, package, dependency, feature, build script,
  `Cargo.lock`, or unexpected generated file.

With no package or target, source formatting, Clippy source lint, `cargo
check`, `cargo test`, rustdoc, doctest, feature combinations,
dependency/advisory audit, and cross-target build are `Not applicable`. No tool
is installed merely to make decision validation appear broader.

### GitHub Actions and required checks

After scaffold completion, Issue #45 owns one metadata-only workflow with:

- `pull_request` and `push` limited to `main`, with no path filters;
- no `pull_request_target`;
- explicit `permissions: contents: read`;
- one GitHub-hosted Ubuntu runner and no OS or target matrix;
- `actions/checkout` pinned to a reviewed full-length commit SHA, with
  `persist-credentials: false`;
- exact toolchain and components matching `rust-toolchain.toml`, without an
  unnecessary third-party Rust setup action;
- version evidence for rustup, rustc, Cargo, rustfmt, and Clippy;
- Cargo metadata plus objective zero-package and zero-member assertions; and
- no cache, secrets, write permissions, artifact upload, advisory scanner,
  release, or publication step.

This ADR does not select the final `actions/checkout` SHA. Issue #45 must
recheck the current upstream release and full SHA immediately before
implementation.

No status check becomes required in this milestone. The workflow must first
exist and run so its workflow, job, and check name can be observed as stable.
Any later branch-protection change requires a separate focused, approved
repository-setting Issue that considers skipped and pending required-check
behavior before enabling it.

### Contributor documentation and deferrals

After the scaffold merges, Issue #44 will document the repository role and
accepted ADR links, zero-member transition, exact pin, pin-versus-MSRV
distinction, commands that actually exist, source checks as `Not applicable`,
toolchain update ownership, deferred decisions, and the meaning of bootstrap
PASS.

This ADR explicitly defers crate/package name and metadata, `publish = false`
or publishable status, crates.io reservation, release cadence, package
publication, stable public API, ABI, serialization compatibility, browser
support, target matrix, `no_std`, WASM compatibility, and MSRV.

## Alternatives Considered

The alternatives below are ranked by fit with the current zero-member state;
the first option in each group is selected.

### Toolchain selection

1. **Exact `1.97.1` pin — selected.** It provides repeatable contributor and
   CI behavior, incorporates the miscompilation fix, and makes updates visible.
   It costs ongoing focused maintenance and initial installation, but is easy
   to reverse through review and has low false-contract risk when clearly
   separated from MSRV.
2. **Exact `1.97.0` pin — rejected.** It is reproducible and matches old Issue
   planning, but knowingly retains a superseded compiler with a documented
   correctness problem. Maintenance and contributor experience are worse, and
   changing it immediately would add avoidable churn.
3. **Floating `stable` — rejected.** It lowers explicit update work but makes
   local and CI behavior drift by installation date. Failures become harder to
   reproduce, contributors can observe different tools, and implicit updates
   can smuggle contract changes. Reversal is simple but diagnosis cost is high.
4. **No repository toolchain file — rejected.** It avoids pin maintenance but
   delegates versions to each environment, weakens evidence, and creates the
   highest contributor ambiguity and false reproducibility risk.

### Workspace transition

1. **Zero-member virtual workspace with a hard gate — selected.** It preserves
   ADR 0001, avoids a false crate boundary, and is cheaply reversible, while
   imposing exact-version validation because documentation gives no broad
   guarantee. Contributors receive an honest transition state.
2. **Placeholder crate — rejected.** It makes ordinary source commands exist,
   but invents package ownership and produces misleading green checks. It adds
   cleanup and migration costs and risks becoming permanent by inertia.
3. **Delay Cargo workspace creation — not selected.** It has the lowest
   immediate compatibility risk, but provides no executable bootstrap evidence
   or shared tooling entrypoint and delays the accepted incremental sequence.
   It remains the fallback if the hard gate fails.
4. **Root package — rejected.** It is conventional Cargo usage and enables real
   checks, but prematurely selects a production package and conflates workspace
   ownership with domain/API ownership. Reversal would require package
   migration and confuse contributors.

### MSRV

1. **No MSRV promise — selected.** It honestly reflects absent consumers and
   package constraints. It preserves full future choice with minimal
   maintenance, provided documentation prevents contributors mistaking the pin
   for a floor.
2. **Treat the current pin as accidental MSRV — rejected.** It appears simple
   but creates an untested compatibility promise, confusing development input
   with consumer support and increasing false-contract risk.
3. **Select a lower MSRV intentionally — rejected for now.** It may later help
   consumers, but without a package or consumers there is no evidence for the
   version, test matrix, maintenance cost, or compatibility trade-off.

### Unsafe enforcement

1. **Future workspace-inherited `deny` — selected.** It gives visible,
   consistent enforcement while allowing a narrower future approved exception.
   Members must explicitly opt in, creating modest review overhead but clear
   contributor behavior and good reversibility.
2. **`forbid` — rejected.** It is strongest against accidental lowering but
   cannot be overridden for an explicitly approved contained exception. That
   irreversibility conflicts with the repository's exception process.
3. **Documentation only — rejected for future members.** It is reversible and
   has no configuration cost but relies on human detection, gives contributors
   late feedback, and risks falsely claiming enforceable policy.

### Workspace tables

1. **Defer tables until a real member — selected.** Every table then has an
   operational inheritor or consumer. This minimizes maintenance and false
   policy signals, though first-member work must add and review them.
2. **Create inert package/dependency/lint tables now — rejected.** They may
   look prepared, but enforce nothing without member opt-in, invite speculative
   values, confuse contributors, and require maintenance before they deliver
   value. Removal is easy technically but can appear contract-breaking.

### Validation and CI

1. **Metadata-only validation — selected.** It tests the only real artifact and
   zero-member invariant locally and later in least-privilege CI. It is narrow,
   understandable, and maintainable, but provides no source assurance.
2. **Vacuous source commands — rejected.** They look familiar but have no target
   to format, lint, build, test, or document and therefore create false green
   coverage and poor contributor expectations.
3. **Broad matrix and advisory tooling — rejected.** These may become valuable
   with targets and dependencies, but now add action, cache, platform, network,
   and maintenance surface without evidence or an auditable graph.
4. **No CI — not selected for Issue #45.** It avoids workflow/security
   maintenance but leaves the bootstrap invariant environment-local and makes
   regressions easier. Metadata-only CI is the smaller durable evidence path.

### Cargo.lock

1. **Defer until an artifact and dependency graph exist — selected.** There is
   no resolution to preserve today. This avoids a meaningless file and keeps
   later application-versus-library ownership explicit, at the cost of making
   lockfile policy future work.
2. **Create or commit during zero-member bootstrap — rejected.** It could imply
   deterministic resolution, but no package/dependency resolution exists. It
   confuses contributors, offers no benefit, and carries high false-contract
   risk despite easy file deletion.

## Consequences

### Positive

- Development and later CI share a reproducible exact toolchain.
- The stale `1.97.0` plan is corrected to miscompilation-fixing `1.97.1`.
- Zero-member validation tests metadata and cardinality rather than reporting
  vacuous source checks.
- No speculative crate, dependency, compatibility, or publication contract is
  created.
- The future workflow is least-privilege and narrowly evidence-driven.
- MSRV, targets, publication, and compatibility remain explicitly deferred.
- Future `unsafe_code = "deny"` enforcement remains technically reversible for
  an explicitly approved contained exception.

### Negative

- An exact pin requires focused maintenance.
- Developers may need to install the pinned toolchain and components.
- Metadata-only validation provides no source assurance.
- Zero-member behavior needs exact-version proof because Cargo documentation
  supplies no broad permanent guarantee.
- First-member work must add real lint, format, build, test, and doc policy.
- No lockfile or dependency audit exists because there is no graph.

### Risks

- **Stale `1.97.0` is copied.** Mitigation: make `1.97.1` explicit and verify
  version evidence in local and CI output.
- **Floating stable causes drift.** Mitigation: use an exact channel and focused
  update ownership.
- **The pin is mistaken for MSRV.** Mitigation: state the distinction in this
  ADR and Issue #44 documentation.
- **Pinned Cargo rejects zero members.** Mitigation: Issue #43 hard gate returns
  `Blocked` and triggers focused reconsideration.
- **A placeholder crate bypasses failure.** Mitigation: prohibit it and require
  objective zero-member assertions and final file inspection.
- **Inert tables are mistaken for enforcement.** Mitigation: add none until a
  member can inherit them.
- **`forbid` prevents a future approved exception.** Mitigation: use
  workspace-inherited `deny` when the first member exists.
- **Metadata CI is represented as source coverage.** Mitigation: label source
  checks `Not applicable` and define bootstrap PASS narrowly.
- **A required check is enabled before its name stabilizes.** Mitigation: defer
  repository settings until successful workflow observation and a new Issue.
- **An action tag moves.** Mitigation: pin a reviewed full commit SHA and
  recheck it at Issue #45 implementation time.
- **Path filters leave a required workflow pending.** Mitigation: use no path
  filters and review skipped/pending behavior before any required check.
- **`pull_request_target` expands privileges.** Mitigation: prohibit that event
  and grant only `contents: read`.
- **Dependency or publication policy is smuggled into bootstrap.** Mitigation:
  require focused future approval and audit the final file set.

### Reversibility

Toolchain pin changes, bootstrap command refinements, and later operational
table additions require focused Issues and Pull Requests. A substantive change
to this accepted policy requires an ADR amendment with renewed durable
approval; replacement of its foundational workspace/toolchain approach or a
new durable architecture/compatibility choice requires a new ADR. Required
checks, branch protection, or other repository settings require a separate
focused repository-setting Issue. None is authorized by this decision.

## Compatibility and Migration

No package, crate, Rust API, ABI, serialized format, consumer, MSRV, target
support matrix, publication, or release exists, so no consumer migration is
required. The exact pin controls repository development and CI only. Future
package, MSRV, target, compatibility, and publication choices require separate
approval. A later toolchain change follows the focused update policy. If
zero-member validation fails, the project must not silently migrate to a
placeholder crate.

No browser protocol, product, adapter, Analysis Result meaning, ordering, or
determinism contract changes.

## Security and License Impact

Bootstrap has zero third-party Rust dependencies and changes no credential,
secret, permission, repository setting, publication, or network-trust policy.
The future workflow uses least privilege and immutable action pinning. This ADR
contains no active vulnerability details. The [Secure Development
contract](../development/SECURE_DEVELOPMENT.md) and [Security
Policy](../../SECURITY.md) remain authoritative.

The [MIT License](../../LICENSE) is unchanged. Dependency license review is
`Not applicable` until a dependency exists. `unsafe Rust` remains prohibited
without an explicitly approved focused Issue; this decision grants no
exception.

## Validation

Acceptance does not claim downstream implementation has passed. Downstream
evidence must demonstrate:

- explicit, attributable, durable maintainer acceptance of ADR 0002;
- exact toolchain and rustup, rustc, Cargo, rustfmt, and Clippy versions;
- successful `cargo metadata --format-version 1 --no-deps` under the pin;
- objective zero `packages` and zero `workspace_members` assertions;
- absence of a placeholder crate, package, `.rs` source, dependency, feature,
  build script, and `Cargo.lock`;
- source-level checks recorded as `Not applicable`, never false passes;
- acceptance of the workflow event, least-privilege permission, immutable
  action pin, credential, and no-cache policy before Issue #45 implementation;
- successful future Pull Request and post-merge workflow runs;
- no required-check or other repository-setting change; and
- Issue #46's independent final PASS/NO-GO audit with failures and unavailable
  checks reported under the validation contract.

Accepted-record review must also verify the filename and number, every template
section, relative links, `Accepted` status, exact `1.97.1` pin and its
distinction from MSRV, the hard zero-member Issue #43 validation gate, the
future `unsafe_code = "deny"` rationale, absence of inert workspace tables, all
scope deferrals, Pull Request #48, durable maintainer approval evidence, and
absence of private AI links, transcripts, secrets, implementation, or
fabricated approval.

## Follow-Up

- ADR 0002 is accepted through the durable maintainer approval recorded in
  Issue #42.
- Issue #43 may begin only after ADR 0002 and the index update are merged and
  Issue #42 is completed; it owns only the accepted scaffold and hard gate.
- Issue #44 remains blocked on scaffold completion.
- Issue #45 remains blocked on policy acceptance and scaffold completion.
- Issue #46 remains blocked on completion of all prerequisite Leaves.
- A separate future Issue owns any required-check repository setting.
- Future Core-domain work owns the first crate, domain, lint, API, dependency,
  MSRV, target, compatibility, publication, and release decisions.

## Approval

Approved by `YT-TechDev`, the current maintainer of record, on 2026-07-30.

Durable approval:
[Issue #42 maintainer architecture decision](https://github.com/YT-TechDev/frontend-analysis/issues/42#issuecomment-5129465895)

The approval is decision-specific and accepts the exact Rust `1.97.1`
development and CI pin, the temporary zero-production-member virtual workspace
with its hard validation gate, the toolchain-update policy, the MSRV and
compatibility deferrals, the future `unsafe_code = "deny"` strategy, the
zero-dependency and no-lockfile bootstrap boundary, the metadata-only CI policy,
the required-check deferral, and all stated boundaries and follow-up ownership.

Any substantive change requires renewed maintainer review.

## References

- [Issue #42: Rust bootstrap toolchain and validation policy](https://github.com/YT-TechDev/frontend-analysis/issues/42)
- [Issue #42 maintainer architecture decision](https://github.com/YT-TechDev/frontend-analysis/issues/42#issuecomment-5129465895)
- [Pull Request #48](https://github.com/YT-TechDev/frontend-analysis/pull/48)
- [Parent Issue #40](https://github.com/YT-TechDev/frontend-analysis/issues/40)
- [ADR 0001](0001-repository-topology-and-workspace-ownership.md)
- [Rust 1.97.1 announcement](https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/)
- [Rust releases](https://blog.rust-lang.org/releases/)
- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Rust 2024 Cargo resolver](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html)
- [rustup overrides](https://rust-lang.github.io/rustup/overrides.html)
- [rustup profiles](https://rust-lang.github.io/rustup/concepts/profiles.html)
- [rustup components](https://rust-lang.github.io/rustup/concepts/components.html)
- [Rust lint levels](https://doc.rust-lang.org/stable/rustc/lints/levels.html)
- [Rust diagnostic attributes](https://doc.rust-lang.org/reference/attributes/diagnostics.html)
- [Clippy documentation](https://doc.rust-lang.org/stable/clippy/)
- [Cargo workspace lints](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table)
- [Cargo.lock FAQ](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control)
- [GitHub Actions workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)
- [GitHub Actions secure use](https://docs.github.com/en/actions/reference/security/secure-use)
- [Required checks troubleshooting](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks)
- [Maintainership](../governance/MAINTAINERSHIP.md)
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md)
- [Validation and Completion Evidence](../development/VALIDATION.md)
- [ADR Process](README.md)
