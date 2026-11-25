//! Background, listen-only CGEventTap for physical modifier keys.
//!
//! This deliberately does not use rdev. rdev 0.5.3's macOS `raw_callback`
//! calls `Keyboard::string_from_code` (TIS/HIToolbox) *before* invoking the
//! user callback, which aborts on a dispatch queue assertion when run off
//! the main thread. Double-Command does not need character translation.
//!
//! The tap is ListenOnly and always returns the original event, so we never
//! swallow input. The callback must not panic or block: decoded events are
//! forwarded over a channel, and a recognized double-tap is handed to a
//! worker thread.

use crate::hotkey_machine::{apply_event, HotkeySettings, MachineState};
use crate::macos_event::{
    decode_cg_event, listen_event_mask, DecodeAction, ModifierTracker, EVENT_FLAGS_CHANGED,
    EVENT_KEY_DOWN, EVENT_KEY_UP,
};
use std::ffi::c_void;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::time::Instant;

const K_CG_HID_EVENT_TAP: u32 = 0;
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const K_CG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;
const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;

#[repr(C)]
struct CGPoint {
    x: f64,
    y: f64,
}

type CFMachPortRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFRunLoopRef = *mut c_void;
type CFRunLoopMode = *const c_void;
type CFAllocatorRef = *const c_void;
type CFRunLoopTimerRef = *mut c_void;
type CFTimeInterval = f64;
type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    typ: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

#[link(name = "CoreGraphics", kind = "framework")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventTapIsEnabled(tap: CFMachPortRef) -> bool;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
    fn CFRunLoopRemoveSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
    fn CFRunLoopAddTimer(rl: CFRunLoopRef, timer: CFRunLoopTimerRef, mode: CFRunLoopMode);
    fn CFRunLoopTimerCreate(
        allocator: CFAllocatorRef,
        fire_date: CFTimeInterval,
        interval: CFTimeInterval,
        flags: usize,
        order: isize,
        callout: unsafe extern "C" fn(CFRunLoopTimerRef, *mut c_void),
        context: *mut CFRunLoopTimerContext,
    ) -> CFRunLoopTimerRef;
    fn CFAbsoluteTimeGetCurrent() -> CFTimeInterval;
    fn CFRunLoopRun();
    fn CFRunLoopStop(rl: CFRunLoopRef);
    fn CFRelease(cf: *const c_void);
    static kCFRunLoopCommonModes: CFRunLoopMode;
}

#[repr(C)]
struct CFRunLoopTimerContext {
    version: isize,
    info: *mut c_void,
    retain: *const c_void,
    release: *const c_void,
    copy_description: *const c_void,
}

struct TapState {
    tap: CFMachPortRef,
    runloop: CFRunLoopRef,
    running: Arc<AtomicBool>,
    settings: Arc<Mutex<HotkeySettings>>,
    machine: Mutex<MachineState>,
    tracker: Mutex<ModifierTracker>,
    trigger_tx: mpsc::SyncSender<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenError {
    EventTapUnavailable,
    RunLoopSourceUnavailable,
}

impl std::fmt::Display for ListenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListenError::EventTapUnavailable => {
                write!(
                    f,
                    "CGEventTapCreate returned null (accessibility missing or revoked)"
                )
            }
            ListenError::RunLoopSourceUnavailable => {
                write!(f, "CFMachPortCreateRunLoopSource returned null")
            }
        }
    }
}

/// Try to create and immediately tear down a listen-only event tap.
/// Used as a runtime smoke test: must not abort, even when permission is denied.
pub fn probe_event_tap() -> Result<(), ListenError> {
    unsafe {
        let tap = CGEventTapCreate(
            K_CG_HID_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
            listen_event_mask(),
            probe_callback,
            std::ptr::null_mut(),
        );
        if tap.is_null() {
            return Err(ListenError::EventTapUnavailable);
        }
        CGEventTapEnable(tap, false);
        CFRelease(tap as *const c_void);
        Ok(())
    }
}

unsafe extern "C" fn probe_callback(
    _proxy: CGEventTapProxy,
    _typ: u32,
    event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    event
}

