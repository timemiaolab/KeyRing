//! On macOS, the built library must not contain the previous input crate's
//! character-translation or Text Input Source symbols.
//!
//! This test inspects `libkeyring_lib.dylib` (or the staticlib/rlib), not
//! the test executable. Scanning the test binary is a false positive: LLVM
//! may place the search-needle fragments next to each other in rodata.

#[cfg(target_os = "macos")]
fn join(a: &str, b: &str) -> Vec<u8> {
    let mut out = Vec::from(a.as_bytes());
    out.extend_from_slice(b.as_bytes());
    out
}

// Halves live in separate functions so they are not adjacent statics.
#[cfg(target_os = "macos")]
fn part_a1() -> String {
    format!("{}{}", "string", "_from")
}
#[cfg(target_os = "macos")]
fn part_a2() -> String {
    format!("{}{}", "_", "code")
}
#[cfg(target_os = "macos")]
fn part_b1() -> String {
    format!("{}{}", "TISCopy", "Current")
}
#[cfg(target_os = "macos")]
fn part_b2() -> String {
    format!("{}{}", "Keyboard", "InputSource")
}
#[cfg(target_os = "macos")]
fn part_c1() -> String {
    format!("{}{}", "TISGet", "Input")
}
#[cfg(target_os = "macos")]
fn part_c2() -> String {
    format!("{}{}", "Source", "Property")
}
#[cfg(target_os = "macos")]
fn part_d1() -> String {
    format!("{}{}", "TSMGet", "Input")
}
#[cfg(target_os = "macos")]
fn part_d2() -> String {
    format!("{}{}", "Source", "Property")
}
#[cfg(target_os = "macos")]
fn part_e1() -> String {
    format!("{}{}", "create_", "string")
}
#[cfg(target_os = "macos")]
fn part_e2() -> String {
    format!("{}{}", "_for", "_key")
}
#[cfg(target_os = "macos")]
fn part_f1() -> String {
    format!("{}{}", "4", "r")
}
#[cfg(target_os = "macos")]
fn part_f2() -> String {
    format!("{}{}", "d", "ev")
}

#[cfg(target_os = "macos")]
fn forbidden_needles() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("s_f_c", join(&part_a1(), &part_a2())),
        ("tis_copy", join(&part_b1(), &part_b2())),
        ("tis_get", join(&part_c1(), &part_c2())),
        ("tsm_get", join(&part_d1(), &part_d2())),
        ("create_str", join(&part_e1(), &part_e2())),
        ("crate_tag", join(&part_f1(), &part_f2())),
    ]
}

#[cfg(target_os = "macos")]
fn find_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[cfg(target_os = "macos")]
fn library_paths() -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;

    let mut roots = Vec::new();
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        roots.push(PathBuf::from(dir));
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
    if let Ok(exe) = std::env::current_exe() {
        let mut cursor = exe.parent();
        for _ in 0..8 {
            if let Some(dir) = cursor {
                if dir.file_name().and_then(|n| n.to_str()) == Some("target") {
                    roots.push(dir.to_path_buf());
                    break;
                }
                if dir.join("libkeyring_lib.dylib").is_file() {
                    roots.push(dir.to_path_buf());
                }
                cursor = dir.parent();
            } else {
                break;
            }
        }
    }

    let names = [
        "libkeyring_lib.dylib",
        "libkeyring_lib.a",
        "libkeyring_lib.rlib",
    ];
    let mut found = Vec::new();
    for root in roots {
        for name in names {
            let direct = root.join("debug").join(name);
            if direct.is_file() {
                found.push(direct);
            }
            if let Ok(entries) = std::fs::read_dir(&root) {
                for entry in entries.flatten() {
                    let p = entry.path().join("debug").join(name);
                    if p.is_file() {
                        found.push(p);
                    }
                }
            }
            let alt = root.join(name);
            if alt.is_file() {
                found.push(alt);
            }
        }
    }
    found.sort();
    found.dedup();
    found.retain(|p| p.is_file());
    found
}

#[cfg(target_os = "macos")]
#[test]
fn macos_library_does_not_contain_input_source_translation_symbols() {
    let paths = library_paths();
    assert!(
        !paths.is_empty(),
        "could not locate libkeyring_lib.dylib/.a/.rlib under the cargo target dir"
    );
    let needles = forbidden_needles();
    let mut failures = Vec::new();
    for path in &paths {
        let bytes = std::fs::read(path).unwrap_or_else(|err| panic!("read {path:?}: {err}"));
        let mut hits = Vec::new();
        for (label, needle) in &needles {
            if find_bytes(&bytes, needle) {
                hits.push((*label).to_string());
            }
        }
        if !hits.is_empty() {
            failures.push(format!("{}: {hits:?}", path.display()));
        }
    }
    assert!(
        failures.is_empty(),
        "macOS library unexpectedly contains forbidden input-source translation symbols: {failures:?}"
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn macos_symbol_guard_is_a_no_op_off_mac() {
    // Keeps the integration test module compiling on Linux CI.
}
