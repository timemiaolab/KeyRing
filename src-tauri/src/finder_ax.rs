//! Read the on-screen bounds of the Finder selection via the public
//! Accessibility API (`AXUIElement`, `AXPosition`, `AXSize`,
//! `AXSelectedChildren`, `AXSelectedRows`, `AXSelected`, `AXTitle`,
//! `AXDescription`, `AXURL`, `AXRole`, `AXWindows`, `AXFocusedWindow`).
//!
//! No private APIs and no user credentials are used. This must run while
//! Finder is still frontmost, before KeyRing takes focus.

use crate::layout::{
    choose_primary, cocoa_rect_to_y_down, AxItem, PositionCapture, Rect, Screen, ScreenLayout,
};
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use std::ffi::c_void;
use std::process::Command;

const AX_ERROR_SUCCESS: i32 = 0;
const AX_ERROR_API_DISABLED: i32 = -25211;
const AX_VALUE_CG_POINT: u32 = 1;
const AX_VALUE_CG_SIZE: u32 = 2;
const CF_URL_POSIX_PATH_STYLE: i32 = 0;
const MAX_DEPTH: usize = 14;
const MAX_NODES: usize = 500;

#[repr(C)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
struct NSPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
struct NSSize {
    width: f64,
    height: f64,
}

#[repr(C)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

type AXUIElementRef = *const c_void;
type AXValueRef = *const c_void;
type CFTypeRef = *const c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementGetPid(element: AXUIElementRef, pid: *mut i32) -> i32;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
    fn AXValueGetValue(value: AXValueRef, typ: u32, value_ptr: *mut c_void) -> u8;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFGetTypeID(cf: CFTypeRef) -> usize;
    fn CFRelease(cf: CFTypeRef);
    fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
    fn CFArrayGetTypeID() -> usize;
    fn CFArrayGetCount(array: CFTypeRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFTypeRef, idx: isize) -> CFTypeRef;
    fn CFStringGetTypeID() -> usize;
    fn CFBooleanGetTypeID() -> usize;
    fn CFBooleanGetValue(value: CFTypeRef) -> u8;
    fn CFURLGetTypeID() -> usize;
    fn CFURLCopyFileSystemPath(url: CFTypeRef, path_style: i32) -> CFTypeRef;
    fn CFURLGetString(url: CFTypeRef) -> CFTypeRef;
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {}

struct AxElem {
    raw: AXUIElementRef,
    owned: bool,
}

impl AxElem {
    fn from_create(raw: AXUIElementRef) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self { raw, owned: true })
        }
    }

    fn from_copy(raw: AXUIElementRef) -> Option<Self> {
        Self::from_create(raw)
    }

    fn borrowed(raw: AXUIElementRef) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self { raw, owned: false })
        }
    }
}

impl Drop for AxElem {
    fn drop(&mut self) {
        if self.owned && !self.raw.is_null() {
            unsafe { CFRelease(self.raw) };
        }
    }
}

enum AxLookup {
    Denied,
    Failed,
    Value(CFTypeRef),
}

fn copy_attribute(element: &AxElem, name: &str) -> AxLookup {
    let attr = CFString::new(name);
    let mut value: CFTypeRef = std::ptr::null();
    let err = unsafe {
        AXUIElementCopyAttributeValue(element.raw, attr.as_concrete_TypeRef(), &mut value)
    };
    if err == AX_ERROR_API_DISABLED {
        return AxLookup::Denied;
    }
    if err != AX_ERROR_SUCCESS || value.is_null() {
        return AxLookup::Failed;
    }
    AxLookup::Value(value)
}

fn cfstring_to_string(ptr: CFTypeRef, create_rule: bool) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let typ = unsafe { CFGetTypeID(ptr) };
    if typ != unsafe { CFStringGetTypeID() } {
        if create_rule {
            unsafe { CFRelease(ptr) };
        }
        return None;
    }
    let string = if create_rule {
        unsafe { CFString::wrap_under_create_rule(ptr as _) }
    } else {
        unsafe { CFString::wrap_under_get_rule(ptr as _) }
    };
    Some(string.to_string())
}

