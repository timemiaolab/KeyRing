pub use crate::hotkey_machine::{
    apply_event, matches_modifier, HotkeySettings, MachineState, TriggerModifier,
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct ListenerHealth {
    pub accessibility_trusted: bool,
    pub running: bool,
    pub last_error: Option<String>,
}

#[cfg(target_os = "macos")]
fn accessibility_trusted(prompt: bool) -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
    }
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;

    unsafe {
        let key = CFString::new("AXTrustedCheckOptionPrompt");
        let val = if prompt {
            CFBoolean::true_value()
        } else {
            CFBoolean::false_value()
        };
        let dict = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), val.as_CFType())]);
        AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef() as *const _)
    }
}

#[cfg(not(target_os = "macos"))]
fn accessibility_trusted(_prompt: bool) -> bool {
    true
}

pub fn check_accessibility_permission(prompt: bool) -> bool {
    accessibility_trusted(prompt)
}

pub fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // Official System Settings deep link. Do not suggest disabling SIP or other unsafe steps.
        let status = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("无法打开系统设置，请手动前往「系统设置 > 隐私与安全性 > 辅助功能」。".into())
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

pub fn spawn_hotkey_listener<F>(
    settings: Arc<Mutex<HotkeySettings>>,
    running: Arc<AtomicBool>,
    health: Arc<Mutex<ListenerHealth>>,
    on_trigger: F,
) where
    F: Fn(f64, f64) + Send + Sync + 'static,
{
    let on_trigger: Arc<dyn Fn(f64, f64) + Send + Sync> = Arc::new(on_trigger);
    let _ = thread::Builder::new()
        .name("keyring-hotkey-listener".into())
        .spawn(move || {
            // Retry when the event tap cannot be created (usually missing Accessibility).
            // A successful listen blocks in a background CFRunLoop / rdev loop, never the
            // Tauri / AppKit main thread.
            while running.load(Ordering::Relaxed) {
                let trusted = check_accessibility_permission(false);
                if let Ok(mut h) = health.lock() {
                    h.accessibility_trusted = trusted;
                    h.running = false;
                }

                if !trusted {
                    if let Ok(mut h) = health.lock() {
                        h.last_error = Some(
                            "系统尚未授权键盘监听。请在「系统设置 > 隐私与安全性 > 辅助功能」中打开 KeyRing，然后返回应用。"
                                .into(),
                        );
                    }
                    thread::sleep(Duration::from_secs(2));
                    continue;
                }

                if let Ok(mut h) = health.lock() {
                    h.running = true;
                    h.last_error = None;
                    h.accessibility_trusted = true;
                }

                match run_platform_listener(
                    Arc::clone(&settings),
                    Arc::clone(&running),
                    Arc::clone(&on_trigger),
                ) {
                    Ok(()) => {
                        if let Ok(mut h) = health.lock() {
                            h.running = false;
                        }
                        if !running.load(Ordering::Relaxed) {
                            break;
                        }
                        // Tap ended without a process exit (for example, permission was
                        // revoked mid-run). Retry instead of leaving the listener dead.
                        thread::sleep(Duration::from_secs(2));
                    }
                    Err(err) => {
                        if let Ok(mut h) = health.lock() {
                            h.running = false;
                            h.last_error = Some(format!(
                                "全局键盘监听未能启动（{err}）。请确认已在「辅助功能」中授权 KeyRing，授权后应用会自动重试。"
                            ));
                        }
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
        });
}

#[cfg(target_os = "macos")]
fn run_platform_listener(
    settings: Arc<Mutex<HotkeySettings>>,
    running: Arc<AtomicBool>,
    on_trigger: Arc<dyn Fn(f64, f64) + Send + Sync>,
) -> Result<(), String> {
    crate::macos_listen::listen_physical_events(settings, running, move |x, y| on_trigger(x, y))
        .map_err(|err| err.to_string())
}

#[cfg(not(target_os = "macos"))]
fn run_platform_listener(
    settings: Arc<Mutex<HotkeySettings>>,
    running: Arc<AtomicBool>,
    on_trigger: Arc<dyn Fn(f64, f64) + Send + Sync>,
) -> Result<(), String> {
    use crate::input_event::from_rdev;
    use rdev::{listen, Event};
    use std::sync::mpsc;
    use std::time::Instant;

    let (tx, rx) = mpsc::sync_channel::<(f64, f64)>(1);
    let worker = Arc::clone(&on_trigger);
    let _ = thread::Builder::new()
        .name("keyring-hotkey-trigger".into())
        .spawn(move || {
            while let Ok((x, y)) = rx.recv() {
                worker(x, y);
            }
        });

    let machine = Mutex::new(MachineState::default());
    let callback = move |event: Event| {
        if !running.load(Ordering::Relaxed) {
            return;
        }
        let Some(input) = from_rdev(event.event_type) else {
            return;
        };
        let cfg = match settings.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => return,
        };
        let mut state = match machine.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if let Some(coords) = apply_event(&mut state, input, &cfg, Instant::now()) {
            drop(state);
            let _ = tx.try_send(coords);
        }
    };

    listen(callback).map_err(|err| format!("{err:?}"))
}
