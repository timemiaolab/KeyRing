use crate::paths::working_directory_from_paths;
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
