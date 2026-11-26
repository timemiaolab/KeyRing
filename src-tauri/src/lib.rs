pub mod context;
pub mod executor;
#[cfg(target_os = "macos")]
pub mod finder_ax;
pub mod hotkey;
pub mod hotkey_machine;
pub mod input_event;
pub mod macos_event;
#[cfg(target_os = "macos")]
pub mod macos_listen;
pub mod icon;
pub mod layout;
pub mod paths;
pub mod tray;

use context::{capture_trigger_context, ContextSnapshot};
use layout::{PlacementStatus, PositionCapture};
use hotkey::{
    check_accessibility_permission, open_accessibility_settings, spawn_hotkey_listener,
    HotkeySettings, ListenerHealth, TriggerModifier,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::AtomicBool,
    Arc, Mutex,
};
use tauri::{AppHandle, Emitter, Listener, LogicalPosition, Manager, Position, State};
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActiveSlot {
    #[serde(rename = "app")]
    App {
        id: String,
        name: String,
        #[serde(rename = "execPath")]
        exec_path: String,
        #[serde(rename = "iconCache")]
        icon_cache: String,
        #[serde(rename = "presetKey", default)]
        preset_key: Option<String>,
        #[serde(rename = "customIconDataUrl", default)]
        custom_icon_data_url: Option<String>,
    },
    #[serde(rename = "builtin")]
    Builtin { id: String, action: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    pub modifier: String,
    #[serde(rename = "intervalMs")]
    pub interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(rename = "ringDiameter")]
    pub ring_diameter: u32,
    #[serde(rename = "innerHoleDiameter")]
    pub inner_hole_diameter: u32,
    #[serde(rename = "iconSize")]
    pub icon_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRingConfig {
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    pub trigger: TriggerConfig,
    pub ui: UiConfig,
    pub autostart: bool,
    #[serde(rename = "terminalApp", default = "default_terminal")]
    pub terminal_app: String,
    #[serde(rename = "activeSlots")]
    pub active_slots: Vec<ActiveSlot>,
    #[serde(rename = "libraryApps")]
    pub library_apps: Vec<serde_json::Value>,
}

fn default_terminal() -> String {
    "Terminal".into()
}

fn default_theme() -> String {
    "light".into()
}

fn default_config() -> KeyRingConfig {
    KeyRingConfig {
        language: "zh-CN".into(),
        theme: "light".into(),
        trigger: TriggerConfig {
            modifier: "Command".into(),
            interval_ms: 300,
        },
        ui: UiConfig {
            ring_diameter: 260,
            inner_hole_diameter: 84,
            icon_size: 36,
        },
        autostart: true,
        terminal_app: "Terminal".into(),
        active_slots: vec![
            ActiveSlot::App {
                id: "app-cursor".into(),
                name: "Cursor".into(),
                exec_path: "/Applications/Cursor.app".into(),
                icon_cache: "icons/cursor.png".into(),
                preset_key: Some("cursor".into()),
                custom_icon_data_url: None,
            },
            ActiveSlot::App {
                id: "app-vscode".into(),
                name: "Visual Studio Code".into(),
                exec_path: "/Applications/Visual Studio Code.app".into(),
                icon_cache: "icons/vscode.png".into(),
                preset_key: Some("vscode".into()),
                custom_icon_data_url: None,
            },
            ActiveSlot::Builtin {
                id: "builtin-terminal".into(),
                action: "open_in_terminal".into(),
            },
            ActiveSlot::Builtin {
                id: "builtin-copy".into(),
                action: "copy_path".into(),
            },
        ],
        library_apps: vec![],
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub platform: String,
    #[serde(rename = "accessibilityTrusted")]
    pub accessibility_trusted: bool,
    #[serde(rename = "listenerRunning")]
    pub listener_running: bool,
    #[serde(rename = "listenerError")]
    pub listener_error: Option<String>,
    pub paused: bool,
    #[serde(rename = "lastPlacementKind")]
    pub last_placement_kind: Option<String>,
    #[serde(rename = "lastPlacementMessage")]
    pub last_placement_message: Option<String>,
    #[serde(rename = "lastPlacementClamp")]
    pub last_placement_clamp: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct PieTriggerPayload {
    x: f64,
    y: f64,
    context: ContextSnapshot,
    position: PositionCapture,
}

pub struct AppStates {
    pub config: Mutex<KeyRingConfig>,
    pub context: Mutex<ContextSnapshot>,
    pub hotkey_settings: Arc<Mutex<HotkeySettings>>,
    pub listener_health: Arc<Mutex<ListenerHealth>>,
    pub last_placement: Mutex<Option<PlacementStatus>>,
    pub config_path: PathBuf,
}

#[tauri::command]
fn get_config(state: State<AppStates>) -> KeyRingConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn save_config(
    app: AppHandle,
    state: State<AppStates>,
    next_config: KeyRingConfig,
) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(&next_config).map_err(|e| e.to_string())?;
    if let Some(parent) = state.config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&state.config_path, serialized).map_err(|e| e.to_string())?;

    if let Ok(mut hk) = state.hotkey_settings.lock() {
        hk.modifier = TriggerModifier::from_str(&next_config.trigger.modifier);
        hk.interval_ms = next_config.trigger.interval_ms;
    }

    {
        use tauri_plugin_autostart::ManagerExt;
        let autolaunch = app.autolaunch();
        if next_config.autostart {
            let _ = autolaunch.enable();
        } else {
            let _ = autolaunch.disable();
        }
    }

    if let Some(tray) = app.tray_by_id("keyring-tray") {
        let paused = state
            .hotkey_settings
            .lock()
            .map(|s| s.paused)
            .unwrap_or(false);
        if let Ok(menu) = tray::build_tray_menu(&app, &next_config.language, next_config.autostart, paused) {
            let _ = tray.set_menu(Some(menu));
        }
    }

    *state.config.lock().unwrap() = next_config.clone();
    let _ = app.emit("config-updated", next_config);
    Ok(())
}

#[tauri::command]
fn hide_pie_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("pie") {
        let _ = win.hide();
    }
}

