# Architecture Layers and Boundaries

## Purpose and Authority

This document is the normative source for layer responsibilities, exclusions,
ownership boundaries, boundary crossings, and dependency rules in Frontend
Analysis. Durable project-wide invariants are owned by
[Architecture Principles](PRINCIPLES.md).

Architecture approval is governed by
[Maintainership and Decision Authority](../governance/MAINTAINERSHIP.md),
documentation authority by the [Documentation Index](../README.md), and
security-sensitive implementation by
[Secure Development](../development/SECURE_DEVELOPMENT.md). This contract
selects no implementation mechanism.

## Conceptual Analysis Pipeline

```text
Browser Runtime
      ↓
Browser Adapter
      ↓
Frontend Analysis Core
      ↓
Analysis Results
      ↓
Presentation Adapter
      ↓
CLI / Desktop / VS Code / Web
```

This is a conceptual observation-to-consumer pipeline. Each arrow denotes a
transformation and ownership handoff across an explicit boundary: raw runtime
observations become normalized inputs, analysis produces result contracts, and
results become consumer-specific representations. The arrows are not a map of
Rust imports, a required call stack, or permission for a stable lower layer to
depend on an outer implementation.

## Dependency and Flow Semantics

### Observation and result flow

Browser observations normally move from Browser Runtime to Browser Adapter to
Core to Analysis Results to presentation. Data crossing each boundary changes
ownership and must satisfy the receiving contract.

### Runtime request and control flow

A product may request capture, analysis, cancellation, refresh, filtering, or
re-analysis through an explicit application boundary. Such requests may move
toward an adapter or Core operation, and responses may move outward. Bidirectional
runtime calls do not create reverse source dependencies or transfer semantic
ownership. No concrete interface mechanism is selected here.

### Source-code and compile-time dependency

Source dependencies point toward stable, browser-independent contracts. Core
does not import browser adapters, browser implementations, protocol models,
Analysis Result consumers, presentation adapters, or products. Outer layers
may consume approved contracts without gaining access to internals.

### Contract and semantic ownership

The layer that owns a contract defines its invariants and meaning; an
implementing or consuming outer layer does not. Data flow transfers custody of
a value, not authority to reinterpret it. Runtime invocation direction never
reverses contract ownership.

## Browser Runtime

The Browser Runtime is an external system being observed or controlled. It may
include a browser engine, page, worker, process, or remote debugging target.

**Responsibilities and semantic owner:** The runtime executes browser
behavior, exposes engine-specific state and events, and enforces its own
lifecycle and protocol semantics. Frontend Analysis does not own browser-engine
correctness.

**Permitted interaction:** An adapter may interact with the runtime through an
approved browser-specific boundary.

**Exclusions and prohibited dependencies:** The runtime is not part of Core,
does not define Core domain semantics, need not adopt this repository's module
boundaries, and cannot make raw protocol objects into Core domain models. This
contract imposes no repository source dependency on the external runtime.

## Browser Adapter

**Responsibilities and semantic owner:** A Browser Adapter owns its
browser-specific connection, protocol transport and lifecycle, decoding,
external-data validation, protocol interpretation, and normalization. It
preserves material source provenance, translates observations into approved
neutral inputs, translates approved application requests into browser-specific
operations, reports unsupported or lossy normalization honestly, and isolates
engine differences.

**Permitted dependencies:** An adapter may depend on approved,
browser-independent boundary contracts that it must produce, consume, or
conform to. This may include a future normalized-input contract accepted by
Core, provided the stable browser-independent side of the boundary owns that
contract. Such a contract dependency permits neither access to Core
implementation details or internals nor redefinition of Core semantics, and it
does not require the adapter to invoke Core directly. Concrete contract
placement and orchestration remain deferred. After focused approval, an adapter
may also depend on its browser-specific protocol facilities. Each browser
protocol remains isolated under its adapter.

**Exclusions and prohibited dependencies:** An adapter does not redefine Core
analysis semantics, own product UI behavior, perform presentation formatting,
hide browser-brand policy that changes findings, leak protocol object graphs
into Core, or promote a protocol type to a stable public domain type. No shared
generic adapter implementation hierarchy or protocol library is selected here.

