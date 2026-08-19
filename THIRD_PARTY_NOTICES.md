# Third-party notices

VocaPip uses the pinned dependencies in package-lock.json and src-tauri/Cargo.lock. Their copyright and license terms remain with their respective authors. The application does not add a runtime network, updater, shell, telemetry, or AI dependency.

## Direct runtime dependencies

- Tauri 2 and official Tauri plugins: Apache-2.0 OR MIT.
- React and React DOM: MIT.
- ts-fsrs: MIT.
- zod: MIT.
- rusqlite and bundled SQLite/libsqlite3-sys: MIT for rusqlite and Public Domain for SQLite.
- serde and serde_json: MIT OR Apache-2.0.
- chrono: MIT OR Apache-2.0.
- uuid: Apache-2.0 OR MIT.
- thiserror: MIT OR Apache-2.0.

The full dependency trees are locked in package-lock.json and Cargo.lock. The bundled resources include application.txt, language-pack.txt, and this notice in src-tauri/resources/licenses.

## Content sources

The language pack is a separately attributed collection. See LICENSES.md, content-pipeline/LICENSES.md, the embedded language-pack metadata, and source-lock.json for the exact Open English WordNet, Kaikki/Wiktextract, Tatoeba, wordfreq, and derived-tiering terms.

## OpenPet research reference

OpenPet is GPL-3.0-or-later and was reviewed at commit 0675f4932a41d66d1b1fdbc6ddd94c46d6bd0ccd (v0.1.6) from https://github.com/X-T-E-R/OpenPet.git. It informed the desktop-pet window audit only. VocaPip does not bundle OpenPet source or its Nia artwork.

The layered SVG desktop-pet character is original VocaPip artwork licensed with the application under GPL-3.0-or-later. It does not incorporate or depend on external pet artwork.

## Tauri skill guidance archive

The third-party Tauri skill guidance was installed from full-stack-skills/tauri-skills commit 17c73560753f0a12aef78b7f625955c52d549438. It is development guidance retained in recovery/. It is not a runtime dependency or bundled application asset.

## Windows packaging

The release executable, NSIS installer, and MSI are unsigned because no code-signing certificate is configured. Windows SmartScreen may display a reputation warning for these local build artifacts.

## KyleBing vocabularybook

The seven `vocabularybook/` source lists are from [KyleBing/english-vocabulary](https://github.com/KyleBing/english-vocabulary) at commit `66f4a6db7b61eab3534d7b84f10eed312bca4ed6`. VocaPip redistributes them under the explicit grant notice at `vocabularybook/LICENSE`; that notice and attribution are included in the generated bundle resources. The documented upstream chain is `kajweb/dict` -> `KyleBing/dict` -> `KyleBing/english-vocabulary`; this grant does not waive upstream rights. No `wallhaven-0p1z90.png` or other image from `vocabularybook/` is bundled.
