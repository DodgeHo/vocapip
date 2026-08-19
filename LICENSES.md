# Licensing overview

## Application code

VocaPip application source code and the project-authored layered SVG pet artwork are licensed under GPL-3.0-or-later. The pet character artwork is original to this project and has no external character-art, sprite, or illustration dependency. The root LICENSE file and the bundled application notice contain the GPL text and election.

No OpenPet source, Nia sprite, or third-party character asset is bundled. No audio file is bundled.

## Official pilot language pack

The bundled pack is zh-CN -> en, pack id zh-CN\_\_en.pilot, pack version 0.1.0, compatibility version 1, and contains 100 records. The collection is distributed under CC BY-SA 4.0. Record and field provenance remains authoritative for downstream reuse.

The pack validator accepts only these record-level license identifiers:

- CC0-1.0
- CC-BY-4.0
- CC-BY-SA-4.0
- CC-BY-2.0-FR
- Public-Domain

Unknown, missing, NC, ND, research-only, education-only, custom-restricted, and other unapproved licenses fail closed.

## Content source mapping

- Open English WordNet 2025 Edition, commit dc343f2683279ecbb13fab4e2fd778d7b162d287, CC BY 4.0: English lemma, display form, part of speech, synset id, and English definition fields.
- Kaikki English Wiktionary extract, dump 2026-08-05, CC BY-SA 4.0: Chinese gloss and translation-sense fields. The selected Wiktextract data is retained under content-pipeline/vendor and is hash-locked.
- Tatoeba exports, snapshot 2026-08-15: linked English and Chinese examples and sentence-pair links, using CC0-1.0 and CC BY-2.0-FR records as recorded per sentence.
- wordfreq 3.1.1: Apache-2.0 software; frequency data is CC BY-SA 4.0. Only frequency values used for pilot ranking are published.
- VocaPip deterministic pilot tier assignment: project-authored CC0-1.0 metadata for the derived T1-T5 and learningReady fields.

Every published record contains a stable id, word, part of speech, Chinese gloss, bilingual example, tier, frequency rank, record license, source id, source URL, and field-level provenance. Exact versions, URLs, artifact sizes, and SHA-256 hashes are in source-lock.json and the language-pipeline reports.

## Compliance notes

The pack's collection license does not remove attribution or share-alike obligations from the underlying sources. Redistributors must retain the notices and follow each source's license. The content pipeline is fail-closed and does not publish records whose license or sense alignment cannot be established.

## vocabularybook/ word-list data

The seven `vocabularybook/*.txt` word lists (初中 / 高中 / 四级 / 六级 / 考研 / 托福 / SAT) are byte-for-byte identical to the files of the same names in KyleBing/english-vocabulary at commit 66f4a6db7b61eab3534d7b84f10eed312bca4ed6. The upstream author, KyleBing, has granted VocaPip and its users a perpetual, worldwide, royalty-free, non-exclusive license to use, modify, and distribute the files, including for commercial purposes, subject to attribution. See `vocabularybook/LICENSE` for the full grant. That grant covers only the upstream author's own rights: the files derive from the chain kajweb/dict → KyleBing/dict → KyleBing/english-vocabulary, and any upstream rights remain with their respective owners.
