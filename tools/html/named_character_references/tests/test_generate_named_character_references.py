from __future__ import annotations

import ast
import copy
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
TOOL_ROOT = ROOT / "tools/html/named_character_references"
GENERATOR = TOOL_ROOT / "generate_named_character_references.py"
VERIFIER = TOOL_ROOT / "verify_generated_named_character_references.py"
ENTITIES = TOOL_ROOT / "inputs/entities.json"
LICENSE = TOOL_ROOT / "WHATWG-LICENSE.txt"
MANIFEST = TOOL_ROOT / "upstream-manifest.json"
GENERATED = (
    ROOT
    / "crates/frontend-analysis-core/src/html/tokenizer/"
    "named_character_references_generated.rs"
)
TOKENIZER_MOD = ROOT / "crates/frontend-analysis-core/src/html/tokenizer/mod.rs"
CRATE_SRC = ROOT / "crates/frontend-analysis-core/src"
TABLE_MARKER = "// BEGIN NAMED CHARACTER REFERENCES"

# The guard below is a bounded, repository-local structural/textual check, not
# a Rust parser. It is tolerant of whitespace, attribute formatting, and
# quoting variation so that obvious equivalent spellings of a bypass are
# rejected rather than only the one literal form.
MOD_DECLARATION = re.compile(
    r"(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;"
)
INCLUDE_MACRO = re.compile(r"include\s*!", re.S)
ATTRIBUTE_START = re.compile(r"#!?\[")
# A `path = "..."` name-value attribute, on whitespace-collapsed text, with or
# without a raw-string literal.
PATH_VALUE = re.compile(r'path=r?#*"([^"]*)"#*')
OPENERS = "([{"
CLOSERS = ")]}"
# One generated row: an ASCII identifier key mapped to a decoded string.
GENERATED_ROW = re.compile(r"\(\s*\"([A-Za-z][A-Za-z0-9]*;?)\"\s*,\s*\"[^\"]*\"\s*\)")
# How much of the real table a single non-generated file may mention in row
# form before it is treated as a second copy. The generated data's own Rust
# tests legitimately pin a handful of challenge cells; nothing else in the
# crate should come close, and a table copy carries orders of magnitude more.
MAX_INCIDENTAL_GENERATED_ROWS = 32


def crate_rust_sources() -> list[Path]:
    return sorted(CRATE_SRC.rglob("*.rs"))


def rust_spans(text: str) -> tuple[list[tuple[int, int]], list[tuple[int, int]]]:
    """Comment spans and string-literal spans in one Rust source file.

    A deliberately small lexical pass, not a Rust parser: it recognizes line
    comments, nested block comments, ordinary and raw string literals, and
    character literals — exactly the forms this repository uses — so that the
    checks below can tell a semantic construct from a mere textual mention of
    one. Anything it does not recognize is left as ordinary code, which is the
    conservative direction: the guard may look at too much, never too little.
    """
    comments: list[tuple[int, int]] = []
    strings: list[tuple[int, int]] = []
    index = 0
    length = len(text)
    while index < length:
        character = text[index]
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = length if end == -1 else end
            comments.append((index, end))
            index = end
            continue
        if text.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < length and depth:
                if text.startswith("/*", end):
                    depth += 1
                    end += 2
                elif text.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            comments.append((index, end))
            index = end
            continue
        if character == "r":
            hashes = index + 1
            while hashes < length and text[hashes] == "#":
                hashes += 1
            if hashes < length and text[hashes] == '"':
                terminator = '"' + "#" * (hashes - index - 1)
                end = text.find(terminator, hashes + 1)
                end = length if end == -1 else end + len(terminator)
                strings.append((index, end))
                index = end
                continue
        if character == '"':
            end = index + 1
            while end < length:
                if text[end] == "\\":
                    end += 2
                    continue
                if text[end] == '"':
                    end += 1
                    break
                end += 1
            strings.append((index, end))
            index = end
            continue
        if character == "'":
            # A character literal, including `'"'` and `'\''`; anything else
            # starting with `'` is a lifetime and is ordinary code.
            if text.startswith("'\\", index):
                end = index + 2
                while end < length and text[end] != "'":
                    end += 1
                index = end + 1
                continue
            if index + 2 < length and text[index + 2] == "'":
                index += 3
                continue
        index += 1
    return comments, strings


def blank_spans(text: str, spans: list[tuple[int, int]]) -> str:
    """Replaces `spans` with spaces, preserving newlines and every offset."""
    characters = list(text)
    for start, end in spans:
        for position in range(start, min(end, len(characters))):
            if characters[position] != "\n":
                characters[position] = " "
    return "".join(characters)


def scannable_source(text: str) -> str:
    """Rust source with comments blanked and string literals intact.

    Comments go because a commented-out construct is a mention, not code.
    Strings stay because the checks below need to read a `#[path]` value and an
    `include!` argument; a match that merely *starts* inside a string literal
    is filtered separately.
    """
    comments, _ = rust_spans(text)
    return blank_spans(text, comments)


