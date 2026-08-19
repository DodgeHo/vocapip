from __future__ import annotations

import copy
import importlib.util
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("languagepet_content_pipeline", ROOT / "src" / "pipeline.py")
PIPELINE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = PIPELINE
SPEC.loader.exec_module(PIPELINE)


class PublishedPackTests(unittest.TestCase):
    def test_published_pack_passes_fail_closed_validator(self):
        self.assertEqual([], PIPELINE.validate())

    def test_unknown_license_and_missing_provenance_are_rejected(self):
        with tempfile.TemporaryDirectory(dir=ROOT) as temporary:
            pack = Path(temporary) / "pack"
            shutil.copytree(PIPELINE.PACK, pack)
            records = PIPELINE.load_records(pack)
            records[0] = copy.deepcopy(records[0])
            records[0]["licenses"].append("LicenseRef-Unknown")
            records[0]["fieldProvenance"].pop("exampleZh")
            (pack / "records.jsonl").write_text(
                "\n".join(PIPELINE.cjson(record) for record in records) + "\n",
                encoding="utf-8",
                newline="\n",
            )

            errors = PIPELINE.validate(pack, hashes=False)
            self.assertTrue(any("unknown record license" in error for error in errors), errors)
            self.assertTrue(
                any("missing provenance ['exampleZh']" in error for error in errors),
                errors,
            )

    def test_tampered_source_hash_is_rejected(self):
        with tempfile.TemporaryDirectory(dir=ROOT) as temporary:
            pack = Path(temporary) / "pack"
            shutil.copytree(PIPELINE.PACK, pack)
            lock_path = pack / "source-lock.json"
            lock = json.loads(lock_path.read_text(encoding="utf-8"))
            lock["artifacts"][0]["sha256"] = "0" * 64
            lock_path.write_text(
                json.dumps(lock, ensure_ascii=False, sort_keys=True, indent=2) + "\n",
                encoding="utf-8",
                newline="\n",
            )

            errors = PIPELINE.validate(pack)
            self.assertTrue(any("source hash mismatch" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main()
