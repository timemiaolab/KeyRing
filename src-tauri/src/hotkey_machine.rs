use crate::input_event::{InputEvent, PhysicalKey};
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

/// Historical rdev Unknown keycodes for Command / Control, kept so existing
/// regressions still pass even after the mac listener stopped using rdev.
const MAC_COMMAND_LEFT: u32 = 55;
const MAC_COMMAND_RIGHT: u32 = 54;
const MAC_CONTROL_LEFT: u32 = 59;
const MAC_CONTROL_RIGHT: u32 = 62;

pub fn matches_modifier(key: PhysicalKey, modifier: TriggerModifier) -> bool {
    match modifier {
        TriggerModifier::Command => matches!(
            key,
            PhysicalKey::MetaLeft
                | PhysicalKey::MetaRight
                | PhysicalKey::Unknown(MAC_COMMAND_LEFT)
                | PhysicalKey::Unknown(MAC_COMMAND_RIGHT)
        ),
        TriggerModifier::Control => matches!(
            key,
            PhysicalKey::ControlLeft
                | PhysicalKey::ControlRight
                | PhysicalKey::Unknown(MAC_CONTROL_LEFT)
                | PhysicalKey::Unknown(MAC_CONTROL_RIGHT)
        ),
        TriggerModifier::Alt => matches!(key, PhysicalKey::Alt | PhysicalKey::AltGr),
    }
}

