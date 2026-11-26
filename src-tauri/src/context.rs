use crate::layout::PositionCapture;
use crate::paths::working_directory_from_paths;
#[cfg(target_os = "macos")]
use crate::layout::{align_paths_with_primary, AxItem};
#[cfg(target_os = "macos")]
use crate::paths::split_captured_paths;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub selected_paths: Vec<String>,
    pub current_directory: Option<String>,
}

impl ContextSnapshot {
    pub fn effective_paths(&self) -> Vec<String> {
        if !self.selected_paths.is_empty() {
            return self.selected_paths.clone();
        }
        if let Some(dir) = &self.current_directory {
            return vec![dir.clone()];
        }
        Vec::new()
    }

    pub fn has_selected_items(&self) -> bool {
        !self.selected_paths.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCapture {
    pub context: ContextSnapshot,
    pub position: PositionCapture,
}

impl TriggerCapture {
    pub fn not_applicable(context: ContextSnapshot) -> Self {
        Self {
            context,
            position: PositionCapture::NotApplicable,
        }
    }
}

impl ContextSnapshot {
    pub fn resolve_working_directory(&self) -> Option<PathBuf> {
        let mut paths = self.selected_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        if paths.is_empty() {
            if let Some(dir) = &self.current_directory {
                paths.push(dir.as_str());
            }
        }
        working_directory_from_paths(paths)
    }
}

/// Must be called while Finder is still frontmost, before KeyRing takes focus.
#[cfg(target_os = "macos")]
pub fn capture_active_context() -> ContextSnapshot {
    use std::process::Command;

    let script = r#"
    tell application "System Events"
        set frontApp to name of first application process whose frontmost is true
    end tell
    if frontApp is "Finder" then
        tell application "Finder"
            set sel to selection
            if (count of sel) > 0 then
                set outPaths to ""
                repeat with itemRef in sel
                    set outPaths to outPaths & (POSIX path of (itemRef as alias)) & linefeed
                end repeat
                return "SEL:" & outPaths
            else if (count of windows) > 0 then
                return "DIR:" & (POSIX path of (target of front window as alias))
            end if
        end tell
    end if
    return ""
    "#;

    let output = match Command::new("osascript").arg("-e").arg(script).output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return ContextSnapshot::default(),
    };

    if let Some(rest) = output.strip_prefix("SEL:") {
        let paths = split_captured_paths(rest);
        return ContextSnapshot {
            selected_paths: paths,
            current_directory: None,
        };
    }

    if let Some(dir) = output.strip_prefix("DIR:") {
        let cleaned = dir.trim().to_string();
        if !cleaned.is_empty() {
            return ContextSnapshot {
                selected_paths: Vec::new(),
                current_directory: Some(cleaned),
            };
        }
    }

    ContextSnapshot::default()
}


#[cfg(target_os = "macos")]
pub fn capture_trigger_context() -> TriggerCapture {
    use crate::finder_ax::{capture_finder_selection_bounds, is_finder_frontmost};

    let frontmost = match is_finder_frontmost() {
        Ok(value) => value,
        Err(PositionCapture::AccessibilityDenied) => {
            let context = capture_active_context();
            return TriggerCapture {
                context,
                position: PositionCapture::AccessibilityDenied,
            };
        }
        Err(other) => {
            let context = capture_active_context();
            return TriggerCapture {
                context,
                position: other,
            };
        }
    };

    let mut context = capture_active_context();
    if !frontmost {
        return TriggerCapture {
            context,
            position: PositionCapture::NoSelection,
        };
    }

    let position = capture_finder_selection_bounds();
    match position {
        PositionCapture::Located {
            bounds,
            title,
            url,
            role,
        } => {
            let primary = AxItem {
                title: title.clone(),
                url: url.clone(),
                role: role.clone(),
                bounds,
            };
            context.selected_paths = align_paths_with_primary(context.selected_paths, &primary);
            TriggerCapture {
                context,
                position: PositionCapture::Located {
                    bounds,
                    title,
                    url,
                    role,
                },
            }
        }
        PositionCapture::AccessibilityDenied => TriggerCapture {
            context,
            position: PositionCapture::AccessibilityDenied,
        },
        PositionCapture::NoSelection | PositionCapture::BoundsUnavailable { .. } => {
            let position = if !context.selected_paths.is_empty() {
                PositionCapture::BoundsUnavailable {
                    reason: "辅助功能树里没有找到选中项的屏幕坐标。当前访达视图可能不受支持。".into(),
                }
            } else {
                PositionCapture::NoSelection
            };
            TriggerCapture { context, position }
        }
        other => TriggerCapture {
            context,
            position: other,
        },
    }
}

#[cfg(target_os = "windows")]
pub fn capture_active_context() -> ContextSnapshot {
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{IShellWindows, ShellWindows};
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let hwnd_fg = GetForegroundWindow();
        if hwnd_fg.0.is_null() {
            CoUninitialize();
            return ContextSnapshot::default();
        }
        let _shell_windows: Result<IShellWindows, _> =
            CoCreateInstance(&ShellWindows, None, CLSCTX_ALL);
        // Iterate ShellWindows matching HWND == hwnd_fg, inspect Document.SelectedItems() -> FolderItem.Path,
        // fallback to Folder.Self.Path without touching the clipboard.
        CoUninitialize();
    }

    ContextSnapshot::default()
}

#[cfg(target_os = "linux")]
pub fn capture_active_context() -> ContextSnapshot {
    // Query D-Bus org.freedesktop.FileManager1 or X11 PRIMARY selection URIs and decode file:// paths
    ContextSnapshot::default()
}

#[cfg(not(target_os = "macos"))]
pub fn capture_trigger_context() -> TriggerCapture {
    TriggerCapture::not_applicable(capture_active_context())
}