fn copy_string(element: &AxElem, name: &str) -> Result<Option<String>, PositionCapture> {
    match copy_attribute(element, name) {
        AxLookup::Denied => Err(PositionCapture::AccessibilityDenied),
        AxLookup::Failed => Ok(None),
        AxLookup::Value(ptr) => {
            let typ = unsafe { CFGetTypeID(ptr) };
            if typ == unsafe { CFStringGetTypeID() } {
                Ok(cfstring_to_string(ptr, true))
            } else if typ == unsafe { CFURLGetTypeID() } {
                let path = unsafe { CFURLCopyFileSystemPath(ptr, CF_URL_POSIX_PATH_STYLE) };
                if path.is_null() {
                    let url_string = unsafe { CFURLGetString(ptr) };
                    let text = cfstring_to_string(url_string, false);
                    unsafe { CFRelease(ptr) };
                    Ok(text)
                } else {
                    let text = cfstring_to_string(path, true);
                    unsafe { CFRelease(ptr) };
                    Ok(text)
                }
            } else {
                unsafe { CFRelease(ptr) };
                Ok(None)
            }
        }
    }
}

fn copy_bool(element: &AxElem, name: &str) -> Result<Option<bool>, PositionCapture> {
    match copy_attribute(element, name) {
        AxLookup::Denied => Err(PositionCapture::AccessibilityDenied),
        AxLookup::Failed => Ok(None),
        AxLookup::Value(ptr) => {
            let typ = unsafe { CFGetTypeID(ptr) };
            let result = if typ == unsafe { CFBooleanGetTypeID() } {
                Some(unsafe { CFBooleanGetValue(ptr) } != 0)
            } else {
                None
            };
            unsafe { CFRelease(ptr) };
            Ok(result)
        }
    }
}

fn copy_array(element: &AxElem, name: &str) -> Result<Vec<AxElem>, PositionCapture> {
    match copy_attribute(element, name) {
        AxLookup::Denied => Err(PositionCapture::AccessibilityDenied),
        AxLookup::Failed => Ok(Vec::new()),
        AxLookup::Value(ptr) => {
            let typ = unsafe { CFGetTypeID(ptr) };
            if typ != unsafe { CFArrayGetTypeID() } {
                unsafe { CFRelease(ptr) };
                return Ok(Vec::new());
            }
            let count = unsafe { CFArrayGetCount(ptr) };
            let mut out = Vec::new();
            for index in 0..count {
                let raw = unsafe { CFArrayGetValueAtIndex(ptr, index) };
                if raw.is_null() {
                    continue;
                }
                unsafe { CFRetain(raw) };
                if let Some(elem) = AxElem::from_copy(raw) {
                    out.push(elem);
                }
            }
            unsafe { CFRelease(ptr) };
            Ok(out)
        }
    }
}

fn copy_point(element: &AxElem) -> Result<Option<(f64, f64)>, PositionCapture> {
    match copy_attribute(element, "AXPosition") {
        AxLookup::Denied => Err(PositionCapture::AccessibilityDenied),
        AxLookup::Failed => Ok(None),
        AxLookup::Value(ptr) => {
            let mut point = CGPoint { x: 0.0, y: 0.0 };
            let ok = unsafe {
                AXValueGetValue(
                    ptr as AXValueRef,
                    AX_VALUE_CG_POINT,
                    &mut point as *mut _ as *mut c_void,
                )
            };
            unsafe { CFRelease(ptr) };
            if ok != 0 {
                Ok(Some((point.x, point.y)))
            } else {
                Ok(None)
            }
        }
    }
}

fn copy_size(element: &AxElem) -> Result<Option<(f64, f64)>, PositionCapture> {
    match copy_attribute(element, "AXSize") {
        AxLookup::Denied => Err(PositionCapture::AccessibilityDenied),
        AxLookup::Failed => Ok(None),
        AxLookup::Value(ptr) => {
            let mut size = CGSize {
                width: 0.0,
                height: 0.0,
            };
            let ok = unsafe {
                AXValueGetValue(
                    ptr as AXValueRef,
                    AX_VALUE_CG_SIZE,
                    &mut size as *mut _ as *mut c_void,
                )
            };
            unsafe { CFRelease(ptr) };
            if ok != 0 {
                Ok(Some((size.width, size.height)))
            } else {
                Ok(None)
            }
        }
    }
}

fn bounds_of(element: &AxElem) -> Result<Option<Rect>, PositionCapture> {
    let Some((x, y)) = copy_point(element)? else {
        return Ok(None);
    };
    let Some((width, height)) = copy_size(element)? else {
        return Ok(None);
    };
    if width < 2.0 || height < 2.0 || !x.is_finite() || !y.is_finite() {
        return Ok(None);
    }
    Ok(Some(Rect::new(x, y, width, height)))
}

fn first_title(candidates: [Option<String>; 3]) -> Option<String> {
    for candidate in candidates {
        if let Some(text) = candidate {
            if !text.trim().is_empty() {
                return Some(text);
            }
        }
    }
    None
}

