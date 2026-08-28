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

# One bounded, repository-local structural scanner backs every check below.
# It is not a Rust parser: it knows comments, string literals, character
# literals, balanced delimiters, attributes, `mod` declarations, and the
# constant-string forms `include!` accepts here — exactly the syntax this guard
# reasons about, and nothing else. Every check shares this one lexical model so
# that no two of them can disagree about where a literal ends.
#
# Comments, string literals and character literals are all *non-structural*:
# their content is text, never delimiter depth. A `']'` written as a character
# literal must not close an enclosing attribute, exactly as a `"]"` written as
# a string must not.
OPENERS = "([{"
CLOSERS = ")]}"
# The single-character Rust escapes, mapped to the value each denotes. Used
# both to bound a character literal and to decode a string literal, so the two
# can never disagree about what an escape is.
SIMPLE_ESCAPE_VALUES = {
    "\\": "\\",
    "'": "'",
    '"': '"',
    "n": "\n",
    "r": "\r",
    "t": "\t",
    "0": "\0",
}
MOD_DECLARATION = re.compile(
    r"(?<![A-Za-z0-9_])(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;"
)
INCLUDE_MACRO = re.compile(r"(?<![A-Za-z0-9_])include\s*!")
ATTRIBUTE_START = re.compile(r"#!?\[")
PATH_NAME = re.compile(r"\s*path\s*=")
CONCAT_MACRO_NAMES = frozenset({"concat"})
# One generated row: an ASCII identifier key mapped to a decoded string.
GENERATED_ROW = re.compile(r"\(\s*\"([A-Za-z][A-Za-z0-9]*;?)\"\s*,\s*\"[^\"]*\"\s*\)")
# How much of the real table a single non-generated file may mention in row
# form before it is treated as a second copy. The generated data's own Rust
# tests legitimately pin a handful of challenge cells; nothing else in the
# crate should come close, and a table copy carries orders of magnitude more.
MAX_INCIDENTAL_GENERATED_ROWS = 32


def crate_rust_sources() -> list[Path]:
    return sorted(CRATE_SRC.rglob("*.rs"))


def lexical_spans(
    text: str,
) -> tuple[list[tuple[int, int]], list[tuple[int, int, str]]]:
    """Comment spans and literal spans in a fragment of Rust source.

    Recognizes line comments, nested block comments, ordinary and raw string
    literals with any hash count, and character literals — the forms this
    repository uses. Each literal is reported as `(start, end, kind)` with
    `kind` one of `"string"`, `"raw"` or `"character"`, so callers can both
    exclude every literal from structural scanning and still tell a string
    literal from a character literal when a *value* is required.

    Anything it does not recognize is left as ordinary code, the conservative
    direction: the guard may look at too much, never too little.
    """
    comments: list[tuple[int, int]] = []
    literals: list[tuple[int, int, str]] = []
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
        if character == "r" and not _is_identifier_continuation(text, index - 1):
            hashes = index + 1
            while hashes < length and text[hashes] == "#":
                hashes += 1
            if hashes < length and text[hashes] == '"':
                terminator = '"' + "#" * (hashes - index - 1)
                end = text.find(terminator, hashes + 1)
                end = length if end == -1 else end + len(terminator)
                literals.append((index, end, "raw"))
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
            literals.append((index, end, "string"))
            index = end
            continue
        if character == "'":
            end = _character_literal_end(text, index)
            if end is not None:
                literals.append((index, end, "character"))
                index = end
                continue
        index += 1
    return comments, literals


def _character_literal_end(text: str, index: int) -> int | None:
    """End offset of the character literal opening at `index`, else `None`.

    A character literal is one escape or exactly one character between two
    `'`, so `'\\x5d'`, `'\\u{5d}'`, `'\\''`, `'\\\\'` and `'"'` are all bounded
    correctly and none of their content is structural. `None` means the `'`
    opens something else — a lifetime or a loop label such as `'a`, `'a:` or
    `'static` — which stays ordinary code, matching how Rust itself resolves
    this same ambiguity.
    """
    length = len(text)
    if index + 1 >= length:
        return None
    if text[index + 1] == "\\":
        end = _escape_end(text, index + 1)
        if end is None:
            return None
    else:
        end = index + 2
    return end + 1 if end < length and text[end] == "'" else None


