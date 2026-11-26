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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdeKind {
    Cursor,
    VsCode,
    Generic,
}

pub fn classify_ide_kind(exec_path: &str, preset_key: Option<&str>) -> IdeKind {
    if let Some(key) = preset_key {
        match key.trim().to_ascii_lowercase().as_str() {
            "cursor" => return IdeKind::Cursor,
            "vscode" => return IdeKind::VsCode,
            _ => {}
        }
    }

    let lower = exec_path.trim().to_ascii_lowercase();
    let file_name = lower.rsplit(['/', '\\']).next().unwrap_or("");
    let file_stem = file_name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(file_name);

    if file_name == "cursor.app"
        || file_name == "cursor.exe"
        || file_stem == "cursor"
        || lower.contains("cursor.app")
    {
        return IdeKind::Cursor;
    }

    if file_name == "visual studio code.app"
        || file_name == "code.exe"
        || file_stem == "code"
        || lower.contains("visual studio code")
    {
        return IdeKind::VsCode;
    }

    IdeKind::Generic
}

pub fn build_ide_invocations(
    kind: IdeKind,
    dirs: &[String],
    files: &[String],
) -> Vec<Vec<String>> {
    match kind {
        IdeKind::Cursor => {
            let mut invocations = Vec::new();
            if !dirs.is_empty() {
                let mut args = vec!["--classic".to_string(), "--new-window".to_string()];
                args.extend(dirs.iter().cloned());
                invocations.push(args);
            }
            if !files.is_empty() {
                let mut args = vec!["--classic".to_string()];
                args.extend(files.iter().cloned());
                invocations.push(args);
            }
            if invocations.is_empty() {
                invocations.push(vec!["--classic".to_string()]);
            }
            invocations
        }
        IdeKind::VsCode => {
            let mut invocations = Vec::new();
            if !dirs.is_empty() {
                let mut args = vec!["--new-window".to_string()];
                args.extend(dirs.iter().cloned());
                invocations.push(args);
            }
            if !files.is_empty() {
                invocations.push(files.to_vec());
            }
            if invocations.is_empty() {
                invocations.push(Vec::new());
            }
            invocations
        }
        IdeKind::Generic => Vec::new(),
    }
}

#[cfg(target_os = "macos")]
fn resolve_macos_ide_binary(exec_path: &str, kind: IdeKind) -> Option<PathBuf> {
    let app_path = Path::new(exec_path);
    let candidates: [&str; 2] = match kind {
        IdeKind::Cursor => [
            "Contents/MacOS/Cursor",
            "Contents/Resources/app/bin/cursor",
        ],
        IdeKind::VsCode => [
            "Contents/Resources/app/bin/code",
            "Contents/MacOS/Electron",
        ],
        IdeKind::Generic => return None,
    };

    for rel in candidates {
        let candidate = app_path.join(rel);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn spawn_detached(program: &Path, args: &[String]) -> Result<(), String> {
    use std::process::Stdio;
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn execute_launch_app(
    exec_path: &str,
    preset_key: Option<&str>,
    ctx: &ContextSnapshot,
) -> Result<(), String> {
    let targets = ctx.effective_paths();
    let kind = classify_ide_kind(exec_path, preset_key);

    if kind != IdeKind::Generic {
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for target in &targets {
            if Path::new(target).is_dir() {
                dirs.push(target.clone());
            } else {
                files.push(target.clone());
            }
        }
        let invocations = build_ide_invocations(kind, &dirs, &files);

        #[cfg(target_os = "macos")]
        {
            if let Some(bin) = resolve_macos_ide_binary(exec_path, kind) {
                for args in &invocations {
                    spawn_detached(&bin, args)?;
                }
                return Ok(());
            }
            for args in &invocations {
                let mut cmd = Command::new("open");
                cmd.arg("-a").arg(exec_path);
                if !args.is_empty() {
                    cmd.arg("--args").args(args);
                }
                cmd.spawn().map_err(|e| e.to_string())?;
            }
            return Ok(());
        }

        #[cfg(not(target_os = "macos"))]
        {
            let bin = Path::new(exec_path);
            for args in &invocations {
                spawn_detached(bin, args)?;
            }
            return Ok(());
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_cursor_and_vscode_by_key_or_path() {
        assert_eq!(
            classify_ide_kind("/Applications/Cursor.app", Some("cursor")),
            IdeKind::Cursor
        );
        assert_eq!(
            classify_ide_kind("/Users/me/Applications/Cursor.app", None),
            IdeKind::Cursor
        );
        assert_eq!(
            classify_ide_kind(r"C:\Users\me\AppData\Local\Programs\cursor\Cursor.exe", None),
            IdeKind::Cursor
        );
        assert_eq!(
            classify_ide_kind("/Applications/Visual Studio Code.app", Some("vscode")),
            IdeKind::VsCode
        );
        assert_eq!(
            classify_ide_kind(r"C:\Program Files\Microsoft VS Code\Code.exe", None),
            IdeKind::VsCode
        );
        assert_eq!(
            classify_ide_kind("/Applications/Sublime Text.app", Some("sublime")),
            IdeKind::Generic
        );
    }

    #[test]
    fn cursor_opens_folder_as_project_and_file_directly_in_classic_ide_mode() {
        let dir_only = build_ide_invocations(IdeKind::Cursor, &["/tmp/project".into()], &[]);
        assert_eq!(
            dir_only,
            vec![vec![
                "--classic".to_string(),
                "--new-window".to_string(),
                "/tmp/project".to_string()
            ]]
        );

        let file_only = build_ide_invocations(IdeKind::Cursor, &[], &["/tmp/a.ts".into()]);
        assert_eq!(
            file_only,
            vec![vec!["--classic".to_string(), "/tmp/a.ts".to_string()]]
        );

        let empty = build_ide_invocations(IdeKind::Cursor, &[], &[]);
        assert_eq!(empty, vec![vec!["--classic".to_string()]]);
    }

    #[test]
    fn vscode_opens_folder_as_project_and_file_directly() {
        let dir_only = build_ide_invocations(IdeKind::VsCode, &["/tmp/project".into()], &[]);
        assert_eq!(
            dir_only,
            vec![vec![
                "--new-window".to_string(),
                "/tmp/project".to_string()
            ]]
        );

        let file_only = build_ide_invocations(IdeKind::VsCode, &[], &["/tmp/a.ts".into()]);
        assert_eq!(file_only, vec![vec!["/tmp/a.ts".to_string()]]);
    }
}