fn read_item(element: &AxElem) -> Result<Option<AxItem>, PositionCapture> {
    let Some(bounds_cocoa) = bounds_of(element)? else {
        return Ok(None);
    };
    // Skip huge containers (windows, scroll areas) — they are not icon bounds.
    if bounds_cocoa.width > 1600.0 && bounds_cocoa.height > 700.0 {
        return Ok(None);
    }
    let role = copy_string(element, "AXRole")?;
    if let Some(role) = role.as_deref() {
        if matches!(
            role,
            "AXWindow"
                | "AXSplitGroup"
                | "AXScrollArea"
                | "AXToolbar"
                | "AXMenu"
                | "AXMenuBar"
                | "AXGrowArea"
                | "AXLayoutArea"
        ) {
            return Ok(None);
        }
    }
    let title = first_title([
        copy_string(element, "AXTitle")?,
        copy_string(element, "AXDescription")?,
        copy_string(element, "AXFilename")?,
    ]);
    let url = copy_string(element, "AXURL")?;
    Ok(Some(AxItem {
        title,
        url,
        role,
        bounds_cocoa,
    }))
}

fn collect_named_selection(
    element: &AxElem,
    depth: usize,
    nodes: &mut usize,
) -> Result<Vec<AxElem>, PositionCapture> {
    if depth > MAX_DEPTH || *nodes > MAX_NODES {
        return Ok(Vec::new());
    }
    *nodes += 1;

    let mut selected = copy_array(element, "AXSelectedChildren")?;
    if selected.is_empty() {
        selected = copy_array(element, "AXSelectedRows")?;
    }
    if selected.is_empty() {
        selected = copy_array(element, "AXSelectedCells")?;
    }
    if !selected.is_empty() {
        return Ok(selected);
    }

    for child in copy_array(element, "AXChildren")? {
        let found = collect_named_selection(&child, depth + 1, nodes)?;
        if !found.is_empty() {
            return Ok(found);
        }
    }
    Ok(Vec::new())
}

fn collect_flagged_selection(
    element: &AxElem,
    depth: usize,
    nodes: &mut usize,
    out: &mut Vec<AxElem>,
) -> Result<(), PositionCapture> {
    if depth > MAX_DEPTH || *nodes > MAX_NODES {
        return Ok(());
    }
    *nodes += 1;
    if copy_bool(element, "AXSelected")? == Some(true) {
        unsafe { CFRetain(element.raw) };
        if let Some(owned) = AxElem::from_copy(element.raw) {
            out.push(owned);
        }
        return Ok(());
    }
    for child in copy_array(element, "AXChildren")? {
        collect_flagged_selection(&child, depth + 1, nodes, out)?;
    }
    Ok(())
}

fn items_from_elements(elements: &[AxElem]) -> Result<Vec<AxItem>, PositionCapture> {
    let mut items = Vec::new();
    for element in elements {
        if let Some(item) = read_item(element)? {
            items.push(item);
            continue;
        }
        let mut sub_nodes = 0;
        let nested = collect_named_selection(element, 0, &mut sub_nodes)?;
        if !nested.is_empty() {
            for child in nested {
                if let Some(item) = read_item(&child)? {
                    items.push(item);
                }
            }
            continue;
        }
        for child in copy_array(element, "AXChildren")? {
            if copy_bool(&child, "AXSelected")? == Some(false) {
                continue;
            }
            if let Some(item) = read_item(&child)? {
                items.push(item);
            }
        }
    }
    Ok(items)
}

