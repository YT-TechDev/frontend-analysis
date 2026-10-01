# ADR 0011: Establish CLI Product and CSS CoreV1 Consumer Boundary

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-10-01 |
| Decision owner / approver | YT-TechDev |
| Linked Issue | [#857](https://github.com/YT-TechDev/frontend-analysis/issues/857) |
| Related Pull Request | [#858](https://github.com/YT-TechDev/frontend-analysis/pull/858) |
| Supersedes | None |
| Superseded by | None |
| Affected normative contracts | None — the existing layer and Rust Core contracts already permit Product consumption of approved Core boundaries and prohibit Product concerns from entering Core. This ADR resolves previously deferred package placement and the first public analysis-consumer boundary without changing those rules. |

## Context

Frontend Analysis has a browser-independent Rust Core but no production Product
consumer. The accepted architecture already places CLI, desktop, VS Code, and
web surfaces outside Core. Products may consume approved Core and Analysis
Result contracts; Core must not depend on concrete Product, presentation,
filesystem, network, or process behavior.

The current physical workspace contains only the Core package. Its CSS
tokenizer, parser, source-evidence reconciliation, and selector qualification
pipeline are production capabilities, but the analysis entry points and result
graph remain crate-private.

Issue #857 selects the first bounded Product candidate:

    authored UTF-8 CSS on stdin
        -> frontend-analysis-cli
        -> narrow public CSS CoreV1 facade
        -> existing CSS tokenizer/parser/selector qualification
        -> deterministic human-readable report

The initial Product command is:

    fa css-selectors < style.css

This selection follows the intentional ECMAScript frontier pause recorded in
Issue #688 comment 5913823937. The purpose is not to resume language-frontier
expansion. It is to expose one already-owned Core capability through one real
consumer, dogfood it, and let demonstrated gaps drive later focused work.

### Relationship to ADR 0006

ADR 0006 correctly deferred the first source-anchored production slice because
no concrete consumer, workflow, uniquely selected analysis question, or
candidate-specific validation envelope existed.

Issue #857 now records the concrete candidate needed by ADR 0006:

1. consumer surface: frontend-analysis-cli, binary fa;
2. input path: bounded stdin;
3. workflow step: explicit fa css-selectors invocation;
4. decision changed by the answer: whether current Core coverage is sufficient
   or a reproducible limitation warrants focused follow-up;
5. analysis question: how the existing CSS CoreV1 selector profile classifies
   selector-list contexts retained from the authored stylesheet;
6. grammar family: CSS;
7. parser/provenance target: the existing project-owned CSS tokenizer, parser,
   Core reconciliation, and selector-qualification path;
8. result semantics: qualified, invalid, unsupported, indeterminate,
   incomplete, resource-limited, and internal failure remain distinct;
9. measurable resource and validation targets: the finite Phase 1 resource
   policy recorded and approved in #857;
10. fixture provenance: repository-owned focused fixtures and accepted Core
    validation evidence.

The finite Phase 1 resource policy was explicitly approved by the maintainer in
[#857 comment 5924774040](https://github.com/YT-TechDev/frontend-analysis/issues/857#issuecomment-5924774040).
That approval accepts the code-derived calibration basis and states that
unobserved runtime high-water marks are not a pre-implementation blocker for
this bounded slice.

ADR 0006 remains historically correct and is not superseded. This ADR records
the later architecture choice made after its reconsideration gate became
satisfiable.

### Decision drivers

The first Product boundary must:

- preserve Core semantic ownership and browser independence;
- establish one genuine external Rust consumer;
- expose only the public Rust surface required by that consumer;
- preserve source-backed evidence without post-hoc reconstruction;
- keep invalid, unsupported, indeterminate, incomplete, and resource-refused
  states distinct;
- retain deterministic synchronous Core execution;
- keep Product acquisition and presentation outside Core;
- avoid serialization and third-party dependencies;
- preserve strict workspace topology validation;
- avoid designing HTML, ECMAScript, or a generic multi-language CLI before
  their Product slices exist.

## Decision

Frontend Analysis will establish its first Product consumer as a separate Cargo
workspace package:

    crates/
    ├── frontend-analysis-core/
    │   └── library target
    └── frontend-analysis-cli/
        └── binary target: fa

The production source dependency is exactly:

    frontend-analysis-cli
        -> frontend-analysis-core

Core must not depend on the CLI.

### Product package ownership

frontend-analysis-cli owns Product responsibilities only:

- argv and command routing;
- bounded stdin acquisition;
- strict UTF-8 conversion;
- invocation-local SourceId assignment;
- Core invocation;
- deterministic human-readable presentation;
- stdout and stderr;
- process exit status.

The package is non-published, inherits the workspace Rust edition and lint
policy, has no build script or feature matrix in Phase 1, and has no
third-party dependency. Its only package dependency is the local
frontend-analysis-core package.

No application crate, presentation crate, or shared CLI framework crate is
introduced.

### Core ownership

frontend-analysis-core continues to own:

- SourceText and source identity semantics;
- tokenization and parsing;
- source-backed evidence;
- coverage, diagnostics, recovery, discard, and unsupported meaning;
- completion and termination meaning;
- resource dimensions, accounting, refusal semantics, and refusal evidence;
- CSS CoreV1 selector qualification;
- result ordering and analysis meaning;
- Core boundary and contract failure.

argv, stdin, stdout, stderr, terminal behavior, filesystem behavior, and process
exit do not enter Core.

### First Product capability

The first Product capability is limited to the existing CSS CoreV1 selector
analysis path:

    SourceText
        -> existing CSS tokenizer
        -> existing CSS parser
        -> existing Core source-evidence reconciliation
        -> existing CSS CoreV1 selector qualification

This ADR authorizes no CSS grammar expansion, parser redesign, or CoreV1 profile
expansion.

### Public consumer boundary

Core will expose one intentional CSS-specific public consumer facade over that
existing internal capability.

The approved conceptual shape is:

    &SourceText
        -> one CSS CoreV1 operation
        -> owned CSS-specific report
        -> read-only presentation-independent views

Exact Rust identifiers may be finalized during the focused implementation
review, but the semantic boundary is fixed by this ADR.

The public facade must expose only the information needed by the named Product
consumer to preserve:

- selected CSS capability/profile;
- upstream completion and coverage state;
- selector execution state;
- ordered selector observations;
- qualification outcome;
- outcome reason where applicable;
- source-backed context evidence;
- source-backed subject evidence where applicable;
- resource refusal and its owning stage;
- returned Core boundary failure.

The facade must preserve distinct meanings for selected-grammar qualification,
selected-grammar invalidity, unsupported selected-profile coverage,
indeterminacy, upstream incompleteness, resource refusal, and Core failure.

No universal cross-language AnalysisResult is introduced.

Existing tokenizer, parser, and selector producer/run-result implementation
types remain private unless a narrowly justified private accessor is required
to project already-owned evidence into the public facade.

### Source evidence

Existing public source primitives remain authoritative:

- SourceId;
- SourceText;
- SourceAnchor;
- SourceRange;
- RawSourceCoordinate.

The facade must consume evidence retained by the authoritative Core path. It
must not obtain source-backed evidence through source search, delimiter search,
rescanning, retokenization, reparsing, endpoint reconstruction,
decoded-length inference, synthetic wrapping, or equivalent reconstruction.

Phase 1 processes exactly one source per invocation and uses SourceId value 0
as an invocation-local deterministic identity. It is not filesystem, URL,
content-hash, runtime, or global project identity.

### Input contract

The first command is fa css-selectors with exactly one authored stylesheet from
stdin.

The Product performs bounded byte acquisition and strict UTF-8 decoding, then
constructs SourceText from the exact decoded string.

The Product must not trim input, normalize line endings, strip a BOM, rewrite
whitespace or escapes, wrap selectors, synthesize CSS rules, or infer language
from source content.

Empty input is a valid source. Invalid UTF-8 is rejected before Core invocation.

### Resource boundary

Product acquisition and Core execution resources remain distinct.

The CLI owns the finite stdin acquisition ceiling.

Core owns the existing tokenizer, parser, and selector resource dimensions,
accounting, exhaustion semantics, and refusal evidence.

The CSS facade encapsulates the finite Phase 1 execution envelope approved in
Issue #857. The exact numeric values remain Issue #857 execution policy rather
than CSS grammar semantics, universal Core capacity guarantees, or public
caller configuration.

This ADR does not expose the internal resource dimensions as configurable
public API. Future caller-configurable limits require a demonstrated consumer
need and focused review.

### Presentation and exit semantics

Phase 1 produces deterministic human-readable text.

No JSON or other serialized representation is selected. Rust Debug output is
not a Product contract. No terminal styling, timestamp, locale-dependent
ordering, or TTY-dependent semantic behavior is required.

The initial process contract is:

- exit 0: an analysis report was successfully produced;
- exit 1: Product command/input acquisition failure;
- exit 2: returned Core boundary/internal failure.

Exit 0 includes ordinary analysis outcomes such as qualified, invalid,
unsupported, indeterminate, incomplete, and resource-limited results. Those
meanings describe analyzed source or bounded coverage; they are not Product
execution failures.

A panic remains a defect rather than an approved exit outcome.

### Workspace validation

The strict production workspace validator will be updated from one accepted
package/member to exactly these two:

- frontend-analysis-core;
- frontend-analysis-cli.

It must continue to reject unexpected packages, targets, source roots, build
scripts, dependencies, and manifest states.

The accepted target/dependency shape is:

- Core: one library target and zero third-party dependencies;
- CLI: one binary target named fa, one local Core dependency, and zero
  third-party dependencies.

If an exact CLI integration-test target is added, it must be explicitly
allowlisted rather than permitting arbitrary targets.

### Repository placement

The CLI initially resides in YT-TechDev/frontend-analysis as another workspace
member.

ADR 0001 remains active. This bounded Product slice does not itself demonstrate
an extraction trigger: it has no independent publication cadence, independent
security ownership, independently versioned cross-repository consumer need, or
repository/CI-scale requirement.

If an ADR 0001 extraction trigger later becomes true, repository placement must
be reconsidered separately.

## Alternatives Considered

### Same Cargo package as frontend-analysis-core

A binary target could be added to the existing package.

Benefits: fewer workspace members and a smaller manifest diff.

Costs: a library target and binary target are still separate Rust crates, so
the binary still cannot consume library crate-private items. A public Core
boundary is still required. More importantly, Product concerns and Core
ownership would share one Cargo package without eliminating any meaningful
boundary work.

Not selected. Package separation is justified by architectural ownership, not
by a Rust privacy workaround.

### Separate frontend-analysis-cli package

Benefits: physical ownership matches the conceptual Product/Core boundary;
dependency direction is explicit; Core remains library-only; the CLI behaves as
a genuine consumer of the approved Core API; strict workspace validation can
enforce the topology.

Costs: one new workspace member, manifest/validator changes, and the need for an
intentional public Core facade.

Selected because these costs are the exact boundaries Phase 1 needs to
establish and validate.

### Intermediate application/facade crate

Benefits: could host orchestration reusable by future Products.

Costs: no second Product or reusable application lifecycle has been
demonstrated. It would add a speculative package, dependency edge, API, and
ownership domain.

Not selected. Reusable orchestration may be reconsidered only after repeated
consumer evidence.

### Separate presentation crate

Benefits: could share formatting among future Products.

Costs: only one Product and one local human-readable output exist. No reusable
presentation contract has been demonstrated.

Not selected. Presentation remains CLI-owned.

### Expose current CSS internals directly

Benefits: less facade mapping.

Costs: implementation structure would become compatibility surface, widening
public API beyond the named consumer and increasing refactoring cost.

Rejected. The consumer receives a narrow capability-specific facade.

### Public caller-configurable Core limits

Benefits: callers could choose execution budgets.

Costs: Phase 1 has one bounded Product consumer. Publishing every internal
resource dimension would create unnecessary compatibility commitments.

Not selected. The facade encapsulates the approved Phase 1 execution envelope.

### JSON output in Phase 1

Benefits: easier machine consumption.

Costs: no machine consumer exists. JSON would create naming, versioning,
ordering, unknown-value, and serialization commitments without current need.

Not selected. Deterministic human-readable output is sufficient for initial
dogfood.

### HTML or ECMAScript as the first Product slice

Benefits: either may later be useful Product capabilities.

Costs: the current productization review identified the existing CSS CoreV1
selector path as the narrow bounded capability with useful result semantics for
the first Product consumer. Selecting another grammar for symmetry would reopen
work without demonstrated benefit.

Not selected for Phase 1. This is not a rejection of later HTML or ECMAScript
Product slices.

## Consequences

### Positive

- Frontend Analysis gains its first real Product consumer.
- Core remains browser-independent and Product-independent.
- Cargo topology makes Product/Core ownership visible and mechanically
  enforceable.
- The first public analysis API is justified by a named consumer.
- Existing CSS semantics and source evidence remain authoritative.
- Real dogfood can precede broader CLI architecture.
- Later HTML and ECMAScript slices can use evidence from a real Product.
- No third-party dependency or serialization contract is introduced.

### Negative

- Core gains its first intentional analysis-oriented public Rust compatibility
  surface.
- Workspace maintenance now covers two packages.
- Strict workspace validation and current-state documentation must be updated.
- The Phase 1 execution envelope is a maintained Product policy decision.
- Deterministic CLI output becomes Product behavior that needs focused tests.

### Risks

Public facade growth: mitigate by exposing only named-consumer needs and keeping
internal run-result types private.

Presentation reinterpretation: mitigate by keeping classifications and evidence
meaning Core-owned; Product formatting may arrange approved information but may
not upgrade or collapse semantic states.

Execution-policy confusion: mitigate by keeping exact limits in Issue #857 and
describing them as Phase 1 execution policy rather than language semantics or
universal defaults.

Premature cross-language abstraction: mitigate by keeping this facade CSS
specific and adding HTML/ECMAScript only through later focused slices.

Validator weakening: mitigate by replacing one exact topology with another
exact topology, not generic discovery.

Repository inertia: mitigate by retaining ADR 0001 extraction triggers.

## Compatibility and Migration

Existing public source primitives retain their current meaning.

This ADR introduces one intentional CSS analysis-consumer API. Internal CSS
tokenizer, parser, selector, and resource implementation types remain private
and gain no public compatibility promise.

Internal implementation may evolve if the public facade continues to preserve
its approved domain meanings, evidence invariants, and deterministic result
semantics.

No serialized representation, browser protocol, IPC model, or external package
publication commitment is introduced.

There is no previous Product consumer to migrate.

For identical authored UTF-8 source, SourceId, approved execution policy, Core
version, and CLI version, Product presentation must not add nondeterministic
ordering, timestamps, random identifiers, locale-dependent semantics, or
scheduling-dependent meaning.

This ADR does not define HTML or ECMAScript commands or result contracts. Future
domains must not be forced into the CSS result shape merely for UI symmetry.

## Security and License Impact

The CLI reads untrusted source bytes from stdin. Product acquisition must be
bounded before an unbounded source buffer is allocated. Core execution remains
bounded by the finite policy approved in Issue #857.

Invalid, unsupported, incomplete, and resource-exhausted states must remain
honest outcomes rather than being upgraded into completeness claims.

No filesystem traversal, networking, browser launch, process spawning, IPC,
dynamic loading, plugin system, telemetry, async runtime, concurrency model, or
repository-authored unsafe Rust is introduced.

No third-party dependency is introduced, so this decision adds no third-party
license or supply-chain obligation. The repository remains MIT licensed.

Existing Secure Development and Security Policy authority remains unchanged.

## Validation

Before production implementation begins:

- this ADR must receive explicit, durable maintainer approval;
- its status must become Accepted only with valid approval evidence;
- Issue #857's already-approved finite Phase 1 resource policy remains the
  execution-policy authority.

Implementation validation must demonstrate:

- exactly the approved two-package workspace topology;
- Core library-only ownership and CLI binary-only Product ownership;
- CLI-to-Core dependency direction with no reverse dependency;
- no third-party dependencies;
- the narrow CSS facade preserves qualified, invalid, unsupported,
  indeterminate, incomplete, resource-refused, and Core-failure meanings;
- source evidence remains owned and is not reconstructed;
- stdin acquisition is finite and exact-source preserving;
- deterministic Product output and the approved three-class exit contract;
- the strict workspace validator rejects unapproved packages, targets,
  dependencies, and source roots.

At minimum, the implementation must pass:

    python3 .github/scripts/validate-rust-workspace-state.py .
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-targets --all-features
    cargo metadata --offline --format-version 1 --locked

Expected Product output in tests must be independently authored rather than
generated by the candidate implementation and blessed as its own oracle.

## Follow-Up

After acceptance:

1. implement only the bounded Issue #857 slice;
2. dogfood real CSS inputs through fa css-selectors;
3. classify reproducible limitations before opening targeted follow-up work;
4. consider HTML and ECMAScript as later independent Product slices;
5. generalize common Product/presentation abstractions only after repeated
   evidence from multiple real commands.

No generic application layer, presentation crate, cross-language result type,
serialization format, file/project mode, or configurable resource framework is
authorized by this ADR.

## Approval

Approved by `YT-TechDev`, the current maintainer of record, on 2026-10-01.

Durable approval:
[PR #858 maintainer architecture approval](https://github.com/YT-TechDev/frontend-analysis/pull/858#issuecomment-5924911080)

The approval is decision-specific and accepts:

- the separate `frontend-analysis-cli` Product package;
- the one-way `frontend-analysis-cli -> frontend-analysis-core` dependency;
- the narrow CSS CoreV1 public consumer boundary;
- the Product/Core ownership split;
- the bounded Phase 1 scope defined by this ADR.

The exact Phase 1 resource envelope remains the separately approved execution
policy recorded in Issue #857 and is not promoted to durable CSS semantics or
public caller configuration by this approval.

Production implementation may proceed only within the approved scope of Issue
#857. Any material change to the package boundary, dependency direction, public
semantic boundary, or stated scope requires renewed focused architecture review.

## References

- [Issue #857: bounded CSS CoreV1 selector Product slice](https://github.com/YT-TechDev/frontend-analysis/issues/857)
- [Issue #857 resource-policy recommendation](https://github.com/YT-TechDev/frontend-analysis/issues/857#issuecomment-5924758446)
- [Issue #857 maintainer resource-policy approval](https://github.com/YT-TechDev/frontend-analysis/issues/857#issuecomment-5924774040)
- [Issue #688: durable ECMAScript semantic research authority](https://github.com/YT-TechDev/frontend-analysis/issues/688)
- [Issue #688 productization freeze checkpoint](https://github.com/YT-TechDev/frontend-analysis/issues/688#issuecomment-5913823937)
- [Issue #855](https://github.com/YT-TechDev/frontend-analysis/issues/855)
- [PR #856](https://github.com/YT-TechDev/frontend-analysis/pull/856)
- [ADR 0001: Repository topology and workspace ownership](0001-repository-topology-and-workspace-ownership.md)
- [ADR 0003: Establish Validated Source Anchors as the first Rust Core domain](0003-validated-source-anchors-first-rust-core-domain.md)
- [ADR 0006: Qualify the First Source-Anchored Analysis Vertical Slice](0006-qualify-first-source-anchored-analysis-vertical-slice.md)
- [ADR 0007: Own Lossless Source Parsers](0007-own-lossless-source-parsers.md)
- [ADR 0010: Define HTML Tree-Construction Architecture](0010-html-tree-construction-architecture.md)
- [Architecture Layers and Boundaries](../architecture/LAYERS.md)
- [Rust Core Contracts](../architecture/RUST_CORE_CONTRACTS.md)
- [Maintainership and Decision Authority](../governance/MAINTAINERSHIP.md)
- [Architecture Decision Record Process](README.md)
- [Secure Development](../development/SECURE_DEVELOPMENT.md)
- [Validation and Completion Evidence](../development/VALIDATION.md)
