#!/usr/bin/env python3
"""Generate bounded, browser-independent ECMAScript Unicode qualification data."""
from __future__ import annotations

import argparse
import hashlib
import html
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence

UNICODE_VERSION = "17.0.0"
ECMA262_SNAPSHOT = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
UNICODE_MIRROR_COMMIT = "a363a170c5ecb1c509535f6730dd19e720443cd9"
EXPECTED_ID_START_COUNT = 145_916
EXPECTED_ID_CONTINUE_COUNT = 149_240
EXPECTED_ZS_COUNT = 17
EXPECTED_INPUT_IDS = frozenset({
    "unicode-derived-core-properties",
    "unicode-derived-general-category",
    "unicode-property-value-aliases",
    "ecma262-nonbinary-unicode-properties",
    "ecma262-binary-unicode-properties",
    "ecma262-binary-unicode-properties-of-strings",
})
EXPECTED_BLOB_SHA1 = {
    "unicode-derived-core-properties": "f327784bf3956436efeb85e213fc637d7b3c0207",
    "unicode-derived-general-category": "41996d6348fdcd7f1a88127766b7e6b21b6159e3",
    "unicode-property-value-aliases": "b92662eda2867baed56e55440e5187d52c1cb341",
    "ecma262-nonbinary-unicode-properties": "2bf328116d7085f3e220c8325d0fb737b3bd19a6",
    "ecma262-binary-unicode-properties": "8ab11640ed8bd6234a3c2b42a36a99847eee76ef",
    "ecma262-binary-unicode-properties-of-strings": "1b6479a126902e04394f8bb684e7d3740bde608e",
}
EXPECTED_BYTE_SIZE = {
    "unicode-derived-core-properties": 1_134_783,
    "unicode-derived-general-category": 277_514,
    "unicode-property-value-aliases": 81_858,
    "ecma262-nonbinary-unicode-properties": 1_033,
    "ecma262-binary-unicode-properties": 10_528,
    "ecma262-binary-unicode-properties-of-strings": 702,
}
EXPECTED_STRING_PROPERTIES = (
    "Basic_Emoji",
    "Emoji_Keycap_Sequence",
    "RGI_Emoji_Modifier_Sequence",
    "RGI_Emoji_Flag_Sequence",
    "RGI_Emoji_Tag_Sequence",
    "RGI_Emoji_ZWJ_Sequence",
    "RGI_Emoji",
)

_RANGE_RE = re.compile(r"^([0-9A-F]{4,6})(?:\.\.([0-9A-F]{4,6}))?\s*;\s*([A-Za-z_]+)(?:\s|$)")
_ROW_RE = re.compile(r"<tr>(.*?)</tr>", re.DOTALL)
_TICK_RE = re.compile(r"`([^`]+)`")
_TABLE_ID_RE = re.compile(r'<emu-table\s+id="([^"]+)">')

class GenerationError(RuntimeError):
    pass

@dataclass(frozen=True, order=True)
class CodePointRange:
    start: int
    end: int
    def __post_init__(self) -> None:
        if not (0 <= self.start <= self.end <= 0x10FFFF):
            raise GenerationError(f"invalid code-point range: {self.start:#x}..{self.end:#x}")
    @property
    def count(self) -> int:
        return self.end - self.start + 1

@dataclass(frozen=True, order=True)
class AliasMapping:
    spelling: str
    canonical: str

@dataclass(frozen=True)
class GeneratedData:
    id_start: tuple[CodePointRange, ...]
    id_continue: tuple[CodePointRange, ...]
    zs: tuple[CodePointRange, ...]
    general_category_aliases: tuple[AliasMapping, ...]
    script_aliases: tuple[AliasMapping, ...]
    nonbinary_property_aliases: tuple[AliasMapping, ...]
    binary_property_aliases: tuple[AliasMapping, ...]
    string_properties: tuple[str, ...]

def git_blob_sha1(data: bytes) -> str:
    return hashlib.sha1(f"blob {len(data)}\0".encode("ascii") + data, usedforsecurity=False).hexdigest()