/// Apply one input event to the double-tap machine.
/// Returns cursor coordinates when a clean double-tap is recognized.
pub fn apply_event(
    state: &mut MachineState,
    event: InputEvent,
    cfg: &HotkeySettings,
    now: Instant,
) -> Option<(f64, f64)> {
    if cfg.paused {
        return None;
    }

    match event {
        InputEvent::MouseMove { x, y } => {
            state.pointer_x = x;
            state.pointer_y = y;
            None
        }
        InputEvent::ButtonPress => {
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
        InputEvent::KeyPress(key) => {
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
        InputEvent::KeyRelease(key) => {
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

                        let within_window = release_delta <= window
                            || (inter_tap_gap <= window
                                && second_hold <= Duration::from_millis(280));

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
    use crate::macos_event::{
        decode_cg_event, DecodeAction, ModifierTracker, EVENT_FLAGS_CHANGED, FLAG_COMMAND,
        FLAG_LEFT_COMMAND, FLAG_RIGHT_COMMAND, KEY_LEFT_COMMAND, KEY_RIGHT_COMMAND,
    };

    fn settings() -> HotkeySettings {
        HotkeySettings {
            modifier: TriggerModifier::Command,
            interval_ms: 300,
            paused: false,
        }
    }

    fn tap_once(
        state: &mut MachineState,
        cfg: &HotkeySettings,
        t0: Instant,
        key: PhysicalKey,
    ) -> Option<(f64, f64)> {
        let a = apply_event(state, InputEvent::KeyPress(key), cfg, t0);
        assert!(a.is_none());
        apply_event(
            state,
            InputEvent::KeyRelease(key),
            cfg,
            t0 + Duration::from_millis(40),
        )
    }

    #[test]
    fn double_tap_left_command_triggers() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        apply_event(
            &mut state,
            InputEvent::MouseMove { x: 120.0, y: 80.0 },
            &cfg,
            t0,
        );
        assert!(tap_once(&mut state, &cfg, t0, PhysicalKey::MetaLeft).is_none());
        let fired = tap_once(
            &mut state,
            &cfg,
            t0 + Duration::from_millis(120),
            PhysicalKey::MetaLeft,
        );
        assert_eq!(fired, Some((120.0, 80.0)));
        assert_eq!(state.stage, Stage::Idle);
    }

    #[test]
    fn left_then_right_command_counts_as_double_tap() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        assert!(tap_once(&mut state, &cfg, t0, PhysicalKey::MetaLeft).is_none());
        let fired = tap_once(
            &mut state,
            &cfg,
            t0 + Duration::from_millis(100),
            PhysicalKey::MetaRight,
        );
        assert!(fired.is_some());
    }

    #[test]
    fn macos_unknown_command_keycodes_match() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        assert!(tap_once(&mut state, &cfg, t0, PhysicalKey::Unknown(55)).is_none());
        assert!(tap_once(
            &mut state,
            &cfg,
            t0 + Duration::from_millis(80),
            PhysicalKey::Unknown(54)
        )
        .is_some());
    }

    #[test]
    fn key_repeat_on_second_press_does_not_reset_to_first_tap() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        assert!(tap_once(&mut state, &cfg, t0, PhysicalKey::MetaLeft).is_none());
        assert!(apply_event(
            &mut state,
            InputEvent::KeyPress(PhysicalKey::MetaLeft),
            &cfg,
            t0 + Duration::from_millis(80)
        )
        .is_none());
        assert_eq!(state.stage, Stage::HoldingSecond);
        // Repeats must stay on the second hold, not fall back to HoldingFirst.
        assert!(apply_event(
            &mut state,
            InputEvent::KeyPress(PhysicalKey::MetaLeft),
            &cfg,
            t0 + Duration::from_millis(100)
        )
        .is_none());
        assert_eq!(state.stage, Stage::HoldingSecond);
        let fired = apply_event(
            &mut state,
            InputEvent::KeyRelease(PhysicalKey::MetaLeft),
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
        assert!(tap_once(&mut state, &cfg, t0, PhysicalKey::MetaLeft).is_none());
        let fired = tap_once(
            &mut state,
            &cfg,
            t0 + Duration::from_millis(500),
            PhysicalKey::MetaLeft,
        );
        assert!(fired.is_none());
        assert_eq!(state.stage, Stage::AwaitingSecond);
    }

    #[test]
    fn other_key_or_mouse_cancels_sequence() {
        let cfg = settings();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        apply_event(
            &mut state,
            InputEvent::KeyPress(PhysicalKey::MetaLeft),
            &cfg,
            t0,
        );
        apply_event(
            &mut state,
            InputEvent::KeyPress(PhysicalKey::Other),
            &cfg,
            t0 + Duration::from_millis(10),
        );
        assert_eq!(state.stage, Stage::Idle);
        apply_event(
            &mut state,
            InputEvent::KeyPress(PhysicalKey::MetaLeft),
            &cfg,
            t0 + Duration::from_millis(20),
        );
        apply_event(
            &mut state,
            InputEvent::KeyRelease(PhysicalKey::MetaLeft),
            &cfg,
            t0 + Duration::from_millis(40),
        );
        apply_event(
            &mut state,
            InputEvent::ButtonPress,
            &cfg,
            t0 + Duration::from_millis(50),
        );
        assert_eq!(state.stage, Stage::Idle);
        let fired = tap_once(
            &mut state,
            &cfg,
            t0 + Duration::from_millis(80),
            PhysicalKey::MetaLeft,
        );
        assert!(fired.is_none());
    }

    #[test]
    fn paused_listener_never_triggers() {
        let mut cfg = settings();
        cfg.paused = true;
        let mut state = MachineState::default();
        let t0 = Instant::now();
        assert!(tap_once(&mut state, &cfg, t0, PhysicalKey::MetaLeft).is_none());
        assert!(tap_once(
            &mut state,
            &cfg,
            t0 + Duration::from_millis(80),
            PhysicalKey::MetaLeft
        )
        .is_none());
        assert_eq!(state.stage, Stage::Idle);
    }

    fn feed_flags(
        tracker: &mut ModifierTracker,
        state: &mut MachineState,
        cfg: &HotkeySettings,
        now: Instant,
        code: u16,
        flags: u64,
    ) -> Option<(f64, f64)> {
        match decode_cg_event(EVENT_FLAGS_CHANGED, code, flags, None, tracker) {
            DecodeAction::Event(event) => apply_event(state, event, cfg, now),
            other => {
                panic!("expected a physical event, got {other:?}");
            }
        }
    }

    #[test]
    fn flags_changed_double_command_triggers_state_machine() {
        let cfg = settings();
        let mut tracker = ModifierTracker::default();
        let mut state = MachineState::default();
        let t0 = Instant::now();
        apply_event(
            &mut state,
            InputEvent::MouseMove { x: 42.0, y: 7.0 },
            &cfg,
            t0,
        );

        assert!(feed_flags(
            &mut tracker,
            &mut state,
            &cfg,
            t0,
            KEY_LEFT_COMMAND,
            FLAG_COMMAND | FLAG_LEFT_COMMAND
        )
        .is_none());
        assert!(feed_flags(
            &mut tracker,
            &mut state,
            &cfg,
            t0 + Duration::from_millis(40),
            KEY_LEFT_COMMAND,
            0
        )
        .is_none());
        assert_eq!(state.stage, Stage::AwaitingSecond);

        assert!(feed_flags(
            &mut tracker,
            &mut state,
            &cfg,
            t0 + Duration::from_millis(120),
            KEY_LEFT_COMMAND,
            FLAG_COMMAND | FLAG_LEFT_COMMAND
        )
        .is_none());
        let fired = feed_flags(
            &mut tracker,
            &mut state,
            &cfg,
            t0 + Duration::from_millis(160),
            KEY_LEFT_COMMAND,
            0,
        );
        assert_eq!(fired, Some((42.0, 7.0)));
        assert_eq!(state.stage, Stage::Idle);
    }

    #[test]
    fn releasing_left_command_while_right_is_held_does_not_count_as_a_tap() {
        // Chord: left down, right down, left up. The left release must not
        // complete a double-tap just because Command is still held on the right.
        let cfg = settings();
        let mut tracker = ModifierTracker::default();
        let mut state = MachineState::default();
        let t0 = Instant::now();

        assert!(feed_flags(
            &mut tracker,
            &mut state,
            &cfg,
            t0,
            KEY_LEFT_COMMAND,
            FLAG_COMMAND | FLAG_LEFT_COMMAND
        )
        .is_none());
        assert!(feed_flags(
            &mut tracker,
            &mut state,
            &cfg,
            t0 + Duration::from_millis(20),
            KEY_RIGHT_COMMAND,
            FLAG_COMMAND | FLAG_LEFT_COMMAND | FLAG_RIGHT_COMMAND
        )
        .is_none());
        // The second Command while already HoldingFirst is a repeat, not tap 2.
        assert_eq!(state.stage, Stage::HoldingFirst);
        assert!(feed_flags(
            &mut tracker,
            &mut state,
            &cfg,
            t0 + Duration::from_millis(40),
            KEY_LEFT_COMMAND,
            FLAG_COMMAND | FLAG_RIGHT_COMMAND
        )
        .is_none());
        assert_eq!(state.stage, Stage::AwaitingSecond);
        assert!(tracker.right_cmd && !tracker.left_cmd);
    }
}
