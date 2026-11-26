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

pub fn build_macos_open_args(
    kind: IdeKind,
    dirs: &[String],
    files: &[String],
) -> Vec<Vec<String>> {
    let mut invocations = Vec::new();
    match kind {
        IdeKind::VsCode => {
            if !dirs.is_empty() {
                let mut args = dirs.to_vec();
                args.push("--args".to_string());
                args.push("--new-window".to_string());
                invocations.push(args);
            }
            if !files.is_empty() {
                invocations.push(files.to_vec());
            }
            if invocations.is_empty() {
                invocations.push(Vec::new());
            }
        }
        IdeKind::Cursor => {
            if !dirs.is_empty() {
                let mut args = dirs.to_vec();
                args.push("--args".to_string());
                args.push("--classic".to_string());
                args.push("--new-window".to_string());
                invocations.push(args);
            }
            if !files.is_empty() {
                let mut args = files.to_vec();
                args.push("--args".to_string());
                args.push("--classic".to_string());
                invocations.push(args);
            }
            if invocations.is_empty() {
                invocations.push(vec!["--args".to_string(), "--classic".to_string()]);
            }
        }
        IdeKind::Generic => {
            let mut combined = dirs.to_vec();
            combined.extend(files.iter().cloned());
            invocations.push(combined);
        }
    }
    invocations
}

#[cfg(target_os = "macos")]
fn resolve_macos_app_bundle(exec_path: &str, kind: IdeKind) -> Option<PathBuf> {
    let trimmed = exec_path.trim();
    if !trimmed.is_empty() {
        let configured = PathBuf::from(trimmed);
        if configured.exists() {
            return Some(configured);
        }
    }

    let bundle_names: &[&str] = match kind {
        IdeKind::VsCode => &["Visual Studio Code.app", "Visual Studio Code - Insiders.app"],
        IdeKind::Cursor => &["Cursor.app"],
        IdeKind::Generic => &[],
    };

    for name in bundle_names {
        let system_app = Path::new("/Applications").join(name);
        if system_app.exists() {
            return Some(system_app);
        }
        if let Some(home) = dirs::home_dir() {
            let user_app = home.join("Applications").join(name);
            if user_app.exists() {
                return Some(user_app);
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn macos_bundle_ids(kind: IdeKind) -> &'static [&'static str] {
    match kind {
        IdeKind::VsCode => &["com.microsoft.VSCode", "com.microsoft.VSCodeInsiders"],
        IdeKind::Cursor => &["com.todesktop.230313mzl4w4u92"],
        IdeKind::Generic => &[],
    }
}

#[cfg(target_os = "macos")]
fn run_macos_open(bundle_path: Option<&Path>, bundle_ids: &[&str], args: &[String]) -> Result<(), String> {
    use std::process::Stdio;

    if let Some(path) = bundle_path {
        let output = Command::new("/usr/bin/open")
            .arg("-a")
            .arg(path)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            return Ok(());
        }
    }

    let mut last_err = String::new();
    for bundle_id in bundle_ids {
        let output = Command::new("/usr/bin/open")
            .arg("-b")
            .arg(bundle_id)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !stderr.is_empty() {
            last_err = stderr;
        }
    }

    if last_err.is_empty() {
        Err("无法通过 macOS LaunchServices 启动应用".to_string())
    } else {
        Err(last_err)
    }
}

#[cfg(target_os = "macos")]
fn resolve_macos_cursor_binary(exec_path: &str, bundle: Option<&Path>) -> Option<PathBuf> {
    let base = bundle.unwrap_or_else(|| Path::new(exec_path));
    for rel in [
        "Contents/MacOS/Cursor",
        "Contents/Resources/app/bin/cursor",
    ] {
        let candidate = base.join(rel);
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

        #[cfg(target_os = "macos")]
        {
            let bundle = resolve_macos_app_bundle(exec_path, kind);
            if kind == IdeKind::Cursor {
                if let Some(bin) = resolve_macos_cursor_binary(exec_path, bundle.as_deref()) {
                    let invocations = build_ide_invocations(kind, &dirs, &files);
                    for args in &invocations {
                        spawn_detached(&bin, args)?;
                    }
                    return Ok(());
                }
            }

            let open_invocations = build_macos_open_args(kind, &dirs, &files);
            let fallback_path = if bundle.is_none() && !exec_path.trim().is_empty() {
                Some(PathBuf::from(exec_path))
            } else {
                None
            };
            let target_bundle = bundle.as_deref().or(fallback_path.as_deref());
            for args in &open_invocations {
                run_macos_open(target_bundle, macos_bundle_ids(kind), args)?;
            }
            return Ok(());
        }

        #[cfg(not(target_os = "macos"))]
        {
            let invocations = build_ide_invocations(kind, &dirs, &files);
            let bin = Path::new(exec_path);
            for args in &invocations {
                spawn_detached(bin, args)?;
            }
            return Ok(());
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("/usr/bin/open");
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

    #[test]
    fn macos_open_places_targets_before_args_for_vscode() {
        let dir_only = build_macos_open_args(IdeKind::VsCode, &["/tmp/project".into()], &[]);
        assert_eq!(
            dir_only,
            vec![vec![
                "/tmp/project".to_string(),
                "--args".to_string(),
                "--new-window".to_string(),
            ]]
        );

        let file_only = build_macos_open_args(IdeKind::VsCode, &[], &["/tmp/a.ts".into()]);
        assert_eq!(file_only, vec![vec!["/tmp/a.ts".to_string()]]);
    }
}