def _escape_end(text: str, position: int) -> int | None:
    """End offset of the escape sequence whose backslash is at `position`.

    `None` for any form that is not a Rust escape, so an unrecognized sequence
    never silently bounds a literal.
    """
    length = len(text)
    if position + 1 >= length:
        return None
    marker = text[position + 1]
    if marker == "x":
        end = position + 4
        return end if end <= length and _is_hexadecimal(text[position + 2 : end]) else None
    if marker == "u":
        if text[position + 2 : position + 3] != "{":
            return None
        close = text.find("}", position + 3)
        return None if close == -1 else close + 1
    return position + 2 if marker in SIMPLE_ESCAPE_VALUES else None


def _is_hexadecimal(digits: str) -> bool:
    return bool(digits) and all(digit in "0123456789abcdefABCDEF" for digit in digits)


def _is_identifier_continuation(text: str, position: int) -> bool:
    return 0 <= position < len(text) and (
        text[position].isalnum() or text[position] == "_"
    )


def decode_rust_string(content: str) -> str | None:
    """The compile-time value of an *ordinary* Rust string literal's content.

    Decoding is explicit and bounded to exactly the escapes Rust defines for a
    string literal: `\\\\`, `\\'`, `\\"`, `\\n`, `\\r`, `\\t`, `\\0`, `\\xNN` (ASCII
    only, as Rust requires in a string), `\\u{...}` (one to six hexadecimal
    digits, `_` separators allowed, a real Unicode scalar), and a backslash
    before a line break, which continues the line and drops the following
    leading whitespace.

    Python's generic `unicode-escape` codec is deliberately *not* used: its
    escape set is not Rust's, it accepts `\\uXXXX` without braces, it rejects
    nothing Rust rejects, and it decodes per byte rather than per scalar. A
    codec that disagrees with `rustc` about a path would answer confidently
    and wrongly.

    `None` for any sequence not in that set — an unknown value is unproven,
    never assumed harmless.
    """
    decoded: list[str] = []
    index = 0
    length = len(content)
    while index < length:
        character = content[index]
        if character != "\\":
            decoded.append(character)
            index += 1
            continue
        marker = content[index + 1 : index + 2]
        if marker in SIMPLE_ESCAPE_VALUES:
            decoded.append(SIMPLE_ESCAPE_VALUES[marker])
            index += 2
            continue
        if marker == "x":
            digits = content[index + 2 : index + 4]
            if len(digits) != 2 or not _is_hexadecimal(digits) or int(digits, 16) > 0x7F:
                return None
            decoded.append(chr(int(digits, 16)))
            index += 4
            continue
        if marker == "u":
            if content[index + 2 : index + 3] != "{":
                return None
            close = content.find("}", index + 3)
            if close == -1:
                return None
            digits = content[index + 3 : close].replace("_", "")
            if not digits or len(digits) > 6 or not _is_hexadecimal(digits):
                return None
            scalar = int(digits, 16)
            if scalar > 0x10FFFF or 0xD800 <= scalar <= 0xDFFF:
                return None
            decoded.append(chr(scalar))
            index = close + 1
            continue
        if marker in ("\n", "\r"):
            index += 1
            while index < length and content[index] in " \t\r\n":
                index += 1
            continue
        return None
    return "".join(decoded)


def blank_spans(text: str, spans: list[tuple[int, int]]) -> str:
    """Replaces `spans` with spaces, preserving newlines and every offset."""
    characters = list(text)
    for start, end in spans:
        for position in range(start, min(end, len(characters))):
            if characters[position] != "\n":
                characters[position] = " "
    return "".join(characters)


def span_mask(text: str, spans: list[tuple[int, int]]) -> bytearray:
    """A per-character flag for `spans`, so membership is a constant-time test."""
    mask = bytearray(len(text))
    for start, end in spans:
        for position in range(start, min(end, len(text))):
            mask[position] = 1
    return mask