def load_manifest(path: Path) -> dict:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise GenerationError(f"cannot read manifest {path}: {error}") from error
    if value.get("schema_version") != 1:
        raise GenerationError("unsupported manifest schema")
    if value.get("unicode_version") != UNICODE_VERSION:
        raise GenerationError("manifest Unicode version is not 17.0.0")
    if value.get("ecma262_snapshot") != ECMA262_SNAPSHOT:
        raise GenerationError("manifest ECMA-262 snapshot does not match the frozen envelope")
    inputs = value.get("inputs")
    if not isinstance(inputs, list):
        raise GenerationError("manifest inputs must be a list")
    ids = [entry.get("id") for entry in inputs if isinstance(entry, dict)]
    if len(ids) != len(inputs) or len(set(ids)) != len(ids):
        raise GenerationError("manifest input ids must be present and unique")
    if set(ids) != EXPECTED_INPUT_IDS:
        raise GenerationError("manifest input set does not match the frozen six-input envelope")
    for entry in inputs:
        input_id = entry["id"]
        source = entry.get("git_mirror") or entry.get("git_source")
        if not isinstance(source, dict):
            raise GenerationError(f"{input_id}: missing Git source identity")
        if source.get("blob_sha1") != EXPECTED_BLOB_SHA1[input_id]:
            raise GenerationError(f"{input_id}: manifest blob does not match frozen upstream identity")
        if source.get("size") != EXPECTED_BYTE_SIZE[input_id]:
            raise GenerationError(f"{input_id}: manifest byte size does not match frozen upstream identity")
        if input_id.startswith("unicode-"):
            if source.get("repository") != "unicode-org/unicodetools":
                raise GenerationError(f"{input_id}: unexpected Unicode Git mirror repository")
            if source.get("commit") != UNICODE_MIRROR_COMMIT:
                raise GenerationError(f"{input_id}: unexpected Unicode Git mirror commit")
            if f"/ucd/{UNICODE_VERSION}/" not in str(source.get("path", "")):
                raise GenerationError(f"{input_id}: Unicode mirror path is not version-pinned")
            if f"/Public/{UNICODE_VERSION}/" not in str(entry.get("official_url", "")):
                raise GenerationError(f"{input_id}: Unicode publication URL is not version-pinned")
        else:
            if source.get("repository") != "tc39/ecma262" or source.get("commit") != ECMA262_SNAPSHOT:
                raise GenerationError(f"{input_id}: unexpected ECMA source identity")
    return value

def read_verified_input(repo_root: Path, entry: dict) -> bytes:
    relative = entry.get("retained_path")
    if not isinstance(relative, str) or not relative:
        raise GenerationError(f"{entry.get('id')}: missing retained path")
    try:
        data = (repo_root / relative).read_bytes()
    except OSError as error:
        raise GenerationError(f"{entry.get('id')}: retained input unavailable") from error
    source = entry.get("git_mirror") or entry.get("git_source") or {}
    if len(data) != source.get("size"):
        raise GenerationError(f"{entry.get('id')}: byte length mismatch")
    actual_blob = git_blob_sha1(data)
    if actual_blob != source.get("blob_sha1"):
        raise GenerationError(f"{entry.get('id')}: Git blob mismatch")
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise GenerationError(f"{entry.get('id')}: source is not UTF-8") from error
    if "required_header" in entry:
        first_line = text.splitlines()[0] if text else ""
        if first_line != entry["required_header"]:
            raise GenerationError(f"{entry.get('id')}: Unicode header mismatch")
    if "required_table_id" in entry:
        match = _TABLE_ID_RE.search(text)
        if match is None or match.group(1) != entry["required_table_id"]:
            raise GenerationError(f"{entry.get('id')}: unexpected ECMA table identity")
    return data

def _validate_sorted_nonoverlap(name: str, ranges: Sequence[CodePointRange]) -> None:
    if not ranges:
        raise GenerationError(f"{name}: no ranges found")
    for previous, current in zip(ranges, ranges[1:]):
        if current.start <= previous.end:
            raise GenerationError(f"{name}: overlapping or unsorted ranges")

def parse_property_ranges(data: bytes, property_name: str) -> tuple[CodePointRange, ...]:
    text = data.decode("utf-8")
    ranges: list[CodePointRange] = []
    for line_number, raw_line in enumerate(text.splitlines(), start=1):
        content = raw_line.split("#", 1)[0].strip()
        if not content:
            continue
        match = _RANGE_RE.fullmatch(content)
        if match is None:
            if f"; {property_name}" in content or f";{property_name}" in content:
                raise GenerationError(f"{property_name}: malformed target record on line {line_number}")
            continue
        if match.group(3) != property_name:
            continue
        start = int(match.group(1), 16)
        end = int(match.group(2), 16) if match.group(2) else start
        ranges.append(CodePointRange(start, end))
    _validate_sorted_nonoverlap(property_name, ranges)
    return tuple(ranges)

