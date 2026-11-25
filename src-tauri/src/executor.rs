use crate::context::ContextSnapshot;
use crate::paths::working_directory_from_paths;
#[cfg(target_os = "windows")]
use crate::paths::powershell_single_quoted;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;

pub fn execute_copy_path(app: &AppHandle, ctx: &ContextSnapshot) -> Result<String, String> {
    let paths = ctx.effective_paths();
    if paths.is_empty() {
        return Err("还没有选中文件或文件夹".into());
    }
    let joined = crate::paths::join_paths_for_clipboard(&paths);
    app.clipboard()
        .write_text(joined.clone())
        .map_err(|e| e.to_string())?;

    let _ = app
        .notification()
        .builder()
        .title("KeyRing")
        .body(&joined)
        .show();

    Ok(joined)
}

fn resolve_cwd(ctx: &ContextSnapshot) -> PathBuf {
    ctx.resolve_working_directory()
        .or_else(|| working_directory_from_paths(ctx.effective_paths().iter().map(|s| s.as_str())))
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| Path::new("/").to_path_buf())
}

pub fn execute_open_in_terminal(
    ctx: &ContextSnapshot,
    terminal_adapter: &str,
) -> Result<String, String> {
    let cwd = resolve_cwd(ctx);
    let cwd_str = cwd.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    {
        open_macos_terminal(terminal_adapter, &cwd_str)?;
    }

    #[cfg(target_os = "windows")]
    {
        if terminal_adapter == "PowerShell" {
            let cd = format!("Set-Location {}", powershell_single_quoted(&cwd_str));
            Command::new("powershell.exe")
                .args(["-NoExit", "-Command", &cd])
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            Command::new("wt.exe")
                .args(["-d", &cwd_str])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        match terminal_adapter {
            "Alacritty" => {
                Command::new("alacritty")
                    .args(["--working-directory", &cwd_str])
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            _ => {
                Command::new("gnome-terminal")
                    .arg(format!("--working-directory={cwd_str}"))
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(cwd_str)
}

#[cfg(target_os = "macos")]
fn open_macos_terminal(terminal_adapter: &str, cwd: &str) -> Result<(), String> {
    match terminal_adapter {
        "iTerm2" => {
            // Path is passed as an AppleScript argument, then quoted form escapes it.
            Command::new("osascript")
                .args([
                    "-e",
                    "on run argv",
                    "-e",
                    "tell application \"iTerm\"",
                    "-e",
                    "activate",
                    "-e",
                    "create window with default profile",
                    "-e",
                    "tell current session of current window to write text \"cd \" & quoted form of (item 1 of argv)",
                    "-e",
                    "end tell",
                    "-e",
                    "end run",
                    cwd,
                ])
                .status()
                .map_err(|e| e.to_string())?;
        }
        "Warp" | "Alacritty" => {
            let app = if terminal_adapter == "Warp" { "Warp" } else { "Alacritty" };
            Command::new("open")
                .args(["-a", app, cwd])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        _ => {
            Command::new("osascript")
                .args([
                    "-e",
                    "on run argv",
                    "-e",
                    "tell application \"Terminal\"",
                    "-e",
                    "activate",
                    "-e",
                    "do script \"cd \" & quoted form of (item 1 of argv)",
                    "-e",
                    "end tell",
                    "-e",
                    "end run",
                    cwd,
                ])
                .status()
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn execute_launch_app(exec_path: &str, ctx: &ContextSnapshot) -> Result<(), String> {
    let targets = ctx.effective_paths();

    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        cmd.arg("-a").arg(exec_path);
        if !targets.is_empty() {
            cmd.args(&targets);
        }
        cmd.spawn().map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let mut cmd = Command::new(exec_path);
        cmd.args(&targets);
        cmd.spawn().map_err(|e| e.to_string())?;
        Ok(())
    }
}
