use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::repository;
use chrono::{DateTime, Duration as ChronoDuration, Local, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{
    App, AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl,
    WebviewWindowBuilder,
};

pub const MAIN_WINDOW: &str = "main";

const PET_MENU_LEARN: &str = "pet_learn";
const PET_MENU_ALWAYS_ON_TOP: &str = "pet_always_on_top";
const PET_MENU_TOGGLE_PAUSE: &str = "pet_toggle_pause";
const PET_MENU_LIBRARY: &str = "pet_library";
const PET_MENU_SETTINGS: &str = "pet_settings";
const PET_MENU_HIDE: &str = "pet_hide";
const PET_MENU_EXIT: &str = "pet_exit";

const PET_FOOT_BOTTOM_MARGIN: f64 = 20.0;
const PET_FOOT_BELOW_Y: f64 = 238.0;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PetSurfaceMode {
    Idle,
    Learning,
    Progress,
    Placement,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BubbleSide {
    Above,
    Below,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PetSurfaceResult {
    pub mode: PetSurfaceMode,
    pub bubble_side: BubbleSide,
}

#[derive(Debug, Clone, Copy)]
struct SurfaceLayout {
    mode: PetSurfaceMode,
    bubble_side: BubbleSide,
}

#[derive(Default)]
pub struct PetSurfaceState {
    layout: Mutex<Option<SurfaceLayout>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LogicalDimensions {
    width: f64,
    height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelPoint {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelWorkArea {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SurfaceGeometry {
    logical_size: LogicalDimensions,
    position: PixelPoint,
    bubble_side: BubbleSide,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct PetActionEvent<'a> {
    action: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    paused: Option<bool>,
}

#[derive(Default)]
pub struct LifecycleState {
    pub exiting: AtomicBool,
}

impl PetSurfaceMode {
    fn logical_size(self) -> LogicalDimensions {
        match self {
            Self::Idle => LogicalDimensions {
                width: 232.0,
                height: 258.0,
            },
            Self::Learning => LogicalDimensions {
                width: 384.0,
                height: 620.0,
            },
            Self::Progress => LogicalDimensions {
                width: 300.0,
                height: 390.0,
            },
            Self::Placement => LogicalDimensions {
                width: 420.0,
                height: 620.0,
            },
        }
    }

    fn supports_below(self) -> bool {
        matches!(self, Self::Learning | Self::Placement)
    }
}

fn anchor_offset(layout: SurfaceLayout) -> (f64, f64) {
    let size = layout.mode.logical_size();
    let y = if layout.mode.supports_below() && layout.bubble_side == BubbleSide::Below {
        PET_FOOT_BELOW_Y.min(size.height - PET_FOOT_BOTTOM_MARGIN)
    } else {
        size.height - PET_FOOT_BOTTOM_MARGIN
    };
    (size.width / 2.0, y)
}

fn scaled(value: f64, scale_factor: f64) -> i32 {
    (value * scale_factor).round() as i32
}

fn clamp_axis(value: i32, start: i32, span: u32, item_span: i32) -> i32 {
    let available = span.saturating_sub(item_span.max(0) as u32);
    let end = start.saturating_add(available.min(i32::MAX as u32) as i32);
    value.clamp(start, end.max(start))
}

fn calculate_surface_geometry(
    mode: PetSurfaceMode,
    anchor: PixelPoint,
    work_area: PixelWorkArea,
    scale_factor: f64,
) -> SurfaceGeometry {
    let logical_size = mode.logical_size();
    let above_layout = SurfaceLayout {
        mode,
        bubble_side: BubbleSide::Above,
    };
    let (_, above_anchor_y) = anchor_offset(above_layout);
    let bubble_side =
        if mode.supports_below() && anchor.y - work_area.y < scaled(above_anchor_y, scale_factor) {
            BubbleSide::Below
        } else {
            BubbleSide::Above
        };
    let layout = SurfaceLayout { mode, bubble_side };
    let (anchor_x, anchor_y) = anchor_offset(layout);
    let physical_width = scaled(logical_size.width, scale_factor);
    let physical_height = scaled(logical_size.height, scale_factor);
    let desired_x = anchor.x.saturating_sub(scaled(anchor_x, scale_factor));
    let desired_y = anchor.y.saturating_sub(scaled(anchor_y, scale_factor));

    SurfaceGeometry {
        logical_size,
        position: PixelPoint {
            x: clamp_axis(desired_x, work_area.x, work_area.width, physical_width),
            y: clamp_axis(desired_y, work_area.y, work_area.height, physical_height),
        },
        bubble_side,
    }
}

fn current_pet_anchor(
    window_position: PixelPoint,
    current_size: (u32, u32),
    scale_factor: f64,
    layout: Option<SurfaceLayout>,
) -> PixelPoint {
    if let Some(layout) = layout {
        let (anchor_x, anchor_y) = anchor_offset(layout);
        return PixelPoint {
            x: window_position
                .x
                .saturating_add(scaled(anchor_x, scale_factor)),
            y: window_position
                .y
                .saturating_add(scaled(anchor_y, scale_factor)),
        };
    }

    PixelPoint {
        x: window_position
            .x
            .saturating_add((current_size.0 / 2) as i32),
        y: window_position.y.saturating_add(
            (current_size.1 as i32).saturating_sub(scaled(PET_FOOT_BOTTOM_MARGIN, scale_factor)),
        ),
    }
}

pub fn set_main_surface_mode(app: &AppHandle, mode: PetSurfaceMode) -> AppResult<PetSurfaceResult> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| AppError::Resource("main window is unavailable".into()))?;
    let monitor = window
        .current_monitor()?
        .or(window.primary_monitor()?)
        .ok_or_else(|| AppError::Resource("no monitor is available for the main window".into()))?;
    let scale_factor = monitor.scale_factor();
    let work_area = monitor.work_area();
    let work_area = PixelWorkArea {
        x: work_area.position.x,
        y: work_area.position.y,
        width: work_area.size.width,
        height: work_area.size.height,
    };
    let position = window.outer_position()?;
    let current_size = window.inner_size()?;
    let surface_state = app.state::<PetSurfaceState>();
    let mut layout = surface_state
        .layout
        .lock()
        .map_err(|_| AppError::Resource("pet surface state lock is poisoned".into()))?;
    let anchor = current_pet_anchor(
        PixelPoint {
            x: position.x,
            y: position.y,
        },
        (current_size.width, current_size.height),
        scale_factor,
        *layout,
    );
    let geometry = calculate_surface_geometry(mode, anchor, work_area, scale_factor);

    let minimum = PetSurfaceMode::Idle.logical_size();
    window.set_min_size(Some(LogicalSize::new(minimum.width, minimum.height)))?;
    window.set_size(LogicalSize::new(
        geometry.logical_size.width,
        geometry.logical_size.height,
    ))?;
    window.set_position(PhysicalPosition::new(
        geometry.position.x,
        geometry.position.y,
    ))?;
    *layout = Some(SurfaceLayout {
        mode,
        bubble_side: geometry.bubble_side,
    });

    Ok(PetSurfaceResult {
        mode,
        bubble_side: geometry.bubble_side,
    })
}

pub fn set_main_always_on_top(app: &AppHandle, enabled: bool) -> AppResult<()> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| AppError::Resource("main window is unavailable".into()))?;
    window.set_always_on_top(enabled)?;
    Ok(())
}

pub fn show_main_window(app: &AppHandle) -> AppResult<()> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| AppError::Resource("main window is unavailable".into()))?;
    window.show()?;
    window.unminimize()?;
    window.set_focus()?;
    Ok(())
}

pub fn show_main_window_passive(app: &AppHandle) -> AppResult<()> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| AppError::Resource("main window is unavailable".into()))?;
    window.show()?;
    window.unminimize()?;
    Ok(())
}

