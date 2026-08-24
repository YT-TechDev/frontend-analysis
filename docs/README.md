# Frontend Analysis Documentation

## Purpose

This index helps maintainers, contributors, and implementation agents determine
where a repository rule belongs, which document owns a topic, whether content is
normative or explanatory, how task records relate to established contracts, and
what to do when documents appear inconsistent. It is authoritative for
documentation classification and conflict resolution, but not for substantive
rules owned by specialized documents.

This index does not interpret, modify, or replace the [MIT License](../LICENSE).

## Knowledge Classes

| Class | Purpose | Authority |
| --- | --- | --- |
| Normative contract | Defines mandatory or approved behavior, responsibilities, boundaries, or processes, such as maintainership, contribution, security, vulnerability reporting, and conduct. | Has an identified topic owner; may use `MUST`, `MUST NOT`, `SHOULD`, and `MAY`; material changes require the applicable approval process. Issues, Pull Requests, templates, guides, and examples cannot silently override it. |
| Decision record | Preserves an approved decision's problem, context, alternatives, evidence, constraints, trade-offs, consequences, approval, and status. | Preserves rationale but does not silently replace an active contract. A decision that changes a contract must also update its authoritative document through approved scope. |
| Guide | Explains how to work within approved contracts. | May summarize requirements but must link to their normative source; it creates no authority by implication. |
| Example | Illustrates one possible use, workflow, structure, or output. | May become outdated, is not a compatibility commitment, and cannot become mandatory architecture without the approved decision process. |
| Task and evidence record | Focused Issues, Pull Requests, reviews, test results, and completion reports that define or demonstrate scoped work. | May contain explicit maintainer approval, implementation scope, and validation evidence, but is not automatically normative merely because it exists or was merged. |

The [ADR Process](decisions/README.md) defines when significant decisions
require an ADR. When its triggers do not apply, focused Issues, Issue comments,
Pull Requests, and approved documentation remain valid durable records. Every
record must still satisfy the
[maintainership requirements](governance/MAINTAINERSHIP.md).

## Current Documentation Map

