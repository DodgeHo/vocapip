use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::models::{PetAppearanceSummary, PetAsset, PetAssetState};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Utc;
use image::GenericImageView;
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

const MAX_BYTES: usize = 5 * 1024 * 1024;
const SYSTEM_APPEARANCES: [&str; 4] = ["mint", "coral", "sun", "night"];

pub fn list_pet_appearances(state: &DbState) -> AppResult<Vec<PetAppearanceSummary>> {
    let connection = state.connect()?;
    let selected: String = connection.query_row(
        "SELECT pet_appearance_id FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    let mut result = SYSTEM_APPEARANCES
        .iter()
        .map(|id| PetAppearanceSummary {
            id: (*id).into(),
            bundled: true,
            selected: selected == *id,
            available_states: vec![
                PetAssetState::Default,
                PetAssetState::Idle,
                PetAssetState::Click,
                PetAssetState::Error,
                PetAssetState::Correct,
                PetAssetState::Upgrade,
            ],
        })
        .collect::<Vec<_>>();
    let states = custom_states(&connection)?;
    result.push(PetAppearanceSummary {
        id: "custom".into(),
        bundled: false,
        selected: selected == "custom",
        available_states: states,
    });
    Ok(result)
}

pub fn import_pet_image(
    state: &DbState,
    asset_state: PetAssetState,
    data_url: &str,
) -> AppResult<PetAsset> {
    let (mime_type, bytes) = decode_data_url(data_url)?;
    if bytes.len() > MAX_BYTES {
        return Err(AppError::InvalidData("pet image exceeds 5 MiB".into()));
    }
    let image = image::load_from_memory(&bytes)
        .map_err(|_| AppError::InvalidData("pet image is not a valid PNG, JPEG, or WebP".into()))?;
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return Err(AppError::InvalidData(
            "pet image dimensions must be 1..4096".into(),
        ));
    }
    let state_name = asset_state.as_str();
    let now = Utc::now().to_rfc3339();
    let id = format!("pet-custom-{state_name}-{}", Uuid::new_v4());
    let connection = state.connect()?;
    connection.execute(
        "INSERT INTO pet_assets (id, appearance_id, state, mime_type, width, height, bytes, created_at, updated_at)
         VALUES (?1, 'custom', ?2, ?3, ?4, ?5, ?6, ?7, ?7)
         ON CONFLICT(appearance_id, state) DO UPDATE SET id = excluded.id, mime_type = excluded.mime_type,
           width = excluded.width, height = excluded.height, bytes = excluded.bytes, updated_at = excluded.updated_at",
        params![id, state_name, mime_type, width, height, bytes, now],
    )?;
    read_pet_asset(&connection, asset_state)?
        .ok_or_else(|| AppError::Resource("saved pet image could not be read".into()))
}

pub fn remove_pet_image(state: &DbState, asset_state: PetAssetState) -> AppResult<()> {
    let connection = state.connect()?;
    connection.execute(
        "DELETE FROM pet_assets WHERE appearance_id = 'custom' AND state = ?1",
        [asset_state.as_str()],
    )?;
    let selected: String = connection.query_row(
        "SELECT pet_appearance_id FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    if selected == "custom" && read_pet_asset(&connection, PetAssetState::Default)?.is_none() {
        connection.execute(
            "UPDATE settings SET pet_appearance_id = 'mint', updated_at = ?1 WHERE id = 1",
            [Utc::now().to_rfc3339()],
        )?;
    }
    Ok(())
}

pub fn get_pet_asset(state: &DbState, asset_state: PetAssetState) -> AppResult<Option<PetAsset>> {
    let connection = state.connect()?;
    read_pet_asset(&connection, asset_state)
}

pub fn set_pet_appearance(state: &DbState, appearance_id: &str) -> AppResult<String> {
    if !SYSTEM_APPEARANCES.contains(&appearance_id) && appearance_id != "custom" {
        return Err(AppError::InvalidData("unknown pet appearance".into()));
    }
    let connection = state.connect()?;
    if appearance_id == "custom" && read_pet_asset(&connection, PetAssetState::Default)?.is_none() {
        return Err(AppError::InvalidData(
            "upload a default custom pet image first".into(),
        ));
    }
    connection.execute(
        "UPDATE settings SET pet_appearance_id = ?1, updated_at = ?2 WHERE id = 1",
        params![appearance_id, Utc::now().to_rfc3339()],
    )?;
    Ok(appearance_id.into())
}

