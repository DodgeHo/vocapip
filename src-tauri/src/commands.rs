use crate::app::{
    hide_main_window, open_or_focus_window, set_main_always_on_top, set_main_surface_mode,
    show_pet_context_menu as show_native_pet_context_menu, PetSurfaceMode, PetSurfaceResult,
    MAIN_WINDOW,
};
use crate::backup;
use crate::db::DbState;
use crate::error::{AppError, CommandResultExt};
use crate::models::*;
use crate::pet_assets;
use crate::repository;
use crate::speech;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt;
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
    let autostart = app.autolaunch();
    let autostart_result = if settings.autostart {
        autostart.enable()
    } else {
        autostart.disable()
    };
    autostart_result
        .map_err(|error| AppError::Resource(format!("unable to update autostart: {error}")))
        .command()?;
    set_main_always_on_top(&app, settings.always_on_top).command()?;
    repository::update_settings(&state, &settings).command()
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
    backup::import_data(&state, &path).map(Some).command()
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
    set_main_always_on_top(&app, enabled).command()?;
    let persisted = repository::set_boolean_setting(&state, "always_on_top", enabled).command()?;
    app.emit_to(
        MAIN_WINDOW,
        "vocapip://pet-action",
        serde_json::json!({ "action": "always-on-top", "enabled": persisted }),
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
    Ok(persisted)
}

#[tauri::command]
pub fn set_quiet_mode_until(
    state: State<'_, DbState>,
    until: Option<String>,
) -> Result<Option<String>, String> {
    repository::set_quiet_until(&state, until.as_deref()).command()
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
    state: State<'_, DbState>,
    asset_state: PetAssetState,
    data_url: String,
) -> Result<PetAsset, String> {
    pet_assets::import_pet_image(&state, asset_state, &data_url).command()
}

#[tauri::command(rename_all = "camelCase")]
pub fn remove_pet_image(
    state: State<'_, DbState>,
    asset_state: PetAssetState,
) -> Result<(), String> {
    pet_assets::remove_pet_image(&state, asset_state).command()
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
    state: State<'_, DbState>,
    appearance_id: String,
) -> Result<String, String> {
    pet_assets::set_pet_appearance(&state, &appearance_id).command()
}
