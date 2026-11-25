use rdev::{EventType, Key};
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
pub enum Stage {
    Idle,
    HoldingFirst,
    AwaitingSecond,
    HoldingSecond,
}

#[derive(Debug, Clone)]
pub struct MachineState {
    pub stage: Stage,
    pub contaminated: bool,
    pub first_release_at: Option<Instant>,
    pub second_press_at: Option<Instant>,
    pub pointer_x: f64,
    pub pointer_y: f64,
}

impl Default for MachineState {
    fn default() -> Self {
        Self {
            stage: Stage::Idle,
            contaminated: false,
            first_release_at: None,
            second_press_at: None,
            pointer_x: 0.0,
            pointer_y: 0.0,
        }
    }
}

/// macOS virtual key codes for Command / Control that rdev may surface as Unknown.
const MAC_COMMAND_LEFT: u32 = 55;
const MAC_COMMAND_RIGHT: u32 = 54;
const MAC_CONTROL_LEFT: u32 = 59;
const MAC_CONTROL_RIGHT: u32 = 62;

pub fn matches_modifier(key: Key, modifier: TriggerModifier) -> bool {
    match modifier {
        TriggerModifier::Command => matches!(
            key,
            Key::MetaLeft | Key::MetaRight | Key::Unknown(MAC_COMMAND_LEFT) | Key::Unknown(MAC_COMMAND_RIGHT)
        ),
        TriggerModifier::Control => matches!(
            key,
            Key::ControlLeft
                | Key::ControlRight
                | Key::Unknown(MAC_CONTROL_LEFT)
                | Key::Unknown(MAC_CONTROL_RIGHT)
        ),
        TriggerModifier::Alt => matches!(key, Key::Alt | Key::AltGr),
    }
}