| Area | Authoritative location | Classification | Ownership |
| --- | --- | --- | --- |
| Shared implementation-agent contract | [Repository Agent Contract](../AGENTS.md) | Normative agent execution contract and router | Shared role boundaries, scope discipline, scope-relevant document routing, escalation, validation honesty, repository side effects, and completion-report minimums. |
| Implementation-agent workflow | [Implementation Agent Workflow](development/AGENT_WORKFLOW.md) | Normative development workflow contract | Detailed execution stages, required inputs, escalation classification, validation, diff review, partial completion, and completion reporting for implementation agents. |
| Validation and completion evidence | [Validation and Completion Evidence](development/VALIDATION.md) | Normative development-validation contract | General validation principles, change-class evidence, current Rust baseline categories, manual-validation records, check statuses, failure handling, and validation evidence inside completion records. |
| Issue hierarchy and slicing | [Issue Model](development/ISSUE_MODEL.md) | Normative development-governance contract | Parent, Child, Leaf, and standalone Leaf responsibilities; hierarchy selection; dependencies; Pull Request slicing; milestone assignment; scope expansion; and Issue completion. |
| Project overview and entrypoint | [Root README](../README.md) | Guide and repository entrypoint | Maintainers approve project-purpose changes; contributors and agents may update the summary within approved scope. It is not the full source of truth for specialized contracts. |
| Licensing | [MIT License](../LICENSE) | Legal license artifact | The license text governs licensing only. This index does not interpret, modify, or override it. |
| Vulnerability reporting | [Security Policy](../SECURITY.md) | Normative contract | Private vulnerability reporting, supported-version handling, sensitive communication, and coordinated disclosure. |
| Community conduct | [Code of Conduct](../.github/CODE_OF_CONDUCT.md) | Normative contract | Community behavior, conduct reporting, investigation, and enforcement. |
| Contribution workflow | [Contributing](../.github/CONTRIBUTING.md) | Normative workflow contract | Issue-first boundaries, trivial-maintenance path, Pull Request workflow, validation reporting, AI-assisted contribution accountability, and review expectations. |
| Secure development | [Secure Development](development/SECURE_DEVELOPMENT.md) | Normative security contract | Secrets, dependencies, untrusted input, logging, local data, network trust, security-sensitive work, and `unsafe Rust`. |
| Maintainership and decision authority | [Maintainership and Decision Authority](governance/MAINTAINERSHIP.md) | Normative governance contract | Maintainer authority, approval boundaries, escalation, durable approval, and future-maintainer rules. |
| Architecture decision records | [ADR Process](decisions/README.md) | Normative decision-record process | ADR triggers, naming, required fields, status lifecycle, approval relationship, deprecation, and supersession. |
| Architecture principles | [Architecture Principles](architecture/PRINCIPLES.md) | Normative architecture contract | Browser independence, ownership, semantic integrity, dependency principles, abstraction criteria, and architecture decision tests. |
| Architecture layers | [Architecture Layers and Boundaries](architecture/LAYERS.md) | Normative architecture contract | Layer responsibilities, exclusions, allowed dependencies, boundary crossings, and cross-cutting capability ownership. |
| Rust Core contracts | [Rust Core Contracts](architecture/RUST_CORE_CONTRACTS.md) | Normative Rust architecture contract | Ownership, borrowing, mutation, domain types, errors, concurrency, async boundaries, visibility, compatibility, and Rust-specific unsafe implementation constraints. |
| Source parser ownership | [Source Parser Ownership](architecture/SOURCE_PARSER_OWNERSHIP.md) | Normative architecture contract | Project-owned HTML, CSS, and ECMAScript parser authority; retained-source provenance; capability and result integrity; third-party parser policy; implementation sequencing; validation; and parser-specific security boundaries. |
| HTML tree-construction architecture | [HTML Tree-Construction Architecture](architecture/HTML_TREE_CONSTRUCTION.md) | Normative architecture contract | Browser-independent tokenizer/tree coordination, private construction-session lifecycle, immutable tree-analysis results, constructed identity/provenance distinctions, completion/resource semantics, runtime-DOM authority separation, and capability-extension rules. |
| JavaScript / ECMAScript semantic architecture | [JavaScript / ECMAScript Architecture](architecture/JAVASCRIPT_ARCHITECTURE.md) | Normative architecture contract | ECMAScript Standard Qualification, semantic capability ownership, qualified host/runtime evidence consumption, scoped lifecycle, qualified result/provenance semantics, and representation-neutral JavaScript analysis boundaries. |
| Validated Source Anchors Guide | [Validated Source Anchors Guide](architecture/VALIDATED_SOURCE_ANCHORS.md) | Guide | Contributor guidance for current source-anchor semantics, layer consumption, accepted and rejected responsibilities, and review triggers. |
| Raw Source Coordinates Guide | [Raw Source Coordinates Guide](architecture/RAW_SOURCE_COORDINATES.md) | Guide | Explanatory contributor guidance for the accepted raw coordinate projection, units, layer conversions, and review triggers. |
| Documentation classification and precedence | This index | Normative documentation-governance contract | Documentation classes, source-of-truth selection, conflict handling, and index maintenance. It cannot redefine substantive authority owned by another specialized contract. |
| Contribution templates | [Pull Request template](../.github/PULL_REQUEST_TEMPLATE.md) and [Issue templates](../.github/ISSUE_TEMPLATE/) | Non-authoritative entrypoints and information-collection forms | Route contributors and collect evidence; they do not approve architecture, public API, dependencies, security exceptions, `unsafe Rust`, releases, or governance changes. |

## Current Rust Core State

This explanatory current-state guide has no independent normative authority.
The root remains a virtual Cargo workspace using resolver 3. Exactly one
production member exists: `crates/frontend-analysis-core`, whose private
`frontend-analysis-core` package sets `publish = false` and uses Edition 2024.
The root `Cargo.lock` is committed. The crate has zero third-party dependencies
and currently owns Validated Source Anchors and Raw Source Line Coordinates; it
is not a generic utility layer. Project-owned source parser architecture is
approved; current HTML tokenizer/parser/Core slices remain crate-private and no
public parser API is complete.

Accepted [ADR 0001](decisions/0001-repository-topology-and-workspace-ownership.md)
owns topology and extraction review, [ADR 0002](decisions/0002-rust-bootstrap-toolchain-and-validation-policy.md)
owns toolchain policy, [ADR 0003](decisions/0003-validated-source-anchors-first-rust-core-domain.md)
owns the selected domain and crate boundary, [ADR 0004](decisions/0004-validated-source-anchor-semantics.md)
owns source-anchor semantics, accepted [ADR 0005](decisions/0005-raw-source-coordinate-semantics.md)
owns raw source-coordinate semantics, accepted
[ADR 0007](decisions/0007-own-lossless-source-parsers.md) owns the project-owned
lossless source-parser strategy and language sequencing, accepted
[ADR 0008](decisions/0008-browser-runtime-evidence-normalization-and-core-import.md)
owns browser-runtime evidence normalization/import ownership, accepted
[ADR 0009](decisions/0009-javascript-semantic-analysis-architecture.md) owns the
JavaScript semantic architecture decision recorded by its specialized contract,
and accepted [ADR 0010](decisions/0010-html-tree-construction-architecture.md)
owns the HTML tree-construction architecture rationale recorded operationally by
the specialized [HTML Tree-Construction Architecture](architecture/HTML_TREE_CONSTRUCTION.md)
contract.

