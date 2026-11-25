pub mod context;
pub mod executor;
pub mod hotkey;
pub mod icon;
pub mod tray;

use context::{capture_active_context, ContextSnapshot};
use hotkey::{spawn_hotkey_listener, HotkeySettings, TriggerModifier};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::AtomicBool,
    Arc, Mutex,
};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State};

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

pub struct AppStates {
    pub config: Mutex<KeyRingConfig>,
    pub context: Mutex<ContextSnapshot>,
    pub hotkey_settings: Arc<Mutex<HotkeySettings>>,
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
        if let Ok(menu) = tray::build_tray_menu(&app, &next_config.language, next_config.autostart, false) {
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
        .ok_or_else(|| "Slot not found".to_string())?;

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
                .unwrap_or_else(|| KeyRingConfig {
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
                    active_slots: vec![],
                    library_apps: vec![],
                });

            let hk_settings = Arc::new(Mutex::new(HotkeySettings {
                modifier: TriggerModifier::from_str(&initial_config.trigger.modifier),
                interval_ms: initial_config.trigger.interval_ms,
                paused: false,
            }));

            tray::setup_tray(app.handle(), &initial_config.language, initial_config.autostart)?;

            app.manage(AppStates {
                config: Mutex::new(initial_config),
                context: Mutex::new(ContextSnapshot::default()),
                hotkey_settings: Arc::clone(&hk_settings),
                config_path,
            });

            let app_handle = app.handle().clone();
            let running = Arc::new(AtomicBool::new(true));

            spawn_hotkey_listener(hk_settings, running, move |x, y| {
                let snapshot = capture_active_context();
                let state = app_handle.state::<AppStates>();
                *state.context.lock().unwrap() = snapshot;

                if let Some(pie_win) = app_handle.get_webview_window("pie") {
                    let ring_size = state.config.lock().unwrap().ui.ring_diameter as i32 + 60;
                    let pos_x = (x as i32) - ring_size / 2;
                    let pos_y = (y as i32) - ring_size / 2;
                    let _ = pie_win.set_position(PhysicalPosition::new(pos_x.max(12), pos_y.max(12)));
                    let _ = pie_win.show();
                    let _ = pie_win.set_focus();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            hide_pie_window,
            execute_slot
        ])
        .run(tauri::generate_context!())
        .expect("error while running KeyRing");
}
