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
- **Authority / version:** WHATWG HTML pinned commit
  `508a037333d8a1806504303aeb489d931fabbef6`; source blob
  `68dbcb98bbe1001c6ae2531be2368c608fbafddd`
- **URL or stable identifier:**
  <https://github.com/whatwg/html/commit/508a037333d8a1806504303aeb489d931fabbef6>
- **Accessed / reviewed date:** 2026-08-24 (post-vertical-slice HTML research and
  architecture review)
- **Used for:** Normative external authority for HTML parsing, tree-construction,
  recovery, parse-context, and authored/non-authored provenance conclusions used
  by the #348 research program and #117 architecture work.
- **Evidence role:** `normative`
- **Related research / architecture:**
  [HTML evidence](../evidence/html/README.md),
  [HTML tree-construction architecture](../architecture/HTML_TREE_CONSTRUCTION.md),
  [Source Parser Ownership](../architecture/SOURCE_PARSER_OWNERSHIP.md),
  [Issue #348](https://github.com/YT-TechDev/frontend-analysis/issues/348)
- **Notes:** The immutable git source is the reproducible authority for
  latest-sensitive findings recorded by the #348 checkpoint. Browser or
  third-party parser behavior remains comparison/challenge evidence, not the
  semantic authority for Core source or tree-construction provenance.

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
