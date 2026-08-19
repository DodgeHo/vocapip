# ADR 0005: VocaPip public rebrand and legacy data compatibility

- Status: Accepted
- Date: 2026-08-19

## Context

The Windows MVP was developed under the working name LanguagePet. Public distribution needs a distinct product name without discarding local progress created by development builds.

## Decision

The public product name and repository slug are **VocaPip** and **vocapip**. The executable, Rust crate, package metadata, installer display name, tray text, window titles, backup label, UI copy, and internal event namespace use the new name. The Tauri identifier is `io.github.dodgeho.vocapip`, and new data is stored in `vocapip.sqlite3`.

On first launch, VocaPip checks the legacy `%APPDATA%\com.languagepet.app\languagepet.sqlite3` location only for a local migration. When present and the new database does not exist, it copies the database and SQLite sidecars into the new application directory without deleting the source. Backup import accepts both `VocaPip` and legacy `LanguagePet` product markers, while new exports use `VocaPip`.

The stable content provenance identifier `languagepet-pilot-tiering-v1` remains unchanged. It identifies the already-published derivation method rather than the application brand; changing it would break provenance continuity without changing the underlying data.

## Consequences

- Existing local learning progress survives the application identifier change.
- VocaPip installs as a distinct Windows application and creates a new autostart entry.
- Users may remove the legacy data directory manually after confirming migration.
- Public source excludes local agent skills, recovery archives, build output, runtime state, screenshots, and large upstream content caches. Release binaries are uploaded separately.
