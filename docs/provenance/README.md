# Research Provenance

Classification: task and evidence record; provenance-only; non-normative.

## Purpose

This directory records where research evidence, knowledge, and decision inputs
came from. It exists so a source can be traced to the question it informed and,
when applicable, to later evidence records or architecture decisions.

Provenance does **not** establish a research conclusion, analysis result, or
architecture decision. It has no independent authority over the sources it
records or over repository contracts.

```text
provenance
    source / origin / evidence traceability

../evidence/
    experiments / falsification / findings / validation records

../architecture/ and ../decisions/
    accepted contracts and approved decisions
```

In particular:

```text
Provenance
!= evidence conclusion
!= research result
!= architecture decision
```

For example, recording that an ECMA-262 section was consulted is provenance.
A claim that a grammar boundary holds under stated conditions belongs in an
evidence record. Adopting a production ownership model belongs in the applicable
architecture contract or decision record.

## Source Classes

Use the smallest useful source class. The following vocabulary is supported:

- Normative specification
- Official documentation
- Standards / test suite
- Academic / research publication
- Engine / runtime implementation
- OSS implementation
- Issue / proposal / design discussion
- Experimental observation
- Secondary reference

These labels are descriptive, not a rigid schema. Add a more precise description
when one label would hide a material distinction.

## Evidence Role / Status

A provenance record may use lightweight role or status labels such as:

- `normative`
- `corroborating`
- `experimental`
- `historical`
- `contradictory`
- `falsified`
- `unresolved`

A role describes how the source was used; it does not promote the provenance
record itself to an authority. Preserve contradictory, falsified, and unresolved
inputs when they materially explain later research or decisions.

## Record Format

Language ledgers should use the following fields. A section-based entry is the
default because it remains readable when `Used for`, `Related research /
architecture`, or `Notes` grows beyond a short phrase.

```text
Source
Source class
Authority / version
URL or stable identifier
Accessed / reviewed date
Used for
Evidence role
Related research / architecture
Notes
```

When version, edition, commit, snapshot, browser/engine build, profile, or access
date affects interpretation, record it explicitly. Prefer immutable identifiers
where the upstream provides them. A moving URL may still be recorded, but it
must not be presented as an immutable snapshot.

## Language Ledgers

- [ECMAScript provenance](ecmascript.md)
- [HTML provenance](html.md)
- [CSS provenance](css.md)

## Maintenance Workflow

1. When research materially relies on a source, add or update its provenance
   entry as part of the same focused research work when practical.
2. Record what the source was used for; a URL without usage context is not a
   sufficient provenance record.
3. Pin edition, version, revision, commit, or other stable identity when that
   identity affects the conclusion or validation envelope.
4. Link to the relevant [evidence record](../evidence/README.md), Issue, Pull
   Request, architecture contract, or ADR instead of copying the conclusion into
   the provenance ledger.
5. If a source later contradicts earlier evidence, preserve that history and
   qualify the role/status rather than silently deleting the source.
6. Do not bulk-reconstruct historical research here. A comprehensive historical
   reference catalog is separate future work.

Primary sources are preferred. Secondary sources may be recorded when they were
actually consulted, but their existence does not substitute for available
primary authority.
