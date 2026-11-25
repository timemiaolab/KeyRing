//! Decode macOS CGEvent fields into physical key events.
//!
//! This module is deliberately pure: it never calls Text Input Source /
//! HIToolbox APIs. Double-Command only needs virtual keycodes and modifier
//! flags, which are safe to read from a background event-tap thread.

use crate::input_event::{InputEvent, PhysicalKey};

/// Virtual keycodes (HIToolbox kVK_*).
pub const KEY_RIGHT_COMMAND: u16 = 54;
pub const KEY_LEFT_COMMAND: u16 = 55;
pub const KEY_LEFT_SHIFT: u16 = 56;
pub const KEY_CAPS_LOCK: u16 = 57;
pub const KEY_LEFT_OPTION: u16 = 58;
pub const KEY_LEFT_CONTROL: u16 = 59;
pub const KEY_RIGHT_SHIFT: u16 = 60;
pub const KEY_RIGHT_OPTION: u16 = 61;
pub const KEY_RIGHT_CONTROL: u16 = 62;

/// CGEvent / NSEvent modifier bits.
pub const FLAG_COMMAND: u64 = 1 << 20;
pub const FLAG_CONTROL: u64 = 1 << 18;
pub const FLAG_OPTION: u64 = 1 << 19;
pub const FLAG_SHIFT: u64 = 1 << 17;

/// Device-dependent bits from IOLLEvent.h, present on modern macOS CGEvents.
pub const FLAG_LEFT_COMMAND: u64 = 0x0000_0008;
pub const FLAG_RIGHT_COMMAND: u64 = 0x0000_0010;
pub const FLAG_LEFT_CONTROL: u64 = 0x0000_0001;
pub const FLAG_RIGHT_CONTROL: u64 = 0x0000_2000;
pub const FLAG_LEFT_OPTION: u64 = 0x0000_0020;
pub const FLAG_RIGHT_OPTION: u64 = 0x0000_0040;
pub const FLAG_LEFT_SHIFT: u64 = 0x0000_0002;
pub const FLAG_RIGHT_SHIFT: u64 = 0x0000_0004;

