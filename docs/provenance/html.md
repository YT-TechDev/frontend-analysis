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

## TC-S10 / Title RCDATA + Named Character Reference Sources

### WHATWG HTML Standard — TC-S10 semantic snapshots

- **Source:** HTML Standard
- **Source class:** Normative specification
- **Authority / version:** Candidate-independent TC-S10 validation pinned
  `whatwg/html@9ead9de8f6751ccb98e91972e580ed6e3314c64a` with source blob
  `c090774473c6b2bc77f48e94167f43f469bba14e`; the subsequent deterministic
  Named Character Reference data gate observed
  `whatwg/html@8ad51e24e9d9e48d92317467f434f7192df9d63d`, whose parent is the
  candidate-validation pin.
- **URL or stable identifier:**
  <https://github.com/whatwg/html/commit/9ead9de8f6751ccb98e91972e580ed6e3314c64a>,
  <https://github.com/whatwg/html/commit/8ad51e24e9d9e48d92317467f434f7192df9d63d>
- **Accessed / reviewed date:** 2026-08-27 through 2026-08-30 (TC-S10
  validation, data-foundation, placement, implementation review, and provenance
  consolidation)
- **Used for:** Normative semantics for the selected InHead `title` lifecycle,
  RCDATA tokenizer behavior, Character Reference and Named Character Reference
  processing, maximum-match behavior, Ambiguous Ampersand handling, appropriate
  end-tag recognition, parse-error ordering, and EOF behavior exercised by the
  bounded TC-S10 program.
