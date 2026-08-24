# HTML Core Analysis: First Vertical Slice

## Purpose and Authority

This document describes the first Core-integrated HTML analysis vertical
slice implemented under Issue #116, on the architecture approved in the
durable maintainer-decision comment on #116 and
[ADR 0007](../decisions/0007-own-lossless-source-parsers.md). It specializes
[Source Parser Ownership](../architecture/SOURCE_PARSER_OWNERSHIP.md) and does
not redefine tokenizer contracts owned by
[HTML Tokenizer Validation](HTML_TOKENIZER_VALIDATION.md) or parser contracts
owned by [HTML Analysis Parser](HTML_ANALYSIS_PARSER.md). It does not claim
complete HTML parsing, tree construction, or DOM behavior.

Issue #116 is complete. This document remains the implementation guide for that
historical/current explicit-start-tag sibling capability. Future HTML
tree-construction architecture is now owned by
[ADR 0010](../decisions/0010-html-tree-construction-architecture.md) and the
specialized normative
[HTML Tree-Construction Architecture](../architecture/HTML_TREE_CONSTRUCTION.md).
Those records do not retroactively change this bounded operation.

## Capability

The operation connects retained `SourceText` through the project-owned HTML
tokenizer and the project-owned explicit-start-tag analysis parser into one
synchronous, crate-visible Core operation that answers exactly the
already-approved question:

> Which explicit start-tag occurrences are recognized in the retained source,
> and what exact raw half-open ranges identify each complete authored tag and
> raw tag-name spelling?

```text
&SourceText
+ HtmlTokenizerLimits
        ↓
Core-owned HTML analysis operation
        ↓
project-owned tokenizer
        ↓
validated HtmlTokenizerRunResult
        ↓
project-owned explicit-start-tag analysis parser
        ↓
Core validation of projected source-backed occurrence evidence
        ↓
HtmlExplicitStartTagAnalysis
```

The operation is implemented as
`crate::html::analysis::analyze_html_explicit_start_tags` in
`crates/frontend-analysis-core/src/html/analysis.rs`, wired from
`crates/frontend-analysis-core/src/html/mod.rs`. This is a capability-specific
integration, not a generic parser-event stream, syntax tree, or universal
Core-analysis entry point.

## Input Ownership

The operation borrows `&SourceText`; the caller retains ownership. The
operation does not clone the complete source. The returned
`HtmlExplicitStartTagAnalysis` may outlive the caller's `SourceText` handle
through the existing `SourceAnchor` ownership contract, which privately
retains immutable source storage independently of the caller's handle.

## Configuration

The existing `HtmlTokenizerLimits` is reused unchanged as the bounded
execution configuration. No `HtmlAnalysisLimits`, `HtmlAnalysisBudget`,
`HtmlParserLimits`, or other configuration wrapper is introduced: this first
Core-integrated capability adds no independently unbounded parser or
integration state.

These tokenizer limits remain specific to this capability. ADR 0010 and
`HTML_TREE_CONSTRUCTION.md` do not reinterpret them as tree-construction limits
or normative HTML constants.

## Internal Stages

Tokenization (`html::tokenizer::producer::tokenize`) and analysis parsing
(`html::parser::analyze_explicit_start_tags`) remain internal implementation
stages. The callable Core boundary exposes neither tokenizer state machines,
mutable parser internals, token-builder state, nor an external parser
abstraction.

The future tree-construction architecture may use coordinated/resumable token
production where its own theorem requires it. That does not make the completed
batch-tokenizer boundary incorrect for this bounded sibling capability.

## Result Ownership

The operation reuses the existing #114/#115-owned `HtmlExplicitStartTagAnalysis`
result rather than introducing a duplicate Core wrapper result. The result
continues to own the validated `HtmlTokenizerRunResult` by value and the
deterministic explicit authored start-tag occurrence vector. Tokenizer
completion, diagnostics, coverage, unsupported evidence, and resource evidence
are not duplicated into a second Core result hierarchy.

