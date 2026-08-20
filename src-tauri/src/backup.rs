use crate::db::{DbState, DATABASE_VERSION};
use crate::error::{AppError, AppResult};
#[cfg(test)]
use crate::models::BootstrapPayload;
use crate::models::{BackupDocument, ExportResult};
#[cfg(test)]
use crate::repository::bootstrap_with_connection;
use crate::repository::{read_settings, validate_settings_with_connection};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::{params_from_iter, Connection, Transaction};
use serde_json::{Map, Number, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

const PRODUCT_NAME: &str = "VocaPip";
const LEGACY_PRODUCT_NAME: &str = "LanguagePet";
const BACKUP_SCHEMA_VERSION: u32 = 2;
const MAX_BACKUP_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PET_ASSET_BYTES: usize = 5 * 1024 * 1024;
const LEGACY_BACKUP_TABLES: &[&str] = &[
    "study_cards",
    "review_logs",
    "learner_profile",
    "placement_attempts",
    "user_progress",
    "settings",
    "favorites",
    "relearn_queue",
    "daily_stats",
    "learning_records",
    "app_runtime",
];
const BACKUP_TABLES: &[&str] = &[
    "study_cards",
    "review_logs",
    "learner_profile",
    "placement_attempts",
    "user_progress",
    "settings",
    "favorites",
    "relearn_queue",
    "daily_stats",
    "learning_records",
    "app_runtime",
    "pet_assets",
];
const DELETE_ORDER: &[&str] = &[
    "pet_assets",
    "review_logs",
    "favorites",
    "relearn_queue",
    "learning_records",
    "daily_stats",
    "placement_attempts",
    "study_cards",
    "learner_profile",
    "user_progress",
    "settings",
    "app_runtime",
];

pub fn export_data(state: &DbState, path: &Path) -> AppResult<ExportResult> {
    validate_export_path(path)?;
    let connection = state.connect()?;
    let mut tables = BTreeMap::new();
    for table in BACKUP_TABLES {
        tables.insert((*table).to_string(), export_table(&connection, table)?);
    }
    let document = BackupDocument {
        schema_version: BACKUP_SCHEMA_VERSION,
        product: PRODUCT_NAME.into(),
        exported_at: Utc::now().to_rfc3339(),
        database_user_version: DATABASE_VERSION,
        tables,
    };
    let bytes = serde_json::to_vec_pretty(&document)?;
    if bytes.len() as u64 > MAX_BACKUP_BYTES {
        return Err(AppError::InvalidData(
            "backup exceeds the 64 MiB safety limit".into(),
        ));
    }

    let temporary = path.with_extension(format!("json.{}.tmp", Uuid::new_v4()));
    let write_result = (|| -> AppResult<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result?;

    Ok(ExportResult {
        path: path.to_string_lossy().into_owned(),
    })
}

#[cfg(test)]
pub fn import_data(state: &DbState, path: &Path) -> AppResult<BootstrapPayload> {
    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| AppError::Resource("database write lock is poisoned".into()))?;
    let mut connection = state.connect()?;
    let transaction = import_document_transaction(&mut connection, path)?;
    transaction.commit()?;
    bootstrap_with_connection(&connection)
}

pub fn import_document_transaction<'a>(
    connection: &'a mut Connection,
    path: &Path,
) -> AppResult<Transaction<'a>> {
    let metadata = validate_import_path(path)?;
    if metadata.len() > MAX_BACKUP_BYTES {
        return Err(AppError::InvalidData(
            "backup exceeds the 64 MiB safety limit".into(),
        ));
    }
    let bytes = fs::read(path)?;
    let document: BackupDocument = serde_json::from_slice(&bytes)?;
    validate_document(&document)?;

    let transaction = connection.transaction()?;
    for table in DELETE_ORDER {
        transaction.execute(&format!("DELETE FROM {table}"), [])?;
    }
    for table in BACKUP_TABLES {
        if let Some(rows) = document.tables.get(*table) {
            import_table(&transaction, table, rows)?;
        }
    }
    ensure_singletons(&transaction)?;
    let imported_settings = read_settings(&transaction)?;
    validate_settings_with_connection(&transaction, &imported_settings)?;
    Ok(transaction)
}

