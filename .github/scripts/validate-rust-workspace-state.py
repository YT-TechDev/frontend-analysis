#!/usr/bin/env python3
"""Validate the approved production Rust workspace policy."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys
import tomllib


TOOLCHAIN = (
    "[toolchain]\n"
    'channel = "1.98.1"\n'
    'components = ["clippy", "rustfmt"]\n'
    'profile = "minimal"\n'
)
CORE_MEMBER = "crates/frontend-analysis-core"
CORE_PACKAGE = "frontend-analysis-core"
CLI_MEMBER = "crates/frontend-analysis-cli"
CLI_PACKAGE = "frontend-analysis-cli"
MEMBERS = [CORE_MEMBER, CLI_MEMBER]
ROOT_KEYS = {"workspace"}
WORKSPACE_KEYS = {"lints", "members", "package", "resolver"}
WORKSPACE_PACKAGE_KEYS = {"edition"}
CORE_MEMBER_KEYS = {"lints", "package"}
CLI_MEMBER_KEYS = {"bin", "dependencies", "lints", "package"}
MEMBER_PACKAGE_KEYS = {"edition", "name", "publish", "version"}
CLI_BIN = [{"name": "fa", "path": "src/main.rs"}]
CLI_DEPENDENCIES = {CORE_PACKAGE: {"path": "../frontend-analysis-core"}}
# The single approved CLI process-boundary integration test (ADR 0011).
CLI_INTEGRATION_TEST = "tests/css_selectors.rs"


class PolicyError(Exception):
    """An actionable workspace-policy violation."""


def fail(condition: bool, message: str) -> None:
    if not condition:
        raise PolicyError(message)


def require_allowed_keys(table: dict, allowed: set[str], description: str) -> None:
    unexpected = sorted(set(table) - allowed)
    fail(
        not unexpected,
        f"{description} contains unapproved keys: {unexpected}; "
        f"allowed keys are {sorted(allowed)}",
    )


def load_toml(path: Path) -> dict:
    try:
        with path.open("rb") as manifest:
            return tomllib.load(manifest)
    except FileNotFoundError as error:
        raise PolicyError(f"required file is missing: {path}") from error
    except tomllib.TOMLDecodeError as error:
        raise PolicyError(f"invalid TOML in {path}: {error}") from error


def cargo_metadata(root: Path) -> dict:
    command = [
        "cargo",
        "metadata",
        "--offline",
        "--format-version",
        "1",
        "--locked",
    ]
    result = subprocess.run(command, cwd=root, capture_output=True, text=True)
    if result.returncode:
        detail = result.stderr.strip() or result.stdout.strip()
        raise PolicyError(f"Cargo metadata validation failed: {detail}")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise PolicyError("Cargo metadata did not return valid JSON") from error


def rust_sources(root: Path) -> list[Path]:
    return sorted(path for path in root.rglob("*.rs") if ".git" not in path.parts)


def validate_toolchain(root: Path) -> None:
    path = root / "rust-toolchain.toml"
    fail(path.is_file(), "rust-toolchain.toml is missing")
    fail(
        path.read_text(encoding="utf-8") == TOOLCHAIN,
        "rust-toolchain.toml does not match the accepted Rust 1.98.1 toolchain",
    )


def validate_production_root(manifest: dict) -> dict:
    require_allowed_keys(manifest, ROOT_KEYS, "root manifest")
    workspace = manifest.get("workspace")
    fail(isinstance(workspace, dict), "root Cargo.toml must define [workspace]")
    require_allowed_keys(workspace, WORKSPACE_KEYS, "[workspace]")
    fail(
        workspace.get("members") == MEMBERS,
        f"workspace members must be exactly {MEMBERS}",
    )
    fail(workspace.get("resolver") == "3", 'workspace resolver must be exactly "3"')

    workspace_package = workspace.get("package")
    fail(isinstance(workspace_package, dict), "[workspace.package] must be a table")
    require_allowed_keys(workspace_package, WORKSPACE_PACKAGE_KEYS, "[workspace.package]")
    fail(
        workspace_package.get("edition") == "2024",
        '[workspace.package] must contain only edition = "2024"',
    )

    fail(
        workspace.get("lints") == {"rust": {"unsafe_code": "deny"}},
        'workspace lint policy must contain only rust.unsafe_code = "deny"',
    )
    return workspace


def validate_member_manifest(
    root: Path, workspace: dict, member_path: str, package_name: str, allowed_keys: set[str]
) -> dict:
    member_manifest = root / member_path / "Cargo.toml"
    fail(
        member_manifest.is_file(),
        f"required member manifest is missing: {member_path}/Cargo.toml",
    )
    member = load_toml(member_manifest)
    require_allowed_keys(member, allowed_keys, f"{package_name} manifest")
    package = member.get("package")
    fail(isinstance(package, dict), f"{package_name} must define [package]")
    require_allowed_keys(package, MEMBER_PACKAGE_KEYS, f"{package_name} [package]")
    fail(package.get("name") == package_name, f"package name must be exactly {package_name}")
    fail(
        isinstance(package.get("version"), str),
        f"{package_name} must define a version",
    )
    fail(package.get("publish") is False, f"{package_name} must set publish = false")

    edition = package.get("edition")
    inherited_edition = (
        isinstance(edition, dict)
        and edition.get("workspace") is True
        and workspace.get("package", {}).get("edition") == "2024"
    )
    fail(edition == "2024" or inherited_edition, f"{package_name} must use Edition 2024")

    lints = member.get("lints")
    fail(
        isinstance(lints, dict) and lints == {"workspace": True},
        f"{package_name} must opt into workspace lints with [lints] workspace = true",
    )
    fail(
        not (root / member_path / "build.rs").exists(),
        f"{package_name} must not have build.rs",
    )
    return member


def validate_production_manifests(root: Path, workspace: dict) -> None:
    validate_member_manifest(root, workspace, CORE_MEMBER, CORE_PACKAGE, CORE_MEMBER_KEYS)
    cli = validate_member_manifest(root, workspace, CLI_MEMBER, CLI_PACKAGE, CLI_MEMBER_KEYS)
    fail(
        cli.get("bin") == CLI_BIN,
        f"{CLI_PACKAGE} must declare exactly one [[bin]]: {CLI_BIN}",
    )
    fail(
        cli.get("dependencies") == CLI_DEPENDENCIES,
        f"{CLI_PACKAGE} [dependencies] must be exactly {CLI_DEPENDENCIES}",
    )


def metadata_package(metadata: dict, name: str) -> dict:
    matches = [package for package in metadata["packages"] if package["name"] == name]
    fail(len(matches) == 1, f"metadata must report exactly one {name} package")
    return matches[0]


def validate_target(
    target: dict, name: str, kind: list[str], crate_types: list[str], source: Path
) -> None:
    fail(
        target["name"] == name
        and target["kind"] == kind
        and target["crate_types"] == crate_types,
        f"target {target['name']} {target['kind']} is not the approved {name} {kind}",
    )
    fail(
        Path(target["src_path"]).resolve() == source.resolve(),
        f"target {name} source must be exactly {source}",
    )


def validate_production(root: Path, metadata: dict) -> None:
    packages = metadata["packages"]
    members = metadata["workspace_members"]
    fail(len(packages) == 2, "production metadata must report exactly two packages")
    fail(len(members) == 2, "production metadata must report exactly two workspace members")
    fail(
        sorted(package["id"] for package in packages) == sorted(members),
        "every package must be a workspace member",
    )

    core = metadata_package(metadata, CORE_PACKAGE)
    cli = metadata_package(metadata, CLI_PACKAGE)
    for package, member in ((core, CORE_MEMBER), (cli, CLI_MEMBER)):
        fail(
            Path(package["manifest_path"]).resolve()
            == (root / member / "Cargo.toml").resolve(),
            f"metadata manifest path must be exactly {member}/Cargo.toml",
        )
        fail(not package["features"], f"{package['name']} must declare no features")

    fail(not core["dependencies"], f"{CORE_PACKAGE} must report zero dependencies")
    targets = core["targets"]
    fail(len(targets) == 1, f"{CORE_PACKAGE} must expose only one library target")
    validate_target(
        targets[0],
        "frontend_analysis_core",
        ["lib"],
        ["lib"],
        root / CORE_MEMBER / "src" / "lib.rs",
    )

    dependencies = cli["dependencies"]
    fail(
        len(dependencies) == 1,
        f"{CLI_PACKAGE} must report exactly one dependency: {CORE_PACKAGE}",
    )
    dependency = dependencies[0]
    fail(
        dependency["name"] == CORE_PACKAGE
        and dependency.get("source") is None
        and dependency.get("kind") is None
        and not dependency.get("optional")
        and Path(dependency.get("path") or "").resolve() == (root / CORE_MEMBER).resolve(),
        f"{CLI_PACKAGE} dependency must be the local {CORE_PACKAGE} path package",
    )
    targets = cli["targets"]
    fail(
        len(targets) == 2,
        f"{CLI_PACKAGE} must expose exactly the fa binary and its approved integration test",
    )
    by_name = {target["name"]: target for target in targets}
    fail(
        sorted(by_name) == ["css_selectors", "fa"],
        f"{CLI_PACKAGE} targets must be exactly fa and css_selectors",
    )
    validate_target(by_name["fa"], "fa", ["bin"], ["bin"], root / CLI_MEMBER / "src" / "main.rs")
    validate_target(
        by_name["css_selectors"],
        "css_selectors",
        ["test"],
        ["bin"],
        root / CLI_MEMBER / CLI_INTEGRATION_TEST,
    )

    sources = rust_sources(root)
    fail(sources, f"production Rust source must exist under {CORE_MEMBER}/src")
    core_root = (root / CORE_MEMBER / "src").resolve()
    cli_root = (root / CLI_MEMBER / "src").resolve()
    cli_test = (root / CLI_MEMBER / CLI_INTEGRATION_TEST).resolve()
    outside = [
        path
        for path in sources
        if not path.resolve().is_relative_to(core_root)
        and not path.resolve().is_relative_to(cli_root)
        and path.resolve() != cli_test
    ]
    fail(not outside, f"Rust source exists outside the approved source roots: {outside}")


def validate_workspace(root: Path) -> None:
    fail(root.is_dir(), f"repository root is not a directory: {root}")
    validate_toolchain(root)
    manifest_path = root / "Cargo.toml"
    manifest = load_toml(manifest_path)
    fail("package" not in manifest, "root Cargo.toml must remain a virtual workspace")
    workspace = manifest.get("workspace")
    fail(isinstance(workspace, dict), "root Cargo.toml must define [workspace]")
    fail(workspace.get("resolver") == "3", 'workspace resolver must be exactly "3"')
    workspace = validate_production_root(manifest)
    fail(
        (root / "Cargo.lock").is_file(),
        "production workspace requires a committed Cargo.lock",
    )
    validate_production_manifests(root, workspace)
    metadata = cargo_metadata(root)
    validate_production(root, metadata)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("repository_root", nargs="?", default=".", type=Path)
    args = parser.parse_args()
    try:
        validate_workspace(args.repository_root.resolve())
    except (OSError, KeyError, TypeError, PolicyError) as error:
        print(f"Rust workspace state rejected: {error}", file=sys.stderr)
        return 1
    print("production")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
