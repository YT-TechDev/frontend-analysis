# frontend-analysis
A browser-independent platform for analyzing, visualizing, and diagnosing modern web applications.

## Documentation

See the [documentation index](docs/README.md) for the repository knowledge map,
authoritative contracts, and source-of-truth rules.

## Rust Core Status

`YT-TechDev/frontend-analysis` is the initial Core-focused Rust workspace owner;
this role does not establish a permanent monorepo. The root remains a virtual
Cargo workspace and currently contains exactly one production member:
`crates/frontend-analysis-core`. The private `frontend-analysis-core` package
sets `publish = false`, has zero third-party Rust dependencies, and is validated
with the committed root `Cargo.lock`. Its current production responsibility
includes Validated Source Anchors and Raw Source Line Coordinates. Raw
coordinates preserve authoritative UTF-8 byte offsets; they do not imply
parser, browser-protocol, Unicode-display, or presentation position
compatibility.

Rust `1.97.1` is pinned for reproducible development and CI, but the pin is not
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

This state does not imply completion or approval of parsers, Browser Adapters,
browser protocols, analysis-result models, diagnostics or evidence graphs,
desktop, CLI, VS Code, or web products, serialization, crates.io publication,
or release automation.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for
the project workflow and contribution requirements.

Participation in this project is governed by the
[Code of Conduct](.github/CODE_OF_CONDUCT.md).
