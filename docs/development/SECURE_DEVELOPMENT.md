# Secure Development

## Purpose and Scope

This document defines the repository-wide secure-development contract for
maintainers, human contributors, Codex, Claude Code, and future implementation
agents. It applies to future Rust Core, browser adapter, tooling, CI, and
automation work. It defines security boundaries and approval requirements, not
production designs, dependencies, or tooling choices.

Core MUST remain browser-independent. Browser-specific connection, protocol,
and credential concerns belong at adapter or integration boundaries and MUST
NOT be presented as Core behavior.

## Normative Language

`MUST` and `MUST NOT` state mandatory repository contracts. `SHOULD` and
`SHOULD NOT` state strong defaults that may be departed from when the reason is
documented and approved at the level appropriate to the risk. `MAY` identifies
an explicitly permitted choice. A requirement for explicit approval means
maintainer approval in a focused Issue before implementation. Contextual
practices remain subject to the concrete feature and its approved security
boundary; deferred topics are not authorized by this document.

## Security Ownership and Escalation

- Maintainers MUST approve repository security contracts and any exceptions to
  them.
- Implementation agents MAY identify risks, alternatives, and recommended
  controls, but MUST NOT approve security exceptions.
- When ambiguity can affect security, implementation MUST stop at that boundary
  and the ambiguity MUST be escalated to maintainers before work continues.
- An implementation Issue MUST NOT silently weaken or override an established
  security contract. A proposed change MUST identify the existing contract and
  obtain explicit approval.
- Security findings and security-sensitive failures MUST NOT be hidden behind a
  generic success result.
- Significant security decisions MUST have a durable public rationale, unless
  publishing that rationale would expose an active vulnerability. Sensitive
  details MUST instead follow the private process in the Security Policy.

These responsibilities do not establish a security team or organization.

## Secrets and Credentials

Secret values and personally identifying private data MUST NOT be committed in
source files, configuration committed to Git, test fixtures, snapshots,
examples, documentation, logs, error messages, screenshots, completion
reports, Pull Request descriptions, or Issue comments. This requirement covers
API keys, access tokens, passwords, private keys, cookies and session tokens,
browser authentication material, personally identifying private data, and
private infrastructure credentials.

- Runtime secrets MUST come from the environment or an approved secret-storage
  boundary; a concrete product is not selected here.
- Examples MUST use unmistakable placeholder names rather than realistic secret
  values.
- Logs, diagnostics, screenshots, and reproduction artifacts MUST be redacted
  before they are shared.
- When exposure is suspected, affected secrets MUST be revoked and rotated as
  appropriate; removing a value from the latest revision is not sufficient.
- If a committed-secret exposure creates a security incident, its report and
  sensitive remediation details MUST use the private vulnerability-reporting
  process.

## Dependencies and Supply Chain

A new dependency or major dependency update MUST have a demonstrated current
need. Dependencies MUST NOT be added only for speculative future use.

Before adoption, the change MUST document review proportional to its risk,
including:

- its purpose and why existing code or current dependencies are insufficient;
- maintenance status and release activity;
- security history where reasonably available;
- transitive dependency impact and license compatibility;
- platform compatibility and, when relevant, Rust version compatibility;
- exposure through public APIs; and
- expected replacement or removal cost.

A trivial development-only tool MAY receive a lighter review than a runtime or
public-API dependency. Review MUST still be sufficient to explain its need and
material supply-chain impact. This contract does not select any dependency.

## Lockfiles and Reproducibility

Lockfile policy MUST be chosen according to artifact type and Rust ecosystem
conventions. Applications, binaries, tools, and deployed products generally
have different reproducibility needs from published Rust libraries, so a
single `Cargo.lock` rule MUST NOT be applied without considering the artifact.

- A focused crate or workspace decision MUST state whether its lockfile is
  committed and why.
- Committed lockfiles MUST be intentionally maintained.
- Dependency-resolution changes MUST be visible and reviewable.
- CI and release builds SHOULD use reproducible dependency resolution where
  practical for the artifact.