#[tauri::command]
fn execute_slot(app: AppHandle, state: State<AppStates>, slot_id: String) -> Result<(), String> {
    let cfg = state.config.lock().unwrap().clone();
    let ctx = state.context.lock().unwrap().clone();

    let slot = cfg
        .active_slots
        .iter()
        .find(|s| match s {
            ActiveSlot::App { id, .. } => id == &slot_id,
            ActiveSlot::Builtin { id, .. } => id == &slot_id,
        })
        .ok_or_else(|| "还没有这个圆环选项".to_string())?;

    match slot {
        ActiveSlot::Builtin { action, .. } if action == "copy_path" => {
            executor::execute_copy_path(&app, &ctx)?;
        }
        ActiveSlot::Builtin { action, .. } if action == "open_in_terminal" => {
            executor::execute_open_in_terminal(&ctx, &cfg.terminal_app)?;
        }
        ActiveSlot::App {
            exec_path,
            preset_key,
            ..
        } => {
            executor::execute_launch_app(exec_path, preset_key.as_deref(), &ctx)?;
        }
        _ => {}
    }

    hide_pie_window(app);
    Ok(())
}

#[tauri::command]
fn get_runtime_status(state: State<AppStates>) -> RuntimeStatus {
    let health = state.listener_health.lock().ok().map(|h| h.clone()).unwrap_or_default();
    let paused = state
        .hotkey_settings
        .lock()
        .map(|s| s.paused)
        .unwrap_or(false);
    let placement = state
        .last_placement
        .lock()
        .ok()
        .and_then(|value| value.clone());
    RuntimeStatus {
        platform: std::env::consts::OS.to_string(),
        accessibility_trusted: health.accessibility_trusted,
        listener_running: health.running,
        listener_error: health.last_error,
        paused,
        last_placement_kind: placement.as_ref().map(|item| item.kind.clone()),
        last_placement_message: placement.as_ref().and_then(|item| item.message.clone()),
        last_placement_clamp: placement.as_ref().and_then(|item| item.clamp.clone()),
    }
}

#[tauri::command]
fn set_listener_paused(
    app: AppHandle,
    state: State<AppStates>,
    paused: bool,
) -> RuntimeStatus {
    if let Ok(mut hk) = state.hotkey_settings.lock() {
        hk.paused = paused;
    }
    if let Some(tray) = app.tray_by_id("keyring-tray") {
        if let Ok(cfg) = state.config.lock() {
            if let Ok(menu) = tray::build_tray_menu(&app, &cfg.language, cfg.autostart, paused) {
                let _ = tray.set_menu(Some(menu));
            }
        }
    }
    let _ = app.emit("runtime-status-changed", ());
    get_runtime_status(state)
}