### Contributor Setup and Validation

With `rustup` available, install the selected Rust `1.97.1` toolchain when
needed, then run the production checks:

```bash
rustup toolchain install 1.97.1 \
  --profile minimal \
  --component rustfmt \
  --component clippy
python3 .github/scripts/validate-rust-workspace-state.py .
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo metadata --offline --format-version 1 --locked
```

The production validator must print exactly `production`. The contributor
workflow is owned by [Contributing](../.github/CONTRIBUTING.md), and evidence
status is owned by [Validation and Completion Evidence](development/VALIDATION.md).

### Validation Applicability

| Category | Current status | Reason |
| --- | --- | --- |
| Toolchain identity | Applicable | Exact Rust `1.97.1` is pinned. |
| Workspace policy and locked Cargo metadata | Applicable | One package, one member, zero dependencies, one library target, and the committed lockfile are validated. |
| Source formatting | Applicable | Production Rust source exists. |
| Clippy source lint | Applicable | The production library and all targets are linted with warnings denied. |
| Tests | Applicable | Implementation and public-contract tests validate the current domain. |
| Rustdoc | Applicable when required by the change | Rust documentation can be built with warnings denied. |
| Cross-target build | Not applicable by default | No target matrix or additional artifact is approved. |

### Deferred Decisions

| Deferred decision | Future owner |
| --- | --- |
| New domains and additional crates | Focused domain Issue/ADR |
| Dependencies and features | Focused dependency Issue |
| Source-parser algorithms, module or crate placement, capability slices, and public APIs | Focused parser work under #106, #107, and #108 and specialized contracts such as `HTML_TREE_CONSTRUCTION.md` where applicable |
| Browser protocols and Browser Adapters | Focused dependency and boundary Issue/ADR |
| Retained line indexes, reverse coordinate mapping, parser/protocol conversion, and source maps | Focused domain or boundary Issue/ADR |
| Serialization | Focused compatibility Issue/ADR |
| MSRV and target support | Focused compatibility Issue/ADR |
| Concurrency and async runtime | Focused domain Issue/ADR |
| WASM, FFI, and unsafe exceptions | Focused compatibility/security Issue/ADR |
| Release automation | Future release Issue |
| Repository extraction | Future topology/placement ADR when ADR 0001 triggers are met |

Current completion does not imply completion of a parser, Browser Adapter,
analysis-result model, CLI, desktop, VS Code, web product, serialization, or
release policy. Returning to the zero-member bootstrap is not accepted.

## Source-of-Truth Rules

Authority follows topic ownership and specificity, not a single global ranking:

1. The root [`AGENTS.md`](../AGENTS.md) governs the concise shared
   implementation-agent execution contract and routing. The [Implementation
   Agent Workflow](development/AGENT_WORKFLOW.md) governs detailed workflow
   mechanics. [`CLAUDE.md`](../CLAUDE.md) adds Claude Code-specific constraints
   and cannot weaken either shared contract. None overrides specialized
   architecture, security, governance, compatibility, or contribution
   contracts.
2. Each normative topic has one authoritative location, and its specialized
   normative document governs the topic it explicitly owns.
3. A general summary does not override a specialized contract.
4. [Maintainership and Decision Authority](governance/MAINTAINERSHIP.md)
   determines maintainer authority and valid approval.
5. The [Security Policy](../SECURITY.md) determines vulnerability reporting;
   [Secure Development](development/SECURE_DEVELOPMENT.md) determines secure
   implementation requirements.
6. The [Code of Conduct](../.github/CODE_OF_CONDUCT.md) determines community
   conduct, and [Contributing](../.github/CONTRIBUTING.md) determines the
   contribution workflow.
7. This index determines documentation classification and the conflict
   procedure. The root README is an entrypoint and summary, not an alternate
   location for specialized contracts.