def count_ranges(ranges: Iterable[CodePointRange]) -> int:
    return sum(item.count for item in ranges)

def _ranges_are_subset(subset: Sequence[CodePointRange], superset: Sequence[CodePointRange]) -> bool:
    sup_index = 0
    for wanted in subset:
        cursor = wanted.start
        while sup_index < len(superset) and superset[sup_index].end < cursor:
            sup_index += 1
        index = sup_index
        while cursor <= wanted.end:
            if index >= len(superset) or superset[index].start > cursor:
                return False
            cursor = superset[index].end + 1
            index += 1
    return True

def _record_alias(mapping: dict[str, str], spelling: str, canonical: str, family: str) -> None:
    existing = mapping.get(spelling)
    if existing is not None and existing != canonical:
        raise GenerationError(f"{family}: conflicting alias {spelling!r}")
    mapping[spelling] = canonical

def parse_property_value_aliases(data: bytes) -> tuple[tuple[AliasMapping, ...], tuple[AliasMapping, ...]]:
    gc: dict[str, str] = {}
    sc: dict[str, str] = {}
    for raw_line in data.decode("utf-8").splitlines():
        line = raw_line.split("#", 1)[0].strip()
        if not line:
            continue
        fields = [field.strip() for field in line.split(";")]
        if len(fields) < 3 or fields[0] not in {"gc", "sc"}:
            continue
        target = gc if fields[0] == "gc" else sc
        canonical = fields[2]
        if not canonical:
            raise GenerationError(f"{fields[0]}: empty canonical alias value")
        for spelling in fields[1:]:
            if spelling:
                _record_alias(target, spelling, canonical, fields[0])
    if not gc or not sc:
        raise GenerationError("PropertyValueAliases: missing gc or sc aliases")
    return (
        tuple(AliasMapping(key, gc[key]) for key in sorted(gc)),
        tuple(AliasMapping(key, sc[key]) for key in sorted(sc)),
    )

def parse_ecma_alias_table(data: bytes) -> tuple[AliasMapping, ...]:
    mappings: dict[str, str] = {}
    inherited_canonical: str | None = None
    rows = _ROW_RE.findall(data.decode("utf-8"))
    if not rows:
        raise GenerationError("ECMA property table contains no rows")
    for row in rows:
        tokens = [html.unescape(token) for token in _TICK_RE.findall(row)]
        if not tokens:
            continue
        if len(tokens) >= 2:
            spelling, canonical = tokens[0], tokens[-1]
            inherited_canonical = canonical
        elif inherited_canonical is not None:
            spelling, canonical = tokens[0], inherited_canonical
        else:
            raise GenerationError("ECMA alias row lacks canonical property context")
        _record_alias(mappings, spelling, canonical, "ECMA property table")
    return tuple(AliasMapping(key, mappings[key]) for key in sorted(mappings))

def parse_ecma_string_properties(data: bytes) -> tuple[str, ...]:
    names: list[str] = []
    for row in _ROW_RE.findall(data.decode("utf-8")):
        tokens = _TICK_RE.findall(row)
        if not tokens:
            continue
        if len(tokens) != 1:
            raise GenerationError("property-of-strings row must contain exactly one property name")
        names.append(tokens[0])
    if tuple(names) != EXPECTED_STRING_PROPERTIES:
        raise GenerationError("pinned ES property-of-strings table does not match frozen seven-property set")
    return tuple(names)