/// Apply one input event to the double-tap machine.
/// Returns cursor coordinates when a clean double-tap is recognized.
pub fn apply_event(
    state: &mut MachineState,
    event: EventType,
    cfg: &HotkeySettings,
    now: Instant,
) -> Option<(f64, f64)> {
    if cfg.paused {
        return None;
    }

    match event {
        EventType::MouseMove { x, y } => {
            state.pointer_x = x;
            state.pointer_y = y;
            None
        }
        EventType::ButtonPress(_) => {
            if matches!(
                state.stage,
                Stage::HoldingFirst | Stage::HoldingSecond | Stage::AwaitingSecond
            ) {
                state.contaminated = true;
                state.stage = Stage::Idle;
                state.first_release_at = None;
                state.second_press_at = None;
            }
            None
        }
        EventType::KeyPress(key) => {
            if matches_modifier(key, cfg.modifier) {
                match state.stage {
                    // Key repeat while holding must not reset the second tap back to the first.
                    Stage::Idle => {
                        state.stage = Stage::HoldingFirst;
                        state.contaminated = false;
                        state.first_release_at = None;
                        state.second_press_at = None;
                    }
                    Stage::AwaitingSecond => {
                        state.stage = Stage::HoldingSecond;
                        state.second_press_at = Some(now);
                        state.contaminated = false;
                    }
                    Stage::HoldingFirst | Stage::HoldingSecond => {}
                }
            } else if matches!(
                state.stage,
                Stage::HoldingFirst | Stage::HoldingSecond | Stage::AwaitingSecond
            ) {
                state.contaminated = true;
                state.stage = Stage::Idle;
                state.first_release_at = None;
                state.second_press_at = None;
            }
            None
        }
        EventType::KeyRelease(key) => {
            if !matches_modifier(key, cfg.modifier) {
                return None;
            }

            match state.stage {
                Stage::HoldingFirst => {
                    if state.contaminated {
                        state.stage = Stage::Idle;
                        state.contaminated = false;
                    } else {
                        state.first_release_at = Some(now);
                        state.stage = Stage::AwaitingSecond;
                    }
                    None
                }
                Stage::HoldingSecond => {
                    if state.contaminated {
                        state.stage = Stage::Idle;
                        state.contaminated = false;
                        state.first_release_at = None;
                        state.second_press_at = None;
                        return None;
                    }
                    if let Some(first_at) = state.first_release_at {
                        let release_delta = now.duration_since(first_at);
                        let window = Duration::from_millis(cfg.interval_ms);
                        let inter_tap_gap = state
                            .second_press_at
                            .map(|p| p.duration_since(first_at))
                            .unwrap_or(release_delta);
                        let second_hold = state
                            .second_press_at
                            .map(|p| now.duration_since(p))
                            .unwrap_or_default();

                        let within_window =
                            release_delta <= window || (inter_tap_gap <= window && second_hold <= Duration::from_millis(280));

                        if within_window {
                            let coords = (state.pointer_x, state.pointer_y);
                            state.stage = Stage::Idle;
                            state.first_release_at = None;
                            state.second_press_at = None;
                            return Some(coords);
                        }
                    }
                    // Window expired; promote second release to a fresh first tap.
                    state.first_release_at = Some(now);
                    state.second_press_at = None;
                    state.stage = Stage::AwaitingSecond;
                    None
                }
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdev::Button;

    fn settings() -> HotkeySettings {
        HotkeySettings {
            modifier: TriggerModifier::Command,
            interval_ms: 300,
            paused: false,
        }
    }

    fn tap_once(state: &mut MachineState, cfg: &HotkeySettings, t0: Instant, key: Key) -> Option<(f64, f64)> {
        let a = apply_event(state, EventType::KeyPress(key), cfg, t0);
        assert!(a.is_none());
        apply_event(
            state,
            EventType::KeyRelease(key),
            cfg,
            t0 + Duration::from_millis(40),
        )
    }

    #[test]
    fn double_tap_left_command_triggers() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        apply_event(&mut state, EventType::MouseMove { x: 120.0, y: 80.0 }, &cfg, t0);
        assert!(tap_once(&mut state, &cfg, t0, Key::MetaLeft).is_none());
        let fired = tap_once(&mut state, &cfg, t0 + Duration::from_millis(120), Key::MetaLeft);
        assert_eq!(fired, Some((120.0, 80.0)));
        assert_eq!(state.stage, Stage::Idle);
    }

    #[test]
    fn left_then_right_command_counts_as_double_tap() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        assert!(tap_once(&mut state, &cfg, t0, Key::MetaLeft).is_none());
        let fired = tap_once(&mut state, &cfg, t0 + Duration::from_millis(100), Key::MetaRight);
        assert!(fired.is_some());
    }

    #[test]
    fn macos_unknown_command_keycodes_match() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        assert!(tap_once(&mut state, &cfg, t0, Key::Unknown(55)).is_none());
        assert!(tap_once(&mut state, &cfg, t0 + Duration::from_millis(80), Key::Unknown(54)).is_some());
    }

    #[test]
    fn key_repeat_on_second_press_does_not_reset_to_first_tap() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        assert!(tap_once(&mut state, &cfg, t0, Key::MetaLeft).is_none());
        assert!(apply_event(&mut state, EventType::KeyPress(Key::MetaLeft), &cfg, t0 + Duration::from_millis(80)).is_none());
        assert_eq!(state.stage, Stage::HoldingSecond);
        // Repeats must stay on the second hold, not fall back to HoldingFirst.
        assert!(apply_event(&mut state, EventType::KeyPress(Key::MetaLeft), &cfg, t0 + Duration::from_millis(100)).is_none());
        assert_eq!(state.stage, Stage::HoldingSecond);
        let fired = apply_event(
            &mut state,
            EventType::KeyRelease(Key::MetaLeft),
            &cfg,
            t0 + Duration::from_millis(140),
        );
        assert!(fired.is_some());
    }

    #[test]
    fn interval_expiry_does_not_fire() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        assert!(tap_once(&mut state, &cfg, t0, Key::MetaLeft).is_none());
        let fired = tap_once(&mut state, &cfg, t0 + Duration::from_millis(500), Key::MetaLeft);
        assert!(fired.is_none());
        assert_eq!(state.stage, Stage::AwaitingSecond);
    }

    #[test]
    fn other_key_or_mouse_cancels_sequence() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        apply_event(&mut state, EventType::KeyPress(Key::MetaLeft), &cfg, t0);
        apply_event(&mut state, EventType::KeyPress(Key::KeyA), &cfg, t0 + Duration::from_millis(10));
        assert_eq!(state.stage, Stage::Idle);
        apply_event(&mut state, EventType::KeyPress(Key::MetaLeft), &cfg, t0 + Duration::from_millis(20));
        apply_event(&mut state, EventType::KeyRelease(Key::MetaLeft), &cfg, t0 + Duration::from_millis(40));
        apply_event(&mut state, EventType::ButtonPress(Button::Left), &cfg, t0 + Duration::from_millis(50));
        assert_eq!(state.stage, Stage::Idle);
        let fired = tap_once(&mut state, &cfg, t0 + Duration::from_millis(80), Key::MetaLeft);
        assert!(fired.is_none());
    }

    #[test]
    fn paused_listener_never_triggers() {
        let mut cfg = settings();
        cfg.paused = true;
        let mut state = MachineState::default();
        let t0 = Instant::now();
        assert!(tap_once(&mut state, &cfg, t0, Key::MetaLeft).is_none());
        assert!(tap_once(&mut state, &cfg, t0 + Duration::from_millis(80), Key::MetaLeft).is_none());
        assert_eq!(state.stage, Stage::Idle);
    }
}