8. [Architecture Principles](architecture/PRINCIPLES.md) governs durable
   architecture principles; [Architecture Layers and Boundaries](architecture/LAYERS.md)
   governs layer responsibilities and dependency boundaries; [Rust Core
   Contracts](architecture/RUST_CORE_CONTRACTS.md) governs Rust-specific Core
   design constraints; [Source Parser Ownership](architecture/SOURCE_PARSER_OWNERSHIP.md)
   governs project-owned language-parser authority, provenance, capability,
   third-party comparison boundaries, sequencing, and parser validation;
   [HTML Tree-Construction Architecture](architecture/HTML_TREE_CONSTRUCTION.md)
   governs specialized HTML tokenizer/tree coordination, private construction,
   immutable tree-result, constructed identity/provenance, completion/resource,
   and runtime-authority rules; and [JavaScript / ECMAScript Architecture](architecture/JAVASCRIPT_ARCHITECTURE.md)
   governs specialized JavaScript qualification, semantic-capability,
   host/runtime-evidence-consumption, lifecycle, provenance, and
   representation-neutrality rules.
9. The [ADR Process](decisions/README.md) governs ADR mechanics. ADRs do not
   override specialized normative contracts.
10. Templates collect information but do not create approval. Guides and
    examples cannot override normative contracts.
11. The [Issue Model](development/ISSUE_MODEL.md) governs Issue hierarchy and
    slicing mechanics. An active Leaf owns task-specific acceptance criteria and
    required checks but cannot override specialized requirements. [Validation
    and Completion Evidence](development/VALIDATION.md) governs general evidence
    by change class and risk; specialized contracts add domain requirements.
    The [Implementation Agent Workflow](development/AGENT_WORKFLOW.md) governs
    execution stages and the overall completion report. Templates only collect
    evidence and create no approval. Passing checks do not override architecture,
    security, governance, or compatibility contracts.
12. A Pull Request merge does not silently create a new contract.
13. Recency, file location, branch name, commit order, document length, silence,
    or lack of objections does not determine authority or resolve a conflict.
14. Private AI conversations and undocumented verbal discussions are not
    durable sources of truth.

When documents govern unrelated topics, neither is globally higher: the
relevant topic owner governs its domain. No document may expand its authority
by implication.

### Cross-Domain Decisions

A dependency may affect architecture and security; a browser connection may
affect adapter ownership and secure development; a public API may affect
architecture and compatibility; and a release declaration may affect
compatibility and governance. For cross-domain work:

- all affected authoritative contracts apply, and none silently cancels
  another;
- every required approval boundary and validation requirement must be
  satisfied;
- ambiguity requires escalation; and
- maintainers must record the resolution and update every affected
authoritative document.

Security does not automatically own all architecture, and architecture does
not automatically own all security decisions.

## Issues and Pull Requests

### Issues

A focused Issue may describe a problem and desired result, establish
implementation scope, identify constraints, collect evidence and alternatives,
and contain explicit maintainer approval. It is not approved merely because it
was opened, assigned, added to a milestone, implemented, or declared approved
by an agent. It may narrow scope but cannot silently override a contract.

An Issue proposing a normative change must identify the existing contract,
state the change, provide evidence and impact analysis, receive explicit
maintainer approval, update the authoritative document, and preserve focused
implementation scope.

### Pull Requests

A Pull Request may implement approved work, update an authoritative document,
and preserve validation, review, and completion evidence. It is not
automatically a normative contract. Maintainer review and merge may approve
routine scoped implementation as permitted by the maintainership contract, but
cannot retroactively authorize an undisclosed architecture, security, or
compatibility change.

### Templates

Issue and Pull Request templates prompt for information. They do not determine
whether an answer is correct, grant approval through a selected checkbox,
replace maintainer review, or override an authoritative contract.

## Superseding Decisions

A decision or contract is not superseded merely because a newer Issue, comment,
document, or Pull Request exists. Supersession requires:

1. identification of the existing decision or contract;
2. an explanation of why it is no longer sufficient;
3. relevant evidence and alternatives considered;
4. compatibility, migration, and security impact where applicable;
5. explicit maintainer approval under the current authority contract;
6. updates to every affected authoritative document;
7. an explicit record identifying the replacement; and
8. focused implementation and validation.

Historical rationale should normally remain accessible. When practical, mark
or comment on the old record with its superseded status, replacement, effective
change, and migration or compatibility consequences. Do not delete history
solely because it is obsolete. Sensitive vulnerability information remains
subject to the private Security Policy process and must not be exposed to
preserve public history. The [ADR Process](decisions/README.md) defines
ADR-specific naming, status, deprecation, and supersession mechanics.

## Ownership and Update Expectations

