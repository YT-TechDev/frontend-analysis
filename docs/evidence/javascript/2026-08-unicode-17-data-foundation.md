# Unicode 17.0 data foundation — Issue #198 implementation evidence

Status: implementation evidence for the bounded Unicode qualification data
foundation. Repository/CI validation remains a separate completion gate.

## Frozen identities

```text
Unicode: 17.0.0
Unicode mirror commit: a363a170c5ecb1c509535f6730dd19e720443cd9
ECMA-262 snapshot: d89c03f2db8a597bc915b363a6518d0cc8acdbc0
```

Retained source blob identities:

```text
DerivedCoreProperties.txt
  f327784bf3956436efeb85e213fc637d7b3c0207
  1,134,783 bytes

extracted/DerivedGeneralCategory.txt
  41996d6348fdcd7f1a88127766b7e6b21b6159e3
  277,514 bytes

PropertyValueAliases.txt
  b92662eda2867baed56e55440e5187d52c1cb341
  81,858 bytes

table-nonbinary-unicode-properties.html
  2bf328116d7085f3e220c8325d0fb737b3bd19a6
  1,033 bytes

table-binary-unicode-properties.html
  8ab11640ed8bd6234a3c2b42a36a99847eee76ef
  10,528 bytes

table-binary-unicode-properties-of-strings.html
  1b6479a126902e04394f8bb684e7d3740bde608e
  702 bytes
```

The Unicode versioned release paths remain the primary publication identity.
The Unicode Git mirror is an independent exact-byte identity, not a replacement
for the versioned Unicode publication.

## Provenance model

The implementation keeps these identities distinct:

```text
upstream publication identity
!= retained Git object identity
!= working-tree byte identity
!= generated representation identity
!= runtime lookup result
```

The generator recomputes the Git blob object identity over
`blob <byte-length>\0<exact-bytes>` before parsing. `.gitattributes` protects the
retained evidence paths against checkout EOL normalization and prevents retained
upstream whitespace from being misclassified as project-authored whitespace
errors. It separately pins LF for the project-authored manifest, generator entry
points, and generated Rust output whose byte identity participates in deterministic
regeneration.

This matters because a Git object can be exact while a filtered working tree is
not. The retained-source contract therefore includes checkout preservation as an
explicit provenance boundary.

## Bounded semantic surface

The generated first-envelope data contains only:

```text
ID_Start                         145,916 code points
ID_Continue                      149,240 code points
General_Category=Zs                  17 code points
General_Category alias spellings     80
Script alias spellings               346
ES non-binary property aliases         6
ES binary property aliases            98
ES binary properties of strings        7
```

`ID_Start` is verified as a semantic subset of `ID_Continue` without requiring
identical physical range partitioning.

ECMAScript grammar-specific `$` / `_` handling is not folded into Unicode
`ID_Start`. `_` remains a Unicode `ID_Continue` fact and any additional grammar
role belongs to later lexical composition.

## Generation and independent equality

The generator is deterministic, standard-library-only, offline, and fail-closed
for missing or drifting retained inputs. Its generated Rust embeds the SHA-256
of `upstream-manifest.json`.

Independent verification does not import generator code. It separately parses
all retained sources and compares complete semantic membership/mappings against
the generated Rust representation.

The implementation gates are intentionally separate:

```text
exact retained bytes
-> deterministic generation
-> generated-output freshness
-> independent complete semantic equality
-> crate-private lookup behavior
-> repository Rust/WASM validation
```

A pass at one stage does not imply a pass at a later stage.

## License and notice boundary

Unicode's exact associated license notice and the pinned `tc39/ecma262`
license-routing document are retained separately and content-addressed.

For the ECMA table portions, the full Alternative Copyright Notice is retained as
`tools/unicode/ECMA262-COPYRIGHT.html`. Its provenance is tied to the exact
`ecmarkup 24.0.0` notice template resolved by the pinned ECMA-262 lockfile, plus
the 2026 annual year/document substitutions. See
`tools/unicode/UPSTREAM-LICENSING.md` for the exact identities and derivation
record.

## Core boundary

The production-facing data lookup remains private to
`frontend-analysis-core`. Lookup accepts a Unicode code point as `u32` so the
data layer does not silently equate Unicode code points with Rust `char` scalar
values.

The implementation introduces no public export, third-party dependency, new
crate/workspace member, build script, runtime file/network access,
serialization, async, concurrency, `unsafe`, parser integration, or browser
adapter behavior.

## Required completion validation

Before Issue #198 can be called complete, the exact candidate head must satisfy:

```text
python3 tools/unicode/generate_ecmascript_unicode.py --check
python3 tools/unicode/verify_generated_ecmascript_unicode.py
python3 -m unittest discover -s tools/unicode/tests -p 'test_*.py'
python3 .github/scripts/validate-rust-workspace-state.py .
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo metadata --offline --format-version 1 --locked
git diff --check
```

Relevant native/WASM validation, exact-head review, repository scope audit, and
license/notice review remain required by the active Issue and repository
validation contract. A missing or unavailable required gate must be reported as
such rather than converted into a completion claim.
