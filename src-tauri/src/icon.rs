use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
pub fn extract_app_icon_png_base64(app_path: &Path, cache_dest: &Path) -> Result<String, String> {
    let plist_path = app_path.join("Contents/Info.plist");
    let value = plist::Value::from_file(&plist_path).map_err(|e| e.to_string())?;
    let icon_name = value
        .as_dictionary()
        .and_then(|d| d.get("CFBundleIconFile"))
        .and_then(|v| v.as_string())
        .ok_or_else(|| "CFBundleIconFile not found".to_string())?;

    let mut icns_file = app_path.join("Contents/Resources").join(icon_name);
    if icns_file.extension().is_none() {
        icns_file.set_extension("icns");
    }

    let file = fs::File::open(&icns_file).map_err(|e| e.to_string())?;
    let icon_family = icns::IconFamily::read(file).map_err(|e| e.to_string())?;

    let icon_type = icon_family
        .available_icons()
        .into_iter()
        .max_by_key(|t| t.pixel_width())
        .ok_or_else(|| "No icon frames in icns".to_string())?;

    let decoded = icon_family.get_icon_with_type(icon_type).map_err(|e| e.to_string())?;
    let rgba = image::RgbaImage::from_raw(decoded.width(), decoded.height(), decoded.data().to_vec())
        .ok_or_else(|| "Invalid RGBA buffer".to_string())?;

    let resized = image::imageops::resize(&rgba, 128, 128, image::imageops::FilterType::Lanczos3);
    if let Some(parent) = cache_dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    resized.save(cache_dest).map_err(|e| e.to_string())?;

    let bytes = fs::read(cache_dest).map_err(|e| e.to_string())?;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

#[cfg(target_os = "windows")]
pub fn extract_app_icon_png_base64(app_path: &Path, cache_dest: &Path) -> Result<String, String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};

    let wide: Vec<u16> = app_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut info = SHFILEINFOW::default();
    unsafe {
        let res = SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            Default::default(),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if res == 0 || info.hIcon.is_invalid() {
            return Err("SHGetFileInfoW failed".into());
        }
    }
    let _ = cache_dest;
    Ok(String::new())
}

#[cfg(target_os = "linux")]
pub fn extract_app_icon_png_base64(desktop_or_bin: &Path, cache_dest: &Path) -> Result<String, String> {
    let content = fs::read_to_string(desktop_or_bin).map_err(|e| e.to_string())?;
    let icon_key = content
        .lines()
        .find_map(|line| line.strip_prefix("Icon="))
        .ok_or_else(|| "Icon= not found in .desktop".to_string())?
        .trim();

    let candidate_dirs = [
        PathBuf::from("/usr/share/icons/hicolor/128x128/apps"),
        PathBuf::from("/usr/share/icons/hicolor/256x256/apps"),
        PathBuf::from("/usr/share/pixmaps"),
    ];

    for dir in candidate_dirs {
        for ext in ["png", "svg"] {
            let candidate = dir.join(format!("{}.{}", icon_key, ext));
            if candidate.exists() {
                if let Some(parent) = cache_dest.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::copy(&candidate, cache_dest);
                let bytes = fs::read(&candidate).map_err(|e| e.to_string())?;
                let mime = if ext == "svg" { "image/svg+xml" } else { "image/png" };
                return Ok(format!("data:{};base64,{}", mime, STANDARD.encode(bytes)));
            }
        }
    }

    Err("Icon asset not located".into())
}
