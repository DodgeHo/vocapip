# ADR-0006: Publish the authorized vocabularybook as nullable schema v2 resources

- Status: accepted
- Date: 2026-08-19

## Context

The local vocabularybook directory contains seven UTF-8 TSV lists supplied with an explicit grant notice identifying KyleBing/english-vocabulary at commit 66f4a6db7b61eab3534d7b84f10eed312bca4ed6. The lists contain headwords and Chinese definitions, but do not provide verified English examples, phonetics, or frequency ranks. Four source lines contain extra tab-separated material.

## Decision

Build seven deterministic zh-CN to en resources with schema version 2. Deduplicate only within each category using Unicode NFKC, casefold, trim, and whitespace collapse; the first source line wins and later source lines are recorded in provenance. Parse POS labels only when they appear verbatim in the source definition. Derive a transparent suggested T1-T5 tier from the category, never a frequency rank.

phonetic, exampleEn, exampleZh, and frequencyRank remain JSON null. The validator rejects non-null values for these fields, unknown licenses, missing provenance, source hash drift, duplicate normalized words, missing categories, and bundled raster images. This avoids fabricating learning content while leaving the resource contract explicit for the main-thread loader integration.

The original seven source files and vocabularybook/LICENSE remain public repository inputs. Generated resources, the exact license notice, source commit, upstream chain, and SHA-256 values are committed under src-tauri/resources/language-packs/vocabularybook. wallhaven-0p1z90.png is not copied or bundled.

## Consequences

The main application loader must accept schema version 2 and nullable content fields, and must expose category filtering and the derived tier policy. The Tauri bundle must include the resources listed in bundle-resources.json. The supplied notice has an unfilled signature line; the repository preserves it verbatim and makes no stronger authentication claim.