- Lockfile changes MUST NOT be concealed in unrelated work.

## Untrusted Input and Parsing

Unless a future approved design establishes a narrower trust boundary, HTML,
CSS, JavaScript source, source maps, browser protocol events, DOM and CSSOM
snapshots, trace files, imported analysis artifacts, user-selected local files,
network responses, adapter payloads, and serialized project data MUST be
treated as potentially untrusted.

Each parser or ingestion path MUST define and validate explicit limits when it
is designed. Concrete values depend on feature requirements and are not set by
this document. Its design MUST consider:

- input size and resource limits;
- recursion and nesting depth;
- malformed or partial data;
- unknown enum values and protocol evolution;
- integer conversion and overflow;
- path traversal;
- decompression or other expansion risks;
- time and memory exhaustion;
- cancellation where work may be long-running; and
- typed error reporting.

Rejected or incomplete input MUST produce an honest failure rather than a
security-relevant partial success. Adapter validation MUST protect Core from
browser-specific protocol assumptions; Core MUST validate its own domain
invariants without assuming a particular browser protocol.

## Logging, Diagnostics, and Error Reporting

Logs, diagnostics, and errors MUST avoid secrets and unnecessary private data
and MUST apply redaction where sensitive values may appear. They SHOULD
distinguish user-facing messages from internal diagnostic context and preserve
enough structured context for investigation.

Full untrusted payloads SHOULD NOT be logged by default. A justified diagnostic
path MAY capture additional context only when its audience, handling, and
redaction boundary are explicit. Security-sensitive failures MUST NOT be
converted into generic success. Private filesystem paths or environment
details MUST NOT be exposed unless they are necessary for the intended
diagnostic audience and handled deliberately.

## Temporary Files and Local Data

A future feature that writes local data MUST:

- use a controlled location and avoid predictable insecure temporary paths;
- limit permissions where the platform supports doing so;
- avoid retaining sensitive data longer than the approved feature requires;
- define ownership and cleanup responsibility;
- report cleanup failure or interruption honestly;
- prevent path traversal and unintended overwrite; and
- document persistence when it could affect user expectations.

The feature's focused Issue MUST resolve any required persistence and retention
policy before implementation. This contract does not choose a storage
technology or production retention period.

## Network and Browser Connections

Every future network or browser-runtime connection MUST define an explicit
trust boundary before implementation. The design MUST address endpoint
ownership, authentication material, transport security, redirects, origin and
host validation, connection permissions, timeout and cancellation behavior,
exposure of local debugging endpoints, protocol input validation, least
privilege, and user awareness when connecting to a browser or remote target.

Local debugging endpoints MUST NOT become remotely accessible implicitly. Any
exposure or remote connection requires explicit maintainer approval, a stated
need, and controls proportionate to the resulting boundary. Authentication
material MUST follow the secrets requirements above.

These requirements apply without selecting a protocol library or connection
architecture. Adapter designs MAY support Chromium/CDP, WebKit, Firefox, local
browser sessions, or remote analysis targets when separately approved.
Browser-specific transport and protocol behavior MUST remain outside Core.

## Unsafe Rust

Unsafe Rust is not permitted without an explicitly approved Issue.
Implementation agents cannot approve an `unsafe` exception.

Before implementation, a proposed exception MUST document:

- why safe Rust is insufficient;
- the exact safety invariants;
- ownership and lifetime assumptions;
- memory and concurrency assumptions;
- the containment boundary and public API impact;
- tests and applicable validation tooling;
- benchmark evidence when performance is the justification;
- maintenance risks and review requirements; and
- a removal or further-containment strategy where practical.

If approved, `unsafe` MUST be minimized and encapsulated. Surrounding safe APIs
MUST preserve the documented invariants. Safety comments MUST explain those
invariants rather than merely restating the operation. Validation tools such as
interpreters, sanitizers, or fuzzers MAY be required based on the change, but
this document does not make a particular tool universal.