fn read_pet_asset(
    connection: &rusqlite::Connection,
    asset_state: PetAssetState,
) -> AppResult<Option<PetAsset>> {
    connection
        .query_row(
            "SELECT id, appearance_id, state, mime_type, width, height, bytes, updated_at
             FROM pet_assets WHERE appearance_id = 'custom' AND state = ?1",
            [asset_state.as_str()],
            |row| {
                let state: String = row.get(2)?;
                let mime_type: String = row.get(3)?;
                let bytes: Vec<u8> = row.get(6)?;
                Ok(PetAsset {
                    id: row.get(0)?,
                    appearance_id: row.get(1)?,
                    state: PetAssetState::try_from(state.as_str()).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, error)),
                        )
                    })?,
                    mime_type: mime_type.clone(),
                    width: row.get(4)?,
                    height: row.get(5)?,
                    data_url: format!("data:{mime_type};base64,{}", STANDARD.encode(bytes)),
                    updated_at: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(AppError::from)
}

fn custom_states(connection: &rusqlite::Connection) -> AppResult<Vec<PetAssetState>> {
    let mut statement = connection
        .prepare("SELECT state FROM pet_assets WHERE appearance_id = 'custom' ORDER BY state")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut states = Vec::new();
    for row in rows {
        states.push(PetAssetState::try_from(row?.as_str()).map_err(AppError::InvalidData)?);
    }
    Ok(states)
}

fn decode_data_url(data_url: &str) -> AppResult<(&str, Vec<u8>)> {
    let (header, encoded) = data_url
        .split_once(",")
        .ok_or_else(|| AppError::InvalidData("pet image must be a data URL".into()))?;
    let mime_type = header
        .strip_prefix("data:")
        .and_then(|value| value.strip_suffix(";base64"))
        .ok_or_else(|| AppError::InvalidData("pet image data URL must be base64".into()))?;
    if !matches!(mime_type, "image/png" | "image/jpeg" | "image/webp") {
        return Err(AppError::InvalidData(
            "only PNG, JPEG, and WebP pet images are supported".into(),
        ));
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| AppError::InvalidData("pet image base64 is invalid".into()))?;
    let detected = image::guess_format(&bytes)
        .map_err(|_| AppError::InvalidData("pet image format could not be detected".into()))?;
    let expected = match mime_type {
        "image/png" => image::ImageFormat::Png,
        "image/jpeg" => image::ImageFormat::Jpeg,
        _ => image::ImageFormat::WebP,
    };
    if detected != expected {
        return Err(AppError::InvalidData(
            "pet image MIME type does not match its bytes".into(),
        ));
    }
    Ok((mime_type, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbState;
    use image::{DynamicImage, ImageFormat, RgbaImage};
    use std::io::Cursor;

    fn data_url() -> String {
        let image = DynamicImage::ImageRgba8(RgbaImage::new(4, 4));
        let mut bytes = Cursor::new(Vec::new());
        image.write_to(&mut bytes, ImageFormat::Png).unwrap();
        format!(
            "data:image/png;base64,{}",
            STANDARD.encode(bytes.into_inner())
        )
    }

    #[test]
    fn saves_reads_and_removes_custom_asset() {
        let dir = tempfile::tempdir().unwrap();
        let state = DbState::initialize(dir.path().join("test.sqlite3")).unwrap();
        let saved = import_pet_image(&state, PetAssetState::Default, &data_url()).unwrap();
        assert_eq!(saved.state, PetAssetState::Default);
        let saved = import_pet_image(&state, PetAssetState::Idle, &data_url()).unwrap();
        assert_eq!(saved.mime_type, "image/png");
        assert!(get_pet_asset(&state, PetAssetState::Idle)
            .unwrap()
            .is_some());
        remove_pet_image(&state, PetAssetState::Idle).unwrap();
        assert!(get_pet_asset(&state, PetAssetState::Idle)
            .unwrap()
            .is_none());
    }
}
