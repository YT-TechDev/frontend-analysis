# frontend-analysis
A browser-independent platform for analyzing, visualizing, and diagnosing modern web applications.

## Documentation

See the [documentation index](docs/README.md) for the repository knowledge map,
authoritative contracts, and source-of-truth rules.

## Rust Core Status

`YT-TechDev/frontend-analysis` is the initial Core-focused Rust workspace owner;
this role does not establish a permanent monorepo. The root remains a virtual
Cargo workspace and currently contains exactly two production members:
`crates/frontend-analysis-core` and `crates/frontend-analysis-cli`. Both
packages set `publish = false`, have zero third-party Rust dependencies, and are
validated with the committed root `Cargo.lock`. The Core package's current
production responsibility includes Validated Source Anchors and Raw Source Line
Coordinates. Raw
coordinates preserve authoritative UTF-8 byte offsets; they do not imply
parser, browser-protocol, Unicode-display, or presentation position
compatibility.

Rust `1.98.1` is pinned for reproducible development and CI, but the pin is not
an MSRV promise. See the [documentation index](docs/README.md) for detailed
current-state and validation guidance and the
[Validated Source Anchors Guide](docs/architecture/VALIDATED_SOURCE_ANCHORS.md)
and [Raw Source Coordinates Guide](docs/architecture/RAW_SOURCE_COORDINATES.md)
for contributor guidance. Accepted
[ADR 0001](docs/decisions/0001-repository-topology-and-workspace-ownership.md),
[ADR 0002](docs/decisions/0002-rust-bootstrap-toolchain-and-validation-policy.md),
[ADR 0003](docs/decisions/0003-validated-source-anchors-first-rust-core-domain.md),
[ADR 0004](docs/decisions/0004-validated-source-anchor-semantics.md), and
[ADR 0005](docs/decisions/0005-raw-source-coordinate-semantics.md) own the
applicable topology, toolchain, crate-boundary, source-anchor, and raw
source-coordinate decisions.

The `frontend-analysis-cli` Product package provides one browser-independent
binary, `fa`, with three capability-oriented commands. Each reads authored
UTF-8 source from stdin and prints deterministic human-readable text:

```bash
fa css-selectors < style.css
fa es-binding-refs < source.js
fa html-tree < source.html
```

These are narrow Product capabilities, not complete language implementations:

- **CSS:** classifies retained selector-list contexts under the bounded CSS
  `CoreV1` selected profile.
- **ECMAScript:** reports selected same-source lexical binding-reference
  relationships for the accepted flat top-level lexical-binding initializer
  profile.
- **HTML:** reports the constructed tree for the accepted bounded
  document-construction profile, including retained authored/synthesis evidence
  and completion boundaries.

The shared Product shell owns command routing, bounded stdin acquisition,
strict UTF-8 decoding, invocation-local `SourceId(0)`, text rendering, and
process exit status. Language semantics remain owned by their narrow Core
consumer capabilities. Results may be incomplete, unsupported, diagnostic, or
resource-limited. Exit status `0` means a report was produced, not that the
source is globally valid; `1` means Product command, input, acquisition, or
output failure; `2` means a returned Core boundary or internal failure. These
commands do not provide complete CSS parsing or browser CSS behavior, general
JavaScript resolution or runtime semantics, or general HTML parsing, DOM
equivalence, or browser runtime behavior. [ADR 0011](docs/decisions/0011-establish-cli-product-and-css-core-v1-consumer-boundary.md)
owns the Product/Core boundary.

Both Rust packages remain `publish = false`. This state does not imply complete
language implementations, Browser Adapters, browser protocols, analysis-result
models, diagnostics or evidence graphs, desktop, VS Code, or web products,
serialization, crates.io publication, or release automation. The Rust toolchain
pin is not an MSRV guarantee. No v0.1.0 release or tag is implied.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for
the project workflow and contribution requirements.

Participation in this project is governed by the
[Code of Conduct](.github/CODE_OF_CONDUCT.md).
