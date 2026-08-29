# WHATWG Named Character Reference data

This directory owns the retained provenance and deterministic, offline data
generation for the complete WHATWG HTML Named Character Reference table. It is
an HTML data boundary; it is not part of `tools/unicode` and does not use
Python's `html.entities` or another entity database.

The retained evidence is:

- `inputs/entities.json`: exact bytes from the official WHATWG publication at
  <https://html.spec.whatwg.org/entities.json>;
- `WHATWG-LICENSE.txt`: exact `LICENSE` bytes from the pinned `whatwg/html`
  commit; and
- `upstream-manifest.json`: metadata recording the distinct normative,
  dataset, retained-license, and derived-data identities.

The manifest is not self-authenticating authority. The generator and verifier
each freeze the expected dataset bytes, WHATWG snapshot, and retained-license
identity independently. A coordinated edit to the retained dataset and
manifest therefore cannot redefine the accepted upstream envelope.

## Offline generation and verification

The one-time upstream acquisition step is deliberately outside the scripts.
Neither script performs network I/O, invokes an external parser, or imports
the other script.

Generate the checked-in canonical Rust table:

```bash
python3 tools/html/named_character_references/generate_named_character_references.py
```

Require exact reproducibility without modifying the repository:

```bash
python3 tools/html/named_character_references/generate_named_character_references.py --check
```

Independently reparse the retained authority and prove complete equality with
the generated Rust representation:

```bash
python3 tools/html/named_character_references/verify_generated_named_character_references.py
```

Run the fail-closed mutation suite:

```bash
python3 -m unittest discover \
  -s tools/html/named_character_references/tests \
  -p 'test_*.py'
```

## Scope boundary

The generated mapping is deliberately only:

```text
exact name without the one leading `&` -> decoded Unicode string
```

It selects no trie, perfect hash, prefix search, cursor/lookahead API, tokenizer
state, diagnostic behavior, resource model, or tree/tokenizer coordination.

## Lifecycle

```text
#392 / #393
-> semantic and provenance data foundation

#398
-> production compiler-sealed ownership and wiring

future TC-S10
-> runtime matcher and Title/RCDATA causal lifecycle
```

The table is now production-owned. The generated source is a lexical artifact
included inside one hand-written private owner module under `html::tokenizer`,
and production code reaches the rows only through that owner's narrow
tokenizer-private wrapper. Its raw declarations are private to the owner, so
they are not an alternate production path to the data.

The generated Rust data tests remain test-only. Accidental production exposure
is a compiler error rather than something this tooling detects, because
ownership enforcement here is compiler-owned:

- `rustc` decides syntax, `cfg` and module selection, privacy, and coherence;
- the owner's private registration trait and token cannot be named from
  outside the owner;
- a second active registration of that trait for that token is rejected by
  trait coherence.

Neither script is a repository Rust scanner. The verifier parses the canonical
generated representation because that representation is its own semantic and
provenance subject; it resolves no module wiring and evaluates no `cfg`,
`#[path]`, or `include!` semantics.

The retained upstream evidence, the deterministic offline generator, the
independent verifier, and the complete 2231-entry semantic mapping are
unchanged by this ownership work. Runtime Named Character Reference matching
and tokenizer behavior remain out of scope.
