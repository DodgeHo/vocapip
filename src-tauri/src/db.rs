use crate::error::{AppError, AppResult};
use crate::models::{LanguagePack, LanguageRecord};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const DATABASE_APPLICATION_ID: i64 = 1_280_333_140;
pub const DATABASE_VERSION: i64 = 4;
const BUILTIN_PACK: &str = include_str!("../resources/language-packs/zh-CN-en.json");
const ALLOWED_RECORD_LICENSES: &[&str] = &[
    "CC0-1.0",
    "CC-BY-2.0-FR",
    "CC-BY-4.0",
    "CC-BY-SA-4.0",
    "Public-Domain",
];

const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../migrations/0001_initial.sql")),
    (2, include_str!("../migrations/0002_indexes.sql")),
    (3, include_str!("../migrations/0003_default_reminder.sql")),
    (
        4,
        include_str!("../migrations/0004_default_not_always_on_top.sql"),
    ),
];

#[derive(Debug)]
pub struct DbState {
    path: PathBuf,
    pub write_lock: Mutex<()>,
}

impl DbState {
    pub fn initialize(path: PathBuf) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let state = Self {
            path,
            write_lock: Mutex::new(()),
        };
        let mut connection = state.connect()?;
        run_migrations(&mut connection)?;
        install_pack_json(&mut connection, BUILTIN_PACK)?;
        Ok(state)
    }

    #[cfg(test)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn connect(&self) -> AppResult<Connection> {
        open_connection(&self.path)
    }
}

pub fn open_connection(path: &Path) -> AppResult<Connection> {
    let connection = Connection::open(path)?;
    connection.pragma_update(None, "foreign_keys", true)?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "FULL")?;
    connection.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(connection)
}

pub fn run_migrations(connection: &mut Connection) -> AppResult<()> {
    let application_id: i64 =
        connection.pragma_query_value(None, "application_id", |row| row.get(0))?;
    if application_id != 0 && application_id != DATABASE_APPLICATION_ID {
        return Err(AppError::InvalidData(
            "database belongs to another application".into(),
        ));
    }

    let current: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current > DATABASE_VERSION {
        return Err(AppError::InvalidData(format!(
            "database schema {current} is newer than supported schema {DATABASE_VERSION}"
        )));
    }

    for (version, sql) in MIGRATIONS {
        if *version > current {
            let transaction = connection.transaction()?;
            transaction.execute_batch(sql)?;
            let applied: i64 =
                transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
            if applied != *version {
                return Err(AppError::InvalidData(format!(
                    "migration {version} did not update user_version"
                )));
            }
            transaction.commit()?;
        }
    }
    Ok(())
}

pub fn install_pack_json(connection: &mut Connection, json: &str) -> AppResult<()> {
    let pack: LanguagePack = serde_json::from_str(json)?;
    validate_pack(&pack)?;

    let existing_version: Option<String> = connection
        .query_row(
            "SELECT pack_version FROM language_packs WHERE id = ?1",
            [&pack.id],
            |row| row.get(0),
        )
        .optional()?;
    if existing_version.as_deref() == Some(pack.pack_version.as_str()) {
        return Ok(());
    }

    let transaction = connection.transaction()?;
    upsert_pack(&transaction, &pack)?;
    for record in &pack.records {
        upsert_record(&transaction, &pack.id, record)?;
    }
    transaction.execute(
        "UPDATE learner_profile SET active_pack_id = COALESCE(active_pack_id, ?1), updated_at = ?2 WHERE id = 1",
        params![pack.id, Utc::now().to_rfc3339()],
    )?;
    transaction.execute(
        "UPDATE settings SET language_pack_id = CASE WHEN language_pack_id = '' OR language_pack_id = 'zh-CN__en.pilot' THEN ?1 ELSE language_pack_id END, updated_at = ?2 WHERE id = 1",
        params![pack.id, Utc::now().to_rfc3339()],
    )?;
    transaction.commit()?;
    Ok(())
}

fn validate_pack(pack: &LanguagePack) -> AppResult<()> {
    if pack.schema_version != 1 {
        return Err(AppError::InvalidData(format!(
            "unsupported language pack schema {}",
            pack.schema_version
        )));
    }
    for value in [
        &pack.id,
        &pack.name,
        &pack.source_language,
        &pack.target_language,
        &pack.explanation_language,
        &pack.pack_version,
        &pack.compatibility_version,
        &pack.pack_license,
    ] {
        if value.trim().is_empty() {
            return Err(AppError::InvalidData(
                "language pack metadata contains an empty field".into(),
            ));
        }
    }
    if pack.records.is_empty() {
        return Err(AppError::InvalidData(
            "language pack contains no records".into(),
        ));
    }
    let mut ids = std::collections::HashSet::new();
    for record in &pack.records {
        validate_record(record)?;
        if !ids.insert(&record.id) {
            return Err(AppError::InvalidData(format!(
                "duplicate language record id {}",
                record.id
            )));
        }
    }
    Ok(())
}