- **Evidence role:** `normative`
- **Related research / architecture:**
  [Issue #390](https://github.com/YT-TechDev/frontend-analysis/issues/390),
  [PR #391](https://github.com/YT-TechDev/frontend-analysis/pull/391),
  [Issue #392](https://github.com/YT-TechDev/frontend-analysis/issues/392),
  [Issue #394](https://github.com/YT-TechDev/frontend-analysis/issues/394),
  [HTML tree-construction architecture](../architecture/HTML_TREE_CONSTRUCTION.md)
- **Notes:** The later `8ad51e24...` snapshot was recorded by #392 as editorial
  relative to the #390/#391 pin and did not reopen the accepted candidate
  theorem. These semantic git identities are deliberately kept separate from
  the machine-readable `entities.json` byte identity below.

### WHATWG Named Character Reference dataset (`entities.json`)

- **Source:** Official WHATWG HTML Named Character Reference machine-readable
  publication
- **Source class:** Normative specification / official data publication
- **Authority / version:** Official publication retained byte-for-byte as
  `tools/html/named_character_references/inputs/entities.json`; 145,897 bytes;
  SHA-256
  `d741d877ac77c4194c4ad526b5b4a19aef8dfe411ab840a466891cdbb9f362e6`;
  repository-local blob `557170b41f47a13a46ec695561eb5fe76da73bdb`.
- **URL or stable identifier:** <https://html.spec.whatwg.org/entities.json>,
  [retained upstream manifest](../../tools/html/named_character_references/upstream-manifest.json)
- **Accessed / reviewed date:** 2026-08-27 through 2026-08-30
- **Used for:** Complete source data for the deterministic 2,231-entry generated
  Rust mapping consumed by later TC-S10 maximum matching, including derivation
  and independent verification of semicolonless-name, multi-scalar-value, and
  maximum-name-length metadata.
- **Evidence role:** `normative`
- **Related research / architecture:**
  [Issue #392](https://github.com/YT-TechDev/frontend-analysis/issues/392),
  [PR #393](https://github.com/YT-TechDev/frontend-analysis/pull/393),
  [Named Character Reference data README](../../tools/html/named_character_references/README.md),
  [PR #399](https://github.com/YT-TechDev/frontend-analysis/pull/399),
  [PR #400](https://github.com/YT-TechDev/frontend-analysis/pull/400)
- **Notes:** The project does not claim that `entities.json` is a tracked blob in
  the pinned `whatwg/html` git tree. The HTML Standard's Named Character
  Reference list remains the normative semantic authority; the official JSON is
  the retained machine-readable publication. Commit identity and dataset-byte
  identity must not be conflated. Python `html.entities` and third-party entity
  databases were explicitly rejected as semantic authority.

### WHATWG HTML license / attribution snapshot for retained Named data

- **Source:** `whatwg/html` `LICENSE`
- **Source class:** Official documentation / license notice
- **Authority / version:** `LICENSE` from
  `whatwg/html@8ad51e24e9d9e48d92317467f434f7192df9d63d`; upstream blob
  `f2dcda46deccefd245749202a88a7837e35c6daa`; retained as
  `tools/html/named_character_references/WHATWG-LICENSE.txt`, 16,315 bytes,
  SHA-256
  `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`.
- **URL or stable identifier:**
  <https://github.com/whatwg/html/blob/8ad51e24e9d9e48d92317467f434f7192df9d63d/LICENSE>,
  [retained upstream manifest](../../tools/html/named_character_references/upstream-manifest.json)
- **Accessed / reviewed date:** 2026-08-27 (Named-data provenance gate)
- **Used for:** Attribution and licensing provenance for the retained WHATWG
  Named Character Reference evidence and generated-data foundation.
- **Evidence role:** `historical`
- **Related research / architecture:**
  [Issue #392](https://github.com/YT-TechDev/frontend-analysis/issues/392),
  [Named Character Reference data README](../../tools/html/named_character_references/README.md)
- **Notes:** This records the notice actually retained by the project. It is not
  a legal classification of which license clause applies to each generated byte
  and does not replace legal review.

### Web Platform Tests (WPT) — TC-S10 challenge evidence

- **Source:** Web Platform Tests
- **Source class:** Standards / test suite
- **Authority / version:** `web-platform-tests/wpt@dd432b1d351796d3f25e1d1f243ba52da16c3a0a`
- **URL or stable identifier:**
  <https://github.com/web-platform-tests/wpt/commit/dd432b1d351796d3f25e1d1f243ba52da16c3a0a>
- **Accessed / reviewed date:** 2026-08-27 (candidate-independent TC-S10
  validation)
- **Used for:** Challenge and corroboration evidence while deriving the selected
  Title/RCDATA/Named Character Reference validation envelope independently from
  production code.
- **Evidence role:** `corroborating`
- **Related research / architecture:**
  [Issue #390](https://github.com/YT-TechDev/frontend-analysis/issues/390),
  [PR #391](https://github.com/YT-TechDev/frontend-analysis/pull/391)
- **Notes:** WPT did not define Frontend Analysis expected values and was not the
  production oracle. WPT and html5lib-family tree-construction data must not be
  counted as two independent semantic confirmations without accounting for
  shared lineage.

### html5lib-tests — TC-S10 challenge evidence

- **Source:** html5lib-tests
- **Source class:** OSS implementation / test suite
- **Authority / version:** `html5lib/html5lib-tests@224991ec10db04f056a89eed8b0bd8695fd2950e`
- **URL or stable identifier:**
  <https://github.com/html5lib/html5lib-tests/commit/224991ec10db04f056a89eed8b0bd8695fd2950e>
- **Accessed / reviewed date:** 2026-08-27 (candidate-independent TC-S10
  validation)
- **Used for:** Challenge and corroboration fixtures for selected tokenizer/tree
  behavior while independently deriving project-owned TC-S10 expectations.
- **Evidence role:** `corroborating`
- **Related research / architecture:**
  [Issue #390](https://github.com/YT-TechDev/frontend-analysis/issues/390),
  [PR #391](https://github.com/YT-TechDev/frontend-analysis/pull/391)
- **Notes:** The corpus was challenge evidence only. It did not define the
  candidate-independent oracle, and its lineage overlap with WPT is preserved
  rather than treated as an independent normative vote.

## TC-S10 Project Knowledge and Design Inputs

### HTML Tree-Construction Architecture and ADR 0010

- **Source:** Frontend Analysis HTML Tree-Construction Architecture and ADR 0010
- **Source class:** Official documentation / project architecture contract
- **Authority / version:** Accepted through Issue #349 / PR #350; recorded by
  merge commit `6411e7e550748bf28e08a042ebc4f2ba1b4c1cf5` and consulted under
  the active repository architecture during TC-S10.
- **URL or stable identifier:**
  [HTML Tree-Construction Architecture](../architecture/HTML_TREE_CONSTRUCTION.md),
  [ADR 0010](../decisions/0010-html-tree-construction-architecture.md),
  [Issue #117](https://github.com/YT-TechDev/frontend-analysis/issues/117)
- **Accessed / reviewed date:** 2026-08-28 through 2026-08-30
- **Used for:** Candidate C ownership boundaries: SourceText provenance,
  tokenizer lexical ownership, tree-construction semantic ownership, Core parse
  coordinator sequencing, private mutable construction sessions, validated
  freeze, immutable analysis results, and separation from runtime browser DOM
  authority.
- **Evidence role:** `design input`
- **Related research / architecture:**
  [HTML evidence](../evidence/html/README.md),
  [Issue #394](https://github.com/YT-TechDev/frontend-analysis/issues/394),
  [PR #400](https://github.com/YT-TechDev/frontend-analysis/pull/400)
- **Notes:** These documents are normative project architecture in their own
  scope. Their presence in this provenance ledger records that TC-S10 research
  and placement consumed them; the ledger does not duplicate or extend their
  authority.

### Issue #117 focused TC-S10 placement checkpoints

- **Source:** Frontend Analysis Issue #117 focused design / placement records
- **Source class:** Issue / proposal / design discussion
- **Authority / version:** Stable issue comments used by the TC-S10 research
  chain:
  `5447018049` (Cursor / non-committing lookahead),
  `5447119161` (RCDATA + Character Reference production domain),
  `5447154856` (Title tree/tokenizer coordination),
  `5447192363` (result / diagnostic / resource placement),
  `5447810492` (exhaustive placement / changed-file checkpoint),
  `5461014840` (compiler-sealed governance replacement), and
  `5461039548` (compiler-sealed owner prerequisite placement).
- **URL or stable identifier:**
  <https://github.com/YT-TechDev/frontend-analysis/issues/117#issuecomment-5447018049>,
  <https://github.com/YT-TechDev/frontend-analysis/issues/117#issuecomment-5447119161>,
  <https://github.com/YT-TechDev/frontend-analysis/issues/117#issuecomment-5447154856>,
  <https://github.com/YT-TechDev/frontend-analysis/issues/117#issuecomment-5447192363>,
  <https://github.com/YT-TechDev/frontend-analysis/issues/117#issuecomment-5447810492>,
  <https://github.com/YT-TechDev/frontend-analysis/issues/117#issuecomment-5461014840>,
  <https://github.com/YT-TechDev/frontend-analysis/issues/117#issuecomment-5461039548>
- **Accessed / reviewed date:** 2026-08-28 through 2026-08-30
- **Used for:** Progressive ownership and placement falsification before
  production: non-committing lookahead, state-domain ownership, semantic
  feedback, diagnostic/provenance/resource boundaries, changed-file derivation,
  and replacement of a handwritten repository Rust scanner with compiler-sealed
  ownership.
- **Evidence role:** `design input`
- **Related research / architecture:**
  [Issue #396](https://github.com/YT-TechDev/frontend-analysis/issues/396),
  [PR #397](https://github.com/YT-TechDev/frontend-analysis/pull/397),
  [Issue #398](https://github.com/YT-TechDev/frontend-analysis/issues/398),
  [PR #399](https://github.com/YT-TechDev/frontend-analysis/pull/399),
  [Issue #394](https://github.com/YT-TechDev/frontend-analysis/issues/394)
- **Notes:** These comments record project-specific placement knowledge, not
  HTML Standard semantics. Historical placement theorems that were later
  superseded remain part of the research history rather than being silently
  deleted.

### Candidate-independent validation, ownership validation, and final TC-S10 review records

- **Source:** Frontend Analysis validation / review artifacts for the selected
  TC-S10 lifecycle
- **Source class:** Experimental observation / project validation record
- **Authority / version:** #390 / merged PR #391 candidate-independent semantic
  validation; #392 / PR #393 deterministic Named-data foundation; #396 / PR
  #397 compiler-sealed ownership validation; #398 / PR #399 production canonical
  owner; #394 post-#399 placement amendment `issuecomment-5461790243`; PR #400
  exact-head review and evidence completion recorded at #394
  `issuecomment-5465273373`. PR #400 merged as
  `fd8789dd853069b4126e429763c2666bd70eff6c`, tree
  `07cdc10a993aec34f4bee07e9430b4193b9e7ea7`.
- **URL or stable identifier:**
  [PR #391](https://github.com/YT-TechDev/frontend-analysis/pull/391),
  [Issue #392](https://github.com/YT-TechDev/frontend-analysis/issues/392),
  [PR #393](https://github.com/YT-TechDev/frontend-analysis/pull/393),
  [PR #397](https://github.com/YT-TechDev/frontend-analysis/pull/397),
  [PR #399](https://github.com/YT-TechDev/frontend-analysis/pull/399),
  <https://github.com/YT-TechDev/frontend-analysis/issues/394#issuecomment-5461790243>,
  <https://github.com/YT-TechDev/frontend-analysis/issues/394#issuecomment-5465273373>,
  [PR #400](https://github.com/YT-TechDev/frontend-analysis/pull/400)
- **Accessed / reviewed date:** 2026-08-27 through 2026-08-30
- **Used for:** Candidate-independent challenge expectations, deterministic data
  provenance, compiler-owned canonical-data wiring, production placement,
  focused remediation guidance, exact-head independent source review, and
  final session-local mechanical evidence replay.
- **Evidence role:** `experimental`
- **Related research / architecture:**
  [HTML evidence](../evidence/html/README.md),
  [HTML tree-construction architecture](../architecture/HTML_TREE_CONSTRUCTION.md),
  [Named Character Reference data README](../../tools/html/named_character_references/README.md)
- **Notes:** These records establish project validation and review provenance;
  they do not replace the WHATWG HTML Standard as semantic authority. The
  historical closed PR #395 was used only as regression/challenge guidance and
  is not recorded here as an accepted production source.

## Adding Entries

Use the field set defined in [Research Provenance](README.md). Add Web Platform
Tests, browser-engine implementations, Issues, experimental observations, or
secondary references only when their identity and actual research use are
verified.
