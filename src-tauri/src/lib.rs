mod app;
mod backup;
mod commands;
mod db;
mod error;
mod models;
mod pet_assets;
mod repository;
mod speech;
mod xp;

use app::{LifecycleState, PetSurfaceMode, PetSurfaceState, MAIN_WINDOW};
use db::DbState;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::ManagerExt;

const DATABASE_FILENAME: &str = "vocapip.sqlite3";
const LEGACY_IDENTIFIER: &str = "com.languagepet.app";
const LEGACY_DATABASE_FILENAME: &str = "languagepet.sqlite3";

fn prepare_database_path(data_dir: &Path) -> std::io::Result<PathBuf> {
    fs::create_dir_all(data_dir)?;
    let database_path = data_dir.join(DATABASE_FILENAME);
    if database_path.exists() {
        return Ok(database_path);
    }

    if let Some(app_data) = std::env::var_os("APPDATA") {
        let legacy_dir = PathBuf::from(app_data).join(LEGACY_IDENTIFIER);
        migrate_legacy_database(&legacy_dir, &database_path)?;
    }
    Ok(database_path)
}

fn migrate_legacy_database(legacy_dir: &Path, database_path: &Path) -> std::io::Result<bool> {
    let source = legacy_dir.join(LEGACY_DATABASE_FILENAME);
    if database_path.exists() || !source.is_file() {
        return Ok(false);
    }

    fs::copy(&source, database_path)?;
    for suffix in ["-wal", "-shm"] {
        let source_sidecar = legacy_dir.join(format!("{LEGACY_DATABASE_FILENAME}{suffix}"));
        if source_sidecar.is_file() {
            let target_sidecar = PathBuf::from(format!("{}{suffix}", database_path.display()));
            if let Err(error) = fs::copy(source_sidecar, target_sidecar) {
                let _ = fs::remove_file(database_path);
                return Err(error);
            }
        }
    }
    Ok(true)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(LifecycleState::default())
        .manage(PetSurfaceState::default())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app::show_main_window(app);
        }))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .app_name("VocaPip")
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .on_menu_event(app::handle_pet_menu_event)
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let database_path = prepare_database_path(&data_dir)?;
            let state = DbState::initialize(database_path)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            let bootstrap = repository::bootstrap(&state)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            let startup_mode = if bootstrap.placement_complete {
                PetSurfaceMode::Idle
            } else {
                PetSurfaceMode::Placement
            };
            let settings = bootstrap.settings;
            app.manage(state);
            app::set_main_always_on_top(app.handle(), settings.always_on_top)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            app::set_main_surface_mode(app.handle(), startup_mode)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            let autostart = app.autolaunch();
            let _ = if settings.autostart {
                autostart.enable()
            } else {
                autostart.disable()
            };
            app::setup_tray(app).map_err(|error| std::io::Error::other(error.to_string()))?;
            app::start_reminder_loop(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != MAIN_WINDOW {
                return;
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                let exiting = window
                    .app_handle()
                    .state::<LifecycleState>()
                    .exiting
                    .load(Ordering::SeqCst);
                if !exiting {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::get_next_card,
            commands::submit_review,
            commands::list_library,
            commands::toggle_favorite,
            commands::queue_relearn,
            commands::update_settings,
            commands::complete_placement,
            commands::export_data,
            commands::import_data,
            commands::open_window,
            commands::hide_main,
            commands::set_always_on_top,
            commands::set_reminders_paused,
            commands::set_quiet_mode_until,
            commands::get_license_documents,
            commands::set_pet_surface_mode,
            commands::show_pet_context_menu,
            commands::speak_text,
            commands::stop_speaking,
            commands::list_pet_appearances,
            commands::import_pet_image,
            commands::remove_pet_image,
            commands::get_pet_asset,
            commands::set_pet_appearance,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VocaPip");
}

#[cfg(test)]
mod rebrand_tests {
    use super::*;

    #[test]
    fn migrates_legacy_database_without_removing_source() {
        let root = tempfile::tempdir().unwrap();
        let legacy_dir = root.path().join(LEGACY_IDENTIFIER);
        let current_dir = root.path().join("io.github.dodgeho.vocapip");
        fs::create_dir_all(&legacy_dir).unwrap();
        fs::create_dir_all(&current_dir).unwrap();
        let legacy_database = legacy_dir.join(LEGACY_DATABASE_FILENAME);
        fs::write(&legacy_database, b"legacy-data").unwrap();
        let current_database = current_dir.join(DATABASE_FILENAME);

        assert!(migrate_legacy_database(&legacy_dir, &current_database).unwrap());
        assert_eq!(fs::read(&current_database).unwrap(), b"legacy-data");
        assert!(legacy_database.exists());
        assert!(!migrate_legacy_database(&legacy_dir, &current_database).unwrap());
    }
}