def within(position: int, spans: list[tuple[int, int]]) -> bool:
    return any(start <= position < end for start, end in spans)


def span_mask(text: str, spans: list[tuple[int, int]]) -> bytearray:
    """A per-character flag for `spans`, so membership is a constant-time test."""
    mask = bytearray(len(text))
    for start, end in spans:
        for position in range(start, min(end, len(text))):
            mask[position] = 1
    return mask


def balanced_end(text: str, start: int, string_positions: bytearray) -> int:
    """End offset of the delimiter group opening at `start`.

    Counts `(`, `[` and `{` against their partners, ignoring anything inside a
    string literal so a delimiter in a path or message cannot unbalance the
    scan. This is what replaces a fixed character-distance window: a construct
    is inspected in full, however long it happens to be.
    """
    depth = 0
    index = start
    length = len(text)
    while index < length:
        if string_positions[index]:
            index += 1
            continue
        character = text[index]
        if character in OPENERS:
            depth += 1
        elif character in CLOSERS:
            depth -= 1
            if depth <= 0:
                return index + 1
        index += 1
    return length


def include_argument_regions(scan: str, string_positions: bytearray) -> list[str]:
    """The complete argument region of every `include!` invocation in `scan`.

    `scan` has comments blanked, so intervening comments are whitespace and an
    invocation mentioned only in a comment is not here at all. An invocation
    whose `include` sits inside a string literal is a mention too, and is
    skipped. Everything else is returned whole — no distance limit.
    """
    regions: list[str] = []
    length = len(scan)
    for match in INCLUDE_MACRO.finditer(scan):
        if string_positions[match.start()]:
            continue
        index = match.end()
        while index < length and scan[index].isspace():
            index += 1
        if index >= length or scan[index] not in OPENERS:
            continue
        regions.append(scan[index : balanced_end(scan, index, string_positions)])
    return regions


def rust_attributes(scan: str, string_positions: bytearray) -> list[str]:
    """Every `#[...]` / `#![...]` attribute in `scan`, each one complete.

    Balanced to its own closing bracket, so an attribute spanning several lines
    is one entry. Attributes blanked as comments or sitting inside a string
    literal are mentions and never appear here.
    """
    attributes: list[str] = []
    for match in ATTRIBUTE_START.finditer(scan):
        if string_positions[match.start()]:
            continue
        bracket = match.end() - 1
        attributes.append(scan[match.start() : balanced_end(scan, bracket, string_positions)])
    return attributes


def attribute_attached_paths(attribute: str) -> list[str]:
    """Every `path = "..."` value this attribute attaches to its item.

    A direct `#[path = "..."]` attaches one, and `cfg_attr` attaches whatever
    sits in its attribute positions — including, recursively, another
    `cfg_attr`. Both reach the same second module authority, so both count.
    `cfg_attr` attaching anything else attaches no path and is left alone: the
    theorem is semantic path indirection, not that `cfg_attr` is forbidden.
    """
    text = "".join(attribute.split())
    if not text.startswith("#") or "[" not in text or not text.endswith("]"):
        return []
    return _attached_paths(text[text.index("[") + 1 : -1])


def _attached_paths(attribute: str) -> list[str]:
    name, arguments = _split_call(attribute)
    if name == "cfg_attr" and arguments is not None:
        return [
            alias
            for part in _split_arguments(arguments)[1:]
            for alias in _attached_paths(part)
        ]
    match = PATH_VALUE.fullmatch(attribute)
    return [match.group(1)] if match else []


def attribute_gates_compilation(attribute: str) -> bool:
    """True when `attribute` can remove the item it decorates from a build.

    Both `#[cfg(...)]` and any `#[cfg_attr(..., cfg(...))]` do, and
    `cfg_attr` nests, so this walks the attached-attribute positions rather
    than matching one spelling. `#[cfg_attr(pred, allow(...))]` attaches no
    `cfg` and therefore gates nothing. Whitespace is removed first, so
    formatting variants are all the same input here.
    """
    text = "".join(attribute.split())
    if not text.startswith("#") or "[" not in text or not text.endswith("]"):
        return False
    return _attaches_cfg(text[text.index("[") + 1 : -1])


def _attaches_cfg(attribute: str) -> bool:
    name, arguments = _split_call(attribute)
    if name == "cfg":
        return True
    if name == "cfg_attr" and arguments is not None:
        # The first argument is the predicate; every later one is an
        # attribute that `cfg_attr` attaches when that predicate holds.
        return any(_attaches_cfg(part) for part in _split_arguments(arguments)[1:])
    return False


