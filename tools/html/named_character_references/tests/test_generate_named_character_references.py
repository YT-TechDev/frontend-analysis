from __future__ import annotations

import ast
import copy
import hashlib
import importlib.util
import json
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
        declaration = "const NAMED_CHARACTER_REFERENCES: &[(&str, &str)] = &["
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
            "ecacc368c8c3c6ddf9023b1e55aceb1cd3f583bbe543b6d83808f21b456c6fce",
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
            "const NAMED_CHARACTER_REFERENCE_ENTRY_COUNT: usize = 2231;"
        )
        lines[index] = "const NAMED_CHARACTER_REFERENCE_ENTRY_COUNT: usize = 2230;"
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

    def seal_index(self, lines: list[str]) -> int:
        return lines.index(generator.OWNERSHIP_REGISTRATION_ITEM)

    def test_canonical_output_declares_the_fixed_ownership_registration_once(self) -> None:
        lines = self.generated_lines()
        self.assertEqual(lines.count(generator.OWNERSHIP_REGISTRATION_ITEM), 1)
        self.assertEqual(lines.count(generator.OWNERSHIP_REGISTRATION_BEGIN), 1)
        self.assertEqual(lines.count(generator.OWNERSHIP_REGISTRATION_END), 1)
        seal = self.seal_index(lines)
        self.assertEqual(lines[seal - 1], generator.OWNERSHIP_REGISTRATION_BEGIN)
        self.assertEqual(lines[seal + 1], generator.OWNERSHIP_REGISTRATION_END)
        self.assertEqual(
            generator.OWNERSHIP_REGISTRATION_ITEM,
            "impl OwnershipRegistration for OwnerToken {}",
        )

    def test_registration_is_fixed_and_independent_of_retained_data(self) -> None:
        # The seal carries no data, so a different retained dataset must render
        # exactly the same registration item.
        source = self.source_object()
        del source["&Afr;"]
        del source["&zwnj;"]
        manifest = MANIFEST.read_bytes()
        full = generator.render_rust(
            generator.parse_and_validate_entities(ENTITIES.read_bytes()), manifest
        )
        reduced_dataset = generator.Dataset(
            generator.parse_and_validate_entities(ENTITIES.read_bytes()).entries[:16],
            2,
            0,
            10,
            ("ApplyFunction;",),
        )
        reduced = generator.render_rust(reduced_dataset, manifest)
        item = generator.OWNERSHIP_REGISTRATION_ITEM.encode("utf-8")
        self.assertEqual(full.count(item), 1)
        self.assertEqual(reduced.count(item), 1)

    def test_canonical_output_declares_no_public_visibility(self) -> None:
        # The generated source is lexically included inside the private owner,
        # so no declaration in it may widen the owner's raw-data boundary.
        for line in self.generated_lines():
            self.assertFalse(
                line.startswith(("pub ", "pub(")),
                f"generated declaration must stay private: {line}",
            )

    def test_generated_missing_registration_fails_canonical_grammar(self) -> None:
        lines = self.generated_lines()
        del lines[self.seal_index(lines)]
        with self.assertRaisesRegex(
            verifier.VerificationError, "ownership registration item"
        ):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_wrong_registration_trait_fails_canonical_grammar(self) -> None:
        lines = self.generated_lines()
        seal = self.seal_index(lines)
        lines[seal] = "impl OtherRegistration for OwnerToken {}"
        with self.assertRaisesRegex(
            verifier.VerificationError, "ownership registration item"
        ):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_wrong_registration_token_fails_canonical_grammar(self) -> None:
        lines = self.generated_lines()
        seal = self.seal_index(lines)
        lines[seal] = "impl OwnershipRegistration for OtherToken {}"
        with self.assertRaisesRegex(
            verifier.VerificationError, "ownership registration item"
        ):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_registration_carrying_data_fails_canonical_grammar(self) -> None:
        lines = self.generated_lines()
        seal = self.seal_index(lines)
        lines[seal] = "impl OwnershipRegistration for OwnerToken { const ROWS: () = (); }"
        with self.assertRaisesRegex(
            verifier.VerificationError, "ownership registration item"
        ):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_duplicate_registration_fails_canonical_grammar(self) -> None:
        lines = self.generated_lines()
        seal = self.seal_index(lines)
        lines.insert(seal + 1, generator.OWNERSHIP_REGISTRATION_ITEM)
        with self.assertRaisesRegex(
            verifier.VerificationError, "ownership registration end marker"
        ):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_trailing_registration_fails_canonical_closure(self) -> None:
        lines = self.generated_lines()
        lines.append(generator.OWNERSHIP_REGISTRATION_ITEM)
        with self.assertRaisesRegex(
            verifier.VerificationError, "unexpected content outside canonical"
        ):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_widened_table_visibility_fails_canonical_grammar(self) -> None:
        lines = self.generated_lines()
        index = lines.index("const NAMED_CHARACTER_REFERENCES: &[(&str, &str)] = &[")
        lines[index] = f"pub(super) {lines[index]}"
        with self.assertRaisesRegex(
            verifier.VerificationError, "table declaration"
        ):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_widened_provenance_visibility_fails_canonical_grammar(self) -> None:
        lines = self.generated_lines()
        index = lines.index("const RETAINED_ENTITIES_SHA256: &str =")
        lines[index] = f"pub(crate) {lines[index]}"
        with self.assertRaisesRegex(
            verifier.VerificationError, "entities hash declaration"
        ):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_generated_widened_metadata_visibility_fails_canonical_grammar(self) -> None:
        lines = self.generated_lines()
        index = lines.index("const NAMED_CHARACTER_REFERENCE_TWO_SCALAR_ENTRY_COUNT: usize = 93;")
        lines[index] = f"pub {lines[index]}"
        with self.assertRaisesRegex(
            verifier.VerificationError, "two-scalar count constant"
        ):
            verifier.parse_generated_rust(self.encoded_lines(lines))

    def test_regenerating_the_seal_is_deterministic_and_stale_output_fails_check(self) -> None:
        first = generator.generate(ROOT)
        self.assertEqual(first, generator.generate(ROOT))
        self.assertEqual(first, GENERATED.read_bytes())
        stale = first.replace(
            generator.OWNERSHIP_REGISTRATION_ITEM.encode("utf-8"), b"", 1
        )
        self.assertNotEqual(stale, first)
        with tempfile.TemporaryDirectory() as temporary:
            root = self.temporary_root(temporary, ENTITIES.read_bytes())
            (root / generator.OUTPUT_PATH).write_bytes(stale)
            # The generator resolves its repository root from its own location,
            # so exercise --check through a copy inside the temporary tree.
            relocated = root / "tools/html/named_character_references" / GENERATOR.name
            relocated.write_bytes(GENERATOR.read_bytes())
            completed = subprocess.run(
                [sys.executable, str(relocated), "--check"],
                cwd=root,
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(completed.returncode, 1, completed.stderr)
            self.assertIn("stale", completed.stderr)
            self.assertEqual((root / generator.OUTPUT_PATH).read_bytes(), stale)


if __name__ == "__main__":
    unittest.main()
