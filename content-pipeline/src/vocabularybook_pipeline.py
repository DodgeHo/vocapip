#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import tempfile
import unicodedata
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib.parse import quote

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT.parent
SOURCE_DIR = WORKSPACE / "vocabularybook"
OUTPUT_DIR = WORKSPACE / "src-tauri" / "resources" / "language-packs" / "vocabularybook"
SOURCE_REPOSITORY = "KyleBing/english-vocabulary"
SOURCE_URL = "https://github.com/KyleBing/english-vocabulary"
SOURCE_COMMIT = "66f4a6db7b61eab3534d7b84f10eed312bca4ed6"
SOURCE_ID = "kylebing-english-vocabulary-66f4a6d"
PACK_VERSION = "2026.08.19"
PACK_LICENSE = "LicenseRef-KyleBing-English-Vocabulary-Explicit-Grant"
LICENSE_SHA256 = "86684be7e0d4efbc53c422e05cceab0031f8ebe5b0c6a32a4b042bc75971e1f1"
SOURCE_LOCK_PATH = WORKSPACE / "source-lock.json"

@dataclass(frozen=True)
class Category:
    order: int
    category_id: str
    display_name: str
    filename: str
    sha256: str
    suggested_tier: str

CATEGORIES = (
    Category(1, "middle-school", "初中", "1 初中-乱序.txt", "97a0c7aa25dc2cc93fac34a533715c30b7fe4b558d8c2c69c2b366e6cd7bbd16", "T1"),
    Category(2, "high-school", "高中", "2 高中-乱序.txt", "7b2905414a9dba3eb829feddc749e94b16ba67cc38426dead21117d6cc30d4ed", "T2"),
    Category(3, "cet4", "四级", "3 四级-乱序.txt", "b6ef93b392837bfad92467ed6a8b3f22da6caf07fcfe8858c4f9eccb72069835", "T2"),
    Category(4, "cet6", "六级", "4 六级-乱序.txt", "4a938c26f5732194330c5453a7fe28d65cf3fd7009fcdfb5bea4517b8ce48ddb", "T3"),
    Category(5, "postgraduate", "考研", "5 考研-乱序.txt", "8a88f5cc466ec18f86f389460e986530444083f5801a131693a057b6b6a5ab17", "T3"),
    Category(6, "toefl", "托福", "6 托福-乱序.txt", "f7028faff8298e1f75421800b8370014f58fa1a76f0724aa5590b220b36bb516", "T4"),
    Category(7, "sat", "SAT", "7 SAT-乱序.txt", "2e373bf3edbe6544c35b62c9720cb946d9dc3140befa709672feb7cc2cc1b641", "T5"),
)
POS_PATTERN = re.compile(r"(?<![A-Za-z])(?:n|v|vt|vi|adj|adv|prep|conj|pron|num|art|int|aux|modal|abbr)\.", re.IGNORECASE)

def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()

def canonical_json(value: Any, *, pretty: bool = False) -> str:
    args = {"ensure_ascii": False, "sort_keys": True}
    if pretty:
        return json.dumps(value, indent=2, **args) + "\n"
    return json.dumps(value, separators=(",", ":"), **args) + "\n"

def normalize_word(word: str) -> str:
    return " ".join(unicodedata.normalize("NFKC", word).strip().casefold().split())

def parse_pos(definition: str) -> list[str]:
    labels = []
    for match in POS_PATTERN.finditer(definition):
        label = match.group(0).lower().rstrip(".")
        if label not in labels:
            labels.append(label)
    return labels

def record_id(category_id: str, normalized_word: str) -> str:
    suffix = hashlib.sha256(normalized_word.encode("utf-8")).hexdigest()[:20]
    return f"vkb-{category_id}-{suffix}"

def source_url(filename: str) -> str:
    return f"{SOURCE_URL}/blob/{SOURCE_COMMIT}/{quote(filename, safe='')}"

def verify_source_lock() -> list[str]:
    if not SOURCE_LOCK_PATH.is_file():
        return ["missing source-lock.json"]
    try:
        lock = json.loads(SOURCE_LOCK_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"source-lock.json cannot be parsed: {error}"]
    entry = next((item for item in lock.get("contentSources", []) if item.get("id") == SOURCE_ID), None)
    if entry is None:
        return ["source-lock.json missing KyleBing vocabulary entry"]
    errors = []
    expected_metadata = {
        "repository": SOURCE_REPOSITORY,
        "commit": SOURCE_COMMIT,
        "license": PACK_LICENSE,
        "licenseFile": "vocabularybook/LICENSE",
        "licenseSha256": LICENSE_SHA256,
    }
    for key, expected in expected_metadata.items():
        if entry.get(key) != expected:
            errors.append(f"source-lock metadata mismatch: {key}")
    artifacts = {artifact.get("path"): artifact.get("sha256") for artifact in entry.get("artifacts", [])}
    expected_artifacts = {f"vocabularybook/{category.filename}": category.sha256 for category in CATEGORIES}
    expected_artifacts["vocabularybook/LICENSE"] = LICENSE_SHA256
    if artifacts != expected_artifacts:
        errors.append("source-lock artifact hashes or paths mismatch")
    return errors

