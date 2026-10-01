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

The `frontend-analysis-cli` Product package provides one binary, `fa`, with one
command that reads exactly one authored UTF-8 stylesheet from stdin and reports
how the bounded CSS `CoreV1` selector profile classifies its retained
selector-list contexts:

```bash
cargo run -q -p frontend-analysis-cli -- css-selectors < style.css
```

It consumes the narrow public `frontend_analysis_core::css::selectors` facade
under a fixed Phase 1 execution envelope and prints deterministic
human-readable text. Exit status `0` means a report was produced (including
invalid, unsupported, indeterminate, incomplete, or resource-limited results),
`1` means a command or stdin acquisition failure, and `2` means a returned Core
failure. It is not a complete CSS parser, validator, or browser replacement.
[ADR 0011](docs/decisions/0011-establish-cli-product-and-css-core-v1-consumer-boundary.md)
owns this Product/Core boundary.

This state does not imply completion or approval of parsers, Browser Adapters,
browser protocols, analysis-result models, diagnostics or evidence graphs,
desktop, VS Code, or web products, CLI capabilities beyond `fa css-selectors`,
serialization, crates.io publication, or release automation.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for
the project workflow and contribution requirements.

Participation in this project is governed by the
[Code of Conduct](.github/CODE_OF_CONDUCT.md).