#[tauri::command]
fn request_accessibility_permission(state: State<AppStates>) -> RuntimeStatus {
    let trusted = check_accessibility_permission(true);
    if let Ok(mut health) = state.listener_health.lock() {
        health.accessibility_trusted = trusted;
        if trusted {
            health.last_error = None;
        }
    }
    get_runtime_status(state)
}

#[tauri::command]
fn open_accessibility_settings_cmd() -> Result<(), String> {
    open_accessibility_settings()
}

#[allow(dead_code)] // used on macOS when a trigger cannot be placed
fn remember_placement(app: &AppHandle, status: PlacementStatus) {
    let state = app.state::<AppStates>();
    if let Ok(mut slot) = state.last_placement.lock() {
        *slot = Some(status.clone());
    }
    let _ = app.emit("placement-status", status.clone());
    if let Some(message) = status.message {
        let _ = app
            .notification()
            .builder()
            .title("KeyRing")
            .body(message)
            .show();
    }
}

fn show_pie_window_at(app: &AppHandle, x: f64, y: f64, size: f64) {
    let Some(pie_win) = app.get_webview_window("pie") else {
        return;
    };
    let _ = pie_win.set_size(tauri::LogicalSize::new(size, size));
    let _ = pie_win.set_position(Position::Logical(LogicalPosition::new(x, y)));
    let _ = pie_win.show();
    let _ = pie_win.set_focus();
    let _ = pie_win.unminimize();
    let _ = app.emit("pie-shown", ());
}

fn show_pie_window_centered_on(app: &AppHandle, x: f64, y: f64) {
    let ring_size = app
        .state::<AppStates>()
        .config
        .lock()
        .map(|c| c.ui.ring_diameter as f64 + 60.0)
        .unwrap_or(320.0);
    show_pie_window_at(app, x - ring_size / 2.0, y - ring_size / 2.0, ring_size);
}

fn handle_pie_trigger(app: &AppHandle, data: PieTriggerPayload) {
    #[cfg(target_os = "macos")]
    {
        handle_macos_pie_trigger(app, data);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = data.position;
        show_pie_window_centered_on(app, data.x, data.y);
    }
}

#[cfg(target_os = "macos")]
fn handle_macos_pie_trigger(app: &AppHandle, data: PieTriggerPayload) {
    use crate::finder_ax::list_screens;
    use crate::layout::{place_pie_above_item, PieMetrics};

    match data.position {
        PositionCapture::Located { bounds, .. } => {
            let layout = list_screens();
            if layout.screens.is_empty() || layout.primary_height <= 0.0 {
                remember_placement(
                    app,
                    PlacementStatus::from_capture(&PositionCapture::BoundsUnavailable {
                        reason: "无法读取显示器信息。".into(),
                    }),
                );
                return;
            }
            // AXPosition / AXSize are already in top-left Y-down global logical points.
            let ring = app
                .state::<AppStates>()
                .config
                .lock()
                .map(|c| c.ui.ring_diameter as f64)
                .unwrap_or(260.0);
            let metrics = PieMetrics::from_ring_diameter(ring);
            let Some(placement) = place_pie_above_item(&bounds, &layout.screens, &metrics) else {
                remember_placement(
                    app,
                    PlacementStatus::from_capture(&PositionCapture::BoundsUnavailable {
                        reason: "没有可用的屏幕来放置圆环。".into(),
                    }),
                );
                return;
            };
            remember_placement(app, PlacementStatus::located(placement.clamp));
            show_pie_window_at(
                app,
                placement.window_x,
                placement.window_y,
                placement.window_size,
            );
        }
        other => {
            remember_placement(app, PlacementStatus::from_capture(&other));
        }
    }
}