class RustSource:
    """The one lexical/structural view every check in this guard shares.

    `code` is the source with comments blanked to spaces and every offset
    preserved, so a commented-out construct is simply absent. `in_literal`
    reports the literals that survive in it — string, raw string *and*
    character alike — so a construct merely quoted is skipped and, crucially,
    so that literal *content* can never control structural delimiter depth.

    Character literals are part of that same mask rather than merely stepped
    over: `']'` is one character of text, and a scanner that let it decrement
    a bracket depth would silently truncate the construct enclosing it.
    """

    def __init__(self, text: str) -> None:
        comments, literals = lexical_spans(text)
        self.code = blank_spans(text, comments)
        self._literals = span_mask(text, [(start, end) for start, end, _ in literals])

    def in_literal(self, position: int) -> bool:
        return bool(self._literals[position])

    def balanced_end(self, start: int) -> int:
        """End offset of the delimiter group opening at `start`.

        Counts `(`, `[` and `{` against their partners while ignoring every
        literal, so no bracket, quote or comma written inside a string *or a
        character literal* can close a group early. This is what replaces any
        fixed distance: a construct is always inspected in full, however long
        it is.
        """
        depth = 0
        index = start
        length = len(self.code)
        while index < length:
            if self.in_literal(index):
                index += 1
                continue
            character = self.code[index]
            if character in OPENERS:
                depth += 1
            elif character in CLOSERS:
                depth -= 1
                if depth <= 0:
                    return index + 1
            index += 1
        return length

    def attributes(self) -> list[tuple[int, int, str]]:
        """Every `#[...]` / `#![...]` attribute, as `(start, end, text)`."""
        found: list[tuple[int, int, str]] = []
        for match in ATTRIBUTE_START.finditer(self.code):
            if self.in_literal(match.start()):
                continue
            end = self.balanced_end(match.end() - 1)
            found.append((match.start(), end, self.code[match.start() : end]))
        return found

    def module_declarations(self) -> list[tuple[str, list[str]]]:
        """Every `mod <name>;`, with the attributes structurally attached to it.

        Attributes are matched by balanced brackets and then associated by
        adjacency — only whitespace may separate an attribute from the next
        attribute or from the declaration. Counting brackets per line would let
        a `]` inside a string end the attribute early and orphan it; this
        cannot.
        """
        attributes = self.attributes()
        declarations: list[tuple[str, list[str]]] = []
        for match in MOD_DECLARATION.finditer(self.code):
            if self.in_literal(match.start()):
                continue
            attached: list[str] = []
            cursor = match.start()
            for start, end, text in reversed(attributes):
                if end > cursor:
                    continue
                if self.code[end:cursor].strip():
                    break
                attached.append(text)
                cursor = start
            declarations.append((match.group(1), list(reversed(attached))))
        return declarations

    def include_arguments(self) -> list[str]:
        """The complete argument region of every `include!` invocation."""
        regions: list[str] = []
        length = len(self.code)
        for match in INCLUDE_MACRO.finditer(self.code):
            if self.in_literal(match.start()):
                continue
            index = match.end()
            while index < length and self.code[index].isspace():
                index += 1
            if index >= length or self.code[index] not in OPENERS:
                continue
            end = self.balanced_end(index)
            regions.append(self.code[index + 1 : end - 1])
        return regions


def split_call(text: str) -> tuple[str, str | None]:
    """`name(args)` -> `(name, args)`; anything else -> `(text, None)`.

    The delimiter may be any of `()`, `[]` or `{}`, and is located by balanced
    scanning over the same lexical model, so a delimiter inside a string or a
    character literal is never mistaken for the call's own.
    """
    stripped = text.strip()
    source = RustSource(stripped)
    for index, character in enumerate(stripped):
        if source.in_literal(index):
            continue
        if character in OPENERS:
            end = source.balanced_end(index)
            if stripped[end:].strip():
                return stripped, None
            return stripped[:index], stripped[index + 1 : end - 1]
    return stripped, None


def split_arguments(arguments: str) -> list[str]:
    """Splits an argument list at its own top-level commas.

    Uses the shared lexical model rather than a private quote counter, so a
    comma, parenthesis or bracket inside an ordinary string, a raw string or a
    character literal is argument *content* and never structure.
    """
    source = RustSource(arguments)
    parts: list[str] = []
    depth = 0
    start = 0
    for index, character in enumerate(arguments):
        if source.in_literal(index):
            continue
        if character in OPENERS:
            depth += 1
        elif character in CLOSERS:
            depth -= 1
        elif character == "," and depth == 0:
            parts.append(arguments[start:index])
            start = index + 1
    parts.append(arguments[start:])
    return [part for part in parts if part.strip()]


def macro_name(name: str) -> str:
    """`std::concat!` -> `concat`; a plain path's last segment, without `!`."""
    return name.strip().rstrip("!").strip().split("::")[-1].strip()


