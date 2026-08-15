#!/usr/bin/env python3
"""Independently verify generated ECMAScript Unicode data against retained sources."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

EXPECTED_STRINGS = (
    "Basic_Emoji",
    "Emoji_Keycap_Sequence",
    "RGI_Emoji_Modifier_Sequence",
    "RGI_Emoji_Flag_Sequence",
    "RGI_Emoji_Tag_Sequence",
    "RGI_Emoji_ZWJ_Sequence",
    "RGI_Emoji",
)

class VerificationError(RuntimeError):
    pass

def source_membership(path: Path, property_name: str) -> set[int]:
    values: set[int] = set()
    previous_end = -1
    for line_number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        body = raw.partition("#")[0].strip()
        if not body or ";" not in body:
            continue
        left, right = [piece.strip() for piece in body.split(";", 1)]
        if right != property_name:
            continue
        if ".." in left:
            first, last = left.split("..", 1)
        else:
            first = last = left
        try:
            start, end = int(first, 16), int(last, 16)
        except ValueError as error:
            raise VerificationError(f"{path}:{line_number}: malformed code point") from error
        if start <= previous_end:
            raise VerificationError(f"{path}:{line_number}: unsorted or overlapping source range")
        previous_end = end
        values.update(range(start, end + 1))
    if not values:
        raise VerificationError(f"{property_name}: no source values")
    return values

def source_value_aliases(path: Path, family: str) -> dict[str, str]:
    result: dict[str, str] = {}
    for line_number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        body = raw.partition("#")[0].strip()
        if not body:
            continue
        fields = [item.strip() for item in body.split(";")]
        if len(fields) < 3 or fields[0] != family:
            continue
        canonical = fields[2]
        for spelling in fields[1:]:
            if not spelling:
                continue
            old = result.setdefault(spelling, canonical)
            if old != canonical:
                raise VerificationError(f"{path}:{line_number}: conflicting {family} alias")
    return result

def source_ecma_aliases(path: Path) -> dict[str, str]:
    text = path.read_text(encoding="utf-8")
    result: dict[str, str] = {}
    canonical: str | None = None
    for row in re.findall(r"<tr>(.*?)</tr>", text, flags=re.S):
        cells = re.findall(r"<td(?:\s[^>]*)?>(.*?)</td>", row, flags=re.S)
        if not cells:
            continue
        first = re.findall(r"`([^`]+)`", cells[0])
        if not first:
            continue
        spelling = first[0]
        if len(cells) >= 2:
            second = re.findall(r"`([^`]+)`", cells[1])
            if not second:
                raise VerificationError(f"{path}: canonical cell lacks property name")
            canonical = second[-1]
        if canonical is None:
            raise VerificationError(f"{path}: alias lacks canonical context")
        old = result.setdefault(spelling, canonical)
        if old != canonical:
            raise VerificationError(f"{path}: conflicting ECMA alias")
    return result

def source_string_properties(path: Path) -> tuple[str, ...]:
    text = path.read_text(encoding="utf-8")
    result: list[str] = []
    for row in re.findall(r"<tr>(.*?)</tr>", text, flags=re.S):
        names = re.findall(r"`([^`]+)`", row)
        if names:
            if len(names) != 1:
                raise VerificationError(f"{path}: string-property row has multiple names")
            result.append(names[0])
    return tuple(result)

def block(text: str, name: str) -> str:
    match = re.search(rf"\b{name}\b[^=]*=\s*&\[(.*?)\];", text, flags=re.S)
    if match is None:
        raise VerificationError(f"generated constant missing: {name}")
    return match.group(1)

def generated_membership(text: str, name: str) -> set[int]:
    values: set[int] = set()
    for first, last in re.findall(r"\(0x([0-9A-F]+),\s*0x([0-9A-F]+)\)", block(text, name)):
        values.update(range(int(first, 16), int(last, 16) + 1))
    return values

def generated_aliases(text: str, name: str) -> dict[str, str]:
    body = block(text, name)
    pattern = re.compile(
        r'\(\s*("(?:\\.|[^"\\])*")\s*,\s*'
        r'("(?:\\.|[^"\\])*")\s*,?\s*\)',
        flags=re.S,
    )
    pairs: list[tuple[str, str]] = []
    position = 0
    for match in pattern.finditer(body):
        if re.fullmatch(r"[\s,]*", body[position : match.start()]) is None:
            raise VerificationError(f"generated alias table has unparsed syntax: {name}")
        pairs.append((json.loads(match.group(1)), json.loads(match.group(2))))
        position = match.end()
    if re.fullmatch(r"[\s,]*", body[position:]) is None:
        raise VerificationError(f"generated alias table has unparsed syntax: {name}")
    if not pairs:
        raise VerificationError(f"generated alias table is empty: {name}")
    result = dict(pairs)
    if len(result) != len(pairs):
        raise VerificationError(f"generated alias table has duplicate spellings: {name}")
    return result

def generated_strings(text: str, name: str) -> tuple[str, ...]:
    return tuple(re.findall(r'^\s*"([^"]+)",\s*$', block(text, name), flags=re.M))

def require_equal(label: str, expected: object, actual: object) -> None:
    if expected != actual:
        if isinstance(expected, set) and isinstance(actual, set):
            missing = sorted(expected - actual)[:8]
            extra = sorted(actual - expected)[:8]
            raise VerificationError(f"{label}: mismatch missing={missing} extra={extra}")
        raise VerificationError(f"{label}: mismatch")

def main() -> int:
    root = Path(__file__).resolve().parents[2]
    generated_path = root / "crates/frontend-analysis-core/src/ecmascript/unicode_generated.rs"
    generated = generated_path.read_text(encoding="utf-8")
    dcp = root / "tools/unicode/inputs/DerivedCoreProperties.txt"
    dgc = root / "tools/unicode/inputs/extracted/DerivedGeneralCategory.txt"
    pva = root / "tools/unicode/inputs/PropertyValueAliases.txt"
    ecma = root / "tools/unicode/inputs/ecma262"

    id_start = source_membership(dcp, "ID_Start")
    id_continue = source_membership(dcp, "ID_Continue")
    zs = source_membership(dgc, "Zs")
    require_equal("ID_Start", id_start, generated_membership(generated, "ID_START_RANGES"))
    require_equal("ID_Continue", id_continue, generated_membership(generated, "ID_CONTINUE_RANGES"))
    require_equal("Zs", zs, generated_membership(generated, "SPACE_SEPARATOR_RANGES"))
    require_equal("gc aliases", source_value_aliases(pva, "gc"), generated_aliases(generated, "GENERAL_CATEGORY_ALIASES"))
    require_equal("sc aliases", source_value_aliases(pva, "sc"), generated_aliases(generated, "SCRIPT_ALIASES"))
    require_equal(
        "non-binary ECMA aliases",
        source_ecma_aliases(ecma / "table-nonbinary-unicode-properties.html"),
        generated_aliases(generated, "NONBINARY_PROPERTY_ALIASES"),
    )
    require_equal(
        "binary ECMA aliases",
        source_ecma_aliases(ecma / "table-binary-unicode-properties.html"),
        generated_aliases(generated, "BINARY_PROPERTY_ALIASES"),
    )
    strings = source_string_properties(ecma / "table-binary-unicode-properties-of-strings.html")
    require_equal("binary properties of strings", strings, generated_strings(generated, "BINARY_PROPERTIES_OF_STRINGS"))
    require_equal("seven-property frozen string set", EXPECTED_STRINGS, strings)

    if 0x24 in id_start or 0x5F in id_start:
        raise VerificationError("ECMAScript grammar additions leaked into Unicode ID_Start")
    if 0x24 in id_continue:
        raise VerificationError("ECMAScript '$' leaked into Unicode ID_Continue")
    if 0x5F not in id_continue:
        raise VerificationError("Unicode ID_Continue unexpectedly excludes underscore")
    if not id_start <= id_continue:
        raise VerificationError("ID_Start is not a subset of ID_Continue")

    print(f"ID_Start={len(id_start)} ID_Continue={len(id_continue)} Zs={len(zs)}")
    print(f"gc_aliases={len(source_value_aliases(pva, 'gc'))} sc_aliases={len(source_value_aliases(pva, 'sc'))}")
    print(f"nonbinary_aliases={len(source_ecma_aliases(ecma / 'table-nonbinary-unicode-properties.html'))}")
    print(f"binary_aliases={len(source_ecma_aliases(ecma / 'table-binary-unicode-properties.html'))}")
    print(f"string_properties={len(strings)}")
    print("complete semantic equality: PASS")
    return 0

if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except VerificationError as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
