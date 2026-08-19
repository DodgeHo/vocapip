# VocaPip 100-entry pilot content pipeline

This directory deterministically builds and validates the offline `zh-CN -> en` pilot pack at `../language-packs/zh-CN__en.pilot.100` and the Tauri runtime JSON at `../src-tauri/resources/language-packs/zh-CN-en.json`.

## Trust model

- English lemmas, parts of speech, and sense definitions are verified against the pinned Open English WordNet 2025 Edition archive.
- Chinese glosses are exact simplified Chinese Mandarin translations from the pinned Kaikki/Wiktextract English Wiktionary snapshot.
- English and Chinese examples are exact linked Tatoeba sentences. Each record retains sentence IDs, authors, per-sentence licenses, and source URLs.
- Frequency values come from the pinned `wordfreq==3.1.1` wheel; ranks are within this 100-word pilot selection.
- Every published field carries provenance. Unknown licenses, missing provenance, banned sources, bad hashes, broken sense links, and unlinked sentence pairs fail closed.
- No expert review is claimed. `learningReady` means that the automated source, license, and alignment checks passed.

## Reproduce

After acquiring the pinned source snapshots recorded in `source-lock.json`, run from the repository root:

```powershell
.\content-pipeline\.venv\Scripts\python.exe -X utf8 content-pipeline\src\pipeline.py build
.\content-pipeline\.venv\Scripts\python.exe -X utf8 content-pipeline\src\pipeline.py validate
.\content-pipeline\.venv\Scripts\python.exe -X utf8 content-pipeline\src\pipeline.py rebuild-check
.\content-pipeline\.venv\Scripts\python.exe -m unittest discover -s content-pipeline\tests -v
```

The build and validation path uses only pinned local source assets and performs no network access.

## Outputs

The auditable pack contains `manifest.json`, `records.jsonl`, `content.sqlite`, `source-lock.json`, `content-report.json`, `LICENSES.md`, `THIRD_PARTY_NOTICES.md`, and `SHA256SUMS`. The same 100 records are converted into the application's fail-closed runtime schema.

Unapproved research inputs and superseded editorial drafts are quarantined under `../recovery/excluded-unpublished` and are not content-pipeline inputs, bundle resources, or redistributable language-pack material.

Large upstream source snapshots are intentionally excluded from the public Git repository. The redistributable 100-record pack, record-level provenance, checksums, and pinned source metadata are committed.

## vocabularybook pipeline

The file src/vocabularybook_pipeline.py validates the supplied authorization notice and seven byte-pinned TSV files under ../vocabularybook, deduplicates within each exam category using Unicode NFKC plus casefold and whitespace normalization, preserves the first source line, and records duplicate lines and source hashes. It produces seven schemaVersion 2 resources plus catalog.json, LICENSE, SHA256SUMS, and bundle-resources.json under ../src-tauri/resources/language-packs/vocabularybook.

The source lists do not contain verified English examples, phonetics, or frequency values. Those fields remain JSON null and the pipeline rejects non-null values. Four input rows with extra TSV columns are retained in provenance. wallhaven-0p1z90.png is explicitly excluded.

Run:

    python -X utf8 content-pipeline\src\vocabularybook_pipeline.py build
    python -X utf8 content-pipeline\src\vocabularybook_pipeline.py validate
    python -X utf8 content-pipeline\src\vocabularybook_pipeline.py rebuild-check
    python -X utf8 -m unittest discover -s content-pipeline\tests -p test_vocabularybook_pipeline.py -v
