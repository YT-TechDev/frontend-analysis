# HTML Tokenizer Validation

## Purpose and Authority

This document is the focused runner and extension contract for the first
project-owned HTML tokenizer validation foundation established by Issue #112.
It specializes the repository-wide validation rules in `VALIDATION.md` and the
source-parser ownership contract. It does not redefine production token,
diagnostic, completion, resource, source-coordinate, or public API semantics.

The validation foundation is candidate-independent. Pinned WHATWG algorithms
and accepted Core contracts inform manually authored fixtures; the tokenizer
under test cannot generate, bless, rewrite, sort, or repair its own expected
results.

## Initial Inventory

The initial corpus contains exactly 72 fixtures:

| Prefix | Count | Responsibility |
| --- | ---: | --- |
| `PRE-` | 10 | UTF-8 input and preprocessing evidence |
| `TOK-` | 12 | Clean supported token observations |
| `ERR-` | 17 | One primary case for every approved diagnostic code |
| `UNSUP-` | 14 | Explicit unsupported or deferred capability boundaries |
| `RES-` | 9 | Resource and invalid-configuration states |
| `ADV-` | 10 | Adversarial and cross-cutting invariants |

The IDs are stable and contiguous within each initial category. Adding or
removing an initial fixture requires an Issue update or explicit review that
identifies the changed capability or risk. Corrected defects add a durable
`REG-<issue>-<slug>` fixture and do not renumber the initial inventory.

## Fixture Authority

Each fixture declares:

- a stable ID and category;
- explicit static UTF-8 source bytes;
- exact zero-based half-open byte ranges;
- expected preprocessing evidence;
- expected tokens and nested authored evidence;
- expected diagnostics, handling, recovery, and subject relation;
- exact processed-prefix and unprocessed-suffix coverage;
- complete, unsupported, resource-limited, invalid-configuration, or internal
  invariant completion meaning;
- explicit limits and usage assertions.

Fixture self-validation runs before candidate execution. It rejects duplicate
IDs, invalid UTF-8, invalid boundaries, authored-byte mismatches, token overlap,
invalid nesting, invalid EOF meaning, broken coverage, unordered diagnostics,
invalid subjects, stopped-diagnostic contradictions, BOM evidence outside the
processed prefix, and inconsistent completion policy.

The coverage pair accounts for every source byte exactly once as processed or
unprocessed, and emitted top-level source ranges must not overlap. Emitted token
ranges are not required to cover bytes that the HTML algorithm deliberately
ignores or abandons, such as an incomplete tag builder. Therefore candidate
proof that consume and reconsume neither duplicate nor omit input belongs to the
#113 state-machine validation, where cursor transitions and committed resource
counters exist. This foundation does not invent a second tokenizer or infer
cursor behavior from candidate output.

A malformed fixture is a failed test. Comparison code must not compensate for
or normalize it.

## Transition-Step Accounting

Expected `usage.transition_steps` is authored directly against the pinned
WHATWG states approved by #109 and the #111 counting rule: one transition step
per attempted specification-state dispatch, including a dispatch caused by a
reconsume instruction. It is never derived from UTF-8 byte length, Unicode
scalar count, emitted token count, diagnostic count, or a sum of those
quantities.

The accounting boundary is precise:

- CR and CRLF preprocessing may produce one normalized input unit;
- a leading BOM skip and preprocessing-only diagnostic do not add tokenizer
  transitions;
- examining a normalized unit under a state is one transition;
- one reconsume instruction causes one additional state examination of the same
  authored input unit;
- examining conceptual EOF under the current state is one transition;
- token or diagnostic emission performed inside that dispatch is not an
  additional transition;
- a non-transition resource refusal commits the examining transition but
  refuses its specific sub-effect without partial mutation;
- an attempted transition rejected by the `TransitionSteps` limit does not
  increment committed transition usage.

`crates/frontend-analysis-core/src/html/tokenizer/validation/corpus/transition_audit.rs`
contains an independently authored 72-entry committed-count inventory and
mechanically checks exact corpus-ID and count agreement. The complete reviewable
derivation is distributed beside the fixtures: PRE, TOK, ERR, and ADV cases
have ordered state traces; the context-changing UNSUP group has one explicit
formula parameterized by authored name length; and every RES case records
preprocessing, attempted and committed transitions, refusal operation,
attempted resource value, and source boundary.