## Security-Sensitive Changes

The following changes require explicit security review: dependency additions;
parsers and deserializers; file or network access; browser debugging
connections; credential handling; local persistence; privilege or permission
changes; sandbox boundaries; process execution; concurrency that affects
security invariants; `unsafe`; public APIs that expose sensitive data; logging
of untrusted or private data; CI workflow permission changes; and release or
artifact signing.

Each such change MUST have:

- a focused Issue;
- an explicit threat or abuse analysis proportional to risk;
- clear ownership;
- validation evidence;
- documented residual risks; and
- maintainer approval.

A minor change does not require a full formal threat model when a smaller
analysis adequately covers its boundary. The reviewer and evidence MUST still
be proportional to the actual risk.

## Validation and Completion Evidence

Security-sensitive work is not complete merely because it compiles. Its
completion report MUST provide evidence proportional to the change. Depending
on the boundary, that evidence MAY include tests for accepted and rejected
inputs, boundary and malformed-input tests, regression tests, dependency or
permission review, manual security scenarios, platform-specific validation,
static analysis, fuzzing, sanitizer or interpreter-based checks, review of logs
and error output, and final diff inspection.

- Checks not run MUST be reported honestly.
- Failed security validation MUST NOT be hidden or weakened merely to merge.
- Unavailable tooling MUST be recorded as a limitation.
- A validation failure that reveals an architectural defect MUST be escalated
  before implementation continues.

A future repository-wide validation contract may add general requirements.
Security-sensitive work MUST satisfy both that contract, once approved, and
the security-specific requirements here.

The following examples make common outcomes explicit:

| Scenario | Outcome | Contract basis |
| --- | --- | --- |
| Commit an API token in a test fixture | Prohibited | Secret values MUST NOT be committed in test fixtures. |
| Add a parser dependency for possible future use | Prohibited | Dependencies MUST have a demonstrated current need and MUST NOT be speculative. |
| Accept a deeply nested malformed adapter payload without defined limits | Prohibited | The ingestion path MUST define and validate limits for nesting and malformed input. |
| Log an entire browser event | Requires explicit approval | Full untrusted payloads SHOULD NOT be logged by default; exceptional capture requires an approved handling and redaction boundary. |
| Write private source data to a temporary file | Requires explicit approval | The focused feature Issue must approve its local-data boundary, permissions, ownership, persistence, and cleanup before implementation. |
| Expose a local debugging endpoint for a remote browser connection | Requires explicit approval | Remote exposure requires an approved trust boundary and proportionate controls. |
| Add `unsafe` for optimization without benchmark evidence | Prohibited | An approved exception is required, and a performance justification MUST include benchmark evidence. |
| Weaken a failing security test merely to merge | Prohibited | Failed security validation MUST NOT be hidden or weakened merely to merge. |
| Apply different lockfile treatment to a published library and a desktop binary | Allowed | Artifact-specific policy is required and MUST be documented for each crate or workspace. |
| Choose a concrete production retention period without an approved feature boundary | Deferred pending architecture decision | The focused feature Issue MUST resolve retention before implementation. |

## Vulnerability Disclosure

Suspected vulnerabilities and sensitive remediation details MUST follow the
private reporting and coordinated-disclosure process in the
[Security Policy](../../SECURITY.md).

## Deferred Design Decisions

The following decisions are deferred until a focused architecture or feature
Issue establishes the relevant requirements and receives maintainer approval:

- concrete cryptographic design;
- authentication and authorization architecture;
- production data-retention periods;
- browser credential-storage mechanisms;
- sandbox implementation;
- network protocol libraries;
- async runtime;
- parser libraries;
- security scanner selection;
- signing infrastructure; and
- any exception to the `unsafe` prohibition.

Deferred does not mean unrestricted or implicitly approved. A future Issue
MUST resolve the applicable security boundary before implementation begins,
and implementation agents MUST stop rather than select one of these designs on
their own.
