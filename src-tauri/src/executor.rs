use crate::context::ContextSnapshot;
use std::path::Path;
use std::process::Command;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;

pub fn execute_copy_path(app: &AppHandle, ctx: &ContextSnapshot) -> Result<String, String> {
    let paths = ctx.effective_paths();
    if paths.is_empty() {
        return Err("No active folder or selected file".into());
    }
    let joined = paths.join("\n");
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

pub fn execute_open_in_terminal(
    ctx: &ContextSnapshot,
    terminal_adapter: &str,
) -> Result<String, String> {
    let cwd = ctx
        .resolve_working_directory()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| Path::new("/").to_path_buf()));
    let cwd_str = cwd.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    {
        let app_name = match terminal_adapter {
            "iTerm2" => "iTerm",
            "Warp" => "Warp",
            "Alacritty" => "Alacritty",
            _ => "Terminal",
        };
        Command::new("open")
            .arg("-a")
            .arg(app_name)
            .arg(&cwd_str)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        if terminal_adapter == "PowerShell" {
            Command::new("powershell.exe")
                .args(["-NoExit", "-Command", &format!("Set-Location '{}'", cwd_str)])
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
        let bin = match terminal_adapter {
            "Alacritty" => "alacritty",
            _ => "gnome-terminal",
        };
        Command::new(bin)
            .arg(format!("--working-directory={}", cwd_str))
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(cwd_str)
}

pub fn execute_launch_app(exec_path: &str, ctx: &ContextSnapshot) -> Result<(), String> {
    let targets = ctx.effective_paths();

    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        cmd.arg("-a").arg(exec_path);
        for target in targets {
            cmd.arg(target);
        }
        cmd.spawn().map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let mut cmd = Command::new(exec_path);
        for target in targets {
            cmd.arg(target);
        }
        cmd.spawn().map_err(|e| e.to_string())?;
        Ok(())
    }
}