The authored occurrence domain is not a constructed-node domain. `SourceId`,
authored ranges, and `origin_token_index` from this result must not be promoted
to constructed-node identity merely because later tree work needs node IDs.

## Core Source-Evidence Validation

The tokenizer and parser remain the semantic owners of token, diagnostic,
coverage, completion, unsupported, and resource-run invariants; this boundary
does not duplicate that validator. It adds exactly one additional
responsibility: validating the parser-projected occurrence evidence against
the exact supplied `SourceText`, implemented by the crate-private
`validate_occurrence_evidence` helper. For every projected occurrence, for
each of the `CompleteTag` and `RawName` roles, the boundary verifies, in
order:

1. **Source identity** — the anchor's `source_id()` equals the supplied
   `SourceText::id()`.
2. **Range revalidation** — the anchor's already-projected range revalidates
   through `SourceText::anchor(start, end)` against the exact supplied
   source.
3. **Fragment reconciliation** — the revalidated anchor's fragment equals the
   occurrence anchor's own retained fragment.

After both roles validate, the boundary checks **containment**: the raw-name
range lies within the complete authored tag range.

This is validation of already-projected evidence, not source discovery. The
boundary performs no source search, delimiter scan, endpoint reconstruction,
decoded-length inference, or retokenization; it revalidates only the exact
ranges the parser already projected.

## Boundary Errors

`HtmlStartTagAnalysisError` (crate-private) is the smallest error vocabulary
needed for failures owned by this boundary:

- `ParserContract(HtmlAnalysisParserContractError)` — the retained parser
  reported its own internal contract violation.
- `OccurrenceSourceIdentityMismatch { occurrence_index, role, expected, actual }`
- `OccurrenceSourceRangeInvalid { occurrence_index, role, error }` (wraps
  `SourceRangeError`)
- `OccurrenceSourceContentMismatch { occurrence_index, role }`
- `OccurrenceContainmentViolation { occurrence_index }`

`HtmlOccurrenceEvidenceRole` distinguishes `CompleteTag` from `RawName`. Every
variant carries only structural evidence (indices, roles, `SourceId`, and
`SourceRangeError`); `Debug` and `Display` never expose arbitrary authored
source content. A parser-internal contract failure and a Core source-evidence
validation failure are both distinct `Err` outcomes, never clean success or
clean absence.

## Completion, Diagnostics, and Resource Propagation

The integration does not reinterpret or upgrade tokenizer meaning:

```text
Tokenizer Complete                              → retained Complete
Tokenizer Complete with diagnostics/recovery     → retained unchanged
Tokenizer Incomplete(UnsupportedCapability)      → retained unchanged
Tokenizer Incomplete(ResourceLimit)              → retained unchanged
Tokenizer Incomplete(InvalidConfiguration)       → retained unchanged
Tokenizer Incomplete(InternalInvariantFailure)   → retained unchanged
Parser/Core contract failure                     → operation Err
```

Zero projected occurrences from an incomplete tokenizer result remain
incomplete; they are never represented as clean absence. No duplicate Core
completion enum or diagnostic vocabulary is introduced.

Future tree construction is governed by the same monotonicity principle but has
its own specialized rule that supported recovery and diagnostics may coexist
with a complete tree result. That later rule does not change this operation's
existing tokenizer lifecycle semantics.

## Abort and Cancellation

No independent cancellation or abort state is introduced. Caller-driven
cancellation is not applicable to this synchronous operation.
`InternalInvariantFailure` is not reinterpreted as cancellation.

ADR 0010 does not add cancellation to this capability. Any future tree
cancellation/abort mechanism remains separately scoped.

## Determinism