def _split_call(attribute: str) -> tuple[str, str | None]:
    if "(" not in attribute or not attribute.endswith(")"):
        return attribute, None
    open_paren = attribute.index("(")
    return attribute[:open_paren], attribute[open_paren + 1 : -1]


def _split_arguments(arguments: str) -> list[str]:
    """Splits an attribute's arguments at top-level commas.

    String literals are stepped over, so a comma or parenthesis inside a path
    or feature name does not split or unbalance the arguments.
    """
    parts: list[str] = []
    depth = 0
    current: list[str] = []
    in_string = False
    escaped = False
    for character in arguments:
        if in_string:
            current.append(character)
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                in_string = False
            continue
        if character == '"':
            in_string = True
            current.append(character)
            continue
        if character == "(":
            depth += 1
        elif character == ")":
            depth -= 1
        if character == "," and depth == 0:
            parts.append("".join(current))
            current = []
            continue
        current.append(character)
    parts.append("".join(current))
    return [part for part in parts if part]


def generated_identifier_names() -> frozenset[str]:
    """The real identifier keys carried by the generated table."""
    return frozenset(GENERATED_ROW.findall(GENERATED.read_text(encoding="utf-8")))


def generated_rows_in(text: str, names: frozenset[str]) -> set[str]:
    """Real generated identifiers appearing in row form in `text`.

    Keyed on the actual generated identifiers rather than on the marker
    comment, so stripping the marker from a copy does not hide it.
    """
    return {name for name in GENERATED_ROW.findall(text) if name in names}


def rust_module_declarations(path: Path) -> list[tuple[str, list[str]]]:
    """Every `mod <name>;` declaration in `path`, with its own attributes.

    Returns one entry per declaration rather than a mapping, so a duplicate
    declaration of the same module is visible instead of silently collapsing.
    Comments are blanked first, so a commented-out declaration or attribute is
    not mistaken for one. An attribute spanning several lines is joined until
    its brackets balance, so splitting one across lines cannot hide it from
    the declaration it decorates.
    """
    declarations: list[tuple[str, list[str]]] = []
    attributes: list[str] = []
    pending = ""
    depth = 0
    for raw in scannable_source(path.read_text(encoding="utf-8")).splitlines():
        line = raw.strip()
        if pending:
            pending = f"{pending} {line}"
            depth += line.count("[") - line.count("]")
            if depth <= 0:
                attributes.append(pending)
                pending = ""
                depth = 0
            continue
        if not line:
            continue
        if line.startswith("#"):
            depth = line.count("[") - line.count("]")
            if depth > 0:
                pending = line
            else:
                attributes.append(line)
            continue
        match = MOD_DECLARATION.fullmatch(line)
        if match:
            declarations.append((match.group(1), attributes))
        attributes = []
    return declarations


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


generator = load_module("named_character_reference_generator", GENERATOR)
verifier = load_module("named_character_reference_verifier", VERIFIER)