def build_data(manifest: dict, repo_root: Path) -> GeneratedData:
    by_id = {entry["id"]: read_verified_input(repo_root, entry) for entry in manifest["inputs"]}
    id_start = parse_property_ranges(by_id["unicode-derived-core-properties"], "ID_Start")
    id_continue = parse_property_ranges(by_id["unicode-derived-core-properties"], "ID_Continue")
    zs = parse_property_ranges(by_id["unicode-derived-general-category"], "Zs")
    if count_ranges(id_start) != EXPECTED_ID_START_COUNT:
        raise GenerationError("ID_Start aggregate count mismatch")
    if count_ranges(id_continue) != EXPECTED_ID_CONTINUE_COUNT:
        raise GenerationError("ID_Continue aggregate count mismatch")
    if count_ranges(zs) != EXPECTED_ZS_COUNT:
        raise GenerationError("Zs aggregate count mismatch")
    if not _ranges_are_subset(id_start, id_continue):
        raise GenerationError("ID_Start must be a subset of ID_Continue")
    gc, sc = parse_property_value_aliases(by_id["unicode-property-value-aliases"])
    nonbinary = parse_ecma_alias_table(by_id["ecma262-nonbinary-unicode-properties"])
    binary = parse_ecma_alias_table(by_id["ecma262-binary-unicode-properties"])
    strings = parse_ecma_string_properties(by_id["ecma262-binary-unicode-properties-of-strings"])
    return GeneratedData(id_start, id_continue, zs, gc, sc, nonbinary, binary, strings)

def _render_ranges(name: str, ranges: Sequence[CodePointRange]) -> list[str]:
    lines = [f"pub(super) const {name}: &[(u32, u32)] = &["]
    lines.extend(f"    (0x{item.start:04X}, 0x{item.end:04X})," for item in ranges)
    lines.append("];")
    return lines

def _render_aliases(name: str, aliases: Sequence[AliasMapping]) -> list[str]:
    lines = [f"pub(super) const {name}: &[(&str, &str)] = &["]
    for item in aliases:
        spelling = json.dumps(item.spelling)
        canonical = json.dumps(item.canonical)
        single_line = f"    ({spelling}, {canonical}),"
        if len(single_line) <= 60:
            lines.append(single_line)
        else:
            lines.extend(["    (", f"        {spelling},", f"        {canonical},", "    ),"])
    lines.append("];")
    return lines

def _render_strings(name: str, values: Sequence[str]) -> list[str]:
    lines = [f"pub(super) const {name}: &[&str] = &["]
    lines.extend(f"    {json.dumps(item)}," for item in values)
    lines.append("];")
    return lines

def render_rust(data: GeneratedData, manifest_bytes: bytes) -> bytes:
    manifest_sha256 = hashlib.sha256(manifest_bytes).hexdigest()
    lines = [
        "// @generated by tools/unicode/generate_ecmascript_unicode.py; do not edit.",
        f'// Unicode {UNICODE_VERSION}; ECMA-262 {ECMA262_SNAPSHOT}.',
        f'// upstream-manifest.json SHA-256: {manifest_sha256}',
        "",
        f'pub(super) const UNICODE_VERSION: &str = "{UNICODE_VERSION}";',
        f'pub(super) const ECMA262_SNAPSHOT: &str = "{ECMA262_SNAPSHOT}";',
        "",
    ]
    sections = [
        _render_ranges("ID_START_RANGES", data.id_start),
        _render_ranges("ID_CONTINUE_RANGES", data.id_continue),
        _render_ranges("SPACE_SEPARATOR_RANGES", data.zs),
        _render_aliases("GENERAL_CATEGORY_ALIASES", data.general_category_aliases),
        _render_aliases("SCRIPT_ALIASES", data.script_aliases),
        _render_aliases("NONBINARY_PROPERTY_ALIASES", data.nonbinary_property_aliases),
        _render_aliases("BINARY_PROPERTY_ALIASES", data.binary_property_aliases),
        _render_strings("BINARY_PROPERTIES_OF_STRINGS", data.string_properties),
    ]
    for section in sections:
        lines.extend(section)
        lines.append("")
    return ("\n".join(lines).rstrip() + "\n").encode("utf-8")

def generate(manifest_path: Path, repo_root: Path) -> bytes:
    manifest = load_manifest(manifest_path)
    return render_rust(build_data(manifest, repo_root), manifest_path.read_bytes())

def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--repo-root", type=Path)
    args = parser.parse_args(argv)
    script = Path(__file__).resolve()
    repo_root = args.repo_root.resolve() if args.repo_root else script.parents[2]
    manifest_path = repo_root / "tools/unicode/upstream-manifest.json"
    output_path = repo_root / "crates/frontend-analysis-core/src/ecmascript/unicode_generated.rs"
    try:
        generated = generate(manifest_path, repo_root)
        if args.check:
            try:
                current = output_path.read_bytes()
            except OSError as error:
                raise GenerationError(f"generated output unavailable: {output_path}") from error
            if current != generated:
                raise GenerationError("generated output is stale; rerun generator")
        else:
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_bytes(generated)
    except GenerationError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