unsafe extern "C" fn tap_callback(
    _proxy: CGEventTapProxy,
    typ: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    // Never panic out of this callback: a panic here would unwind through
    // CoreGraphics and can kill the process. Always return the original event.
    let Some(state) = (unsafe { (user_info as *mut TapState).as_mut() }) else {
        return event;
    };
    if event.is_null() {
        return event;
    }
    if !state.running.load(Ordering::Relaxed) {
        if !state.runloop.is_null() {
            unsafe { CFRunLoopStop(state.runloop) };
        }
        return event;
    }

    let keycode = if typ == EVENT_FLAGS_CHANGED || typ == EVENT_KEY_DOWN || typ == EVENT_KEY_UP {
        unsafe { CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) }.max(0) as u16
    } else {
        0
    };
    let flags = unsafe { CGEventGetFlags(event) };
    let location = {
        let point = unsafe { CGEventGetLocation(event) };
        Some((point.x, point.y))
    };

    let action = {
        let Ok(mut tracker) = state.tracker.lock() else {
            return event;
        };
        decode_cg_event(typ, keycode, flags, location, &mut tracker)
    };

    match action {
        DecodeAction::ReenableTap => {
            if !state.tap.is_null() {
                unsafe { CGEventTapEnable(state.tap, true) };
            }
        }
        DecodeAction::Event(input) => {
            let cfg = match state.settings.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => return event,
            };
            let mut machine = match state.machine.lock() {
                Ok(guard) => guard,
                Err(_) => return event,
            };
            if let Some(coords) = apply_event(&mut machine, input, &cfg, Instant::now()) {
                drop(machine);
                let _ = state.trigger_tx.try_send(coords);
            }
        }
        DecodeAction::Ignore => {}
    }
    event
}

unsafe extern "C" fn watchdog_timer(_timer: CFRunLoopTimerRef, info: *mut c_void) {
    let Some(state) = (unsafe { (info as *mut TapState).as_ref() }) else {
        return;
    };
    if !state.running.load(Ordering::Relaxed) {
        if !state.runloop.is_null() {
            unsafe { CFRunLoopStop(state.runloop) };
        }
        return;
    }
    if state.tap.is_null() {
        return;
    }
    let enabled = unsafe { CGEventTapIsEnabled(state.tap) };
    if !enabled {
        unsafe { CGEventTapEnable(state.tap, true) };
        if !unsafe { CGEventTapIsEnabled(state.tap) } {
            // Permission appears to have been revoked. Leave the run loop so
            // the outer retry can wait and try again. Do not abort.
            if !state.runloop.is_null() {
                unsafe { CFRunLoopStop(state.runloop) };
            }
        }
    }
}

/// Block the current thread in a dedicated CFRunLoop until `running` is false
/// or the tap dies. Must be called from a background thread, never the AppKit
/// / Tauri main thread.
pub fn listen_physical_events<F>(
    settings: Arc<Mutex<HotkeySettings>>,
    running: Arc<AtomicBool>,
    on_trigger: F,
) -> Result<(), ListenError>
where
    F: Fn(f64, f64) + Send + 'static,
{
    let (tx, rx) = mpsc::sync_channel::<(f64, f64)>(1);
    std::thread::Builder::new()
        .name("keyring-hotkey-trigger".into())
        .spawn(move || {
            while let Ok((x, y)) = rx.recv() {
                on_trigger(x, y);
            }
        })
        .map_err(|_| ListenError::EventTapUnavailable)?;

    unsafe {
        let state = Box::new(TapState {
            tap: std::ptr::null_mut(),
            runloop: std::ptr::null_mut(),
            running: Arc::clone(&running),
            settings,
            machine: Mutex::new(MachineState::default()),
            tracker: Mutex::new(ModifierTracker::default()),
            trigger_tx: tx,
        });
        let state_ptr = Box::into_raw(state);

        let tap = CGEventTapCreate(
            K_CG_HID_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
            listen_event_mask(),
            tap_callback,
            state_ptr as *mut c_void,
        );
        if tap.is_null() {
            drop(Box::from_raw(state_ptr));
            return Err(ListenError::EventTapUnavailable);
        }
        (*state_ptr).tap = tap;

        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
        if source.is_null() {
            CFRelease(tap as *const c_void);
            drop(Box::from_raw(state_ptr));
            return Err(ListenError::RunLoopSourceUnavailable);
        }

        let runloop = CFRunLoopGetCurrent();
        (*state_ptr).runloop = runloop;
        CFRunLoopAddSource(runloop, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);

        let mut timer_ctx = CFRunLoopTimerContext {
            version: 0,
            info: state_ptr as *mut c_void,
            retain: std::ptr::null(),
            release: std::ptr::null(),
            copy_description: std::ptr::null(),
        };
        let fire = CFAbsoluteTimeGetCurrent() + 2.0;
        let timer = CFRunLoopTimerCreate(
            std::ptr::null(),
            fire,
            2.0,
            0,
            0,
            watchdog_timer,
            &mut timer_ctx,
        );
        if !timer.is_null() {
            CFRunLoopAddTimer(runloop, timer, kCFRunLoopCommonModes);
        }

        CFRunLoopRun();

        if !timer.is_null() {
            CFRelease(timer as *const c_void);
        }
        CFRunLoopRemoveSource(runloop, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, false);
        CFRelease(source as *const c_void);
        CFRelease(tap as *const c_void);
        drop(Box::from_raw(state_ptr));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::probe_event_tap;

    #[test]
    fn probe_event_tap_does_not_abort() {
        // GitHub-hosted macOS runners typically lack Accessibility for this
        // process, so creation may fail. The important contract is: we return
        // an error instead of hitting a HIToolbox dispatch assertion.
        let _ = probe_event_tap();
    }
}
