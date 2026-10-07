# CSS Research Provenance

Classification: task and evidence record; provenance-only; non-normative.

This ledger records sources actually used by the CSS research program. It does
not restate or decide CSS findings. See the
[CSS evidence record](../evidence/css/README.md) for conclusions, frozen
capability boundaries, and validation status.

## Initial Ledger Status

The current high-level CSS evidence record establishes that applicable CSSWG
specifications are normative and that each capability must record the exact
specification snapshot/profile it uses. That record does not identify one exact
CSSWG snapshot that is safe to initialize here as a repository-wide CSS
provenance fact.

Accordingly, this foundation does not guess a CSS specification edition or
snapshot. The dated initial ledger did not yet have a capability-specific
entry. The verified CSS Mixins 1 source below was later recorded by #418;
it is **not** a single frozen CSSWG revision for every CSS capability.
Additional CSSWG sources may be recorded only from verified, actually used
focused records, without reconstructing their research history in bulk.

## Verified Focused Source

### CSS Custom Functions and Mixins Level 1 — selected dashed-function qualification

- **Source:** CSS Custom Functions and Mixins Module Level 1,
  `css-mixins-1/Overview.bs`
- **Source class:** CSSWG Editor's Draft (ED; upstream work status:
  Exploring), used as selected draft/profile semantic authority
- **Authority / version:** `w3c/csswg-drafts@b1ebca428ca1ab224f5fc1d2da5df1d493c9d282`;
  this is a capability-specific immutable source identity, not a
  repository-wide CSSWG snapshot.
- **URL or stable identifier:**
  <https://github.com/w3c/csswg-drafts/blob/b1ebca428ca1ab224f5fc1d2da5df1d493c9d282/css-mixins-1/Overview.bs>
- **Accessed / reviewed date:** Recorded in the original #418 research hub,
  opened 2026-09-01; this ledger refresh does not claim a fresh upstream
  source audit.
- **Used for:** Distinguishing `<dashed-function>` names beginning `--`
  and the deferred parse-time versus post-substitution grammar-validity
  boundary in selected authored declaration-value qualification.
- **Evidence role:** `normative` for that focused draft/profile question;
  not evidence of general CSS support.
- **Related research / architecture:**
  [#418 initial retained CSS knowledge](https://github.com/YT-TechDev/frontend-analysis/issues/418),
  [PR #417](https://github.com/YT-TechDev/frontend-analysis/pull/417),
  [CSS authored-value checkpoint](../evidence/css/2026-09-authored-value-qualification-and-transform-coverage-checkpoint.md).
- **Notes:** The exact pin and semantic use are owned by #418. Other CSS
  Values, selector, transform, and property sources are governed by their
  focused research/validation records; this entry does not manufacture
  missing revisions or assert that every CSSWG source was reviewed on
  2026-09-01.

## Entry Template

### Source name

- **Source:**
- **Source class:**
- **Authority / version:**
- **URL or stable identifier:**
- **Accessed / reviewed date:**
- **Used for:**
- **Evidence role:**
- **Related research / architecture:**
- **Notes:**

Use the field set and maintenance rules defined in
[Research Provenance](README.md). Preserve contradictory, falsified, historical,
or unresolved source roles when they materially explain later CSS findings.
