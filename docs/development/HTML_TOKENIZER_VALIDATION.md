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
counters exist. This foundation does not invent a second tokenizer trace or
infer cursor behavior from output shape.

A malformed fixture is a failed test. Comparison code must not compensate for
or normalize it.

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