pub fn hide_main_window(app: &AppHandle) -> AppResult<()> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| AppError::Resource("main window is unavailable".into()))?;
    window.hide()?;
    Ok(())
}

pub fn toggle_main_window(app: &AppHandle) -> AppResult<()> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| AppError::Resource("main window is unavailable".into()))?;
    if window.is_visible()? {
        window.hide()?;
    } else {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    }
    Ok(())
}

pub fn open_or_focus_window(app: &AppHandle, view: &str) -> AppResult<()> {
    if view == "pet" {
        return show_main_window(app);
    }
    let (label, title, width, height) = match view {
        "library" => ("library", "VocaPip - Word Library", 980.0, 720.0),
        "settings" => ("settings", "VocaPip - Settings", 760.0, 700.0),
        _ => {
            return Err(AppError::InvalidData(format!(
                "unsupported window view {view}"
            )))
        }
    };
    if let Some(window) = app.get_webview_window(label) {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        label,
        WebviewUrl::App(format!("index.html?view={view}").into()),
    )
    .title(title)
    .inner_size(width, height)
    .min_inner_size(640.0, 520.0)
    .resizable(true)
    .build()?;
    Ok(())
}

fn toggle_context_always_on_top(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<DbState>();
    let settings = state
        .connect()
        .and_then(|connection| repository::read_settings(&connection))?;
    let enabled = !settings.always_on_top;
    set_main_always_on_top(app, enabled)?;
    repository::set_boolean_setting(&state, "always_on_top", enabled)?;
    app.emit_to(
        MAIN_WINDOW,
        "vocapip://pet-action",
        PetActionEvent {
            action: "always-on-top",
            enabled: Some(enabled),
            paused: None,
        },
    )?;
    Ok(())
}