Equal source bytes, `SourceId`, `HtmlTokenizerLimits`, the explicit-start-tag
analysis capability, and implementation revision produce equivalent analysis
meaning: occurrence count, order, `origin_token_index`, complete and raw-name
ranges, raw spelling, tokenizer completion, diagnostics, coverage, and
resource/usage evidence.

This deterministic authored-occurrence meaning is not a promise of future
constructed-node identity encoding or cross-result node stability.

## Native Demonstration Boundary

Tests are the native demonstration; no binary, example target, CLI, or
filesystem/network acquisition path is added. Focused tests in
`crates/frontend-analysis-core/src/html/analysis/tests.rs` include a native
UTF-8/raw-spelling vertical slice (`é<DiV>x<span>`, independently computed
byte ranges), a raw-coordinate projection case, a result-lifetime case
proving the returned analysis remains valid after the caller's `SourceText`
handle drops, completion/diagnostics/unsupported/resource-limit/invalid-
configuration propagation cases, and direct corruption tests against
`validate_occurrence_evidence` for foreign `SourceId`, invalid range, content
mismatch, and containment violation.

## Candidate-Independent and Generated Validation

`crates/frontend-analysis-core/src/html/tokenizer/validation/core_analysis_gate.rs`
is a test-only module beside the tokenizer's own candidate-independent tests,
reusing their private fixture/gold types without widening visibility or
duplicating the corpus. It runs the complete 76-fixture candidate-independent
corpus (72 initial plus 4 supplemental `REG-` fixtures) and the existing
bounded, deterministic, dependency-free 4,096-case generator directly through
`SourceText -> analyze_html_explicit_start_tags(...)`, with no
`catch_unwind`, so a production panic on any case fails the test naturally.
Expected occurrences are derived independently from the existing #112 gold,
never from production output. This is an additional validation layer on top
of the existing direct parser gate (`parser_gate.rs`), not a replacement for
it.

The later TC-S1 candidate-independent validation is a separate architecture
record under #117. It does not alter these tests or convert architecture GOLD
into production implementation.

## WASM Compile-Only Status

No `wasm-bindgen`, JavaScript binding, serialization, or product wrapper is
introduced. Runtime WASM behavior is not claimed by a compile-only check.

The new tree-construction architecture likewise creates no WASM runtime promise.

## Public Visibility

`analyze_html_explicit_start_tags`, `HtmlStartTagAnalysisError`, and
`HtmlOccurrenceEvidenceRole` are all `pub(crate)`, as are the reused
`HtmlExplicitStartTagAnalysis` and `HtmlExplicitStartTagOccurrence` types.

```text
public-export delta: 0
```

No public HTML parser or analysis API, serialization contract, ABI
commitment, or JavaScript binding is introduced by this capability.

ADR 0010 and the specialized tree-construction contract also create no public
API merely by being accepted.

## Limitations

This capability answers only the explicit authored start-tag occurrence
question. It does not build an open-element stack, match start and end tags,
infer parent/child relationships or nesting depth, or synthesize implied
structure from token order.

The approved tree-construction architecture does not broaden this result or
claim that TC-S1 has been implemented.

## #117 Tree-Construction Boundary

Historically, matching, nesting, open-element stacks, implied structure, foster
parenting, adoption-agency behavior, foreign-content tree semantics, and
synthesized provenance were deferred to #117 while #116 finished the first
vertical slice. #116 is now complete, and the later #348/#117 research,
validation, and maintainer decision have resolved the durable architecture
boundary.

Future HTML tree construction is now governed by:

- [ADR 0010 — Define HTML Tree-Construction Architecture](../decisions/0010-html-tree-construction-architecture.md), which records why Candidate C was selected; and
- [HTML Tree-Construction Architecture](../architecture/HTML_TREE_CONSTRUCTION.md), which defines the specialized normative invariants.

TC-S1 — Disabled-Scripting Document Shell Construction — is the first approved
bounded production candidate, but production placement and implementation remain
separate gates. This document continues to own only the implemented first
explicit-start-tag Core slice.
