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
    'channel = "1.97.1"\n'
    'components = ["clippy", "rustfmt"]\n'
    'profile = "minimal"\n'
)
MEMBER = "crates/frontend-analysis-core"
PACKAGE = "frontend-analysis-core"
ROOT_KEYS = {"workspace"}
WORKSPACE_KEYS = {"lints", "members", "package", "resolver"}
WORKSPACE_PACKAGE_KEYS = {"edition"}
MEMBER_KEYS = {"lints", "package"}
MEMBER_PACKAGE_KEYS = {"edition", "name", "publish", "version"}


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
        "rust-toolchain.toml does not match the accepted Rust 1.97.1 toolchain",
    )


def validate_production_root(manifest: dict) -> dict:
    require_allowed_keys(manifest, ROOT_KEYS, "root manifest")
    workspace = manifest.get("workspace")
    fail(isinstance(workspace, dict), "root Cargo.toml must define [workspace]")
    require_allowed_keys(workspace, WORKSPACE_KEYS, "[workspace]")
    fail(workspace.get("members") == [MEMBER], f"workspace member must be exactly {MEMBER}")
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


def validate_production_manifest(root: Path, workspace: dict) -> dict:
    member_manifest = root / MEMBER / "Cargo.toml"
    fail(member_manifest.is_file(), f"required member manifest is missing: {MEMBER}/Cargo.toml")
    member = load_toml(member_manifest)
    require_allowed_keys(member, MEMBER_KEYS, "production member manifest")
    package = member.get("package")
    fail(isinstance(package, dict), "production member must define [package]")
    require_allowed_keys(package, MEMBER_PACKAGE_KEYS, "production [package]")
    fail(package.get("name") == PACKAGE, f"package name must be exactly {PACKAGE}")
    fail(
        isinstance(package.get("version"), str),
        "production package must define a version",
    )
    fail(package.get("publish") is False, "production package must set publish = false")

    edition = package.get("edition")
    inherited_edition = (
        isinstance(edition, dict)
        and edition.get("workspace") is True
        and workspace.get("package", {}).get("edition") == "2024"
    )
    fail(edition == "2024" or inherited_edition, "production package must use Edition 2024")

    lints = member.get("lints")
    fail(
        isinstance(lints, dict) and lints == {"workspace": True},
        "production package must opt into workspace lints with [lints] workspace = true",
    )
    fail(not (root / MEMBER / "build.rs").exists(), "production package must not have build.rs")
    return member


def validate_production(root: Path, metadata: dict) -> None:
    packages = metadata["packages"]
    members = metadata["workspace_members"]
    fail(len(packages) == 1, "production metadata must report exactly one package")
    fail(len(members) == 1, "production metadata must report exactly one workspace member")
    package = packages[0]
    fail(package["id"] == members[0], "the sole package must be the sole workspace member")
    fail(package["name"] == PACKAGE, f"metadata package name must be exactly {PACKAGE}")
    expected_manifest = (root / MEMBER / "Cargo.toml").resolve()
    fail(
        Path(package["manifest_path"]).resolve() == expected_manifest,
        f"metadata manifest path must be exactly {MEMBER}/Cargo.toml",
    )
    fail(not package["dependencies"], "production metadata must report zero dependencies")

    targets = package["targets"]
    fail(len(targets) == 1, "production package must expose only one library target")
    target = targets[0]
    fail(
        target["kind"] == ["lib"] and target["crate_types"] == ["lib"],
        "production package target must be a library only",
    )
    fail(target["name"] == "frontend_analysis_core", "library target name is not approved")
    expected_source = (root / MEMBER / "src" / "lib.rs").resolve()
    fail(
        Path(target["src_path"]).resolve() == expected_source,
        f"library target source must be exactly {MEMBER}/src/lib.rs",
    )

    sources = rust_sources(root)
    source_root = (root / MEMBER / "src").resolve()
    fail(sources, f"production Rust source must exist under {MEMBER}/src")
    outside = [path for path in sources if not path.resolve().is_relative_to(source_root)]
    fail(not outside, f"Rust source exists outside {MEMBER}/src: {outside}")


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
    validate_production_manifest(root, workspace)
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
