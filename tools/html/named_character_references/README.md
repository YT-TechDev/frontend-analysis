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

Generate the checked-in test-only Rust table:

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
Both the generated table and its Rust data tests are wired only under
`cfg(test)`. Production tokenizer behavior remains unchanged by this data gate.
