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

ROOT = Path(__file__).resolve().parents[3]
GENERATOR = ROOT / "tools/unicode/generate_ecmascript_unicode.py"
VERIFIER = ROOT / "tools/unicode/verify_generated_ecmascript_unicode.py"
MANIFEST = ROOT / "tools/unicode/upstream-manifest.json"
GENERATED = ROOT / "crates/frontend-analysis-core/src/ecmascript/unicode_generated.rs"

SPEC = importlib.util.spec_from_file_location("generate_ecmascript_unicode", GENERATOR)
assert SPEC is not None and SPEC.loader is not None
module = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = module
SPEC.loader.exec_module(module)

EXPECTED_NOTICE_SHA256 = "64e2027b6e75ad4e8a5be0047abcd43d89b5417e5ed460918590276eae5e9853"
EXPECTED_NOTICE_BLOB_SHA1 = "f665822a5195ac456838c6a44e0341bbb3fc21ed"
EXPECTED_ECMARKUP_TEMPLATE_BLOB_SHA1 = "4296cb3c430a099cdebfccdc36db3b987db3fa08"


class GeneratorTests(unittest.TestCase):
    def manifest(self) -> dict:
        return json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_manifest_pins_the_six_input_envelope(self) -> None:
        manifest = module.load_manifest(MANIFEST)
        self.assertEqual(manifest["unicode_version"], "17.0.0")
        self.assertEqual(
            manifest["ecma262_snapshot"],
            "d89c03f2db8a597bc915b363a6518d0cc8acdbc0",
        )
        self.assertEqual(
            {entry["id"] for entry in manifest["inputs"]},
            module.EXPECTED_INPUT_IDS,
        )

    def test_manifest_pins_unicode_publication_and_mirror_identity(self) -> None:
        for entry in self.manifest()["inputs"]:
            if not entry["id"].startswith("unicode-"):
                continue
            source = entry["git_mirror"]
            self.assertEqual(source["repository"], "unicode-org/unicodetools")
            self.assertEqual(source["commit"], module.UNICODE_MIRROR_COMMIT)
            self.assertIn("/Public/17.0.0/", entry["official_url"])
            self.assertIn("/ucd/17.0.0/", source["path"])

    def test_manifest_pins_ecma_snapshot_identity(self) -> None:
        for entry in self.manifest()["inputs"]:
            if not entry["id"].startswith("ecma262-"):
                continue
            source = entry["git_source"]
            self.assertEqual(source["repository"], "tc39/ecma262")
            self.assertEqual(source["commit"], module.ECMA262_SNAPSHOT)

    def test_all_retained_inputs_match_frozen_byte_identity(self) -> None:
        for entry in self.manifest()["inputs"]:
            data = (ROOT / entry["retained_path"]).read_bytes()
            source = entry.get("git_mirror") or entry["git_source"]
            self.assertEqual(len(data), source["size"], entry["id"])
            self.assertEqual(module.git_blob_sha1(data), source["blob_sha1"], entry["id"])

    def test_all_retained_inputs_have_expected_headers_or_table_ids(self) -> None:
        for entry in self.manifest()["inputs"]:
            data = module.read_verified_input(ROOT, entry)
            self.assertTrue(data)

    def test_git_blob_identity_is_content_and_length_sensitive(self) -> None:
        self.assertEqual(
            module.git_blob_sha1(b"hello\n"),
            "ce013625030ba8dba906f756967f9e9ca394464a",
        )
        self.assertNotEqual(module.git_blob_sha1(b"hello\n"), module.git_blob_sha1(b"hello"))

    def test_manifest_rejects_unicode_version_drift(self) -> None:
        manifest = self.manifest()
        manifest["unicode_version"] = "16.0.0"
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "manifest.json"
            path.write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaisesRegex(module.GenerationError, "Unicode version"):
                module.load_manifest(path)

    def test_manifest_rejects_ecma_snapshot_drift(self) -> None:
        manifest = self.manifest()
        manifest["ecma262_snapshot"] = "0" * 40
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "manifest.json"
            path.write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaisesRegex(module.GenerationError, "ECMA-262 snapshot"):
                module.load_manifest(path)

    def test_manifest_rejects_input_set_drift(self) -> None:
        manifest = self.manifest()
        manifest["inputs"] = manifest["inputs"][:-1]
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "manifest.json"
            path.write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaisesRegex(module.GenerationError, "six-input envelope"):
                module.load_manifest(path)

    def test_manifest_rejects_frozen_blob_drift(self) -> None:
        manifest = self.manifest()
        source = manifest["inputs"][0]["git_mirror"]
        source["blob_sha1"] = "0" * 40
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "manifest.json"
            path.write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaisesRegex(module.GenerationError, "frozen upstream identity"):
                module.load_manifest(path)

    def test_modified_retained_input_fails_closed(self) -> None:
        entry = copy.deepcopy(self.manifest()["inputs"][0])
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            target = root / entry["retained_path"]
            target.parent.mkdir(parents=True)
            original = (ROOT / entry["retained_path"]).read_bytes()
            target.write_bytes(original + b"# drift\n")
            with self.assertRaisesRegex(module.GenerationError, "byte length mismatch"):
                module.read_verified_input(root, entry)

    def test_malformed_target_range_fails_closed(self) -> None:
        with self.assertRaisesRegex(module.GenerationError, "malformed target record"):
            module.parse_property_ranges(b"0041.. ; ID_Start\n", "ID_Start")

    def test_unsorted_or_overlapping_ranges_fail_closed(self) -> None:
        with self.assertRaisesRegex(module.GenerationError, "overlapping or unsorted"):
            module.parse_property_ranges(
                b"0042 ; ID_Start\n0041 ; ID_Start\n",
                "ID_Start",
            )
        with self.assertRaisesRegex(module.GenerationError, "overlapping or unsorted"):
            module.parse_property_ranges(
                b"0041..0045 ; ID_Start\n0045..0047 ; ID_Start\n",
                "ID_Start",
            )

    def test_out_of_unicode_range_fails_closed(self) -> None:
        with self.assertRaisesRegex(module.GenerationError, "invalid code-point range"):
            module.parse_property_ranges(b"110000 ; ID_Start\n", "ID_Start")

    def test_subset_check_is_semantic_not_partition_dependent(self) -> None:
        subset = (module.CodePointRange(0x41, 0x45),)
        adjacent_cover = (
            module.CodePointRange(0x40, 0x42),
            module.CodePointRange(0x43, 0x45),
        )
        with_gap = (
            module.CodePointRange(0x40, 0x42),
            module.CodePointRange(0x44, 0x45),
        )
        self.assertTrue(module._ranges_are_subset(subset, adjacent_cover))
        self.assertFalse(module._ranges_are_subset(subset, with_gap))

    def test_conflicting_property_value_alias_fails_closed(self) -> None:
        data = b"gc ; X ; First\ngc ; X ; Second\nsc ; Latn ; Latin\n"
        with self.assertRaisesRegex(module.GenerationError, "conflicting alias"):
            module.parse_property_value_aliases(data)

    def test_ecma_alias_without_canonical_context_fails_closed(self) -> None:
        data = b"<table><tr><td>`AliasOnly`</td></tr></table>"
        with self.assertRaisesRegex(module.GenerationError, "lacks canonical"):
            module.parse_ecma_alias_table(data)

    def test_property_of_strings_is_exactly_the_frozen_seven(self) -> None:
        entry = next(
            item
            for item in self.manifest()["inputs"]
            if item["id"] == "ecma262-binary-unicode-properties-of-strings"
        )
        values = module.parse_ecma_string_properties(module.read_verified_input(ROOT, entry))
        self.assertEqual(values, module.EXPECTED_STRING_PROPERTIES)
        with self.assertRaisesRegex(module.GenerationError, "seven-property"):
            module.parse_ecma_string_properties(b"<table><tr><td>`Basic_Emoji`</td></tr></table>")

    def test_real_unicode_aggregate_counts_match_frozen_release(self) -> None:
        data = module.build_data(module.load_manifest(MANIFEST), ROOT)
        self.assertEqual(module.count_ranges(data.id_start), 145_916)
        self.assertEqual(module.count_ranges(data.id_continue), 149_240)
        self.assertEqual(module.count_ranges(data.zs), 17)
        self.assertTrue(module._ranges_are_subset(data.id_start, data.id_continue))

    def test_all_zs_code_points_and_adjacent_boundaries_are_exact(self) -> None:
        data = module.build_data(module.load_manifest(MANIFEST), ROOT)
        zs = {
            code_point
            for item in data.zs
            for code_point in range(item.start, item.end + 1)
        }
        expected = {
            0x0020,
            0x00A0,
            0x1680,
            *range(0x2000, 0x200B),
            0x202F,
            0x205F,
            0x3000,
        }
        self.assertEqual(zs, expected)
        for code_point in [
            0x001F,
            0x0021,
            0x009F,
            0x00A1,
            0x167F,
            0x1681,
            0x1FFF,
            0x200B,
            0x202E,
            0x2030,
            0x205E,
            0x2060,
            0x2FFF,
            0x3001,
        ]:
            self.assertNotIn(code_point, zs)

    def test_real_alias_counts_match_frozen_inputs(self) -> None:
        data = module.build_data(module.load_manifest(MANIFEST), ROOT)
        self.assertEqual(len(data.general_category_aliases), 80)
        self.assertEqual(len(data.script_aliases), 346)
        self.assertEqual(len(data.nonbinary_property_aliases), 6)
        self.assertEqual(len(data.binary_property_aliases), 98)
        self.assertEqual(len(data.string_properties), 7)

    def test_alias_tables_are_sorted_for_binary_search_lookup(self) -> None:
        data = module.build_data(module.load_manifest(MANIFEST), ROOT)
        for aliases in (
            data.general_category_aliases,
            data.script_aliases,
            data.nonbinary_property_aliases,
            data.binary_property_aliases,
        ):
            spellings = [item.spelling for item in aliases]
            self.assertEqual(spellings, sorted(spellings))

    def test_string_property_source_order_is_not_treated_as_sorted(self) -> None:
        data = module.build_data(module.load_manifest(MANIFEST), ROOT)
        self.assertNotEqual(list(data.string_properties), sorted(data.string_properties))
        lookup = (ROOT / "crates/frontend-analysis-core/src/ecmascript/unicode.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("BINARY_PROPERTIES_OF_STRINGS.contains(&spelling)", lookup)
        self.assertNotIn("BINARY_PROPERTIES_OF_STRINGS.binary_search", lookup)

    def test_generation_is_byte_deterministic_and_current(self) -> None:
        first = module.generate(MANIFEST, ROOT)
        second = module.generate(MANIFEST, ROOT)
        self.assertEqual(first, second)
        self.assertEqual(first, GENERATED.read_bytes())
        self.assertEqual(
            hashlib.sha256(first).hexdigest(),
            "037b82774fda5e69b5e89d87656ee41e07a725dc5795b0e5be16de8521135b73",
        )

    def test_generated_output_records_manifest_identity(self) -> None:
        expected = hashlib.sha256(MANIFEST.read_bytes()).hexdigest()
        text = GENERATED.read_text(encoding="utf-8")
        self.assertIn(f"upstream-manifest.json SHA-256: {expected}", text)
        self.assertIn('pub(super) const UNICODE_VERSION: &str = "17.0.0";', text)
        self.assertIn(module.ECMA262_SNAPSHOT, text)

    def test_independent_complete_equality_verifier_passes(self) -> None:
        completed = subprocess.run(
            [sys.executable, str(VERIFIER)],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("complete semantic equality: PASS", completed.stdout)

    def test_generator_and_verifier_do_not_import_network_or_process_clients(self) -> None:
        forbidden = {
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
            imported: set[str] = set()
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    imported.update(alias.name for alias in node.names)
                elif isinstance(node, ast.ImportFrom) and node.module:
                    imported.add(node.module)
            self.assertTrue(imported.isdisjoint(forbidden), f"{path}: {sorted(imported & forbidden)}")

    def test_exact_upstream_license_routing_files_are_retained(self) -> None:
        expected = {
            "tools/unicode/UNICODE-LICENSE.txt": "d7e7973c2fd6f2586a8999a69dc21e39af26be0f",
            "tools/unicode/ECMA262-LICENSE.md": "6e6b6a4742069c493c4b66e06dad9e5bd9f05f9a",
        }
        for relative, blob in expected.items():
            self.assertEqual(module.git_blob_sha1((ROOT / relative).read_bytes()), blob)

    def test_ecma_redistribution_notice_is_complete_and_content_addressed(self) -> None:
        notice = (ROOT / "tools/unicode/ECMA262-COPYRIGHT.html").read_bytes()
        self.assertEqual(hashlib.sha256(notice).hexdigest(), EXPECTED_NOTICE_SHA256)
        self.assertEqual(module.git_blob_sha1(notice), EXPECTED_NOTICE_BLOB_SHA1)
        text = notice.decode("utf-8")
        self.assertIn("ALTERNATIVE COPYRIGHT NOTICE AND COPYRIGHT LICENSE", text)
        self.assertIn("&copy; 2026 Ecma International", text)
        self.assertIn("ECMAScript® 2026 Language&nbsp;Specification", text)
        self.assertNotIn("!YEAR!", text)
        self.assertNotIn("!DOCUMENT!", text)

    def test_ecma_notice_provenance_records_exact_template_blob(self) -> None:
        text = (ROOT / "tools/unicode/UPSTREAM-LICENSING.md").read_text(encoding="utf-8")
        self.assertIn(EXPECTED_ECMARKUP_TEMPLATE_BLOB_SHA1, text)
        self.assertIn("ecmarkup 24.0.0", text)
        self.assertIn(EXPECTED_NOTICE_SHA256, text)

    def test_git_attributes_protect_only_exact_retained_evidence(self) -> None:
        lines = {
            line.strip()
            for line in (ROOT / ".gitattributes").read_text(encoding="utf-8").splitlines()
            if line.strip() and not line.lstrip().startswith("#")
        }
        self.assertEqual(
            lines,
            {
                "tools/unicode/inputs/** -text -whitespace",
                "tools/unicode/UNICODE-LICENSE.txt -text -whitespace",
                "tools/unicode/ECMA262-LICENSE.md -text -whitespace",
                "tools/unicode/ECMA262-COPYRIGHT.html -text -whitespace",
                "tools/unicode/upstream-manifest.json text eol=lf",
                "tools/unicode/generate_ecmascript_unicode.py text eol=lf",
                "tools/unicode/verify_generated_ecmascript_unicode.py text eol=lf",
                "crates/frontend-analysis-core/src/ecmascript/unicode_generated.rs text eol=lf",
            },
        )

    def test_rust_lookup_remains_crate_private(self) -> None:
        lib = (ROOT / "crates/frontend-analysis-core/src/lib.rs").read_text(encoding="utf-8")
        lookup = (ROOT / "crates/frontend-analysis-core/src/ecmascript/unicode.rs").read_text(
            encoding="utf-8"
        )
        module_root = (
            ROOT / "crates/frontend-analysis-core/src/ecmascript/mod.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("mod ecmascript;", lib)
        self.assertNotIn("pub mod ecmascript;", lib)
        self.assertNotIn("pub use ecmascript", lib)
        self.assertEqual(module_root, "mod unicode;\nmod unicode_generated;\n")
        self.assertIn("use super::unicode_generated::{", lookup)
        self.assertNotIn("mod unicode_generated;", lookup)
        self.assertIn("pub(super) fn is_id_start(code_point: u32)", lookup)
        self.assertNotIn("pub fn is_id_start", lookup)


if __name__ == "__main__":
    unittest.main()