fn validate_record(record: &LanguageRecord) -> AppResult<()> {
    for (name, value) in [
        ("id", &record.id),
        ("word", &record.word),
        ("partOfSpeech", &record.part_of_speech),
        ("definitionZh", &record.definition_zh),
        ("exampleEn", &record.example_en),
        ("exampleZh", &record.example_zh),
        ("license", &record.license),
        ("sourceId", &record.source_id),
        ("sourceUrl", &record.source_url),
    ] {
        if value.trim().is_empty() {
            return Err(AppError::InvalidData(format!(
                "record {} has empty {name}",
                record.id
            )));
        }
    }
    if !ALLOWED_RECORD_LICENSES.contains(&record.license.as_str()) {
        return Err(AppError::InvalidData(format!(
            "record {} uses non-allowlisted license {}",
            record.id, record.license
        )));
    }
    if !record.provenance.is_object() {
        return Err(AppError::InvalidData(format!(
            "record {} has no field-level provenance object",
            record.id
        )));
    }
    if record.frequency_rank < 0 {
        return Err(AppError::InvalidData(format!(
            "record {} has an invalid frequency rank",
            record.id
        )));
    }
    Ok(())
}

fn upsert_pack(transaction: &Transaction<'_>, pack: &LanguagePack) -> AppResult<()> {
    transaction.execute(
        "INSERT INTO language_packs (id, name, source_language, target_language, explanation_language, pack_version, compatibility_version, record_count, pack_license, imported_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, source_language = excluded.source_language, target_language = excluded.target_language,
           explanation_language = excluded.explanation_language, pack_version = excluded.pack_version, compatibility_version = excluded.compatibility_version,
           record_count = excluded.record_count, pack_license = excluded.pack_license, imported_at = excluded.imported_at",
        params![pack.id, pack.name, pack.source_language, pack.target_language, pack.explanation_language, pack.pack_version,
            pack.compatibility_version, pack.records.len() as i64, pack.pack_license, Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

fn upsert_record(
    transaction: &Transaction<'_>,
    pack_id: &str,
    record: &LanguageRecord,
) -> AppResult<()> {
    let provenance = serde_json::to_string(&record.provenance)?;
    transaction.execute(
        "INSERT INTO lexemes (id, pack_id, word, phonetic, part_of_speech, definition_zh, example_en, example_zh, tier, frequency_rank, record_license, source_id, source_url, provenance_json, learning_ready)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
         ON CONFLICT(id) DO UPDATE SET pack_id = excluded.pack_id, word = excluded.word, phonetic = excluded.phonetic,
           part_of_speech = excluded.part_of_speech, definition_zh = excluded.definition_zh, example_en = excluded.example_en,
           example_zh = excluded.example_zh, tier = excluded.tier, frequency_rank = excluded.frequency_rank,
           record_license = excluded.record_license, source_id = excluded.source_id, source_url = excluded.source_url,
           provenance_json = excluded.provenance_json, learning_ready = excluded.learning_ready",
        params![record.id, pack_id, record.word, record.phonetic, record.part_of_speech, record.definition_zh,
            record.example_en, record.example_zh, record.tier.as_str(), record.frequency_rank, record.license,
            record.source_id, record.source_url, provenance, record.learning_ready as i64],
    )?;
    let now = Utc::now().to_rfc3339();
    transaction.execute(
        "INSERT OR IGNORE INTO study_cards (id, lexeme_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![format!("card:{}", record.id), record.id, now],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_incremental_and_idempotent() {
        let mut connection = Connection::open_in_memory().unwrap();
        run_migrations(&mut connection).unwrap();
        run_migrations(&mut connection).unwrap();
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        let app_id: i64 = connection
            .pragma_query_value(None, "application_id", |row| row.get(0))
            .unwrap();
        assert_eq!(version, DATABASE_VERSION);
        assert_eq!(app_id, DATABASE_APPLICATION_ID);
    }

    #[test]
    fn default_settings_do_not_force_the_pet_on_top() {
        let mut connection = Connection::open_in_memory().unwrap();
        run_migrations(&mut connection).unwrap();
        let always_on_top: i64 = connection
            .query_row(
                "SELECT always_on_top FROM settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(always_on_top, 0);
    }

    #[test]
    fn embedded_pack_passes_fail_closed_validation() {
        let mut connection = Connection::open_in_memory().unwrap();
        run_migrations(&mut connection).unwrap();
        install_pack_json(&mut connection, BUILTIN_PACK).unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM lexemes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 100);
    }
}
