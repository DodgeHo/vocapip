from __future__ import annotations

import copy
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("vocabularybook_pipeline", ROOT / "src" / "vocabularybook_pipeline.py")
PIPELINE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = PIPELINE
SPEC.loader.exec_module(PIPELINE)

class VocabularybookPipelineTests(unittest.TestCase):
    def test_authorized_inputs_and_published_resources_validate(self):
        self.assertEqual([], PIPELINE.verify_inputs())
        self.assertEqual([], PIPELINE.validate())

    def test_rebuild_is_deterministic(self):
        PIPELINE.rebuild_check()

    def test_source_lock_matches_verified_inputs(self):
        self.assertEqual([], PIPELINE.verify_source_lock())

    def test_missing_fields_are_null_not_fabricated(self):
        records, _ = PIPELINE.parse_category(PIPELINE.CATEGORIES[0])
        for record in records:
            self.assertIsNone(record["phonetic"])
            self.assertIsNone(record["exampleEn"])
            self.assertIsNone(record["exampleZh"])
            self.assertIsNone(record["frequencyRank"])

    def test_category_deduplication_preserves_first_source_line(self):
        records, stats = PIPELINE.parse_category(PIPELINE.CATEGORIES[0])
        self.assertEqual(stats["sourceLineCount"] - stats["duplicateLineCount"], len(records))
        self.assertTrue(any(record["provenance"]["duplicateCount"] for record in records))

    def test_unknown_license_and_missing_provenance_fail_closed(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "pack"
            PIPELINE.build(output)
            path = output / "zh-CN-en-vocabularybook-middle-school.json"
            pack = json.loads(path.read_text(encoding="utf-8"))
            pack["records"][0] = copy.deepcopy(pack["records"][0])
            pack["records"][0]["license"] = "LicenseRef-Unknown"
            pack["records"][0]["provenance"].pop("sourceLine")
            path.write_text(PIPELINE.canonical_json(pack), encoding="utf-8", newline="\n")
            errors = PIPELINE.validate(output)
            self.assertTrue(any("unknown license" in error for error in errors), errors)
            self.assertTrue(any("missing provenance field sourceLine" in error for error in errors), errors)

    def test_unapproved_image_is_not_bundled(self):
        self.assertFalse(any(path.suffix.lower() in {".png", ".jpg", ".jpeg", ".webp"} for path in PIPELINE.OUTPUT_DIR.rglob("*")))

if __name__ == "__main__":
    unittest.main()
