//! On macOS, the test binary (and therefore the library it links) must not
//! contain rdev character-translation or TIS/HIToolbox symbols.

#[cfg(target_os = "macos")]
fn forbidden_needles() -> Vec<Vec<u8>> {
    vec![
        [b"string_from".as_slice(), b"_code"].concat(),
        [b"TISCopyCurrentKeyboard".as_slice(), b"InputSource"].concat(),
        [b"TISGetInputSource".as_slice(), b"Property"].concat(),
        [b"TSMGetInputSource".as_slice(), b"Property"].concat(),
        [b"create_string_for".as_slice(), b"_key"].concat(),
        [b"4rd".as_slice(), b"ev"].concat(),
    ]
}

#[cfg(target_os = "macos")]
#[test]
fn macos_test_binary_does_not_contain_rdev_tis_symbols() {
    let exe = std::env::current_exe().expect("current test binary");
    let bytes = std::fs::read(&exe).unwrap_or_else(|err| panic!("read {exe:?}: {err}"));
    let mut hits = Vec::new();
    for needle in forbidden_needles() {
        if find_bytes(&bytes, &needle) {
            hits.push(String::from_utf8_lossy(&needle).into_owned());
        }
    }
    assert!(
        hits.is_empty(),
        "macOS binary {:?} unexpectedly contains {hits:?}",
        exe.file_name().unwrap_or_default()
    );
}

#[cfg(target_os = "macos")]
fn find_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[cfg(not(target_os = "macos"))]
#[test]
fn macos_symbol_guard_is_a_no_op_off_mac() {
    // Keeps the integration test module compiling on Linux CI.
}