fn toggle_context_reminders(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<DbState>();
    let settings = state
        .connect()
        .and_then(|connection| repository::read_settings(&connection))?;
    let paused = !settings.reminders_paused;
    repository::set_boolean_setting(&state, "reminders_paused", paused)?;
    app.emit_to(
        MAIN_WINDOW,
        "vocapip://pet-action",
        PetActionEvent {
            action: "reminders-paused",
            enabled: None,
            paused: Some(paused),
        },
    )?;
    Ok(())
}

fn handle_pet_menu_action(app: &AppHandle, id: &str) -> AppResult<()> {
    match id {
        PET_MENU_LEARN => {
            show_main_window(app)?;
            app.emit_to(
                MAIN_WINDOW,
                "vocapip://pet-action",
                PetActionEvent {
                    action: "learn",
                    enabled: None,
                    paused: None,
                },
            )?;
        }
        PET_MENU_ALWAYS_ON_TOP => toggle_context_always_on_top(app)?,
        PET_MENU_TOGGLE_PAUSE => toggle_context_reminders(app)?,
        PET_MENU_LIBRARY => open_or_focus_window(app, "library")?,
        PET_MENU_SETTINGS => open_or_focus_window(app, "settings")?,
        PET_MENU_HIDE => hide_main_window(app)?,
        PET_MENU_EXIT => {
            app.state::<LifecycleState>()
                .exiting
                .store(true, Ordering::SeqCst);
            app.exit(0);
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_pet_menu_event(app: &AppHandle, event: MenuEvent) {
    if let Err(error) = handle_pet_menu_action(app, event.id().as_ref()) {
        eprintln!("unable to handle pet menu action: {error}");
    }
}

pub fn show_pet_context_menu(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<DbState>();
    let settings = state
        .connect()
        .and_then(|connection| repository::read_settings(&connection))?;
    let learn = MenuItem::with_id(app, PET_MENU_LEARN, "学习一个词", true, None::<&str>)?;
    let always_on_top = CheckMenuItem::with_id(
        app,
        PET_MENU_ALWAYS_ON_TOP,
        "始终置顶",
        true,
        settings.always_on_top,
        None::<&str>,
    )?;
    let pause_label = if settings.reminders_paused {
        "继续提醒"
    } else {
        "暂停提醒"
    };
    let pause = CheckMenuItem::with_id(
        app,
        PET_MENU_TOGGLE_PAUSE,
        pause_label,
        true,
        settings.reminders_paused,
        None::<&str>,
    )?;
    let library = MenuItem::with_id(app, PET_MENU_LIBRARY, "单词本", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, PET_MENU_SETTINGS, "设置", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, PET_MENU_HIDE, "隐藏", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, PET_MENU_EXIT, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &learn,
            &always_on_top,
            &pause,
            &library,
            &settings_item,
            &hide,
            &exit,
        ],
    )?;
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| AppError::Resource("main window is unavailable".into()))?;
    window.popup_menu(&menu)?;
    Ok(())
}

pub fn setup_tray(app: &App) -> AppResult<()> {
    let show_hide = MenuItem::with_id(app, "show_hide", "显示 / 隐藏桌宠", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "toggle_pause", "暂停 / 继续提醒", true, None::<&str>)?;
    let quiet = MenuItem::with_id(app, "quiet_hour", "安静模式 1 小时", true, None::<&str>)?;
    let library = MenuItem::with_id(app, "library", "单词本", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&show_hide, &pause, &quiet, &library, &settings, &exit],
    )?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("VocaPip")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show_hide" => {
                let _ = toggle_main_window(app);
            }
            "toggle_pause" => {
                let state = app.state::<DbState>();
                if let Ok(settings) = state
                    .connect()
                    .and_then(|db| repository::read_settings(&db))
                {
                    let _ = repository::set_boolean_setting(
                        &state,
                        "reminders_paused",
                        !settings.reminders_paused,
                    );
                }
            }
            "quiet_hour" => {
                let until = (Utc::now() + ChronoDuration::hours(1)).to_rfc3339();
                let state = app.state::<DbState>();
                let _ = repository::set_quiet_until(&state, Some(&until));
            }
            "library" => {
                let _ = open_or_focus_window(app, "library");
            }
            "settings" => {
                let _ = open_or_focus_window(app, "settings");
            }
            "exit" => {
                app.state::<LifecycleState>()
                    .exiting
                    .store(true, Ordering::SeqCst);
                app.exit(0);
            }
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