| Role | Responsibilities and boundaries |
| --- | --- |
| Maintainers | Are accountable for normative contracts; approve normative changes and supersession, resolve conflicts, ensure durable rationale, decide whether documents are authoritative, and protect browser independence and repository purpose. |
| Contributors | May identify stale or contradictory content, propose corrections, provide evidence, update documentation within approved scope, and request clarification. Authorship or editing grants no authority. |
| Implementation agents | May inspect documentation, report broken links or conflicts, propose alternatives, implement approved updates, and validate links and structure. They cannot independently select authority during a contract conflict, approve or supersede a contract, promote an example to architecture, or rely on private AI context as the only durable rationale. |

### Documentation Update Requirements

Work must update documentation when it changes repository contracts, public
behavior, architecture boundaries, ownership, compatibility, security
boundaries, contribution workflow, or validation expectations. A normative
change should update its authoritative document in the same focused change
when practical. If work must be split, the approved Issue must define the
dependency, the contract update cannot be omitted, and implementation cannot
be called complete while required normative documentation remains unresolved.
Purely internal mechanical changes need no documentation update when they do
not affect behavior, contracts, ownership, or contributor understanding.

## Adding or Changing Documentation

Before adding a document, identify its purpose, knowledge class, topic owner,
normative or explanatory status, audience, update responsibility, links to
authoritative contracts, and whether an existing document already owns the
topic. Do not add documentation merely to duplicate rules, reserve a
speculative directory, create a placeholder, preserve private AI output, avoid
updating an authoritative source, or introduce unapproved future architecture.

When a new authoritative document is approved, add it to this map, identify its
domain and relationship to existing contracts, validate links, and remove or
revise duplicated normative text.

## Resolving Conflicts

1. Stop implementation at the affected boundary.
2. Identify the conflicting statements.
3. Classify each document and identify the topic each owns.
4. Check for an explicit approved supersession.
5. Do not decide from recency or preference alone.
6. Collect evidence and impact, then request maintainer review.
7. Record the decision durably.
8. Update every affected authoritative document.
9. Validate links and remove contradictory duplicated rules.
10. Resume implementation only within the resolved contract.

If a conflict exposes an active vulnerability, use the private Security Policy
process and do not publish sensitive details here.

### Required Scenario Outcomes

| Scenario | Authority | Proceed? | Escalation | Durable record and documentation update |
| --- | --- | --- | --- | --- |
| Task Issue conflicts with Secure Development | Secure Development | Stop at the conflict. | Maintainer review of a focused contract proposal. | Record approval and rationale; update Secure Development before affected work. |
| Root README has an outdated governance summary | Maintainership and Decision Authority | Only unaffected work proceeds. | Report the mismatch for maintainer review. | Correct the README summary; retain governance as the authority. |
| Pull Request template checkbox conflicts with Contributing | Contributing | Do not rely on the checkbox. | Maintainer review of the mismatch. | Correct the template; selection records no approval. |
| Approved decision changes a future normative architecture rule | The applicable architecture contract and Maintainership and Decision Authority | Proceed only after approval and contract update. | Obtain explicit maintainer approval. | Preserve rationale, explicitly supersede the old rule, and update the authoritative architecture contract; this index creates no such contract. |
| Example contradicts an established public contract | The established public contract | Do not rely on the example. | Report the conflict to the topic owner. | Correct or remove the example; do not redefine compatibility. |
| Security and architecture both apply | Both specialized domain contracts | Proceed only when both are satisfied. | Escalate ambiguity and satisfy all approval boundaries. | Record the cross-domain resolution and update every affected contract. |
| Newer Issue contradicts an older approved contract | The approved contract | Stop affected work. | Request explicit maintainer approval; recency is insufficient. | Record the decision and update the contract if change is approved. |
| Merged Pull Request hides an architecture decision | Maintainership and Decision Authority and the applicable architecture contract | Stop affected work; merge alone is insufficient. | Escalate for focused durable approval. | Record rationale and approval, then correct documentation and implementation scope. |
| Agent finds two normative documents claiming one topic | Maintainer determines ownership under Maintainership and Decision Authority | Agent stops at the boundary and does not choose. | Maintainers resolve ownership. | Record resolution; remove duplicate normative content or convert it to a linked summary. |
| Private AI conversation is the only major-decision rationale | Maintainership and Decision Authority | Do not treat the decision as adequately recorded. | Request durable evidence, rationale, and approval. | Add them to an approved repository record; private conversation links are neither required nor accepted as the source of truth. |

This model has no circular precedence: the maintainership contract defines who
may approve; specialized contracts define their substantive domains; this
index classifies documentation and defines conflict handling. None derives or
expands its authority from a template, task record, summary, or newer document.
