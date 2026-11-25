use std::path::{Path, PathBuf};

/// Working directory for "open in terminal": a selected folder, or the parent of a selected file.
pub fn working_directory_from_paths<'a, I>(paths: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = &'a str>,
{
    let first = paths.into_iter().next()?;
    if first.is_empty() {
        return None;
    }
    let path = PathBuf::from(first);
    if path.is_dir() {
        return Some(path);
    }
    // Prefer the explicit parent even when the path does not exist on this machine
    // (unit tests, or a path captured on another host).
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            return Some(parent.to_path_buf());
        }
    }
    Some(path)
}

/// Split newline-joined captured paths and drop empty lines.
pub fn split_captured_paths(joined: &str) -> Vec<String> {
    joined
        .lines()
        .map(|line| line.trim().trim_end_matches('/').to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

pub fn join_paths_for_clipboard(paths: &[String]) -> String {
    paths.join("\n")
}

/// Escape a path for a PowerShell single-quoted literal. No interpolation.
pub fn powershell_single_quoted(path: &str) -> String {
    format!("'{}'", path.replace('\'', "''"))
}

pub fn is_plausible_path(path: &str) -> bool {
    let p = Path::new(path);
    p.is_absolute() || path.starts_with("~/") || p.components().count() > 0 && !path.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_is_used_as_working_directory() {
        let dir = std::env::temp_dir();
        let got = working_directory_from_paths([dir.to_str().unwrap()]).unwrap();
        assert_eq!(got, dir);
    }

    #[test]
    fn file_uses_parent_directory() {
        // Do not require the file to exist: parent must still be taken.
        let got = working_directory_from_paths([r"/Users/test/My Project/notes.md"]).unwrap();
        assert_eq!(got, PathBuf::from("/Users/test/My Project"));
    }

    #[test]
    fn spaces_and_quotes_are_preserved() {
        let raw = r#"/Users/test/My "Project"/draft.txt"#;
        let got = working_directory_from_paths([raw]).unwrap();
        assert_eq!(got, PathBuf::from(r#"/Users/test/My "Project""#));
        assert_eq!(
            powershell_single_quoted(raw),
            r#"'/Users/test/My "Project"/draft.txt'"#
        );
        assert_eq!(
            powershell_single_quoted("O'Brien/folder"),
            "'O''Brien/folder'"
        );
    }

    #[test]
    fn split_and_join_roundtrip() {
        let raw = "/Users/test/My Folder/a.txt\n/Users/test/My Folder/b.txt\n\n";
        let paths = split_captured_paths(raw);
        assert_eq!(paths.len(), 2);
        assert!(paths[0].contains(' '));
        assert_eq!(join_paths_for_clipboard(&paths), paths.join("\n"));
    }
}