pub fn start_reminder_loop(app: AppHandle) {
    std::thread::Builder::new()
        .name("vocapip-reminders".into())
        .spawn(move || {
            let mut last_emitted = Instant::now();
            loop {
                std::thread::sleep(Duration::from_secs(15));
                let state = app.state::<DbState>();
                let settings = match state
                    .connect()
                    .and_then(|db| repository::read_settings(&db))
                {
                    Ok(settings) => settings,
                    Err(_) => continue,
                };
                let interval = Duration::from_secs(
                    settings.reminder_interval_minutes.clamp(5, 240) as u64 * 60,
                );
                if last_emitted.elapsed() < interval || reminders_suppressed(&settings) {
                    continue;
                }
                let _ = show_main_window_passive(&app);
                let _ = app.emit("vocapip://reminder-due", ());
                last_emitted = Instant::now();
            }
        })
        .expect("failed to start local reminder thread");
}

fn reminders_suppressed(settings: &crate::models::AppSettings) -> bool {
    if settings.reminders_paused {
        return true;
    }
    if settings
        .quiet_mode_until
        .as_deref()
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .is_some_and(|until| until.with_timezone(&Utc) > Utc::now())
    {
        return true;
    }
    if !settings.quiet_hours_enabled {
        return false;
    }
    let Ok(start) = NaiveTime::parse_from_str(&settings.quiet_start, "%H:%M") else {
        return true;
    };
    let Ok(end) = NaiveTime::parse_from_str(&settings.quiet_end, "%H:%M") else {
        return true;
    };
    let now = Local::now().time();
    if start <= end {
        now >= start && now < end
    } else {
        now >= start || now < end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AppSettings;

    fn settings() -> AppSettings {
        AppSettings {
            reminder_interval_minutes: 30,
            daily_quota: 10,
            quiet_hours_enabled: false,
            quiet_start: "22:00".into(),
            quiet_end: "08:00".into(),
            autostart: false,
            always_on_top: true,
            language_pack_id: "zh-CN__en.pilot".into(),
            reminders_paused: false,
            quiet_mode_until: None,
        }
    }

    #[test]
    fn paused_reminders_are_suppressed() {
        let mut value = settings();
        value.reminders_paused = true;
        assert!(reminders_suppressed(&value));
    }

    #[test]
    fn expired_quiet_mode_is_not_suppressed() {
        let mut value = settings();
        value.quiet_mode_until = Some((Utc::now() - ChronoDuration::minutes(1)).to_rfc3339());
        assert!(!reminders_suppressed(&value));
    }

    #[test]
    fn surface_modes_use_expected_logical_sizes() {
        assert_eq!(
            PetSurfaceMode::Idle.logical_size(),
            LogicalDimensions {
                width: 232.0,
                height: 258.0
            }
        );
        assert_eq!(
            PetSurfaceMode::Learning.logical_size(),
            LogicalDimensions {
                width: 384.0,
                height: 620.0
            }
        );
        assert_eq!(
            PetSurfaceMode::Progress.logical_size(),
            LogicalDimensions {
                width: 300.0,
                height: 390.0
            }
        );
        assert_eq!(
            PetSurfaceMode::Placement.logical_size(),
            LogicalDimensions {
                width: 420.0,
                height: 620.0
            }
        );
    }

    #[test]
    fn scaling_keeps_the_idle_foot_anchor_stable() {
        for scale_factor in [1.0, 1.25, 1.5] {
            let anchor = PixelPoint { x: 1400, y: 900 };
            let geometry = calculate_surface_geometry(
                PetSurfaceMode::Idle,
                anchor,
                PixelWorkArea {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                scale_factor,
            );
            let layout = SurfaceLayout {
                mode: PetSurfaceMode::Idle,
                bubble_side: geometry.bubble_side,
            };
            let (offset_x, offset_y) = anchor_offset(layout);
            assert_eq!(
                geometry.position.x + scaled(offset_x, scale_factor),
                anchor.x
            );
            assert_eq!(
                geometry.position.y + scaled(offset_y, scale_factor),
                anchor.y
            );
        }
    }

    #[test]
    fn geometry_supports_negative_monitor_coordinates() {
        let geometry = calculate_surface_geometry(
            PetSurfaceMode::Learning,
            PixelPoint { x: -1700, y: 900 },
            PixelWorkArea {
                x: -1920,
                y: 0,
                width: 1920,
                height: 1040,
            },
            1.0,
        );
        assert_eq!(geometry.bubble_side, BubbleSide::Above);
        assert_eq!(geometry.position, PixelPoint { x: -1892, y: 300 });
    }

    #[test]
    fn learning_surface_expands_below_when_top_space_is_insufficient() {
        let geometry = calculate_surface_geometry(
            PetSurfaceMode::Learning,
            PixelPoint { x: 500, y: 260 },
            PixelWorkArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            1.0,
        );
        assert_eq!(geometry.bubble_side, BubbleSide::Below);
        assert_eq!(geometry.position, PixelPoint { x: 308, y: 22 });
    }

    #[test]
    fn placement_surface_clamps_to_scaled_work_area_edges() {
        let geometry = calculate_surface_geometry(
            PetSurfaceMode::Placement,
            PixelPoint { x: 2860, y: 1530 },
            PixelWorkArea {
                x: 0,
                y: 0,
                width: 2880,
                height: 1560,
            },
            1.5,
        );
        assert_eq!(geometry.bubble_side, BubbleSide::Above);
        assert_eq!(geometry.position, PixelPoint { x: 2250, y: 630 });
    }

    #[test]
    fn first_transition_uses_the_existing_window_bottom_center_as_anchor() {
        let anchor = current_pet_anchor(PixelPoint { x: -400, y: 100 }, (360, 640), 1.25, None);
        assert_eq!(anchor, PixelPoint { x: -220, y: 715 });
    }
}
