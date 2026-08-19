use crate::db::{DbState, DATABASE_VERSION};
use crate::error::{AppError, AppResult};
use crate::models::{BackupDocument, BootstrapPayload, ExportResult};
use crate::repository::bootstrap_with_connection;
use chrono::Utc;
use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::{params_from_iter, Connection};
use serde_json::{Map, Number, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

const PRODUCT_NAME: &str = "VocaPip";
const LEGACY_PRODUCT_NAME: &str = "LanguagePet";
const MAX_BACKUP_BYTES: u64 = 64 * 1024 * 1024;
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
];
const DELETE_ORDER: &[&str] = &[
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
        schema_version: 1,
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

pub fn import_data(state: &DbState, path: &Path) -> AppResult<BootstrapPayload> {
    let metadata = validate_import_path(path)?;
    if metadata.len() > MAX_BACKUP_BYTES {
        return Err(AppError::InvalidData(
            "backup exceeds the 64 MiB safety limit".into(),
        ));
    }
    let bytes = fs::read(path)?;
    let document: BackupDocument = serde_json::from_slice(&bytes)?;
    validate_document(&document)?;

    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| AppError::Resource("database write lock is poisoned".into()))?;
    let mut connection = state.connect()?;
    let transaction = connection.transaction()?;
    for table in DELETE_ORDER {
        transaction.execute(&format!("DELETE FROM {table}"), [])?;
    }
    for table in BACKUP_TABLES {
        let rows = document
            .tables
            .get(*table)
            .ok_or_else(|| AppError::InvalidData(format!("backup omits table {table}")))?;
        import_table(&transaction, table, rows)?;
    }
    ensure_singletons(&transaction)?;
    transaction.commit()?;
    bootstrap_with_connection(&connection)
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
    if document.schema_version != 1
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
    if document.tables.len() != BACKUP_TABLES.len()
        || document
            .tables
            .keys()
            .any(|table| !BACKUP_TABLES.contains(&table.as_str()))
    {
        return Err(AppError::InvalidData(
            "backup contains an unknown or missing table".into(),
        ));
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
        ValueRef::Blob(_) => Value::Null,
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
            .map(|column| json_to_sql(&object[column]))
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

fn json_to_sql(value: &Value) -> AppResult<SqlValue> {
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
        Value::Array(_) | Value::Object(_) => Err(AppError::InvalidData(
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

    #[test]
    fn export_then_import_restores_progress() {
        let directory = tempfile::tempdir().unwrap();
        let state = DbState::initialize(directory.path().join("user.sqlite")).unwrap();
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
    fn export_rejects_non_json_destination() {
        let directory = tempfile::tempdir().unwrap();
        let state = DbState::initialize(directory.path().join("user.sqlite")).unwrap();
        let result = export_data(&state, &directory.path().join("backup.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn import_rejects_relative_paths() {
        let result = validate_import_path(Path::new("backup.json"));
        assert!(result.is_err());
    }

    #[test]
    fn legacy_languagepet_backup_remains_supported() {
        let document = BackupDocument {
            schema_version: 1,
            product: LEGACY_PRODUCT_NAME.into(),
            exported_at: Utc::now().to_rfc3339(),
            database_user_version: DATABASE_VERSION,
            tables: BACKUP_TABLES
                .iter()
                .map(|table| ((*table).to_string(), Vec::new()))
                .collect(),
        };
        assert!(validate_document(&document).is_ok());
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