def string_literal_value(literal: str) -> str | None:
    """The compile-time *value* of one complete string literal.

    `None` unless the whole fragment is exactly one string literal, so a
    fragment that merely contains a literal is never mistaken for one, and a
    character literal — now a literal in its own right — is never mistaken for
    a string.

    An ordinary literal is decoded, because `"a\\x5fb"` and `"a_b"` name the
    same file and a guard that compared the undecoded spelling would answer
    "different" about the same module authority. A raw literal is returned
    exactly as written, because that is precisely what `r"a\\x5fb"` means.
    """
    text = literal.strip()
    if not text:
        return None
    _, literals = lexical_spans(text)
    if len(literals) != 1:
        return None
    start, end, kind = literals[0]
    if (start, end) != (0, len(text)):
        return None
    if kind == "string":
        if len(text) < 2 or not text.endswith('"'):
            return None
        return decode_rust_string(text[1:-1])
    if kind != "raw":
        return None
    hashes = 1
    while hashes < len(text) and text[hashes] == "#":
        hashes += 1
    return text[hashes + 1 : len(text) - hashes]


def constant_string_value(expression: str) -> str | None:
    """The compile-time value of the bounded constant-string forms covered here.

    Exactly a string literal — ordinary or raw — or a `concat!` of such forms,
    nested to any depth. `env!`, `option_env!`, `stringify!`, user macros and
    arbitrary expressions are deliberately *not* evaluated: this is a bounded
    repository-local evaluator, not a Rust interpreter. Those return `None`,
    and the caller treats an unevaluable `include!` as unproven authority
    rather than as harmless.
    """
    text = expression.strip()
    literal = string_literal_value(text)
    if literal is not None:
        return literal
    name, arguments = split_call(text)
    if arguments is None or macro_name(name) not in CONCAT_MACRO_NAMES:
        return None
    values = [constant_string_value(part) for part in split_arguments(arguments)]
    if any(value is None for value in values):
        return None
    return "".join(values)


def attribute_attached_paths(attribute: str) -> list[str | None]:
    """Every `path = "..."` value this attribute attaches to its item.

    A direct `#[path = "..."]` attaches one, and `cfg_attr` attaches whatever
    sits in its attribute positions — including, recursively, another
    `cfg_attr`. Both reach the same second module authority, so both count.
    `cfg_attr` attaching anything else attaches no path and is left alone: the
    theorem is semantic path indirection, not that `cfg_attr` is forbidden.

    Each attached path is reported as its decoded compile-time value, or as
    `None` when this bounded evaluator cannot determine that value. `None` is
    an *unproven* second module authority, not a harmless one, and the caller
    rejects it rather than reading it as "no path".
    """
    text = attribute.strip()
    if not text.startswith("#") or "[" not in text or not text.endswith("]"):
        return []
    return _attached_paths(text[text.index("[") + 1 : -1])


def _attached_paths(attribute: str) -> list[str | None]:
    name, arguments = split_call(attribute)
    if arguments is not None and macro_name(name) == "cfg_attr":
        return [
            alias
            for part in split_arguments(arguments)[1:]
            for alias in _attached_paths(part)
        ]
    match = PATH_NAME.match(attribute)
    if match is None:
        return []
    return [constant_string_value(attribute[match.end() :])]


def attribute_gates_compilation(attribute: str) -> bool:
    """True when `attribute` can remove the item it decorates from a build.

    Both `#[cfg(...)]` and any `#[cfg_attr(..., cfg(...))]` do, and `cfg_attr`
    nests, so this walks the attached-attribute positions rather than matching
    one spelling. `#[cfg_attr(pred, allow(...))]` attaches no `cfg` and
    therefore gates nothing.
    """
    text = attribute.strip()
    if not text.startswith("#") or "[" not in text or not text.endswith("]"):
        return False
    return _attaches_cfg(text[text.index("[") + 1 : -1])


def _attaches_cfg(attribute: str) -> bool:
    name, arguments = split_call(attribute)
    identifier = macro_name(name)
    if identifier == "cfg":
        return True
    if identifier == "cfg_attr" and arguments is not None:
        # The first argument is the predicate; every later one is an
        # attribute that `cfg_attr` attaches when that predicate holds.
        return any(_attaches_cfg(part) for part in split_arguments(arguments)[1:])
    return False


def generated_identifier_names() -> frozenset[str]:
    """The real identifier keys carried by the generated table."""
    return frozenset(GENERATED_ROW.findall(GENERATED.read_text(encoding="utf-8")))