def verify_inputs(source_dir: Path = SOURCE_DIR) -> list[str]:
    errors = verify_source_lock() if source_dir == SOURCE_DIR else []
    license_path = source_dir / "LICENSE"
    if not license_path.is_file():
        errors.append("missing vocabularybook/LICENSE")
    elif sha256(license_path) != LICENSE_SHA256:
        errors.append("vocabularybook/LICENSE SHA-256 mismatch")
    else:
        text = license_path.read_text(encoding="utf-8")
        for marker in (SOURCE_REPOSITORY, SOURCE_COMMIT, "perpetual, worldwide, royalty-free", "kajweb/dict", "KyleBing/dict"):
            if marker not in text:
                errors.append(f"vocabularybook/LICENSE missing required marker: {marker}")
    expected = {category.filename for category in CATEGORIES}
    actual = {path.name for path in source_dir.glob("*.txt")}
    if actual != expected:
        errors.append(f"unexpected vocabulary text file set: {sorted(actual ^ expected)!r}")
    for category in CATEGORIES:
        path = source_dir / category.filename
        if not path.is_file():
            errors.append(f"missing source file: {category.filename}")
        elif sha256(path) != category.sha256:
            errors.append(f"source SHA-256 mismatch: {category.filename}")
    return errors

def parse_category(category: Category, source_dir: Path = SOURCE_DIR):
    path = source_dir / category.filename
    lines = path.read_text(encoding="utf-8", errors="strict").splitlines()
    records = []
    by_word = {}
    duplicate_lines = conflicting_definitions = extra_column_lines = 0
    for line_number, raw_line in enumerate(lines, 1):
        columns = raw_line.split("\t")
        if len(columns) < 2:
            raise ValueError(f"{category.filename}:{line_number}: expected at least two TSV columns")
        word, definition = columns[0].strip(), columns[1].strip()
        if not word or not definition:
            raise ValueError(f"{category.filename}:{line_number}: empty word or primary definition")
        normalized = normalize_word(word)
        if not normalized or not re.search(r"[a-z]", normalized):
            raise ValueError(f"{category.filename}:{line_number}: headword has no Latin letter")
        if len(columns) > 2:
            extra_column_lines += 1
        existing = by_word.get(normalized)
        if existing is not None:
            duplicate_lines += 1
            existing["provenance"]["duplicateSourceLines"].append(line_number)
            existing["provenance"]["duplicateCount"] += 1
            if definition != existing["definitionZh"]:
                conflicting_definitions += 1
                existing["provenance"]["distinctDuplicateDefinitionCount"] += 1
            continue
        provenance = {
            "sourceFile": f"vocabularybook/{category.filename}",
            "sourceSha256": category.sha256,
            "sourceLine": line_number,
            "sourceColumnCount": len(columns),
            "sourceLineSha256": hashlib.sha256(raw_line.encode("utf-8")).hexdigest(),
            "duplicateSourceLines": [],
            "duplicateCount": 0,
            "distinctDuplicateDefinitionCount": 0,
            "normalization": "Unicode NFKC, trim, casefold, collapse whitespace",
            "fieldSources": {
                "word": {"column": 1, "line": line_number},
                "definitionZh": {"column": 2, "line": line_number},
                "partOfSpeech": {"method": "labels extracted verbatim from definitionZh"},
                "tier": {"method": "deterministic category-to-suggested-tier mapping"},
            },
            "intentionallyAbsent": ["phonetic", "exampleEn", "exampleZh", "frequencyRank"],
        }
        if len(columns) > 2:
            provenance["extraSourceColumns"] = columns[2:]
        record = {
            "id": record_id(category.category_id, normalized), "word": word, "normalizedWord": normalized,
            "phonetic": None, "partOfSpeech": parse_pos(definition) or None, "definitionZh": definition,
            "exampleEn": None, "exampleZh": None, "frequencyRank": None, "tier": category.suggested_tier,
            "categoryId": category.category_id, "license": PACK_LICENSE, "sourceId": SOURCE_ID,
            "sourceUrl": source_url(category.filename), "learningReady": True, "provenance": provenance,
        }
        by_word[normalized] = record
        records.append(record)
    stats = {
        "sourceLineCount": len(lines), "recordCount": len(records), "duplicateLineCount": duplicate_lines,
        "conflictingDuplicateDefinitionCount": conflicting_definitions, "extraColumnLineCount": extra_column_lines,
    }
    return records, stats

