# HTML Research Provenance

Classification: task and evidence record; provenance-only; non-normative.

This ledger records sources actually used by the HTML research program. It does
not restate or decide HTML findings. See the
[HTML evidence record](../evidence/html/README.md) for conclusions and validation
status.

## Initial Sources

### WHATWG HTML Standard

- **Source:** HTML Standard
- **Source class:** Normative specification
- **Authority / version:** WHATWG Living Standard; no immutable snapshot is
  currently pinned by the high-level HTML evidence record
- **URL or stable identifier:** <https://html.spec.whatwg.org/multipage/>
- **Accessed / reviewed date:** 2026-08-20 (provenance review)
- **Used for:** Normative external authority for HTML parsing and authored-source
  behavior relevant to the project-owned HTML source-analysis workstream.
- **Evidence role:** `normative`
- **Related research / architecture:**
  [HTML evidence](../evidence/html/README.md),
  [Source Parser Ownership](../architecture/SOURCE_PARSER_OWNERSHIP.md)
- **Notes:** Browser or third-party parser behavior remains comparison evidence,
  not the semantic authority for Core contracts. Future capability work should
  pin a snapshot or commit when version drift would affect the evidence envelope.

### WHATWG Infra Standard

- **Source:** Infra Standard
- **Source class:** Normative specification
- **Authority / version:** WHATWG Living Standard; referenced by the current HTML
  evidence baseline
- **URL or stable identifier:** <https://infra.spec.whatwg.org/>
- **Accessed / reviewed date:** 2026-08-20 (provenance review)
- **Used for:** Referenced common algorithms, terminology, and data-model concepts
  underlying the HTML Standard where those definitions affect HTML research.
- **Evidence role:** `normative`
- **Related research / architecture:** [HTML evidence](../evidence/html/README.md)
- **Notes:** Record an immutable upstream identity in future work when a specific
  Infra revision materially affects a conclusion.

### WHATWG Encoding Standard

- **Source:** Encoding Standard
- **Source class:** Normative specification
- **Authority / version:** WHATWG Living Standard; referenced by the current HTML
  evidence baseline
- **URL or stable identifier:** <https://encoding.spec.whatwg.org/>
- **Accessed / reviewed date:** 2026-08-20 (provenance review)
- **Used for:** Referenced encoding behavior and terminology where HTML parsing
  research depends on the HTML Standard's encoding integration.
- **Evidence role:** `normative`
- **Related research / architecture:** [HTML evidence](../evidence/html/README.md)
- **Notes:** This entry records source provenance only; it does not assert that
  the current bounded HTML slice implements the full Encoding Standard.

## Adding Entries

Use the field set defined in [Research Provenance](README.md). Add Web Platform
Tests, browser-engine implementations, Issues, experimental observations, or
secondary references only when their identity and actual research use are
verified.