## Resource Ownership

Active token builders own interpreted evidence that can become emitted output.
Tag names, attribute names, attribute values, and collected character output
therefore count toward `RetainedInterpretedBytes` while their builders are
active. They are not temporary buffers.

`TemporaryBufferBytes` is reserved for a genuine state-local scratch buffer
whose contents are not yet retained output evidence, such as a future
character-reference or script-matching temporary buffer. The first bounded Data
capability owns no such buffer, so the initial 72-fixture corpus intentionally
contains no `TemporaryBufferBytes` exhaustion result. Execution coverage for
that resource is:

```text
Blocked — no approved first-slice state owns a genuine state-local temporary buffer.
```

It must not be reported as Passed or simulated through a retained tag/name/value
builder.

## UNSUP-004 Correction

The initial `UNSUP-004` fixture (`"<?x>"`, processing-instruction boundary)
omitted the mandatory `TagOpen('?')` parse-error observation. It recorded no
diagnostics and a trigger spanning `[0,2)`, even though `TagOpen` examining
`?` is the same dispatch that the pinned WHATWG Tag open state requires to
emit `UnexpectedQuestionMarkInsteadOfTagName` unconditionally, regardless of
what follows.

`ERR-006` (`"<?"`) already recorded that diagnostic correctly for the
identical dispatch and served as an independent same-dispatch cross-check.
`UNSUP-004` was corrected to match `ERR-006`'s prefix: one
`UnexpectedQuestionMarkInsteadOfTagName` diagnostic at `[1,2)`, a processed
prefix of `[0,2)`, and an `Unsupported(ProcessingInstruction)` trigger of
`[2,2)`, with the trailing authored `"x>"` left in the unprocessed suffix.
`transition_steps` remains `2`; the correction changes recorded diagnostics
and coverage, not step accounting.

The correction was derived directly from the #109/#111 approved contracts and
the pinned WHATWG snapshot, cross-checked against `ERR-006`, and not from any
production tokenizer's output. The stable fixture ID and the 72-fixture
initial inventory are unchanged.

## Emission-Conditioned Diagnostic Contract

The #111 domain contract distinguishes observation-conditioned diagnostics
from emission-conditioned diagnostics. `EndTagWithAttributes` and
`EndTagWithTrailingSolidus` are the only first-slice diagnostic codes
classified emission-conditioned: their underlying parse-error fact exists
only once the corresponding end-tag token emission commits. A final
`HtmlTokenizerRunResult` containing either diagnostic must relate it to
`EmittedToken { token_index }` referencing an emitted `HtmlToken::Tag` of
kind `End` carrying the corresponding structured evidence (non-empty
attributes, or a recorded trailing solidus). If the end-tag token does not
emit, the diagnostic itself must not appear in the final result.

Every other first-slice diagnostic code remains observation-conditioned: its
underlying fact becomes true once the approved specification observation and
its committed recovery sub-effect occur, and it does not become false merely
because a later, independent top-level token emission is refused. Such a
diagnostic may validly finish with `AbandonedInput { region }` when the tag
was completed or partially constructed but never emitted.
`Recovered(CompletedTagWithMissingAttributeValue)` denotes builder/
token-construction completion, not emitted-vector insertion, and does not
itself require token emission.

This is a project-owned contract clarification rather than a pinned-standard
requirement: the pinned WHATWG snapshot distinguishes end-tag-token creation
from end-tag-token emission and defines both diagnostics as conditions on the
emitted end-tag token, but assigns no diagnostic-subject or termination model
of its own.

`crates/frontend-analysis-core/src/html/tokenizer/result.rs` enforces this at
`HtmlTokenizerRunResult::new` through the crate-private
`HtmlTokenizerDiagnosticCode::is_emission_conditioned` classifier and
dedicated `HtmlTokenizerRunContractError` variants. The existing `ERR-016`
and `ERR-017` fixture location/context policy (first authored attribute
name / `AttributeName`, and the authored trailing solidus /
`SelfClosingStartTag`) is unchanged, and the 72-fixture initial corpus is
unchanged. Supplemental cross-product regression coverage for this
distinction is deferred to a separate #112 follow-up.