## Frontend Analysis Core

**Responsibilities and semantic owner:** Core owns browser-independent domain
and analysis semantics, validates normalized domain inputs, performs analysis,
produces browser-independent evidence, findings, and diagnostics, enforces
analysis invariants, preserves determinism where practical, and exposes
reusable behavior through approved contracts. It remains testable without a
live browser where the analysis permits.

**Permitted dependencies:** Core may depend only on approved stable,
browser-independent contracts and genuinely lower-level facilities that obey
the architecture principles.

**Exclusions and prohibited dependencies:** Core has no browser protocol
imports, browser transport, browser-brand branching, browser connection
credentials, product lifecycle, presentation layout or styling, or assumption
of one caller. It must not depend on browser adapters, browser implementations,
React, Electron, Tauri, VS Code APIs, DOM or browser UI code, presentation
frameworks, or concrete CLI, desktop, web, editor, or other products. Direct
filesystem, network, or process access is prohibited unless a future approved
Core contract demonstrates Core ownership.

This contract defines no Core crate, module, trait, method, async behavior, or
public API.

## Analysis Results

Analysis Results form a domain contract boundary, not necessarily a separate
process, service, crate, schema, or active runtime layer.

**Responsibilities and semantic owner:** Result contracts carry
browser-independent analysis meaning, preserve evidence and provenance,
distinguish source observations from derived findings, represent uncertainty
and unsupported states honestly, and provide presentation-independent semantics
to consumers. Core owns their meaning and production; the result contracts own
evidence and diagnostic semantics.

**Permitted dependencies:** Results may depend only on approved
browser-independent domain contracts. Presentation adapters may consume them,
while owning any derived consumer representation; products own display and
interaction state.

**Exclusions and prohibited dependencies:** Results contain no React
components, product view state, primary browser-protocol payload contract,
display colors, icons, panel placement, or localized prose as domain evidence.
They cannot be mutated to rewrite meaning while being represented as unchanged.
No production result types, schemas, severity levels, or serialization are
defined here.

## Presentation Adapter

**Responsibilities and semantic owner:** A Presentation Adapter consumes
approved Analysis Results and owns mappings into consumer-facing view models or
transport representations. It may group, filter, sort, localize, format, and
visually encode results, while preserving traceability and distinguishing
derived presentation state from Core output. It adapts product requests to an
approved application boundary.

**Permitted dependencies:** It may depend on Analysis Result and application
contracts and on presentation technology selected through future approval.

**Exclusions and prohibited dependencies:** It does not hide re-analysis in
rendering, mutate evidence meaning, create Core findings from UI state,
interpret browser protocols except while separately acting through an adapter
boundary, access Core internals, or redefine diagnostic or compatibility
semantics. Its technology must not leak into Core; no framework is selected.

## Products

Potential products include a CLI, desktop application, VS Code extension, and
web application. This list is contextual and is not an implementation promise.

**Responsibilities and semantic owner:** A Product owns user interaction,
product lifecycle, presentation composition, command routing, configuration
entry, product-owned persistence, integration of approved adapters and Core
capabilities, and product-specific error presentation.

**Permitted dependencies:** It may compose approved adapters, result and
application contracts, and presentation adapters without reaching through
their public boundaries.

**Exclusions and prohibited dependencies:** A Product does not own Core
analysis meaning, directly mutate Analysis Results, place browser protocol
logic outside an approved Browser Adapter, use Core internals as a product API,
or introduce hidden architecture decisions for framework convenience. It must
not access raw browser protocol objects outside an adapter boundary.

## Application Orchestration

Application orchestration is a responsibility, not a mandatory additional
layer, crate, or service in the pipeline. It coordinates browser capture,
adapter lifecycle, Core invocation, cancellation or refresh, result delivery,
and product lifecycle. Its concrete owner is deferred and may later be a
product, application adapter, or another approved boundary.

Wherever it is placed, orchestration must not move browser protocol semantics
into Core, move Core semantics into presentation, or create circular source
dependencies. Its contracts require focused approval. UI re-analysis requests
must cross an explicit boundary; Core must not call concrete UI or Browser
Adapter implementations. No orchestration API is designed here.

