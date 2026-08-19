#!/usr/bin/env python3
"""Inspect Open English WordNet senses from the pinned JSON release asset."""
from __future__ import annotations
import argparse, json, zipfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ARCHIVE = ROOT / "vendor" / "english-wordnet-2025-json.zip"
def load_synsets(archive: zipfile.ZipFile) -> dict[str, dict]:
    synsets = {}
    for name in archive.namelist():
        if name.startswith(("noun.", "verb.", "adj.", "adv.")) and name.endswith(".json"):
            synsets.update(json.loads(archive.read(name)))
    return synsets
def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("lemmas", nargs="+")
    parser.add_argument("--archive", type=Path, default=DEFAULT_ARCHIVE)
    parser.add_argument("--limit", type=int, default=5)
    args = parser.parse_args()
    with zipfile.ZipFile(args.archive) as archive:
        synsets = load_synsets(archive)
        for lemma in args.lemmas:
            entries = json.loads(archive.read(f"entries-{lemma[0].lower()}.json"))
            entry = entries.get(lemma) or entries.get(lemma.lower())
            print(f"\n## {lemma}")
            if not entry:
                print("NOT FOUND"); continue
            for pos, pos_data in entry.items():
                print(f"[{pos}]")
                for sense in pos_data.get("sense", [])[:args.limit]:
                    sid = sense["synset"]
                    print(f"  {sid}: {'; '.join(synsets[sid].get('definition', []))}")
    return 0
if __name__ == "__main__":
    raise SystemExit(main())
