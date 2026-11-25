//! Platform-neutral input events for the double-tap state machine.
//! macOS maps these from physical keycodes and modifier flags only.
//! Windows and Linux map them from rdev after that crate has already
//! delivered an event to our callback.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalKey {
    MetaLeft,
    MetaRight,
    ControlLeft,
    ControlRight,
    Alt,
    AltGr,
    Other,
    Unknown(u32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    KeyPress(PhysicalKey),
    KeyRelease(PhysicalKey),
    MouseMove { x: f64, y: f64 },
    ButtonPress,
    ButtonRelease,
    Wheel,
}

#[cfg(not(target_os = "macos"))]
pub fn from_rdev(event: rdev::EventType) -> Option<InputEvent> {
    Some(match event {
        rdev::EventType::KeyPress(key) => InputEvent::KeyPress(map_rdev_key(key)),
        rdev::EventType::KeyRelease(key) => InputEvent::KeyRelease(map_rdev_key(key)),
        rdev::EventType::MouseMove { x, y } => InputEvent::MouseMove { x, y },
        rdev::EventType::ButtonPress(_) => InputEvent::ButtonPress,
        rdev::EventType::ButtonRelease(_) => InputEvent::ButtonRelease,
        rdev::EventType::Wheel { .. } => InputEvent::Wheel,
    })
}

#[cfg(not(target_os = "macos"))]
fn map_rdev_key(key: rdev::Key) -> PhysicalKey {
    match key {
        rdev::Key::MetaLeft => PhysicalKey::MetaLeft,
        rdev::Key::MetaRight => PhysicalKey::MetaRight,
        rdev::Key::ControlLeft => PhysicalKey::ControlLeft,
        rdev::Key::ControlRight => PhysicalKey::ControlRight,
        rdev::Key::Alt => PhysicalKey::Alt,
        rdev::Key::AltGr => PhysicalKey::AltGr,
        rdev::Key::Unknown(code) => match code {
            55 => PhysicalKey::MetaLeft,
            54 => PhysicalKey::MetaRight,
            59 => PhysicalKey::ControlLeft,
            62 => PhysicalKey::ControlRight,
            other => PhysicalKey::Unknown(other),
        },
        _ => PhysicalKey::Other,
    }
}
