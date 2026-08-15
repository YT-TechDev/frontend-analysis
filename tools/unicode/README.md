# ECMAScript Unicode qualification data

This directory owns the offline, deterministic data-generation boundary for the
first ECMAScript Standard Qualification envelope.

The retained upstream inputs remain the semantic-data authority. Generated Rust
is a project representation derived from those inputs and is not a replacement
for their provenance.

## Frozen source envelope

Unicode is pinned to Unicode 17.0.0. The primary publication identity is the
versioned Unicode UCD release, while the `unicode-org/unicodetools` mirror at
commit `a363a170c5ecb1c509535f6730dd19e720443cd9` supplies an independent Git blob
identity for the exact retained bytes.

ECMAScript property vocabulary is pinned to ECMA-262 snapshot
`d89c03f2db8a597bc915b363a6518d0cc8acdbc0`.

`upstream-manifest.json` records the exact retained paths, Git blob identities,
byte sizes, Unicode publication URLs, version headers, and ECMA table IDs.
Generation fails closed if any of those identities drift.

The six retained semantic inputs are:

- `inputs/DerivedCoreProperties.txt`;
- `inputs/extracted/DerivedGeneralCategory.txt`;
- `inputs/PropertyValueAliases.txt`;
- `inputs/ecma262/table-nonbinary-unicode-properties.html`;
- `inputs/ecma262/table-binary-unicode-properties.html`; and
- `inputs/ecma262/table-binary-unicode-properties-of-strings.html`.

The retained UCD files are complete pinned upstream files rather than
project-authored slices. This keeps source identity and regeneration auditable
without introducing another derivation format.

## Selected semantic surface

Generation is intentionally bounded to:

- Unicode `ID_Start` membership;
- Unicode `ID_Continue` membership;
- Unicode `General_Category=Zs` membership;
- Unicode General_Category value names and aliases;
- Unicode Script value names and aliases;
- the pinned ES2026 non-binary Unicode property names and aliases;
- the pinned ES2026 binary Unicode property names and aliases; and
- the seven pinned ES2026 binary properties of strings.

The generator does not add ECMAScript grammar-specific `$` or `_` rules to
Unicode `ID_Start`. `_` remains present only where Unicode itself places it,
including `ID_Continue`. Grammar composition belongs to later ECMAScript lexical
logic.

Full RegExp property membership, normalization, case folding, locale/Intl data,
emoji matching sequences, parser behavior, and browser/runtime integration are
outside this boundary.

## Git checkout invariant

The retained sources and notices are content-addressed over exact bytes.
`.gitattributes` disables Git text normalization and whitespace-error
classification only for those exact-evidence paths.

This is a provenance requirement, not a repository-wide text policy. In
particular, the upstream Unicode `PropertyValueAliases.txt` contains retained
trailing whitespace that must not be silently normalized or "fixed".

A separate narrow attribute rule forces LF for the project-authored manifest,
generator/verifier entry points, and generated Rust output. Those files are not
upstream evidence, but their exact working-tree bytes participate in manifest
hashing, executable shebang behavior, or generated-output freshness checks.

## Generation and verification

Generation is standard-library-only and performs no network or filesystem reads
outside the repository inputs it is explicitly given.

```bash
python3 tools/unicode/generate_ecmascript_unicode.py
python3 tools/unicode/generate_ecmascript_unicode.py --check
python3 tools/unicode/verify_generated_ecmascript_unicode.py
python3 -m unittest discover -s tools/unicode/tests -p 'test_*.py'
```

`generate_ecmascript_unicode.py` verifies source identity before parsing and
emits `crates/frontend-analysis-core/src/ecmascript/unicode_generated.rs`.

`verify_generated_ecmascript_unicode.py` is intentionally independent of the
generator. It reparses the retained sources without importing generator code and
requires complete equality between the source-derived semantic sets/mappings and
the generated Rust representation.

## Runtime boundary

The Rust lookup is private to the Core and browser-independent. It accepts a
Unicode code point as `u32`; it does not use `char` as the semantic-data boundary,
because Rust Unicode scalar values are not the same domain as all Unicode code
points.

No public export, dependency, build script, runtime network/file I/O, parser,
browser protocol, serialization, async, concurrency, or `unsafe` capability is
introduced here.

## Upstream notices

See [UPSTREAM-LICENSING.md](UPSTREAM-LICENSING.md). Exact upstream routing/license
files are retained separately from the project-maintained provenance explanation
and from the rendered ECMA redistribution notice.