def generated_rows_in(source: RustSource, names: frozenset[str]) -> set[str]:
    """Real generated identifiers appearing in row form as *code* in `source`.

    Keyed on the actual generated identifiers rather than on the marker
    comment, so stripping the marker from a copy does not hide it.

    A row counts only when the row itself is code: in a real table the row's
    own `(` is code and its two string literals are its cells, whereas a
    row-shaped example written *inside* one enclosing literal — a fixture that
    quotes Rust source, say — has that `(` as literal content. The same mask
    that keeps a `']'` from closing an attribute distinguishes the two, so
    quoted examples cannot accumulate toward the second-copy threshold.
    """
    return {
        match.group(1)
        for match in GENERATED_ROW.finditer(source.code)
        if match.group(1) in names and not source.in_literal(match.start())
    }


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

    def test_ordinary_string_escapes_decode_to_their_rust_values(self) -> None:
        """The guard compares path *values*, so it must decode like `rustc`."""
        for literal, expected in [
            (r'"\\"', "\\"),
            (r'"\""', '"'),
            (r'"\'"', "'"),
            (r'"\n"', "\n"),
            (r'"\r"', "\r"),
            (r'"\t"', "\t"),
            (r'"\0"', "\0"),
            (r'"\x41\u{42}C"', "ABC"),
            (r'"\u{00_5f}"', "_"),
            ('"a\\\n            b"', "ab"),
        ]:
            self.assertEqual(string_literal_value(literal), expected, literal)

    def test_unknown_or_invalid_escapes_are_unevaluable_rather_than_harmless(
        self,
    ) -> None:
        """An unknown value is unproven authority, never an assumed-safe one."""
        for literal in [
            r'"\q"',
            r'"\e"',
            r'"\u5f"',
            r'"\u{}"',
            r'"\u{5f"',
            r'"\u{110000}"',
            r'"\u{d800}"',
            r'"\x80"',
            r'"\xZZ"',
            r'"\x5"',
            '"abc\\"',
            "'a'",
        ]:
            self.assertIsNone(string_literal_value(literal), literal)
        for expression in ['env!("X")', "SOME_CONST", r'concat!("a", \'b\')']:
            self.assertIsNone(constant_string_value(expression), expression)

        # An attached path whose value cannot be determined is reported as an
        # unproven authority, never dropped as though no path were attached.
        for attribute in ["#[path = SOME_CONST]", r'#[cfg_attr(all(), path = "a\q")]']:
            self.assertEqual(attribute_attached_paths(attribute), [None], attribute)

    def test_raw_string_literals_are_never_escape_decoded(self) -> None:
        """`r"a\\x5fb"` *is* a backslash, an `x` and two digits, so it stays so."""
        self.assertEqual(
            string_literal_value(r'r"named_character\x5freferences_generated.rs"'),
            r"named_character\x5freferences_generated.rs",
        )
        self.assertEqual(string_literal_value(r'r#"a\n"#'), r"a\n")
        self.assertEqual(string_literal_value(r'r"a"'), "a")

    def test_escaped_spellings_of_the_generated_path_are_detected(self) -> None:
        """Escapes must not launder the one path that reaches the table."""
        for argument in [
            r'"named_character\x5freferences_generated.rs"',
            r'"named_character\u{5f}references_generated.rs"',
            r'concat!("named_character", "\x5f", "references_generated.rs")',
        ]:
            value = constant_string_value(argument)
            self.assertIsNotNone(value, argument)
            self.assertIn("named_character_reference", value, argument)

        for attribute in [
            r'#[path = "named_character\x5freferences_generated.rs"]',
            r'#[cfg_attr(all(), path = "named_character\u{5f}references_generated.rs")]',
        ]:
            self.assertEqual(
                attribute_attached_paths(attribute),
                ["named_character_references_generated.rs"],
                attribute,
            )

    def test_character_literals_are_lexical_content_not_structure(self) -> None:
        """A `']'` is one character of text; it cannot end its own attribute."""
        for character in [
            "']'",
            "'['",
            "')'",
            "'('",
            "'}'",
            r"'\x5d'",
            r"'\u{5d}'",
            r"'\''",
            r"'\\'",
            '\'"\'',
        ]:
            declaration = (
                f'#[cfg_attr(all(), doc = concat!("x", {character}), '
                f'path = "{GENERATED.name}")]\nmod duplicate;\n'
            )
            source = RustSource(declaration)
            attached = [
                alias
                for _, attributes in source.module_declarations()
                for attribute in attributes
                for alias in attribute_attached_paths(attribute)
            ]
            self.assertEqual(attached, [GENERATED.name], character)

    def test_lifetimes_and_labels_are_not_character_literals(self) -> None:
        """`'a`, `'a:` and `'static` open a lifetime or a label, not a literal."""
        for body in [
            "fn f<'a>(x: &'a str) -> &'a str { x }",
            "fn f(x: &'static str) {}",
            "impl Foo<'_> {}\nstruct Bar<'a, 'b>(&'a u8, &'b u8);",
            "fn f() { 'outer: loop { break 'outer; } }",
        ]:
            _, literals = lexical_spans(body)
            self.assertEqual(
                [body[start:end] for start, end, kind in literals if kind == "character"],
                [],
                body,
            )

    def test_row_shapes_inside_a_literal_do_not_count_as_a_table_copy(self) -> None:
        """A quoted example of the table is a mention; real rows are a copy."""
        names = generated_identifier_names()
        sample = sorted(names)[: MAX_INCIDENTAL_GENERATED_ROWS + 8]
        rows = ", ".join(f'("{name}", "v")' for name in sample)
        quoted = f'const FIXTURE: &str = r#"const T: &[(&str, &str)] = &[{rows}];"#;\n'
        real = f"const T: &[(&str, &str)] = &[{rows}];\n"
        self.assertEqual(generated_rows_in(RustSource(quoted), names), set())
        self.assertEqual(generated_rows_in(RustSource(real), names), set(sample))

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
          of intervening comment or whitespace can carry the path out of view,
          and its bounded compile-time value is evaluated, so splitting the
          path across `concat!` fragments does not hide it. An `include!`
          whose argument this guard cannot evaluate is unproven authority and
          is rejected rather than assumed harmless;
        - no attribute aliases the generated data through `path`, whether
          written directly or attached by `cfg_attr` — including recursively,
          and whatever the formatting. Its bounded compile-time value is
          evaluated on the same terms, and an attached path this guard cannot
          evaluate is unproven authority and is rejected;
        - both are compared as decoded *values*, never as spellings, so
          `"named_character\\x5freferences_generated.rs"` — which names exactly
          the generated file — is caught. Raw literals stay undecoded, because
          for them the spelling is the value;
        - no other crate file carries the generated rows, detected by the real
          identifiers themselves rather than by the marker comment, so
          stripping the marker from a copy does not hide it; and
        - a construct only counts when it is code. A commented-out `include!`
          or `#[path]`, and a row-shaped example written inside a literal, are
          mentions rather than indirection, and are accepted. Delimiter depth
          is likewise carried only by code: a `']'` character literal is one
          character of text and cannot end the attribute enclosing it.

        This is a bounded repository-local guard, not a Rust parser.
        """
        sources = crate_rust_sources()
        self.assertIn(GENERATED, sources)

        # 1. Module wiring, scanned crate-wide rather than in one file, so a
        #    duplicate declaration elsewhere cannot hide.
        declarations = [
            (path, name, attributes)
            for path in sources
            for name, attributes in RustSource(
                path.read_text(encoding="utf-8")
            ).module_declarations()
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
            source = RustSource(path.read_text(encoding="utf-8"))

            for argument in source.include_arguments():
                included = constant_string_value(argument)
                self.assertIsNotNone(
                    included,
                    f"{path}: include! argument is not a form this guard can "
                    f"evaluate, so its source authority is unproven: {argument!r}",
                )
                self.assertNotIn(
                    "named_character_reference",
                    included,
                    f"{path}: include! must not reach the generated data",
                )
            for _, _, attribute in source.attributes():
                for alias in attribute_attached_paths(attribute):
                    self.assertIsNotNone(
                        alias,
                        f"{path}: an attached path is not a form this guard can "
                        f"evaluate, so its module authority is unproven: "
                        f"{attribute!r}",
                    )
                    self.assertNotIn(
                        "named_character_reference",
                        alias,
                        f"{path}: path must not alias the generated data",
                    )

            # 4. Exactly one file carries the generated rows. Commented-out
            #    rows do not count; real ones do.
            if path == GENERATED:
                continue
            carried = generated_rows_in(source, names)
            self.assertLessEqual(
                len(carried),
                MAX_INCIDENTAL_GENERATED_ROWS,
                f"{path}: a second copy of the generated table",
            )

        self.assertIn(TABLE_MARKER, GENERATED.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
