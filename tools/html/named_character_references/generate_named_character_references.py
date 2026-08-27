#!/usr/bin/env python3
"""Generate the frozen WHATWG Named Character Reference semantic table."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence

WHATWG_HTML_SNAPSHOT = "8ad51e24e9d9e48d92317467f434f7192df9d63d"
WHATWG_HTML_PARENT = "9ead9de8f6751ccb98e91972e580ed6e3314c64a"
DATASET_URL = "https://html.spec.whatwg.org/entities.json"
DATASET_PATH = Path("tools/html/named_character_references/inputs/entities.json")
DATASET_BYTE_SIZE = 145_897
DATASET_SHA256 = "d741d877ac77c4194c4ad526b5b4a19aef8dfe411ab840a466891cdbb9f362e6"
DATASET_GIT_BLOB_SHA1 = "557170b41f47a13a46ec695561eb5fe76da73bdb"
LICENSE_PATH = Path("tools/html/named_character_references/WHATWG-LICENSE.txt")
LICENSE_BYTE_SIZE = 16_315
LICENSE_SHA256 = "85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864"
LICENSE_GIT_BLOB_SHA1 = "f2dcda46deccefd245749202a88a7837e35c6daa"
MANIFEST_PATH = Path("tools/html/named_character_references/upstream-manifest.json")
OUTPUT_PATH = Path(
    "crates/frontend-analysis-core/src/html/tokenizer/"
    "named_character_references_generated.rs"
)

EXPECTED_ENTRY_COUNT = 2_231
EXPECTED_SEMICOLONLESS_COUNT = 106
EXPECTED_TWO_SCALAR_COUNT = 93
EXPECTED_MAXIMUM_KEY_BYTE_LENGTH = 32
EXPECTED_MAXIMUM_KEYS = ("CounterClockwiseContourIntegral;",)

_ENTITY_KEY_RE = re.compile(r"&[A-Za-z0-9]+;?", flags=re.ASCII)
_CHALLENGE_CELLS = {
    "&amp;": (0x26,),
    "&amp": (0x26,),
    "&lt;": (0x3C,),
    "&AMP;": (0x26,),
    "&not": (0xAC,),
    "&not;": (0xAC,),
    "&notin;": (0x2209,),
    "&acE;": (0x223E, 0x0333),
    "&CounterClockwiseContourIntegral;": (0x2233,),
    "&Afr;": (0x1D504,),
}


class GenerationError(RuntimeError):
    """Raised when retained evidence or generated semantics fail closed."""


@dataclass(frozen=True)
class Entity:
    source_name: str
    generated_name: str
    codepoints: tuple[int, ...]
    characters: str


@dataclass(frozen=True)
class Dataset:
    entries: tuple[Entity, ...]
    semicolonless_count: int
    two_scalar_count: int
    maximum_key_byte_length: int
    maximum_keys: tuple[str, ...]


def git_blob_sha1(data: bytes) -> str:
    header = f"blob {len(data)}\0".encode("ascii")
    return hashlib.sha1(header + data, usedforsecurity=False).hexdigest()


def _strict_json(data: bytes, label: str) -> object:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise GenerationError(f"{label}: input is not UTF-8") from error

    def reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for name, value in pairs:
            if name in result:
                raise GenerationError(f"{label}: duplicate JSON member {name!r}")
            result[name] = value
        return result

    def reject_non_finite(token: str) -> object:
        raise GenerationError(f"{label}: non-finite JSON number {token}")

    try:
        return json.loads(
            text,
            object_pairs_hook=reject_duplicates,
            parse_constant=reject_non_finite,
        )
    except GenerationError:
        raise
    except json.JSONDecodeError as error:
        raise GenerationError(f"{label}: malformed JSON: {error.msg}") from error


def _require_object(value: object, label: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise GenerationError(f"{label}: expected JSON object")
    return value


def _require_exact_keys(value: dict[str, object], expected: set[str], label: str) -> None:
    actual = set(value)
    if actual != expected:
        missing = sorted(expected - actual)
        extra = sorted(actual - expected)
        raise GenerationError(f"{label}: object fields mismatch missing={missing} extra={extra}")


def _require_integer(value: object, label: str) -> int:
    if type(value) is not int:
        raise GenerationError(f"{label}: expected genuine JSON integer")
    return value


def _read_file(path: Path, label: str) -> bytes:
    try:
        return path.read_bytes()
    except OSError as error:
        raise GenerationError(f"{label}: retained file unavailable") from error


def validate_frozen_bytes(
    data: bytes,
    *,
    label: str,
    byte_size: int,
    sha256: str,
    git_blob: str,
) -> None:
    if len(data) != byte_size:
        raise GenerationError(f"{label}: frozen byte size mismatch")
    if hashlib.sha256(data).hexdigest() != sha256:
        raise GenerationError(f"{label}: frozen SHA-256 mismatch")
    if git_blob_sha1(data) != git_blob:
        raise GenerationError(f"{label}: frozen Git blob mismatch")


def load_frozen_evidence(repo_root: Path) -> tuple[bytes, bytes]:
    dataset = _read_file(repo_root / DATASET_PATH, "entities.json")
    license_bytes = _read_file(repo_root / LICENSE_PATH, "WHATWG LICENSE")
    validate_frozen_bytes(
        dataset,
        label="entities.json",
        byte_size=DATASET_BYTE_SIZE,
        sha256=DATASET_SHA256,
        git_blob=DATASET_GIT_BLOB_SHA1,
    )
    validate_frozen_bytes(
        license_bytes,
        label="WHATWG LICENSE",
        byte_size=LICENSE_BYTE_SIZE,
        sha256=LICENSE_SHA256,
        git_blob=LICENSE_GIT_BLOB_SHA1,
    )
    return dataset, license_bytes


def parse_manifest_bytes(data: bytes) -> dict[str, object]:
    manifest = _require_object(_strict_json(data, "upstream manifest"), "upstream manifest")
    _require_exact_keys(
        manifest,
        {"schema_version", "whatwg_html", "dataset", "derived"},
        "upstream manifest",
    )
    if _require_integer(manifest["schema_version"], "manifest schema_version") != 1:
        raise GenerationError("unsupported manifest schema")

    whatwg = _require_object(manifest["whatwg_html"], "manifest whatwg_html")
    _require_exact_keys(whatwg, {"repository", "commit", "parent", "license"}, "manifest whatwg_html")
    if whatwg["repository"] != "whatwg/html":
        raise GenerationError("manifest WHATWG repository mismatch")
    if whatwg["commit"] != WHATWG_HTML_SNAPSHOT:
        raise GenerationError("manifest WHATWG snapshot does not match frozen authority")
    if whatwg["parent"] != WHATWG_HTML_PARENT:
        raise GenerationError("manifest WHATWG parent does not match frozen authority")

    license_value = _require_object(whatwg["license"], "manifest license")
    _require_exact_keys(
        license_value,
        {
            "upstream_path",
            "upstream_blob_sha1",
            "retained_path",
            "byte_size",
            "sha256",
            "local_blob_sha1",
        },
        "manifest license",
    )
    expected_license: dict[str, object] = {
        "upstream_path": "LICENSE",
        "upstream_blob_sha1": LICENSE_GIT_BLOB_SHA1,
        "retained_path": LICENSE_PATH.as_posix(),
        "byte_size": LICENSE_BYTE_SIZE,
        "sha256": LICENSE_SHA256,
        "local_blob_sha1": LICENSE_GIT_BLOB_SHA1,
    }
    _require_integer(license_value["byte_size"], "manifest license byte_size")
    if license_value != expected_license:
        raise GenerationError("manifest license identity does not match frozen authority")

    dataset = _require_object(manifest["dataset"], "manifest dataset")
    _require_exact_keys(
        dataset,
        {"official_url", "retained_path", "byte_size", "sha256", "local_blob_sha1"},
        "manifest dataset",
    )
    expected_dataset: dict[str, object] = {
        "official_url": DATASET_URL,
        "retained_path": DATASET_PATH.as_posix(),
        "byte_size": DATASET_BYTE_SIZE,
        "sha256": DATASET_SHA256,
        "local_blob_sha1": DATASET_GIT_BLOB_SHA1,
    }
    _require_integer(dataset["byte_size"], "manifest dataset byte_size")
    if dataset != expected_dataset:
        raise GenerationError("manifest dataset identity does not match frozen authority")

    derived = _require_object(manifest["derived"], "manifest derived")
    _require_exact_keys(
        derived,
        {
            "entry_count",
            "semicolonless_entry_count",
            "two_scalar_entry_count",
            "maximum_generated_key_byte_length",
            "maximum_generated_keys",
        },
        "manifest derived",
    )
    integer_fields = {
        "entry_count": EXPECTED_ENTRY_COUNT,
        "semicolonless_entry_count": EXPECTED_SEMICOLONLESS_COUNT,
        "two_scalar_entry_count": EXPECTED_TWO_SCALAR_COUNT,
        "maximum_generated_key_byte_length": EXPECTED_MAXIMUM_KEY_BYTE_LENGTH,
    }
    for name, expected in integer_fields.items():
        if _require_integer(derived[name], f"manifest derived {name}") != expected:
            raise GenerationError(f"manifest derived {name} does not match frozen authority")
    maximum_keys = derived["maximum_generated_keys"]
    if not isinstance(maximum_keys, list) or any(type(item) is not str for item in maximum_keys):
        raise GenerationError("manifest maximum_generated_keys must be a string array")
    if tuple(maximum_keys) != EXPECTED_MAXIMUM_KEYS:
        raise GenerationError("manifest maximum_generated_keys does not match frozen authority")
    return manifest


def load_manifest(path: Path) -> tuple[dict[str, object], bytes]:
    data = _read_file(path, "upstream manifest")
    return parse_manifest_bytes(data), data


def _validate_entity_key(name: object) -> str:
    if type(name) is not str:
        raise GenerationError("entity key must be a JSON string")
    if not name.startswith("&"):
        raise GenerationError(f"invalid entity key {name!r}: missing leading ampersand")
    if name.startswith("&&"):
        raise GenerationError(f"invalid entity key {name!r}: multiple leading ampersands")
    if not name.isascii():
        raise GenerationError(f"invalid entity key {name!r}: non-ASCII spelling")
    if _ENTITY_KEY_RE.fullmatch(name) is None:
        raise GenerationError(f"invalid entity key {name!r}: malformed named-reference spelling")
    return name


def _validate_entity_record(name: str, value: object) -> Entity:
    record = _require_object(value, f"entity {name}")
    _require_exact_keys(record, {"codepoints", "characters"}, f"entity {name}")

    codepoints_value = record["codepoints"]
    if not isinstance(codepoints_value, list):
        raise GenerationError(f"entity {name}: codepoints must be a JSON array")
    if len(codepoints_value) not in {1, 2}:
        raise GenerationError(f"entity {name}: codepoints length must be exactly one or two")
    codepoints: list[int] = []
    for index, value_item in enumerate(codepoints_value):
        if type(value_item) is not int:
            raise GenerationError(
                f"entity {name}: codepoints[{index}] must be a genuine JSON integer"
            )
        if not 0 <= value_item <= 0x10FFFF:
            raise GenerationError(f"entity {name}: codepoints[{index}] is outside Unicode range")
        if 0xD800 <= value_item <= 0xDFFF:
            raise GenerationError(f"entity {name}: codepoints[{index}] is a surrogate")
        codepoints.append(value_item)

    characters = record["characters"]
    if type(characters) is not str:
        raise GenerationError(f"entity {name}: characters must be a JSON string")
    if any(0xD800 <= ord(character) <= 0xDFFF for character in characters):
        raise GenerationError(f"entity {name}: characters contains an unpaired surrogate")
    expected_characters = "".join(chr(codepoint) for codepoint in codepoints)
    if characters != expected_characters:
        raise GenerationError(f"entity {name}: codepoints and characters mismatch")

    return Entity(name, name[1:], tuple(codepoints), characters)


def parse_and_validate_entities(data: bytes) -> Dataset:
    source = _require_object(_strict_json(data, "entities.json"), "entities.json")
    if len(source) != EXPECTED_ENTRY_COUNT:
        raise GenerationError(
            f"entities.json: entry count mismatch expected={EXPECTED_ENTRY_COUNT} actual={len(source)}"
        )

    entries: list[Entity] = []
    for raw_name, value in source.items():
        name = _validate_entity_key(raw_name)
        entries.append(_validate_entity_record(name, value))
    entries.sort(key=lambda item: item.generated_name.encode("ascii"))

    generated_names = [item.generated_name for item in entries]
    if len(set(generated_names)) != len(generated_names):
        raise GenerationError("stripping one leading ampersand is not injective")
    by_source = {item.source_name: item for item in entries}
    for item in entries:
        if item.source_name.endswith(";"):
            continue
        counterpart = by_source.get(item.source_name + ";")
        if counterpart is None:
            raise GenerationError(f"entity {item.source_name}: missing semicolon counterpart")
        if counterpart.codepoints != item.codepoints or counterpart.characters != item.characters:
            raise GenerationError(f"entity {item.source_name}: semicolon counterpart meaning mismatch")

    for name, expected_codepoints in _CHALLENGE_CELLS.items():
        item = by_source.get(name)
        if item is None or item.codepoints != expected_codepoints:
            raise GenerationError(f"challenge cell {name} does not match frozen semantics")

    semicolonless_count = sum(not item.generated_name.endswith(";") for item in entries)
    two_scalar_count = sum(len(item.codepoints) == 2 for item in entries)
    maximum_key_byte_length = max(len(item.generated_name.encode("ascii")) for item in entries)
    maximum_keys = tuple(
        item.generated_name
        for item in entries
        if len(item.generated_name.encode("ascii")) == maximum_key_byte_length
    )
    if semicolonless_count != EXPECTED_SEMICOLONLESS_COUNT:
        raise GenerationError("derived semicolonless entry count mismatch")
    if two_scalar_count != EXPECTED_TWO_SCALAR_COUNT:
        raise GenerationError("derived two-scalar entry count mismatch")
    if maximum_key_byte_length != EXPECTED_MAXIMUM_KEY_BYTE_LENGTH:
        raise GenerationError("derived maximum generated key byte length mismatch")
    if maximum_keys != EXPECTED_MAXIMUM_KEYS:
        raise GenerationError("derived maximum generated key set mismatch")
    return Dataset(
        tuple(entries),
        semicolonless_count,
        two_scalar_count,
        maximum_key_byte_length,
        maximum_keys,
    )


def _validate_manifest_derived(manifest: dict[str, object], dataset: Dataset) -> None:
    derived = _require_object(manifest["derived"], "manifest derived")
    observed: dict[str, object] = {
        "entry_count": len(dataset.entries),
        "semicolonless_entry_count": dataset.semicolonless_count,
        "two_scalar_entry_count": dataset.two_scalar_count,
        "maximum_generated_key_byte_length": dataset.maximum_key_byte_length,
        "maximum_generated_keys": list(dataset.maximum_keys),
    }
    if derived != observed:
        raise GenerationError("manifest derived metadata disagrees with retained dataset")


def _rust_scalar_string(codepoints: tuple[int, ...]) -> str:
    return '"' + "".join(f"\\u{{{codepoint:X}}}" for codepoint in codepoints) + '"'


def render_rust(dataset: Dataset, manifest_bytes: bytes) -> bytes:
    manifest_sha256 = hashlib.sha256(manifest_bytes).hexdigest()
    lines = [
        "// @generated by tools/html/named_character_references/"
        "generate_named_character_references.py; do not edit.",
        f"// WHATWG HTML snapshot: {WHATWG_HTML_SNAPSHOT}",
        f"// retained entities.json SHA-256: {DATASET_SHA256}",
        f"// upstream-manifest.json SHA-256: {manifest_sha256}",
        "",
        f'pub(super) const WHATWG_HTML_SNAPSHOT: &str = "{WHATWG_HTML_SNAPSHOT}";',
        "pub(super) const RETAINED_ENTITIES_SHA256: &str =",
        f'    "{DATASET_SHA256}";',
        "pub(super) const UPSTREAM_MANIFEST_SHA256: &str =",
        f'    "{manifest_sha256}";',
        "pub(super) const NAMED_CHARACTER_REFERENCE_ENTRY_COUNT: usize = "
        f"{len(dataset.entries)};",
        "pub(super) const NAMED_CHARACTER_REFERENCE_SEMICOLONLESS_ENTRY_COUNT: usize = "
        f"{dataset.semicolonless_count};",
        "pub(super) const NAMED_CHARACTER_REFERENCE_TWO_SCALAR_ENTRY_COUNT: usize = "
        f"{dataset.two_scalar_count};",
        "pub(super) const NAMED_CHARACTER_REFERENCE_MAXIMUM_NAME_BYTE_LENGTH: usize = "
        f"{dataset.maximum_key_byte_length};",
        "",
        "pub(super) const NAMED_CHARACTER_REFERENCE_MAXIMUM_NAMES: &[&str] =",
        "    &[" + ", ".join(json.dumps(name) for name in dataset.maximum_keys) + "];",
    ]
    lines.extend(
        [
            "",
            "// BEGIN NAMED CHARACTER REFERENCES",
            "pub(super) const NAMED_CHARACTER_REFERENCES: &[(&str, &str)] = &[",
        ]
    )
    for item in dataset.entries:
        lines.append(
            f"    ({json.dumps(item.generated_name)}, {_rust_scalar_string(item.codepoints)}),"
        )
    lines.extend(["];", "// END NAMED CHARACTER REFERENCES"])
    return ("\n".join(lines) + "\n").encode("utf-8")


def generate(repo_root: Path) -> bytes:
    dataset_bytes, _license_bytes = load_frozen_evidence(repo_root)
    manifest, manifest_bytes = load_manifest(repo_root / MANIFEST_PATH)
    dataset = parse_and_validate_entities(dataset_bytes)
    _validate_manifest_derived(manifest, dataset)
    return render_rust(dataset, manifest_bytes)


def _atomic_write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.chmod(temporary, 0o644)
        os.replace(temporary, path)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    repo_root = Path(__file__).resolve().parents[3]
    output_path = repo_root / OUTPUT_PATH
    try:
        generated = generate(repo_root)
        if args.check:
            current = _read_file(output_path, "generated Rust output")
            if current != generated:
                raise GenerationError("generated Rust output is stale")
        else:
            _atomic_write(output_path, generated)
    except GenerationError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