fn finder_pid() -> Option<i32> {
    let output = Command::new("/usr/bin/pgrep")
        .args(["-x", "Finder"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse::<i32>().ok())
        .max()
}

/// Walk Finder's accessibility tree and return the primary selected item.
///
/// Supported well: icon view, list view, column view, and the desktop, when
/// macOS exposes `AXSelectedChildren` / `AXSelectedRows` or `AXSelected` with
/// `AXPosition`/`AXSize`. Some newer views (for example Gallery) may not
/// publish a stable selected-element position; the caller must surface
/// `BoundsUnavailable` instead of guessing.
pub fn capture_finder_selection_bounds() -> PositionCapture {
    if !crate::hotkey::check_accessibility_permission(false) {
        return PositionCapture::AccessibilityDenied;
    }
    let Some(pid) = finder_pid() else {
        return PositionCapture::BoundsUnavailable {
            reason: "没有找到正在运行的访达进程。".into(),
        };
    };
    let Some(app) = AxElem::from_create(unsafe { AXUIElementCreateApplication(pid) }) else {
        return PositionCapture::BoundsUnavailable {
            reason: "无法连接到访达的辅助功能元素。".into(),
        };
    };

    match capture_from_app(&app) {
        Ok(items) => {
            let Some(primary) = choose_primary(&items) else {
                return PositionCapture::NoSelection;
            };
            PositionCapture::from_primary(primary.item)
        }
        Err(status) => status,
    }
}

fn capture_from_app(app: &AxElem) -> Result<Vec<AxItem>, PositionCapture> {
    let mut roots: Vec<AxElem> = Vec::new();
    if let Some(focused) = copy_single_element(app, "AXFocusedWindow")? {
        roots.push(focused);
    }
    for window in copy_array(app, "AXWindows")? {
        roots.push(window);
    }
    if roots.is_empty() {
        if let Some(app_root) = AxElem::borrowed(app.raw) {
            roots.push(app_root);
        }
    }

    let mut collected = Vec::new();
    for root in &roots {
        let mut nodes = 0;
        let selected = collect_named_selection(root, 0, &mut nodes)?;
        if !selected.is_empty() {
            let items = items_from_elements(&selected)?;
            if !items.is_empty() {
                collected = items;
                break;
            }
        }
    }

    if collected.is_empty() {
        for root in &roots {
            let mut nodes = 0;
            let mut flagged = Vec::new();
            collect_flagged_selection(root, 0, &mut nodes, &mut flagged)?;
            let items = items_from_elements(&flagged)?;
            if !items.is_empty() {
                collected = items;
                break;
            }
        }
    }

    Ok(collected)
}

fn copy_single_element(element: &AxElem, name: &str) -> Result<Option<AxElem>, PositionCapture> {
    match copy_attribute(element, name) {
        AxLookup::Denied => Err(PositionCapture::AccessibilityDenied),
        AxLookup::Failed => Ok(None),
        AxLookup::Value(ptr) => Ok(AxElem::from_copy(ptr)),
    }
}

pub fn is_finder_frontmost() -> Result<bool, PositionCapture> {
    let Some(finder) = finder_pid() else {
        return Ok(false);
    };
    let Some(system) = AxElem::from_create(unsafe { AXUIElementCreateSystemWide() }) else {
        return Ok(false);
    };
    let Some(focused) = copy_single_element(&system, "AXFocusedApplication")? else {
        return Ok(false);
    };
    let mut pid: i32 = 0;
    let err = unsafe { AXUIElementGetPid(focused.raw, &mut pid) };
    if err != AX_ERROR_SUCCESS {
        if err == AX_ERROR_API_DISABLED {
            return Err(PositionCapture::AccessibilityDenied);
        }
        return Ok(false);
    }
    Ok(pid == finder)
}

/// Logical Y-down screen frames from `NSScreen`, in points.
/// Must be called on the main thread.
pub fn list_screens() -> ScreenLayout {
    use objc2::runtime::{AnyClass, AnyObject};
    use objc2::msg_send;

    objc2::rc::autoreleasepool(|_| {
        let Some(cls) = AnyClass::get("NSScreen") else {
            return ScreenLayout::new(Vec::new(), 0.0);
        };
        let screens: *mut AnyObject = unsafe { msg_send![cls, screens] };
        if screens.is_null() {
            return ScreenLayout::new(Vec::new(), 0.0);
        }
        let count: usize = unsafe { msg_send![screens, count] };
        let mut cocoa_frames = Vec::new();
        for index in 0..count {
            let screen: *mut AnyObject = unsafe { msg_send![screens, objectAtIndex: index] };
            if screen.is_null() {
                continue;
            }
            let frame: NSRect = unsafe { msg_send![screen, frame] };
            let visible: NSRect = unsafe { msg_send![screen, visibleFrame] };
            let scale: f64 = unsafe { msg_send![screen, backingScaleFactor] };
            cocoa_frames.push((frame, visible, scale));
        }

        let primary_height = cocoa_frames
            .iter()
            .find(|(frame, _, _)| frame.origin.x == 0.0 && frame.origin.y == 0.0)
            .or_else(|| cocoa_frames.first())
            .map(|(frame, _, _)| frame.size.height)
            .unwrap_or(0.0);

        let screens = cocoa_frames
            .into_iter()
            .map(|(frame, visible, scale)| Screen {
                frame: cocoa_rect_to_y_down(
                    Rect::new(
                        frame.origin.x,
                        frame.origin.y,
                        frame.size.width,
                        frame.size.height,
                    ),
                    primary_height,
                ),
                visible_frame: cocoa_rect_to_y_down(
                    Rect::new(
                        visible.origin.x,
                        visible.origin.y,
                        visible.size.width,
                        visible.size.height,
                    ),
                    primary_height,
                ),
                scale,
            })
            .collect();

        ScreenLayout::new(screens, primary_height)
    })
}
