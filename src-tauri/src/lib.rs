pub mod context;
pub mod executor;
pub mod hotkey;
pub mod hotkey_machine;
pub mod icon;
pub mod paths;
pub mod tray;

use context::{capture_active_context, ContextSnapshot};
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

fn default_config() -> KeyRingConfig {
    KeyRingConfig {
        language: "zh-CN".into(),
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
}

#[derive(Clone, Serialize, Deserialize)]
struct PieTriggerPayload {
    x: f64,
    y: f64,
    context: ContextSnapshot,
}

pub struct AppStates {
    pub config: Mutex<KeyRingConfig>,
    pub context: Mutex<ContextSnapshot>,
    pub hotkey_settings: Arc<Mutex<HotkeySettings>>,
    pub listener_health: Arc<Mutex<ListenerHealth>>,
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
        ActiveSlot::App { exec_path, .. } => {
            executor::execute_launch_app(exec_path, &ctx)?;
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
    RuntimeStatus {
        platform: std::env::consts::OS.to_string(),
        accessibility_trusted: health.accessibility_trusted,
        listener_running: health.running,
        listener_error: health.last_error,
        paused,
    }
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

fn show_pie_window(app: &AppHandle, x: f64, y: f64) {
    let Some(pie_win) = app.get_webview_window("pie") else {
        return;
    };
    let ring_size = app
        .state::<AppStates>()
        .config
        .lock()
        .map(|c| c.ui.ring_diameter as f64 + 60.0)
        .unwrap_or(320.0);

    let pos_x = x - ring_size / 2.0;
    let pos_y = y - ring_size / 2.0;

    // Logical coordinates match rdev/CGEvent point space on macOS and avoid
    // the previous PhysicalPosition + max(12) clamp that broke multi-monitor.
    let _ = pie_win.set_size(tauri::LogicalSize::new(ring_size, ring_size));
    let _ = pie_win.set_position(Position::Logical(LogicalPosition::new(pos_x, pos_y)));
    let _ = pie_win.show();
    let _ = pie_win.set_focus();
    let _ = pie_win.unminimize();
    let _ = app.emit("pie-shown", ());
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
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
                    *state.context.lock().unwrap() = data.context;
                    show_pie_window(&app_handle, data.x, data.y);
                }
            });

            let emit_handle = app.handle().clone();
            let running = Arc::new(AtomicBool::new(true));
            spawn_hotkey_listener(
                hk_settings,
                running,
                listener_health,
                move |x, y| {
                    // Capture Finder/Explorer selection while it is still frontmost.
                    let context = capture_active_context();
                    let _ = emit_handle.emit(
                        "hotkey-triggered",
                        PieTriggerPayload { x, y, context },
                    );
                },
            );

            if let Some(config_win) = app.get_webview_window("config") {
                let _ = config_win.show();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            hide_pie_window,
            execute_slot,
            get_runtime_status,
            request_accessibility_permission,
            open_accessibility_settings_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running KeyRing");
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