pub const EVENT_LEFT_MOUSE_DOWN: u32 = 1;
pub const EVENT_RIGHT_MOUSE_DOWN: u32 = 3;
pub const EVENT_OTHER_MOUSE_DOWN: u32 = 25;
pub const EVENT_MOUSE_MOVED: u32 = 5;
pub const EVENT_LEFT_MOUSE_DRAGGED: u32 = 6;
pub const EVENT_RIGHT_MOUSE_DRAGGED: u32 = 7;
pub const EVENT_KEY_DOWN: u32 = 10;
pub const EVENT_KEY_UP: u32 = 11;
pub const EVENT_FLAGS_CHANGED: u32 = 12;
pub const EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFF_FFFE;
pub const EVENT_TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFF_FFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierTracker {
    pub left_cmd: bool,
    pub right_cmd: bool,
    pub left_ctrl: bool,
    pub right_ctrl: bool,
    pub left_alt: bool,
    pub right_alt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecodeAction {
    Event(InputEvent),
    ReenableTap,
    Ignore,
}

pub fn physical_key_from_code(code: u16) -> PhysicalKey {
    match code {
        KEY_LEFT_COMMAND => PhysicalKey::MetaLeft,
        KEY_RIGHT_COMMAND => PhysicalKey::MetaRight,
        KEY_LEFT_CONTROL => PhysicalKey::ControlLeft,
        KEY_RIGHT_CONTROL => PhysicalKey::ControlRight,
        KEY_LEFT_OPTION => PhysicalKey::Alt,
        KEY_RIGHT_OPTION => PhysicalKey::AltGr,
        _ => PhysicalKey::Unknown(u32::from(code)),
    }
}

fn family_device_bits(code: u16) -> (u64, u64, u64) {
    match code {
        KEY_LEFT_COMMAND | KEY_RIGHT_COMMAND => {
            (FLAG_LEFT_COMMAND, FLAG_RIGHT_COMMAND, FLAG_COMMAND)
        }
        KEY_LEFT_CONTROL | KEY_RIGHT_CONTROL => {
            (FLAG_LEFT_CONTROL, FLAG_RIGHT_CONTROL, FLAG_CONTROL)
        }
        KEY_LEFT_OPTION | KEY_RIGHT_OPTION => (FLAG_LEFT_OPTION, FLAG_RIGHT_OPTION, FLAG_OPTION),
        KEY_LEFT_SHIFT | KEY_RIGHT_SHIFT => (FLAG_LEFT_SHIFT, FLAG_RIGHT_SHIFT, FLAG_SHIFT),
        _ => (0, 0, 0),
    }
}

fn is_left_code(code: u16) -> bool {
    matches!(
        code,
        KEY_LEFT_COMMAND | KEY_LEFT_CONTROL | KEY_LEFT_OPTION | KEY_LEFT_SHIFT
    )
}

fn get_side(tracker: &ModifierTracker, code: u16) -> Option<bool> {
    match code {
        KEY_LEFT_COMMAND => Some(tracker.left_cmd),
        KEY_RIGHT_COMMAND => Some(tracker.right_cmd),
        KEY_LEFT_CONTROL => Some(tracker.left_ctrl),
        KEY_RIGHT_CONTROL => Some(tracker.right_ctrl),
        KEY_LEFT_OPTION => Some(tracker.left_alt),
        KEY_RIGHT_OPTION => Some(tracker.right_alt),
        _ => None,
    }
}

fn set_side(tracker: &mut ModifierTracker, code: u16, down: bool) {
    match code {
        KEY_LEFT_COMMAND => tracker.left_cmd = down,
        KEY_RIGHT_COMMAND => tracker.right_cmd = down,
        KEY_LEFT_CONTROL => tracker.left_ctrl = down,
        KEY_RIGHT_CONTROL => tracker.right_ctrl = down,
        KEY_LEFT_OPTION => tracker.left_alt = down,
        KEY_RIGHT_OPTION => tracker.right_alt = down,
        _ => {}
    }
}

/// Determine whether `code`'s physical modifier is down in `flags`, using
/// device-specific bits when present and falling back to the keycode plus
/// the previous per-key state when only the family mask is available.
pub fn modifier_is_down(code: u16, flags: u64, was_down: bool) -> bool {
    let (left_bit, right_bit, family_bit) = family_device_bits(code);
    if left_bit == 0 {
        return false;
    }
    let device_bits = flags & (left_bit | right_bit);
    if device_bits != 0 || (flags & family_bit == 0 && was_down) {
        let bit = if is_left_code(code) {
            left_bit
        } else {
            right_bit
        };
        if flags & family_bit == 0 {
            return false;
        }
        if device_bits != 0 {
            return flags & bit != 0;
        }
    }
    if flags & family_bit == 0 {
        return false;
    }
    // Family still held. This FlagsChanged is for `code`, so invert only
    // that side: a previously-up key is a press, a previously-down key is
    // a release while the other side remains held.
    !was_down
}

/// Decode one CGEvent into a physical input action.
///
/// `keycode` and `flags` must come from `CGEventGetIntegerValueField`
/// / `CGEventGetFlags`. No input-source or Unicode translation is performed.
pub fn decode_cg_event(
    typ: u32,
    keycode: u16,
    flags: u64,
    location: Option<(f64, f64)>,
    tracker: &mut ModifierTracker,
) -> DecodeAction {
    match typ {
        EVENT_TAP_DISABLED_BY_TIMEOUT | EVENT_TAP_DISABLED_BY_USER_INPUT => {
            DecodeAction::ReenableTap
        }
        EVENT_MOUSE_MOVED | EVENT_LEFT_MOUSE_DRAGGED | EVENT_RIGHT_MOUSE_DRAGGED => {
            if let Some((x, y)) = location {
                DecodeAction::Event(InputEvent::MouseMove { x, y })
            } else {
                DecodeAction::Ignore
            }
        }
        EVENT_LEFT_MOUSE_DOWN | EVENT_RIGHT_MOUSE_DOWN | EVENT_OTHER_MOUSE_DOWN => {
            DecodeAction::Event(InputEvent::ButtonPress)
        }
        EVENT_KEY_DOWN => {
            DecodeAction::Event(InputEvent::KeyPress(physical_key_from_code(keycode)))
        }
        EVENT_KEY_UP => {
            DecodeAction::Event(InputEvent::KeyRelease(physical_key_from_code(keycode)))
        }
        EVENT_FLAGS_CHANGED => decode_flags_changed(keycode, flags, tracker),
        _ => DecodeAction::Ignore,
    }
}

pub fn decode_flags_changed(
    keycode: u16,
    flags: u64,
    tracker: &mut ModifierTracker,
) -> DecodeAction {
    let Some(was_down) = get_side(tracker, keycode) else {
        // Caps lock or an unknown modifier: treat as a non-trigger key so a
        // chord involving it can cancel a pending double-tap, but only on
        // a transition we can see via the family flag is not required here.
        return DecodeAction::Event(InputEvent::KeyPress(physical_key_from_code(keycode)));
    };
    let now_down = modifier_is_down(keycode, flags, was_down);
    if now_down == was_down {
        return DecodeAction::Ignore;
    }
    set_side(tracker, keycode, now_down);
    let key = physical_key_from_code(keycode);
    if now_down {
        DecodeAction::Event(InputEvent::KeyPress(key))
    } else {
        DecodeAction::Event(InputEvent::KeyRelease(key))
    }
}

/// Event mask for a listen-only tap: modifiers, keys, mouse buttons, movement,
/// and the tap-disabled notifications. No text or IME events are required.
pub fn listen_event_mask() -> u64 {
    // Disabled-by-timeout / user-input are delivered to the callback even when
    // they are not in the mask. Do not shift those 0xFFFFFFFE values as bits.
    (1u64 << EVENT_LEFT_MOUSE_DOWN)
        | (1u64 << EVENT_RIGHT_MOUSE_DOWN)
        | (1u64 << EVENT_OTHER_MOUSE_DOWN)
        | (1u64 << EVENT_MOUSE_MOVED)
        | (1u64 << EVENT_KEY_DOWN)
        | (1u64 << EVENT_KEY_UP)
        | (1u64 << EVENT_FLAGS_CHANGED)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(typ: u32, code: u16, flags: u64, tracker: &mut ModifierTracker) -> DecodeAction {
        decode_cg_event(typ, code, flags, None, tracker)
    }

    #[test]
    fn left_command_press_and_release_with_device_bits() {
        let mut tracker = ModifierTracker::default();
        let press = apply(
            EVENT_FLAGS_CHANGED,
            KEY_LEFT_COMMAND,
            FLAG_COMMAND | FLAG_LEFT_COMMAND,
            &mut tracker,
        );
        assert_eq!(
            press,
            DecodeAction::Event(InputEvent::KeyPress(PhysicalKey::MetaLeft))
        );
        assert!(tracker.left_cmd);
        assert!(!tracker.right_cmd);

        let release = apply(EVENT_FLAGS_CHANGED, KEY_LEFT_COMMAND, 0, &mut tracker);
        assert_eq!(
            release,
            DecodeAction::Event(InputEvent::KeyRelease(PhysicalKey::MetaLeft))
        );
        assert!(!tracker.left_cmd);
    }

    #[test]
    fn right_command_is_independent_of_left() {
        let mut tracker = ModifierTracker::default();
        apply(
            EVENT_FLAGS_CHANGED,
            KEY_LEFT_COMMAND,
            FLAG_COMMAND | FLAG_LEFT_COMMAND,
            &mut tracker,
        );
        let press_right = apply(
            EVENT_FLAGS_CHANGED,
            KEY_RIGHT_COMMAND,
            FLAG_COMMAND | FLAG_LEFT_COMMAND | FLAG_RIGHT_COMMAND,
            &mut tracker,
        );
        assert_eq!(
            press_right,
            DecodeAction::Event(InputEvent::KeyPress(PhysicalKey::MetaRight))
        );
        assert!(tracker.left_cmd && tracker.right_cmd);

        let release_left = apply(
            EVENT_FLAGS_CHANGED,
            KEY_LEFT_COMMAND,
            FLAG_COMMAND | FLAG_RIGHT_COMMAND,
            &mut tracker,
        );
        assert_eq!(
            release_left,
            DecodeAction::Event(InputEvent::KeyRelease(PhysicalKey::MetaLeft))
        );
        assert!(!tracker.left_cmd && tracker.right_cmd);

        let release_right = apply(EVENT_FLAGS_CHANGED, KEY_RIGHT_COMMAND, 0, &mut tracker);
        assert_eq!(
            release_right,
            DecodeAction::Event(InputEvent::KeyRelease(PhysicalKey::MetaRight))
        );
        assert!(!tracker.right_cmd);
    }

    #[test]
    fn family_flag_only_press_and_release_uses_keycode() {
        let mut tracker = ModifierTracker::default();
        let press = apply(
            EVENT_FLAGS_CHANGED,
            KEY_LEFT_COMMAND,
            FLAG_COMMAND,
            &mut tracker,
        );
        assert_eq!(
            press,
            DecodeAction::Event(InputEvent::KeyPress(PhysicalKey::MetaLeft))
        );
        let release = apply(EVENT_FLAGS_CHANGED, KEY_LEFT_COMMAND, 0, &mut tracker);
        assert_eq!(
            release,
            DecodeAction::Event(InputEvent::KeyRelease(PhysicalKey::MetaLeft))
        );
    }

    #[test]
    fn family_flag_stays_set_when_releasing_one_of_two_commands() {
        let mut tracker = ModifierTracker::default();
        apply(
            EVENT_FLAGS_CHANGED,
            KEY_LEFT_COMMAND,
            FLAG_COMMAND,
            &mut tracker,
        );
        // Second command pressed: family flag stays set, no device bits.
        let press_right = apply(
            EVENT_FLAGS_CHANGED,
            KEY_RIGHT_COMMAND,
            FLAG_COMMAND,
            &mut tracker,
        );
        assert_eq!(
            press_right,
            DecodeAction::Event(InputEvent::KeyPress(PhysicalKey::MetaRight))
        );
        assert!(tracker.left_cmd && tracker.right_cmd);

        let release_left = apply(
            EVENT_FLAGS_CHANGED,
            KEY_LEFT_COMMAND,
            FLAG_COMMAND,
            &mut tracker,
        );
        assert_eq!(
            release_left,
            DecodeAction::Event(InputEvent::KeyRelease(PhysicalKey::MetaLeft))
        );
        assert!(!tracker.left_cmd && tracker.right_cmd);
    }

    #[test]
    fn unchanged_flags_are_ignored() {
        let mut tracker = ModifierTracker::default();
        apply(
            EVENT_FLAGS_CHANGED,
            KEY_LEFT_COMMAND,
            FLAG_COMMAND | FLAG_LEFT_COMMAND,
            &mut tracker,
        );
        let again = apply(
            EVENT_FLAGS_CHANGED,
            KEY_LEFT_COMMAND,
            FLAG_COMMAND | FLAG_LEFT_COMMAND,
            &mut tracker,
        );
        assert_eq!(again, DecodeAction::Ignore);
        assert!(tracker.left_cmd);
    }

    #[test]
    fn regular_key_and_mouse_are_physical_only() {
        let mut tracker = ModifierTracker::default();
        let key = apply(EVENT_KEY_DOWN, 0, 0, &mut tracker);
        assert_eq!(
            key,
            DecodeAction::Event(InputEvent::KeyPress(PhysicalKey::Unknown(0)))
        );
        let mouse = apply(EVENT_LEFT_MOUSE_DOWN, 0, 0, &mut tracker);
        assert_eq!(mouse, DecodeAction::Event(InputEvent::ButtonPress));
    }

    #[test]
    fn mouse_move_uses_provided_location() {
        let mut tracker = ModifierTracker::default();
        let action = decode_cg_event(EVENT_MOUSE_MOVED, 0, 0, Some((-120.5, 40.0)), &mut tracker);
        assert_eq!(
            action,
            DecodeAction::Event(InputEvent::MouseMove { x: -120.5, y: 40.0 })
        );
    }

    #[test]
    fn disabled_tap_requests_reenable_instead_of_panicking() {
        let mut tracker = ModifierTracker::default();
        assert_eq!(
            apply(EVENT_TAP_DISABLED_BY_TIMEOUT, 0, 0, &mut tracker),
            DecodeAction::ReenableTap
        );
        assert_eq!(
            apply(EVENT_TAP_DISABLED_BY_USER_INPUT, 0, 0, &mut tracker),
            DecodeAction::ReenableTap
        );
    }

    #[test]
    fn control_and_option_sides_decode() {
        let mut tracker = ModifierTracker::default();
        assert_eq!(
            apply(
                EVENT_FLAGS_CHANGED,
                KEY_LEFT_CONTROL,
                FLAG_CONTROL | FLAG_LEFT_CONTROL,
                &mut tracker
            ),
            DecodeAction::Event(InputEvent::KeyPress(PhysicalKey::ControlLeft))
        );
        assert_eq!(
            apply(
                EVENT_FLAGS_CHANGED,
                KEY_RIGHT_CONTROL,
                FLAG_CONTROL | FLAG_LEFT_CONTROL | FLAG_RIGHT_CONTROL,
                &mut tracker
            ),
            DecodeAction::Event(InputEvent::KeyPress(PhysicalKey::ControlRight))
        );
        assert_eq!(
            apply(
                EVENT_FLAGS_CHANGED,
                KEY_LEFT_OPTION,
                FLAG_OPTION | FLAG_LEFT_OPTION,
                &mut tracker
            ),
            DecodeAction::Event(InputEvent::KeyPress(PhysicalKey::Alt))
        );
    }

    #[test]
    fn physical_key_table_matches_known_virtual_keycodes() {
        assert_eq!(physical_key_from_code(55), PhysicalKey::MetaLeft);
        assert_eq!(physical_key_from_code(54), PhysicalKey::MetaRight);
        assert_eq!(physical_key_from_code(59), PhysicalKey::ControlLeft);
        assert_eq!(physical_key_from_code(62), PhysicalKey::ControlRight);
    }
}