class NamedCharacterReferenceDataTests(unittest.TestCase):
    maxDiff = 2_000

    def source_object(self) -> dict[str, object]:
        return json.loads(ENTITIES.read_text(encoding="utf-8"))

    def manifest_object(self) -> dict[str, object]:
        return json.loads(MANIFEST.read_text(encoding="utf-8"))

    def semantic_bytes(self, value: object) -> bytes:
        return json.dumps(value, ensure_ascii=True, separators=(",", ":")).encode("utf-8")

    def source_semantics(self):
        return verifier.parse_source(ENTITIES.read_bytes())

    def generated_lines(self) -> list[str]:
        return GENERATED.read_text(encoding="utf-8").splitlines()

    def table_row_start(self, lines: list[str]) -> int:
        declaration = "pub(super) const NAMED_CHARACTER_REFERENCES: &[(&str, &str)] = &["
        return lines.index(declaration) + 1

    def table_row_end(self, lines: list[str]) -> int:
        return lines.index("// END NAMED CHARACTER REFERENCES") - 1

    def encoded_lines(self, lines: list[str]) -> bytes:
        return ("\n".join(lines) + "\n").encode("utf-8")

    def verify_generated_mutation(self, data: bytes) -> None:
        parsed = verifier.parse_generated_rust(data)
        verifier.verify_generated_semantics(
            self.source_semantics(),
            parsed,
            hashlib.sha256(MANIFEST.read_bytes()).hexdigest(),
        )

    def temporary_root(
        self, temporary: str, dataset: bytes, manifest: bytes | None = None
    ) -> Path:
        root = Path(temporary)
        dataset_path = root / generator.DATASET_PATH
        dataset_path.parent.mkdir(parents=True)
        dataset_path.write_bytes(dataset)
        license_path = root / generator.LICENSE_PATH
        license_path.parent.mkdir(parents=True, exist_ok=True)
        license_path.write_bytes(LICENSE.read_bytes())
        manifest_path = root / generator.MANIFEST_PATH
        manifest_path.parent.mkdir(parents=True, exist_ok=True)
        manifest_path.write_bytes(MANIFEST.read_bytes() if manifest is None else manifest)
        output = root / generator.OUTPUT_PATH
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(GENERATED.read_bytes())
        return root

    def test_retained_evidence_matches_both_independent_frozen_envelopes(self) -> None:
        dataset, license_bytes = generator.load_frozen_evidence(ROOT)
        self.assertEqual(len(dataset), 145_897)
        self.assertEqual(len(license_bytes), 16_315)
        verifier._assert_frozen(
            dataset,
            label="entities.json",
            expected_size=verifier.PINNED_ENTITIES_SIZE,
            expected_sha256=verifier.PINNED_ENTITIES_SHA256,
            expected_blob=verifier.PINNED_ENTITIES_BLOB,
        )
        verifier._assert_frozen(
            license_bytes,
            label="WHATWG LICENSE",
            expected_size=verifier.PINNED_LICENSE_SIZE,
            expected_sha256=verifier.PINNED_LICENSE_SHA256,
            expected_blob=verifier.PINNED_LICENSE_BLOB,
        )

    def test_manifest_agrees_with_both_independent_frozen_envelopes(self) -> None:
        generator.parse_manifest_bytes(MANIFEST.read_bytes())
        verifier.parse_manifest(MANIFEST.read_bytes())

    def test_manifest_duplicate_member_fails_closed_in_both_parsers(self) -> None:
        corrupted = MANIFEST.read_bytes().replace(
            b'  "schema_version": 1,',
            b'  "schema_version": 1,\n  "schema_version": 1,',
            1,
        )
        with self.assertRaisesRegex(generator.GenerationError, "duplicate JSON member"):
            generator.parse_manifest_bytes(corrupted)
        with self.assertRaisesRegex(verifier.VerificationError, "duplicate JSON member"):
            verifier.parse_manifest(corrupted)

    def test_wrong_whatwg_snapshot_fails_at_manifest_semantics(self) -> None:
        manifest = self.manifest_object()
        manifest["whatwg_html"]["commit"] = "0" * 40
        data = self.semantic_bytes(manifest)
        with self.assertRaisesRegex(generator.GenerationError, "WHATWG snapshot"):
            generator.parse_manifest_bytes(data)
        with self.assertRaisesRegex(verifier.VerificationError, "WHATWG snapshot"):
            verifier.parse_manifest(data)

    def test_manifest_byte_sizes_require_genuine_json_integers(self) -> None:
        manifest = self.manifest_object()
        manifest["dataset"]["byte_size"] = float(manifest["dataset"]["byte_size"])
        data = self.semantic_bytes(manifest)
        with self.assertRaisesRegex(generator.GenerationError, "genuine JSON integer"):
            generator.parse_manifest_bytes(data)
        with self.assertRaisesRegex(verifier.VerificationError, "genuine JSON integer"):
            verifier.parse_manifest(data)

    def test_wrong_retained_byte_size_fails_identity_layer(self) -> None:
        with self.assertRaisesRegex(generator.GenerationError, "byte size"):
            generator.validate_frozen_bytes(
                ENTITIES.read_bytes() + b"\n",
                label="entities.json",
                byte_size=generator.DATASET_BYTE_SIZE,
                sha256=generator.DATASET_SHA256,
                git_blob=generator.DATASET_GIT_BLOB_SHA1,
            )

    def test_wrong_retained_sha256_fails_identity_layer(self) -> None:
        corrupted = bytearray(ENTITIES.read_bytes())
        corrupted[10] ^= 1
        with self.assertRaisesRegex(generator.GenerationError, "SHA-256"):
            generator.validate_frozen_bytes(
                bytes(corrupted),
                label="entities.json",
                byte_size=generator.DATASET_BYTE_SIZE,
                sha256=generator.DATASET_SHA256,
                git_blob=generator.DATASET_GIT_BLOB_SHA1,
            )

    def test_dataset_and_manifest_cannot_jointly_redefine_generator_identity(self) -> None:
        corrupted = bytearray(ENTITIES.read_bytes())
        corrupted[10] ^= 1
        manifest = self.manifest_object()
        manifest["dataset"]["sha256"] = hashlib.sha256(corrupted).hexdigest()
        manifest["dataset"]["local_blob_sha1"] = generator.git_blob_sha1(corrupted)
        with tempfile.TemporaryDirectory() as temporary:
            root = self.temporary_root(temporary, bytes(corrupted), self.semantic_bytes(manifest))
            with self.assertRaisesRegex(generator.GenerationError, "frozen SHA-256"):
                generator.generate(root)

    def test_dataset_and_manifest_cannot_jointly_redefine_verifier_identity(self) -> None:
        corrupted = bytearray(ENTITIES.read_bytes())
        corrupted[10] ^= 1
        manifest = self.manifest_object()
        manifest["dataset"]["sha256"] = hashlib.sha256(corrupted).hexdigest()
        manifest["dataset"]["local_blob_sha1"] = generator.git_blob_sha1(corrupted)
        with tempfile.TemporaryDirectory() as temporary:
            root = self.temporary_root(temporary, bytes(corrupted), self.semantic_bytes(manifest))
            with self.assertRaisesRegex(verifier.VerificationError, "independently frozen SHA-256"):
                verifier.verify_root(root)

    def test_omitted_entity_reaches_semantic_entry_count_validation(self) -> None:
        source = self.source_object()
        source.pop(next(iter(source)))
        with self.assertRaisesRegex(generator.GenerationError, "entry count mismatch"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_extra_entity_reaches_semantic_entry_count_validation(self) -> None:
        source = self.source_object()
        source["&CodexGateOnly;"] = {"codepoints": [65], "characters": "A"}
        with self.assertRaisesRegex(generator.GenerationError, "entry count mismatch"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_duplicate_top_level_entity_fails_strict_json_semantics(self) -> None:
        data = (
            b'{"&amp;":{"codepoints":[38],"characters":"&"},'
            b'"&amp;":{"codepoints":[38],"characters":"&"}}'
        )
        with self.assertRaisesRegex(generator.GenerationError, "duplicate JSON member '&amp;'"):
            generator.parse_and_validate_entities(data)
        with self.assertRaisesRegex(verifier.VerificationError, "duplicate JSON member '&amp;'"):
            verifier.parse_source(data)

    def test_duplicate_nested_field_fails_strict_json_semantics(self) -> None:
        data = b'{"&amp;":{"codepoints":[38],"codepoints":[38],"characters":"&"}}'
        with self.assertRaisesRegex(generator.GenerationError, "duplicate JSON member 'codepoints'"):
            generator.parse_and_validate_entities(data)
        with self.assertRaisesRegex(verifier.VerificationError, "duplicate JSON member 'codepoints'"):
            verifier.parse_source(data)

    def test_non_finite_json_extensions_fail_closed_in_both_parsers(self) -> None:
        for token in (b"NaN", b"Infinity", b"-Infinity"):
            with self.subTest(token=token):
                data = b'{"&amp;":{"codepoints":[' + token + b'],"characters":"&"}}'
                with self.assertRaisesRegex(generator.GenerationError, "non-finite JSON"):
                    generator.parse_and_validate_entities(data)
                with self.assertRaisesRegex(verifier.VerificationError, "non-finite JSON"):
                    verifier.parse_source(data)

    def test_malformed_json_fails_closed(self) -> None:
        with self.assertRaisesRegex(generator.GenerationError, "malformed JSON"):
            generator.parse_and_validate_entities(b"{")

    def test_unexpected_json_root_type_fails_closed(self) -> None:
        with self.assertRaisesRegex(generator.GenerationError, "expected JSON object"):
            generator.parse_and_validate_entities(b"[]")

    def test_mismatched_codepoints_and_characters_reaches_value_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"]["characters"] = "X"
        with self.assertRaisesRegex(generator.GenerationError, "codepoints and characters mismatch"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_boolean_codepoint_reaches_strict_scalar_type_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"]["codepoints"] = [True]
        with self.assertRaisesRegex(generator.GenerationError, "genuine JSON integer"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))
        with self.assertRaisesRegex(verifier.VerificationError, "non-integer codepoint"):
            verifier.parse_source(self.semantic_bytes(source))

    def test_negative_codepoint_reaches_scalar_range_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"] = {"codepoints": [-1], "characters": "&"}
        with self.assertRaisesRegex(generator.GenerationError, "outside Unicode range"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_codepoint_above_unicode_maximum_reaches_scalar_range_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"] = {"codepoints": [0x110000], "characters": "&"}
        with self.assertRaisesRegex(generator.GenerationError, "outside Unicode range"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_u10ffff_is_a_valid_scalar_boundary(self) -> None:
        entity = generator._validate_entity_record(
            "&Boundary;",
            {"codepoints": [0x10FFFF], "characters": chr(0x10FFFF)},
        )
        self.assertEqual(entity.codepoints, (0x10FFFF,))

    def test_u10ffff_mutation_reaches_authoritative_challenge_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"] = {"codepoints": [0x10FFFF], "characters": chr(0x10FFFF)}
        source["&amp"] = {"codepoints": [0x10FFFF], "characters": chr(0x10FFFF)}
        with self.assertRaisesRegex(generator.GenerationError, "challenge cell &amp;"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_surrogate_codepoint_reaches_scalar_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"] = {"codepoints": [0xD800], "characters": "\ud800"}
        with self.assertRaisesRegex(generator.GenerationError, "is a surrogate"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_unpaired_surrogate_characters_reaches_text_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"]["characters"] = "\ud800"
        with self.assertRaisesRegex(generator.GenerationError, "unpaired surrogate"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_three_codepoint_value_reaches_shape_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"] = {"codepoints": [65, 66, 67], "characters": "ABC"}
        with self.assertRaisesRegex(generator.GenerationError, "exactly one or two"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_record_missing_field_reaches_exact_schema_semantics(self) -> None:
        source = self.source_object()
        del source["&amp;"]["characters"]
        with self.assertRaisesRegex(generator.GenerationError, "object fields mismatch"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_record_additional_field_reaches_exact_schema_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"]["extra"] = None
        with self.assertRaisesRegex(generator.GenerationError, "object fields mismatch"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_float_codepoint_reaches_strict_scalar_type_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"]["codepoints"] = [38.0]
        with self.assertRaisesRegex(generator.GenerationError, "genuine JSON integer"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_missing_semicolon_counterpart_reaches_relationship_semantics(self) -> None:
        source = self.source_object()
        source.pop("&amp;")
        source["&CodexReplacement;"] = {"codepoints": [65], "characters": "A"}
        with self.assertRaisesRegex(generator.GenerationError, "missing semicolon counterpart"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_mismatched_semicolon_counterpart_reaches_relationship_semantics(self) -> None:
        source = self.source_object()
        source["&amp;"] = {"codepoints": [88], "characters": "X"}
        with self.assertRaisesRegex(generator.GenerationError, "counterpart meaning mismatch"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_malformed_entity_key_reaches_key_semantics(self) -> None:
        source = self.source_object()
        value = source.pop(next(iter(source)))
        source["&bad-name;"] = value
        with self.assertRaisesRegex(generator.GenerationError, "malformed named-reference spelling"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_non_ascii_entity_key_reaches_key_semantics(self) -> None:
        source = self.source_object()
        value = source.pop(next(iter(source)))
        source["&caf\u00e9;"] = value
        with self.assertRaisesRegex(generator.GenerationError, "non-ASCII spelling"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_double_leading_ampersand_reaches_key_semantics(self) -> None:
        source = self.source_object()
        value = source.pop(next(iter(source)))
        source["&&amp;"] = value
        with self.assertRaisesRegex(generator.GenerationError, "multiple leading ampersands"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_empty_entity_name_reaches_key_semantics(self) -> None:
        source = self.source_object()
        value = source.pop(next(iter(source)))
        source["&"] = value
        with self.assertRaisesRegex(generator.GenerationError, "malformed named-reference spelling"):
            generator.parse_and_validate_entities(self.semantic_bytes(source))

    def test_exact_names_and_duplicate_decoded_values_are_preserved(self) -> None:
        dataset = generator.parse_and_validate_entities(ENTITIES.read_bytes())
        names = {entry.generated_name for entry in dataset.entries}
        decoded = [entry.characters for entry in dataset.entries]
        self.assertEqual(len(names), 2_231)
        self.assertLess(len(set(decoded)), len(decoded))
        self.assertTrue({"AMP;", "amp;", "amp", "not", "not;"} <= names)
        self.assertNotEqual("AMP;", "amp;")
        self.assertNotEqual("amp", "amp;")

    def test_generation_is_independent_of_source_member_order(self) -> None:
        source = self.source_object()
        reversed_source = dict(reversed(list(source.items())))
        first = generator.parse_and_validate_entities(ENTITIES.read_bytes())
        second = generator.parse_and_validate_entities(self.semantic_bytes(reversed_source))
        manifest = MANIFEST.read_bytes()
        self.assertEqual(generator.render_rust(first, manifest), generator.render_rust(second, manifest))

    def test_derived_complete_data_envelope_is_exact(self) -> None:
        dataset = generator.parse_and_validate_entities(ENTITIES.read_bytes())
        self.assertEqual(len(dataset.entries), 2_231)
        self.assertEqual(dataset.semicolonless_count, 106)
        self.assertEqual(dataset.two_scalar_count, 93)
        self.assertEqual(dataset.maximum_key_byte_length, 32)
        self.assertEqual(dataset.maximum_keys, ("CounterClockwiseContourIntegral;",))

    def test_generated_output_is_byte_deterministic_and_frozen(self) -> None:
        first = generator.generate(ROOT)
        second = generator.generate(ROOT)
        self.assertEqual(first, second)
        self.assertEqual(first, GENERATED.read_bytes())
        self.assertEqual(
            hashlib.sha256(first).hexdigest(),
            "0fb26ff21c34626bad20db5e62ea6b2d5e71ed0d1725df6d6d03642ecbb76c69",
        )

    def test_generated_header_records_all_required_authority(self) -> None:
        text = GENERATED.read_text(encoding="utf-8")
        manifest_sha = hashlib.sha256(MANIFEST.read_bytes()).hexdigest()
        self.assertIn(generator.WHATWG_HTML_SNAPSHOT, text)
        self.assertIn(generator.DATASET_SHA256, text)
        self.assertIn(manifest_sha, text)

    def test_generated_missing_row_fails_complete_semantic_equality(self) -> None:
        lines = self.generated_lines()
        del lines[self.table_row_start(lines)]
        with self.assertRaisesRegex(verifier.VerificationError, "key-set mismatch"):
            self.verify_generated_mutation(self.encoded_lines(lines))

    def test_generated_altered_value_fails_complete_semantic_equality(self) -> None:
        lines = self.generated_lines()
        row = self.table_row_start(lines)
        lines[row] = lines[row].replace(r"\u{C6}", r"\u{C7}")
        with self.assertRaisesRegex(verifier.VerificationError, "decoded value mismatch"):
            self.verify_generated_mutation(self.encoded_lines(lines))

    def test_generated_duplicate_row_fails_parser(self) -> None:
        lines = self.generated_lines()
        row = self.table_row_start(lines)
        lines.insert(row + 1, lines[row])
        with self.assertRaisesRegex(verifier.VerificationError, "duplicate generated key"):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_reordered_rows_fail_canonical_sorting(self) -> None:
        lines = self.generated_lines()
        row = self.table_row_start(lines)
        lines[row], lines[row + 1] = lines[row + 1], lines[row]
        with self.assertRaisesRegex(verifier.VerificationError, "canonical ASCII key order"):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_unexpected_syntax_inside_region_fails_parser(self) -> None:
        lines = self.generated_lines()
        lines.insert(self.table_row_start(lines), "    unexpected!();")
        with self.assertRaisesRegex(verifier.VerificationError, "unexpected syntax inside"):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_extra_row_fails_complete_semantic_equality(self) -> None:
        lines = self.generated_lines()
        lines.insert(self.table_row_end(lines), r'    ("zzzzzzzzCodex;", "\u{41}"),')
        with self.assertRaisesRegex(verifier.VerificationError, "key-set mismatch"):
            self.verify_generated_mutation(self.encoded_lines(lines))

    def test_generated_metadata_mutation_fails_semantic_verification(self) -> None:
        lines = self.generated_lines()
        index = lines.index(
            "pub(super) const NAMED_CHARACTER_REFERENCE_ENTRY_COUNT: usize = 2231;"
        )
        lines[index] = "pub(super) const NAMED_CHARACTER_REFERENCE_ENTRY_COUNT: usize = 2230;"
        with self.assertRaisesRegex(verifier.VerificationError, "metadata differs"):
            self.verify_generated_mutation(self.encoded_lines(lines))

    def test_generated_missing_final_lf_fails_closed(self) -> None:
        with self.assertRaisesRegex(verifier.VerificationError, "final LF"):
            verifier.parse_generated_rust(GENERATED.read_bytes().removesuffix(b"\n"))

    def test_independent_complete_verifier_passes(self) -> None:
        source = verifier.verify_root(ROOT)
        self.assertEqual(len(source.values), 2_231)

    def test_generator_check_is_zero_mutation_and_passes(self) -> None:
        before = GENERATED.read_bytes()
        completed = subprocess.run(
            [sys.executable, str(GENERATOR), "--check"],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(GENERATED.read_bytes(), before)

    def test_verifier_cli_passes_complete_equality(self) -> None:
        completed = subprocess.run(
            [sys.executable, str(VERIFIER)],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("complete semantic equality: PASS", completed.stdout)

    def test_generator_and_verifier_have_no_network_or_external_database_imports(self) -> None:
        forbidden = {
            "html",
            "html.entities",
            "http",
            "http.client",
            "requests",
            "socket",
            "subprocess",
            "urllib",
            "urllib.request",
        }
        for path in (GENERATOR, VERIFIER):
            tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
            imports: set[str] = set()
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    imports.update(alias.name for alias in node.names)
                elif isinstance(node, ast.ImportFrom) and node.module:
                    imports.add(node.module)
            self.assertTrue(imports.isdisjoint(forbidden), f"{path}: {imports & forbidden}")
            non_standard = {
                name
                for name in imports
                if name.split(".", 1)[0] not in sys.stdlib_module_names
                and name != "__future__"
            }
            self.assertEqual(non_standard, set(), f"{path}: {sorted(non_standard)}")

    def test_verifier_does_not_import_generator_code_or_constants(self) -> None:
        tree = ast.parse(VERIFIER.read_text(encoding="utf-8"), filename=str(VERIFIER))
        imports = [node for node in ast.walk(tree) if isinstance(node, (ast.Import, ast.ImportFrom))]
        rendered = "\n".join(ast.unparse(node) for node in imports)
        self.assertNotIn("generate_named_character_references", rendered)
        self.assertNotIn("named_character_reference_generator", rendered)
        self.assertIn(verifier.PINNED_ENTITIES_SHA256, VERIFIER.read_text(encoding="utf-8"))
        self.assertIn(generator.DATASET_SHA256, GENERATOR.read_text(encoding="utf-8"))

    def test_gitattributes_preserve_unicode_rules_and_bound_html_evidence_exactly(self) -> None:
        text = (ROOT / ".gitattributes").read_text(encoding="utf-8")
        self.assertIn("tools/unicode/inputs/** -text -whitespace", text)
        self.assertIn(
            "tools/html/named_character_references/inputs/entities.json -text -whitespace",
            text,
        )
        self.assertIn(
            "tools/html/named_character_references/WHATWG-LICENSE.txt -text -whitespace",
            text,
        )
        self.assertNotIn("tools/html/named_character_references/** -text", text)

    def test_generated_data_wiring_matches_the_post_tc_s10_boundary(self) -> None:
        """The generated semantic table is the single production authority.

        #392 established the complete generated table behind a
        production-hard-zero gate, so the table itself was wired
        `#[cfg(test)]`-only and production tokenizer behavior was unchanged.
        TC-S10 is the separately approved first production consumer of that
        same generated table, so test-only wiring is no longer the durable
        invariant. The invariant that replaces it is deliberately no weaker,
        and rejects obvious equivalent spellings of each bypass:

        - the generated table is declared exactly once in the tokenizer
          module, under no attribute that can gate it out of a production
          build — `#[cfg(...)]` and `#[cfg_attr(..., cfg(...))]` alike;
        - the generated Rust data tests stay `#[cfg(test)]`-only;
        - no duplicate declaration of either module exists anywhere else in
          the crate;
        - no `include!` invocation anywhere in the crate reaches the generated
          data. The whole balanced argument region is inspected, so no amount
          of intervening comment or whitespace can carry the path out of view;
        - no attribute aliases the generated data through `path`, whether
          written directly or attached by `cfg_attr` — including recursively,
          and whatever the formatting;
        - no other crate file carries the generated rows, detected by the real
          identifiers themselves rather than by the marker comment, so
          stripping the marker from a copy does not hide it; and
        - a construct only counts when it is code. A commented-out `include!`
          or `#[path]` is a mention, not indirection, and is accepted.

        This is a bounded repository-local guard, not a Rust parser.
        """
        sources = crate_rust_sources()
        self.assertIn(GENERATED, sources)

        # 1. Module wiring, scanned crate-wide rather than in one file, so a
        #    duplicate declaration elsewhere cannot hide.
        declarations = [
            (path, name, attributes)
            for path in sources
            for name, attributes in rust_module_declarations(path)
        ]

        generated = [
            (path, attributes)
            for path, name, attributes in declarations
            if name == GENERATED.stem
        ]
        self.assertEqual(
            [path for path, _ in generated],
            [TOKENIZER_MOD],
            "the generated table is declared exactly once, in the tokenizer module",
        )
        self.assertFalse(
            [
                attribute
                for attribute in generated[0][1]
                if attribute_gates_compilation(attribute)
            ],
            "the generated table must be unconditionally production-visible",
        )

        data_tests = [
            (path, attributes)
            for path, name, attributes in declarations
            if name == "named_character_references_data_tests"
        ]
        self.assertEqual([path for path, _ in data_tests], [TOKENIZER_MOD])
        self.assertIn(
            "#[cfg(test)]",
            data_tests[0][1],
            "the generated Rust data tests remain test-only",
        )

        # 2/3. No *code* indirection reaches the generated data. Comments are
        #      blanked and constructs inside string literals are skipped, so a
        #      mere mention is not mistaken for indirection; the argument and
        #      path text themselves stay readable. Both constructs are scanned
        #      by balanced delimiters, never by a distance from where they
        #      start.
        names = generated_identifier_names()
        self.assertEqual(len(names), generator.EXPECTED_ENTRY_COUNT)
        for path in sources:
            raw = path.read_text(encoding="utf-8")
            comments, strings = rust_spans(raw)
            scan = blank_spans(raw, comments)
            string_positions = span_mask(raw, strings)

            for region in include_argument_regions(scan, string_positions):
                self.assertNotIn(
                    "named_character_reference",
                    region,
                    f"{path}: include! must not reach the generated data",
                )
            for attribute in rust_attributes(scan, string_positions):
                for alias in attribute_attached_paths(attribute):
                    self.assertNotIn(
                        "named_character_reference",
                        alias,
                        f"{path}: path must not alias the generated data",
                    )

            # 4. Exactly one file carries the generated rows. Commented-out
            #    rows do not count; real ones do.
            if path == GENERATED:
                continue
            carried = generated_rows_in(scan, names)
            self.assertLessEqual(
                len(carried),
                MAX_INCIDENTAL_GENERATED_ROWS,
                f"{path}: a second copy of the generated table",
            )

        self.assertIn(TABLE_MARKER, GENERATED.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