fn validate_export_path(path: &Path) -> AppResult<()> {
    validate_backup_path(path)?;
    let parent = path.parent().ok_or_else(|| {
        AppError::InvalidData("backup destination has no parent directory".into())
    })?;
    if !parent.is_dir() {
        return Err(AppError::InvalidData(
            "backup destination directory does not exist".into(),
        ));
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(AppError::InvalidData(
            "backup destination cannot be a symbolic link".into(),
        )),
        Ok(metadata) if !metadata.is_file() => Err(AppError::InvalidData(
            "backup destination must be a regular file".into(),
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn validate_import_path(path: &Path) -> AppResult<fs::Metadata> {
    validate_backup_path(path)?;
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AppError::InvalidData(
            "backup source must be a regular file".into(),
        ));
    }
    Ok(metadata)
}

fn validate_backup_path(path: &Path) -> AppResult<()> {
    if !path.is_absolute() {
        return Err(AppError::InvalidData("backup path must be absolute".into()));
    }
    let is_json = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"));
    if !is_json {
        return Err(AppError::InvalidData(
            "backup file must use the .json extension".into(),
        ));
    }
    Ok(())
}

fn validate_document(document: &BackupDocument) -> AppResult<()> {
    if ![1, BACKUP_SCHEMA_VERSION].contains(&document.schema_version)
        || ![PRODUCT_NAME, LEGACY_PRODUCT_NAME].contains(&document.product.as_str())
    {
        return Err(AppError::InvalidData(
            "file is not a supported VocaPip backup".into(),
        ));
    }
    if document.database_user_version > DATABASE_VERSION {
        return Err(AppError::InvalidData(format!(
            "backup schema {} is newer than supported schema {DATABASE_VERSION}",
            document.database_user_version
        )));
    }
    let expected_tables = if document.schema_version == 1 {
        LEGACY_BACKUP_TABLES
    } else {
        BACKUP_TABLES
    };
    if document.tables.len() != expected_tables.len()
        || document
            .tables
            .keys()
            .any(|table| !expected_tables.contains(&table.as_str()))
    {
        return Err(AppError::InvalidData(
            "backup contains an unknown or missing table".into(),
        ));
    }
    if document.schema_version == BACKUP_SCHEMA_VERSION {
        let custom_selected = document.tables.get("settings").is_some_and(|rows| {
            rows.iter()
                .any(|row| row.get("pet_appearance_id").and_then(Value::as_str) == Some("custom"))
        });
        let has_default = document.tables.get("pet_assets").is_some_and(|rows| {
            rows.iter().any(|row| {
                row.get("appearance_id").and_then(Value::as_str) == Some("custom")
                    && row.get("state").and_then(Value::as_str) == Some("default")
            })
        });
        if custom_selected && !has_default {
            return Err(AppError::InvalidData(
                "custom pet appearance requires a default image in the backup".into(),
            ));
        }
    }
    Ok(())
}

fn export_table(connection: &Connection, table: &str) -> AppResult<Vec<Value>> {
    let mut statement = connection.prepare(&format!("SELECT * FROM {table}"))?;
    let column_names = statement
        .column_names()
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();
    let rows = statement.query_map([], |row| {
        let mut object = Map::new();
        for (index, name) in column_names.iter().enumerate() {
            object.insert(name.clone(), sql_to_json(row.get_ref(index)?));
        }
        Ok(Value::Object(object))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn sql_to_json(value: ValueRef<'_>) -> Value {
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(value) => Value::Number(value.into()),
        ValueRef::Real(value) => Number::from_f64(value)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        ValueRef::Text(value) => Value::String(String::from_utf8_lossy(value).into_owned()),
        ValueRef::Blob(value) => serde_json::json!({
            "$base64": BASE64.encode(value),
        }),
    }
}

fn import_table(
    transaction: &rusqlite::Transaction<'_>,
    table: &str,
    rows: &[Value],
) -> AppResult<()> {
    let allowed_columns = table_columns(transaction, table)?;
    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| AppError::InvalidData(format!("{table} row is not an object")))?;
        if object.is_empty() || object.keys().any(|key| !allowed_columns.contains(key)) {
            return Err(AppError::InvalidData(format!(
                "{table} row contains an invalid column"
            )));
        }
        let columns = object.keys().cloned().collect::<Vec<_>>();
        let placeholders = (1..=columns.len())
            .map(|index| format!("?{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "INSERT INTO {table} ({}) VALUES ({placeholders})",
            columns.join(", ")
        );
        let values = columns
            .iter()
            .map(|column| json_to_sql(table, column, &object[column]))
            .collect::<AppResult<Vec<_>>>()?;
        transaction.execute(&sql, params_from_iter(values))?;
    }
    Ok(())
}

fn table_columns(connection: &Connection, table: &str) -> AppResult<HashSet<String>> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<HashSet<_>, _>>()?;
    Ok(columns)
}

fn json_to_sql(table: &str, column: &str, value: &Value) -> AppResult<SqlValue> {
    match value {
        Value::Null => Ok(SqlValue::Null),
        Value::Bool(value) => Ok(SqlValue::Integer(i64::from(*value))),
        Value::Number(value) => {
            if let Some(integer) = value.as_i64() {
                Ok(SqlValue::Integer(integer))
            } else if let Some(float) = value.as_f64() {
                Ok(SqlValue::Real(float))
            } else {
                Err(AppError::InvalidData("unsupported numeric value".into()))
            }
        }
        Value::String(value) => Ok(SqlValue::Text(value.clone())),
        Value::Object(object) => {
            if table != "pet_assets" || column != "bytes" {
                return Err(AppError::InvalidData(format!(
                    "{table}.{column} contains unsupported nested JSON"
                )));
            }
            if object.len() != 1 || !object.contains_key("$base64") {
                return Err(AppError::InvalidData(format!(
                    "{table}.{column} contains an invalid binary value"
                )));
            }
            let encoded = object["$base64"].as_str().ok_or_else(|| {
                AppError::InvalidData(format!("{table}.{column} binary value is not text"))
            })?;
            let bytes = BASE64.decode(encoded).map_err(|_| {
                AppError::InvalidData(format!("{table}.{column} contains invalid base64"))
            })?;
            if bytes.is_empty() || bytes.len() > MAX_PET_ASSET_BYTES {
                return Err(AppError::InvalidData(format!(
                    "{table}.{column} binary value exceeds the supported size"
                )));
            }
            Ok(SqlValue::Blob(bytes))
        }
        Value::Array(_) => Err(AppError::InvalidData(
            "nested JSON is not valid in a database cell".into(),
        )),
    }
}

fn ensure_singletons(connection: &Connection) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT OR IGNORE INTO learner_profile (id, updated_at) VALUES (1, ?1)",
        [&now],
    )?;
    connection.execute(
        "INSERT OR IGNORE INTO user_progress (id, updated_at) VALUES (1, ?1)",
        [&now],
    )?;
    connection.execute(
        "INSERT OR IGNORE INTO settings (id, updated_at) VALUES (1, ?1)",
        [&now],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbState;

    fn test_state(path: std::path::PathBuf) -> DbState {
        DbState::initialize(path).unwrap()
    }

    #[test]
    fn export_then_import_restores_progress() {
        let directory = tempfile::tempdir().unwrap();
        let state = test_state(directory.path().join("user.sqlite"));
        state
            .connect()
            .unwrap()
            .execute("UPDATE user_progress SET total_xp = 123", [])
            .unwrap();
        let backup = directory.path().join("backup.json");
        export_data(&state, &backup).unwrap();
        state
            .connect()
            .unwrap()
            .execute("UPDATE user_progress SET total_xp = 0", [])
            .unwrap();
        let restored = import_data(&state, &backup).unwrap();
        assert_eq!(restored.progress.xp, 123);
    }

    #[test]
    fn export_then_import_restores_speech_pet_settings_and_asset_blob() {
        let directory = tempfile::tempdir().unwrap();
        let state = test_state(directory.path().join("user.sqlite"));
        let asset_bytes = vec![0_u8, 1, 2, 3, 250, 255];
        state
            .connect()
            .unwrap()
            .execute(
                "UPDATE settings SET speech_enabled = 1, pet_appearance_id = 'custom' WHERE id = 1",
                [],
            )
            .unwrap();
        state
            .connect()
            .unwrap()
            .execute(
                "INSERT INTO pet_assets (id, appearance_id, state, mime_type, width, height, bytes, created_at, updated_at)
                 VALUES ('asset-1', 'custom', 'default', 'image/png', 2, 3, ?1, '2026-08-19T00:00:00Z', '2026-08-19T00:00:00Z')",
                [&asset_bytes],
            )
            .unwrap();

        let backup = directory.path().join("backup.json");
        export_data(&state, &backup).unwrap();
        let document: BackupDocument =
            serde_json::from_slice(&std::fs::read(&backup).unwrap()).unwrap();
        assert_eq!(document.schema_version, BACKUP_SCHEMA_VERSION);
        let encoded = &document.tables["pet_assets"][0]["bytes"]["$base64"];
        assert_eq!(encoded, &Value::String(BASE64.encode(&asset_bytes)));

        state
            .connect()
            .unwrap()
            .execute(
                "UPDATE settings SET speech_enabled = 0, pet_appearance_id = 'mint' WHERE id = 1",
                [],
            )
            .unwrap();
        state
            .connect()
            .unwrap()
            .execute("DELETE FROM pet_assets", [])
            .unwrap();

        import_data(&state, &backup).unwrap();
        let connection = state.connect().unwrap();
        let settings: (i64, String) = connection
            .query_row(
                "SELECT speech_enabled, pet_appearance_id FROM settings WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(settings, (1, "custom".to_owned()));
        let restored_bytes: Vec<u8> = connection
            .query_row(
                "SELECT bytes FROM pet_assets WHERE id = 'asset-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(restored_bytes, asset_bytes);
    }

    #[test]
    fn export_rejects_non_json_destination() {
        let directory = tempfile::tempdir().unwrap();
        let state = test_state(directory.path().join("user.sqlite"));
        let result = export_data(&state, &directory.path().join("backup.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn import_rejects_relative_paths() {
        let result = validate_import_path(Path::new("backup.json"));
        assert!(result.is_err());
    }

    #[test]
    fn malformed_import_keeps_existing_database_unchanged() {
        let directory = tempfile::tempdir().unwrap();
        let state = test_state(directory.path().join("user.sqlite"));
        state
            .connect()
            .unwrap()
            .execute("UPDATE user_progress SET total_xp = 321", [])
            .unwrap();
        let backup = directory.path().join("broken.json");
        let mut document: BackupDocument = BackupDocument {
            schema_version: BACKUP_SCHEMA_VERSION,
            product: PRODUCT_NAME.into(),
            exported_at: Utc::now().to_rfc3339(),
            database_user_version: DATABASE_VERSION,
            tables: BACKUP_TABLES
                .iter()
                .map(|table| ((*table).to_string(), Vec::new()))
                .collect(),
        };
        document.tables.insert(
            "user_progress".into(),
            vec![serde_json::json!({ "not_a_column": 1 })],
        );
        std::fs::write(&backup, serde_json::to_vec(&document).unwrap()).unwrap();

        assert!(import_data(&state, &backup).is_err());
        let xp: i64 = state
            .connect()
            .unwrap()
            .query_row(
                "SELECT total_xp FROM user_progress WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(xp, 321);
    }

    #[test]
    fn custom_backup_without_default_image_is_rejected() {
        let document = BackupDocument {
            schema_version: BACKUP_SCHEMA_VERSION,
            product: PRODUCT_NAME.into(),
            exported_at: Utc::now().to_rfc3339(),
            database_user_version: DATABASE_VERSION,
            tables: BACKUP_TABLES
                .iter()
                .map(|table| ((*table).to_string(), Vec::new()))
                .collect(),
        };
        let mut document = document;
        document.tables.insert(
            "settings".into(),
            vec![serde_json::json!({ "pet_appearance_id": "custom" })],
        );
        assert!(validate_document(&document).is_err());
    }

    #[test]
    fn legacy_languagepet_backup_remains_supported() {
        let document = BackupDocument {
            schema_version: 1,
            product: LEGACY_PRODUCT_NAME.into(),
            exported_at: Utc::now().to_rfc3339(),
            database_user_version: DATABASE_VERSION,
            tables: LEGACY_BACKUP_TABLES
                .iter()
                .map(|table| ((*table).to_string(), Vec::new()))
                .collect(),
        };
        assert!(validate_document(&document).is_ok());
    }

    #[test]
    fn legacy_backup_without_new_settings_and_assets_is_accepted() {
        let document = BackupDocument {
            schema_version: 1,
            product: PRODUCT_NAME.into(),
            exported_at: Utc::now().to_rfc3339(),
            database_user_version: 4,
            tables: LEGACY_BACKUP_TABLES
                .iter()
                .map(|table| ((*table).to_string(), Vec::new()))
                .collect(),
        };
        assert!(validate_document(&document).is_ok());
    }

    #[test]
    fn malformed_pet_asset_binary_value_is_rejected() {
        let value = serde_json::json!({"$base64": "not valid base64%%%"});
        assert!(json_to_sql("pet_assets", "bytes", &value).is_err());
    }

    #[test]
    fn binary_values_are_not_accepted_in_non_blob_columns() {
        let value = serde_json::json!({"$base64": BASE64.encode([1_u8, 2, 3])});
        assert!(json_to_sql("settings", "speech_enabled", &value).is_err());
    }

    #[test]
    fn unknown_tables_fail_closed() {
        let mut document = BackupDocument {
            schema_version: 1,
            product: PRODUCT_NAME.into(),
            exported_at: Utc::now().to_rfc3339(),
            database_user_version: DATABASE_VERSION,
            tables: BTreeMap::new(),
        };
        document.tables.insert("scripts".into(), Vec::new());
        assert!(validate_document(&document).is_err());
    }
}
