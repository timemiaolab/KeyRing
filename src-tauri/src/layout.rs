//! Screen-space conversion and pie placement above a Finder selection.
//!
//! Coordinate spaces:
//! - `CocoaRect`: AppKit / Accessibility points, origin at the bottom-left of
//!   the primary display, Y increases upward. This is what `AXPosition` uses.
//! - `Rect` (Y-down): origin at the top-left of the primary display, Y increases
//!   downward, still in points (not pixels). This matches Tauri `LogicalPosition`
//!   and `CGEvent` locations on macOS.
//!
//! Backing scale is recorded for tests but never used to multiply coordinates.
//! Mixed-DPI arrangements stay in global logical points.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn top(&self) -> f64 {
        self.y
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    pub fn area(&self) -> f64 {
        (self.width * self.height).abs()
    }

    pub fn overlaps_x(&self, other: &Rect) -> bool {
        self.x < other.right() && other.x < self.right()
    }

    pub fn is_icon_like(&self) -> bool {
        if self.width < 16.0 || self.height < 16.0 {
            return false;
        }
        let ratio = if self.height == 0.0 {
            0.0
        } else {
            self.width / self.height
        };
        (0.7..=1.4).contains(&ratio)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Screen {
    pub frame: Rect,
    pub visible_frame: Rect,
    pub scale: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenLayout {
    pub screens: Vec<Screen>,
    /// Height of the primary display in points; used to convert Cocoa → Y-down.
    pub primary_height: f64,
}

impl ScreenLayout {
    pub fn new(screens: Vec<Screen>, primary_height: f64) -> Self {
        Self {
            screens,
            primary_height,
        }
    }
}

/// Convert an Accessibility / AppKit rectangle into Y-down logical points.
pub fn cocoa_rect_to_y_down(rect: Rect, primary_height: f64) -> Rect {
    Rect {
        x: rect.x,
        y: primary_height - (rect.y + rect.height),
        width: rect.width,
        height: rect.height,
    }
}

pub fn y_down_rect_to_cocoa(rect: Rect, primary_height: f64) -> Rect {
    Rect {
        x: rect.x,
        y: primary_height - (rect.y + rect.height),
        width: rect.width,
        height: rect.height,
    }
}

/// Geometry of the transparent pie webview versus the visible ring.
///
/// The window is larger than the painted ring (`ring_diameter + 60`, matching
/// the existing window size). The ring itself is centered in that window;
/// the extra 30pt on each side is transparent padding (plus a 22pt SVG hover
/// pad inside the ring canvas). Placement uses the visible ring, not the
/// webview's top-left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PieMetrics {
    pub ring_diameter: f64,
    pub window_size: f64,
    pub gap_above_target: f64,
}

pub const WINDOW_SIZE_PADDING: f64 = 60.0;
pub const DEFAULT_GAP_ABOVE_TARGET: f64 = 8.0;

impl PieMetrics {
    pub fn from_ring_diameter(ring_diameter: f64) -> Self {
        Self {
            ring_diameter,
            window_size: ring_diameter + WINDOW_SIZE_PADDING,
            gap_above_target: DEFAULT_GAP_ABOVE_TARGET,
        }
    }

    /// Distance from the window's top or left edge to the visible ring edge.
    pub fn ring_inset(&self) -> f64 {
        (self.window_size - self.ring_diameter) / 2.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClampReason {
    None,
    Top,
    Horizontal,
    TopAndHorizontal,
    VisibleFrame,
}

impl ClampReason {
    pub fn from_flags(clamped_x: bool, clamped_y: bool, overflow: bool) -> Self {
        if overflow {
            return ClampReason::VisibleFrame;
        }
        match (clamped_x, clamped_y) {
            (false, false) => ClampReason::None,
            (false, true) => ClampReason::Top,
            (true, false) => ClampReason::Horizontal,
            (true, true) => ClampReason::TopAndHorizontal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PiePlacement {
    pub window_x: f64,
    pub window_y: f64,
    pub window_size: f64,
    pub clamp: ClampReason,
    pub screen_index: usize,
}

/// Place the pie so the visible ring is horizontally centered above `item`
/// (Y-down logical points). The window is then clamped to the item's screen
/// `visible_frame`.
///
/// Fallback when there is not enough room above the item:
/// 1. Prefer the ideal slot (ring bottom + gap = item top, ring centered on item).
/// 2. Identify the screen that contains the item's center (else the nearest).
/// 3. Clamp the window into that screen's visible frame (menu bar / Dock excluded).
///    If the top edge overflows, the window is pushed down — it may overlap the
///    icon rather than jump below it or to the cursor.
/// 4. If the window is larger than the visible frame, pin it to the visible
///    origin; the overflow is reported as `VisibleFrame`.
/// 5. Never use the cursor position or the Finder window's bounds.
pub fn place_pie_above_item(
    item: &Rect,
    screens: &[Screen],
    metrics: &PieMetrics,
) -> Option<PiePlacement> {
    if screens.is_empty() || metrics.window_size <= 0.0 || metrics.ring_diameter <= 0.0 {
        return None;
    }

    let screen_index = screen_index_for_item(item, screens)?;
    let visible = screens[screen_index].visible_frame;
    let (item_cx, _) = item.center();
    let inset = metrics.ring_inset();
    let size = metrics.window_size;

    let ideal_x = item_cx - size / 2.0;
    // window_y + inset + ring_diameter + gap = item.top
    let ideal_y = item.top() - metrics.gap_above_target - inset - metrics.ring_diameter;

    let (window_x, clamped_x, overflow_x) = clamp_axis(ideal_x, size, visible.x, visible.width);
    let (window_y, clamped_y, overflow_y) = clamp_axis(ideal_y, size, visible.y, visible.height);

    Some(PiePlacement {
        window_x,
        window_y,
        window_size: size,
        clamp: ClampReason::from_flags(clamped_x, clamped_y, overflow_x || overflow_y),
        screen_index,
    })
}

fn clamp_axis(ideal: f64, size: f64, origin: f64, span: f64) -> (f64, bool, bool) {
    if span <= 0.0 {
        return (origin, true, true);
    }
    if size >= span {
        return (origin, ideal != origin, true);
    }
    let max = origin + span - size;
    let clamped = ideal.clamp(origin, max);
    (clamped, (clamped - ideal).abs() > f64::EPSILON, false)
}

pub fn screen_index_for_item(item: &Rect, screens: &[Screen]) -> Option<usize> {
    if screens.is_empty() {
        return None;
    }
    let (cx, cy) = item.center();
    if let Some(index) = screens.iter().position(|s| s.frame.contains_point(cx, cy)) {
        return Some(index);
    }
    let mut best = 0usize;
    let mut best_dist = f64::MAX;
    for (index, screen) in screens.iter().enumerate() {
        let (sx, sy) = screen.frame.center();
        let dist = (sx - cx).hypot(sy - cy);
        if dist < best_dist {
            best_dist = dist;
            best = index;
        }
    }
    Some(best)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxItem {
    pub title: Option<String>,
    pub url: Option<String>,
    pub role: Option<String>,
    pub bounds_cocoa: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryChoice<'a> {
    pub index: usize,
    pub item: &'a AxItem,
}

/// Deterministic primary item among a multi-selection.
///
/// * Default: the first item in Accessibility selected-children / selected-rows
///   order (icon view, list view, desktop).
/// * Column-view path: when items do not overlap horizontally and are not
///   icon-sized squares, pick the rightmost (the most specific column).
pub fn choose_primary(items: &[AxItem]) -> Option<PrimaryChoice<'_>> {
    if items.is_empty() {
        return None;
    }
    if looks_like_column_path(items) {
        let (index, item) = items
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                let ax = a.bounds_cocoa.x + a.bounds_cocoa.width / 2.0;
                let bx = b.bounds_cocoa.x + b.bounds_cocoa.width / 2.0;
                ax.partial_cmp(&bx).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        return Some(PrimaryChoice { index, item });
    }
    Some(PrimaryChoice {
        index: 0,
        item: &items[0],
    })
}

fn looks_like_column_path(items: &[AxItem]) -> bool {
    if items.len() < 2 {
        return false;
    }
    if items.iter().all(|item| item.bounds_cocoa.is_icon_like()) {
        return false;
    }
    for (index, item) in items.iter().enumerate() {
        for other in items.iter().skip(index + 1) {
            if item.bounds_cocoa.overlaps_x(&other.bounds_cocoa) {
                return false;
            }
        }
    }
    true
}

pub fn file_url_to_posix(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let rest = if let Some(rest) = trimmed.strip_prefix("file://localhost") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("file://") {
        rest
    } else {
        return None;
    };
    let path = if rest.starts_with('/') {
        rest.to_string()
    } else {
        format!("/{rest}")
    };
    Some(percent_decode(&path).trim_end_matches('/').to_string())
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (from_hex(bytes[index + 1]), from_hex(bytes[index + 2])) {
                out.push((hi << 4) | lo);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn basename_of_path(path: &str) -> &str {
    path.trim_end_matches('/').rsplit('/').next().unwrap_or(path)
}

fn names_equal(left: &str, right: &str) -> bool {
    left == right || left.eq_ignore_ascii_case(right)
}

/// Keep every captured path, but move the primary item to the front when it
/// can be matched by URL or file name. If nothing matches, paths are unchanged.
pub fn align_paths_with_primary(paths: Vec<String>, primary: &AxItem) -> Vec<String> {
    if paths.is_empty() {
        return paths;
    }
    let url_posix = primary
        .url
        .as_deref()
        .and_then(file_url_to_posix);
    let title = primary.title.as_deref();

    let position = paths.iter().position(|path| {
        let trimmed = path.trim_end_matches('/');
        if let Some(posix) = url_posix.as_deref() {
            if names_equal(trimmed, posix) || names_equal(path, posix) {
                return true;
            }
        }
        if let Some(title) = title {
            if names_equal(basename_of_path(trimmed), title.trim()) {
                return true;
            }
        }
        false
    });

    let Some(index) = position else {
        return paths;
    };
    if index == 0 {
        return paths;
    }
    let mut aligned = paths;
    let primary_path = aligned.remove(index);
    aligned.insert(0, primary_path);
    aligned
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PositionCapture {
    Located {
        #[serde(rename = "cocoaRect")]
        cocoa_rect: Rect,
        title: Option<String>,
        url: Option<String>,
        role: Option<String>,
    },
    NoSelection,
    AccessibilityDenied,
    BoundsUnavailable {
        reason: String,
    },
    NotApplicable,
}

impl PositionCapture {
    pub fn from_primary(primary: &AxItem) -> Self {
        PositionCapture::Located {
            cocoa_rect: primary.bounds_cocoa,
            title: primary.title.clone(),
            url: primary.url.clone(),
            role: primary.role.clone(),
        }
    }

    pub fn user_message(&self) -> Option<String> {
        match self {
            PositionCapture::Located { .. } | PositionCapture::NotApplicable => None,
            PositionCapture::NoSelection => Some(
                "请先在访达中选中文件或文件夹，然后再双击 Command。圆环会显示在选中项的上方，不会使用鼠标位置代替。"
                    .into(),
            ),
            PositionCapture::AccessibilityDenied => Some(
                "需要辅助功能权限才能定位选中项在屏幕上的位置。请打开「系统设置」→「隐私与安全性」→「辅助功能」，打开 KeyRing。授权后不必重启，再试一次即可。我们不会建议关闭系统完整性保护或其他不安全做法。"
                    .into(),
            ),
            PositionCapture::BoundsUnavailable { reason } => Some(format!(
                "已选中项目，但无法读取它在屏幕上的位置。{reason}圆环不会用鼠标位置或访达窗口位置代替。可以改用图标视图后再试。"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacementStatus {
    pub kind: String,
    pub message: Option<String>,
    pub clamp: Option<String>,
}

impl PlacementStatus {
    pub fn from_capture(capture: &PositionCapture) -> Self {
        let kind = match capture {
            PositionCapture::Located { .. } => "located",
            PositionCapture::NoSelection => "no_selection",
            PositionCapture::AccessibilityDenied => "accessibility_denied",
            PositionCapture::BoundsUnavailable { .. } => "bounds_unavailable",
            PositionCapture::NotApplicable => "not_applicable",
        };
        Self {
            kind: kind.into(),
            message: capture.user_message(),
            clamp: None,
        }
    }

    pub fn located(clamp: ClampReason) -> Self {
        Self {
            kind: "located".into(),
            message: None,
            clamp: match clamp {
                ClampReason::None => None,
                other => Some(format!("{other:?}").to_ascii_lowercase()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn primary_screen() -> Screen {
        // 1440×900 logical points, 25pt menu bar, 60pt dock at bottom.
        Screen {
            frame: Rect::new(0.0, 0.0, 1440.0, 900.0),
            visible_frame: Rect::new(0.0, 25.0, 1440.0, 815.0),
            scale: 2.0,
        }
    }

    fn left_screen() -> Screen {
        Screen {
            frame: Rect::new(-1920.0, -180.0, 1920.0, 1080.0),
            visible_frame: Rect::new(-1920.0, -180.0, 1920.0, 1080.0),
            scale: 1.0,
        }
    }

    fn above_screen() -> Screen {
        Screen {
            frame: Rect::new(0.0, -800.0, 1440.0, 800.0),
            visible_frame: Rect::new(0.0, -800.0, 1440.0, 800.0),
            scale: 2.0,
        }
    }

    fn metrics() -> PieMetrics {
        PieMetrics::from_ring_diameter(260.0)
    }

    #[test]
    fn cocoa_conversion_on_primary_display() {
        // Cocoa item: bottom-left at (100, 800), size 48×48, primary height 900.
        // Top of item in Cocoa is 848. Y-down top is 900 - 848 = 52.
        let cocoa = Rect::new(100.0, 800.0, 48.0, 48.0);
        let y_down = cocoa_rect_to_y_down(cocoa, 900.0);
        assert_eq!(y_down, Rect::new(100.0, 52.0, 48.0, 48.0));
        assert_eq!(y_down_rect_to_cocoa(y_down, 900.0), cocoa);
    }

    #[test]
    fn cocoa_conversion_keeps_negative_x_on_left_display() {
        let cocoa = Rect::new(-500.0, 200.0, 64.0, 64.0);
        let y_down = cocoa_rect_to_y_down(cocoa, 900.0);
        assert_eq!(y_down.x, -500.0);
        assert_eq!(y_down.y, 900.0 - 264.0);
        assert_eq!(y_down.width, 64.0);
    }

    #[test]
    fn cocoa_conversion_for_display_above_primary() {
        // Upper display occupies Cocoa y = 900..1700. Item near its top.
        let cocoa = Rect::new(120.0, 1636.0, 48.0, 48.0);
        let y_down = cocoa_rect_to_y_down(cocoa, 900.0);
        assert_eq!(y_down.y, 900.0 - 1684.0);
        assert!(y_down.y < 0.0, "items on a display above primary are negative in Y-down");
        assert_eq!(y_down.x, 120.0);
    }

    #[test]
    fn conversion_ignores_backing_scale() {
        // Same logical Cocoa rect must yield the same Y-down rect at scale 1 or 2.
        let cocoa = Rect::new(40.0, 100.0, 50.0, 50.0);
        let a = cocoa_rect_to_y_down(cocoa, 900.0);
        let b = cocoa_rect_to_y_down(cocoa, 900.0);
        assert_eq!(a, b);
        assert_ne!(a.x, cocoa.x * 2.0);
        assert_ne!(a.width, cocoa.width * 2.0);
    }

    #[test]
    fn places_visible_ring_centered_above_item_not_window_corner() {
        let item = Rect::new(400.0, 400.0, 80.0, 80.0);
        let screens = vec![primary_screen()];
        let metrics = metrics();
        let placed = place_pie_above_item(&item, &screens, &metrics).unwrap();

        assert_eq!(placed.window_size, 320.0);
        assert_eq!(metrics.ring_inset(), 30.0);
        assert_eq!(placed.clamp, ClampReason::None);

        let item_cx = 440.0;
        assert!((placed.window_x + placed.window_size / 2.0 - item_cx).abs() < 1e-9);
        // Window top-left is NOT the item center; that would ignore the inset.
        assert!((placed.window_x - item_cx).abs() > 1.0);

        let ring_bottom = placed.window_y + metrics.ring_inset() + metrics.ring_diameter;
        assert!((ring_bottom + metrics.gap_above_target - item.top()).abs() < 1e-9);
    }

    #[test]
    fn clamps_to_visible_top_when_space_above_is_short() {
        // Item just under the menu bar (visible top = 25).
        let item = Rect::new(200.0, 40.0, 48.0, 48.0);
        let screens = vec![primary_screen()];
        let placed = place_pie_above_item(&item, &screens, &metrics()).unwrap();
        assert_eq!(placed.window_y, 25.0);
        assert!(matches!(
            placed.clamp,
            ClampReason::Top | ClampReason::TopAndHorizontal
        ));
        // Must not jump to a cursor-like position or the screen center.
        assert!(placed.window_y < item.y + 1.0);
    }

    #[test]
    fn clamps_horizontally_on_left_edge() {
        let item = Rect::new(4.0, 500.0, 32.0, 32.0);
        let screens = vec![primary_screen()];
        let placed = place_pie_above_item(&item, &screens, &metrics()).unwrap();
        assert_eq!(placed.window_x, 0.0);
        assert!(matches!(
            placed.clamp,
            ClampReason::Horizontal | ClampReason::TopAndHorizontal
        ));
    }

    #[test]
    fn clamps_top_and_horizontal_together() {
        let item = Rect::new(6.0, 36.0, 24.0, 24.0);
        let screens = vec![primary_screen()];
        let placed = place_pie_above_item(&item, &screens, &metrics()).unwrap();
        assert_eq!(placed.window_x, 0.0);
        assert_eq!(placed.window_y, 25.0);
        assert_eq!(placed.clamp, ClampReason::TopAndHorizontal);
    }

    #[test]
    fn places_on_left_display_with_negative_coordinates() {
        let item = Rect::new(-800.0, 200.0, 64.0, 64.0);
        let screens = vec![primary_screen(), left_screen()];
        let placed = place_pie_above_item(&item, &screens, &metrics()).unwrap();
        assert_eq!(placed.screen_index, 1);
        let ring_cx = placed.window_x + placed.window_size / 2.0;
        assert!((ring_cx - (item.x + item.width / 2.0)).abs() < 1e-9);
        assert!(placed.window_x < 0.0);
        assert!(placed.window_x >= -1920.0);
    }

    #[test]
    fn places_on_display_above_primary() {
        let item = Rect::new(200.0, -400.0, 48.0, 48.0);
        let screens = vec![primary_screen(), above_screen()];
        let placed = place_pie_above_item(&item, &screens, &metrics()).unwrap();
        assert_eq!(placed.screen_index, 1);
        assert!(placed.window_y < 0.0);
        assert!(placed.window_y >= -800.0);
    }

    #[test]
    fn mixed_scale_uses_points_not_pixels() {
        let item = Rect::new(-500.0, 100.0, 48.0, 48.0);
        let screens = vec![primary_screen(), left_screen()];
        assert_eq!(screens[0].scale, 2.0);
        assert_eq!(screens[1].scale, 1.0);
        let placed = place_pie_above_item(&item, &screens, &metrics()).unwrap();
        // A mistaken physical conversion would multiply the left screen by scale 1
        // and the item (captured in points) by the primary scale 2.
        assert!((placed.window_x + 160.0 + 500.0 - 24.0).abs() < 1e-9);
        assert!(placed.window_x < 0.0);
        assert!(placed.window_x > -1920.0);
    }

    #[test]
    fn window_larger_than_visible_frame_pins_to_origin() {
        let tiny = Screen {
            frame: Rect::new(0.0, 0.0, 200.0, 200.0),
            visible_frame: Rect::new(0.0, 20.0, 200.0, 160.0),
            scale: 1.0,
        };
        let item = Rect::new(40.0, 80.0, 20.0, 20.0);
        let placed = place_pie_above_item(&item, &[tiny], &metrics()).unwrap();
        assert_eq!(placed.window_x, 0.0);
        assert_eq!(placed.window_y, 20.0);
        assert_eq!(placed.clamp, ClampReason::VisibleFrame);
    }

    #[test]
    fn icon_multiselect_uses_first_ax_item() {
        let items = vec![
            AxItem {
                title: Some("A".into()),
                url: None,
                role: Some("AXImage".into()),
                bounds_cocoa: Rect::new(0.0, 0.0, 64.0, 64.0),
            },
            AxItem {
                title: Some("B".into()),
                url: None,
                role: Some("AXImage".into()),
                bounds_cocoa: Rect::new(80.0, 0.0, 64.0, 64.0),
            },
        ];
        let primary = choose_primary(&items).unwrap();
        assert_eq!(primary.index, 0);
        assert_eq!(primary.item.title.as_deref(), Some("A"));
    }

    #[test]
    fn list_rows_use_first_ax_item() {
        let items = vec![
            AxItem {
                title: Some("notes.md".into()),
                url: None,
                role: Some("AXRow".into()),
                bounds_cocoa: Rect::new(20.0, 400.0, 600.0, 22.0),
            },
            AxItem {
                title: Some("draft.md".into()),
                url: None,
                role: Some("AXRow".into()),
                bounds_cocoa: Rect::new(20.0, 378.0, 600.0, 22.0),
            },
        ];
        let primary = choose_primary(&items).unwrap();
        assert_eq!(primary.index, 0);
        assert_eq!(primary.item.title.as_deref(), Some("notes.md"));
    }

    #[test]
    fn column_view_picks_rightmost_item() {
        let items = vec![
            AxItem {
                title: Some("Projects".into()),
                url: None,
                role: Some("AXTextField".into()),
                bounds_cocoa: Rect::new(10.0, 200.0, 180.0, 20.0),
            },
            AxItem {
                title: Some("KeyRing".into()),
                url: None,
                role: Some("AXTextField".into()),
                bounds_cocoa: Rect::new(210.0, 140.0, 180.0, 20.0),
            },
            AxItem {
                title: Some("README.md".into()),
                url: None,
                role: Some("AXTextField".into()),
                bounds_cocoa: Rect::new(410.0, 80.0, 180.0, 20.0),
            },
        ];
        let primary = choose_primary(&items).unwrap();
        assert_eq!(primary.item.title.as_deref(), Some("README.md"));
        assert_eq!(primary.index, 2);
    }

    #[test]
    fn align_paths_moves_primary_to_front_and_keeps_the_rest() {
        let paths = vec![
            "/Users/test/My Folder/A.txt".into(),
            "/Users/test/My Folder/B file.txt".into(),
            "/Users/test/My Folder/C.txt".into(),
        ];
        let primary = AxItem {
            title: Some("B file.txt".into()),
            url: Some("file:///Users/test/My%20Folder/B%20file.txt".into()),
            role: Some("AXImage".into()),
            bounds_cocoa: Rect::new(0.0, 0.0, 48.0, 48.0),
        };
        let aligned = align_paths_with_primary(paths, &primary);
        assert_eq!(aligned[0], "/Users/test/My Folder/B file.txt");
        assert_eq!(aligned[1], "/Users/test/My Folder/A.txt");
        assert_eq!(aligned[2], "/Users/test/My Folder/C.txt");
    }

    #[test]
    fn align_paths_unchanged_when_title_does_not_match() {
        let paths = vec!["/Users/test/A.txt".into(), "/Users/test/B.txt".into()];
        let primary = AxItem {
            title: Some("Unknown".into()),
            url: None,
            role: None,
            bounds_cocoa: Rect::new(0.0, 0.0, 20.0, 20.0),
        };
        assert_eq!(
            align_paths_with_primary(paths.clone(), &primary),
            paths
        );
    }

    #[test]
    fn file_url_with_spaces_and_quotes_decodes() {
        let url = r#"file:///Users/test/My%20%22Project%22/draft.txt"#;
        assert_eq!(
            file_url_to_posix(url).as_deref(),
            Some(r#"/Users/test/My "Project"/draft.txt"#)
        );
    }

    #[test]
    fn permission_and_missing_bounds_messages_do_not_mention_mouse_fallback() {
        let denied = PositionCapture::AccessibilityDenied.user_message().unwrap();
        assert!(denied.contains("辅助功能"));
        assert!(!denied.contains("SIP"));
        assert!(!denied.to_lowercase().contains("mouse"));

        let missing = PositionCapture::BoundsUnavailable {
            reason: "当前视图没有提供选中项的屏幕坐标。".into(),
        }
        .user_message()
        .unwrap();
        assert!(missing.contains("不会用鼠标位置"));
        assert!(missing.contains("访达窗口"));
    }

    #[test]
    fn empty_selection_and_empty_screens_are_explicit() {
        assert!(choose_primary(&[]).is_none());
        let item = Rect::new(10.0, 10.0, 10.0, 10.0);
        assert!(place_pie_above_item(&item, &[], &metrics()).is_none());
    }
}