pub fn show_config_window(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.show();

    let win = match app.get_webview_window("config") {
        Some(existing) => Some(existing),
        None => app
            .config()
            .app
            .windows
            .iter()
            .find(|w| w.label == "config")
            .and_then(|cfg| tauri::WebviewWindowBuilder::from_config(app, cfg).ok())
            .and_then(|builder| builder.build().ok()),
    };

    if let Some(win) = win {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_config_window(app);
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if matches!(window.label(), "config" | "pie") {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            let config_dir = app.path().app_config_dir().unwrap_or_else(|_| PathBuf::from("."));
            let config_path = config_dir.join("config.json");
            let initial_config: KeyRingConfig = fs::read_to_string(&config_path)
                .ok()
                .and_then(|raw| serde_json::from_str(&raw).ok())
                .unwrap_or_else(default_config);

            let hk_settings = Arc::new(Mutex::new(HotkeySettings {
                modifier: TriggerModifier::from_str(&initial_config.trigger.modifier),
                interval_ms: initial_config.trigger.interval_ms,
                paused: false,
            }));
            let listener_health = Arc::new(Mutex::new(ListenerHealth {
                accessibility_trusted: check_accessibility_permission(false),
                running: false,
                last_error: None,
            }));

            tray::setup_tray(
                app.handle(),
                &initial_config.language,
                initial_config.autostart,
            )?;

            app.manage(AppStates {
                config: Mutex::new(initial_config),
                context: Mutex::new(ContextSnapshot::default()),
                hotkey_settings: Arc::clone(&hk_settings),
                listener_health: Arc::clone(&listener_health),
                last_placement: Mutex::new(None),
                config_path,
            });

            // Ask for Accessibility on the main thread so the system prompt can appear.
            let trusted = check_accessibility_permission(true);
            if let Ok(mut health) = listener_health.lock() {
                health.accessibility_trusted = trusted;
            }

            let app_handle = app.handle().clone();
            app.listen("hotkey-triggered", move |event| {
                let payload = event.payload();
                let parsed: Result<PieTriggerPayload, _> = serde_json::from_str(payload);
                if let Ok(data) = parsed {
                    let state = app_handle.state::<AppStates>();
                    *state.context.lock().unwrap() = data.context.clone();
                    handle_pie_trigger(&app_handle, data);
                }
            });

            let emit_handle = app.handle().clone();
            let running = Arc::new(AtomicBool::new(true));
            spawn_hotkey_listener(
                hk_settings,
                running,
                listener_health,
                move |x, y| {
                    // Capture Finder selection and its screen rect while Finder
                    // is still frontmost, before the pie window takes focus.
                    let capture = capture_trigger_context();
                    let _ = emit_handle.emit(
                        "hotkey-triggered",
                        PieTriggerPayload {
                            x,
                            y,
                            context: capture.context,
                            position: capture.position,
                        },
                    );
                },
            );

            show_config_window(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            hide_pie_window,
            execute_slot,
            get_runtime_status,
            set_listener_paused,
            request_accessibility_permission,
            open_accessibility_settings_cmd
        ])
        .build(tauri::generate_context!())
        .expect("error while building KeyRing");

    #[cfg(target_os = "macos")]
    app.run(|app_handle, event| {
        if let tauri::RunEvent::Reopen { .. } = event {
            show_config_window(app_handle);
        }
    });

    #[cfg(not(target_os = "macos"))]
    app.run(|_app_handle, _event| {});
}

#[cfg(test)]
mod config_sync_tests {
    use super::*;

    #[test]
    fn default_config_includes_cursor_and_terminal() {
        let cfg = default_config();
        let ids: Vec<_> = cfg
            .active_slots
            .iter()
            .map(|s| match s {
                ActiveSlot::App { id, .. } | ActiveSlot::Builtin { id, .. } => id.as_str(),
            })
            .collect();
        assert!(ids.contains(&"app-cursor"));
        assert!(ids.contains(&"builtin-terminal"));
    }

    #[test]
    fn config_roundtrip_keeps_preset_and_paths_with_spaces() {
        let mut cfg = default_config();
        cfg.active_slots.push(ActiveSlot::App {
            id: "custom".into(),
            name: "My App".into(),
            exec_path: r"/Applications/My App.app".into(),
            icon_cache: "".into(),
            preset_key: Some("custom".into()),
            custom_icon_data_url: Some("data:image/png;base64,abc".into()),
        });
        let json = serde_json::to_string(&cfg).unwrap();
        let back: KeyRingConfig = serde_json::from_str(&json).unwrap();
        match &back.active_slots.last().unwrap() {
            ActiveSlot::App {
                exec_path,
                preset_key,
                custom_icon_data_url,
                ..
            } => {
                assert_eq!(exec_path, r"/Applications/My App.app");
                assert_eq!(preset_key.as_deref(), Some("custom"));
                assert_eq!(
                    custom_icon_data_url.as_deref(),
                    Some("data:image/png;base64,abc")
                );
            }
            _ => panic!("expected app slot"),
        }
    }
}
