# VocaPip Implementation Status

Last updated: 2026-08-19 (Asia/Taipei)

## Product gates

| Gate | Status | Evidence |
| --- | --- | --- |
| Original PRD preserved | PASS | docs/VocaPip-PRD.md, 988 lines, SHA-256 E88EA03BBF91CE23BA7EFEA4A682BC0B1AF82CE70DBD37553AEFAEB4E4DE3C38. |
| Pinned Tauri skills | PASS | 52 skills installed from 17c73560753f0a12aef78b7f625955c52d549438; archive retained in recovery/. |
| Base architecture decision | PASS | Independent Tauri 2 shell; see ADR-0002. OpenPet audit is pinned to 0675f4932a41d66d1b1fdbc6ddd94c46d6bd0ccd. |
| End-to-end learning slice | PASS | Launch -> pet -> reveal -> four-rating review -> SQLite FSRS log -> XP/level -> restart persistence verified. |
| Offline language pack | PASS | 100 records, T1-T5 = 30/25/20/15/10, field provenance and fail-closed validator pass. |
| Tray/window/single instance | PASS | Hide-to-tray, Alt+F4 hide, tray restore, passive reminders, no default focus, no default always-on-top, and single-instance behavior verified. |
| Wordbook/settings/import/export | PASS | Search/filter/favorite/relearn/recent study, settings, license view, and validated native backup flows verified by E2E tests. |
| Automated verification | PASS | Frontend, Rust, content pipeline, migration, FSRS/XP/adaptation, and Playwright checks pass; final rerun is recorded below. |
| Offline audit | PASS | Static audit and runtime TCP observation report zero non-loopback connections. |
| Windows production build | PASS | Release executable, NSIS installer, and MSI built. Artifacts are unsigned. |

## Implemented surface

- Tauri 2 + React + TypeScript + Rust desktop application for Windows 10/11 x64.
- Transparent, borderless, draggable pet window; default alwaysOnTop=false, focus=false, and an explicit settings toggle.
- Closing the main window or using the pet hide control hides it to the system tray instead of exiting. Tray menu supports show/hide, pause/resume reminders, one-hour quiet mode, wordbook, settings, and exit.
- Real offline learning bubble with word, part of speech, English example, Chinese reveal/translation, four FSRS ratings, and pet feedback states.
- SQLite migrations, WAL, language-pack metadata, cards, FSRS snapshots/logs, XP, level, streak, settings, favorites, learning records, and import/export.
- ts-fsrs controls scheduling only. XP/level and anti-repeat rules are separate.
- First-run placement, T1-T5 tiering, 80% comfort / 20% challenge allocation, and a 100-record pilot pack.
- Centralized Chinese UI copy and language-pack schema with source/target/explanation languages plus pack and compatibility versions.
- CSP and capabilities use a minimal permission surface; runtime HTTP, WebSocket, telemetry, updater, online dictionary, and AI calls are absent.

## Verification log

| Date | Command / check | Result |
| --- | --- | --- |
| 2026-08-18 | Full PRD read | PASS, 988 lines. |
| 2026-08-18 | Git initialization and status review | PASS; no user files reverted. |
| 2026-08-18 | OpenPet inspection | PASS; exact commit and license recorded in ADR/source-lock. |
| 2026-08-18 | Content pipeline build/validate/rebuild-check | PASS; 100 records, deterministic rebuild. |
| 2026-08-18 | Content pipeline unittest discovery | PASS; 3 tests. |
| 2026-08-18 | npm run format:check | PASS. |
| 2026-08-18 | npm run lint | PASS. |
| 2026-08-18 | npm run typecheck | PASS. |
| 2026-08-18 | npm run test | PASS; 8/8 tests. |
| 2026-08-18 | npm run test:e2e | PASS; 3/3 offline renderer workflows. |
| 2026-08-18 | npm run build | PASS. |
| 2026-08-18 | cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check | PASS after final formatting. |
| 2026-08-18 | cargo test --manifest-path src-tauri/Cargo.toml --locked | PASS; 22 tests. |
| 2026-08-18 | cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings | PASS. |
| 2026-08-18 | content-pipeline unittest discovery | PASS; 3 tests. |
| 2026-08-18 | content-pipeline build, validate, and rebuild-check | PASS; 100 records and deterministic output. |
| 2026-08-18 | Content license validator | PASS; unknown license, missing provenance, sense mismatch, and hash errors fail closed. |
| 2026-08-18 | scripts/verify-offline.ps1 | PASS with `powershell.exe -NoProfile`; report at artifacts/offline-audit.json. |
| 2026-08-18 | Runtime network observation | PASS; 0 TCP connections and 0 non-loopback connections; report at artifacts/runtime-network-observation.json. |
| 2026-08-18 | Window behavior at 150% Windows scaling | PASS; hide, tray restore, no forced foreground, and no unintended topmost behavior. |
| 2026-08-18 | Production Tauri build | PASS; executable + NSIS + MSI created. |
| 2026-08-18 | Final `npm run tauri:build` | PASS; release executable, NSIS installer, and MSI regenerated after the UI rework. The MSI build downloaded the configured offline WebView2 installer during packaging only. |
| 2026-08-18 | Migration 4 regression check | PASS; fresh and existing databases no longer force always_on_top=1. |
| 2026-08-18 | Fresh release window smoke test | PASS; production executable starts non-topmost and non-foreground, a close request hides the Tauri window while keeping the process alive, and a second launch restores the hidden window through single-instance handling without creating a second process. |
| 2026-08-18 | Native pointer and surface smoke test | PASS; real Windows mouse drag moved the idle window from `(76,76)` to `(206,187)` without opening learning, learning expanded to `384×620`, close returned to `232×258`, and progress details remained attached above the pet. Screenshots are in `artifacts/ui-rework/`. |
| 2026-08-18 | Root source-lock artifact verification | PASS; 103 artifacts, all paths, byte counts, and SHA-256 hashes match. |
| 2026-08-19 | VocaPip public rebrand | PASS; product, package, Rust crate, Tauri identifier, UI copy, events, documentation, and Windows bundle names updated. Stable historical provenance IDs remain unchanged by design; see ADR-0005. |
| 2026-08-19 | Legacy data migration regression | PASS; first launch copies the former LanguagePet SQLite database plus WAL/SHM sidecars when required, preserves the old files, and accepts legacy backups. |
| 2026-08-19 | npm run verify:web | PASS; format, lint, TypeScript, Vitest 8/8, and Playwright 3/3. |
| 2026-08-19 | cargo test --manifest-path src-tauri/Cargo.toml --locked | PASS; 24/24 tests. |
| 2026-08-19 | cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings | PASS. |
| 2026-08-19 | cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check | PASS. |
| 2026-08-19 | Content pipeline unittest discovery and 100-record validation | PASS; 3/3 tests and fail-closed language-pack validation. |
| 2026-08-19 | scripts/verify-offline.ps1 | PASS; no runtime network APIs or permissions detected. |
| 2026-08-19 | npm run tauri:build | PASS; VocaPip executable, NSIS installer, and MSI package generated. |
| 2026-08-19 | Public GitHub repository | PASS; https://github.com/DodgeHo/vocapip is PUBLIC with main as the default branch. Initial release commit: 8e34f131570250701635d6f14d450c8d6654bcc1. |
| 2026-08-19 | GitHub Release v0.1.0 | PASS; non-draft, non-prerelease release published with EXE, NSIS, MSI, portable ZIP, and SHA256SUMS.txt. |