def pack_document(category: Category, records, stats):
    return {
        "schemaVersion": 2, "id": f"zh-CN__en.vocabularybook.{category.category_id}",
        "name": f"中文到英文 {category.display_name}词库", "sourceLanguage": "zh-CN", "targetLanguage": "en",
        "explanationLanguage": "zh-CN", "packVersion": PACK_VERSION, "compatibilityVersion": "2",
        "packLicense": PACK_LICENSE,
        "category": {"id": category.category_id, "displayName": category.display_name, "order": category.order,
                     "suggestedTier": category.suggested_tier, "tierIsDerived": True},
        "source": {"id": SOURCE_ID, "repository": SOURCE_REPOSITORY, "url": SOURCE_URL, "commit": SOURCE_COMMIT,
                   "file": category.filename, "sha256": category.sha256, "license": PACK_LICENSE,
                   "licensePath": "vocabularybook/LICENSE", "licenseSha256": LICENSE_SHA256,
                   "upstreamChain": ["kajweb/dict", "KyleBing/dict", SOURCE_REPOSITORY]},
        "contentPolicy": {"missingSourceFieldsRemainNull": True, "deduplicationScope": "category",
                          "deduplicationWinner": "first source line",
                          "normalization": "Unicode NFKC, trim, casefold, collapse whitespace"},
        "statistics": stats, "records": records,
    }

def bundle_resources_document():
    resources = [
        "resources/language-packs/vocabularybook/catalog.json",
        "resources/language-packs/vocabularybook/LICENSE",
        "resources/language-packs/vocabularybook/SHA256SUMS",
        *[
            f"resources/language-packs/vocabularybook/zh-CN-en-vocabularybook-{category.category_id}.json"
            for category in CATEGORIES
        ],
    ]
    return {
        "schemaVersion": 1,
        "description": "Suggested Tauri bundle resource mapping; main-thread integration must add these paths to tauri.conf.json.",
        "sourceAttribution": f"{SOURCE_REPOSITORY}@{SOURCE_COMMIT}",
        "resources": resources,
        "excluded": ["vocabularybook/wallhaven-0p1z90.png"],
        "runtimeExpectation": "The Rust resource loader must accept schemaVersion 2 and nullable phonetic/example/frequencyRank fields; no placeholder values may be substituted.",
    }

def build(output_dir: Path = OUTPUT_DIR, source_dir: Path = SOURCE_DIR):
    errors = verify_inputs(source_dir)
    if errors:
        raise ValueError("; ".join(errors))
    output_dir.mkdir(parents=True, exist_ok=True)
    for old in output_dir.glob("*.json"):
        if old.name != "bundle-resources.json":
            old.unlink()
    summaries = []
    total_records = total_lines = 0
    for category in CATEGORIES:
        records, stats = parse_category(category, source_dir)
        document = pack_document(category, records, stats)
        name = f"zh-CN-en-vocabularybook-{category.category_id}.json"
        output = output_dir / name
        output.write_text(canonical_json(document), encoding="utf-8", newline="\n")
        total_records += stats["recordCount"]
        total_lines += stats["sourceLineCount"]
        summaries.append({"id": document["id"], "category": document["category"], "resource": name,
                          "sha256": sha256(output), "bytes": output.stat().st_size, **stats})
    shutil.copyfile(source_dir / "LICENSE", output_dir / "LICENSE")
    catalog = {
        "schemaVersion": 1, "id": "zh-CN__en.vocabularybook.catalog", "packVersion": PACK_VERSION,
        "compatibilityVersion": "2", "categoryCount": len(summaries), "sourceLineCount": total_lines,
        "recordCount": total_records, "packLicense": PACK_LICENSE, "licenseResource": "LICENSE",
        "licenseSha256": LICENSE_SHA256, "sourceRepository": SOURCE_REPOSITORY, "sourceCommit": SOURCE_COMMIT,
        "upstreamChain": ["kajweb/dict", "KyleBing/dict", SOURCE_REPOSITORY], "categories": summaries,
    }
    catalog_path = output_dir / "catalog.json"
    catalog_path.write_text(canonical_json(catalog, pretty=True), encoding="utf-8", newline="\n")
    (output_dir / "bundle-resources.json").write_text(
        canonical_json(bundle_resources_document(), pretty=True), encoding="utf-8", newline="\n"
    )
    sums = []
    for path in sorted(output_dir.iterdir(), key=lambda item: item.name):
        if path.is_file() and path.name != "SHA256SUMS":
            sums.append(f"{sha256(path)}  {path.name}")
    (output_dir / "SHA256SUMS").write_text("\n".join(sums) + "\n", encoding="ascii", newline="\n")
    return catalog

def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))

