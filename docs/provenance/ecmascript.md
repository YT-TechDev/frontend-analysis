# ECMAScript Research Provenance

Classification: task and evidence record; provenance-only; non-normative.

This ledger records sources actually used by the current ECMAScript research
program. It does not restate or decide ECMAScript findings. See the
[JavaScript / ECMAScript evidence records](../evidence/javascript/README.md) and
[JavaScript / ECMAScript architecture](../architecture/JAVASCRIPT_ARCHITECTURE.md)
for their separate responsibilities.

## Initial Sources

### ECMA-262 — ECMAScript 2026

- **Source:** ECMA-262, ECMAScript 2026
- **Source class:** Normative specification
- **Authority / version:** 17th edition, June 2026; project qualification snapshot
  `tc39/ecma262@d89c03f2db8a597bc915b363a6518d0cc8acdbc0`
- **URL or stable identifier:**
  <https://github.com/tc39/ecma262/commit/d89c03f2db8a597bc915b363a6518d0cc8acdbc0>
- **Accessed / reviewed date:** 2026-08-20 (provenance review)
- **Used for:** Primary normative baseline for the current ECMAScript Standard
  Qualification work, including grammar, profile, and static-validity
  obligations.
- **Evidence role:** `normative`
- **Related research / architecture:**
  [ECMAScript evidence baseline](../evidence/javascript/README.md),
  [first Standard Qualification validation foundation](../evidence/javascript/2026-08-first-standard-qualification-validation-foundation.md),
  [JavaScript / ECMAScript architecture](../architecture/JAVASCRIPT_ARCHITECTURE.md)
- **Notes:** The repository distinguishes the selected immutable snapshot from
  later draft material. This provenance entry does not convert post-2026 draft
  content into ES2026 normative evidence.

### Test262

- **Source:** Test262 ECMAScript conformance test suite
- **Source class:** Standards / test suite
- **Authority / version:** Pinned challenge-evidence revision
  `tc39/test262@3655e7464de3d52643ecddd4b5f9f4f3e7f62398`
- **URL or stable identifier:**
  <https://github.com/tc39/test262/commit/3655e7464de3d52643ecddd4b5f9f4f3e7f62398>
- **Accessed / reviewed date:** 2026-08-20 (provenance review)
- **Used for:** Candidate-independent challenge evidence for the bounded
  ECMAScript Standard Qualification validation envelope after independent
  mapping to project-owned normative rule identities.
- **Evidence role:** `corroborating`
- **Related research / architecture:**
  [first Standard Qualification validation foundation](../evidence/javascript/2026-08-first-standard-qualification-validation-foundation.md)
- **Notes:** Test262 pass/fail behavior and metadata do not define Frontend
  Analysis semantics. The current evidence contract also verifies selected path,
  Git blob, relevant frontmatter, and effective-source transformation rather
  than treating a Test262 path alone as source identity.

### Unicode Standard 17.0.0

- **Source:** The Unicode Standard, Version 17.0.0, including the Unicode
  Character Database data used by the bounded ECMAScript qualification
  foundation
- **Source class:** Normative specification / official data publication
- **Authority / version:** Unicode 17.0.0; exact-byte mirror identity currently
  recorded by the project as commit
  `a363a170c5ecb1c509535f6730dd19e720443cd9`
- **URL or stable identifier:** <https://www.unicode.org/versions/Unicode17.0.0/>
- **Accessed / reviewed date:** 2026-08-20 (provenance review)
- **Used for:** Unicode property and identifier data required by the bounded
  ECMAScript qualification profile, including `ID_Start`, `ID_Continue`,
  General Category aliases, Script aliases, and ECMAScript-relevant Unicode
  property tables.
- **Evidence role:** `normative`
- **Related research / architecture:**
  [Unicode 17.0 data foundation](../evidence/javascript/2026-08-unicode-17-data-foundation.md),
  [first Standard Qualification validation foundation](../evidence/javascript/2026-08-first-standard-qualification-validation-foundation.md)
- **Notes:** The versioned Unicode publication is the primary publication
  identity. The project-recorded Git mirror commit is retained as an independent
  exact-byte identity, not as a replacement for the Unicode publication.
  Individual Unicode data properties retain their own normative/informative
  status; this ledger entry does not flatten those distinctions.

## Adding Entries

Use the field set defined in [Research Provenance](README.md). Add TC39 proposals,
engine/runtime implementations, academic papers, OSS analyzers, Issues, or other
materials only when the repository can identify the source accurately and the
research actually relied on it.
