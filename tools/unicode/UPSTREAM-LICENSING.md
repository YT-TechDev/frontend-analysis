# Upstream licensing and notice provenance

Issue #198 retains standards material only with a separately auditable notice
boundary. This record documents provenance; it is not a substitute for legal
advice.

## Unicode

`UNICODE-LICENSE.txt` is retained byte-for-byte from the pinned Unicode source.
Its Git blob identity is:

```text
d7e7973c2fd6f2586a8999a69dc21e39af26be0f
```

The retained Unicode 17.0.0 UCD inputs are kept alongside that notice.

## ECMA-262 source repository routing notice

`ECMA262-LICENSE.md` is retained byte-for-byte from `tc39/ecma262` snapshot
`d89c03f2db8a597bc915b363a6518d0cc8acdbc0`. Its Git blob identity is:

```text
6e6b6a4742069c493c4b66e06dad9e5bd9f05f9a
```

That file routes natural-language specification text to Ecma International's
Alternative Copyright Notice; it is preserved as source-repository provenance
rather than treated as the full redistribution notice.

## ECMA redistribution notice

The pinned ECMA-262 `package-lock.json` resolves `ecmarkup 24.0.0`. The exact
`tc39/ecmarkup` v24.0.0 boilerplate used for the Alternative Copyright Notice is
`boilerplate/alternative-copyright.html`, Git blob:

```text
4296cb3c430a099cdebfccdc36db3b987db3fa08
```

`ECMA262-COPYRIGHT.html` is the rendered notice retained for redistribution with
the three ECMA-262 table portions. It is deterministically derived from that
pinned template by substituting:

```text
!YEAR!     -> 2026
!DOCUMENT! -> ECMAScript® 2026 Language&nbsp;Specification https://tc39.es/ecma262/
```

The rendered result matches the corresponding year/document values in the
official ECMA-262 2026 annual output. Its identities are:

```text
Git blob SHA-1:
f665822a5195ac456838c6a44e0341bbb3fc21ed

SHA-256:
64e2027b6e75ad4e8a5be0047abcd43d89b5417e5ed460918590276eae5e9853
```

The template itself is not a runtime or semantic input and is not vendored. The
exact template blob identity and substitution rule are recorded so the notice
provenance remains independently reproducible without adding a package or build
dependency.

## Boundary

License and notice files are not semantic authority for generated Unicode data.
Likewise, successful source-byte verification does not by itself prove semantic
equality; generator and independent semantic verification remain separate gates.
