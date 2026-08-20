use crate::app::{
    hide_main_window, open_or_focus_window, restore_native_settings, set_main_surface_mode,
    show_pet_context_menu as show_native_pet_context_menu, synchronize_native_settings,
    PetSurfaceMode, PetSurfaceResult, BACKUP_IMPORTED_EVENT, MAIN_WINDOW,
    PET_APPEARANCE_CHANGED_EVENT, PET_ASSETS_CHANGED_EVENT, SETTINGS_CHANGED_EVENT,
};
use crate::backup;
use crate::db::DbState;
use crate::error::{AppError, CommandResultExt};
use crate::models::*;
use crate::pet_assets;
use crate::repository;
use crate::speech;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn bootstrap(state: State<'_, DbState>) -> Result<BootstrapPayload, String> {
    repository::bootstrap(&state).command()
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_next_card(
    state: State<'_, DbState>,
    exclude_card_id: Option<String>,
) -> Result<Option<LearningCard>, String> {
    repository::get_next_card(&state, exclude_card_id.as_deref()).command()
}

#[tauri::command(rename_all = "camelCase")]
pub fn submit_review(
    state: State<'_, DbState>,
    card_id: String,
    rating: ReviewRating,
    reviewed_at: String,
    fsrs_card: FsrsCardSnapshot,
) -> Result<ReviewOutcome, String> {
    repository::submit_review(&state, &card_id, rating, &reviewed_at, &fsrs_card).command()
}

#[tauri::command]
pub fn list_library(
    state: State<'_, DbState>,
    query: LibraryQuery,
) -> Result<LibraryPayload, String> {
    repository::list_library(&state, &query).command()
}

#[tauri::command(rename_all = "camelCase")]
pub fn toggle_favorite(state: State<'_, DbState>, card_id: String) -> Result<LearningCard, String> {
    repository::toggle_favorite(&state, &card_id).command()
}

#[tauri::command(rename_all = "camelCase")]
pub fn queue_relearn(state: State<'_, DbState>, card_id: String) -> Result<LearningCard, String> {
    repository::queue_relearn(&state, &card_id).command()
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, DbState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| "保存设置失败：数据库写锁不可用".to_string())?;
    let connection = state
        .connect()
        .map_err(|error| format!("保存设置失败：读取当前设置失败：{error}"))?;
    let previous = repository::read_settings(&connection)
        .map_err(|error| format!("保存设置失败：读取当前设置失败：{error}"))?;
    repository::validate_settings_with_connection(&connection, &settings)
        .map_err(|error| format!("保存设置失败：设置校验失败：{error}"))?;
    repository::validate_settings_version(&previous, &settings)
        .map_err(|error| format!("保存设置失败：并发冲突：{error}"))?;
    synchronize_native_settings(&app, &previous, &settings)
        .map_err(|error| format!("保存设置失败：原生操作失败：{error}"))?;
    let saved = match repository::update_settings_with_connection(&connection, &settings) {
        Ok(saved) => saved,
        Err(error) => {
            let compensation = restore_native_settings(&app, &previous, &settings).err();
            return Err(match compensation {
                Some(compensation) => format!(
                    "保存设置失败：数据库写入失败：{error}；原生状态补偿失败：{compensation}"
                ),
                None => format!("保存设置失败：数据库写入失败：{error}"),
            });
        }
    };
    app.emit(SETTINGS_CHANGED_EVENT, &saved)
        .map_err(|error| format!("设置已保存，但通知其他窗口失败：{error}"))?;
    if previous.pet_appearance_id != saved.pet_appearance_id {
        app.emit(PET_APPEARANCE_CHANGED_EVENT, &saved.pet_appearance_id)
            .map_err(|error| format!("设置已保存，但通知宠物外观失败：{error}"))?;
    }
    Ok(saved)
}

#[tauri::command]
pub fn complete_placement(
    state: State<'_, DbState>,
    submission: PlacementSubmission,
) -> Result<BootstrapPayload, String> {
    repository::complete_placement(&state, &submission).command()
}