## Boundary Rules

- Raw protocol input terminates at the Browser Adapter boundary.
- Only approved normalized browser-independent input enters Core.
- Core output crosses through Analysis Result contracts.
- Presentation-specific state begins at the Presentation Adapter boundary.
- Product-specific behavior remains outside Core.
- Material provenance survives normalization; loss is explicit.
- Unsupported browser behavior is represented honestly.
- No layer reaches through another layer to mutate internal state.
- Global state, utility modules, callbacks, and feature flags cannot bypass a
  boundary.
- A callback or inversion point is owned by the stable side of its boundary and
  implemented externally.
- Runtime request direction does not establish source dependency ownership.

## Cross-Cutting Capabilities

Logging and tracing, configuration, metrics, clocks, filesystem, network,
caching, feature flags, error transport, and test support each require a named
owner, an explicit boundary, and a demonstrated reason to cross layers. They
must not hide dependencies on outer implementations or reinterpret evidence or
diagnostics.

A helper imported by every layer is not automatically acceptable. A global
`common`, `shared`, `utils`, or service-locator facility must not become a
dependency bypass. A capability may be a genuinely domain-neutral lower-level
facility or an outer implementation of a stable-side-owned contract, subject
to focused review. This document chooses no logging, configuration, or
dependency-injection tool.

## Dependency Matrix

Rows are source owners or interacting systems; columns are targets. “Contract
consumption” means the source may depend only on the target's approved stable
contract definitions, not its implementation or internals. “Contract
production” means the source produces values governed by the target contract
without authority to redefine it. “External interaction” is runtime interaction
with an external system, not a repository source dependency. “Allowed” is an
internal self-relationship or explicitly permitted relationship that still
respects semantic ownership. “Prohibited” permits no direct source dependency,
implementation access, or semantic ownership transfer. “Not applicable” means
no meaningful repository dependency relationship is represented.

| From ↓ / To → | Browser Runtime | Browser Adapter | Core | Analysis Results | Presentation Adapter | Product |
| --- | --- | --- | --- | --- | --- | --- |
| Browser Runtime | Not applicable | External interaction | Prohibited | Prohibited | Prohibited | Prohibited |
| Browser Adapter | External interaction | Allowed | Contract consumption | Prohibited | Prohibited | Prohibited |
| Core | Prohibited | Prohibited | Allowed | Contract production | Prohibited | Prohibited |
| Analysis Results | Prohibited | Prohibited | Prohibited | Allowed | Prohibited | Prohibited |
| Presentation Adapter | Prohibited | Prohibited | Prohibited | Contract consumption | Allowed | Prohibited |
| Product | Prohibited | Contract consumption | Contract consumption | Contract consumption | Contract consumption | Allowed |

The Browser Runtime is external and is not required to depend on this
repository. Adapter “external interaction” covers protocol communication.
Browser Adapter contract consumption permits only conformance to or use of
stable browser-independent boundary contracts; it permits no Core implementation
or internal access and requires no direct runtime dependency from Adapter to
Core. Core contract production means Core produces Analysis Results according
to the result contract. Analysis Results remain a domain contract boundary and
are not required to be a separate crate. Product contract consumption remains
orchestration through approved boundaries: it grants access to neither raw
protocol objects nor Core internals. Runtime requests may travel opposite the
observation pipeline without changing this compile-time matrix.

## Representative Scenarios

Each verdict follows the owners and boundaries above. “Escalation” identifies
the review needed before a proposal may exceed the current contract.

### 1. CDP event type imported directly by Core

- **Verdict:** Prohibited.
- **Owner:** Browser Adapter owns CDP interpretation; Core owns analysis.
- **Boundary:** Core receives only approved browser-independent input.
- **Rationale:** A CDP type creates a protocol dependency in Core.
- **Escalation:** Redesign and explicit architecture review are required.

### 2. Core result rendered by React

- **Verdict:** Conditional.
- **Owner:** Presentation Adapter owns React-specific transformation; Core owns
  result meaning.
- **Boundary:** Analysis Result to Presentation Adapter.
- **Rationale:** Rendering is allowed outside Core; Core cannot import React.
- **Escalation:** Framework selection requires focused approval.