## Build artifacts

- Debug executable: src-tauri/target/debug/vocapip.exe
- Release executable: src-tauri/target/release/vocapip.exe
- NSIS installer: src-tauri/target/release/bundle/nsis/VocaPip_0.1.0_x64-setup.exe
- MSI installer: src-tauri/target/release/bundle/msi/VocaPip_0.1.0_x64_zh-CN.msi
- Release staging: artifacts/release/v0.1.0/ (ignored from Git; includes installers, raw executable, portable ZIP, release notes, and SHA-256 checksums).
- Public repository: https://github.com/DodgeHo/vocapip
- GitHub Release: https://github.com/DodgeHo/vocapip/releases/tag/v0.1.0

## Known limitations

- The pilot pack contains 100 records; it is not the PRD's later 1,000/8,000-record expansion target.
- No audio is bundled in the MVP.
- Windows artifacts are unsigned and may trigger SmartScreen warnings.
- WebView2 is configured with the offline installer mode; the target machine still needs a compatible Windows/WebView2 runtime path during installation.
- A local Vite loopback server is used only for development; the bundled production app has no runtime network capability.

## Content subtask update (2026-08-19)

- vocabularybook schema v2 content pipeline: PASS for this subtask. Seven source SHA-256 values and the supplied vocabularybook/LICENSE SHA-256 are verified fail closed.
- Generated resources: PASS. 34,142 category-deduplicated records from 54,356 source lines across 7 categories. Duplicate counts and source line provenance are retained.
- Missing factual fields: PASS. Phonetic, English example, Chinese example, and frequency rank are JSON null; validators reject non-null substitutions.
- Image exclusion: PASS. wallhaven-0p1z90.png is not copied into generated resources and is ignored by vocabularybook/.gitignore.
- Main-thread integration pending: extend src-tauri/src/models.rs and db.rs to accept schema v2 nullable fields, category IDs, and resource catalog loading; update src-tauri/tauri.conf.json resources from bundle-resources.json; add UI category selection and empty-example rendering.

| 2026-08-19 | vocabularybook content pipeline build | PASS; 34,142 records, 7 categories, 54,356 source lines. |
| 2026-08-19 | vocabularybook content validation | PASS; source/license hashes, provenance, deduplication, null-field policy, and image exclusion. |
| 2026-08-19 | vocabularybook content unittest | PASS; 6/6 tests. |
| 2026-08-19 | vocabularybook deterministic rebuild-check | PASS. |

## 2026-08-19 release verification

- Version is now `0.2.0`; release artifacts were built locally on Windows x64.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`: PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml --locked`: PASS, 30 tests.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`: PASS.
- `npm run format:check`: PASS.
- `npm run lint`: PASS.
- `npm run typecheck`: PASS.
- `npm run test`: PASS, 14 tests.
- `npm run test:e2e`: PASS, 3 tests, including renderer-offline workflows.
- `scripts/verify-vocabularybook.ps1`: PASS; 7/7 source files and deterministic pack validation.
- `scripts/verify-offline.ps1`: PASS; static audit report at `artifacts/offline-audit.json`.
- `npm run tauri:build`: PASS; executable, NSIS and MSI produced. The MSI packaging step downloaded the configured WebView2 offline installer at build time; the application runtime remains offline-only.
- Portable ZIP staged at `artifacts/VocaPip-0.2.0-windows-x64-portable.zip`; installers are unsigned because no signing certificate is configured.
