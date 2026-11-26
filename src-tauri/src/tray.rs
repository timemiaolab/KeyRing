use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub struct TrayLabels {
    pub manage: &'static str,
    pub pause: &'static str,
    pub autostart: &'static str,
    pub quit: &'static str,
}

pub fn labels_for_locale(locale: &str) -> TrayLabels {
    match locale {
        "zh-TW" => TrayLabels {
            manage: "管理 KeyRing",
            pause: "暫停全域監聽",
            autostart: "開機自動啟動",
            quit: "結束 KeyRing",
        },
        "en-US" => TrayLabels {
            manage: "Manage KeyRing",
            pause: "Pause Global Listener",
            autostart: "Launch at Login",
            quit: "Quit KeyRing",
        },
        "ja-JP" => TrayLabels {
            manage: "KeyRing を管理",
            pause: "監視を一時停止",
            autostart: "ログイン時に起動",
            quit: "KeyRing を終了",
        },
        "ko-KR" => TrayLabels {
            manage: "KeyRing 관리",
            pause: "전역 감지 일시중지",
            autostart: "로그인 시 자동 실행",
            quit: "KeyRing 종료",
        },
        "fr-FR" => TrayLabels {
            manage: "Gérer KeyRing",
            pause: "Suspendre l'écoute",
            autostart: "Lancer au démarrage",
            quit: "Quitter KeyRing",
        },
        "de-DE" => TrayLabels {
            manage: "KeyRing verwalten",
            pause: "Listener pausieren",
            autostart: "Beim Systemstart öffnen",
            quit: "KeyRing beenden",
        },
        "es-ES" => TrayLabels {
            manage: "Administrar KeyRing",
            pause: "Pausar escucha",
            autostart: "Iniciar con el sistema",
            quit: "Salir de KeyRing",
        },
        "ru-RU" => TrayLabels {
            manage: "Настроить KeyRing",
            pause: "Приостановить перехват",
            autostart: "Автозапуск",
            quit: "Выйти из KeyRing",
        },
        "pt-BR" => TrayLabels {
            manage: "Gerenciar KeyRing",
            pause: "Pausar escuta",
            autostart: "Iniciar com o sistema",
            quit: "Sair do KeyRing",
        },
        _ => TrayLabels {
            manage: "管理 KeyRing",
            pause: "暂停全局监听",
            autostart: "开机自启动",
            quit: "退出 KeyRing",
        },
    }
}

pub fn build_tray_menu(
    app: &AppHandle,
    locale: &str,
    autostart: bool,
    paused: bool,
) -> tauri::Result<Menu<tauri::Wry>> {
    let labels = labels_for_locale(locale);
    let manage_item = MenuItem::with_id(app, "tray_manage", labels.manage, true, None::<&str>)?;
    let pause_item = CheckMenuItem::with_id(app, "tray_pause", labels.pause, true, paused, None::<&str>)?;
    let autostart_item = CheckMenuItem::with_id(app, "tray_autostart", labels.autostart, true, autostart, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "tray_quit", labels.quit, true, None::<&str>)?;

    Menu::with_items(app, &[&manage_item, &pause_item, &autostart_item, &sep, &quit_item])
}

pub fn setup_tray(app: &AppHandle, locale: &str, autostart: bool) -> tauri::Result<()> {
    let menu = build_tray_menu(app, locale, autostart, false)?;
    let mut builder = TrayIconBuilder::with_id("keyring-tray")
        .menu(&menu)
        .show_menu_on_left_click(cfg!(target_os = "macos"))
        .tooltip("KeyRing")
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                crate::show_config_window(tray.app_handle());
            }
            #[cfg(not(target_os = "macos"))]
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } => {
                crate::show_config_window(tray.app_handle());
            }
            _ => {}
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "tray_manage" => {
                crate::show_config_window(app);
            }
            "tray_pause" => {
                let state = app.state::<crate::AppStates>();
                let paused = if let Ok(mut hk) = state.hotkey_settings.lock() {
                    hk.paused = !hk.paused;
                    hk.paused
                } else {
                    false
                };
                let language = state
                    .config
                    .lock()
                    .map(|c| c.language.clone())
                    .unwrap_or_else(|_| "zh-CN".into());
                let autostart = state.config.lock().map(|c| c.autostart).unwrap_or(true);
                if let Some(tray) = app.tray_by_id("keyring-tray") {
                    if let Ok(menu) = build_tray_menu(app, &language, autostart, paused) {
                        let _ = tray.set_menu(Some(menu));
                    }
                }
                let _ = app.emit("runtime-status-changed", ());
            }
            "tray_autostart" => {
                use tauri_plugin_autostart::ManagerExt;
                let state = app.state::<crate::AppStates>();
                let updated = {
                    let mut cfg = state.config.lock().unwrap();
                    cfg.autostart = !cfg.autostart;
                    cfg.clone()
                };
                if let Ok(serialized) = serde_json::to_string_pretty(&updated) {
                    if let Some(parent) = state.config_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(&state.config_path, serialized);
                }
                let autolaunch = app.autolaunch();
                if updated.autostart {
                    let _ = autolaunch.enable();
                } else {
                    let _ = autolaunch.disable();
                }
                let paused = state.hotkey_settings.lock().map(|s| s.paused).unwrap_or(false);
                if let Some(tray) = app.tray_by_id("keyring-tray") {
                    if let Ok(menu) = build_tray_menu(app, &updated.language, updated.autostart, paused) {
                        let _ = tray.set_menu(Some(menu));
                    }
                }
                let _ = app.emit("config-updated", updated);
            }
            "tray_quit" => {
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