def validate(output_dir: Path = OUTPUT_DIR, source_dir: Path = SOURCE_DIR):
    errors = verify_inputs(source_dir)
    required = {"catalog.json", "LICENSE", "SHA256SUMS", *(f"zh-CN-en-vocabularybook-{c.category_id}.json" for c in CATEGORIES)}
    missing = sorted(name for name in required if not (output_dir / name).is_file())
    if missing:
        return errors + [f"missing output files: {missing!r}"]
    if sha256(output_dir / "LICENSE") != LICENSE_SHA256:
        errors.append("bundled LICENSE SHA-256 mismatch")
    catalog = load_json(output_dir / "catalog.json")
    if catalog.get("sourceCommit") != SOURCE_COMMIT or catalog.get("packLicense") != PACK_LICENSE:
        errors.append("catalog source commit or license mismatch")
    observed_total = 0
    for category in CATEGORIES:
        pack = load_json(output_dir / f"zh-CN-en-vocabularybook-{category.category_id}.json")
        if pack.get("schemaVersion") != 2 or pack.get("compatibilityVersion") != "2":
            errors.append(f"{category.category_id}: incompatible schema metadata")
            continue
        if pack.get("packLicense") != PACK_LICENSE or pack.get("source", {}).get("sha256") != category.sha256:
            errors.append(f"{category.category_id}: source or license mismatch")
        records = pack.get("records")
        if not isinstance(records, list) or not records:
            errors.append(f"{category.category_id}: missing records")
            continue
        observed_total += len(records)
        ids, words = set(), set()
        for record in records:
            prefix = f"{category.category_id}/{record.get('id', 'unknown')}"
            if record.get("id") in ids or record.get("normalizedWord") in words:
                errors.append(f"{prefix}: duplicate id or normalized word")
            ids.add(record.get("id")); words.add(record.get("normalizedWord"))
            if record.get("license") != PACK_LICENSE or record.get("sourceId") != SOURCE_ID:
                errors.append(f"{prefix}: unknown license or source")
            if any(record.get(field) is not None for field in ("phonetic", "exampleEn", "exampleZh", "frequencyRank")):
                errors.append(f"{prefix}: unsupported field was fabricated")
            provenance = record.get("provenance")
            if not isinstance(provenance, dict):
                errors.append(f"{prefix}: missing provenance")
                continue
            for key in ("sourceFile", "sourceSha256", "sourceLine", "sourceLineSha256", "fieldSources", "intentionallyAbsent"):
                if key not in provenance:
                    errors.append(f"{prefix}: missing provenance field {key}")
            if provenance.get("sourceSha256") != category.sha256:
                errors.append(f"{prefix}: provenance source hash mismatch")
        expected_records, expected_stats = parse_category(category, source_dir)
        if records != expected_records or pack.get("statistics") != expected_stats:
            errors.append(f"{category.category_id}: output is not the deterministic source parse")
    if observed_total != catalog.get("recordCount"):
        errors.append("catalog record count mismatch")
    for line in (output_dir / "SHA256SUMS").read_text(encoding="ascii").splitlines():
        digest, name = line.split("  ", 1)
        if not (output_dir / name).is_file() or sha256(output_dir / name) != digest:
            errors.append(f"SHA256SUMS mismatch: {name}")
    if any(path.suffix.lower() in {".png", ".jpg", ".jpeg", ".webp"} for path in output_dir.rglob("*")):
        errors.append("image asset present in vocabularybook output")
    return errors

def rebuild_check():
    with tempfile.TemporaryDirectory(prefix="vocapip-vocabularybook-") as temporary:
        rebuilt = Path(temporary) / "output"
        build(rebuilt)
        expected = {path.name: sha256(path) for path in OUTPUT_DIR.iterdir() if path.is_file()}
        actual = {path.name: sha256(path) for path in rebuilt.iterdir() if path.is_file()}
        if expected != actual:
            raise RuntimeError("non-deterministic vocabularybook rebuild")

def main(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("build", "validate", "rebuild-check", "report"))
    args = parser.parse_args(argv)
    if args.command == "build":
        catalog = build(); print(f"built {catalog['recordCount']} records in {catalog['categoryCount']} categories"); return 0
    if args.command == "validate":
        errors = validate()
        if errors:
            for error in errors: print(f"ERROR: {error}")
            return 1
        print("vocabularybook validation passed"); return 0
    if args.command == "rebuild-check":
        rebuild_check(); print("vocabularybook deterministic rebuild passed"); return 0
    catalog = load_json(OUTPUT_DIR / "catalog.json")
    for category in catalog["categories"]:
        print(f"{category['category']['displayName']}: {category['recordCount']} records from {category['sourceLineCount']} lines; {category['duplicateLineCount']} duplicates")
    print(f"total: {catalog['recordCount']} records from {catalog['sourceLineCount']} lines"); return 0

if __name__ == "__main__":
    raise SystemExit(main())