#[tauri::command]
pub async fn export_data(
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Option<ExportResult>, String> {
    let selected = app
        .dialog()
        .file()
        .add_filter("VocaPip backup", &["json"])
        .set_file_name("VocaPip-backup.json")
        .blocking_save_file();
    let Some(selected) = selected else {
        return Ok(None);
    };
    let path = selected
        .into_path()
        .map_err(|_| AppError::InvalidData("backup destination is not a local file path".into()))
        .command()?;
    backup::export_data(&state, &path).map(Some).command()
}

#[tauri::command]
pub async fn import_data(
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Option<BootstrapPayload>, String> {
    let selected = app
        .dialog()
        .file()
        .add_filter("VocaPip backup", &["json"])
        .blocking_pick_file();
    let Some(selected) = selected else {
        return Ok(None);
    };
    let path = selected
        .into_path()
        .map_err(|_| AppError::InvalidData("backup source is not a local file path".into()))
        .command()?;
    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| "导入数据失败：数据库写锁不可用".to_string())?;
    let mut connection = state
        .connect()
        .map_err(|error| format!("导入数据失败：打开数据库失败：{error}"))?;
    let previous = repository::read_settings(&connection)
        .map_err(|error| format!("导入数据失败：读取当前设置失败：{error}"))?;
    let transaction = backup::import_document_transaction(&mut connection, &path)
        .map_err(|error| format!("导入数据失败：读取或校验备份失败：{error}"))?;
    let imported = repository::read_settings(&transaction)
        .map_err(|error| format!("导入数据失败：读取导入设置失败：{error}"))?;
    if let Err(error) = synchronize_native_settings(&app, &previous, &imported) {
        return Err(format!("导入数据失败：原生状态同步失败：{error}"));
    }
    if let Err(error) = transaction.commit() {
        let compensation = restore_native_settings(&app, &previous, &imported).err();
        return Err(match compensation {
            Some(compensation) => {
                format!("导入数据失败：数据库提交失败：{error}；原生状态补偿失败：{compensation}")
            }
            None => format!("导入数据失败：数据库提交失败：{error}"),
        });
    }
    let bootstrap = repository::bootstrap_with_connection(&connection)
        .map_err(|error| format!("导入数据失败：重新加载数据失败：{error}"))?;
    app.emit(BACKUP_IMPORTED_EVENT, &bootstrap)
        .map_err(|error| format!("备份已导入，但通知其他窗口失败：{error}"))?;
    app.emit(SETTINGS_CHANGED_EVENT, &bootstrap.settings)
        .map_err(|error| format!("备份已导入，但通知设置窗口失败：{error}"))?;
    app.emit(PET_ASSETS_CHANGED_EVENT, serde_json::json!({}))
        .map_err(|error| format!("备份已导入，但通知图片窗口失败：{error}"))?;
    app.emit(
        PET_APPEARANCE_CHANGED_EVENT,
        &bootstrap.settings.pet_appearance_id,
    )
    .map_err(|error| format!("备份已导入，但通知宠物外观失败：{error}"))?;
    Ok(Some(bootstrap))
}

#[tauri::command]
pub fn open_window(app: AppHandle, view: String) -> Result<(), String> {
    open_or_focus_window(&app, &view).command()
}

#[tauri::command]
pub fn hide_main(app: AppHandle) -> Result<(), String> {
    hide_main_window(&app).command()
}

#[tauri::command]
pub fn set_pet_surface_mode(
    app: AppHandle,
    mode: PetSurfaceMode,
) -> Result<PetSurfaceResult, String> {
    set_main_surface_mode(&app, mode).command()
}

#[tauri::command]
pub fn show_pet_context_menu(app: AppHandle) -> Result<(), String> {
    show_native_pet_context_menu(&app).command()
}

#[tauri::command]
pub fn set_always_on_top(
    app: AppHandle,
    state: State<'_, DbState>,
    enabled: bool,
) -> Result<bool, String> {
    let persisted = crate::app::persist_always_on_top(&app, &state, enabled).command()?;
    app.emit_to(
        MAIN_WINDOW,
        "vocapip://pet-action",
        serde_json::json!({ "action": "always-on-top", "enabled": persisted }),
    )
    .map_err(|error| error.to_string())?;
    app.emit(
        SETTINGS_CHANGED_EVENT,
        serde_json::json!({ "alwaysOnTop": persisted }),
    )
    .map_err(|error| error.to_string())?;
    Ok(persisted)
}

#[tauri::command]
pub fn set_reminders_paused(
    app: AppHandle,
    state: State<'_, DbState>,
    paused: bool,
) -> Result<bool, String> {
    let persisted =
        repository::set_boolean_setting(&state, "reminders_paused", paused).command()?;
    app.emit_to(
        MAIN_WINDOW,
        "vocapip://pet-action",
        serde_json::json!({ "action": "reminders-paused", "paused": persisted }),
    )
    .map_err(|error| error.to_string())?;
    app.emit(
        SETTINGS_CHANGED_EVENT,
        serde_json::json!({ "remindersPaused": persisted }),
    )
    .map_err(|error| error.to_string())?;
    Ok(persisted)
}

#[tauri::command]
pub fn set_quiet_mode_until(
    app: AppHandle,
    state: State<'_, DbState>,
    until: Option<String>,
) -> Result<Option<String>, String> {
    let persisted = repository::set_quiet_until(&state, until.as_deref()).command()?;
    app.emit(
        SETTINGS_CHANGED_EVENT,
        serde_json::json!({
            "quietModeUntil": persisted,
        }),
    )
    .map_err(|error| error.to_string())?;
    Ok(persisted)
}

#[tauri::command]
pub fn get_license_documents() -> Vec<LicenseDocument> {
    vec![
        LicenseDocument {
            id: "application".into(),
            title: "VocaPip application".into(),
            body: include_str!("../resources/licenses/application.txt").into(),
        },
        LicenseDocument {
            id: "language-pack".into(),
            title: "Bundled language pack".into(),
            body: include_str!("../resources/licenses/language-pack.txt").into(),
        },
        LicenseDocument {
            id: "third-party".into(),
            title: "Third-party software".into(),
            body: include_str!("../resources/licenses/third-party.txt").into(),
        },
    ]
}

#[tauri::command(rename_all = "camelCase")]
pub fn speak_text(text: String, language: String, rate: f64) -> Result<(), String> {
    speech::speak_text(text, language, rate).command()
}

#[tauri::command]
pub fn stop_speaking() -> Result<(), String> {
    speech::stop_speaking().command()
}

#[tauri::command]
pub fn list_pet_appearances(
    state: State<'_, DbState>,
) -> Result<Vec<PetAppearanceSummary>, String> {
    pet_assets::list_pet_appearances(&state).command()
}

#[tauri::command(rename_all = "camelCase")]
pub fn import_pet_image(
    app: AppHandle,
    state: State<'_, DbState>,
    asset_state: PetAssetState,
    data_url: String,
) -> Result<PetAsset, String> {
    let saved = pet_assets::import_pet_image(&state, asset_state, &data_url).command()?;
    app.emit(
        PET_ASSETS_CHANGED_EVENT,
        serde_json::json!({ "state": asset_state }),
    )
    .map_err(|error| error.to_string())?;
    if asset_state == PetAssetState::Default {
        app.emit(PET_APPEARANCE_CHANGED_EVENT, "custom")
            .map_err(|error| error.to_string())?;
    }
    Ok(saved)
}

#[tauri::command(rename_all = "camelCase")]
pub fn remove_pet_image(
    app: AppHandle,
    state: State<'_, DbState>,
    asset_state: PetAssetState,
) -> Result<String, String> {
    let appearance = pet_assets::remove_pet_image(&state, asset_state).command()?;
    app.emit(
        PET_ASSETS_CHANGED_EVENT,
        serde_json::json!({ "state": asset_state }),
    )
    .map_err(|error| error.to_string())?;
    app.emit(PET_APPEARANCE_CHANGED_EVENT, &appearance)
        .map_err(|error| error.to_string())?;
    Ok(appearance)
}

#[tauri::command]
pub fn clear_pet_images(app: AppHandle, state: State<'_, DbState>) -> Result<String, String> {
    let appearance = pet_assets::clear_pet_images(&state).command()?;
    app.emit(
        PET_ASSETS_CHANGED_EVENT,
        serde_json::json!({ "cleared": true }),
    )
    .map_err(|error| error.to_string())?;
    app.emit(PET_APPEARANCE_CHANGED_EVENT, &appearance)
        .map_err(|error| error.to_string())?;
    Ok(appearance)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_pet_asset(
    state: State<'_, DbState>,
    asset_state: PetAssetState,
) -> Result<Option<PetAsset>, String> {
    pet_assets::get_pet_asset(&state, asset_state).command()
}

#[tauri::command(rename_all = "camelCase")]
pub fn set_pet_appearance(
    app: AppHandle,
    state: State<'_, DbState>,
    appearance_id: String,
) -> Result<String, String> {
    let appearance = pet_assets::set_pet_appearance(&state, &appearance_id).command()?;
    app.emit(PET_APPEARANCE_CHANGED_EVENT, &appearance)
        .map_err(|error| error.to_string())?;
    Ok(appearance)
}