## Actual Observation and Comparison

The test-only observation adapter reads crate-private production accessors and
builds one immutable canonical observation. It may slice the retained source by
already validated anchors. It must not search the source, replay recognition,
repair ranges, sort output, discard partial evidence, or construct expected
fixtures.

Structural field comparison is authoritative. Debug strings, snapshots, hashes,
serialized bytes, allocation addresses, and enum discriminants are not oracles.
Mismatch reports identify the fixture and first owned semantic path.

Repeated candidate validation must run each fixture at least three times with
identical source and limits. It must also repeat with another `SourceId`; source
identity must remain validated by production contracts but must not alter the
semantic observation. Before #113 provides the candidate entry point, #112
proves this comparison path using a production-constructed empty complete run;
full-corpus candidate execution remains blocked rather than being reported as
Passed.

## Deterministic Generated Inputs

The standard-library-only generator uses a fixed ordered alphabet containing
ASCII letters and digits, tokenizer delimiters, whitespace, FF, NUL, quotes,
and selected multi-byte UTF-8 scalars.

The bounds are fixed:

- maximum 4,096 cases per focused test;
- maximum 64 raw UTF-8 bytes per case;
- deterministic seed corpus and enumeration order;
- no ambient randomness, clock, filesystem, network, thread scheduling, or
  external process input.

Before Issue #113 supplies a tokenizer entry point, tests validate generator
count, order, UTF-8 validity, bounds, and minimization order. After integration,
generated cases must validate panic freedom, explicit transition-limit
termination, source-range validity, coverage partition, and repeated semantic
determinism.

Generated testing is bounded evidence, not exhaustive HTML conformance or a
performance claim.

## Panic and Termination

Production code must not add `catch_unwind`. A test harness may catch a panic
only to identify a failing fixture or generated case; the caught panic remains a
failure.

Termination is governed by the deterministic transition-step resource limit,
not wall-clock timeout threads. A candidate must return an explicit incomplete
resource result before exceeding policy.

## Fuzz Posture

No `cargo-fuzz`, libFuzzer, nightly toolchain, separate fuzz package, or fuzz
dependency is approved by Issue #112.

Future separately approved targets are:

- `html_tokenizer_valid_utf8`: arbitrary valid UTF-8 with panic, termination,
  range, coverage, and determinism checks;
- `html_tokenizer_acquisition_boundary`: byte acquisition and UTF-8 rejection
  before `SourceText` construction.

Until separately approved and implemented, fuzz execution is recorded as:

```text
Not run — fuzz infrastructure intentionally not approved.
```

Every minimized fuzz failure becomes a durable regression fixture before the
defect is closed.

## Differential Posture

No external parser or browser is used by the initial foundation. Future
comparison is non-authoritative, version-pinned, and isolated from Core types.
Disagreement is evidence requiring investigation; it cannot rewrite gold
expectations automatically.

For Issue #112, differential execution is:

```text
Not applicable — no external comparison dependency or adapter is in scope.
```

## Commands

Run from the repository root:

```bash
python3 .github/scripts/validate-rust-workspace-state.py .
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo metadata --offline --format-version 1 --locked
```

The production workspace validator must print exactly `production`. The suite
must remain offline and dependency-free unless a later focused dependency review
explicitly changes that contract.

## Extension Review

A fixture or helper change must identify:

1. the pinned algorithm, accepted contract, defect, or risk it validates;
2. why the expected values are independent from candidate output;
3. exact source bytes and byte ranges;
4. expected token, diagnostic, completion, and coverage meaning;
5. deterministic and resource impact;
6. whether an existing ID changes meaning;
7. regression handling and future fuzz relevance;
8. final validation status and intentionally unavailable checks.

Stop and return to architecture review if the change requires a new dependency,
fuzz toolchain, workspace member, build script, feature, external parser,
browser runner, production serialization, public export, async, concurrency,
`unsafe Rust`, or broader tokenizer capability.