### 3. Adapter normalizes CDP and WebKit events into a shared input

- **Verdict:** Allowed when the shared input is an approved
  browser-independent contract.
- **Owner:** Separate adapters own protocol decoding and normalization; Core
  owns analysis semantics.
- **Boundary:** Browser Adapter to Core normalized-input boundary.
- **Rationale:** Adapters may consume or conform to the approved stable
  browser-independent contract without accessing Core implementation details,
  redefining Core semantics, or causing Core to branch on CDP or WebKit types.
  Loss and unsupported observations remain explicit; direct orchestration is
  unspecified.
- **Escalation:** The neutral contract and any protocol dependency require
  focused approval; no input type is defined here.

### 4. UI requests re-analysis through an application boundary

- **Verdict:** Allowed.
- **Owner:** Product owns the user request, orchestration coordinates it, and
  Core owns analysis.
- **Boundary:** Explicit application boundary, through which Browser Adapter
  and Core may each expose approved contracts to orchestration.
- **Rationale:** Runtime control flow does not make Core depend on the UI and
  does not decide concrete orchestration placement.
- **Escalation:** A concrete application API requires focused approval.

### 5. Common logging helper referenced by every layer

- **Verdict:** Conditional, and prohibited when it is a mixed-responsibility
  dependency bypass.
- **Owner:** A named capability owner must be established.
- **Boundary:** Domain-neutral lower-level facility or explicit capability
  contract.
- **Rationale:** Ubiquity alone does not justify reverse or mixed dependencies.
- **Escalation:** Focused infrastructure and ownership review is required.

### 6. Browser adapter adds a browser-only field directly to a Core result

- **Verdict:** Prohibited unless promoted by an approved browser-independent
  contract change.
- **Owner:** Adapter owns adapter-specific evidence; result contracts own result
  semantics.
- **Boundary:** Approved neutral provenance at the adapter-to-Core boundary.
- **Rationale:** An adapter cannot unilaterally extend Core meaning.
- **Escalation:** Explicit architecture and result-contract review is required.

### 7. Presentation adapter changes a diagnostic from uncertain to confirmed

- **Verdict:** Prohibited.
- **Owner:** Analysis Result contract owns diagnostic certainty.
- **Boundary:** Result-to-presentation boundary preserves meaning.
- **Rationale:** Presentation cannot redefine analysis certainty.
- **Escalation:** Any semantic change belongs in a focused Core/result-contract
  proposal, not presentation.

### 8. Product stores Analysis Results for later display

- **Verdict:** Allowed when persistence remains product-owned and preserves
  meaning and provenance.
- **Owner:** Product owns persistence; result contracts own stored meaning.
- **Boundary:** Product consumes Analysis Results.
- **Rationale:** Product storage does not become a Core dependency.
- **Escalation:** A shared serialization or storage contract requires focused
  approval.

### 9. Core emits a CSS color for each diagnostic

- **Verdict:** Prohibited.
- **Owner:** Presentation owns color; Core/result contracts own domain meaning.
- **Boundary:** Presentation derives visual encoding from Analysis Results.
- **Rationale:** CSS color is presentation state, not evidence.
- **Escalation:** A domain severity or category requires a separately approved
  contract; no values are defined here.

### 10. Feature flag enables CDP-specific behavior inside Core

- **Verdict:** Prohibited.
- **Owner:** Browser Adapter owns CDP behavior; Core owns browser-independent
  analysis.
- **Boundary:** Protocol behavior terminates at the adapter boundary.
- **Rationale:** Feature flags cannot bypass browser independence.
- **Escalation:** Redesign through the adapter boundary and explicit
  architecture review are required.

## Deferred Boundaries

Concrete Browser Adapter interfaces, normalized input models, evidence models,
result schemas, diagnostic taxonomy, application orchestration API,
cancellation and streaming models, synchronous versus asynchronous execution,
concurrency, process boundaries, IPC, serialization, persistent storage,
caching, logging implementation, error types, crate and module structure,
public Rust API, framework selection, and protocol libraries are deferred.

Deferred does not mean unrestricted. Each requires a focused approved Issue
before introduction.
