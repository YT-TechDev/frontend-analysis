#!/usr/bin/env python3
"""Independently verify retained WHATWG entities against generated Rust data.

The canonical generated representation is this script's own semantic and
provenance subject, so it is parsed exactly and fails closed on anything it
cannot account for. It parses no other repository Rust, resolves no module
wiring, and evaluates no `cfg`, `#[path]`, or `include!` semantics.
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

PINNED_WHATWG_COMMIT = "8ad51e24e9d9e48d92317467f434f7192df9d63d"
PINNED_WHATWG_PARENT = "9ead9de8f6751ccb98e91972e580ed6e3314c64a"
PINNED_ENTITIES_URL = "https://html.spec.whatwg.org/entities.json"
PINNED_ENTITIES_PATH = Path("tools/html/named_character_references/inputs/entities.json")
PINNED_ENTITIES_SIZE = 145_897
PINNED_ENTITIES_SHA256 = "d741d877ac77c4194c4ad526b5b4a19aef8dfe411ab840a466891cdbb9f362e6"
PINNED_ENTITIES_BLOB = "557170b41f47a13a46ec695561eb5fe76da73bdb"
PINNED_LICENSE_PATH = Path("tools/html/named_character_references/WHATWG-LICENSE.txt")
PINNED_LICENSE_SIZE = 16_315
PINNED_LICENSE_SHA256 = "85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864"
PINNED_LICENSE_BLOB = "f2dcda46deccefd245749202a88a7837e35c6daa"
RETAINED_MANIFEST_PATH = Path("tools/html/named_character_references/upstream-manifest.json")
GENERATED_RUST_PATH = Path(
    "crates/frontend-analysis-core/src/html/tokenizer/"
    "named_character_references_generated.rs"
)

PINNED_ENTRY_COUNT = 2_231
PINNED_SEMICOLONLESS_COUNT = 106
PINNED_TWO_SCALAR_COUNT = 93
PINNED_MAXIMUM_NAME_BYTES = 32
PINNED_MAXIMUM_NAMES = ("CounterClockwiseContourIntegral;",)

# The canonical generated source is lexically included inside the private owner
# module, so it declares no visibility of its own and carries exactly one fixed
# ownership registration naming the owner's private authority.
EXPECTED_REGISTRATION_BEGIN = "// BEGIN CANONICAL OWNERSHIP REGISTRATION"
EXPECTED_REGISTRATION_ITEM = "impl OwnershipRegistration for OwnerToken {}"
EXPECTED_REGISTRATION_END = "// END CANONICAL OWNERSHIP REGISTRATION"

_SOURCE_NAME = re.compile(r"&[A-Za-z0-9]+;?", flags=re.ASCII)
_GENERATED_NAME = r"[A-Za-z0-9]+;?"
_ROW = re.compile(
    rf'^    \("({_GENERATED_NAME})", "((?:\\u\{{(?:0|[1-9A-F][0-9A-F]{{0,5}})\}}){{1,2}})"\),$'
)
_MAXIMUM_NAMES = re.compile(rf'^    &\["({_GENERATED_NAME})"\];$')
_CHALLENGES = {
    "amp;": (0x26,),
    "amp": (0x26,),
    "lt;": (0x3C,),
    "AMP;": (0x26,),
    "not": (0xAC,),
    "not;": (0xAC,),
    "notin;": (0x2209,),
    "acE;": (0x223E, 0x0333),
    "CounterClockwiseContourIntegral;": (0x2233,),
    "Afr;": (0x1D504,),
}


class VerificationError(RuntimeError):
    """Raised when independent verification cannot prove exact equality."""


@dataclass(frozen=True)
class SourceData:
    values: dict[str, tuple[int, ...]]
    semicolonless_count: int
    two_scalar_count: int
    maximum_name_bytes: int
    maximum_names: tuple[str, ...]


@dataclass(frozen=True)
class GeneratedData:
    values: dict[str, tuple[int, ...]]
    ordered_names: tuple[str, ...]
    maximum_names: tuple[str, ...]
    header_snapshot: str
    header_entities_sha256: str
    header_manifest_sha256: str
    constant_snapshot: str
    constant_entities_sha256: str
    constant_manifest_sha256: str
    entry_count: int
    semicolonless_count: int
    two_scalar_count: int
    maximum_name_bytes: int


def _blob_identity(data: bytes) -> str:
    prefix = f"blob {len(data)}\0".encode("ascii")
    return hashlib.sha1(prefix + data, usedforsecurity=False).hexdigest()


def _read(path: Path, label: str) -> bytes:
    try:
        return path.read_bytes()
    except OSError as error:
        raise VerificationError(f"{label}: file unavailable") from error


def _assert_frozen(
    data: bytes,
    *,
    label: str,
    expected_size: int,
    expected_sha256: str,
    expected_blob: str,
) -> None:
    if len(data) != expected_size:
        raise VerificationError(f"{label}: independently frozen byte size mismatch")
    if hashlib.sha256(data).hexdigest() != expected_sha256:
        raise VerificationError(f"{label}: independently frozen SHA-256 mismatch")
    if _blob_identity(data) != expected_blob:
        raise VerificationError(f"{label}: independently frozen Git blob mismatch")


def _decode_json_strict(data: bytes, label: str) -> object:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise VerificationError(f"{label}: not UTF-8") from error

    def pairs_to_unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise VerificationError(f"{label}: duplicate JSON member {key!r}")
            result[key] = value
        return result

    def reject_extension(token: str) -> object:
        raise VerificationError(f"{label}: non-finite JSON number {token}")

    try:
        return json.loads(
            text,
            object_pairs_hook=pairs_to_unique_object,
            parse_constant=reject_extension,
        )
    except VerificationError:
        raise
    except json.JSONDecodeError as error:
        raise VerificationError(f"{label}: malformed JSON: {error.msg}") from error


def _object(value: object, label: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise VerificationError(f"{label}: expected object")
    return value


def _fields(value: dict[str, object], names: set[str], label: str) -> None:
    if set(value) != names:
        raise VerificationError(f"{label}: unexpected object fields")


def _json_integer(value: object, label: str) -> int:
    if type(value) is not int:
        raise VerificationError(f"{label}: expected genuine JSON integer")
    return value


def parse_manifest(data: bytes) -> dict[str, object]:
    root = _object(_decode_json_strict(data, "manifest"), "manifest")
    _fields(root, {"schema_version", "whatwg_html", "dataset", "derived"}, "manifest")
    if _json_integer(root["schema_version"], "manifest schema_version") != 1:
        raise VerificationError("manifest schema is not independently supported")

    context = _object(root["whatwg_html"], "manifest whatwg_html")
    _fields(context, {"repository", "commit", "parent", "license"}, "manifest whatwg_html")
    if context["repository"] != "whatwg/html":
        raise VerificationError("manifest WHATWG repository mismatch")
    if context["commit"] != PINNED_WHATWG_COMMIT:
        raise VerificationError("manifest WHATWG snapshot differs from independent pin")
    if context["parent"] != PINNED_WHATWG_PARENT:
        raise VerificationError("manifest WHATWG parent differs from independent pin")

    license_value = _object(context["license"], "manifest license")
    _fields(
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
    if license_value != {
        "upstream_path": "LICENSE",
        "upstream_blob_sha1": PINNED_LICENSE_BLOB,
        "retained_path": PINNED_LICENSE_PATH.as_posix(),
        "byte_size": PINNED_LICENSE_SIZE,
        "sha256": PINNED_LICENSE_SHA256,
        "local_blob_sha1": PINNED_LICENSE_BLOB,
    }:
        raise VerificationError("manifest license provenance differs from independent pin")
    _json_integer(license_value["byte_size"], "manifest license byte_size")

    dataset_value = _object(root["dataset"], "manifest dataset")
    _fields(
        dataset_value,
        {"official_url", "retained_path", "byte_size", "sha256", "local_blob_sha1"},
        "manifest dataset",
    )
    if dataset_value != {
        "official_url": PINNED_ENTITIES_URL,
        "retained_path": PINNED_ENTITIES_PATH.as_posix(),
        "byte_size": PINNED_ENTITIES_SIZE,
        "sha256": PINNED_ENTITIES_SHA256,
        "local_blob_sha1": PINNED_ENTITIES_BLOB,
    }:
        raise VerificationError("manifest dataset identity differs from independent pin")
    _json_integer(dataset_value["byte_size"], "manifest dataset byte_size")

    derived = _object(root["derived"], "manifest derived")
    _fields(
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
    expected_numbers = {
        "entry_count": PINNED_ENTRY_COUNT,
        "semicolonless_entry_count": PINNED_SEMICOLONLESS_COUNT,
        "two_scalar_entry_count": PINNED_TWO_SCALAR_COUNT,
        "maximum_generated_key_byte_length": PINNED_MAXIMUM_NAME_BYTES,
    }
    for key, expected in expected_numbers.items():
        if _json_integer(derived[key], f"manifest {key}") != expected:
            raise VerificationError(f"manifest {key} differs from independent pin")
    names = derived["maximum_generated_keys"]
    if not isinstance(names, list) or any(type(item) is not str for item in names):
        raise VerificationError("manifest maximum names is not a string array")
    if tuple(names) != PINNED_MAXIMUM_NAMES:
        raise VerificationError("manifest maximum names differ from independent pin")
    return root


def parse_source(data: bytes) -> SourceData:
    root = _object(_decode_json_strict(data, "entities source"), "entities source")
    if len(root) != PINNED_ENTRY_COUNT:
        raise VerificationError(
            f"entities source: entry count mismatch expected={PINNED_ENTRY_COUNT} actual={len(root)}"
        )

    generated: dict[str, tuple[int, ...]] = {}
    raw_values: dict[str, tuple[int, ...]] = {}
    for name_value, record_value in root.items():
        if type(name_value) is not str or _SOURCE_NAME.fullmatch(name_value) is None:
            raise VerificationError(f"entities source: malformed exact name {name_value!r}")
        if name_value.startswith("&&") or not name_value.isascii():
            raise VerificationError(f"entities source: malformed exact name {name_value!r}")
        record = _object(record_value, f"source entity {name_value}")
        _fields(record, {"codepoints", "characters"}, f"source entity {name_value}")
        raw_codepoints = record["codepoints"]
        if not isinstance(raw_codepoints, list) or len(raw_codepoints) not in {1, 2}:
            raise VerificationError(f"source entity {name_value}: invalid codepoints shape")
        codepoints: list[int] = []
        for codepoint in raw_codepoints:
            if type(codepoint) is not int:
                raise VerificationError(f"source entity {name_value}: non-integer codepoint")
            if codepoint < 0 or codepoint > 0x10FFFF:
                raise VerificationError(f"source entity {name_value}: out-of-range codepoint")
            if 0xD800 <= codepoint <= 0xDFFF:
                raise VerificationError(f"source entity {name_value}: surrogate codepoint")
            codepoints.append(codepoint)
        characters = record["characters"]
        if type(characters) is not str:
            raise VerificationError(f"source entity {name_value}: characters is not a string")
        if any(0xD800 <= ord(character) <= 0xDFFF for character in characters):
            raise VerificationError(f"source entity {name_value}: unpaired surrogate text")
        if characters != "".join(chr(codepoint) for codepoint in codepoints):
            raise VerificationError(f"source entity {name_value}: decoded meaning mismatch")
        value = tuple(codepoints)
        raw_values[name_value] = value
        generated_name = name_value[1:]
        if generated_name in generated:
            raise VerificationError("leading-ampersand stripping is not injective")
        generated[generated_name] = value

    for raw_name, value in raw_values.items():
        if raw_name.endswith(";"):
            continue
        counterpart = raw_values.get(raw_name + ";")
        if counterpart is None:
            raise VerificationError(f"source entity {raw_name}: missing semicolon counterpart")
        if counterpart != value:
            raise VerificationError(f"source entity {raw_name}: counterpart meaning differs")

    for name, value in _CHALLENGES.items():
        if generated.get(name) != value:
            raise VerificationError(f"source challenge {name} mismatch")

    semicolonless_count = sum(not name.endswith(";") for name in generated)
    two_scalar_count = sum(len(value) == 2 for value in generated.values())
    maximum_name_bytes = max(len(name.encode("ascii")) for name in generated)
    maximum_names = tuple(
        sorted(
            name for name in generated if len(name.encode("ascii")) == maximum_name_bytes
        )
    )
    if semicolonless_count != PINNED_SEMICOLONLESS_COUNT:
        raise VerificationError("source semicolonless count differs from independent pin")
    if two_scalar_count != PINNED_TWO_SCALAR_COUNT:
        raise VerificationError("source two-scalar count differs from independent pin")
    if maximum_name_bytes != PINNED_MAXIMUM_NAME_BYTES:
        raise VerificationError("source maximum name length differs from independent pin")
    if maximum_names != PINNED_MAXIMUM_NAMES:
        raise VerificationError("source maximum name set differs from independent pin")
    return SourceData(
        generated,
        semicolonless_count,
        two_scalar_count,
        maximum_name_bytes,
        maximum_names,
    )


def _consume_exact(lines: list[str], index: int, expected: str, label: str) -> int:
    if index >= len(lines) or lines[index] != expected:
        raise VerificationError(f"generated Rust: {label} does not match canonical grammar")
    return index + 1


def _consume_capture(
    lines: list[str], index: int, pattern: str, label: str
) -> tuple[str, int]:
    if index >= len(lines):
        raise VerificationError(f"generated Rust: missing {label}")
    match = re.fullmatch(pattern, lines[index])
    if match is None:
        raise VerificationError(f"generated Rust: malformed {label}")
    return match.group(1), index + 1


def _decode_rust_scalars(encoded: str) -> tuple[int, ...]:
    pieces = re.findall(r"\\u\{([0-9A-F]+)\}", encoded)
    if not pieces or "".join(f"\\u{{{piece}}}" for piece in pieces) != encoded:
        raise VerificationError("generated Rust: unparsed decoded scalar syntax")
    values = tuple(int(piece, 16) for piece in pieces)
    if len(values) not in {1, 2}:
        raise VerificationError("generated Rust: decoded value is not one or two scalars")
    for value in values:
        if value > 0x10FFFF or 0xD800 <= value <= 0xDFFF:
            raise VerificationError("generated Rust: decoded value is not a Unicode scalar")
    return values


def parse_generated_rust(data: bytes) -> GeneratedData:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise VerificationError("generated Rust is not UTF-8") from error
    if not text.endswith("\n"):
        raise VerificationError("generated Rust lacks canonical final LF")
    lines = text.splitlines()
    index = 0
    index = _consume_exact(
        lines,
        index,
        "// @generated by tools/html/named_character_references/"
        "generate_named_character_references.py; do not edit.",
        "generated header",
    )
    header_snapshot, index = _consume_capture(
        lines, index, r"// WHATWG HTML snapshot: ([0-9a-f]{40})", "snapshot header"
    )
    header_entities_sha256, index = _consume_capture(
        lines,
        index,
        r"// retained entities\.json SHA-256: ([0-9a-f]{64})",
        "entities hash header",
    )
    header_manifest_sha256, index = _consume_capture(
        lines,
        index,
        r"// upstream-manifest\.json SHA-256: ([0-9a-f]{64})",
        "manifest hash header",
    )
    index = _consume_exact(lines, index, "", "header separator")
    index = _consume_exact(
        lines, index, EXPECTED_REGISTRATION_BEGIN, "ownership registration begin marker"
    )
    index = _consume_exact(
        lines, index, EXPECTED_REGISTRATION_ITEM, "ownership registration item"
    )
    index = _consume_exact(
        lines, index, EXPECTED_REGISTRATION_END, "ownership registration end marker"
    )
    index = _consume_exact(lines, index, "", "ownership registration separator")
    constant_snapshot, index = _consume_capture(
        lines,
        index,
        r'const WHATWG_HTML_SNAPSHOT: &str = "([0-9a-f]{40})";',
        "snapshot constant",
    )
    index = _consume_exact(
        lines,
        index,
        "const RETAINED_ENTITIES_SHA256: &str =",
        "entities hash declaration",
    )
    constant_entities_sha256, index = _consume_capture(
        lines,
        index,
        r'    "([0-9a-f]{64})";',
        "entities hash constant",
    )
    index = _consume_exact(
        lines,
        index,
        "const UPSTREAM_MANIFEST_SHA256: &str =",
        "manifest hash declaration",
    )
    constant_manifest_sha256, index = _consume_capture(
        lines,
        index,
        r'    "([0-9a-f]{64})";',
        "manifest hash constant",
    )
    entry_count_text, index = _consume_capture(
        lines,
        index,
        r"const NAMED_CHARACTER_REFERENCE_ENTRY_COUNT: usize = ([0-9]+);",
        "entry count constant",
    )
    semicolonless_text, index = _consume_capture(
        lines,
        index,
        r"const NAMED_CHARACTER_REFERENCE_SEMICOLONLESS_ENTRY_COUNT: usize = ([0-9]+);",
        "semicolonless count constant",
    )
    two_scalar_text, index = _consume_capture(
        lines,
        index,
        r"const NAMED_CHARACTER_REFERENCE_TWO_SCALAR_ENTRY_COUNT: usize = ([0-9]+);",
        "two-scalar count constant",
    )
    maximum_length_text, index = _consume_capture(
        lines,
        index,
        r"const NAMED_CHARACTER_REFERENCE_MAXIMUM_NAME_BYTE_LENGTH: usize = ([0-9]+);",
        "maximum length constant",
    )
    index = _consume_exact(lines, index, "", "metadata separator")
    index = _consume_exact(
        lines,
        index,
        "const NAMED_CHARACTER_REFERENCE_MAXIMUM_NAMES: &[&str] =",
        "maximum names declaration",
    )
    if index >= len(lines):
        raise VerificationError("generated Rust: missing maximum-name value")
    maximum_match = _MAXIMUM_NAMES.fullmatch(lines[index])
    if maximum_match is None:
        raise VerificationError("generated Rust: unexpected syntax in maximum-name region")
    maximum_names = [maximum_match.group(1)]
    index += 1
    index = _consume_exact(lines, index, "", "table separator")
    index = _consume_exact(
        lines, index, "// BEGIN NAMED CHARACTER REFERENCES", "table begin marker"
    )
    index = _consume_exact(
        lines,
        index,
        "const NAMED_CHARACTER_REFERENCES: &[(&str, &str)] = &[",
        "table declaration",
    )

    ordered_names: list[str] = []
    values: dict[str, tuple[int, ...]] = {}
    while index < len(lines) and lines[index] != "];":
        match = _ROW.fullmatch(lines[index])
        if match is None:
            raise VerificationError("generated Rust: unexpected syntax inside generated data region")
        name = match.group(1)
        if name in values:
            raise VerificationError(f"generated Rust: duplicate generated key {name}")
        ordered_names.append(name)
        values[name] = _decode_rust_scalars(match.group(2))
        index += 1
    index = _consume_exact(lines, index, "];", "table terminator")
    index = _consume_exact(
        lines, index, "// END NAMED CHARACTER REFERENCES", "table end marker"
    )
    if index != len(lines):
        raise VerificationError("generated Rust: unexpected content outside canonical representation")
    if ordered_names != sorted(ordered_names, key=lambda name: name.encode("ascii")):
        raise VerificationError("generated Rust: rows are not in canonical ASCII key order")

    return GeneratedData(
        values,
        tuple(ordered_names),
        tuple(maximum_names),
        header_snapshot,
        header_entities_sha256,
        header_manifest_sha256,
        constant_snapshot,
        constant_entities_sha256,
        constant_manifest_sha256,
        int(entry_count_text),
        int(semicolonless_text),
        int(two_scalar_text),
        int(maximum_length_text),
    )


def verify_generated_semantics(
    source: SourceData, generated: GeneratedData, manifest_sha256: str
) -> None:
    if generated.header_snapshot != PINNED_WHATWG_COMMIT:
        raise VerificationError("generated header snapshot mismatch")
    if generated.constant_snapshot != PINNED_WHATWG_COMMIT:
        raise VerificationError("generated snapshot constant mismatch")
    if generated.header_entities_sha256 != PINNED_ENTITIES_SHA256:
        raise VerificationError("generated header dataset hash mismatch")
    if generated.constant_entities_sha256 != PINNED_ENTITIES_SHA256:
        raise VerificationError("generated dataset hash constant mismatch")
    if generated.header_manifest_sha256 != manifest_sha256:
        raise VerificationError("generated header manifest hash mismatch")
    if generated.constant_manifest_sha256 != manifest_sha256:
        raise VerificationError("generated manifest hash constant mismatch")

    source_keys = set(source.values)
    generated_keys = set(generated.values)
    if source_keys != generated_keys:
        missing = sorted(source_keys - generated_keys)[:8]
        extra = sorted(generated_keys - source_keys)[:8]
        raise VerificationError(f"generated key-set mismatch missing={missing} extra={extra}")
    for name in sorted(source_keys):
        if source.values[name] != generated.values[name]:
            raise VerificationError(f"generated decoded value mismatch for {name}")

    semicolonless = sum(not name.endswith(";") for name in generated.values)
    two_scalar = sum(len(value) == 2 for value in generated.values.values())
    maximum_length = max(len(name.encode("ascii")) for name in generated.values)
    maximum_names = tuple(
        name
        for name in generated.ordered_names
        if len(name.encode("ascii")) == maximum_length
    )
    expected_metadata = (
        len(source.values),
        source.semicolonless_count,
        source.two_scalar_count,
        source.maximum_name_bytes,
        source.maximum_names,
    )
    observed_metadata = (
        generated.entry_count,
        generated.semicolonless_count,
        generated.two_scalar_count,
        generated.maximum_name_bytes,
        generated.maximum_names,
    )
    if observed_metadata != expected_metadata:
        raise VerificationError("generated metadata differs from source-derived metadata")
    if (semicolonless, two_scalar, maximum_length, maximum_names) != (
        source.semicolonless_count,
        source.two_scalar_count,
        source.maximum_name_bytes,
        source.maximum_names,
    ):
        raise VerificationError("generated row-derived metadata mismatch")
    for name, expected in _CHALLENGES.items():
        if generated.values.get(name) != expected:
            raise VerificationError(f"generated challenge {name} mismatch")


def verify_root(repo_root: Path) -> SourceData:
    entities = _read(repo_root / PINNED_ENTITIES_PATH, "entities.json")
    license_bytes = _read(repo_root / PINNED_LICENSE_PATH, "WHATWG LICENSE")
    _assert_frozen(
        entities,
        label="entities.json",
        expected_size=PINNED_ENTITIES_SIZE,
        expected_sha256=PINNED_ENTITIES_SHA256,
        expected_blob=PINNED_ENTITIES_BLOB,
    )
    _assert_frozen(
        license_bytes,
        label="WHATWG LICENSE",
        expected_size=PINNED_LICENSE_SIZE,
        expected_sha256=PINNED_LICENSE_SHA256,
        expected_blob=PINNED_LICENSE_BLOB,
    )
    manifest_bytes = _read(repo_root / RETAINED_MANIFEST_PATH, "manifest")
    manifest = parse_manifest(manifest_bytes)
    source = parse_source(entities)
    derived = _object(manifest["derived"], "manifest derived")
    if derived != {
        "entry_count": len(source.values),
        "semicolonless_entry_count": source.semicolonless_count,
        "two_scalar_entry_count": source.two_scalar_count,
        "maximum_generated_key_byte_length": source.maximum_name_bytes,
        "maximum_generated_keys": list(source.maximum_names),
    }:
        raise VerificationError("manifest derived data disagrees with independently parsed source")
    generated = parse_generated_rust(_read(repo_root / GENERATED_RUST_PATH, "generated Rust"))
    verify_generated_semantics(source, generated, hashlib.sha256(manifest_bytes).hexdigest())
    return source


def main() -> int:
    repo_root = Path(__file__).resolve().parents[3]
    try:
        source = verify_root(repo_root)
    except VerificationError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    print(
        f"entries={len(source.values)} "
        f"semicolonless={source.semicolonless_count} "
        f"two_scalar={source.two_scalar_count} "
        f"maximum_name_bytes={source.maximum_name_bytes}"
    )
    print(f"maximum_names={','.join(source.maximum_names)}")
    print("complete semantic equality: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
