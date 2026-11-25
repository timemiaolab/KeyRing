use rdev::{listen, Event, EventType, Key};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerModifier {
    Command,
    Control,
    Alt,
}

impl TriggerModifier {
    pub fn from_str(value: &str) -> Self {
        match value {
            "Control" => Self::Control,
            "Alt" => Self::Alt,
            _ => Self::Command,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HotkeySettings {
    pub modifier: TriggerModifier,
    pub interval_ms: u64,
    pub paused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Idle,
    HoldingFirst,
    AwaitingSecond,
    HoldingSecond,
}

struct MachineState {
    stage: Stage,
    contaminated: bool,
    first_release_at: Option<Instant>,
    pointer_x: f64,
    pointer_y: f64,
}

impl Default for MachineState {
    fn default() -> Self {
        Self {
            stage: Stage::Idle,
            contaminated: false,
            first_release_at: None,
            pointer_x: 0.0,
            pointer_y: 0.0,
        }
    }
}

fn matches_modifier(key: Key, modifier: TriggerModifier) -> bool {
    match modifier {
        TriggerModifier::Command => matches!(key, Key::MetaLeft | Key::MetaRight),
        TriggerModifier::Control => matches!(key, Key::ControlLeft | Key::ControlRight),
        TriggerModifier::Alt => matches!(key, Key::Alt | Key::AltGr),
    }
}

#[cfg(target_os = "macos")]
pub fn ensure_accessibility_permission() -> bool {
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
        let val = CFBoolean::true_value();
        let dict = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), val.as_CFType())]);
        AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef() as *const _)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn ensure_accessibility_permission() -> bool {
    true
}

pub fn spawn_hotkey_listener<F>(
    settings: Arc<Mutex<HotkeySettings>>,
    running: Arc<AtomicBool>,
    on_trigger: F,
) where
    F: Fn(f64, f64) + Send + Sync + 'static,
{
    let machine = Arc::new(Mutex::new(MachineState::default()));

    thread::spawn(move || {
        let _ = ensure_accessibility_permission();

        let callback = move |event: Event| {
            if !running.load(Ordering::Relaxed) {
                return;
            }

            let cfg = match settings.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => return,
            };
            if cfg.paused {
                return;
            }

            let mut state = match machine.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };

            match event.event_type {
                EventType::MouseMove { x, y } => {
                    state.pointer_x = x;
                    state.pointer_y = y;
                }
                EventType::ButtonPress(_) => {
                    if matches!(state.stage, Stage::HoldingFirst | Stage::HoldingSecond | Stage::AwaitingSecond) {
                        state.contaminated = true;
                        state.stage = Stage::Idle;
                        state.first_release_at = None;
                    }
                }
                EventType::KeyPress(key) => {
                    if matches_modifier(key, cfg.modifier) {
                        match state.stage {
                            Stage::Idle => {
                                state.stage = Stage::HoldingFirst;
                                state.contaminated = false;
                            }
                            Stage::AwaitingSecond => {
                                state.stage = Stage::HoldingSecond;
                                state.contaminated = false;
                            }
                            _ => {}
                        }
                    } else if matches!(
                        state.stage,
                        Stage::HoldingFirst | Stage::HoldingSecond | Stage::AwaitingSecond
                    ) {
                        state.contaminated = true;
                        state.stage = Stage::Idle;
                        state.first_release_at = None;
                    }
                }
                EventType::KeyRelease(key) => {
                    if !matches_modifier(key, cfg.modifier) {
                        return;
                    }

                    let now = Instant::now();
                    match state.stage {
                        Stage::HoldingFirst => {
                            if state.contaminated {
                                state.stage = Stage::Idle;
                                state.contaminated = false;
                            } else {
                                state.first_release_at = Some(now);
                                state.stage = Stage::AwaitingSecond;
                            }
                        }
                        Stage::HoldingSecond => {
                            if state.contaminated {
                                state.stage = Stage::Idle;
                                state.contaminated = false;
                                state.first_release_at = None;
                                return;
                            }
                            if let Some(first_at) = state.first_release_at {
                                let delta = now.duration_since(first_at);
                                if delta <= Duration::from_millis(cfg.interval_ms) {
                                    let (x, y) = (state.pointer_x, state.pointer_y);
                                    state.stage = Stage::Idle;
                                    state.first_release_at = None;
                                    drop(state);
                                    on_trigger(x, y);
                                    return;
                                }
                            }
                            // Window expired; promote second release to fresh first tap
                            state.first_release_at = Some(now);
                            state.stage = Stage::AwaitingSecond;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        };

        if let Err(err) = listen(callback) {
            eprintln!("KeyRing rdev listener error: {:?}", err);
        }
    });
}
